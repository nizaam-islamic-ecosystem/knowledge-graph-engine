//! KG-owned forward index.
//!
//! The Phase 6 `ForwardIndexAccess` contract is preserved. Phase 7 adds the
//! first physical KG implementation: a small file-backed derived index which
//! maps source-owned `ObjectReference`s to deterministic `IndexAssignedId`s.
//!
//! `IndexAssignedId` remains Indexing-owned. The KG stores the opaque mapping
//! needed by its derived access path and never promotes it to KG identity.

use std::collections::BTreeMap;

use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::definition::TargetReferenceType;
use nizaam_indexing::index::reference::ObjectReference;
use serde_json::Value;

use crate::storage::{Document, DocumentId, DocumentStore, DocumentStoreError, FileDocumentStore};

use super::{IndexChangeAction, IndexSynchronizationChange};

/// Logical forward lookup boundary retained from Phase 6.
pub trait ForwardIndexAccess {
    /// Adapter-specific failure type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Resolves a source-owned object reference to its Indexing assigned id.
    fn lookup_forward(
        &self,
        object_reference: &ObjectReference,
    ) -> Result<Option<IndexAssignedId>, Self::Error>;
}

/// Executes one forward lookup through a supplied adapter.
pub fn lookup_forward<A>(
    adapter: &A,
    object_reference: &ObjectReference,
) -> Result<Option<IndexAssignedId>, A::Error>
where
    A: ForwardIndexAccess,
{
    adapter.lookup_forward(object_reference)
}

/// File-backed KG forward index.
#[derive(Debug)]
pub struct FileForwardIndex {
    store: FileDocumentStore,
    target_types: BTreeMap<String, String>,
}

impl FileForwardIndex {
    /// Opens the default forward index below `~/.nizaam` on the current OS.
    pub fn open_default() -> Result<Self, ForwardIndexError> {
        Self::open(FileDocumentStore::default_root().map_err(ForwardIndexError::Storage)?)
    }

    /// Opens the forward index below an explicit Nizaam storage root.
    pub fn open(root: impl Into<std::path::PathBuf>) -> Result<Self, ForwardIndexError> {
        let store = FileDocumentStore::new(root.into()).map_err(ForwardIndexError::Storage)?;
        let target_types = load_map(&store, FORWARD_DOCUMENT)?;
        Ok(Self {
            store,
            target_types,
        })
    }

    /// Inserts or replaces one forward mapping.
    pub fn insert(
        &mut self,
        object_reference: ObjectReference,
        target_type: TargetReferenceType,
    ) -> Result<IndexAssignedId, ForwardIndexError> {
        let key = reference_key(&object_reference);
        let target_type = target_type.as_str().to_owned();
        self.target_types.insert(key, target_type.clone());
        self.persist()?;
        let target_type = TargetReferenceType::new(target_type)
            .map_err(|error| ForwardIndexError::InvalidTargetType(error.to_string()))?;
        Ok(IndexAssignedId::generate(&target_type, &object_reference))
    }

    /// Removes a forward mapping if present.
    pub fn remove(
        &mut self,
        object_reference: &ObjectReference,
    ) -> Result<bool, ForwardIndexError> {
        let removed = self
            .target_types
            .remove(&reference_key(object_reference))
            .is_some();
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    /// Applies one explicit canonical-to-forward index synchronization change.
    pub fn synchronize(
        &mut self,
        change: &IndexSynchronizationChange,
    ) -> Result<(), ForwardIndexError> {
        match change.action() {
            IndexChangeAction::Upsert => {
                self.insert(change.reference().clone(), change.target_type().clone())?;
            }
            IndexChangeAction::Remove => {
                self.remove(change.reference())?;
            }
        }
        Ok(())
    }

    /// Rebuilds the derived mapping from canonical source references.
    pub fn rebuild<I>(&mut self, entries: I) -> Result<(), ForwardIndexError>
    where
        I: IntoIterator<Item = (ObjectReference, TargetReferenceType)>,
    {
        let mut next = BTreeMap::new();
        for (reference, target_type) in entries {
            next.insert(reference_key(&reference), target_type.as_str().to_owned());
        }
        self.target_types = next;
        self.persist()
    }

    fn persist(&mut self) -> Result<(), ForwardIndexError> {
        let bytes = serde_json::to_vec(&self.target_types)
            .map_err(|error| ForwardIndexError::Serialization(error.to_string()))?;
        let id =
            DocumentId::new(FORWARD_DOCUMENT).map_err(|_| ForwardIndexError::InvalidDocumentId)?;
        let document = Document::new(bytes);
        if self
            .store
            .contains(&id)
            .map_err(ForwardIndexError::Storage)?
        {
            self.store
                .replace(&id, document)
                .map_err(ForwardIndexError::Storage)
        } else {
            self.store
                .create(id, document)
                .map_err(ForwardIndexError::Storage)
        }
    }
}

impl ForwardIndexAccess for FileForwardIndex {
    type Error = ForwardIndexError;

