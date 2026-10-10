//! Logical Indexing state observations used by the Knowledge Graph query layer.
//!
//! The Knowledge Graph does not own an index registry, active-version registry,
//! publication lifecycle, or consistency policy. This module only carries the
//! facts that a caller has observed from Nizaam Indexing so query planning can
//! make deterministic decisions without recreating Indexing state ownership.

use nizaam_indexing::IndexAssignedId;
use nizaam_indexing::index::reference::ObjectReference;

/// Factual queryability state observed for one Indexing-backed source object.
///
/// This is deliberately observational rather than prescriptive. In particular,
/// `Stale` does not mean that a query must be rejected. The Indexing consistency
/// policy remains authoritative for that decision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexQueryability {
    /// The observed index state is queryable for the requested operation.
    Queryable,
    /// The observed index state is not currently available for querying.
    Unavailable,
    /// The observed state is queryable but known to lag the relevant source state.
    Stale,
    /// The observed index is undergoing a rebuild or equivalent transition.
    Rebuilding,
}

impl IndexQueryability {
    /// Returns whether the observation says the index can currently be queried.
    #[must_use]
    pub const fn is_queryable(self) -> bool {
        matches!(self, Self::Queryable | Self::Stale)
    }
}

/// Factual version information observed from Indexing.
///
/// The values are intentionally opaque to the Knowledge Graph. KG does not
/// parse, compare, publish, or activate Indexing versions here.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexVersionObservation {
    version: String,
    source_version: Option<String>,
    schema_version: Option<String>,
}

impl IndexVersionObservation {
    /// Creates an opaque Indexing version observation.
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

    /// Attaches an opaque source-version observation.
    #[must_use]
    pub fn with_source_version(mut self, source_version: impl Into<String>) -> Self {
        self.source_version = Some(source_version.into());
        self
    }

    /// Attaches an opaque schema-version observation.
    #[must_use]
    pub fn with_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = Some(schema_version.into());
        self
    }

    /// Returns the observed Indexing version identifier.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the optional source-version observation.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }

    /// Returns the optional schema-version observation.
    #[must_use]
    pub fn schema_version(&self) -> Option<&str> {
        self.schema_version.as_deref()
    }
}

/// A KG-owned snapshot of Indexing facts for one source-owned object.
///
/// The assigned identifier remains Indexing-owned. The KG stores it only as an
/// opaque external reference and never substitutes its own `IndexId`.
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
    /// Creates a factual Indexing state observation.
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

    /// Attaches an observed logical index version.
    #[must_use]
    pub fn with_version(mut self, version: IndexVersionObservation) -> Self {
        self.version = Some(version);
        self
    }

    /// Attaches the source-side update sequence observed during synchronization.
    #[must_use]
    pub const fn with_source_update_sequence(mut self, sequence: u64) -> Self {
        self.source_update_sequence = Some(sequence);
        self
    }

    /// Attaches the indexed-side update sequence represented by the observation.
    #[must_use]
    pub const fn with_indexed_update_sequence(mut self, sequence: u64) -> Self {
        self.indexed_update_sequence = Some(sequence);
        self
    }

    /// Returns Indexing's assigned identity for the source-owned object.
    #[must_use]
    pub fn assigned_id(&self) -> &IndexAssignedId {
        &self.assigned_id
    }

    /// Returns the source-owned object reference.
    #[must_use]
    pub fn object_reference(&self) -> &ObjectReference {
        &self.object_reference
    }

    /// Returns the observed queryability state.
    #[must_use]
    pub const fn queryability(&self) -> IndexQueryability {
        self.queryability
    }

    /// Returns the optional opaque version observation.
    #[must_use]
    pub fn version(&self) -> Option<&IndexVersionObservation> {
        self.version.as_ref()
    }

    /// Returns the observed source-side update sequence.
    #[must_use]
    pub const fn source_update_sequence(&self) -> Option<u64> {
        self.source_update_sequence
    }

    /// Returns the observed indexed-side update sequence.
    #[must_use]
    pub const fn indexed_update_sequence(&self) -> Option<u64> {
        self.indexed_update_sequence
    }
}

/// Errors for structurally invalid KG-owned Indexing observations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexStateError {
    /// A required opaque version identifier was empty.
    EmptyVersion,
}

impl core::fmt::Display for IndexStateError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyVersion => {
                formatter.write_str("Indexing version observation must not be empty")
            }
        }
    }
}

impl std::error::Error for IndexStateError {}

#[cfg(test)]
mod tests {
    use super::{IndexQueryability, IndexStateObservation, IndexVersionObservation};
    use nizaam_indexing::index::reference::ObjectReference;
    use nizaam_indexing::{IndexAssignedId, TargetReferenceType};

    fn reference() -> ObjectReference {
        ObjectReference::new("kg", "entity-1").expect("valid object reference")
    }

    fn assigned_id(reference: &ObjectReference) -> IndexAssignedId {
        let target_type = TargetReferenceType::new("entity").expect("valid target reference type");
        IndexAssignedId::generate(&target_type, reference)
    }

    #[test]
    fn version_observation_rejects_empty_identity() {
        assert!(IndexVersionObservation::new("  ").is_err());
    }

    #[test]
    fn observation_preserves_indexing_owned_identity_and_facts() {
        let reference = reference();
        let observation = IndexStateObservation::new(
            assigned_id(&reference),
            reference.clone(),
            IndexQueryability::Stale,
        )
        .with_version(
            IndexVersionObservation::new("version-2")
                .expect("valid version")
                .with_source_version("source-7")
                .with_schema_version("schema-3"),
        )
        .with_source_update_sequence(7)
        .with_indexed_update_sequence(6);

        assert_eq!(observation.object_reference(), &reference);
        assert!(observation.queryability().is_queryable());
        assert_eq!(observation.version().unwrap().version(), "version-2");
        assert_eq!(
            observation.version().unwrap().source_version(),
            Some("source-7")
        );
        assert_eq!(
            observation.version().unwrap().schema_version(),
            Some("schema-3")
        );
        assert_eq!(observation.source_update_sequence(), Some(7));
        assert_eq!(observation.indexed_update_sequence(), Some(6));
    }

    #[test]
    fn unavailable_and_rebuilding_are_not_queryable() {
        assert!(!IndexQueryability::Unavailable.is_queryable());
        assert!(!IndexQueryability::Rebuilding.is_queryable());
        assert!(IndexQueryability::Queryable.is_queryable());
    }
}
