//! KG-owned search index plus the Phase 6 Indexing retrieval compatibility adapter.
//!
//! The legacy `IndexSearchAccess` contract is preserved so Phase 6 consumers do
//! not change. Phase 7 adds a separate KG-owned lexical search index; it does
//! not delegate KG search ownership to Nizaam Indexing.

use std::collections::{BTreeMap, BTreeSet};

use nizaam_core::operation::OperationContext;
use nizaam_indexing::index::reference::ObjectReference;
use nizaam_indexing::query::planner::RetrievalPlan;
use nizaam_indexing::query::result::QueryResult;
use nizaam_indexing::query::retrieval::{self, ProviderRetriever, RetrievalError};

use crate::storage::{Document, DocumentId, DocumentStore, DocumentStoreError, FileDocumentStore};

use super::{IndexChangeAction, IndexSynchronizationChange};

/// Phase 6 logical retrieval adapter retained for compatibility.
pub trait IndexSearchAccess {
    type Error: std::error::Error + Send + Sync + 'static;
    fn search(
        &self,
        plan: &RetrievalPlan,
        context: &OperationContext,
    ) -> Result<QueryResult, Self::Error>;
}

impl<P> IndexSearchAccess for P
where
    P: ProviderRetriever,
{
    type Error = RetrievalError<P::Error>;

    fn search(
        &self,
        plan: &RetrievalPlan,
        context: &OperationContext,
    ) -> Result<QueryResult, Self::Error> {
        retrieval::execute(plan, self, context)
    }
}

pub fn search<A>(
    adapter: &A,
    plan: &RetrievalPlan,
    context: &OperationContext,
) -> Result<QueryResult, A::Error>
where
    A: IndexSearchAccess,
{
    adapter.search(plan, context)
}

/// One result from the KG-owned lexical index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KgSearchHit {
    reference: ObjectReference,
    score: u32,
}
impl KgSearchHit {
    #[must_use]
    pub fn reference(&self) -> &ObjectReference {
        &self.reference
    }
    #[must_use]
    pub const fn score(&self) -> u32 {
        self.score
    }
}

/// File-backed KG-owned lexical search index.
#[derive(Debug)]
pub struct FileSearchIndex {
    store: FileDocumentStore,
    terms: BTreeMap<String, BTreeSet<String>>,
    references: BTreeMap<String, StoredReference>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct StoredReference {
    source: String,
    object_reference: String,
}

impl FileSearchIndex {
    /// Opens the default KG search index below the platform user's `.nizaam` root.
    pub fn open_default() -> Result<Self, SearchIndexError> {
        Self::open(FileDocumentStore::default_root().map_err(SearchIndexError::Storage)?)
    }

    /// Opens the search index below an explicit root.
    pub fn open(root: impl Into<std::path::PathBuf>) -> Result<Self, SearchIndexError> {
        let store = FileDocumentStore::new(root.into()).map_err(SearchIndexError::Storage)?;
        let id = DocumentId::new(SEARCH_INDEX_DOCUMENT)
            .map_err(|_| SearchIndexError::InvalidDocumentId)?;
        let (terms, references) = if let Some(document) =
            store.get(&id).map_err(SearchIndexError::Storage)?
        {
            let persisted: PersistedSearchIndex = serde_json::from_slice(document.as_bytes())
                .map_err(|error| SearchIndexError::Serialization(error.to_string()))?;
            if persisted.terms.values().any(|keys| {
                keys.iter()
                    .any(|key| !persisted.references.contains_key(key))
            }) {
                return Err(SearchIndexError::CorruptDocument);
            }
            (persisted.terms, persisted.references)
        } else {
            (
                load(&store, SEARCH_DOCUMENT)?,
                load(&store, REFERENCES_DOCUMENT)?,
            )
        };
        Ok(Self {
            store,
            terms,
            references,
        })
    }

