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

use crate::storage::{Document, DocumentId, DocumentStore, FileDocumentStore};

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
    state_store: FileDocumentStore,
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
        let state_store =
            FileDocumentStore::new(root.clone()).map_err(IndexSynchronizationError::Storage)?;
        // Read the three index files and their state snapshots while holding the
        // same lock used for synchronization, preventing a mixed-generation open.
        let _lock = state_store
            .acquire_exclusive_lock()
            .map_err(IndexSynchronizationError::Storage)?;
        let forward_state = load_index_state(&state_store, KgIndexKind::Forward)?;
        let reverse_state = load_index_state(&state_store, KgIndexKind::Reverse)?;
        let search_state = load_index_state(&state_store, KgIndexKind::Search)?;
        let forward =
            FileForwardIndex::open(root.clone()).map_err(IndexSynchronizationError::Forward)?;
        let reverse =
            FileReverseIndex::open(root.clone()).map_err(IndexSynchronizationError::Reverse)?;
        let search = FileSearchIndex::open(root).map_err(IndexSynchronizationError::Search)?;
        drop(_lock);
        Ok(Self {
            forward,
            reverse,
            search,
            state_store,
            forward_state,
            reverse_state,
            search_state,
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

    fn refresh_states_from_disk(&mut self) -> Result<(), String> {
        self.forward_state = load_index_state(&self.state_store, KgIndexKind::Forward)
            .map_err(|error| error.to_string())?;
        self.reverse_state = load_index_state(&self.state_store, KgIndexKind::Reverse)
            .map_err(|error| error.to_string())?;
        self.search_state = load_index_state(&self.state_store, KgIndexKind::Search)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn state_can_apply(state: &KgIndexState, commit: &crate::storage::CommittedChangeSet) -> bool {
        let base = commit
            .change_set()
            .base_version()
            .map(|version| version.as_str());
        let built_from = state.version().built_from_kg_version();
        if built_from == commit.version().id().as_str() {
            return true; // Idempotent retry of a commit already marked synchronized.
        }
        let has_no_history = built_from == "uninitialized";
        let is_initial_transition = commit.version().sequence() == 1 && base.is_some();
        if has_no_history {
            is_initial_transition
        } else {
            base == Some(built_from)
        }
    }

    fn synchronize_forward(
        &mut self,
        commit: &crate::storage::CommittedChangeSet,
        updates: &[IndexSynchronizationChange],
    ) -> IndexSynchronizationResult {
        let previous = self.forward_state.clone();
        if !Self::state_can_apply(&previous, commit) {
            return self.record_failure(
                KgIndexKind::Forward,
                "forward index is not at the committed Change Set base version",
            );
        }
        if previous.version().built_from_kg_version() == commit.version().id().as_str() {
            return IndexSynchronizationResult::synchronized(previous);
        }
        if let Err(error) = self.forward.synchronize_batch_locked(updates) {
            return self.record_failure(KgIndexKind::Forward, &error.to_string());
        }
        let next = match previous.synchronized_to(commit.version().id().as_str()) {
            Ok(state) => state,
            Err(error) => return self.record_failure(KgIndexKind::Forward, &error.to_string()),
        };
        match persist_index_state(&self.state_store, &next) {
            Ok(()) => {
                self.forward_state = next.clone();
                IndexSynchronizationResult::synchronized(next)
            }
            Err(error) => {
                let _ = self.refresh_states_from_disk();
                self.record_failure(
                    KgIndexKind::Forward,
                    &format!("index data saved but state persistence failed: {error}"),
                )
            }
        }
    }

    fn synchronize_reverse(
        &mut self,
        commit: &crate::storage::CommittedChangeSet,
        updates: &[IndexSynchronizationChange],
    ) -> IndexSynchronizationResult {
        let previous = self.reverse_state.clone();
        if !Self::state_can_apply(&previous, commit) {
            return self.record_failure(
                KgIndexKind::Reverse,
                "reverse index is not at the committed Change Set base version",
            );
        }
        if previous.version().built_from_kg_version() == commit.version().id().as_str() {
            return IndexSynchronizationResult::synchronized(previous);
        }
        if let Err(error) = self.reverse.synchronize_batch_locked(updates) {
            return self.record_failure(KgIndexKind::Reverse, &error.to_string());
        }
        let next = match previous.synchronized_to(commit.version().id().as_str()) {
            Ok(state) => state,
            Err(error) => return self.record_failure(KgIndexKind::Reverse, &error.to_string()),
        };
        match persist_index_state(&self.state_store, &next) {
            Ok(()) => {
                self.reverse_state = next.clone();
                IndexSynchronizationResult::synchronized(next)
            }
            Err(error) => {
                let _ = self.refresh_states_from_disk();
                self.record_failure(
                    KgIndexKind::Reverse,
                    &format!("index data saved but state persistence failed: {error}"),
                )
            }
        }
    }

    fn synchronize_search(
        &mut self,
        commit: &crate::storage::CommittedChangeSet,
        updates: &[IndexSynchronizationChange],
    ) -> IndexSynchronizationResult {
        let previous = self.search_state.clone();
        if !Self::state_can_apply(&previous, commit) {
            return self.record_failure(
                KgIndexKind::Search,
                "search index is not at the committed Change Set base version",
            );
        }
        if previous.version().built_from_kg_version() == commit.version().id().as_str() {
            return IndexSynchronizationResult::synchronized(previous);
        }
        if let Err(error) = self.search.synchronize_batch_locked(updates) {
            return self.record_failure(KgIndexKind::Search, &error.to_string());
        }
        let next = match previous.synchronized_to(commit.version().id().as_str()) {
            Ok(state) => state,
            Err(error) => return self.record_failure(KgIndexKind::Search, &error.to_string()),
        };
        match persist_index_state(&self.state_store, &next) {
            Ok(()) => {
                self.search_state = next.clone();
                IndexSynchronizationResult::synchronized(next)
            }
            Err(error) => {
                let _ = self.refresh_states_from_disk();
                self.record_failure(
                    KgIndexKind::Search,
                    &format!("index data saved but state persistence failed: {error}"),
                )
            }
        }
    }

    fn record_failure(&mut self, kind: KgIndexKind, message: &str) -> IndexSynchronizationResult {
        let state = match kind {
            KgIndexKind::Forward => self.forward_state.clone().mark_synchronization_failed(),
            KgIndexKind::Reverse => self.reverse_state.clone().mark_synchronization_failed(),
            KgIndexKind::Search => self.search_state.clone().mark_synchronization_failed(),
        };
        if let Err(error) = persist_index_state(&self.state_store, &state) {
            let combined = format!("{message}; failed to persist index state: {error}");
            match kind {
                KgIndexKind::Forward => self.forward_state = state.clone(),
                KgIndexKind::Reverse => self.reverse_state = state.clone(),
                KgIndexKind::Search => self.search_state = state.clone(),
            }
            return IndexSynchronizationResult::failed(state, combined);
        }
        match kind {
            KgIndexKind::Forward => self.forward_state = state.clone(),
            KgIndexKind::Reverse => self.reverse_state = state.clone(),
            KgIndexKind::Search => self.search_state = state.clone(),
        }
        IndexSynchronizationResult::failed(state, message)
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
        let _lock = match self.state_store.acquire_exclusive_lock() {
            Ok(lock) => lock,
            Err(error) => {
                let message = format!("failed to lock derived-index synchronization: {error}");
                // Do not mutate persistent state without the lock. Report each
                // index as unavailable/stale in this attempt; the next owner of
                // the lock will reload and persist the appropriate status.
                let forward_state = self.forward_state.clone().mark_synchronization_failed();
                let reverse_state = self.reverse_state.clone().mark_synchronization_failed();
                let search_state = self.search_state.clone().mark_synchronization_failed();
                let forward = IndexSynchronizationResult::failed(forward_state, &message);
                let reverse = IndexSynchronizationResult::failed(reverse_state, &message);
                let search = IndexSynchronizationResult::failed(search_state, &message);
                return IndexSynchronizationReport {
                    canonical_version: kg_version.to_owned(),
                    forward,
                    reverse,
                    search,
                };
            }
        };
        if let Err(message) = self.refresh_states_from_disk() {
            let forward = self.record_failure(KgIndexKind::Forward, &message);
            let reverse = self.record_failure(KgIndexKind::Reverse, &message);
            let search = self.record_failure(KgIndexKind::Search, &message);
            return IndexSynchronizationReport {
                canonical_version: kg_version.to_owned(),
                forward,
                reverse,
                search,
            };
        }

        let changes = commit.change_set().changes();
        let updates_are_valid = updates.iter().all(|update| {
            changes
                .iter()
                .any(|change| change.target() == update.target())
        }) && changes.iter().all(|change| {
            updates
                .iter()
                .any(|update| update.target() == change.target())
        });
        if !updates_are_valid {
            let message = "index synchronization projections must cover every committed Change Set target and contain no unrelated targets";
            let forward = self.record_failure(KgIndexKind::Forward, message);
            let reverse = self.record_failure(KgIndexKind::Reverse, message);
            let search = self.record_failure(KgIndexKind::Search, message);
            return IndexSynchronizationReport {
                canonical_version: kg_version.to_owned(),
                forward,
                reverse,
                search,
            };
        }

        IndexSynchronizationReport {
            canonical_version: kg_version.to_owned(),
            forward: self.synchronize_forward(commit, updates),
            reverse: self.synchronize_reverse(commit, updates),
            search: self.synchronize_search(commit, updates),
        }
    }
}

const FORWARD_STATE_DOCUMENT: &str = "indexes/state-forward.json";
const REVERSE_STATE_DOCUMENT: &str = "indexes/state-reverse.json";
const SEARCH_STATE_DOCUMENT: &str = "indexes/state-search.json";

fn state_document(kind: KgIndexKind) -> &'static str {
    match kind {
        KgIndexKind::Forward => FORWARD_STATE_DOCUMENT,
        KgIndexKind::Reverse => REVERSE_STATE_DOCUMENT,
        KgIndexKind::Search => SEARCH_STATE_DOCUMENT,
    }
}

fn load_index_state(
    store: &FileDocumentStore,
    kind: KgIndexKind,
) -> Result<KgIndexState, IndexSynchronizationError> {
    let id = DocumentId::new(state_document(kind))
        .map_err(|error| IndexSynchronizationError::State(error.to_string()))?;
    let Some(document) = store.get(&id).map_err(IndexSynchronizationError::Storage)? else {
        return Ok(KgIndexState::uninitialized(kind));
    };
    let state: KgIndexState = serde_json::from_slice(document.as_bytes())
        .map_err(|error| IndexSynchronizationError::State(error.to_string()))?;
    if state.kind() != kind {
        return Err(IndexSynchronizationError::State(format!(
            "persisted index state kind does not match {}",
            state_document(kind)
        )));
    }
    Ok(state)
}

fn persist_index_state(store: &FileDocumentStore, state: &KgIndexState) -> Result<(), String> {
    let id = DocumentId::new(state_document(state.kind())).map_err(|error| error.to_string())?;
    let document = Document::new(serde_json::to_vec(state).map_err(|error| error.to_string())?);
    if store.contains(&id).map_err(|error| error.to_string())? {
        store
            .replace(&id, document)
            .map_err(|error| error.to_string())
    } else {
        store
            .create(id, document)
            .map_err(|error| error.to_string())
    }
}

/// Errors encountered while opening the physical synchronization coordinator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexSynchronizationError {
    Storage(crate::storage::DocumentStoreError),
    Forward(ForwardIndexError),
    Reverse(ReverseIndexError),
    Search(SearchIndexError),
    State(String),
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
            Self::State(error) => write!(formatter, "index state initialization failed: {error}"),
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
        let committed_version = commit.version().id().as_str().to_owned();
        drop(synchronizer);
        let reopened = KgIndexSynchronizer::open(&root).unwrap();
        for state in [
            reopened.forward_state(),
            reopened.reverse_state(),
            reopened.search_state(),
        ] {
            assert_eq!(state.queryability(), IndexQueryability::Queryable);
            assert_eq!(state.version().generation(), 1);
            assert_eq!(state.version().built_from_kg_version(), committed_version);
        }

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
        let failed_version = report
            .search()
            .state()
            .version()
            .built_from_kg_version()
            .to_owned();
        drop(synchronizer);
        let reopened = KgIndexSynchronizer::open(&root).unwrap();
        assert_eq!(
            reopened.search_state().queryability(),
            IndexQueryability::Stale
        );
        assert_eq!(
            reopened.search_state().version().built_from_kg_version(),
            failed_version
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn incomplete_change_projections_are_not_reported_as_synchronized() {
        use crate::identity::EntityId;
        use crate::storage::{
            Document, FileRepository, Repository, RepositoryIndexSynchronization, RepositoryRecord,
        };
        use nizaam_indexing::index::definition::TargetReferenceType;
        use nizaam_indexing::index::reference::ObjectReference;

        let root = root();
        let mut repository = FileRepository::open(&root).unwrap();
        let version = repository.current_version().unwrap();
        let a = crate::versioning::ChangeTarget::Entity(EntityId::new("a").unwrap());
        let b = crate::versioning::ChangeTarget::Entity(EntityId::new("b").unwrap());
        let mut changes = crate::versioning::ChangeSet::new(Some(version.id().clone()));
        changes.push(crate::versioning::Change::new(
            crate::versioning::ChangeKind::Created,
            a.clone(),
        ));
        changes.push(crate::versioning::Change::new(
            crate::versioning::ChangeKind::Created,
            b.clone(),
        ));
        let records = [
            RepositoryRecord::new(a.clone(), Document::from_slice(b"a")),
            RepositoryRecord::new(b.clone(), Document::from_slice(b"b")),
        ];
        let only_a = IndexSynchronizationChange::upsert(
            a.clone(),
            ObjectReference::new("kg", "a").unwrap(),
            TargetReferenceType::new("entity").unwrap(),
            Some("A"),
        );
        let mut synchronizer = KgIndexSynchronizer::open(&root).unwrap();
        let (_, report) = repository
            .commit_change_set_and_synchronize(&changes, &records, &[only_a], &mut synchronizer)
            .unwrap();
        assert!(!report.is_fully_synchronized());
        assert!(
            report
                .forward()
                .error()
                .unwrap()
                .contains("cover every committed")
        );
        assert_eq!(
            repository.current_state().unwrap().len(),
            2,
            "canonical commit remains intact"
        );
        assert_eq!(
            synchronizer
                .forward_state()
                .version()
                .built_from_kg_version(),
            "uninitialized"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
