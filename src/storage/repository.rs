//! Semantic repository boundary and initial file-backed repository.
//!
//! The repository exposes semantic targets and logical KG state while keeping
//! the physical document provider behind [`DocumentStore`]. The initial
//! implementation stores one canonical state document, which gives the first
//! Phase 7 provider a simple atomic commit boundary without leaking filesystem
//! concepts into the semantic API.

use core::fmt;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::document_store::{Document, DocumentStore, DocumentStoreError, FileDocumentStore};
use super::snapshot::Snapshot;
use crate::versioning::{
    Change, ChangeKind, ChangeSet, ChangeSetId, ChangeTarget, KgVersion, KgVersionId,
};

const CURRENT_STATE_DOCUMENT: &str = "kg/current-state.json";
const STORAGE_SCHEMA_VERSION: u64 = 1;

/// One semantic KG record stored by the repository.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryRecord {
    target: ChangeTarget,
    document: Document,
}

impl RepositoryRecord {
    /// Creates a repository record for one semantic target.
    #[must_use]
    pub fn new(target: ChangeTarget, document: Document) -> Self {
        Self { target, document }
    }

    /// Returns the semantic target represented by this record.
    #[must_use]
    pub fn target(&self) -> &ChangeTarget {
        &self.target
    }

    /// Returns the opaque serialized semantic representation.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Consumes the record and returns its semantic target and document.
    #[must_use]
    pub fn into_parts(self) -> (ChangeTarget, Document) {
        (self.target, self.document)
    }
}

/// Repository-level failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepositoryError {
    /// The requested semantic object does not exist.
    NotFound(ChangeTarget),
    /// A semantic object already exists where creation was expected.
    AlreadyExists(ChangeTarget),
    /// The repository's current version differs from the transaction's base
    /// version.
    VersionConflict {
        /// Version expected by the transaction.
        expected: KgVersionId,
        /// Version currently published by the repository.
        actual: KgVersionId,
    },
    /// A Change Set does not satisfy the repository contract.
    InvalidChangeSet(String),
    /// A staged repository record does not match its Change Set target.
    TargetMismatch {
        /// Target declared by the Change Set.
        expected: ChangeTarget,
        /// Target supplied by the repository record.
        actual: ChangeTarget,
    },
    /// The persisted canonical state is malformed or cannot be decoded.
    PersistenceCorruption(String),
    /// A physical storage operation failed.
    Storage(DocumentStoreError),
    /// A snapshot could not be created or read.
    SnapshotUnavailable(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(target) => {
                write!(
                    formatter,
                    "semantic repository record was not found: {target:?}"
                )
            }
            Self::AlreadyExists(target) => {
                write!(
                    formatter,
                    "semantic repository record already exists: {target:?}"
                )
            }
            Self::VersionConflict { expected, actual } => {
                write!(
                    formatter,
                    "repository version conflict: expected {expected}, actual {actual}"
                )
            }
            Self::InvalidChangeSet(message) => {
                write!(formatter, "invalid Change Set: {message}")
            }
            Self::TargetMismatch { expected, actual } => {
                write!(
                    formatter,
                    "repository record target mismatch: expected {expected:?}, actual {actual:?}"
                )
            }
            Self::PersistenceCorruption(message) => {
                write!(formatter, "repository persistence is corrupt: {message}")
            }
            Self::Storage(error) => write!(formatter, "repository storage failure: {error}"),
            Self::SnapshotUnavailable(message) => {
                write!(formatter, "repository snapshot unavailable: {message}")
            }
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<DocumentStoreError> for RepositoryError {
    fn from(error: DocumentStoreError) -> Self {
        Self::Storage(error)
    }
}

/// Semantic repository contract.
pub trait Repository {
    /// Reads one semantic record from the current canonical state.
    fn read(&self, target: &ChangeTarget) -> Result<Option<RepositoryRecord>, RepositoryError>;

    /// Reads the complete current canonical state.
    fn current_state(&self) -> Result<Vec<RepositoryRecord>, RepositoryError>;

    /// Returns the currently published logical KG version.
    fn current_version(&self) -> Result<KgVersion, RepositoryError>;

    /// Atomically commits a validated Change Set and its associated records.
    fn commit_change_set(
        &mut self,
        change_set: &ChangeSet,
        records: &[RepositoryRecord],
    ) -> Result<KgVersion, RepositoryError>;

    /// Creates a coherent immutable snapshot of the current canonical state.
    fn snapshot(&self) -> Result<Snapshot, RepositoryError>;
}

/// One canonical Change Set that has already been durably committed.
///
/// This publication envelope is intentionally separate from derived-index
/// synchronization. It lets a KG-owned index consumer observe the exact
/// committed Change Set and resulting KG version without becoming part of the
/// repository's canonical storage transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedChangeSet {
    version: KgVersion,
    change_set: ChangeSet,
    records: Vec<RepositoryRecord>,
}

