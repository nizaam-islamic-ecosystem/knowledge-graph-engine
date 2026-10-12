//! Change Set transaction boundary for the Knowledge Graph.
//!
//! Phase 7 initially uses Change Set transactions rather than a general-purpose
//! database transaction framework.
//!
//! A transaction captures the KG version from which it started, accumulates
//! semantic changes, validates that the base version is still current, and
//! delegates one atomic commit to the repository.

use core::fmt;

use super::repository::{
    CommittedChangeSet, Repository, RepositoryError, RepositoryIndexSynchronization,
    RepositoryIndexSynchronizer, RepositoryRecord,
};
use crate::versioning::{Change, ChangeSet, ChangeTarget, KgVersion, KgVersionId};

/// Lifecycle state of a Change Set transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionState {
    /// The transaction can still accept changes and be committed.
    Open,

    /// The transaction has been committed successfully.
    Committed,

    /// The transaction has been explicitly discarded.
    RolledBack,
}

/// Transaction-specific failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionError {
    /// The transaction is no longer open.
    Closed(TransactionState),

    /// The staged semantic record does not match the Change Set target.
    TargetMismatch {
        /// Target declared by the Change.
        expected: ChangeTarget,
        /// Target attached to the staged record.
        actual: ChangeTarget,
    },

    /// The repository changed after this transaction began.
    Conflict {
        /// Version identity captured when the transaction began.
        expected: KgVersionId,
        /// Version identity currently published by the repository.
        actual: KgVersionId,
    },

    /// The repository rejected the commit.
    Repository(RepositoryError),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed(state) => {
                write!(formatter, "transaction is no longer open: {state:?}")
            }
            Self::TargetMismatch { expected, actual } => {
                write!(
                    formatter,
                    "transaction target mismatch: expected {expected:?}, actual {actual:?}"
                )
            }
            Self::Conflict { expected, actual } => {
                write!(
                    formatter,
                    "transaction conflict: expected version {}, actual {}",
                    expected.as_str(),
                    actual.as_str()
                )
            }
            Self::Repository(error) => {
                write!(formatter, "transaction repository failure: {error}")
            }
        }
    }
}

impl std::error::Error for TransactionError {}

impl From<RepositoryError> for TransactionError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

/// A pending Change Set transaction.
///
/// The repository is borrowed mutably only for the transaction lifetime.
/// Changes are kept locally until [`ChangeSetTransaction::commit`] succeeds,
/// preventing partial semantic updates from being exposed through this
/// abstraction.
pub struct ChangeSetTransaction<'repository, R: Repository + ?Sized> {
    repository: &'repository mut R,
    change_set: ChangeSet,
    records: Vec<RepositoryRecord>,
    state: TransactionState,
}

impl<'repository, R: Repository + ?Sized> ChangeSetTransaction<'repository, R> {
    /// Begins a transaction against the repository's current KG version.
    pub fn begin(repository: &'repository mut R) -> Result<Self, TransactionError> {
        let current_version = repository.current_version()?;

        Ok(Self {
            repository,
            change_set: ChangeSet::new(Some(current_version.id().clone())),
            records: Vec::new(),
            state: TransactionState::Open,
        })
    }

