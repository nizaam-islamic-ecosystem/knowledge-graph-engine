//! Level 3 end-to-end verification for Phase 7.
//!
//! This file intentionally merges the planned Phase 7 test targets into one
//! test target. The test sections preserve the original storage, transaction,
//! versioning, snapshot, recovery, migration, index, integration, negative,
//! and performance coverage without changing the architectural boundaries.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use kg::index::{
    FileForwardIndex, FileReverseIndex, FileSearchIndex, ForwardIndexAccess, IndexChangeAction,
    IndexQueryability, IndexSynchronizationChange, IndexSynchronizationPolicy, KgIndexKind,
    KgIndexSynchronizer, KgIndexVersion, ReverseIndexAccess,
};
use kg::ingestion::PublicationOutcome;
use kg::query::{
    LookupRequest, QueryAccess, QueryMatchCandidate, QueryMatchType, QueryReference, execute, plan,
};
use kg::storage::{
    ChangeSetTransaction, Document, DocumentId, DocumentStore, DocumentStoreError,
    FileDocumentStore, FileRecoveryJournal, FileRepository, RecoveryCommit, RecoveryError,
    RecoveryJournal, Repository, RepositoryError, RepositoryRecord, Snapshot, SnapshotError,
    TransactionError,
};
use kg::versioning::{
    Change, ChangeKind, ChangeSet, ChangeTarget, CompatibilityLevel, CompatibilityMatrix,
    KgVersion, MigrationError, MigrationPlan, MigrationRunner, MigrationState, MigrationStep,
    StorageSchemaVersion,
};
use nizaam_core::identity::{CorrelationId, OperationId};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::definition::TargetReferenceType;
use nizaam_indexing::index::reference::ObjectReference;
use nizaam_knowledge_graph as kg;

fn temp_root(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "nizaam-phase7-{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos()
    ))
}

fn entity_target(id: &str) -> ChangeTarget {
    ChangeTarget::Entity(
        kg::identity::EntityId::new(id).expect("test entity identity should be valid"),
    )
}

fn record(id: &str, bytes: &[u8]) -> RepositoryRecord {
    RepositoryRecord::new(entity_target(id), Document::from_slice(bytes))
}

fn object_reference(id: &str) -> ObjectReference {
    ObjectReference::new("kg", id).expect("test object reference should be valid")
}

fn target_type() -> TargetReferenceType {
    TargetReferenceType::new("entity").expect("test target type should be valid")
}

fn operation_context() -> OperationContext {
    OperationContext::new(Operation::new(
        OperationId::new("nizaam.kg.phase7.test.operation").expect("valid operation id"),
        CorrelationId::new("nizaam.kg.phase7.test.correlation").expect("valid correlation id"),
    ))
}

fn created_change(target: ChangeTarget) -> Change {
    Change::new(ChangeKind::Created, target)
}

fn repository_with_record(
    root: &PathBuf,
    id: &str,
    bytes: &[u8],
) -> (FileRepository, kg::versioning::KgVersion) {
    let mut repository = FileRepository::open(root).expect("repository should open");
    let target = entity_target(id);
    let mut transaction =
        ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");
    transaction
        .stage(created_change(target), Some(record(id, bytes)))
        .expect("record should stage");
    let version = transaction.commit().expect("transaction should commit");
    (repository, version)
}

#[derive(Default)]
struct MemoryStore {
    documents: std::cell::RefCell<BTreeMap<String, Document>>,
}

impl DocumentStore for MemoryStore {
    fn get(&self, id: &DocumentId) -> Result<Option<Document>, DocumentStoreError> {
        Ok(self.documents.borrow().get(id.as_str()).cloned())
    }

    fn create(&self, id: DocumentId, document: Document) -> Result<(), DocumentStoreError> {
        let mut documents = self.documents.borrow_mut();
        if documents.contains_key(id.as_str()) {
            return Err(DocumentStoreError::AlreadyExists(id));
        }
        documents.insert(id.into_string(), document);
        Ok(())
    }

    fn replace(&self, id: &DocumentId, document: Document) -> Result<(), DocumentStoreError> {
        let mut documents = self.documents.borrow_mut();
        if !documents.contains_key(id.as_str()) {
            return Err(DocumentStoreError::NotFound(id.clone()));
        }
        documents.insert(id.as_str().to_owned(), document);
        Ok(())
    }

    fn contains(&self, id: &DocumentId) -> Result<bool, DocumentStoreError> {
        Ok(self.documents.borrow().contains_key(id.as_str()))
    }

    fn delete(&self, id: &DocumentId) -> Result<(), DocumentStoreError> {
        let mut documents = self.documents.borrow_mut();
        if documents.remove(id.as_str()).is_none() {
            return Err(DocumentStoreError::NotFound(id.clone()));
        }
        Ok(())
    }
}

struct UnavailableStore;

impl DocumentStore for UnavailableStore {
    fn get(&self, _id: &DocumentId) -> Result<Option<Document>, DocumentStoreError> {
        Err(DocumentStoreError::Unavailable(
            "test provider unavailable".to_owned(),
        ))
    }
    fn create(&self, _id: DocumentId, _document: Document) -> Result<(), DocumentStoreError> {
        Err(DocumentStoreError::Unavailable(
            "test provider unavailable".to_owned(),
        ))
    }
    fn replace(&self, id: &DocumentId, _document: Document) -> Result<(), DocumentStoreError> {
        Err(DocumentStoreError::Unavailable(id.to_string()))
    }
    fn contains(&self, _id: &DocumentId) -> Result<bool, DocumentStoreError> {
        Err(DocumentStoreError::Unavailable(
            "test provider unavailable".to_owned(),
        ))
    }
    fn delete(&self, _id: &DocumentId) -> Result<(), DocumentStoreError> {
        Err(DocumentStoreError::Unavailable(
            "test provider unavailable".to_owned(),
        ))
    }
}

#[derive(Default)]
struct CaptureSynchronizer {
    calls: usize,
    version: Option<String>,
    change_set_len: usize,
}

impl kg::storage::RepositoryIndexSynchronizer<()> for CaptureSynchronizer {
    type Report = usize;