impl CommittedChangeSet {
    fn new(version: KgVersion, change_set: ChangeSet, records: &[RepositoryRecord]) -> Self {
        Self {
            version,
            change_set,
            records: records.to_vec(),
        }
    }

    /// Returns the newly committed KG version.
    #[must_use]
    pub fn version(&self) -> &KgVersion {
        &self.version
    }

    /// Returns the semantic Change Set that produced the committed version.
    #[must_use]
    pub fn change_set(&self) -> &ChangeSet {
        &self.change_set
    }

    /// Returns the repository records supplied with the committed Change Set.
    #[must_use]
    pub fn records(&self) -> &[RepositoryRecord] {
        &self.records
    }
}

/// Provider-neutral synchronization callback invoked after a canonical commit.
///
/// The repository does not know the concrete derived-index update type. The
/// caller supplies that projection explicitly, keeping semantic storage
/// independent from physical index representation.
pub trait RepositoryIndexSynchronizer<Update> {
    /// Report produced by one synchronization attempt.
    type Report;

    /// Applies derived-index synchronization after canonical publication.
    ///
    /// Implementations must treat the supplied commit as already durable.
    /// Synchronization failure therefore cannot rewrite canonical repository
    /// state.
    fn synchronize(&mut self, commit: &CommittedChangeSet, updates: &[Update]) -> Self::Report;
}

/// Extension boundary connecting canonical repository commits to derived-index
/// synchronization without coupling the storage module to a concrete index
/// implementation.
pub trait RepositoryIndexSynchronization: Repository {
    /// Commits canonical state first, then publishes the committed Change Set
    /// to the supplied derived-index synchronizer.
    fn commit_change_set_and_synchronize<Update, S>(
        &mut self,
        change_set: &ChangeSet,
        records: &[RepositoryRecord],
        updates: &[Update],
        synchronizer: &mut S,
    ) -> Result<(CommittedChangeSet, S::Report), RepositoryError>
    where
        S: RepositoryIndexSynchronizer<Update>;
}

impl<R> RepositoryIndexSynchronization for R
where
    R: Repository + ?Sized,
{
    fn commit_change_set_and_synchronize<Update, S>(
        &mut self,
        change_set: &ChangeSet,
        records: &[RepositoryRecord],
        updates: &[Update],
        synchronizer: &mut S,
    ) -> Result<(CommittedChangeSet, S::Report), RepositoryError>
    where
        S: RepositoryIndexSynchronizer<Update>,
    {
        let version = self.commit_change_set(change_set, records)?;
        let commit = CommittedChangeSet::new(version, change_set.clone(), records);
        let report = synchronizer.synchronize(&commit, updates);
        Ok((commit, report))
    }
}

/// Marker trait for repository implementations backed by the document-store
/// abstraction.
pub trait DocumentBackedRepository: Repository {
    /// Returns access to the underlying document-store boundary.
    fn document_store(&self) -> &dyn DocumentStore;
}

/// Initial file-backed semantic repository.
#[derive(Debug)]
pub struct FileRepository {
    store: FileDocumentStore,
    version: KgVersion,
    records: Vec<RepositoryRecord>,
    history: Vec<ChangeSet>,
}

