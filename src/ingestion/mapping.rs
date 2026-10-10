//! Typed semantic mapping candidates and deterministic deduplication hooks.
//!
//! Mapping produces candidate values only. It does not mutate canonical entities
//! or assertions. Entity resolution is delegated to the existing Phase 3 resolver.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::entity::ExternalIdentifier;
use crate::identity::{
    ConceptId, EntityId, EvidenceId, KnowledgeAssertionId, ReferenceId, SourceId,
};
use crate::resolution::{EntityCandidateProfile, ResolutionInput, ResolutionResult, Resolver};

use super::normalize::{NormalizationError, NormalizationMetadata, NormalizedRecord};
use super::raw::{SourceRecordMetadata, SourceRecordMetadataError};

/// Stable value-key for one mapping output from a source record.
///
/// This is not a generated KG identity. It is derived from source-owned
/// metadata and a deterministic mapping-output ordinal.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CandidateKey {
    source_id: SourceId,
    source_record_key: String,
    source_version: Option<String>,
    output_ordinal: u32,
}

impl CandidateKey {
    /// Creates a stable candidate key from one source record and output ordinal.
    pub fn new(
        source_id: SourceId,
        source_record_key: impl Into<String>,
        source_version: Option<String>,
        output_ordinal: u32,
    ) -> Result<Self, MappingError> {
        let source_record_key = source_record_key.into();
        validate_label(&source_record_key, "source record key")?;
        if let Some(version) = &source_version {
            validate_label(version, "source version")?;
        }
        Ok(Self {
            source_id,
            source_record_key,
            source_version,
            output_ordinal,
        })
    }

    /// Returns the source namespace.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the source-provided record key.
    #[must_use]
    pub fn source_record_key(&self) -> &str {
        &self.source_record_key
    }

    /// Returns the source version/snapshot when present.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }

    /// Returns the stable output ordinal for one-to-many mappings.
    #[must_use]
    pub const fn output_ordinal(&self) -> u32 {
        self.output_ordinal
    }
}

/// Version information for one semantic-mapping stage.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MappingMetadata {
    stage_version: String,
    configuration_version: String,
}

impl MappingMetadata {
    /// Creates validated mapping-stage metadata.
    pub fn new(
        stage_version: impl Into<String>,
        configuration_version: impl Into<String>,
    ) -> Result<Self, MappingError> {
        let stage_version = stage_version.into();
        let configuration_version = configuration_version.into();
        validate_label(&stage_version, "mapping stage version")?;
        validate_label(&configuration_version, "mapping configuration version")?;
        Ok(Self {
            stage_version,
            configuration_version,
        })
    }

    /// Returns the mapping implementation version.
    #[must_use]
    pub fn stage_version(&self) -> &str {
        &self.stage_version
    }

    /// Returns the mapping configuration version.
    #[must_use]
    pub fn configuration_version(&self) -> &str {
        &self.configuration_version
    }
}

/// A semantic candidate emitted by a source-specific mapping stage.
#[derive(Clone, Debug, PartialEq)]
pub struct MappedCandidate<T> {
    key: CandidateKey,
    source: SourceRecordMetadata,
    normalization: NormalizationMetadata,
    mapping: MappingMetadata,
    payload: T,
    external_identifiers: BTreeSet<ExternalIdentifier>,
    resolution_input: Option<ResolutionInput>,
}

impl<T> MappedCandidate<T> {
    /// Creates one mapped output from a normalized source record.
    pub fn from_normalized<I>(
        normalized: NormalizedRecord<I>,
        output_ordinal: u32,
        payload: T,
        mapping: MappingMetadata,
    ) -> Result<Self, MappingError> {
        let (source, _normalized_payload, normalization) = normalized.into_parts();
        source
            .validate()
            .map_err(MappingError::InvalidSourceMetadata)?;
        let external_identifiers = source.external_identifiers().clone();
        let key = CandidateKey::new(
            source.source_id().clone(),
            source.source_record_key(),
            source.source_version().map(str::to_owned),
            output_ordinal,
        )?;
        Ok(Self {
            key,
            source,
            normalization,
            mapping,
            payload,
            external_identifiers,
            resolution_input: None,
        })
    }