    /// Indexes one source-owned reference and its searchable text.
    pub fn upsert(
        &mut self,
        reference: ObjectReference,
        text: &str,
    ) -> Result<(), SearchIndexError> {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(SearchIndexError::Storage)?;
        self.reload_from_disk()?;
        let mut next_terms = self.terms.clone();
        let mut next_references = self.references.clone();
        let key = reference_key(&reference);
        remove_terms_for_reference(&mut next_terms, &key);
        next_references.insert(
            key.clone(),
            StoredReference {
                source: reference.source().to_owned(),
                object_reference: reference.object_reference().to_owned(),
            },
        );
        for term in tokenize(text) {
            next_terms.entry(term).or_default().insert(key.clone());
        }
        if let Err(error) = self.persist(&next_terms, &next_references) {
            let _ = self.reload_from_disk();
            return Err(error);
        }
        self.terms = next_terms;
        self.references = next_references;
        Ok(())
    }

    /// Removes one indexed reference.
    pub fn remove(&mut self, reference: &ObjectReference) -> Result<bool, SearchIndexError> {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(SearchIndexError::Storage)?;
        self.reload_from_disk()?;
        let key = reference_key(reference);
        if !self.references.contains_key(&key) {
            return Ok(false);
        }
        let mut next_terms = self.terms.clone();
        let mut next_references = self.references.clone();
        next_references.remove(&key);
        remove_terms_for_reference(&mut next_terms, &key);
        if let Err(error) = self.persist(&next_terms, &next_references) {
            let _ = self.reload_from_disk();
            return Err(error);
        }
        self.terms = next_terms;
        self.references = next_references;
        Ok(true)
    }

    /// Applies one explicit canonical-to-search-index synchronization change.
    pub fn synchronize(
        &mut self,
        change: &IndexSynchronizationChange,
    ) -> Result<(), SearchIndexError> {
        self.synchronize_batch(std::slice::from_ref(change))
    }

    /// Applies a collection of changes and publishes the complete search index
    /// as one atomic document replacement.
    pub fn synchronize_batch(
        &mut self,
        changes: &[IndexSynchronizationChange],
    ) -> Result<(), SearchIndexError> {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(SearchIndexError::Storage)?;
        self.synchronize_batch_locked(changes)
    }

    pub(crate) fn synchronize_batch_locked(
        &mut self,
        changes: &[IndexSynchronizationChange],
    ) -> Result<(), SearchIndexError> {
        self.reload_from_disk()?;
        let mut next_terms = self.terms.clone();
        let mut next_references = self.references.clone();
        for change in changes {
            let key = reference_key(change.reference());
            match change.action() {
                IndexChangeAction::Upsert => {
                    let text = change
                        .search_text()
                        .ok_or(SearchIndexError::MissingSearchText)?;
                    remove_terms_for_reference(&mut next_terms, &key);
                    next_references.insert(
                        key.clone(),
                        StoredReference {
                            source: change.reference().source().to_owned(),
                            object_reference: change.reference().object_reference().to_owned(),
                        },
                    );
                    for term in tokenize(text) {
                        next_terms.entry(term).or_default().insert(key.clone());
                    }
                }
                IndexChangeAction::Remove => {
                    next_references.remove(&key);
                    remove_terms_for_reference(&mut next_terms, &key);
                }
            }
        }
        if let Err(error) = self.persist(&next_terms, &next_references) {
            let _ = self.reload_from_disk();
            return Err(error);
        }
        self.terms = next_terms;
        self.references = next_references;
        Ok(())
    }

    pub(crate) fn reload_from_disk(&mut self) -> Result<(), SearchIndexError> {
        let id = DocumentId::new(SEARCH_INDEX_DOCUMENT)
            .map_err(|_| SearchIndexError::InvalidDocumentId)?;
        if let Some(document) = self.store.get(&id).map_err(SearchIndexError::Storage)? {
            let persisted: PersistedSearchIndex = serde_json::from_slice(document.as_bytes())
                .map_err(|error| SearchIndexError::Serialization(error.to_string()))?;
            if persisted.terms.values().any(|keys| {
                keys.iter()
                    .any(|key| !persisted.references.contains_key(key))
            }) {
                return Err(SearchIndexError::CorruptDocument);
            }
            self.terms = persisted.terms;
            self.references = persisted.references;
        } else {
            self.terms = load(&self.store, SEARCH_DOCUMENT)?;
            self.references = load(&self.store, REFERENCES_DOCUMENT)?;
        }
        Ok(())
    }