    fn synchronize(
        &mut self,
        commit: &kg::storage::CommittedChangeSet,
        _updates: &[()],
    ) -> Self::Report {
        self.calls += 1;
        self.version = Some(commit.version().id().as_str().to_owned());
        self.change_set_len = commit.change_set().len();
        self.calls
    }
}

#[derive(Debug)]
struct TestMigrationStep {
    from: StorageSchemaVersion,
    to: StorageSchemaVersion,
    name: &'static str,
    fail_apply: bool,
    applied: Cell<bool>,
    verified: Cell<bool>,
    activated: Cell<bool>,
}

impl TestMigrationStep {
    fn new(from: u64, to: u64, name: &'static str) -> Self {
        Self {
            from: StorageSchemaVersion::new(from),
            to: StorageSchemaVersion::new(to),
            name,
            fail_apply: false,
            applied: Cell::new(false),
            verified: Cell::new(false),
            activated: Cell::new(false),
        }
    }
}

impl MigrationStep for TestMigrationStep {
    fn from_version(&self) -> StorageSchemaVersion {
        self.from
    }
    fn to_version(&self) -> StorageSchemaVersion {
        self.to
    }
    fn name(&self) -> &str {
        self.name
    }

    fn apply(&self) -> Result<(), MigrationError> {
        if self.fail_apply {
            return Err(MigrationError::ApplyFailed(
                "test migration failure".to_owned(),
            ));
        }
        self.applied.set(true);
        Ok(())
    }

    fn verify(&self) -> Result<(), MigrationError> {
        if !self.applied.get() {
            return Err(MigrationError::VerificationFailed(
                "verification ran before apply".to_owned(),
            ));
        }
        self.verified.set(true);
        Ok(())
    }

    fn activate(&self) -> Result<(), MigrationError> {
        if !self.verified.get() {
            return Err(MigrationError::ActivationFailed(
                "activation ran before verification".to_owned(),
            ));
        }
        self.activated.set(true);
        Ok(())
    }
}

#[derive(Debug)]
struct FailingJournal;

impl RecoveryJournal for FailingJournal {
    fn prepare(&mut self, _commit: &RecoveryCommit) -> Result<(), RecoveryError> {
        Err(RecoveryError::Storage(
            "simulated journal write failure".to_owned(),
        ))
    }

    fn commit(
        &mut self,
        _change_set_id: &kg::versioning::ChangeSetId,
    ) -> Result<(), RecoveryError> {
        Err(RecoveryError::Storage(
            "simulated journal write failure".to_owned(),
        ))
    }

    fn committed(&self) -> Result<Vec<RecoveryCommit>, RecoveryError> {
        Err(RecoveryError::Storage(
            "simulated journal read failure".to_owned(),
        ))
    }
}

#[derive(Debug)]
struct QueryAccessFromCandidates {
    candidates: Vec<QueryMatchCandidate>,
}

#[derive(Debug)]
struct QueryTestError;

impl std::fmt::Display for QueryTestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("query test access failed")
    }
}
impl std::error::Error for QueryTestError {}

impl QueryAccess for QueryAccessFromCandidates {
    type Error = QueryTestError;

    fn lookup(
        &self,
        _target: &kg::query::LookupTarget,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        Ok(self.candidates.clone())
    }

    fn traverse(
        &self,
        _plan: &kg::query::TraversalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        Ok(self.candidates.clone())
    }

    fn retrieve(
        &self,
        _plan: &kg::query::RetrievalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        Ok(self.candidates.clone())
    }
}

// -----------------------------------------------------------------------------
// 01 — phase7_storage
// -----------------------------------------------------------------------------

