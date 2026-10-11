//! Knowledge Graph storage, transaction, snapshot, and recovery boundaries.
//!
//! The storage module exposes the provider-neutral storage contract together
//! with the initial file-backed provider. Semantic repository consumers should
//! use the re-exported repository and transaction types rather than depending
//! on provider-specific filesystem details.

pub mod document_store;
pub mod recovery;
pub mod repository;
pub mod snapshot;
pub mod transaction;

pub use document_store::{
    Document, DocumentId, DocumentStore, DocumentStoreError, FileDocumentStore,
};
pub use recovery::{
    FileRecoveryJournal, RecoveredState, RecoveryCommit, RecoveryError, RecoveryJournal,
    recover_from_snapshot,
};
pub use repository::{
    CommittedChangeSet, DocumentBackedRepository, FileRepository, Repository, RepositoryError,
    RepositoryIndexSynchronization, RepositoryIndexSynchronizer, RepositoryRecord,
};
pub use snapshot::{Snapshot, SnapshotError};
pub use transaction::{ChangeSetTransaction, TransactionError, TransactionState};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    use crate::versioning::{Change, ChangeKind, ChangeSet, ChangeTarget, KgVersion};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-storage-{prefix}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos()
        ))
    }

    fn entity_target(id: &str) -> ChangeTarget {
        ChangeTarget::Entity(EntityId::new(id).expect("test entity identity should be valid"))
    }

    fn repository_record(id: &str, bytes: &[u8]) -> RepositoryRecord {
        RepositoryRecord::new(entity_target(id), Document::from_slice(bytes))
    }

    #[test]
    fn storage_public_api_connects_provider_repository_transaction_and_snapshot() {
        let root = temp_root("public-api");

        let store = FileDocumentStore::new(&root).expect("file-backed document store should open");

        let document_id =
            DocumentId::new("test/document.json").expect("test document id should be valid");

        store
            .create(document_id.clone(), Document::from_slice(br#"{"value":1}"#))
            .expect("document should be created");

        assert_eq!(
            store
                .get(&document_id)
                .expect("document should be readable")
                .expect("document should exist")
                .as_bytes(),
            br#"{"value":1}"#
        );

        let mut repository = FileRepository::open(&root).expect("repository should open");

        let mut transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        let target = entity_target("entity-1");

        transaction
            .stage(
                Change::new(ChangeKind::Created, target.clone()),
                Some(RepositoryRecord::new(
                    target.clone(),
                    Document::from_slice(b"entity-1"),
                )),
            )
            .expect("change should stage");

        let committed_version = transaction.commit().expect("transaction should commit");

        let snapshot = repository.snapshot().expect("snapshot should be created");

        assert_eq!(snapshot.version(), &committed_version);
        assert_eq!(
            snapshot
                .read(&target)
                .expect("snapshot should contain committed entity")
                .document()
                .as_bytes(),
            b"entity-1"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn committed_transaction_survives_repository_restart_and_snapshot_remains_coherent() {
        let root = temp_root("restart");

        let target = entity_target("entity-1");

        let committed_version = {
            let mut repository = FileRepository::open(&root).expect("repository should open");

            let mut transaction =
                ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

            transaction
                .stage(
                    Change::new(ChangeKind::Created, target.clone()),
                    Some(repository_record("entity-1", b"persisted")),
                )
                .expect("change should stage");

            transaction.commit().expect("transaction should commit")
        };

        let reopened = FileRepository::open(&root).expect("repository should reopen");

        assert_eq!(
            reopened
                .current_version()
                .expect("current version should load"),
            committed_version
        );

        assert_eq!(
            reopened
                .read(&target)
                .expect("repository read should succeed")
                .expect("committed record should exist")
                .document()
                .as_bytes(),
            b"persisted"
        );

        let snapshot = reopened
            .snapshot()
            .expect("snapshot should be created after restart");

        assert_eq!(snapshot.version(), &committed_version);
        assert!(snapshot.contains(&target));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rolled_back_transaction_does_not_change_canonical_state() {
        let root = temp_root("rollback");

        let initial_version;
        {
            let mut repository = FileRepository::open(&root).expect("repository should open");

            initial_version = repository
                .current_version()
                .expect("initial version should be readable");

            let target = entity_target("entity-rollback");

            let mut transaction =
                ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

            transaction
                .stage(
                    Change::new(ChangeKind::Created, target.clone()),
                    Some(repository_record("entity-rollback", b"not-committed")),
                )
                .expect("change should stage");

            transaction.rollback().expect("rollback should succeed");

            assert_eq!(transaction.state(), TransactionState::RolledBack);
            assert_eq!(
                repository
                    .current_version()
                    .expect("version should remain unchanged"),
                initial_version
            );
            assert!(
                repository
                    .read(&target)
                    .expect("repository read should succeed")
                    .is_none()
            );
        }

        let reopened = FileRepository::open(&root).expect("repository should reopen");

        assert_eq!(
            reopened
                .current_version()
                .expect("version should survive restart"),
            initial_version
        );
        assert!(
            reopened
                .current_state()
                .expect("state should load")
                .is_empty()
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_replays_committed_change_and_ignores_uncommitted_prepare() {
        let root = temp_root("recovery");

        let base_version = KgVersion::initial();
        let snapshot = Snapshot::new(base_version.clone(), Vec::new())
            .expect("initial snapshot should be valid");

        let target = entity_target("entity-recovered");
        let mut change_set = ChangeSet::new(Some(base_version.id().clone()));

        change_set.push(Change::new(ChangeKind::Created, target.clone()));

        let next_version = KgVersion::next(&base_version, change_set.id().clone());

        let committed = RecoveryCommit::new(
            base_version.id().clone(),
            next_version.clone(),
            change_set.clone(),
            vec![repository_record("entity-recovered", b"recovered")],
        )
        .expect("recovery commit should be valid");

        let mut journal = FileRecoveryJournal::open(&root).expect("recovery journal should open");

        journal
            .prepare(&committed)
            .expect("prepare should be durable");

        assert!(
            journal
                .committed()
                .expect("journal should be readable")
                .is_empty(),
            "an incomplete prepare must not become canonical"
        );

        journal
            .commit(committed.change_set().id())
            .expect("commit marker should be durable");

        let recovered =
            recover_from_snapshot(&snapshot, &journal).expect("committed journal should replay");

        assert_eq!(recovered.version(), &next_version);
        assert_eq!(recovered.records().len(), 1);
        assert_eq!(recovered.records()[0].target(), &target);
        assert_eq!(recovered.records()[0].document().as_bytes(), b"recovered");
        assert_eq!(recovered.replayed_change_sets(), &[change_set]);

        let _ = std::fs::remove_dir_all(root);
    }
}