    /// Rebuilds the search index from canonical reference/text pairs.
    pub fn rebuild<I>(&mut self, entries: I) -> Result<(), SearchIndexError>
    where
        I: IntoIterator<Item = (ObjectReference, String)>,
    {
        let _lock = self
            .store
            .acquire_exclusive_lock()
            .map_err(SearchIndexError::Storage)?;
        let mut next_terms: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut next_references: BTreeMap<String, StoredReference> = BTreeMap::new();
        for (reference, text) in entries {
            let key = reference_key(&reference);
            next_references.insert(
                key.clone(),
                StoredReference {
                    source: reference.source().to_owned(),
                    object_reference: reference.object_reference().to_owned(),
                },
            );
            for term in tokenize(&text) {
                next_terms.entry(term).or_default().insert(key.clone());
            }
        }
        self.persist(&next_terms, &next_references)?;
        self.terms = next_terms;
        self.references = next_references;
        Ok(())
    }

    /// Executes deterministic lexical search. Results are ordered by descending
    /// term-match count and then by stable reference key.
    pub fn search(&self, query: &str) -> Result<Vec<KgSearchHit>, SearchIndexError> {
        let mut scores = BTreeMap::<String, u32>::new();
        for term in tokenize(query) {
            if let Some(keys) = self.terms.get(&term) {
                for key in keys {
                    *scores.entry(key.clone()).or_default() += 1;
                }
            }
        }

        let mut hits = scores
            .into_iter()
            .filter_map(|(key, score)| {
                let stored = self.references.get(&key)?;
                let reference =
                    ObjectReference::new(stored.source.clone(), stored.object_reference.clone())
                        .ok()?;
                Some(KgSearchHit { reference, score })
            })
            .collect::<Vec<_>>();
        hits.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| reference_key(&left.reference).cmp(&reference_key(&right.reference)))
        });
        Ok(hits)
    }

    fn persist(
        &self,
        terms: &BTreeMap<String, BTreeSet<String>>,
        references: &BTreeMap<String, StoredReference>,
    ) -> Result<(), SearchIndexError> {
        let persisted = PersistedSearchIndex {
            terms: terms.clone(),
            references: references.clone(),
        };
        persist_document(&self.store, SEARCH_INDEX_DOCUMENT, &persisted)
    }
}

/// Public KG-owned search access boundary.
pub trait KgSearchIndexAccess {
    type Error: std::error::Error + Send + Sync + 'static;
    fn search_kg(&self, query: &str) -> Result<Vec<KgSearchHit>, Self::Error>;
}
impl KgSearchIndexAccess for FileSearchIndex {
    type Error = SearchIndexError;
    fn search_kg(&self, query: &str) -> Result<Vec<KgSearchHit>, Self::Error> {
        self.search(query)
    }
}

/// Errors from the KG-owned search index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchIndexError {
    Storage(DocumentStoreError),
    Serialization(String),
    InvalidDocumentId,
    CorruptDocument,
    MissingSearchText,
}
impl std::fmt::Display for SearchIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "search index storage failure: {error}"),
            Self::Serialization(error) => write!(f, "search index serialization failure: {error}"),
            Self::InvalidDocumentId => f.write_str("invalid search index document id"),
            Self::CorruptDocument => f.write_str("search index document is corrupt"),
            Self::MissingSearchText => {
                f.write_str("search-index synchronization requires searchable text")
            }
        }
    }
}
impl std::error::Error for SearchIndexError {}

const SEARCH_DOCUMENT: &str = "indexes/search.json";
const REFERENCES_DOCUMENT: &str = "indexes/search-references.json";
const SEARCH_INDEX_DOCUMENT: &str = "indexes/search-index.json";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct PersistedSearchIndex {
    terms: BTreeMap<String, BTreeSet<String>>,
    references: BTreeMap<String, StoredReference>,
}

fn remove_terms_for_reference(terms: &mut BTreeMap<String, BTreeSet<String>>, key: &str) {
    let term_names = terms.keys().cloned().collect::<Vec<_>>();
    let mut empty_terms = Vec::new();
    for term in term_names {
        if let Some(keys) = terms.get_mut(&term) {
            keys.remove(key);
            if keys.is_empty() {
                empty_terms.push(term);
            }
        }
    }
    for term in empty_terms {
        terms.remove(&term);
    }
}