impl FileRepository {
    /// Opens or initializes the default Nizaam repository under the current
    /// operating-system user home (`~/.nizaam`).
    pub fn open_default() -> Result<Self, RepositoryError> {
        Self::open(FileDocumentStore::default_root()?)
    }

    /// Opens an existing file-backed repository or initializes an empty one.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, RepositoryError> {
        let store = FileDocumentStore::new(root)?;
        // Serialize initialization with commits and other first-time openers.
        // Always recheck the state file only after owning the same advisory lock
        // used by commit_change_set. A concurrent opener may observe the lock
        // briefly; wait for that opener to finish rather than surfacing a
        // transient initialization conflict to the caller.
        let _lock = loop {
            match store.acquire_exclusive_lock() {
                Ok(lock) => break lock,
                Err(DocumentStoreError::Conflict(_)) => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Err(error) => return Err(RepositoryError::Storage(error)),
            }
        };
        let state_id = current_state_id()?;

        if let Some(document) = store.get(&state_id)? {
            let (version, records, history) = decode_state(document.as_bytes())?;
            drop(_lock);
            Ok(Self {
                store,
                version,
                records,
                history,
            })
        } else {
            let repository = Self {
                store,
                version: KgVersion::initial(),
                records: Vec::new(),
                history: Vec::new(),
            };
            repository.persist_current_state()?;
            drop(_lock);
            Ok(repository)
        }
    }

    /// Returns the provider root used by this repository.
    pub fn root(&self) -> &Path {
        self.store.root()
    }

    /// Returns historical Change Sets in commit order.
    pub fn change_history(&self) -> &[ChangeSet] {
        &self.history
    }

    fn reload_from_disk(&mut self) -> Result<(), RepositoryError> {
        let state_id = current_state_id()?;
        let document = self.store.get(&state_id)?.ok_or_else(|| {
            RepositoryError::PersistenceCorruption("current state document is missing".to_owned())
        })?;
        let (version, records, history) = decode_state(document.as_bytes())?;
        self.version = version;
        self.records = records;
        self.history = history;
        Ok(())
    }

    fn persist_current_state(&self) -> Result<(), RepositoryError> {
        let state_id = current_state_id()?;
        let document = Document::new(encode_state(&self.version, &self.records, &self.history)?);

        if self.store.contains(&state_id)? {
            self.store.replace(&state_id, document)?;
        } else {
            self.store.create(state_id, document)?;
        }

        Ok(())
    }
}

impl Repository for FileRepository {
    fn read(&self, target: &ChangeTarget) -> Result<Option<RepositoryRecord>, RepositoryError> {
        Ok(self
            .records
            .iter()
            .find(|record| record.target() == target)
            .cloned())
    }

    fn current_state(&self) -> Result<Vec<RepositoryRecord>, RepositoryError> {
        Ok(self.records.clone())
    }

    fn current_version(&self) -> Result<KgVersion, RepositoryError> {
        Ok(self.version.clone())
    }

    fn commit_change_set(
        &mut self,
        change_set: &ChangeSet,
        records: &[RepositoryRecord],
    ) -> Result<KgVersion, RepositoryError> {
        if change_set.is_empty() {
            return Err(RepositoryError::InvalidChangeSet(
                "a committed Change Set must contain at least one semantic change".to_owned(),
            ));
        }

        let expected = change_set.base_version().ok_or_else(|| {
            RepositoryError::InvalidChangeSet(
                "a committed Change Set must reference a base KG version".to_owned(),
            )
        })?;

        let lock = self.store.acquire_exclusive_lock()?;
        self.reload_from_disk()?;

        if self.version.id() != expected {
            drop(lock);
            return Err(RepositoryError::VersionConflict {
                expected: expected.clone(),
                actual: self.version.id().clone(),
            });
        }

        for record in records {
            if !change_set
                .changes()
                .iter()
                .any(|change| change.target() == record.target())
            {
                drop(lock);
                return Err(RepositoryError::TargetMismatch {
                    expected: change_set
                        .changes()
                        .first()
                        .map(|change| change.target().clone())
                        .unwrap_or_else(|| record.target().clone()),
                    actual: record.target().clone(),
                });
            }
        }

        let mut next_records = self.records.clone();
        for record in records {
            if let Some(existing) = next_records
                .iter_mut()
                .find(|existing| existing.target() == record.target())
            {
                *existing = record.clone();
            } else {
                next_records.push(record.clone());
            }
        }

        let next_version = KgVersion::next(&self.version, change_set.id().clone());
        let mut next_history = self.history.clone();
        next_history.push(change_set.clone());

        let document = Document::new(encode_state(&next_version, &next_records, &next_history)?);
        let state_id = current_state_id()?;
        self.store.replace(&state_id, document)?;

        self.version = next_version.clone();
        self.records = next_records;
        self.history = next_history;
        drop(lock);

        Ok(next_version)
    }