    fn lookup_forward(
        &self,
        object_reference: &ObjectReference,
    ) -> Result<Option<IndexAssignedId>, Self::Error> {
        let Some(target_type) = self.target_types.get(&reference_key(object_reference)) else {
            return Ok(None);
        };
        let target_type = TargetReferenceType::new(target_type.clone())
            .map_err(|error| ForwardIndexError::InvalidTargetType(error.to_string()))?;
        Ok(Some(IndexAssignedId::generate(
            &target_type,
            object_reference,
        )))
    }
}

/// Errors produced by the physical forward index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForwardIndexError {
    /// Underlying document-store failure.
    Storage(DocumentStoreError),
    /// JSON serialization/deserialization failure.
    Serialization(String),
    /// Invalid physical document identifier.
    InvalidDocumentId,
    /// Invalid Indexing target-reference type.
    InvalidTargetType(String),
    /// The persisted document has the wrong JSON shape.
    CorruptDocument,
}

impl std::fmt::Display for ForwardIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "forward index storage failure: {error}"),
            Self::Serialization(error) => write!(f, "forward index serialization failure: {error}"),
            Self::InvalidDocumentId => f.write_str("invalid forward index document id"),
            Self::InvalidTargetType(error) => {
                write!(f, "invalid forward index target type: {error}")
            }
            Self::CorruptDocument => f.write_str("forward index document is corrupt"),
        }
    }
}

impl std::error::Error for ForwardIndexError {}

const FORWARD_DOCUMENT: &str = "indexes/forward.json";

fn reference_key(reference: &ObjectReference) -> String {
    format!(
        "{}\u{1f}{}",
        reference.source(),
        reference.object_reference()
    )
}

fn load_map(
    store: &FileDocumentStore,
    document_name: &str,
) -> Result<BTreeMap<String, String>, ForwardIndexError> {
    let id = DocumentId::new(document_name).map_err(|_| ForwardIndexError::InvalidDocumentId)?;
    let Some(document) = store.get(&id).map_err(ForwardIndexError::Storage)? else {
        return Ok(BTreeMap::new());
    };
    let value: Value = serde_json::from_slice(document.as_bytes())
        .map_err(|error| ForwardIndexError::Serialization(error.to_string()))?;
    serde_json::from_value(value).map_err(|_| ForwardIndexError::CorruptDocument)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-forward-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn physical_forward_lookup_survives_reopen() {
        let root = root();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let expected = IndexAssignedId::generate(&target_type, &reference);

        let mut index = FileForwardIndex::open(&root).unwrap();
        assert_eq!(
            index.insert(reference.clone(), target_type).unwrap(),
            expected
        );
        drop(index);

        let index = FileForwardIndex::open(&root).unwrap();
        assert_eq!(index.lookup_forward(&reference).unwrap(), Some(expected));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rebuild_replaces_derived_state() {
        let root = root();
        let a = ObjectReference::new("kg", "a").unwrap();
        let b = ObjectReference::new("kg", "b").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let mut index = FileForwardIndex::open(&root).unwrap();
        index.rebuild([(b.clone(), target_type.clone())]).unwrap();
        assert!(index.lookup_forward(&a).unwrap().is_none());
        assert!(index.lookup_forward(&b).unwrap().is_some());
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn synchronization_updates_and_removes_one_forward_entry() {
        let root = root();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type = TargetReferenceType::new("entity").unwrap();
        let target = crate::versioning::ChangeTarget::Entity(
            crate::identity::EntityId::new("entity-1").unwrap(),
        );
        let mut index = FileForwardIndex::open(&root).unwrap();

        let upsert = IndexSynchronizationChange::upsert(
            target.clone(),
            reference.clone(),
            target_type.clone(),
            Some("entity"),
        );
        index.synchronize(&upsert).unwrap();
        assert!(index.lookup_forward(&reference).unwrap().is_some());

        let remove = IndexSynchronizationChange::remove(target, reference.clone(), target_type);
        index.synchronize(&remove).unwrap();
        assert!(index.lookup_forward(&reference).unwrap().is_none());

        let _ = std::fs::remove_dir_all(root);
    }
}