fn reference_key(reference: &ObjectReference) -> String {
    format!(
        "{}\u{1f}{}",
        reference.source(),
        reference.object_reference()
    )
}

fn tokenize(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
}

fn load<T>(store: &FileDocumentStore, name: &str) -> Result<T, SearchIndexError>
where
    T: serde::de::DeserializeOwned + Default,
{
    let id = DocumentId::new(name).map_err(|_| SearchIndexError::InvalidDocumentId)?;
    let Some(document) = store.get(&id).map_err(SearchIndexError::Storage)? else {
        return Ok(T::default());
    };
    serde_json::from_slice(document.as_bytes())
        .map_err(|error| SearchIndexError::Serialization(error.to_string()))
}

fn persist_document<T: serde::Serialize>(
    store: &FileDocumentStore,
    name: &str,
    value: &T,
) -> Result<(), SearchIndexError> {
    let id = DocumentId::new(name).map_err(|_| SearchIndexError::InvalidDocumentId)?;
    let document = Document::new(
        serde_json::to_vec(value)
            .map_err(|error| SearchIndexError::Serialization(error.to_string()))?,
    );
    if store.contains(&id).map_err(SearchIndexError::Storage)? {
        store
            .replace(&id, document)
            .map_err(SearchIndexError::Storage)
    } else {
        store
            .create(id, document)
            .map_err(SearchIndexError::Storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nizaam-search-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn lexical_search_is_deterministic_and_persistent() {
        let root = root();
        let a = ObjectReference::new("kg", "a").unwrap();
        let b = ObjectReference::new("kg", "b").unwrap();
        let mut index = FileSearchIndex::open(&root).unwrap();
        index.upsert(a.clone(), "Quran verse knowledge").unwrap();
        index.upsert(b.clone(), "knowledge graph").unwrap();
        let hits = index.search("quran knowledge").unwrap();
        assert_eq!(hits[0].reference(), &a);
        assert_eq!(hits[0].score(), 2);
        drop(index);
        let reopened = FileSearchIndex::open(&root).unwrap();
        assert_eq!(reopened.search("quran").unwrap()[0].reference(), &a);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn separate_open_handles_preserve_search_updates_in_one_atomic_document() {
        let root = root();
        let a = ObjectReference::new("kg", "a").unwrap();
        let b = ObjectReference::new("kg", "b").unwrap();
        let mut first = FileSearchIndex::open(&root).unwrap();
        let mut second = FileSearchIndex::open(&root).unwrap();

        first.upsert(a.clone(), "quran").unwrap();
        second.upsert(b.clone(), "hadith").unwrap();

        let combined = root.join(SEARCH_INDEX_DOCUMENT);
        assert!(combined.is_file());
        let reopened = FileSearchIndex::open(&root).unwrap();
        assert_eq!(reopened.search("quran").unwrap()[0].reference(), &a);
        assert_eq!(reopened.search("hadith").unwrap()[0].reference(), &b);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn synchronization_requires_search_text_for_upserts() {
        let root = root();
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target_type =
            nizaam_indexing::index::definition::TargetReferenceType::new("entity").unwrap();
        let target = crate::versioning::ChangeTarget::Entity(
            crate::identity::EntityId::new("entity-1").unwrap(),
        );
        let mut index = FileSearchIndex::open(&root).unwrap();

        let missing_text = IndexSynchronizationChange::upsert(
            target.clone(),
            reference.clone(),
            target_type.clone(),
            None::<String>,
        );
        assert_eq!(
            index.synchronize(&missing_text),
            Err(SearchIndexError::MissingSearchText)
        );

        let upsert = IndexSynchronizationChange::upsert(
            target.clone(),
            reference.clone(),
            target_type.clone(),
            Some("Quran"),
        );
        index.synchronize(&upsert).unwrap();
        assert_eq!(index.search("quran").unwrap()[0].reference(), &reference);

        let remove = IndexSynchronizationChange::remove(target, reference, target_type);
        index.synchronize(&remove).unwrap();
        assert!(index.search("quran").unwrap().is_empty());

        let _ = std::fs::remove_dir_all(root);
    }
}