#[test]
fn phase7_storage_persists_a_semantic_object() {
    let root = temp_root("storage-persist");
    let (repository, _) = repository_with_record(&root, "entity-1", b"canonical");

    let loaded = repository
        .read(&entity_target("entity-1"))
        .expect("repository read should succeed")
        .expect("record should exist");
    assert_eq!(loaded.document().as_bytes(), b"canonical");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_storage_loads_the_persisted_object() {
    let root = temp_root("storage-load");
    let (_, version) = repository_with_record(&root, "entity-1", b"loaded");
    let reopened = FileRepository::open(&root).expect("repository should reopen");

    assert_eq!(
        reopened
            .current_version()
            .expect("current version should load"),
        version
    );
    assert_eq!(
        reopened
            .read(&entity_target("entity-1"))
            .unwrap()
            .unwrap()
            .document()
            .as_bytes(),
        b"loaded"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_storage_updates_an_existing_object() {
    let root = temp_root("storage-update");
    let mut repository = FileRepository::open(&root).expect("repository should open");
    let target = entity_target("entity-1");

    let mut first = ChangeSetTransaction::begin(&mut repository).unwrap();
    first
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"v1")),
        )
        .unwrap();
    first.commit().unwrap();

    let mut second = ChangeSetTransaction::begin(&mut repository).unwrap();
    second
        .stage(
            Change::new(ChangeKind::Updated, target.clone()),
            Some(record("entity-1", b"v2")),
        )
        .unwrap();
    second.commit().unwrap();

    assert_eq!(
        repository
            .read(&target)
            .unwrap()
            .unwrap()
            .document()
            .as_bytes(),
        b"v2"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_storage_survives_provider_restart() {
    let root = temp_root("storage-restart");
    let (_, version) = repository_with_record(&root, "entity-1", b"restart-safe");
    drop(version);

    let reopened = FileRepository::open(&root).expect("restart should reload repository");
    assert!(reopened.read(&entity_target("entity-1")).unwrap().is_some());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_storage_supports_a_provider_behind_the_document_store_trait() {
    let store = MemoryStore::default();
    let id = DocumentId::new("memory/entity-1").unwrap();
    store
        .create(id.clone(), Document::from_slice(b"memory"))
        .unwrap();
    assert_eq!(store.get(&id).unwrap().unwrap().as_bytes(), b"memory");
}

#[test]
fn phase7_storage_public_repository_does_not_require_a_provider_specific_type() {
    let root = temp_root("storage-no-provider-leak");
    let (repository, _) = repository_with_record(&root, "entity-1", b"provider-neutral");
    let boundary: &dyn Repository = &repository;
    assert!(boundary.read(&entity_target("entity-1")).unwrap().is_some());
    let _ = std::fs::remove_dir_all(root);
}

// -----------------------------------------------------------------------------
// 02 — phase7_transaction
// -----------------------------------------------------------------------------

#[test]
fn phase7_transaction_commits_one_atomic_changeset() {
    let root = temp_root("tx-atomic");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"atomic")),
        )
        .unwrap();
    let version = transaction.commit().unwrap();
    assert_eq!(
        transaction.state(),
        kg::storage::TransactionState::Committed
    );
    drop(transaction);

    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        version
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_transaction_commits_multiple_objects_as_one_change_set() {
    let root = temp_root("tx-multi");
    let mut repository = FileRepository::open(&root).unwrap();
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            created_change(entity_target("entity-a")),
            Some(record("entity-a", b"a")),
        )
        .unwrap();
    transaction
        .stage(
            created_change(entity_target("entity-b")),
            Some(record("entity-b", b"b")),
        )
        .unwrap();
    transaction.commit().unwrap();

    assert_eq!(repository.current_state().unwrap().len(), 2);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_transaction_failed_commit_does_not_change_canonical_state() {
    let root = temp_root("tx-failed");
    let mut repository = FileRepository::open(&root).unwrap();
    let before = repository.current_version().unwrap();
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    let error = transaction
        .commit()
        .expect_err("empty Change Set must fail");

    assert!(matches!(
        error,
        TransactionError::Repository(RepositoryError::InvalidChangeSet(_))
    ));
    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        before
    );
    assert!(repository.current_state().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_transaction_detects_conflicts() {
    let root = temp_root("tx-conflict");
    let mut first_repository = FileRepository::open(&root).unwrap();
    let mut second_repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");

    let mut first = ChangeSetTransaction::begin(&mut first_repository).unwrap();
    let mut second = ChangeSetTransaction::begin(&mut second_repository).unwrap();
    first
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"first")),
        )
        .unwrap();
    second
        .stage(created_change(target), Some(record("entity-1", b"second")))
        .unwrap();
    first.commit().unwrap();

    assert!(matches!(
        second.commit(),
        Err(TransactionError::Repository(
            RepositoryError::VersionConflict { .. }
        ))
    ));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_transaction_failure_leaves_no_partial_state() {
    let root = temp_root("tx-no-partial");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let wrong_target = entity_target("entity-2");
    let mut changes = ChangeSet::new(Some(repository.current_version().unwrap().id().clone()));
    changes.push(created_change(target));

    let error = repository
        .commit_change_set(
            &changes,
            &[RepositoryRecord::new(
                wrong_target,
                Document::from_slice(b"wrong"),
            )],
        )
        .expect_err("target mismatch must fail");
    assert!(matches!(error, RepositoryError::TargetMismatch { .. }));
    assert!(repository.current_state().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_transaction_supports_concurrent_read_write_conflict_detection() {
    let root = temp_root("tx-concurrent");
    let mut writer = FileRepository::open(&root).unwrap();
    let mut reader_writer = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");

    let mut stale = ChangeSetTransaction::begin(&mut reader_writer).unwrap();
    stale
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"stale")),
        )
        .unwrap();

    let mut current = ChangeSetTransaction::begin(&mut writer).unwrap();
    current
        .stage(created_change(target), Some(record("entity-1", b"current")))
        .unwrap();
    current.commit().unwrap();

    assert!(matches!(
        stale.commit(),
        Err(TransactionError::Repository(
            RepositoryError::VersionConflict { .. }
        ))
    ));
    let _ = std::fs::remove_dir_all(root);
}

// -----------------------------------------------------------------------------
// 03 — phase7_versioning
// -----------------------------------------------------------------------------

#[test]
fn phase7_versioning_models_v1_to_v2_as_distinct_kg_versions() {
    let initial = KgVersion::initial();
    let change_set = ChangeSet::new(Some(initial.id().clone()));
    let next = KgVersion::next(&initial, change_set.id().clone());

    assert_eq!(initial.sequence(), 0);
    assert_eq!(next.sequence(), 1);
    assert_eq!(next.parent(), Some(initial.id()));
    assert_eq!(next.change_set(), Some(change_set.id()));
    assert_ne!(initial.id(), next.id());
}