    fn snapshot(&self) -> Result<Snapshot, RepositoryError> {
        Snapshot::new(self.version.clone(), self.records.clone())
            .map_err(|error| RepositoryError::SnapshotUnavailable(error.to_string()))
    }
}

impl DocumentBackedRepository for FileRepository {
    fn document_store(&self) -> &dyn DocumentStore {
        &self.store
    }
}

fn current_state_id() -> Result<crate::storage::document_store::DocumentId, RepositoryError> {
    crate::storage::document_store::DocumentId::new(CURRENT_STATE_DOCUMENT)
        .map_err(RepositoryError::Storage)
}

fn encode_state(
    version: &KgVersion,
    records: &[RepositoryRecord],
    history: &[ChangeSet],
) -> Result<Vec<u8>, RepositoryError> {
    let value = json!({
        "schema_version": STORAGE_SCHEMA_VERSION,
        "version": encode_version(version),
        "records": records.iter().map(encode_record).collect::<Vec<_>>(),
        "history": history.iter().map(encode_change_set).collect::<Vec<_>>(),
    });

    serde_json::to_vec(&value)
        .map_err(|error| RepositoryError::PersistenceCorruption(error.to_string()))
}

fn decode_state(
    bytes: &[u8],
) -> Result<(KgVersion, Vec<RepositoryRecord>, Vec<ChangeSet>), RepositoryError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| RepositoryError::PersistenceCorruption(error.to_string()))?;

    let schema_version = required_u64(&value, "schema_version")?;
    if schema_version != STORAGE_SCHEMA_VERSION {
        return Err(RepositoryError::PersistenceCorruption(format!(
            "unsupported storage schema version {schema_version}"
        )));
    }

    let version_value = Value::Object(required_object(&value, "version")?.clone());
    let version = decode_version(&version_value)?;
    let records = required_array(&value, "records")?
        .iter()
        .map(decode_record)
        .collect::<Result<Vec<_>, _>>()?;
    let history = required_array(&value, "history")?
        .iter()
        .map(decode_change_set)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((version, records, history))
}

fn encode_version(version: &KgVersion) -> Value {
    json!({
        "id": version.id().as_str(),
        "sequence": version.sequence(),
        "parent": version.parent().map(|id| id.as_str()),
        "change_set": version.change_set().map(|id| id.as_str()),
    })
}

fn decode_version(value: &Value) -> Result<KgVersion, RepositoryError> {
    let id = KgVersionId::new(required_string(value, "id")?)
        .map_err(|_| corrupt("invalid KG version identity"))?;
    let sequence = required_u64(value, "sequence")?;
    let parent = optional_string(value, "parent")?
        .map(|value| {
            KgVersionId::new(value).map_err(|_| corrupt("invalid parent version identity"))
        })
        .transpose()?;
    let change_set = optional_string(value, "change_set")?
        .map(|value| ChangeSetId::new(value).map_err(|_| corrupt("invalid Change Set identity")))
        .transpose()?;

    Ok(KgVersion::from_parts(id, sequence, parent, change_set))
}

fn encode_record(record: &RepositoryRecord) -> Value {
    json!({
        "kind": record.target().kind(),
        "id": record.target().id(),
        "bytes": record.document().as_bytes(),
    })
}