    /// Adds a source-scoped external identifier to this candidate.
    ///
    /// The identifier must be owned by the same source namespace as the mapped
    /// record. The original identifier value is preserved exactly.
    pub fn with_external_identifier(
        mut self,
        identifier: ExternalIdentifier,
    ) -> Result<Self, MappingError> {
        if identifier.source_id() != self.key.source_id() {
            return Err(MappingError::ExternalIdentifierSourceMismatch {
                expected: self.key.source_id().clone(),
                actual: identifier.source_id().clone(),
            });
        }
        self.external_identifiers.insert(identifier);
        Ok(self)
    }

    /// Adds an existing Phase 3 resolution input for ambiguous/source-derived
    /// mappings. The caller still supplies candidate profiles at resolution time.
    pub fn with_resolution_input(mut self, input: ResolutionInput) -> Result<Self, MappingError> {
        if let Some(input_source) = input.source_id()
            && input_source != self.source.source_id()
        {
            return Err(MappingError::ResolutionSourceMismatch);
        }
        if let Some(external) = input.external_identifier()
            && external.source_id() != self.source.source_id()
        {
            return Err(MappingError::ResolutionSourceMismatch);
        }
        self.resolution_input = Some(input);
        Ok(self)
    }

    /// Returns this candidate's deterministic source-derived key.
    #[must_use]
    pub fn key(&self) -> &CandidateKey {
        &self.key
    }

    /// Returns source metadata preserved through normalization and mapping.
    #[must_use]
    pub fn source(&self) -> &SourceRecordMetadata {
        &self.source
    }

    /// Returns source-scoped external identifiers preserved from the source record
    /// and any compatible identifiers explicitly added by the mapping stage.
    #[must_use]
    pub fn external_identifiers(&self) -> &BTreeSet<ExternalIdentifier> {
        &self.external_identifiers
    }

    /// Returns the version/configuration of the normalization that preceded mapping.
    #[must_use]
    pub fn normalization(&self) -> &NormalizationMetadata {
        &self.normalization
    }

    /// Returns the mapping stage version/configuration.
    #[must_use]
    pub fn mapping(&self) -> &MappingMetadata {
        &self.mapping
    }

    /// Returns the mapped semantic payload.
    #[must_use]
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Returns whether entity-resolution input is attached.
    #[must_use]
    pub fn requires_resolution(&self) -> bool {
        self.resolution_input.is_some()
    }

    /// Returns the existing Phase 3 resolution input, if this candidate needs it.
    #[must_use]
    pub fn resolution_input(&self) -> Option<&ResolutionInput> {
        self.resolution_input.as_ref()
    }

    /// Delegates candidate resolution to the existing deterministic Phase 3 resolver.
    ///
    /// Returns `None` for deterministic mappings that did not request resolution.
    #[must_use]
    pub fn resolve_with(
        &self,
        resolver: &Resolver,
        profiles: &[EntityCandidateProfile],
    ) -> Option<ResolutionResult> {
        self.resolution_input
            .as_ref()
            .map(|input| resolver.resolve(input, profiles))
    }
}

/// Contract for source-specific semantic mapping.
pub trait SemanticMapper<Input, Output> {
    /// Returns the mapper version used in provenance/reprocessing decisions.
    fn stage_version(&self) -> &str;

    /// Returns the configuration version used by this mapper.
    fn configuration_version(&self) -> &str;

    /// Maps one normalized source record to zero or more semantic candidates.
    fn map_record(
        &self,
        input: NormalizedRecord<Input>,
    ) -> Result<Vec<MappedCandidate<Output>>, MappingError>;
}

/// Source-level deduplication key applied before entity resolution.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceDeduplicationKey {
    source_id: SourceId,
    source_record_key: String,
    source_version: Option<String>,
}

