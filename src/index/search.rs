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
        let terms = load(&store, SEARCH_DOCUMENT)?;
        let references = load(&store, REFERENCES_DOCUMENT)?;
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
        let key = reference_key(&reference);
        self.remove_terms_for_reference(&key);
        self.references.insert(
            key.clone(),
            StoredReference {
                source: reference.source().to_owned(),
                object_reference: reference.object_reference().to_owned(),
            },
        );

        for term in tokenize(text) {
            self.terms.entry(term).or_default().insert(key.clone());
        }
        self.persist()
    }

    /// Removes one indexed reference.
    pub fn remove(&mut self, reference: &ObjectReference) -> Result<bool, SearchIndexError> {
        let key = reference_key(reference);
        let existed = self.references.remove(&key).is_some();
        if existed {
            self.remove_terms_for_reference(&key);
            self.persist()?;
        }
        Ok(existed)
    }

    /// Applies one explicit canonical-to-search-index synchronization change.
    pub fn synchronize(
        &mut self,
        change: &IndexSynchronizationChange,
    ) -> Result<(), SearchIndexError> {
        match change.action() {
            IndexChangeAction::Upsert => {
                let text = change
                    .search_text()
                    .ok_or(SearchIndexError::MissingSearchText)?;
                self.upsert(change.reference().clone(), text)?;
            }
            IndexChangeAction::Remove => {
                self.remove(change.reference())?;
            }
        }
        Ok(())
    }

    /// Rebuilds the search index from canonical reference/text pairs.
    pub fn rebuild<I>(&mut self, entries: I) -> Result<(), SearchIndexError>
    where
        I: IntoIterator<Item = (ObjectReference, String)>,
    {
        self.terms.clear();
        self.references.clear();
        for (reference, text) in entries {
            let key = reference_key(&reference);
            self.references.insert(
                key.clone(),
                StoredReference {
                    source: reference.source().to_owned(),
                    object_reference: reference.object_reference().to_owned(),
                },
            );
            for term in tokenize(&text) {
                self.terms.entry(term).or_default().insert(key.clone());
            }
        }
        self.persist()
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

    fn remove_terms_for_reference(&mut self, key: &str) {
        let terms = self.terms.keys().cloned().collect::<Vec<_>>();
        let mut empty_terms = Vec::new();

        for term in terms {
            if let Some(keys) = self.terms.get_mut(&term) {
                keys.remove(key);
                if keys.is_empty() {
                    empty_terms.push(term);
                }
            }
        }

        for term in empty_terms {
            self.terms.remove(&term);
        }
    }

    fn persist(&mut self) -> Result<(), SearchIndexError> {
        persist_document(&mut self.store, SEARCH_DOCUMENT, &self.terms)?;
        persist_document(&mut self.store, REFERENCES_DOCUMENT, &self.references)
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
    store: &mut FileDocumentStore,
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