fn decode_record(value: &Value) -> Result<RepositoryRecord, RepositoryError> {
    let target = ChangeTarget::from_kind_and_id(
        required_string(value, "kind")?,
        required_string(value, "id")?,
    )
    .map_err(corrupt)?;

    let bytes = required_array(value, "bytes")?
        .iter()
        .map(|byte| {
            let value = byte
                .as_u64()
                .ok_or_else(|| corrupt("record byte must be an unsigned integer"))?;
            u8::try_from(value).map_err(|_| corrupt("record byte is outside 0..=255"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RepositoryRecord::new(target, Document::new(bytes)))
}

fn encode_change_set(change_set: &ChangeSet) -> Value {
    json!({
        "id": change_set.id().as_str(),
        "base_version": change_set.base_version().map(|id| id.as_str()),
        "changes": change_set.changes().iter().map(|change| json!({
            "kind": change.kind().as_str(),
            "target_kind": change.target().kind(),
            "target_id": change.target().id(),
        })).collect::<Vec<_>>(),
    })
}

fn decode_change_set(value: &Value) -> Result<ChangeSet, RepositoryError> {
    let id = ChangeSetId::new(required_string(value, "id")?)
        .map_err(|_| corrupt("invalid Change Set identity"))?;
    let base_version = optional_string(value, "base_version")?
        .map(|value| {
            KgVersionId::new(value).map_err(|_| corrupt("invalid Change Set base version"))
        })
        .transpose()?;
    let changes = required_array(value, "changes")?
        .iter()
        .map(|change| {
            let kind = ChangeKind::from_str(required_string(change, "kind")?)
                .ok_or_else(|| corrupt("unknown Change Set change kind"))?;
            let target = ChangeTarget::from_kind_and_id(
                required_string(change, "target_kind")?,
                required_string(change, "target_id")?,
            )
            .map_err(corrupt)?;
            Ok(Change::new(kind, target))
        })
        .collect::<Result<Vec<_>, RepositoryError>>()?;

    Ok(ChangeSet::from_parts(id, base_version, changes))
}

fn required_object<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, RepositoryError> {
    value
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| corrupt(format!("missing or invalid object field '{field}'")))
}

fn required_array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, RepositoryError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| corrupt(format!("missing or invalid array field '{field}'")))
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, RepositoryError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| corrupt(format!("missing or invalid string field '{field}'")))
}

fn optional_string<'a>(value: &'a Value, field: &str) -> Result<Option<&'a str>, RepositoryError> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(corrupt(format!("invalid optional string field '{field}'"))),
    }
}

fn required_u64(value: &Value, field: &str) -> Result<u64, RepositoryError> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| corrupt(format!("missing or invalid integer field '{field}'")))
}

