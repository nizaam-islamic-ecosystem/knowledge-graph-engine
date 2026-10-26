//! Phase 7 physical persistence test for a realistic batch of word entities.
//!
//! This suite deliberately exercises the real Phase 1 `Entity` model, the real
//! Phase 7 Change Set transaction, the file-backed repository, and the actual
//! canonical state document on disk. The entity payload is serialized only in
//! this test because Phase 7 intentionally keeps repository documents opaque;
//! the repository must not become coupled to a particular semantic serializer.
//!
//! The physical layout asserted here is the implementation's current layout:
//!
//!   ~/.nizaam/kg/current-state.json                  (default KG)
//!   ~/.nizaam/test-runs/kg-phase7-*/kg/current-state.json (isolated test runs)
//!
//! A single canonical state document contains many semantic records. The test
//! therefore verifies 100 records inside that document rather than expecting
//! 100 separate physical files.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use kg::entity::{Alias, Entity, Name};
use kg::identity::EntityId;
use kg::storage::{
    ChangeSetTransaction, Document, FileDocumentStore, FileRecoveryJournal, FileRepository,
    RecoveryCommit, RecoveryJournal, Repository, RepositoryError, RepositoryRecord, Snapshot,
};
use kg::versioning::{Change, ChangeKind, ChangeSet, ChangeTarget, KgVersion};
use nizaam_knowledge_graph::{self as kg, DocumentStore};
use serde_json::{Value, json};

const WORD_COUNT: usize = 100;

