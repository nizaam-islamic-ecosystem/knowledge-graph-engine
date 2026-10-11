//! KG-owned physical derived indexes.
//!
//! Phase 6's logical adapters remain public for compatibility. Phase 7 adds
//! file-backed KG forward, reverse, and lexical search indexes plus explicit
//! derived-index state metadata. Nizaam Indexing remains responsible only for
//! Indexing-native identities and infrastructure; it does not own these KG
//! derived structures.

mod forward;
mod index_state;
mod reverse;
mod search;

pub use forward::{FileForwardIndex, ForwardIndexAccess, ForwardIndexError, lookup_forward};
pub use index_state::{
    IndexQueryability, IndexStateError, IndexStateObservation, IndexSynchronizationPolicy,
    IndexVersionObservation, KgIndexKind, KgIndexState, KgIndexVersion,
};
pub use reverse::{FileReverseIndex, ReverseIndexAccess, ReverseIndexError, lookup_reverse};
pub use search::{
    FileSearchIndex, IndexSearchAccess, KgSearchHit, KgSearchIndexAccess, SearchIndexError, search,
};

use crate::storage::FileDocumentStore;

/// Explicit physical change to apply to the KG-owned derived indexes.
///
/// The semantic repository deliberately does not infer this projection from an
/// opaque `RepositoryRecord`. The caller supplies the source-owned reference,
/// Indexing target type, and optional searchable text explicitly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexSynchronizationChange {
    target: crate::versioning::ChangeTarget,
    action: IndexChangeAction,
    reference: nizaam_indexing::index::reference::ObjectReference,
    target_type: nizaam_indexing::index::definition::TargetReferenceType,
    search_text: Option<String>,
}

impl IndexSynchronizationChange {
    /// Creates an upsert change for all applicable KG-owned derived indexes.
    pub fn upsert(
        target: crate::versioning::ChangeTarget,
        reference: nizaam_indexing::index::reference::ObjectReference,
        target_type: nizaam_indexing::index::definition::TargetReferenceType,
        search_text: Option<impl Into<String>>,
    ) -> Self {
        Self {
            target,
            action: IndexChangeAction::Upsert,
            reference,
            target_type,
            search_text: search_text.map(Into::into),
        }
    }

    /// Creates a removal change for all applicable KG-owned derived indexes.
    pub fn remove(
        target: crate::versioning::ChangeTarget,
        reference: nizaam_indexing::index::reference::ObjectReference,
        target_type: nizaam_indexing::index::definition::TargetReferenceType,
    ) -> Self {
        Self {
            target,
            action: IndexChangeAction::Remove,
            reference,
            target_type,
            search_text: None,
        }
    }

    #[must_use]
    pub const fn target(&self) -> &crate::versioning::ChangeTarget {
        &self.target
    }

    #[must_use]
    pub const fn action(&self) -> IndexChangeAction {
        self.action
    }

    #[must_use]
    pub fn reference(&self) -> &nizaam_indexing::index::reference::ObjectReference {
        &self.reference
    }

    #[must_use]
    pub fn target_type(&self) -> &nizaam_indexing::index::definition::TargetReferenceType {
        &self.target_type
    }

    #[must_use]
    pub fn search_text(&self) -> Option<&str> {
        self.search_text.as_deref()
    }
}

/// Mutation kind used by the derived-index synchronization boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexChangeAction {
    Upsert,
    Remove,
}

/// Result of synchronizing one derived index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexSynchronizationResult {
    kind: KgIndexKind,
    state: KgIndexState,
    error: Option<String>,
}

impl IndexSynchronizationResult {
    fn synchronized(state: KgIndexState) -> Self {
        Self {
            kind: state.kind(),
            state,
            error: None,
        }
    }

    fn failed(state: KgIndexState, error: impl Into<String>) -> Self {
        Self {
            kind: state.kind(),
            state,
            error: Some(error.into()),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> KgIndexKind {
        self.kind
    }

    #[must_use]
    pub const fn state(&self) -> &KgIndexState {
        &self.state
    }

    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    #[must_use]
    pub const fn is_synchronized(&self) -> bool {
        self.error.is_none() && matches!(self.state.queryability(), IndexQueryability::Queryable)
    }
}

/// Report returned after a canonical Change Set is propagated to the three
/// KG-owned physical indexes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexSynchronizationReport {
    canonical_version: String,
    forward: IndexSynchronizationResult,
    reverse: IndexSynchronizationResult,
    search: IndexSynchronizationResult,
}

impl IndexSynchronizationReport {
    #[must_use]
    pub fn canonical_version(&self) -> &str {
        &self.canonical_version
    }