impl SourceDeduplicationKey {
    /// Derives the source key from a validated source record.
    pub fn from_metadata(metadata: &SourceRecordMetadata) -> Result<Self, MappingError> {
        metadata
            .validate()
            .map_err(MappingError::InvalidSourceMetadata)?;
        Ok(Self {
            source_id: metadata.source_id().clone(),
            source_record_key: metadata.source_record_key().to_owned(),
            source_version: metadata.source_version().map(str::to_owned),
        })
    }

    /// Returns the source-scoped record identity.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the source-provided record key.
    #[must_use]
    pub fn source_record_key(&self) -> &str {
        &self.source_record_key
    }

    /// Returns the source snapshot version.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }
}

/// Result of registering a key with a deduplicator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeduplicationResult<K> {
    /// The key was not observed before this insertion.
    FirstSeen,
    /// An equivalent key was already registered.
    Duplicate { existing: K },
}

/// In-memory source-level deduplicator for a single run or source snapshot.
#[derive(Clone, Debug, Default)]
pub struct SourceDeduplicator {
    seen: BTreeSet<SourceDeduplicationKey>,
}

impl SourceDeduplicator {
    /// Creates an empty source-level deduplicator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `FirstSeen` or `Duplicate` for the source record key.
    pub fn insert(
        &mut self,
        metadata: &SourceRecordMetadata,
    ) -> Result<DeduplicationResult<SourceDeduplicationKey>, MappingError> {
        let key = SourceDeduplicationKey::from_metadata(metadata)?;
        if self.seen.insert(key.clone()) {
            Ok(DeduplicationResult::FirstSeen)
        } else {
            Ok(DeduplicationResult::Duplicate { existing: key })
        }
    }

    /// Returns how many distinct source records have been observed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Returns whether no source records have been observed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

/// Typed identity used for semantic deduplication after resolution.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalSemanticIdentity {
    /// A canonical entity.
    Entity(EntityId),
    /// A canonical concept.
    Concept(ConceptId),
    /// A canonical knowledge assertion.
    Assertion(KnowledgeAssertionId),
    /// A canonical source.
    Source(SourceId),
    /// A canonical reference.
    Reference(ReferenceId),
    /// A canonical evidence item.
    Evidence(EvidenceId),
    /// A typed value representing a future semantic-object category.
    Opaque {
        type_name: String,
        identifier: String,
    },
}

/// Composite key for exact semantic-subgraph deduplication.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticDeduplicationKey(BTreeSet<CanonicalSemanticIdentity>);

impl SemanticDeduplicationKey {
    /// Creates a nonempty deterministic key from canonical semantic identities.
    pub fn new<I>(identities: I) -> Result<Self, MappingError>
    where
        I: IntoIterator<Item = CanonicalSemanticIdentity>,
    {
        let identities = identities.into_iter().collect::<BTreeSet<_>>();
        if identities.is_empty() {
            return Err(MappingError::EmptySemanticDeduplicationKey);
        }
        for identity in &identities {
            if let CanonicalSemanticIdentity::Opaque {
                type_name,
                identifier,
            } = identity
            {
                validate_label(type_name, "semantic identity type")?;
                validate_label(identifier, "semantic identity value")?;
            }
        }
        Ok(Self(identities))
    }

    /// Returns the canonical identities in deterministic order.
    #[must_use]
    pub fn identities(&self) -> &BTreeSet<CanonicalSemanticIdentity> {
        &self.0
    }
}

/// Semantic deduplicator that preserves the first candidate associated with a key.
#[derive(Clone, Debug, Default)]
pub struct SemanticDeduplicator {
    canonical: BTreeMap<SemanticDeduplicationKey, CandidateKey>,
}