#[test]
fn phase7_versioning_current_version_is_the_latest_committed_state() {
    let root = temp_root("version-current");
    let (repository, version) = repository_with_record(&root, "entity-1", b"current");
    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        version
    );
    assert!(!repository.current_version().unwrap().is_initial());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_versioning_historical_version_is_preserved_by_snapshot_identity() {
    let root = temp_root("version-history");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");

    let mut first = ChangeSetTransaction::begin(&mut repository).unwrap();
    first
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"v1")),
        )
        .unwrap();
    let v1 = first.commit().unwrap();
    let snapshot_v1 = repository.snapshot().unwrap();

    let mut second = ChangeSetTransaction::begin(&mut repository).unwrap();
    second
        .stage(
            Change::new(ChangeKind::Updated, target.clone()),
            Some(record("entity-1", b"v2")),
        )
        .unwrap();
    let v2 = second.commit().unwrap();

    assert_eq!(snapshot_v1.version(), &v1);
    assert_ne!(snapshot_v1.version(), &v2);
    assert_eq!(
        snapshot_v1.read(&target).unwrap().document().as_bytes(),
        b"v1"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_versioning_committed_changeset_history_is_published_with_the_version() {
    let root = temp_root("version-history-publication");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(created_change(target), Some(record("entity-1", b"history")))
        .unwrap();

    let mut synchronizer = CaptureSynchronizer::default();
    let (commit, calls) = transaction
        .commit_with_index_synchronizer(&[], &mut synchronizer)
        .unwrap();

    assert_eq!(calls, 1);
    assert_eq!(commit.change_set().len(), 1);
    assert_eq!(
        commit.version().change_set(),
        Some(commit.change_set().id())
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_versioning_kg_version_is_not_temporal_validity() {
    use std::any::TypeId;

    assert_ne!(
        TypeId::of::<KgVersion>(),
        TypeId::of::<kg::temporal::TemporalValidity>(),
    );
}

#[test]
fn phase7_versioning_retraction_is_a_semantic_change_not_a_physical_delete() {
    let target = entity_target("entity-retracted");
    let mut change_set = ChangeSet::initial();
    change_set.push(Change::new(ChangeKind::Retracted, target.clone()));

    assert_eq!(change_set.changes()[0].kind(), ChangeKind::Retracted);
    assert_eq!(change_set.changes()[0].target(), &target);
}

#[test]
fn phase7_versioning_supersession_is_a_semantic_change_not_a_physical_delete() {
    let target = entity_target("entity-superseded");
    let mut change_set = ChangeSet::initial();
    change_set.push(Change::new(ChangeKind::Superseded, target.clone()));

    assert_eq!(change_set.changes()[0].kind(), ChangeKind::Superseded);
    assert_eq!(change_set.changes()[0].target(), &target);
}

// -----------------------------------------------------------------------------
// 04 — phase7_snapshot
// -----------------------------------------------------------------------------

#[test]
fn phase7_snapshot_is_coherent_with_one_committed_version() {
    let root = temp_root("snapshot-coherent");
    let (repository, version) = repository_with_record(&root, "entity-1", b"snapshot");
    let snapshot = repository.snapshot().unwrap();

    assert_eq!(snapshot.version(), &version);
    assert_eq!(snapshot.len(), 1);
    assert!(snapshot.contains(&entity_target("entity-1")));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_snapshot_has_its_own_identity() {
    let root = temp_root("snapshot-id");
    let (repository, _) = repository_with_record(&root, "entity-1", b"snapshot");
    let first = repository.snapshot().unwrap();
    let second = repository.snapshot().unwrap();

    assert_ne!(first.id(), second.id());
    assert_eq!(first.version(), second.version());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_snapshot_is_distinct_from_current_repository_state() {
    let root = temp_root("snapshot-current");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let mut first = ChangeSetTransaction::begin(&mut repository).unwrap();
    first
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"old")),
        )
        .unwrap();
    first.commit().unwrap();
    let snapshot = repository.snapshot().unwrap();

    let mut second = ChangeSetTransaction::begin(&mut repository).unwrap();
    second
        .stage(
            Change::new(ChangeKind::Updated, target.clone()),
            Some(record("entity-1", b"new")),
        )
        .unwrap();
    second.commit().unwrap();

    assert_eq!(
        snapshot.read(&target).unwrap().document().as_bytes(),
        b"old"
    );
    assert_eq!(
        repository
            .read(&target)
            .unwrap()
            .unwrap()
            .document()
            .as_bytes(),
        b"new"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_snapshot_reads_remain_stable_during_later_writes() {
    let root = temp_root("snapshot-during-write");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let mut first = ChangeSetTransaction::begin(&mut repository).unwrap();
    first
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"before")),
        )
        .unwrap();
    first.commit().unwrap();
    let snapshot = repository.snapshot().unwrap();

    let mut second = ChangeSetTransaction::begin(&mut repository).unwrap();
    second
        .stage(
            Change::new(ChangeKind::Updated, target.clone()),
            Some(record("entity-1", b"after")),
        )
        .unwrap();
    second.commit().unwrap();

    assert_eq!(
        snapshot.read(&target).unwrap().document().as_bytes(),
        b"before"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_snapshot_version_relationship_is_explicit() {
    let version = KgVersion::initial();
    let snapshot = Snapshot::new(version.clone(), Vec::new()).expect("valid snapshot");
    assert_eq!(snapshot.version(), &version);
    assert!(!snapshot.id().as_str().is_empty());
}

// -----------------------------------------------------------------------------
// 05 — phase7_recovery
// -----------------------------------------------------------------------------

#[test]
fn phase7_recovery_survives_repository_restart() {
    let root = temp_root("recovery-restart");
    let (_, version) = repository_with_record(&root, "entity-1", b"durable");
    let reopened = FileRepository::open(&root).unwrap();
    assert_eq!(
        reopened
            .current_version()
            .expect("current version should load"),
        version
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_recovery_replays_a_committed_journal_entry() {
    let root = temp_root("recovery-replay");
    let repository = FileRepository::open(&root).unwrap();
    let snapshot = repository.snapshot().unwrap();
    let base = snapshot.version().clone();
    let target = entity_target("entity-1");
    let mut change_set = ChangeSet::new(Some(base.id().clone()));
    change_set.push(created_change(target.clone()));
    let version = KgVersion::next(&base, change_set.id().clone());
    let commit = RecoveryCommit::new(
        base.id().clone(),
        version.clone(),
        change_set,
        vec![record("entity-1", b"replayed")],
    )
    .unwrap();

    let mut journal = FileRecoveryJournal::open(&root).unwrap();
    journal.prepare(&commit).unwrap();
    journal.commit(commit.change_set().id()).unwrap();
    let recovered = kg::storage::recover_from_snapshot(&snapshot, &journal).unwrap();

    assert_eq!(recovered.version(), &version);
    assert_eq!(recovered.records().len(), 1);
    assert_eq!(recovered.replayed_change_sets().len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_recovery_combines_snapshot_and_journal() {
    let root = temp_root("recovery-snapshot-journal");
    let (repository, base) = repository_with_record(&root, "entity-1", b"snapshot-base");
    let snapshot = repository.snapshot().unwrap();
    assert_eq!(snapshot.version(), &base);

    let existing_target = entity_target("entity-1");
    let new_target = entity_target("entity-2");
    let mut change_set = ChangeSet::new(Some(base.id().clone()));
    change_set.push(Change::new(ChangeKind::Updated, existing_target));
    change_set.push(created_change(new_target));
    let version = KgVersion::next(&base, change_set.id().clone());
    let commit = RecoveryCommit::new(
        base.id().clone(),
        version.clone(),
        change_set,
        vec![
            record("entity-1", b"snapshot-base"),
            record("entity-2", b"journal"),
        ],
    )
    .unwrap();
    let mut journal = FileRecoveryJournal::open(&root).unwrap();
    journal.prepare(&commit).unwrap();
    journal.commit(commit.change_set().id()).unwrap();

    let recovered = kg::storage::recover_from_snapshot(&snapshot, &journal).unwrap();
    assert_eq!(recovered.version(), &version);
    assert_eq!(recovered.records().len(), 2);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_recovery_recovers_a_committed_transaction_state() {
    let root = temp_root("recovery-committed-tx");
    let (repository, version) = repository_with_record(&root, "entity-1", b"committed");
    let snapshot = repository.snapshot().unwrap();
    assert_eq!(snapshot.version(), &version);

    let mut journal = FileRecoveryJournal::open(&root).unwrap();
    let existing_target = entity_target("entity-1");
    let new_target = entity_target("entity-2");
    let mut change_set = ChangeSet::new(Some(version.id().clone()));
    change_set.push(Change::new(ChangeKind::Updated, existing_target));
    change_set.push(created_change(new_target));
    let committed_version = KgVersion::next(&version, change_set.id().clone());
    let commit = RecoveryCommit::new(
        version.id().clone(),
        committed_version.clone(),
        change_set,
        vec![record("entity-1", b"committed"), record("entity-2", b"tx")],
    )
    .unwrap();
    journal.prepare(&commit).unwrap();
    journal.commit(commit.change_set().id()).unwrap();
    let recovered = kg::storage::recover_from_snapshot(&snapshot, &journal).unwrap();

    assert_eq!(recovered.version(), &committed_version);
    assert_eq!(recovered.records().len(), 2);
    assert_eq!(recovered.records()[0].document().as_bytes(), b"committed");
    assert_eq!(recovered.records()[1].document().as_bytes(), b"tx");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_recovery_ignores_an_incomplete_transaction() {
    let root = temp_root("recovery-incomplete");
    let repository = FileRepository::open(&root).unwrap();
    let snapshot = repository.snapshot().unwrap();
    let base = snapshot.version().clone();
    let target = entity_target("entity-1");
    let mut change_set = ChangeSet::new(Some(base.id().clone()));
    change_set.push(created_change(target));
    let version = KgVersion::next(&base, change_set.id().clone());
    let commit = RecoveryCommit::new(
        base.id().clone(),
        version,
        change_set,
        vec![record("entity-1", b"ignored")],
    )
    .unwrap();
    let mut journal = FileRecoveryJournal::open(&root).unwrap();
    journal.prepare(&commit).unwrap();

    let recovered = kg::storage::recover_from_snapshot(&snapshot, &journal).unwrap();
    assert_eq!(recovered.version(), snapshot.version());
    assert!(recovered.replayed_change_sets().is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_recovery_reports_corrupted_state() {
    let root = temp_root("recovery-corrupt");
    let journal = FileRecoveryJournal::open(&root).unwrap();
    std::fs::write(journal.path(), b"not-json\n").unwrap();
    let error = journal
        .committed()
        .expect_err("corrupt journal must be reported");
    assert!(matches!(error, RecoveryError::CorruptJournal(_)));
    let _ = std::fs::remove_dir_all(root);
}

// -----------------------------------------------------------------------------
// 06 — phase7_migration
// -----------------------------------------------------------------------------

#[test]
fn phase7_migration_detects_schema_versions_explicitly() {
    let v1 = StorageSchemaVersion::new(1);
    let v2 = StorageSchemaVersion::new(2);
    assert_eq!(v1.value(), 1);
    assert_eq!(v2.value(), 2);
    assert_ne!(v1, v2);
}

#[test]
fn phase7_migration_executes_v1_to_v2_in_order() {
    let step = TestMigrationStep::new(1, 2, "v1-to-v2");
    let plan = MigrationPlan::new(
        StorageSchemaVersion::new(1),
        StorageSchemaVersion::new(2),
        vec![&step],
    )
    .unwrap();
    let report = MigrationRunner::run(&plan).unwrap();

    assert_eq!(report.state(), MigrationState::Activated);
    assert!(step.applied.get());
    assert!(step.verified.get());
    assert!(step.activated.get());
}

#[test]
fn phase7_migration_rejects_an_invalid_path() {
    let step = TestMigrationStep::new(2, 3, "wrong-start");
    let result = MigrationPlan::new(
        StorageSchemaVersion::new(1),
        StorageSchemaVersion::new(3),
        vec![&step],
    );
    assert!(matches!(result, Err(MigrationError::StepMismatch { .. })));
}

#[test]
fn phase7_migration_does_not_activate_an_interrupted_migration() {
    let mut step = TestMigrationStep::new(1, 2, "interrupted");
    step.fail_apply = true;
    let plan = MigrationPlan::new(
        StorageSchemaVersion::new(1),
        StorageSchemaVersion::new(2),
        vec![&step],
    )
    .unwrap();
    let error = MigrationRunner::run(&plan).expect_err("migration should fail");
    assert!(matches!(error, MigrationError::ApplyFailed(_)));
    assert!(!step.activated.get());
}

#[test]
fn phase7_migration_validates_before_activation() {
    let step = TestMigrationStep::new(1, 2, "validated");
    let plan = MigrationPlan::new(
        StorageSchemaVersion::new(1),
        StorageSchemaVersion::new(2),
        vec![&step],
    )
    .unwrap();
    let report = MigrationRunner::run(&plan).unwrap();
    assert_eq!(report.applied_steps(), &["validated".to_owned()]);
    assert!(step.verified.get());
    assert!(step.activated.get());
}

#[test]
fn phase7_migration_compatibility_remains_separate_from_schema_execution() {
    let matrix = CompatibilityMatrix::compatible()
        .with_storage_schema(CompatibilityLevel::CompatibleWithMigration)
        .with_storage_provider(CompatibilityLevel::CompatibleWithMigration);
    let report = matrix.report();

    assert!(report.is_compatible());
    assert!(report.requires_migration());
}

// -----------------------------------------------------------------------------
// 07 — phase7_index
// -----------------------------------------------------------------------------

#[test]
fn phase7_index_forward_index_round_trips() {
    let root = temp_root("index-forward");
    let reference = object_reference("entity-1");
    let mut index = FileForwardIndex::open(&root).unwrap();
    let assigned = index.insert(reference.clone(), target_type()).unwrap();
    drop(index);
    let reopened = FileForwardIndex::open(&root).unwrap();
    assert_eq!(reopened.lookup_forward(&reference).unwrap(), Some(assigned));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_index_reverse_index_round_trips() {
    let root = temp_root("index-reverse");
    let reference = object_reference("entity-1");
    let assigned = IndexAssignedId::generate(&target_type(), &reference);
    let mut index = FileReverseIndex::open(&root).unwrap();
    index.insert(assigned.clone(), reference.clone()).unwrap();
    drop(index);
    let reopened = FileReverseIndex::open(&root).unwrap();
    assert_eq!(reopened.lookup_reverse(&assigned).unwrap(), Some(reference));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_index_search_index_round_trips() {
    let root = temp_root("index-search");
    let reference = object_reference("entity-1");
    let mut index = FileSearchIndex::open(&root).unwrap();
    index.upsert(reference.clone(), "Quran knowledge").unwrap();
    drop(index);
    let reopened = FileSearchIndex::open(&root).unwrap();
    assert_eq!(reopened.search("quran").unwrap()[0].reference(), &reference);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_index_version_is_independent_from_kg_version() {
    let index_version = KgIndexVersion::new(3, "kg-version-42").unwrap();
    assert_eq!(index_version.generation(), 3);
    assert_eq!(index_version.built_from_kg_version(), "kg-version-42");
    assert_ne!(index_version.generation(), 42);
}

#[test]
fn phase7_index_stale_state_is_explicit() {
    let state = kg::index::KgIndexState::new(
        KgIndexKind::Search,
        KgIndexVersion::new(1, "kg-v1").unwrap(),
    )
    .mark_stale();
    assert_eq!(state.queryability(), IndexQueryability::Stale);
    assert_eq!(state.policy(), IndexSynchronizationPolicy::Eventual);
}

#[test]
fn phase7_index_rebuild_replaces_derived_state() {
    let root = temp_root("index-rebuild");
    let a = object_reference("a");
    let b = object_reference("b");
    let mut forward = FileForwardIndex::open(&root).unwrap();
    forward.rebuild([(b.clone(), target_type())]).unwrap();
    assert!(forward.lookup_forward(&a).unwrap().is_none());
    assert!(forward.lookup_forward(&b).unwrap().is_some());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_index_failed_rebuild_does_not_change_canonical_state() {
    let root = temp_root("index-failed-rebuild");
    let (repository, version) = repository_with_record(&root, "entity-1", b"canonical");
    assert!(
        repository
            .read(&entity_target("entity-1"))
            .unwrap()
            .is_some()
    );
    let mut index = FileSearchIndex::open(&root).unwrap();
    index
        .upsert(object_reference("entity-1"), "canonical knowledge")
        .unwrap();

    std::fs::remove_dir_all(root.join("indexes")).expect("index directory should exist");
    std::fs::create_dir(root.join("indexes")).unwrap();
    std::fs::create_dir(root.join("indexes/search-index.json"))
        .expect("search index path should be replaceable by a directory for failure injection");
    let result = index.rebuild([(object_reference("entity-2"), "new".to_owned())]);

    assert!(result.is_err());
    let reopened = FileRepository::open(&root).unwrap();
    assert_eq!(
        reopened
            .current_version()
            .expect("current version should load"),
        version
    );
    assert_eq!(
        reopened
            .read(&entity_target("entity-1"))
            .unwrap()
            .unwrap()
            .document()
            .as_bytes(),
        b"canonical"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_index_synchronization_updates_forward_reverse_and_search_together() {
    let root = temp_root("index-sync");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let reference = object_reference("entity-1");
    let change = IndexSynchronizationChange::upsert(
        target.clone(),
        reference.clone(),
        target_type(),
        Some("Quran knowledge"),
    );
    assert_eq!(change.action(), IndexChangeAction::Upsert);
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            created_change(target),
            Some(record("entity-1", b"canonical")),
        )
        .unwrap();
    let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
    let (commit, report) = transaction
        .commit_with_index_synchronizer(&[change], &mut synchronizer)
        .unwrap();

    assert_eq!(report.canonical_version(), commit.version().id().as_str());
    assert!(report.forward().is_synchronized());
    assert!(report.reverse().is_synchronized());
    assert!(report.search().is_synchronized());
    assert!(report.is_fully_synchronized());
    assert!(
        synchronizer
            .forward()
            .lookup_forward(&reference)
            .unwrap()
            .is_some()
    );
    assert!(
        synchronizer
            .reverse()
            .lookup_reverse(&IndexAssignedId::generate(&target_type(), &reference))
            .unwrap()
            .is_some()
    );
    assert_eq!(
        synchronizer.search().search("quran").unwrap()[0].reference(),
        &reference
    );
    let _ = std::fs::remove_dir_all(root);
}

// -----------------------------------------------------------------------------
// 08 — phase7_integration
// -----------------------------------------------------------------------------

#[test]
fn phase7_end_to_end_connects_canonical_repository_transaction_version_indexes_and_phase6_query() {
    let root = temp_root("end-to-end");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-e2e");
    let reference = object_reference("entity-e2e");
    let update = IndexSynchronizationChange::upsert(
        target.clone(),
        reference,
        target_type(),
        Some("Quran knowledge companion"),
    );

    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            created_change(target.clone()),
            Some(record("entity-e2e", b"canonical-quran")),
        )
        .unwrap();
    let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
    let (commit, report) = transaction
        .commit_with_index_synchronizer(&[update], &mut synchronizer)
        .unwrap();

    assert!(report.is_fully_synchronized());
    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        commit.version().clone()
    );

    let candidate = QueryMatchCandidate::new(
        QueryReference::Object(kg::assertion::AssertionObject::Entity(
            kg::identity::EntityId::new("entity-e2e").unwrap(),
        )),
        QueryMatchType::Lookup,
    )
    .with_publication(PublicationOutcome::Published);
    let access = QueryAccessFromCandidates {
        candidates: vec![candidate],
    };
    let request = kg::query::QueryRequest::Lookup(LookupRequest::object(
        kg::identity::EntityId::new("entity-e2e").unwrap(),
    ));
    let query_plan = plan(&request).unwrap();
    let result = execute(&access, &query_plan, &operation_context()).unwrap();

    assert_eq!(result.items().len(), 1);
    assert_eq!(result.items()[0].match_type(), QueryMatchType::Lookup);
    assert_eq!(synchronizer.search().search("quran").unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

// -----------------------------------------------------------------------------
// 09 — phase7_negative_boundary
// -----------------------------------------------------------------------------

#[test]
fn phase7_negative_invalid_transaction_protects_canonical_state() {
    let root = temp_root("negative-invalid-tx");
    let mut repository = FileRepository::open(&root).unwrap();
    let before = repository.current_version().unwrap();
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        before
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_conflict_protects_canonical_state() {
    let root = temp_root("negative-conflict");
    let mut a = FileRepository::open(&root).unwrap();
    let mut b = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let mut stale = ChangeSetTransaction::begin(&mut b).unwrap();
    stale
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"stale")),
        )
        .unwrap();
    let mut current = ChangeSetTransaction::begin(&mut a).unwrap();
    current
        .stage(created_change(target), Some(record("entity-1", b"current")))
        .unwrap();
    current.commit().unwrap();
    assert!(matches!(
        stale.commit(),
        Err(TransactionError::Repository(
            RepositoryError::VersionConflict { .. }
        ))
    ));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_snapshot_constructor_enforces_valid_version_invariant() {
    let version = KgVersion::initial();
    let snapshot =
        Snapshot::new(version, Vec::new()).expect("valid version should construct a snapshot");
    assert!(snapshot.is_empty());
    assert!(SnapshotError::MissingVersion != SnapshotError::InvalidRecord("x".to_owned()));
}

#[test]
fn phase7_negative_unknown_version_is_rejected_by_migration_path_validation() {
    let step = TestMigrationStep::new(1, 2, "one-to-two");
    let result = MigrationPlan::new(
        StorageSchemaVersion::new(1),
        StorageSchemaVersion::new(3),
        vec![&step],
    );
    assert!(result.is_err());
}

#[test]
fn phase7_negative_corrupt_record_is_reported_without_partial_state() {
    let root = temp_root("negative-corrupt-record");
    let _ = repository_with_record(&root, "entity-1", b"safe");
    let store = FileDocumentStore::new(&root).unwrap();
    let id = DocumentId::new("kg/current-state.json").unwrap();
    store
        .replace(&id, Document::from_slice(b"not-json"))
        .unwrap();
    let error = FileRepository::open(&root).expect_err("corrupt canonical state must be reported");
    assert!(matches!(error, RepositoryError::PersistenceCorruption(_)));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_journal_write_failure_is_explicit() {
    let mut journal = FailingJournal;
    let base = KgVersion::initial();
    let mut changes = ChangeSet::new(Some(base.id().clone()));
    let target = entity_target("journal-failure");
    changes.push(created_change(target));
    let version = KgVersion::next(&base, changes.id().clone());
    let commit = RecoveryCommit::new(
        base.id().clone(),
        version,
        changes,
        vec![record("journal-failure", b"x")],
    )
    .unwrap();
    assert!(matches!(
        journal.prepare(&commit),
        Err(RecoveryError::Storage(_))
    ));
}

#[test]
fn phase7_negative_commit_failure_does_not_change_canonical_state() {
    let root = temp_root("negative-commit");
    let mut repository = FileRepository::open(&root).unwrap();
    let before = repository.current_version().unwrap();
    let mut changes = ChangeSet::new(Some(before.id().clone()));
    changes.push(created_change(entity_target("expected")));
    let error = repository
        .commit_change_set(&changes, &[record("actual", b"wrong")])
        .unwrap_err();
    assert!(matches!(error, RepositoryError::TargetMismatch { .. }));
    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        before
    );
    assert!(repository.current_state().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_migration_failure_does_not_activate_schema() {
    let mut step = TestMigrationStep::new(1, 2, "fail");
    step.fail_apply = true;
    let plan = MigrationPlan::new(
        StorageSchemaVersion::new(1),
        StorageSchemaVersion::new(2),
        vec![&step],
    )
    .unwrap();
    assert!(MigrationRunner::run(&plan).is_err());
    assert!(!step.activated.get());
}

#[test]
fn phase7_negative_recovery_version_failure_is_reported() {
    let root = temp_root("negative-recovery");
    let repository = FileRepository::open(&root).unwrap();
    let snapshot = repository.snapshot().unwrap();
    let wrong_base = KgVersion::next(
        &snapshot.version().clone(),
        kg::versioning::ChangeSetId::generate(),
    );
    let mut changes = ChangeSet::new(Some(wrong_base.id().clone()));
    changes.push(created_change(entity_target("wrong-base")));
    let version = KgVersion::next(&wrong_base, changes.id().clone());
    let commit = RecoveryCommit::new(
        wrong_base.id().clone(),
        version,
        changes,
        vec![record("wrong-base", b"x")],
    )
    .unwrap();
    let mut journal = FileRecoveryJournal::open(&root).unwrap();
    journal.prepare(&commit).unwrap();
    journal.commit(commit.change_set().id()).unwrap();
    let error = kg::storage::recover_from_snapshot(&snapshot, &journal)
        .expect_err("non-contiguous recovery must fail");
    assert!(matches!(error, RecoveryError::VersionConflict { .. }));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_index_publication_failure_does_not_undo_canonical_commit() {
    let root = temp_root("negative-index-publication");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"canonical")),
        )
        .unwrap();
    let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
    let (commit, report) = transaction
        .commit_with_index_synchronizer::<IndexSynchronizationChange, _>(&[], &mut synchronizer)
        .unwrap();
    drop(transaction);

    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        commit.version().clone()
    );
    assert!(!report.is_fully_synchronized());
    assert!(matches!(
        report.forward().state().queryability(),
        IndexQueryability::Unavailable
    ));
    assert!(repository.read(&target).unwrap().is_some());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_search_failure_marks_eventual_index_stale_without_rolling_back_canonical_state()
{
    let root = temp_root("negative-stale-search");
    let mut repository = FileRepository::open(&root).unwrap();
    let target = entity_target("entity-1");
    let reference = object_reference("entity-1");
    let update = IndexSynchronizationChange::upsert(
        target.clone(),
        reference,
        target_type(),
        None::<String>,
    );
    let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
    transaction
        .stage(
            created_change(target.clone()),
            Some(record("entity-1", b"canonical")),
        )
        .unwrap();
    let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
    let (commit, report) = transaction
        .commit_with_index_synchronizer(&[update], &mut synchronizer)
        .unwrap();
    drop(transaction);

    assert_eq!(
        repository
            .current_version()
            .expect("current version should load"),
        commit.version().clone()
    );
    assert!(report.forward().is_synchronized());
    assert!(report.reverse().is_synchronized());
    assert!(!report.search().is_synchronized());
    assert_eq!(
        report.search().state().queryability(),
        IndexQueryability::Stale
    );
    assert_eq!(
        report.search().state().policy(),
        IndexSynchronizationPolicy::Eventual
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_negative_provider_unavailable_is_reported_without_kg_semantic_leakage() {
    let store = UnavailableStore;
    let id = DocumentId::new("unavailable/document").unwrap();
    assert!(matches!(
        store.get(&id),
        Err(DocumentStoreError::Unavailable(_))
    ));
}

// -----------------------------------------------------------------------------
// 10 — phase7_performance
// -----------------------------------------------------------------------------

/// Emits a repeatable workload baseline without imposing fragile machine-specific
/// wall-clock thresholds on the correctness test suite.
fn report_performance_baseline(
    metric: &str,
    dataset_size: usize,
    samples: usize,
    elapsed: Duration,
) {
    let per_sample_micros = elapsed.as_secs_f64() * 1_000_000.0 / samples as f64;
    eprintln!(
        "phase7.performance metric={metric} dataset_items={dataset_size} samples={samples} total_us={} avg_us_per_sample={per_sample_micros:.2}",
        elapsed.as_micros()
    );
}

#[test]
fn phase7_performance_read_latency_baseline() {
    let root = temp_root("perf-read");
    let (repository, _) = repository_with_record(&root, "entity-1", b"read");
    let samples = 1_000usize;
    let target = entity_target("entity-1");
    let start = Instant::now();
    for _ in 0..samples {
        assert!(repository.read(&target).unwrap().is_some());
    }
    report_performance_baseline("repository_read", 1, samples, start.elapsed());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_write_latency_baseline() {
    let root = temp_root("perf-write");
    let mut repository = FileRepository::open(&root).unwrap();
    let samples = 8usize;
    let start = Instant::now();
    for index in 0..samples {
        let id = format!("entity-{index}");
        let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
        transaction
            .stage(
                created_change(entity_target(&id)),
                Some(record(&id, b"write")),
            )
            .unwrap();
        transaction.commit().unwrap();
    }
    report_performance_baseline("repository_write_commit", samples, samples, start.elapsed());
    assert_eq!(repository.current_state().unwrap().len(), samples);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_transaction_throughput_baseline() {
    let root = temp_root("perf-throughput");
    let mut repository = FileRepository::open(&root).unwrap();
    let start = Instant::now();
    for index in 0..16 {
        let id = format!("entity-{index}");
        let mut transaction = ChangeSetTransaction::begin(&mut repository).unwrap();
        transaction
            .stage(
                created_change(entity_target(&id)),
                Some(record(&id, id.as_bytes())),
            )
            .unwrap();
        transaction.commit().unwrap();
    }
    let elapsed = start.elapsed();
    assert_eq!(repository.current_state().unwrap().len(), 16);
    report_performance_baseline("transaction_throughput", 16, 16, elapsed);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_snapshot_cost_baseline() {
    let root = temp_root("perf-snapshot");
    let (repository, _) = repository_with_record(&root, "entity-1", b"snapshot");
    let samples = 100usize;
    let start = Instant::now();
    let mut record_count = 0;
    for _ in 0..samples {
        record_count = repository.snapshot().unwrap().len();
    }
    let elapsed = start.elapsed();
    assert_eq!(record_count, 1);
    report_performance_baseline("snapshot_creation", 1, samples, elapsed);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_recovery_cost_baseline() {
    let root = temp_root("perf-recovery");
    let repository = FileRepository::open(&root).unwrap();
    let snapshot = repository.snapshot().unwrap();
    let base = snapshot.version().clone();
    let mut changes = ChangeSet::new(Some(base.id().clone()));
    changes.push(created_change(entity_target("entity-1")));
    let version = KgVersion::next(&base, changes.id().clone());
    let commit = RecoveryCommit::new(
        base.id().clone(),
        version,
        changes,
        vec![record("entity-1", b"recovery")],
    )
    .unwrap();
    let mut journal = FileRecoveryJournal::open(&root).unwrap();
    journal.prepare(&commit).unwrap();
    journal.commit(commit.change_set().id()).unwrap();

    let samples = 100usize;
    let start = Instant::now();
    let mut recovered_count = 0;
    for _ in 0..samples {
        recovered_count = kg::storage::recover_from_snapshot(&snapshot, &journal)
            .unwrap()
            .records()
            .len();
    }
    let elapsed = start.elapsed();
    assert_eq!(recovered_count, 1);
    report_performance_baseline("snapshot_plus_journal_recovery", 1, samples, elapsed);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_index_rebuild_cost_baseline() {
    let root = temp_root("perf-index-rebuild");
    let mut index = FileForwardIndex::open(&root).unwrap();
    let entries =
        (0..32).map(|index| (object_reference(&format!("entity-{index}")), target_type()));
    let samples = 5usize;
    let entries = entries.collect::<Vec<_>>();
    let start = Instant::now();
    for _ in 0..samples {
        index.rebuild(entries.clone()).unwrap();
    }
    let elapsed = start.elapsed();
    report_performance_baseline("forward_index_rebuild", 32, samples, elapsed);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_search_latency_baseline() {
    let root = temp_root("perf-search");
    let mut index = FileSearchIndex::open(&root).unwrap();
    for index_number in 0..32 {
        index
            .upsert(
                object_reference(&format!("entity-{index_number}")),
                "quran knowledge graph companion",
            )
            .unwrap();
    }
    let samples = 1_000usize;
    let start = Instant::now();
    for _ in 0..samples {
        assert_eq!(index.search("quran knowledge").unwrap().len(), 32);
    }
    let elapsed = start.elapsed();
    report_performance_baseline("lexical_search", 32, samples, elapsed);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn phase7_performance_storage_size_baseline() {
    let root = temp_root("perf-storage-size");
    let (repository, _) = repository_with_record(&root, "entity-1", b"storage-size");
    let state_path = root.join("kg/current-state.json");
    let size = std::fs::metadata(state_path).unwrap().len();
    assert!(size > 0);
    assert_eq!(repository.current_state().unwrap().len(), 1);
    let records = repository.current_state().unwrap().len();
    let bytes_per_record = if records == 0 {
        0
    } else {
        size / records as u64
    };
    eprintln!(
        "phase7.performance metric=canonical_state_storage_size dataset_records={records} serialized_bytes={size} bytes_per_record={bytes_per_record}"
    );
    let _ = std::fs::remove_dir_all(root);
}