    #[must_use]
    pub const fn forward(&self) -> &IndexSynchronizationResult {
        &self.forward
    }

    #[must_use]
    pub const fn reverse(&self) -> &IndexSynchronizationResult {
        &self.reverse
    }

    #[must_use]
    pub const fn search(&self) -> &IndexSynchronizationResult {
        &self.search
    }

    #[must_use]
    pub const fn is_fully_synchronized(&self) -> bool {
        self.forward.is_synchronized()
            && self.reverse.is_synchronized()
            && self.search.is_synchronized()
    }
}

/// KG-owned synchronization coordinator for forward, reverse, and search
/// indexes.
///
/// Forward and reverse use the strong policy. Search uses the eventual policy:
/// a search synchronization failure leaves canonical state committed and marks
/// search stale instead of pretending it is current.
#[derive(Debug)]
pub struct KgIndexSynchronizer {
    forward: FileForwardIndex,
    reverse: FileReverseIndex,
    search: FileSearchIndex,
    forward_state: KgIndexState,
    reverse_state: KgIndexState,
    search_state: KgIndexState,
}

impl KgIndexSynchronizer {
    /// Opens all KG-owned physical indexes below the default Nizaam root.
    pub fn open_default() -> Result<Self, IndexSynchronizationError> {
        Self::open(FileDocumentStore::default_root().map_err(IndexSynchronizationError::Storage)?)
    }

    /// Opens all KG-owned physical indexes below one storage root.
    pub fn open(root: impl Into<std::path::PathBuf>) -> Result<Self, IndexSynchronizationError> {
        let root = root.into();
        Ok(Self {
            forward: FileForwardIndex::open(root.clone())
                .map_err(IndexSynchronizationError::Forward)?,
            reverse: FileReverseIndex::open(root.clone())
                .map_err(IndexSynchronizationError::Reverse)?,
            search: FileSearchIndex::open(root).map_err(IndexSynchronizationError::Search)?,
            forward_state: KgIndexState::uninitialized(KgIndexKind::Forward),
            reverse_state: KgIndexState::uninitialized(KgIndexKind::Reverse),
            search_state: KgIndexState::uninitialized(KgIndexKind::Search),
        })
    }

    #[must_use]
    pub const fn forward(&self) -> &FileForwardIndex {
        &self.forward
    }

    #[must_use]
    pub const fn reverse(&self) -> &FileReverseIndex {
        &self.reverse
    }

    #[must_use]
    pub const fn search(&self) -> &FileSearchIndex {
        &self.search
    }

    #[must_use]
    pub const fn forward_state(&self) -> &KgIndexState {
        &self.forward_state
    }

    #[must_use]
    pub const fn reverse_state(&self) -> &KgIndexState {
        &self.reverse_state
    }

    #[must_use]
    pub const fn search_state(&self) -> &KgIndexState {
        &self.search_state
    }

    fn synchronize_forward(
        &mut self,
        kg_version: &str,
        updates: &[IndexSynchronizationChange],
    ) -> IndexSynchronizationResult {
        if updates.is_empty() {
            let state = self.forward_state.clone().mark_synchronization_failed();
            self.forward_state = state.clone();
            return IndexSynchronizationResult::failed(
                state,
                "no explicit forward-index projection was supplied",
            );
        }

        for update in updates {
            if let Err(error) = self.forward.synchronize(update) {
                let state = self.forward_state.clone().mark_synchronization_failed();
                self.forward_state = state.clone();
                return IndexSynchronizationResult::failed(state, error.to_string());
            }
        }

        match self.forward_state.synchronized_to(kg_version) {
            Ok(state) => {
                self.forward_state = state.clone();
                IndexSynchronizationResult::synchronized(state)
            }
            Err(error) => {
                let state = self.forward_state.clone().mark_synchronization_failed();
                self.forward_state = state.clone();
                IndexSynchronizationResult::failed(state, error.to_string())
            }
        }
    }

    fn synchronize_reverse(
        &mut self,
        kg_version: &str,
        updates: &[IndexSynchronizationChange],
    ) -> IndexSynchronizationResult {
        if updates.is_empty() {
            let state = self.reverse_state.clone().mark_synchronization_failed();
            self.reverse_state = state.clone();
            return IndexSynchronizationResult::failed(
                state,
                "no explicit reverse-index projection was supplied",
            );
        }

        for update in updates {
            if let Err(error) = self.reverse.synchronize(update) {
                let state = self.reverse_state.clone().mark_synchronization_failed();
                self.reverse_state = state.clone();
                return IndexSynchronizationResult::failed(state, error.to_string());
            }
        }

        match self.reverse_state.synchronized_to(kg_version) {
            Ok(state) => {
                self.reverse_state = state.clone();
                IndexSynchronizationResult::synchronized(state)
            }
            Err(error) => {
                let state = self.reverse_state.clone().mark_synchronization_failed();
                self.reverse_state = state.clone();
                IndexSynchronizationResult::failed(state, error.to_string())
            }
        }
    }