impl SemanticDeduplicator {
    /// Creates an empty semantic deduplicator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a resolved semantic key, returning the original candidate if duplicate.
    pub fn insert(
        &mut self,
        key: SemanticDeduplicationKey,
        candidate: CandidateKey,
    ) -> DeduplicationResult<CandidateKey> {
        if let Some(existing) = self.canonical.get(&key) {
            DeduplicationResult::Duplicate {
                existing: existing.clone(),
            }
        } else {
            self.canonical.insert(key, candidate);
            DeduplicationResult::FirstSeen
        }
    }

    /// Returns the number of canonical semantic keys observed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.canonical.len()
    }

    /// Returns whether no semantic keys have been observed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.canonical.is_empty()
    }
}

/// Mapping-stage errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MappingError {
    /// A required mapping label is empty or contains a control character.
    InvalidLabel { field: &'static str },
    /// Source metadata failed validation.
    InvalidSourceMetadata(SourceRecordMetadataError),
    /// A normalized value could not be created.
    InvalidNormalization(NormalizationError),
    /// The attached Phase 3 resolution input refers to a different source namespace.
    ResolutionSourceMismatch,
    /// An external identifier belongs to a source other than the mapped candidate.
    ExternalIdentifierSourceMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    /// Semantic deduplication requires at least one canonical semantic identity.
    EmptySemanticDeduplicationKey,
}

