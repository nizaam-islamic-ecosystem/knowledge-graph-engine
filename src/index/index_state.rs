//! KG-owned derived-index state and version metadata.
//!
//! Phase 6's Indexing observations remain intact below. Phase 7 adds a separate
//! KG-owned state model so a derived index can be stale or rebuilding without
//! turning an Indexing-native version into a KG semantic version.

use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::reference::ObjectReference;

/// Factual queryability state observed for one Indexing-backed source object.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize,
)]
pub enum IndexQueryability {
    Queryable,
    Unavailable,
    Stale,
    Rebuilding,
}

impl IndexQueryability {
    /// Returns whether the observed state can currently be queried.
    #[must_use]
    pub const fn is_queryable(self) -> bool {
        matches!(self, Self::Queryable | Self::Stale)
    }
}

/// Opaque Indexing version observation retained for Phase 6 compatibility.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexVersionObservation {
    version: String,
    source_version: Option<String>,
    schema_version: Option<String>,
}

impl IndexVersionObservation {
    pub fn new(version: impl Into<String>) -> Result<Self, IndexStateError> {
        let version = version.into();
        if version.trim().is_empty() {
            return Err(IndexStateError::EmptyVersion);
        }
        Ok(Self {
            version,
            source_version: None,
            schema_version: None,
        })
    }

    #[must_use]
    pub fn with_source_version(mut self, source_version: impl Into<String>) -> Self {
        self.source_version = Some(source_version.into());
        self
    }

    #[must_use]
    pub fn with_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = Some(schema_version.into());
        self
    }

    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }
    #[must_use]
    pub fn schema_version(&self) -> Option<&str> {
        self.schema_version.as_deref()
    }
}

/// KG-owned snapshot of Indexing facts for one source object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexStateObservation {
    assigned_id: IndexAssignedId,
    object_reference: ObjectReference,
    queryability: IndexQueryability,
    version: Option<IndexVersionObservation>,
    source_update_sequence: Option<u64>,
    indexed_update_sequence: Option<u64>,
}

impl IndexStateObservation {
    #[must_use]
    pub fn new(
        assigned_id: IndexAssignedId,
        object_reference: ObjectReference,
        queryability: IndexQueryability,
    ) -> Self {
        Self {
            assigned_id,
            object_reference,
            queryability,
            version: None,
            source_update_sequence: None,
            indexed_update_sequence: None,
        }
    }
    #[must_use]
    pub fn with_version(mut self, version: IndexVersionObservation) -> Self {
        self.version = Some(version);
        self
    }
    #[must_use]
    pub const fn with_source_update_sequence(mut self, sequence: u64) -> Self {
        self.source_update_sequence = Some(sequence);
        self
    }
    #[must_use]
    pub const fn with_indexed_update_sequence(mut self, sequence: u64) -> Self {
        self.indexed_update_sequence = Some(sequence);
        self
    }
    #[must_use]
    pub fn assigned_id(&self) -> &IndexAssignedId {
        &self.assigned_id
    }
    #[must_use]
    pub fn object_reference(&self) -> &ObjectReference {
        &self.object_reference
    }
    #[must_use]
    pub const fn queryability(&self) -> IndexQueryability {
        self.queryability
    }
    #[must_use]
    pub fn version(&self) -> Option<&IndexVersionObservation> {
        self.version.as_ref()
    }
    #[must_use]
    pub const fn source_update_sequence(&self) -> Option<u64> {
        self.source_update_sequence
    }
    #[must_use]
    pub const fn indexed_update_sequence(&self) -> Option<u64> {
        self.indexed_update_sequence
    }
}

/// Kind of a KG-owned derived index.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize,
)]
pub enum KgIndexKind {
    Forward,
    Reverse,
    Search,
}

/// Synchronization guarantee declared for one KG-owned derived index.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize,
)]
pub enum IndexSynchronizationPolicy {
    /// Forward/reverse indexes are critical derived access paths.
    Strong,
    /// Search may lag canonical state and must expose explicit staleness.
    Eventual,
}

impl IndexSynchronizationPolicy {
    #[must_use]
    pub const fn for_kind(kind: KgIndexKind) -> Self {
        match kind {
            KgIndexKind::Forward | KgIndexKind::Reverse => Self::Strong,
            KgIndexKind::Search => Self::Eventual,
        }
    }
}

/// Independent version metadata for one KG-owned derived index.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KgIndexVersion {
    generation: u64,
    built_from_kg_version: String,
}

impl KgIndexVersion {
    pub fn new(
        generation: u64,
        built_from_kg_version: impl Into<String>,
    ) -> Result<Self, IndexStateError> {
        let built_from_kg_version = built_from_kg_version.into();
        if built_from_kg_version.trim().is_empty() {
            return Err(IndexStateError::EmptyKgVersion);
        }
        Ok(Self {
            generation,
            built_from_kg_version,
        })
    }
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    #[must_use]
    pub fn built_from_kg_version(&self) -> &str {
        &self.built_from_kg_version
    }
}

/// State metadata for a derived KG index.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KgIndexState {
    kind: KgIndexKind,
    queryability: IndexQueryability,
    policy: IndexSynchronizationPolicy,
    version: KgIndexVersion,
}