fn test_root(prefix: &str) -> PathBuf {
    let nizaam_root = FileDocumentStore::default_root()
        .expect("Nizaam home directory should resolve for integration tests");
    let root = nizaam_root.join("test-runs").join(format!(
        "kg-phase7-{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("KG test-run root should be created under .nizaam");
    root
}

fn word_entity(index: usize) -> Entity {
    let id = EntityId::new(format!("word-{index:03}"))
        .expect("deterministic word entity identity should be valid");
    Entity::new(
        id,
        Name::new(format!("word-{index:03}"), "en"),
        vec![Alias::new(format!("word {index:03}"), "en")],
    )
}

fn word_target(index: usize) -> ChangeTarget {
    ChangeTarget::Entity(
        EntityId::new(format!("word-{index:03}"))
            .expect("deterministic word entity identity should be valid"),
    )
}

// Stable, clearly attributable IDs for the explicit real-~/.nizaam test. These
// are intentionally different from generic fixture IDs so the test never
// overwrites unrelated user entities that happen to be named `word-042`.
fn real_word_entity(index: usize) -> Entity {
    Entity::new(
        EntityId::new(format!("phase7-storage-word-{index:03}"))
            .expect("real-storage verification entity identity should be valid"),
        Name::new(format!("word-{index:03}"), "en"),
        vec![Alias::new(format!("word {index:03}"), "en")],
    )
}

fn real_word_target(index: usize) -> ChangeTarget {
    ChangeTarget::Entity(
        EntityId::new(format!("phase7-storage-word-{index:03}"))
            .expect("real-storage verification identity should be valid"),
    )
}

fn stage_all_real_words(repository: &mut FileRepository) -> kg::versioning::KgVersion {
    let mut transaction = ChangeSetTransaction::begin(repository)
        .expect("real-storage word batch transaction should begin");
    for index in 0..WORD_COUNT {
        let entity = real_word_entity(index);
        let target = real_word_target(index);
        transaction
            .stage(
                Change::new(ChangeKind::Created, target.clone()),
                Some(RepositoryRecord::new(target, encode_word_entity(&entity))),
            )
            .expect("real-storage word entity should stage successfully");
    }
    transaction
        .commit()
        .expect("real-storage word batch should commit atomically")
}

fn verify_real_words(repository: &FileRepository) {
    for index in 0..WORD_COUNT {
        let record = repository
            .read(&real_word_target(index))
            .expect("real-storage lookup should succeed")
            .expect("real-storage word entity must survive restart");
        let value = decode_word_document(record.document());
        assert_eq!(
            value["id"],
            Value::from(format!("phase7-storage-word-{index:03}"))
        );
        assert_eq!(
            value["name"]["value"],
            Value::from(format!("word-{index:03}"))
        );
        assert_eq!(value["name"]["language"], Value::from("en"));
        assert_eq!(value["aliases"].as_array().unwrap().len(), 1);
        assert_eq!(
            value["aliases"][0]["value"],
            Value::from(format!("word {index:03}"))
        );
        assert_eq!(value["aliases"][0]["language"], Value::from("en"));
    }
}

fn assert_physical_state_contains_real_words(root: &Path) -> Value {
    let state_path = root.join("kg/current-state.json");
    assert!(
        state_path.is_file(),
        "canonical state must exist at {}",
        state_path.display()
    );
    let bytes = std::fs::read(&state_path).expect("canonical state must be readable");
    assert!(!bytes.is_empty(), "canonical state must not be empty");
    let state: Value = serde_json::from_slice(&bytes).expect("canonical state must be valid JSON");
    assert_eq!(state["schema_version"], Value::from(1u64));
    let records = state["records"]
        .as_array()
        .expect("canonical state must contain a records array");
    let entity_records = records
        .iter()
        .filter(|record| record["kind"] == "entity")
        .filter_map(|record| record["id"].as_str().map(|id| (id, record)))
        .collect::<std::collections::BTreeMap<_, _>>();
    for index in 0..WORD_COUNT {
        let id = format!("phase7-storage-word-{index:03}");
        let record = entity_records
            .get(id.as_str())
            .unwrap_or_else(|| panic!("canonical file must contain {id}"));
        let stored_bytes = record["bytes"]
            .as_array()
            .expect("canonical entity record must contain its stored document bytes")
            .iter()
            .map(|byte| {
                u8::try_from(
                    byte.as_u64()
                        .expect("persisted document byte must be an unsigned integer"),
                )
                .expect("persisted document byte must be in 0..=255")
            })
            .collect::<Vec<_>>();
        let payload: Value = serde_json::from_slice(&stored_bytes)
            .expect("physical canonical entity bytes must contain valid word JSON");
        assert_eq!(payload["id"], Value::from(id.as_str()));
        assert_eq!(
            payload["name"]["value"],
            Value::from(format!("word-{index:03}"))
        );
        assert_eq!(payload["name"]["language"], Value::from("en"));
        assert_eq!(
            payload["aliases"][0]["value"],
            Value::from(format!("word {index:03}"))
        );
        assert_eq!(payload["aliases"][0]["language"], Value::from("en"));
    }
    state
}

/// The Phase 7 repository intentionally stores opaque semantic bytes. This
/// test fixture provides a deterministic representation of the real `Entity`
/// values so that the test can verify semantic data survives physical storage
/// and restart without introducing a production serializer prematurely.
fn encode_word_entity(entity: &Entity) -> Document {
    let value = json!({
        "id": entity.id().as_str(),
        "name": {
            "value": entity.name().value(),
            "language": entity.name().language(),
        },
        "aliases": entity
            .aliases()
            .iter()
            .map(|alias| json!({
                "value": alias.value(),
                "language": alias.language(),
            }))
            .collect::<Vec<_>>(),
    });

    Document::new(serde_json::to_vec(&value).expect("word entity JSON should serialize"))
}

fn decode_word_document(document: &Document) -> Value {
    serde_json::from_slice(document.as_bytes()).expect("persisted word document must be JSON")
}

fn stage_all_words(repository: &mut FileRepository) -> kg::versioning::KgVersion {
    let mut transaction =
        ChangeSetTransaction::begin(repository).expect("word batch transaction should begin");

    for index in 0..WORD_COUNT {
        let entity = word_entity(index);
        let target = ChangeTarget::Entity(entity.id().clone());
        transaction
            .stage(
                Change::new(ChangeKind::Created, target),
                Some(RepositoryRecord::new(
                    word_target(index),
                    encode_word_entity(&entity),
                )),
            )
            .expect("word entity should stage successfully");
    }

    transaction
        .commit()
        .expect("100 word entities should commit atomically")
}

fn assert_physical_state_has_100_words(root: &Path) -> Value {
    let state_path = root.join("kg/current-state.json");
    assert!(
        state_path.is_file(),
        "canonical state must exist at {}",
        state_path.display()
    );

    let bytes = std::fs::read(&state_path).expect("canonical state must be readable");
    assert!(!bytes.is_empty(), "canonical state must not be empty");

    let state: Value = serde_json::from_slice(&bytes).expect("canonical state must be valid JSON");
    assert_eq!(state["schema_version"], Value::from(1u64));

    let records = state["records"]
        .as_array()
        .expect("canonical state must contain a records array");
    assert_eq!(records.len(), WORD_COUNT);

    let ids = records
        .iter()
        .map(|record| {
            assert_eq!(record["kind"], Value::from("entity"));
            record["id"]
                .as_str()
                .expect("record id must be a string")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    let expected = (0..WORD_COUNT)
        .map(|index| format!("word-{index:03}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, expected);

    state
}

#[test]
fn one_hundred_real_word_entities_are_persisted_to_the_nizaam_state_layout() {
    let root = test_root("batch-persist");
    let mut repository = FileRepository::open(&root).expect("repository should open");

    let version = stage_all_words(&mut repository);

    assert_eq!(version.sequence(), 1);
    assert_eq!(repository.current_state().unwrap().len(), WORD_COUNT);
    assert_eq!(repository.change_history().len(), 1);

    let state = assert_physical_state_has_100_words(&root);
    assert_eq!(state["history"].as_array().unwrap().len(), 1);

    // This run's canonical state remains under ~/.nizaam/test-runs so its
    // physical files can be inspected without touching the default KG state.
}

#[test]
fn real_nizaam_storage_persists_and_reloads_one_hundred_word_entities_without_deleting_them() {
    // This persistence test intentionally runs during a normal `cargo test`
    // invocation so it creates/verifies the real ~/.nizaam/kg/current-state.json.
    // It only adds namespaced records and never removes or rewrites unrelated data.
    let root = FileDocumentStore::default_root().expect("Nizaam home directory should resolve");
    let state_path = root.join("kg/current-state.json");
    eprintln!(
        "REAL NIZAAM STORAGE TEST: canonical records are being verified at {}",
        state_path.display()
    );

    let mut repository =
        FileRepository::open_default().expect("default Nizaam repository should open");
    let before = repository
        .current_state()
        .expect("existing canonical state should be readable");
    let before_version = repository
        .current_version()
        .expect("existing canonical version should be readable");

    let present = (0..WORD_COUNT)
        .filter(|index| {
            repository
                .read(&real_word_target(*index))
                .unwrap()
                .is_some()
        })
        .count();

    let expected_version = match present {
        0 => stage_all_real_words(&mut repository),
        WORD_COUNT => {
            // Re-running the test is idempotent: verify existing records rather
            // than writing another history entry or replacing any user records.
            verify_real_words(&repository);
            before_version.clone()
        }
        count => panic!(
            "found {count} of {WORD_COUNT} persistent verification records at {}; refusing to mutate a partial batch",
            state_path.display()
        ),
    };

    verify_real_words(&repository);
    assert_physical_state_contains_real_words(&root);
    assert!(state_path.is_file(), "canonical state must remain on disk");

    // All pre-existing canonical records must still be present with identical
    // contents. The test only ever adds its own namespaced IDs and never
    // removes the canonical file or the user's existing records.
    for record in &before {
        let persisted = repository
            .read(record.target())
            .expect("pre-existing record lookup should succeed")
            .expect("pre-existing canonical record must be preserved");
        assert_eq!(persisted.document(), record.document());
    }
    if present == 0 {
        assert_eq!(
            expected_version.sequence(),
            before_version.sequence() + 1,
            "one batch should create exactly one new KG version"
        );
    }

    // Simulate shutdown/restart against the same actual default `.nizaam` root.
    drop(repository);
    let reopened = FileRepository::open_default().expect("default Nizaam repository should reopen");
    assert_eq!(reopened.current_version().unwrap(), expected_version);
    verify_real_words(&reopened);
    for record in &before {
        let persisted = reopened
            .read(record.target())
            .expect("reopened pre-existing record lookup should succeed")
            .expect("pre-existing record must survive restart");
        assert_eq!(persisted.document(), record.document());
    }

    assert_physical_state_contains_real_words(&root);
    eprintln!(
        "REAL NIZAAM STORAGE TEST COMPLETE: 100 verification entities remain inside {}",
        state_path.display()
    );
}

#[test]
fn all_one_hundred_word_entities_survive_repository_restart_with_content_intact() {
    let root = test_root("restart");
    let version = {
        let mut repository = FileRepository::open(&root).expect("repository should open");
        stage_all_words(&mut repository)
    };

    let reopened = FileRepository::open(&root).expect("repository should reopen");
    assert_eq!(reopened.current_version(), Ok(version));
    assert_eq!(reopened.current_state().unwrap().len(), WORD_COUNT);

    for index in 0..WORD_COUNT {
        let target = word_target(index);
        let record = reopened
            .read(&target)
            .expect("repository read should succeed")
            .expect("word entity must exist after restart");
        let value = decode_word_document(record.document());

        assert_eq!(value["id"], Value::from(format!("word-{index:03}")));
        assert_eq!(value["name"]["language"], Value::from("en"));
        assert_eq!(
            value["name"]["value"],
            Value::from(format!("word-{index:03}"))
        );
        assert_eq!(value["aliases"].as_array().unwrap().len(), 1);
    }

    assert_physical_state_has_100_words(&root);
}

#[test]
fn updating_one_word_preserves_the_other_ninety_nine_records() {
    let root = test_root("single-update");
    let mut repository = FileRepository::open(&root).expect("repository should open");
    stage_all_words(&mut repository);

    let target = word_target(42);
    let updated = Entity::new(
        EntityId::new("word-042").unwrap(),
        Name::new("updated-word-042", "en"),
        vec![Alias::new("updated word 042", "en")],
    );

    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            Change::new(ChangeKind::Updated, target.clone()),
            Some(RepositoryRecord::new(
                target.clone(),
                encode_word_entity(&updated),
            )),
        )
        .unwrap();
    transaction.commit().unwrap();

    assert_eq!(repository.current_state().unwrap().len(), WORD_COUNT);

    for index in 0..WORD_COUNT {
        let value = decode_word_document(
            repository
                .read(&word_target(index))
                .unwrap()
                .expect("word must remain present")
                .document(),
        );

        if index == 42 {
            assert_eq!(value["name"]["value"], Value::from("updated-word-042"));
        } else {
            assert_eq!(
                value["name"]["value"],
                Value::from(format!("word-{index:03}"))
            );
        }
    }

    assert_physical_state_has_100_words(&root);
}

#[test]
fn rolling_back_a_staged_hundred_word_batch_writes_no_canonical_records() {
    let root = test_root("rollback");
    let mut repository = FileRepository::open(&root).expect("repository should open");

    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    for index in 0..WORD_COUNT {
        let entity = word_entity(index);
        transaction
            .stage(
                Change::new(ChangeKind::Created, word_target(index)),
                Some(RepositoryRecord::new(
                    word_target(index),
                    encode_word_entity(&entity),
                )),
            )
            .unwrap();
    }

    transaction.rollback().unwrap();
    assert_eq!(repository.current_state().unwrap().len(), 0);
    assert!(repository.current_version().unwrap().is_initial());

    let state = std::fs::read(root.join("kg/current-state.json")).unwrap();
    let value: Value = serde_json::from_slice(&state).unwrap();
    assert_eq!(value["records"].as_array().unwrap().len(), 0);
    assert_eq!(value["history"].as_array().unwrap().len(), 0);
}

#[test]
fn a_failed_hundred_word_batch_cannot_publish_a_partial_canonical_state() {
    let root = test_root("atomic-failure");
    let mut repository = FileRepository::open(&root).expect("repository should open");
    let before = repository.current_version().unwrap();

    let mut changes = kg::versioning::ChangeSet::new(Some(before.id().clone()));
    let mut records = Vec::with_capacity(WORD_COUNT);

    for index in 0..WORD_COUNT {
        changes.push(Change::new(ChangeKind::Created, word_target(index)));
        records.push(RepositoryRecord::new(
            word_target(index),
            encode_word_entity(&word_entity(index)),
        ));
    }

    // The Change Set does not contain this target. The repository must reject
    // the complete commit before replacing the canonical state document.
    records[WORD_COUNT - 1] = RepositoryRecord::new(
        ChangeTarget::Entity(EntityId::new("not-part-of-the-batch").unwrap()),
        Document::from_slice(b"invalid"),
    );

    let error = repository
        .commit_change_set(&changes, &records)
        .expect_err("target mismatch must reject the entire batch");
    assert!(matches!(error, RepositoryError::TargetMismatch { .. }));

    assert_eq!(repository.current_version().unwrap(), before);
    assert!(repository.current_state().unwrap().is_empty());

    let value: Value =
        serde_json::from_slice(&std::fs::read(root.join("kg/current-state.json")).unwrap())
            .unwrap();
    assert_eq!(value["records"].as_array().unwrap().len(), 0);
    assert_eq!(value["history"].as_array().unwrap().len(), 0);
}

#[test]
fn stale_writer_cannot_overwrite_a_committed_hundred_word_batch() {
    let root = test_root("stale-writer");
    let mut current = FileRepository::open(&root).unwrap();
    let mut stale = FileRepository::open(&root).unwrap();

    stage_all_words(&mut current);

    let mut stale_transaction = ChangeSetTransaction::begin(&mut stale).unwrap();
    let entity = word_entity(7);
    stale_transaction
        .stage(
            Change::new(ChangeKind::Created, word_target(7)),
            Some(RepositoryRecord::new(
                word_target(7),
                encode_word_entity(&entity),
            )),
        )
        .unwrap();

    // The stale repository instance reloads the current state during commit,
    // so it must observe the version created by the 100-word batch.
    let mut current_update = ChangeSetTransaction::begin(&mut current).unwrap();
    let updated = Entity::new(
        EntityId::new("word-007").unwrap(),
        Name::new("current-word-007", "en"),
        vec![Alias::new("current word 007", "en")],
    );
    current_update
        .stage(
            Change::new(ChangeKind::Updated, word_target(7)),
            Some(RepositoryRecord::new(
                word_target(7),
                encode_word_entity(&updated),
            )),
        )
        .unwrap();
    current_update.commit().unwrap();

    assert!(stale_transaction.commit().is_err());

    let reopened = FileRepository::open(&root).unwrap();
    assert_eq!(reopened.current_state().unwrap().len(), WORD_COUNT);
    let value = decode_word_document(
        reopened
            .read(&word_target(7))
            .unwrap()
            .expect("updated word must exist")
            .document(),
    );
    assert_eq!(value["name"]["value"], Value::from("current-word-007"));
}

#[test]
fn one_hundred_word_records_can_be_recovered_from_the_durable_journal() {
    let root = test_root("recovery");
    let base = KgVersion::initial();
    let mut change_set = ChangeSet::new(Some(base.id().clone()));
    let mut records = Vec::with_capacity(WORD_COUNT);

    for index in 0..WORD_COUNT {
        let target = word_target(index);
        change_set.push(Change::new(ChangeKind::Created, target.clone()));
        records.push(RepositoryRecord::new(
            target,
            encode_word_entity(&word_entity(index)),
        ));
    }

    let version = KgVersion::next(&base, change_set.id().clone());
    let commit = RecoveryCommit::new(base.id().clone(), version.clone(), change_set, records)
        .expect("100-word recovery commit should satisfy the recovery contract");

    let snapshot = Snapshot::new(base, Vec::new()).expect("initial snapshot should be valid");
    let mut journal = FileRecoveryJournal::open(&root).expect("recovery journal should open");
    journal.prepare(&commit).expect("prepare should be durable");
    journal
        .commit(commit.change_set().id())
        .expect("commit marker should be durable");

    let recovered = kg::storage::recover_from_snapshot(&snapshot, &journal)
        .expect("100-word journal should recover successfully");

    assert_eq!(recovered.version(), &version);
    assert_eq!(recovered.records().len(), WORD_COUNT);
    assert_eq!(recovered.replayed_change_sets().len(), 1);

    for index in 0..WORD_COUNT {
        let record = recovered
            .records()
            .iter()
            .find(|record| record.target() == &word_target(index))
            .expect("recovered word record must exist");
        let value = decode_word_document(record.document());
        assert_eq!(value["id"], Value::from(format!("word-{index:03}")));
    }

    assert!(root.join("recovery/journal.jsonl").is_file());
}

#[test]
fn corrupting_the_physical_hundred_word_state_is_detected_on_restart() {
    let root = test_root("corruption");
    {
        let mut repository = FileRepository::open(&root).unwrap();
        stage_all_words(&mut repository);
    }

    let store = FileDocumentStore::new(&root).unwrap();
    let state_id = kg::storage::DocumentId::new("kg/current-state.json").unwrap();
    store
        .replace(&state_id, Document::from_slice(b"not-json"))
        .unwrap();

    let error = FileRepository::open(&root).expect_err("corrupt state must not be accepted");
    assert!(matches!(error, RepositoryError::PersistenceCorruption(_)));
}