    fn synchronize_search(
        &mut self,
        kg_version: &str,
        updates: &[IndexSynchronizationChange],
    ) -> IndexSynchronizationResult {
        if updates.is_empty() {
            let state = self.search_state.clone().mark_synchronization_failed();
            self.search_state = state.clone();
            return IndexSynchronizationResult::failed(
                state,
                "no explicit search-index projection was supplied",
            );
        }

        for update in updates {
            if let Err(error) = self.search.synchronize(update) {
                let state = self.search_state.clone().mark_synchronization_failed();
                self.search_state = state.clone();
                return IndexSynchronizationResult::failed(state, error.to_string());
            }
        }

        match self.search_state.synchronized_to(kg_version) {
            Ok(state) => {
                self.search_state = state.clone();
                IndexSynchronizationResult::synchronized(state)
            }
            Err(error) => {
                let state = self.search_state.clone().mark_synchronization_failed();
                self.search_state = state.clone();
                IndexSynchronizationResult::failed(state, error.to_string())
            }
        }
    }
}

impl crate::storage::RepositoryIndexSynchronizer<IndexSynchronizationChange>
    for KgIndexSynchronizer
{
    type Report = IndexSynchronizationReport;

    fn synchronize(
        &mut self,
        commit: &crate::storage::CommittedChangeSet,
        updates: &[IndexSynchronizationChange],
    ) -> Self::Report {
        let kg_version = commit.version().id().as_str();
        let updates_are_valid = updates.iter().all(|update| {
            commit
                .change_set()
                .changes()
                .iter()
                .any(|change| change.target() == update.target())
        });

        if !updates_are_valid {
            let message =
                "index synchronization projection does not match the committed Change Set";
            let forward_state = self.forward_state.clone().mark_synchronization_failed();
            let reverse_state = self.reverse_state.clone().mark_synchronization_failed();
            let search_state = self.search_state.clone().mark_synchronization_failed();
            self.forward_state = forward_state.clone();
            self.reverse_state = reverse_state.clone();
            self.search_state = search_state.clone();
            return IndexSynchronizationReport {
                canonical_version: kg_version.to_owned(),
                forward: IndexSynchronizationResult::failed(forward_state, message),
                reverse: IndexSynchronizationResult::failed(reverse_state, message),
                search: IndexSynchronizationResult::failed(search_state, message),
            };
        }

        IndexSynchronizationReport {
            canonical_version: kg_version.to_owned(),
            forward: self.synchronize_forward(kg_version, updates),
            reverse: self.synchronize_reverse(kg_version, updates),
            search: self.synchronize_search(kg_version, updates),
        }
    }
}

/// Errors encountered while opening the physical synchronization coordinator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexSynchronizationError {
    Storage(crate::storage::DocumentStoreError),
    Forward(ForwardIndexError),
    Reverse(ReverseIndexError),
    Search(SearchIndexError),
}

impl core::fmt::Display for IndexSynchronizationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Storage(error) => {
                write!(formatter, "index synchronization storage failure: {error}")
            }
            Self::Forward(error) => {
                write!(formatter, "forward index initialization failed: {error}")
            }
            Self::Reverse(error) => {
                write!(formatter, "reverse index initialization failed: {error}")
            }
            Self::Search(error) => write!(formatter, "search index initialization failed: {error}"),
        }
    }
}