impl KgIndexState {
    pub fn new(kind: KgIndexKind, version: KgIndexVersion) -> Self {
        Self {
            kind,
            queryability: IndexQueryability::Queryable,
            policy: IndexSynchronizationPolicy::for_kind(kind),
            version,
        }
    }

    /// Creates an explicitly stale state before the first successful sync.
    #[must_use]
    pub fn uninitialized(kind: KgIndexKind) -> Self {
        Self::new(
            kind,
            KgIndexVersion::new(0, "uninitialized")
                .expect("static uninitialized index version is valid"),
        )
        .mark_stale()
    }

    /// Overrides the synchronization policy while preserving index state.
    #[must_use]
    pub const fn with_policy(mut self, policy: IndexSynchronizationPolicy) -> Self {
        self.policy = policy;
        self
    }
    #[must_use]
    pub const fn kind(&self) -> KgIndexKind {
        self.kind
    }
    #[must_use]
    pub const fn queryability(&self) -> IndexQueryability {
        self.queryability
    }
    #[must_use]
    pub fn version(&self) -> &KgIndexVersion {
        &self.version
    }
    #[must_use]
    pub const fn policy(&self) -> IndexSynchronizationPolicy {
        self.policy
    }

    /// Marks this index synchronized against a newly committed KG version.
    pub fn synchronized_to(&self, kg_version: impl Into<String>) -> Result<Self, IndexStateError> {
        Ok(Self {
            kind: self.kind,
            queryability: IndexQueryability::Queryable,
            policy: self.policy,
            version: KgIndexVersion::new(self.version.generation().saturating_add(1), kg_version)?,
        })
    }

    /// Marks a failed synchronization according to the declared policy.
    #[must_use]
    pub fn mark_synchronization_failed(mut self) -> Self {
        self.queryability = match self.policy {
            IndexSynchronizationPolicy::Strong => IndexQueryability::Unavailable,
            IndexSynchronizationPolicy::Eventual => IndexQueryability::Stale,
        };
        self
    }
    #[must_use]
    pub fn mark_stale(mut self) -> Self {
        self.queryability = IndexQueryability::Stale;
        self
    }
    #[must_use]
    pub fn mark_rebuilding(mut self) -> Self {
        self.queryability = IndexQueryability::Rebuilding;
        self
    }
    #[must_use]
    pub fn mark_unavailable(mut self) -> Self {
        self.queryability = IndexQueryability::Unavailable;
        self
    }
    #[must_use]
    pub fn mark_queryable(mut self) -> Self {
        self.queryability = IndexQueryability::Queryable;
        self
    }
}

/// Errors for index state metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexStateError {
    EmptyVersion,
    EmptyKgVersion,
}

impl core::fmt::Display for IndexStateError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyVersion => {
                formatter.write_str("Indexing version observation must not be empty")
            }
            Self::EmptyKgVersion => {
                formatter.write_str("KG index built-from version must not be empty")
            }
        }
    }
}
impl std::error::Error for IndexStateError {}

#[cfg(test)]
mod tests {
    use super::*;
    use nizaam_indexing::index::definition::TargetReferenceType;

    #[test]
    fn kg_index_version_is_distinct_from_kg_version() {
        let version = KgIndexVersion::new(3, "kg-v42").unwrap();
        let state = KgIndexState::new(KgIndexKind::Search, version).mark_stale();
        assert_eq!(state.kind(), KgIndexKind::Search);
        assert_eq!(state.queryability(), IndexQueryability::Stale);
        assert_eq!(state.version().generation(), 3);
        assert_eq!(state.version().built_from_kg_version(), "kg-v42");
    }

    #[test]
    fn phase6_indexing_observation_remains_opaque() {
        let reference = ObjectReference::new("kg", "entity-1").unwrap();
        let target = TargetReferenceType::new("entity").unwrap();
        let assigned = IndexAssignedId::generate(&target, &reference);
        let observation = IndexStateObservation::new(
            assigned.clone(),
            reference.clone(),
            IndexQueryability::Stale,
        );
        assert_eq!(observation.assigned_id(), &assigned);
        assert_eq!(observation.object_reference(), &reference);
        assert!(observation.queryability().is_queryable());
    }
    #[test]
    fn synchronization_policy_and_version_advance_are_explicit() {
        let state = KgIndexState::uninitialized(KgIndexKind::Forward);
        assert_eq!(state.policy(), IndexSynchronizationPolicy::Strong);
        assert_eq!(state.queryability(), IndexQueryability::Stale);

        let synchronized = state
            .synchronized_to("kg-v1")
            .expect("synchronization should produce a valid version");
        assert_eq!(synchronized.queryability(), IndexQueryability::Queryable);
        assert_eq!(synchronized.version().generation(), 1);
        assert_eq!(synchronized.version().built_from_kg_version(), "kg-v1");
    }

    #[test]
    fn failed_synchronization_respects_strong_and_eventual_policies() {
        let forward =
            KgIndexState::uninitialized(KgIndexKind::Forward).mark_synchronization_failed();
        assert_eq!(forward.queryability(), IndexQueryability::Unavailable);

        let search = KgIndexState::uninitialized(KgIndexKind::Search).mark_synchronization_failed();
        assert_eq!(search.queryability(), IndexQueryability::Stale);
    }
}
