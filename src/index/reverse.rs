//! KG-owned reverse index.
//!
//! The Phase 6 reverse adapter remains available. Phase 7 adds a file-backed
//! derived mapping from the opaque Indexing assignment identity to the
//! source-owned `ObjectReference`.

use std::collections::BTreeMap;

use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::reference::ObjectReference;
use serde_json::Value;

use crate::storage::{Document, DocumentId, DocumentStore, DocumentStoreError, FileDocumentStore};

use super::{IndexChangeAction, IndexSynchronizationChange};

/// Logical reverse lookup boundary retained from Phase 6.
pub trait ReverseIndexAccess {
    /// Adapter-specific failure type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Resolves an Indexing assigned id to its source-owned object reference.
    fn lookup_reverse(
        &self,
        assigned_id: &IndexAssignedId,
    ) -> Result<Option<ObjectReference>, Self::Error>;
}

/// Executes one reverse lookup through a supplied adapter.
pub fn lookup_reverse<A>(
    adapter: &A,
    assigned_id: &IndexAssignedId,
) -> Result<Option<ObjectReference>, A::Error>
where
    A: ReverseIndexAccess,
{
    adapter.lookup_reverse(assigned_id)
}

/// File-backed KG reverse index.
#[derive(Debug)]
pub struct FileReverseIndex {
    store: FileDocumentStore,
    entries: BTreeMap<String, StoredReference>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct StoredReference {
    source: String,
    object_reference: String,
}

impl FileReverseIndex {
    /// Opens the default reverse index below the platform user's `.nizaam` root.
    pub fn open_default() -> Result<Self, ReverseIndexError> {
        Self::open(FileDocumentStore::default_root().map_err(ReverseIndexError::Storage)?)
    }

    /// Opens the reverse index below an explicit root.
    pub fn open(root: impl Into<std::path::PathBuf>) -> Result<Self, ReverseIndexError> {
        let store = FileDocumentStore::new(root.into()).map_err(ReverseIndexError::Storage)?;
        let entries = load_entries(&store)?;
        Ok(Self { store, entries })
    }

    /// Inserts or replaces one reverse mapping.
    pub fn insert(
        &mut self,
        assigned_id: IndexAssignedId,
        object_reference: ObjectReference,
    ) -> Result<(), ReverseIndexError> {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(ReverseIndexError::Storage)?;
        let mut next = load_entries(&self.store)?;
        next.insert(
            assigned_id.to_string(),
            StoredReference {
                source: object_reference.source().to_owned(),
                object_reference: object_reference.object_reference().to_owned(),
            },
        );
        if let Err(error) = self.persist(&next) {
            let _ = self.reload_from_disk();
            return Err(error);
        }
        self.entries = next;
        Ok(())
    }

    /// Removes a reverse mapping if present.
    pub fn remove(&mut self, assigned_id: &IndexAssignedId) -> Result<bool, ReverseIndexError> {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(ReverseIndexError::Storage)?;
        let key = assigned_id.to_string();
        let mut next = load_entries(&self.store)?;
        if !next.contains_key(&key) {
            self.entries = next;
            return Ok(false);
        }
        next.remove(&key);
        if let Err(error) = self.persist(&next) {
            let _ = self.reload_from_disk();
            return Err(error);
        }
        self.entries = next;
        Ok(true)
    }

    /// Applies one explicit canonical-to-reverse index synchronization change.
    pub fn synchronize(
        &mut self,
        change: &IndexSynchronizationChange,
    ) -> Result<(), ReverseIndexError> {
        self.synchronize_batch(std::slice::from_ref(change))
    }

    /// Applies a collection of changes and persists the resulting map once.
    pub fn synchronize_batch(
        &mut self,
        changes: &[IndexSynchronizationChange],
    ) -> Result<(), ReverseIndexError> {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(ReverseIndexError::Storage)?;
        self.synchronize_batch_locked(changes)
    }

    pub(crate) fn synchronize_batch_locked(
        &mut self,
        changes: &[IndexSynchronizationChange],
    ) -> Result<(), ReverseIndexError> {
        let mut next = load_entries(&self.store)?;
        for change in changes {
            let assigned_id =
                IndexAssignedId::generate(change.target_type(), change.reference()).to_string();
            match change.action() {
                IndexChangeAction::Upsert => {
                    next.insert(
                        assigned_id,
                        StoredReference {
                            source: change.reference().source().to_owned(),
                            object_reference: change.reference().object_reference().to_owned(),
                        },
                    );
                }
                IndexChangeAction::Remove => {
                    next.remove(&assigned_id);
                }
            }
        }
        if let Err(error) = self.persist(&next) {
            let _ = self.reload_from_disk();
            return Err(error);
        }
        self.entries = next;
        Ok(())
    }

    pub(crate) fn reload_from_disk(&mut self) -> Result<(), ReverseIndexError> {
        self.entries = load_entries(&self.store)?;
        Ok(())
    }

    /// Rebuilds the reverse index from canonical references and assigned ids.
    pub fn rebuild<I>(&mut self, entries: I) -> Result<(), ReverseIndexError>
    where
        I: IntoIterator<Item = (IndexAssignedId, ObjectReference)>,
    {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(ReverseIndexError::Storage)?;
        let next = entries
            .into_iter()
            .map(|(assigned_id, reference)| {
                (
                    assigned_id.to_string(),
                    StoredReference {
                        source: reference.source().to_owned(),
                        object_reference: reference.object_reference().to_owned(),
                    },
                )
            })
            .collect();
        self.persist(&next)?;
        self.entries = next;
        Ok(())
    }