impl std::error::Error for IndexSynchronizationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use nizaam_indexing::IndexAssignedId;
    use nizaam_indexing::index::definition::TargetReferenceType;
    use nizaam_indexing::index::reference::ObjectReference;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-index-module-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn public_index_module_connects_forward_reverse_search_and_state() {
        let root = root();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();

        let mut forward = FileForwardIndex::open(&root).unwrap();
        let assigned = forward
            .insert(reference.clone(), target_type.clone())
            .unwrap();

        let mut reverse = FileReverseIndex::open(&root).unwrap();
        reverse.insert(assigned.clone(), reference.clone()).unwrap();

        let mut search = FileSearchIndex::open(&root).unwrap();
        search.upsert(reference.clone(), "Quran knowledge").unwrap();

        assert_eq!(
            forward.lookup_forward(&reference).unwrap(),
            Some(assigned.clone())
        );
        assert_eq!(
            reverse.lookup_reverse(&assigned).unwrap(),
            Some(reference.clone())
        );
        assert_eq!(search.search("quran").unwrap()[0].reference(), &reference);

        let state = KgIndexState::new(
            KgIndexKind::Search,
            KgIndexVersion::new(1, "kg-v1").unwrap(),
        );
        assert_eq!(state.queryability(), IndexQueryability::Queryable);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_or_stale_derived_index_state_does_not_change_canonical_identity() {
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let assigned = IndexAssignedId::generate(&target_type, &reference);

        let stale = KgIndexState::new(
            KgIndexKind::Forward,
            KgIndexVersion::new(2, "kg-v1").unwrap(),
        )
        .mark_stale();

        assert_eq!(stale.queryability(), IndexQueryability::Stale);
        assert_eq!(
            assigned,
            IndexAssignedId::generate(&target_type, &reference)
        );
        assert_eq!(reference.source(), "kg");
        assert_eq!(reference.object_reference(), "entity-1");
    }
    #[test]
    fn repository_commit_synchronizes_all_three_derived_indexes() {
        use crate::identity::EntityId;
        use crate::storage::{
            Document, FileRepository, Repository, RepositoryIndexSynchronization, RepositoryRecord,
        };
        use nizaam_indexing::index::definition::TargetReferenceType;
        use nizaam_indexing::index::reference::ObjectReference;

        let root = root();
        let mut repository = FileRepository::open(&root).unwrap();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let target = crate::versioning::ChangeTarget::Entity(EntityId::new("entity-1").unwrap());
        let mut change_set = crate::versioning::ChangeSet::new(Some(
            repository.current_version().unwrap().id().clone(),
        ));
        change_set.push(crate::versioning::Change::new(
            crate::versioning::ChangeKind::Created,
            target.clone(),
        ));
        let record = RepositoryRecord::new(target.clone(), Document::from_slice(b"entity-1"));
        let update = IndexSynchronizationChange::upsert(
            target.clone(),
            reference.clone(),
            target_type.clone(),
            Some("Quran knowledge"),
        );

        let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
        let (commit, report) = repository
            .commit_change_set_and_synchronize(&change_set, &[record], &[update], &mut synchronizer)
            .unwrap();

        assert_eq!(report.canonical_version(), commit.version().id().as_str());
        assert!(report.is_fully_synchronized());
        assert!(
            synchronizer
                .forward()
                .lookup_forward(&reference)
                .unwrap()
                .is_some()
        );
        let assigned = nizaam_indexing::IndexAssignedId::generate(&target_type, &reference);
        assert_eq!(
            synchronizer.reverse().lookup_reverse(&assigned).unwrap(),
            Some(reference.clone())
        );
        assert_eq!(
            synchronizer.search().search("quran").unwrap()[0].reference(),
            &reference
        );
        assert_eq!(
            synchronizer
                .forward_state()
                .version()
                .built_from_kg_version(),
            commit.version().id().as_str()
        );
        assert_eq!(
            synchronizer.search_state().policy(),
            IndexSynchronizationPolicy::Eventual
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn search_sync_failure_leaves_canonical_commit_and_marks_search_stale() {
        use crate::identity::EntityId;
        use crate::storage::{
            Document, FileRepository, Repository, RepositoryIndexSynchronization, RepositoryRecord,
        };
        use nizaam_indexing::index::definition::TargetReferenceType;
        use nizaam_indexing::index::reference::ObjectReference;

        let root = root();
        let mut repository = FileRepository::open(&root).unwrap();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let target = crate::versioning::ChangeTarget::Entity(EntityId::new("entity-1").unwrap());
        let mut change_set = crate::versioning::ChangeSet::new(Some(
            repository.current_version().unwrap().id().clone(),
        ));
        change_set.push(crate::versioning::Change::new(
            crate::versioning::ChangeKind::Created,
            target.clone(),
        ));
        let record = RepositoryRecord::new(target.clone(), Document::from_slice(b"entity-1"));
        let update =
            IndexSynchronizationChange::upsert(target, reference, target_type, None::<String>);

        let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
        let (commit, report) = repository
            .commit_change_set_and_synchronize(&change_set, &[record], &[update], &mut synchronizer)
            .unwrap();

        assert!(!report.is_fully_synchronized());
        assert!(report.search().error().is_some());
        assert_eq!(
            report.search().state().queryability(),
            IndexQueryability::Stale
        );
        assert_eq!(
            repository
                .current_version()
                .expect("current version should load"),
            *commit.version(),
        );
        assert_eq!(repository.current_state().unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }
}