impl fmt::Display for MappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel { field } => write!(formatter, "{field} is invalid"),
            Self::InvalidSourceMetadata(error) => {
                write!(formatter, "invalid mapping source: {error}")
            }
            Self::InvalidNormalization(error) => {
                write!(formatter, "invalid normalization result: {error}")
            }
            Self::ResolutionSourceMismatch => {
                formatter.write_str("resolution input source differs from mapped candidate source")
            }
            Self::ExternalIdentifierSourceMismatch { expected, actual } => write!(
                formatter,
                "external identifier source {actual} differs from mapped candidate source {expected}"
            ),
            Self::EmptySemanticDeduplicationKey => {
                formatter.write_str("semantic deduplication key must not be empty")
            }
        }
    }
}
impl std::error::Error for MappingError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), MappingError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(MappingError::InvalidLabel { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateKey, CanonicalSemanticIdentity, DeduplicationResult, MappedCandidate,
        MappingError, MappingMetadata, SemanticDeduplicationKey, SemanticDeduplicator,
        SourceDeduplicator,
    };
    use crate::entity::ExternalIdentifier;
    use crate::identity::{ConceptId, EntityId, SourceId};
    use crate::ingestion::normalize::{NormalizationMetadata, NormalizedRecord};
    use crate::ingestion::raw::{SourceClass, SourceRecordMetadata};
    use crate::temporal::Instant;

    fn source_metadata(record_key: &str, version: Option<&str>) -> SourceRecordMetadata {
        SourceRecordMetadata::new(
            SourceId::new("source-mapping").unwrap(),
            SourceClass::structured_data(),
            record_key,
            version.map(str::to_owned),
            None,
            [],
        )
        .unwrap()
    }

    fn candidate_key(record_key: &str, ordinal: u32) -> CandidateKey {
        CandidateKey::new(
            SourceId::new("source-mapping").unwrap(),
            record_key,
            Some("snapshot-1".to_owned()),
            ordinal,
        )
        .unwrap()
    }

    #[test]
    fn mapping_preserves_source_and_stage_version_metadata() {
        let normalized = NormalizedRecord::new(
            source_metadata("record-1", Some("snapshot-1")),
            "normalized payload".to_owned(),
            NormalizationMetadata::new("normalizer-v1", "config-v1", Instant::from_unix_seconds(1))
                .unwrap(),
        )
        .unwrap();
        let candidate = MappedCandidate::from_normalized(
            normalized,
            0,
            vec!["entity-candidate".to_owned()],
            MappingMetadata::new("mapper-v1", "mapping-config-v1").unwrap(),
        )
        .unwrap();

        assert_eq!(candidate.key(), &candidate_key("record-1", 0));
        assert_eq!(candidate.source().source_record_key(), "record-1");
        assert_eq!(candidate.normalization().stage_version(), "normalizer-v1");
        assert_eq!(candidate.mapping().stage_version(), "mapper-v1");
        assert!(!candidate.requires_resolution());
    }

    #[test]
    fn mapping_preserves_source_scoped_external_identifiers() {
        let external = ExternalIdentifier::new(
            SourceId::new("source-mapping").unwrap(),
            "external-person-42",
        )
        .unwrap();
        let metadata = SourceRecordMetadata::new(
            SourceId::new("source-mapping").unwrap(),
            SourceClass::structured_data(),
            "record-external",
            Some("snapshot-1".to_owned()),
            None,
            [external.clone()],
        )
        .unwrap();
        let normalized = NormalizedRecord::new(
            metadata,
            "payload".to_owned(),
            NormalizationMetadata::new("normalizer-v1", "config-v1", Instant::from_unix_seconds(1))
                .unwrap(),
        )
        .unwrap();
        let candidate = MappedCandidate::from_normalized(
            normalized,
            0,
            (),
            MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
        )
        .unwrap();

        assert!(candidate.external_identifiers().contains(&external));
    }

    #[test]
    fn candidate_rejects_external_identifiers_owned_by_another_source() {
        let normalized = NormalizedRecord::new(
            source_metadata("record-external", Some("snapshot-1")),
            (),
            NormalizationMetadata::new("normalizer-v1", "config-v1", Instant::from_unix_seconds(1))
                .unwrap(),
        )
        .unwrap();
        let candidate = MappedCandidate::from_normalized(
            normalized,
            0,
            (),
            MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
        )
        .unwrap();
        let external =
            ExternalIdentifier::new(SourceId::new("source-other").unwrap(), "external-person-42")
                .unwrap();

        assert_eq!(
            candidate.with_external_identifier(external),
            Err(MappingError::ExternalIdentifierSourceMismatch {
                expected: SourceId::new("source-mapping").unwrap(),
                actual: SourceId::new("source-other").unwrap(),
            })
        );
    }

    #[test]
    fn source_deduplication_is_snapshot_scoped_and_idempotent() {
        let first = source_metadata("record-1", Some("snapshot-1"));
        let mut deduplicator = SourceDeduplicator::new();
        assert_eq!(
            deduplicator.insert(&first).unwrap(),
            DeduplicationResult::FirstSeen
        );
        assert!(matches!(
            deduplicator.insert(&first).unwrap(),
            DeduplicationResult::Duplicate { .. }
        ));
        assert_eq!(
            deduplicator
                .insert(&source_metadata("record-1", Some("snapshot-2")))
                .unwrap(),
            DeduplicationResult::FirstSeen
        );
        assert_eq!(deduplicator.len(), 2);
    }

    #[test]
    fn semantic_deduplication_retains_the_first_candidate_mapping() {
        let key = SemanticDeduplicationKey::new([
            CanonicalSemanticIdentity::Entity(EntityId::new("entity-1").unwrap()),
            CanonicalSemanticIdentity::Concept(ConceptId::new("concept-1").unwrap()),
        ])
        .unwrap();
        let mut deduplicator = SemanticDeduplicator::new();
        let first = candidate_key("record-a", 0);
        let second = candidate_key("record-b", 0);

        assert_eq!(
            deduplicator.insert(key.clone(), first.clone()),
            DeduplicationResult::FirstSeen
        );
        assert_eq!(
            deduplicator.insert(key, second),
            DeduplicationResult::Duplicate { existing: first }
        );
    }

    #[test]
    fn invalid_semantic_deduplication_keys_are_rejected() {
        assert!(SemanticDeduplicationKey::new([]).is_err());
        assert!(
            SemanticDeduplicationKey::new([CanonicalSemanticIdentity::Opaque {
                type_name: " ".to_owned(),
                identifier: "opaque-1".to_owned(),
            }])
            .is_err()
        );
    }
}