fn corrupt(message: impl Into<String>) -> RepositoryError {
    RepositoryError::PersistenceCorruption(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-repository-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos()
        ))
    }

    fn target() -> ChangeTarget {
        ChangeTarget::Entity(EntityId::new("entity-1").expect("valid entity"))
    }

    #[test]
    fn concurrent_first_open_does_not_erase_a_committed_record() {
        let root = std::env::temp_dir().join(format!(
            "nizaam-repository-open-race-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let first_root = root.clone();
        let second_root = root.clone();
        let first =
            std::thread::spawn(move || FileRepository::open(first_root).expect("first open"));
        let second =
            std::thread::spawn(move || FileRepository::open(second_root).expect("second open"));
        let mut a = first.join().unwrap();
        let _b = second.join().unwrap();
        let target = ChangeTarget::Entity(crate::identity::EntityId::new("race-entity").unwrap());
        let mut changes = ChangeSet::new(Some(a.current_version().unwrap().id().clone()));
        changes.push(Change::new(ChangeKind::Created, target.clone()));
        a.commit_change_set(
            &changes,
            &[RepositoryRecord::new(
                target,
                Document::from_slice(b"saved"),
            )],
        )
        .unwrap();
        drop(a);
        let reopened = FileRepository::open(&root).unwrap();
        assert_eq!(reopened.current_state().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_openers_preserve_records_committed_after_initialization() {
        let root = temp_root();
        let mut repository = FileRepository::open(&root).unwrap();
        let version = repository.current_version().unwrap();
        let target = ChangeTarget::Entity(EntityId::new("concurrent-commit").unwrap());
        let mut change_set = ChangeSet::new(Some(version.id().clone()));
        change_set.push(Change::new(ChangeKind::Created, target.clone()));
        repository
            .commit_change_set(
                &change_set,
                &[RepositoryRecord::new(
                    target,
                    Document::from_slice(b"durable"),
                )],
            )
            .unwrap();
        drop(repository);

        let start = std::sync::Arc::new(std::sync::Barrier::new(3));
        let mut threads = Vec::new();
        for _ in 0..2 {
            let root = root.clone();
            let start = start.clone();
            threads.push(std::thread::spawn(move || {
                start.wait();
                FileRepository::open(root).unwrap().current_state().unwrap()
            }));
        }
        start.wait();
        for thread in threads {
            assert_eq!(thread.join().unwrap().len(), 1);
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn file_repository_initializes_and_reopens_current_state() {
        let root = temp_root();
        let repository = FileRepository::open(&root).expect("repository should open");
        let version = repository.current_version().expect("version should exist");
        drop(repository);

        let reopened = FileRepository::open(&root).expect("repository should reopen");
        assert_eq!(
            reopened.current_version().expect("version should exist"),
            version
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
    fn persisted_state_round_trips_semantic_records() {
        let root = temp_root();
        let mut repository = FileRepository::open(&root).expect("repository should open");
        let target = target();
        let mut change_set = ChangeSet::new(Some(
            repository
                .current_version()
                .expect("version should exist")
                .id()
                .clone(),
        ));
        change_set.push(Change::new(ChangeKind::Created, target.clone()));

        repository
            .commit_change_set(
                &change_set,
                &[RepositoryRecord::new(
                    target.clone(),
                    Document::from_slice(b"entity"),
                )],
            )
            .expect("commit should succeed");
        drop(repository);

        let reopened = FileRepository::open(&root).expect("repository should reopen");
        assert_eq!(
            reopened
                .read(&target)
                .expect("read should succeed")
                .unwrap()
                .document(),
            &Document::from_slice(b"entity")
        );
        assert_eq!(reopened.change_history().len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn committed_change_set_is_published_only_after_canonical_commit() {
        #[derive(Default)]
        struct RecordingSynchronizer {
            seen_version: Option<String>,
            seen_changes: usize,
        }

        impl RepositoryIndexSynchronizer<()> for RecordingSynchronizer {
            type Report = usize;

            fn synchronize(&mut self, commit: &CommittedChangeSet, updates: &[()]) -> Self::Report {
                self.seen_version = Some(commit.version().id().as_str().to_owned());
                self.seen_changes = commit.change_set().len();
                updates.len()
            }
        }

        let root = temp_root();
        let mut repository = FileRepository::open(&root).expect("repository should open");
        let target = target();
        let mut change_set = ChangeSet::new(Some(
            repository
                .current_version()
                .expect("version should exist")
                .id()
                .clone(),
        ));
        change_set.push(Change::new(ChangeKind::Created, target.clone()));

        let mut synchronizer = RecordingSynchronizer::default();
        let (commit, report) = repository
            .commit_change_set_and_synchronize(
                &change_set,
                &[RepositoryRecord::new(
                    target,
                    Document::from_slice(b"entity"),
                )],
                &[()],
                &mut synchronizer,
            )
            .expect("canonical commit should succeed");

        assert_eq!(report, 1);
        assert_eq!(commit.version(), &repository.current_version().unwrap());
        assert_eq!(commit.change_set(), &change_set);
        assert_eq!(commit.records().len(), 1);
        assert_eq!(
            synchronizer.seen_version.as_deref(),
            Some(commit.version().id().as_str())
        );
        assert_eq!(synchronizer.seen_changes, 1);

        let _ = std::fs::remove_dir_all(root);
    }
}