    fn persist(
        &self,
        entries: &BTreeMap<String, StoredReference>,
    ) -> Result<(), ReverseIndexError> {
        let bytes = serde_json::to_vec(entries)
            .map_err(|error| ReverseIndexError::Serialization(error.to_string()))?;
        let id =
            DocumentId::new(REVERSE_DOCUMENT).map_err(|_| ReverseIndexError::InvalidDocumentId)?;
        let document = Document::new(bytes);
        if self
            .store
            .contains(&id)
            .map_err(ReverseIndexError::Storage)?
        {
            self.store
                .replace(&id, document)
                .map_err(ReverseIndexError::Storage)
        } else {
            self.store
                .create(id, document)
                .map_err(ReverseIndexError::Storage)
        }
    }
}

impl ReverseIndexAccess for FileReverseIndex {
    type Error = ReverseIndexError;

    fn lookup_reverse(
        &self,
        assigned_id: &IndexAssignedId,
    ) -> Result<Option<ObjectReference>, Self::Error> {
        let Some(reference) = self.entries.get(&assigned_id.to_string()) else {
            return Ok(None);
        };
        ObjectReference::new(reference.source.clone(), reference.object_reference.clone())
            .map(Some)
            .map_err(|error| ReverseIndexError::InvalidReference(error.to_string()))
    }
}

/// Errors produced by the physical reverse index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReverseIndexError {
    /// Underlying document-store failure.
    Storage(DocumentStoreError),
    /// JSON serialization/deserialization failure.
    Serialization(String),
    /// Invalid physical document identifier.
    InvalidDocumentId,
    /// Invalid persisted object reference.
    InvalidReference(String),
    /// Persisted JSON has the wrong structure.
    CorruptDocument,
}

impl std::fmt::Display for ReverseIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "reverse index storage failure: {error}"),
            Self::Serialization(error) => write!(f, "reverse index serialization failure: {error}"),
            Self::InvalidDocumentId => f.write_str("invalid reverse index document id"),
            Self::InvalidReference(error) => write!(f, "invalid reverse index reference: {error}"),
            Self::CorruptDocument => f.write_str("reverse index document is corrupt"),
        }
    }
}

impl std::error::Error for ReverseIndexError {}

const REVERSE_DOCUMENT: &str = "indexes/reverse.json";

fn load_entries(
    store: &FileDocumentStore,
) -> Result<BTreeMap<String, StoredReference>, ReverseIndexError> {
    let id = DocumentId::new(REVERSE_DOCUMENT).map_err(|_| ReverseIndexError::InvalidDocumentId)?;
    let Some(document) = store.get(&id).map_err(ReverseIndexError::Storage)? else {
        return Ok(BTreeMap::new());
    };
    let value: Value = serde_json::from_slice(document.as_bytes())
        .map_err(|error| ReverseIndexError::Serialization(error.to_string()))?;
    serde_json::from_value(value).map_err(|_| ReverseIndexError::CorruptDocument)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nizaam_indexing::index::definition::TargetReferenceType;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-reverse-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn synchronization_updates_and_removes_one_reverse_entry() {
        let root = root();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let target = crate::versioning::ChangeTarget::Entity(
            crate::identity::EntityId::new("entity-1").unwrap(),
        );
        let assigned_id = IndexAssignedId::generate(&target_type, &reference);
        let mut index = FileReverseIndex::open(&root).unwrap();
        let upsert = IndexSynchronizationChange::upsert(
            target.clone(),
            reference.clone(),
            target_type.clone(),
            Some("entity"),
        );
        index.synchronize(&upsert).unwrap();
        assert_eq!(
            index.lookup_reverse(&assigned_id).unwrap(),
            Some(reference.clone())
        );
        let remove = IndexSynchronizationChange::remove(target, reference.clone(), target_type);
        index.synchronize(&remove).unwrap();
        assert!(index.lookup_reverse(&assigned_id).unwrap().is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn separate_open_handles_preserve_each_others_updates() {
        let root = root();
        let a = ObjectReference::new("kg", "a").unwrap();
        let b = ObjectReference::new("kg", "b").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let assigned_a = IndexAssignedId::generate(&target_type, &a);
        let assigned_b = IndexAssignedId::generate(&target_type, &b);
        let mut first = FileReverseIndex::open(&root).unwrap();
        let mut second = FileReverseIndex::open(&root).unwrap();
        first.insert(assigned_a.clone(), a.clone()).unwrap();
        second.insert(assigned_b.clone(), b.clone()).unwrap();
        let reopened = FileReverseIndex::open(&root).unwrap();
        assert_eq!(reopened.lookup_reverse(&assigned_a).unwrap(), Some(a));
        assert_eq!(reopened.lookup_reverse(&assigned_b).unwrap(), Some(b));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn physical_reverse_lookup_survives_reopen() {
        let root = root();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let assigned_id = IndexAssignedId::generate(&target_type, &reference);

        let mut index = FileReverseIndex::open(&root).unwrap();
        index
            .insert(assigned_id.clone(), reference.clone())
            .unwrap();
        drop(index);

        let index = FileReverseIndex::open(&root).unwrap();
        assert_eq!(index.lookup_reverse(&assigned_id).unwrap(), Some(reference));
        let _ = std::fs::remove_dir_all(root);
    }
}