    /// Returns the transaction's current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> TransactionState {
        self.state
    }

    /// Returns the Change Set being accumulated.
    #[must_use]
    pub fn change_set(&self) -> &ChangeSet {
        &self.change_set
    }

    /// Returns the KG version from which this transaction started.
    #[must_use]
    pub fn base_version(&self) -> Option<&crate::versioning::KgVersionId> {
        self.change_set.base_version()
    }

    /// Returns the currently staged repository records.
    #[must_use]
    pub fn records(&self) -> &[RepositoryRecord] {
        &self.records
    }

    /// Stages one semantic Change Set entry.
    ///
    /// The optional repository record carries the serialized semantic value
    /// associated with the change. Changes such as retraction or unlinking may
    /// legitimately carry no replacement record because their semantic meaning
    /// is represented by the Change Set itself.
    pub fn stage(
        &mut self,
        change: Change,
        record: Option<RepositoryRecord>,
    ) -> Result<(), TransactionError> {
        self.ensure_open()?;

        if let Some(record) = &record
            && change.target() != record.target()
        {
            return Err(TransactionError::TargetMismatch {
                expected: change.target().clone(),
                actual: record.target().clone(),
            });
        }

        self.change_set.push(change);

        if let Some(record) = record {
            self.records.push(record);
        }

        Ok(())
    }

    /// Validates that the repository has not advanced since this transaction
    /// began.
    pub fn validate(&self) -> Result<(), TransactionError> {
        self.ensure_open()?;

        let expected = self
            .change_set
            .base_version()
            .expect("Change Set transactions always have a base version");

        let actual = self.repository.current_version()?;

        if actual.id() != expected {
            return Err(TransactionError::Conflict {
                expected: expected.clone(),
                actual: actual.id().clone(),
            });
        }

        Ok(())
    }

    /// Atomically commits the Change Set through the repository.
    ///
    /// A successful commit produces the new canonical KG version. A failed
    /// repository commit leaves canonical state under the repository's atomic
    /// commit contract.
    pub fn commit(&mut self) -> Result<KgVersion, TransactionError> {
        self.ensure_open()?;
        self.validate()?;

        let version = self
            .repository
            .commit_change_set(&self.change_set, &self.records)?;

        self.state = TransactionState::Committed;

        Ok(version)
    }

    /// Commits canonical state and then publishes the committed Change Set to
    /// an explicit derived-index synchronizer.
    ///
    /// Canonical persistence happens first. A synchronization failure is
    /// represented by the synchronizer's report and cannot undo the committed
    /// KG state.
    pub fn commit_with_index_synchronizer<Update, S>(
        &mut self,
        updates: &[Update],
        synchronizer: &mut S,
    ) -> Result<(CommittedChangeSet, S::Report), TransactionError>
    where
        R: RepositoryIndexSynchronization,
        S: RepositoryIndexSynchronizer<Update>,
    {
        self.ensure_open()?;
        self.validate()?;

        let result = self.repository.commit_change_set_and_synchronize(
            &self.change_set,
            &self.records,
            updates,
            synchronizer,
        )?;

        self.state = TransactionState::Committed;

        Ok(result)
    }

    /// Explicitly rolls back the pending transaction.
    ///
    /// Rollback only discards local pending state. No repository mutation has
    /// occurred before a successful commit.
    pub fn rollback(&mut self) -> Result<(), TransactionError> {
        self.ensure_open()?;

        self.records.clear();
        self.state = TransactionState::RolledBack;

        Ok(())
    }

    fn ensure_open(&self) -> Result<(), TransactionError> {
        if self.state != TransactionState::Open {
            return Err(TransactionError::Closed(self.state));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    use crate::storage::document_store::Document;
    use crate::storage::repository::{Repository, RepositoryRecord};
    use crate::versioning::{ChangeKind, ChangeTarget};

    struct FakeRepository {
        version: KgVersion,
        commits: usize,
    }

    impl Default for FakeRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FakeRepository {
        fn new() -> Self {
            Self {
                version: KgVersion::initial(),
                commits: 0,
            }
        }
    }

    impl Repository for FakeRepository {
        fn read(
            &self,
            _target: &ChangeTarget,
        ) -> Result<Option<RepositoryRecord>, RepositoryError> {
            Ok(None)
        }

        fn current_state(&self) -> Result<Vec<RepositoryRecord>, RepositoryError> {
            Ok(Vec::new())
        }

        fn current_version(&self) -> Result<KgVersion, RepositoryError> {
            Ok(self.version.clone())
        }

        fn commit_change_set(
            &mut self,
            change_set: &ChangeSet,
            _records: &[RepositoryRecord],
        ) -> Result<KgVersion, RepositoryError> {
            let current = self.version.clone();

            if change_set.base_version() != Some(current.id()) {
                return Err(RepositoryError::VersionConflict {
                    expected: current.id().clone(),
                    actual: current.id().clone(),
                });
            }

            let next = KgVersion::next(&current, change_set.id().clone());

            self.version = next.clone();
            self.commits += 1;

            Ok(next)
        }

        fn snapshot(&self) -> Result<crate::storage::snapshot::Snapshot, RepositoryError> {
            crate::storage::snapshot::Snapshot::new(self.version.clone(), Vec::new())
                .map_err(|error| RepositoryError::SnapshotUnavailable(error.to_string()))
        }
    }

    fn entity_target() -> ChangeTarget {
        ChangeTarget::Entity(EntityId::new("entity-1").expect("valid entity identity"))
    }

    #[test]
    fn begin_captures_current_repository_version() {
        let mut repository = FakeRepository::new();
        let expected_version = repository.version.id().clone();

        let transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        assert_eq!(transaction.base_version(), Some(&expected_version));
        assert_eq!(transaction.state(), TransactionState::Open);
    }

    #[test]
    fn stage_preserves_change_and_matching_record() {
        let mut repository = FakeRepository::new();
        let mut transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        let target = entity_target();

        transaction
            .stage(
                Change::new(ChangeKind::Created, target.clone()),
                Some(RepositoryRecord::new(
                    target,
                    Document::from_slice(b"entity"),
                )),
            )
            .expect("matching stage should succeed");

        assert_eq!(transaction.change_set().len(), 1);
        assert_eq!(transaction.records().len(), 1);
    }

    #[test]
    fn mismatched_record_target_is_rejected_before_mutation() {
        let mut repository = FakeRepository::new();
        let mut transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        let expected = entity_target();
        let actual =
            ChangeTarget::Entity(EntityId::new("entity-2").expect("valid entity identity"));

        let error = transaction
            .stage(
                Change::new(ChangeKind::Created, expected.clone()),
                Some(RepositoryRecord::new(
                    actual.clone(),
                    Document::from_slice(b"entity"),
                )),
            )
            .expect_err("mismatched target must be rejected");

        assert!(matches!(error, TransactionError::TargetMismatch { .. }));
        assert!(transaction.change_set().is_empty());
        assert!(transaction.records().is_empty());
    }

    #[test]
    fn rollback_discards_pending_changes_without_repository_commit() {
        let mut repository = FakeRepository::new();
        let mut transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        let target = entity_target();

        transaction
            .stage(
                Change::new(ChangeKind::Created, target.clone()),
                Some(RepositoryRecord::new(
                    target,
                    Document::from_slice(b"entity"),
                )),
            )
            .expect("stage should succeed");

        transaction.rollback().expect("rollback should succeed");

        assert_eq!(transaction.state(), TransactionState::RolledBack);
        assert!(transaction.records().is_empty());
        assert_eq!(repository.commits, 0);
    }

    #[test]
    fn successful_commit_advances_repository_and_closes_transaction() {
        let mut repository = FakeRepository::new();

        let initial = repository.version.clone();

        let mut transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        transaction
            .stage(Change::new(ChangeKind::Created, entity_target()), None)
            .expect("stage should succeed");

        let committed = transaction.commit().expect("commit should succeed");

        assert_eq!(committed.sequence(), initial.sequence() + 1);
        assert_eq!(transaction.state(), TransactionState::Committed);
        assert_eq!(repository.commits, 1);
    }
    #[test]
    fn commit_with_index_synchronizer_publishes_after_commit() {
        #[derive(Default)]
        struct RecordingSynchronizer {
            seen: bool,
        }

        impl super::super::repository::RepositoryIndexSynchronizer<()> for RecordingSynchronizer {
            type Report = bool;

            fn synchronize(
                &mut self,
                commit: &super::super::repository::CommittedChangeSet,
                _updates: &[()],
            ) -> Self::Report {
                self.seen =
                    !commit.version().id().as_str().is_empty() && !commit.change_set().is_empty();
                self.seen
            }
        }

        let mut repository = FakeRepository::new();
        let target =
            ChangeTarget::Entity(crate::identity::EntityId::new("entity-1").expect("valid entity"));
        let mut transaction =
            ChangeSetTransaction::begin(&mut repository).expect("transaction should begin");

        transaction
            .stage(
                Change::new(ChangeKind::Created, target.clone()),
                Some(RepositoryRecord::new(
                    target,
                    crate::storage::document_store::Document::from_slice(b"entity"),
                )),
            )
            .expect("change should stage");

        let mut synchronizer = RecordingSynchronizer::default();
        let (commit, report) = transaction
            .commit_with_index_synchronizer(&[()], &mut synchronizer)
            .expect("commit should succeed");

        assert!(report);
        assert!(synchronizer.seen);
        assert_eq!(transaction.state(), TransactionState::Committed);
        assert_eq!(commit.change_set().len(), 1);
    }
}
