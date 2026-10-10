//! Logical per-candidate publication and Indexing-readiness coordination.
//!
//! A publication record contains the complete typed semantic subgraph accepted
//! by the logical KG boundary, not only a list of assertions. This module does
//! not provide physical persistence, transactions, storage versioning, index
//! activation, or Indexing lifecycle behavior.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::assertion::{AssertionObject, KnowledgeAssertion, KnowledgeAssertionValidationError};
use crate::concept::Concept;
use crate::entity::{Entity, ExternalIdentifier, Mention};
use crate::evidence::{Evidence, EvidenceSupport, EvidenceValidationError};
use crate::identity::{
    AgentId, ConceptId, EntityId, EvidenceId, KnowledgeAssertionId, LexicalFormId, MentionId,
    ReferenceId, SourceId,
};
use crate::provenance::ProvenanceRecord;
use crate::resolution::ExternalIdentifierCrosswalk;
use crate::source::{Reference, Source, SourceError};
use crate::temporal::Instant;
use crate::uncertainty::Contradiction;

use crate::integration::indexing::{
    IndexingIntegrationError, IndexingPublicationReadiness, IndexingReadinessReceipt,
    IndexingSynchronizationRecord, IndexingSynchronizationStatus,
};

use super::approval::{ApprovalDecision, ApprovalError, ApprovalPolicy, SourceAuthenticity};
use super::mapping::{CandidateKey, MappedCandidate};
use super::validation::ValidationResult;

/// All typed semantic values accepted as one logical publication unit.
///
/// Values that already exist canonically may be referenced by candidate-local
/// links without being duplicated in these vectors. The `existing_*` identity
/// sets document those external dependencies for validation of references such
/// as evidence support and crosswalks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CanonicalKnowledgeSubgraph {
    entities: Vec<Entity>,
    mentions: Vec<Mention>,
    concepts: Vec<Concept>,
    sources: Vec<Source>,
    references: Vec<Reference>,
    assertions: Vec<KnowledgeAssertion>,
    evidence: Vec<Evidence>,
    evidence_supports: Vec<EvidenceSupport>,
    provenance_records: Vec<ProvenanceRecord>,
    external_identifier_crosswalks: Vec<ExternalIdentifierCrosswalk>,
    contradictions: Vec<Contradiction>,
    existing_entity_ids: BTreeSet<EntityId>,
    existing_concept_ids: BTreeSet<ConceptId>,
    existing_mention_ids: BTreeSet<MentionId>,
    existing_source_ids: BTreeSet<SourceId>,
    existing_reference_ids: BTreeSet<ReferenceId>,
    existing_lexical_form_ids: BTreeSet<LexicalFormId>,
    existing_assertion_ids: BTreeSet<KnowledgeAssertionId>,
    existing_evidence_ids: BTreeSet<EvidenceId>,
    source_authenticity: BTreeMap<SourceId, SourceAuthenticity>,
}

impl CanonicalKnowledgeSubgraph {
    /// Creates an empty builder. A publishable candidate must add at least one
    /// semantic object; empty publication payloads are rejected.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds entities to the candidate-local semantic subgraph.
    #[must_use]
    pub fn with_entities<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Entity>,
    {
        self.entities.extend(values);
        self
    }

    /// Adds mentions to the candidate-local semantic subgraph.
    #[must_use]
    pub fn with_mentions<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Mention>,
    {
        self.mentions.extend(values);
        self
    }

    /// Adds concepts to the candidate-local semantic subgraph.
    #[must_use]
    pub fn with_concepts<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Concept>,
    {
        self.concepts.extend(values);
        self
    }

    /// Adds sources to the candidate-local semantic subgraph.
    #[must_use]
    pub fn with_sources<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Source>,
    {
        self.sources.extend(values);
        self
    }

    /// Adds generic references to the candidate-local semantic subgraph.
    #[must_use]
    pub fn with_references<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Reference>,
    {
        self.references.extend(values);
        self
    }

    /// Adds canonical knowledge assertions to the candidate-local subgraph.
    #[must_use]
    pub fn with_assertions<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = KnowledgeAssertion>,
    {
        self.assertions.extend(values);
        self
    }

    /// Adds evidence objects to the candidate-local subgraph.
    #[must_use]
    pub fn with_evidence<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Evidence>,
    {
        self.evidence.extend(values);
        self
    }

    /// Adds role-qualified evidence/assertion links.
    #[must_use]
    pub fn with_evidence_supports<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = EvidenceSupport>,
    {
        self.evidence_supports.extend(values);
        self
    }

    /// Adds historical knowledge-provenance records.
    #[must_use]
    pub fn with_provenance_records<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = ProvenanceRecord>,
    {
        self.provenance_records.extend(values);
        self
    }

    /// Adds source-to-canonical external-identifier crosswalks.
    #[must_use]
    pub fn with_external_identifier_crosswalks<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = ExternalIdentifierCrosswalk>,
    {
        self.external_identifier_crosswalks.extend(values);
        self
    }

    /// Adds first-class contradiction records that belong to the publication.
    #[must_use]
    pub fn with_contradictions<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Contradiction>,
    {
        self.contradictions.extend(values);
        self
    }

    /// Declares entity identities already present in canonical state and therefore
    /// permitted as references without re-publishing the entity.
    #[must_use]
    pub fn with_existing_entity_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.existing_entity_ids.extend(ids);
        self
    }

    /// Declares concept identities already present in canonical state and therefore
    /// permitted as assertion subject/object references without republishing them.
    #[must_use]
    pub fn with_existing_concept_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = ConceptId>,
    {
        self.existing_concept_ids.extend(ids);
        self
    }

    /// Declares mention identities already present in canonical state.
    #[must_use]
    pub fn with_existing_mention_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = MentionId>,
    {
        self.existing_mention_ids.extend(ids);
        self
    }

    /// Declares source identities already present in canonical state.
    #[must_use]
    pub fn with_existing_source_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = SourceId>,
    {
        self.existing_source_ids.extend(ids);
        self
    }

    /// Declares reference identities already present in canonical state.
    #[must_use]
    pub fn with_existing_reference_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = ReferenceId>,
    {
        self.existing_reference_ids.extend(ids);
        self
    }

    /// Declares lexical-form identities already present in canonical state.
    ///
    /// Phase 5 does not materialize lexical forms inside the publication subgraph,
    /// so lexical-form assertion endpoints must be declared as existing.
    #[must_use]
    pub fn with_existing_lexical_form_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = LexicalFormId>,
    {
        self.existing_lexical_form_ids.extend(ids);
        self
    }

    /// Records explicit authenticity status for a source represented by this publication.
    #[must_use]
    pub fn with_source_authenticity(
        mut self,
        source_id: SourceId,
        authenticity: SourceAuthenticity,
    ) -> Self {
        self.source_authenticity.insert(source_id, authenticity);
        self
    }

    /// Returns the declared authenticity status for one source, if present.
    #[must_use]
    pub fn source_authenticity_for(&self, source_id: &SourceId) -> Option<SourceAuthenticity> {
        self.source_authenticity.get(source_id).copied()
    }

    /// Returns all explicit source-authenticity status metadata.
    #[must_use]
    pub fn source_authenticity_records(&self) -> &BTreeMap<SourceId, SourceAuthenticity> {
        &self.source_authenticity
    }

    /// Declares assertion identities already present in canonical state.
    #[must_use]
    pub fn with_existing_assertion_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = KnowledgeAssertionId>,
    {
        self.existing_assertion_ids.extend(ids);
        self
    }

    /// Declares evidence identities already present in canonical state.
    #[must_use]
    pub fn with_existing_evidence_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.existing_evidence_ids.extend(ids);
        self
    }

    /// Returns candidate-local entities.
    #[must_use]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
    /// Returns candidate-local mentions.
    #[must_use]
    pub fn mentions(&self) -> &[Mention] {
        &self.mentions
    }
    /// Returns candidate-local concepts.
    #[must_use]
    pub fn concepts(&self) -> &[Concept] {
        &self.concepts
    }
    /// Returns candidate-local sources.
    #[must_use]
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }
    /// Returns candidate-local generic references.
    #[must_use]
    pub fn references(&self) -> &[Reference] {
        &self.references
    }
    /// Returns candidate-local assertions.
    #[must_use]
    pub fn assertions(&self) -> &[KnowledgeAssertion] {
        &self.assertions
    }
    /// Returns candidate-local evidence.
    #[must_use]
    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }
    /// Returns evidence/assertion associations.
    #[must_use]
    pub fn evidence_supports(&self) -> &[EvidenceSupport] {
        &self.evidence_supports
    }
    /// Returns provenance records included in the publication.
    #[must_use]
    pub fn provenance_records(&self) -> &[ProvenanceRecord] {
        &self.provenance_records
    }
    /// Returns external identifier crosswalks.
    #[must_use]
    pub fn external_identifier_crosswalks(&self) -> &[ExternalIdentifierCrosswalk] {
        &self.external_identifier_crosswalks
    }
    /// Returns contradiction records included in the publication.
    #[must_use]
    pub fn contradictions(&self) -> &[Contradiction] {
        &self.contradictions
    }

    /// Returns whether the payload contains no semantic objects or relationships.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
            && self.mentions.is_empty()
            && self.concepts.is_empty()
            && self.sources.is_empty()
            && self.references.is_empty()
            && self.assertions.is_empty()
            && self.evidence.is_empty()
            && self.evidence_supports.is_empty()
            && self.provenance_records.is_empty()
            && self.external_identifier_crosswalks.is_empty()
            && self.contradictions.is_empty()
    }

    /// Validates value-level invariants and relevant candidate-local references.
    pub fn validate(&self) -> Result<(), SubgraphValidationError> {
        if self.is_empty() {
            return Err(SubgraphValidationError::EmptySubgraph);
        }
        ensure_unique(self.entities.iter().map(|value| value.id()), "entity")?;
        ensure_unique(self.mentions.iter().map(|value| value.id()), "mention")?;
        ensure_unique(self.concepts.iter().map(|value| value.id()), "concept")?;
        ensure_unique(self.sources.iter().map(|value| value.id()), "source")?;
        ensure_unique(self.references.iter().map(|value| value.id()), "reference")?;
        ensure_unique(self.assertions.iter().map(|value| value.id()), "assertion")?;
        ensure_unique(self.evidence.iter().map(|value| value.id()), "evidence")?;
        ensure_unique(
            self.contradictions.iter().map(|value| value.id()),
            "contradiction",
        )?;

        for source in &self.sources {
            source
                .validate()
                .map_err(SubgraphValidationError::InvalidSource)?;
        }
        for assertion in &self.assertions {
            assertion
                .validate()
                .map_err(SubgraphValidationError::InvalidAssertion)?;
        }
        for evidence in &self.evidence {
            evidence
                .validate()
                .map_err(SubgraphValidationError::InvalidEvidence)?;
        }

        let local_assertions = self
            .assertions
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();
        let local_evidence = self
            .evidence
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();
        for support in &self.evidence_supports {
            if !local_assertions.contains(support.assertion_id())
                && !self.existing_assertion_ids.contains(support.assertion_id())
            {
                return Err(SubgraphValidationError::UnknownEvidenceSupportAssertion {
                    id: support.assertion_id().clone(),
                });
            }
            if !local_evidence.contains(support.evidence_id())
                && !self.existing_evidence_ids.contains(support.evidence_id())
            {
                return Err(SubgraphValidationError::UnknownEvidenceSupportEvidence {
                    id: support.evidence_id().clone(),
                });
            }
        }

        let local_entities = self
            .entities
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();
        let local_concepts = self
            .concepts
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();
        let local_mentions = self
            .mentions
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();
        let local_sources = self
            .sources
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();
        let local_references = self
            .references
            .iter()
            .map(|value| value.id().clone())
            .collect::<BTreeSet<_>>();

        let known_entities = local_entities
            .union(&self.existing_entity_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let known_concepts = local_concepts
            .union(&self.existing_concept_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let known_mentions = local_mentions
            .union(&self.existing_mention_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let known_sources = local_sources
            .union(&self.existing_source_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let known_references = local_references
            .union(&self.existing_reference_ids)
            .cloned()
            .collect::<BTreeSet<_>>();

        for assertion in &self.assertions {
            for object in [assertion.subject(), assertion.object()] {
                match object {
                    AssertionObject::Entity(id) if !known_entities.contains(id) => {
                        return Err(SubgraphValidationError::UnknownAssertionEntity {
                            id: id.clone(),
                        });
                    }
                    AssertionObject::Concept(id) if !known_concepts.contains(id) => {
                        return Err(SubgraphValidationError::UnknownAssertionConcept {
                            id: id.clone(),
                        });
                    }
                    AssertionObject::Mention(id) if !known_mentions.contains(id) => {
                        return Err(SubgraphValidationError::UnknownAssertionMention {
                            id: id.clone(),
                        });
                    }
                    AssertionObject::Source(id) if !known_sources.contains(id) => {
                        return Err(SubgraphValidationError::UnknownAssertionSource {
                            id: id.clone(),
                        });
                    }
                    AssertionObject::Reference(id) if !known_references.contains(id) => {
                        return Err(SubgraphValidationError::UnknownAssertionReference {
                            id: id.clone(),
                        });
                    }
                    AssertionObject::LexicalForm(id)
                        if !self.existing_lexical_form_ids.contains(id) =>
                    {
                        return Err(SubgraphValidationError::UnknownAssertionLexicalForm {
                            id: id.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }

        for source_id in self.source_authenticity.keys() {
            if !known_sources.contains(source_id) {
                return Err(SubgraphValidationError::UnknownSourceAuthenticitySource {
                    id: source_id.clone(),
                });
            }
        }
        // Entity-owned external identifiers and crosswalk records must agree within
        // this candidate subgraph. A source-scoped ID must not identify two entities.
        let mut crosswalk_targets = BTreeMap::<ExternalIdentifier, EntityId>::new();
        for entity in &self.entities {
            for external in entity.external_identifiers() {
                if let Some(existing) = crosswalk_targets.get(external)
                    && existing != entity.id()
                {
                    return Err(SubgraphValidationError::ConflictingExternalCrosswalk);
                }
                crosswalk_targets.insert(external.clone(), entity.id().clone());
            }
        }
        for crosswalk in &self.external_identifier_crosswalks {
            let external = (*crosswalk.external_identifier()).clone();
            let entity = crosswalk.canonical_entity().clone();
            if !local_entities.contains(&entity) && !self.existing_entity_ids.contains(&entity) {
                return Err(SubgraphValidationError::UnknownCrosswalkEntity { id: entity });
            }
            if let Some(existing) = crosswalk_targets.get(&external)
                && existing != &entity
            {
                return Err(SubgraphValidationError::ConflictingExternalCrosswalk);
            }
            crosswalk_targets.insert(external, entity);
        }

        let all_assertions = local_assertions
            .union(&self.existing_assertion_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        for contradiction in &self.contradictions {
            if let Some(missing) = contradiction
                .assertion_ids()
                .iter()
                .find(|id| !all_assertions.contains(*id))
            {
                return Err(SubgraphValidationError::UnknownContradictionAssertion {
                    id: (*missing).clone(),
                });
            }
        }
        Ok(())
    }

    /// Returns the total count of candidate-local semantic objects and links.
    #[must_use]
    pub fn object_count(&self) -> usize {
        self.entities.len()
            + self.mentions.len()
            + self.concepts.len()
            + self.sources.len()
            + self.references.len()
            + self.assertions.len()
            + self.evidence.len()
            + self.evidence_supports.len()
            + self.provenance_records.len()
            + self.external_identifier_crosswalks.len()
            + self.contradictions.len()
    }
}

/// Immutable exact-content revision token for validation and approval decisions.
///
/// Clones share a captured typed subgraph. Publication compares the captured
/// content structurally with the proposal, ensuring content changes invalidate
/// stale review decisions without relying on a lossy hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubgraphRevision(Arc<CanonicalKnowledgeSubgraph>);

impl SubgraphRevision {
    /// Captures the exact proposed subgraph for a later validation/approval binding.
    #[must_use]
    pub fn capture(subgraph: &CanonicalKnowledgeSubgraph) -> Self {
        Self(Arc::new(subgraph.clone()))
    }

    /// Returns whether a proposed subgraph is exactly the captured content.
    #[must_use]
    pub fn matches(&self, subgraph: &CanonicalKnowledgeSubgraph) -> bool {
        self.0.as_ref() == subgraph
    }
}

/// Outcome recorded for one candidate's logical publication decision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PublicationOutcome {
    /// The validated, approved candidate was accepted into canonical logical state.
    Published,
    /// Publication was blocked by a precondition.
    Blocked,
    /// A governance decision rejected the candidate.
    Rejected,
    /// An already-published candidate was withdrawn through a separate governed operation.
    Withdrawn,
}

/// Immutable publication decision value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationDecision {
    candidate: CandidateKey,
    outcome: PublicationOutcome,
    decided_at: Instant,
    reason: String,
}

impl PublicationDecision {
    /// Creates a publication outcome record.
    pub fn new(
        candidate: CandidateKey,
        outcome: PublicationOutcome,
        decided_at: Instant,
        reason: impl Into<String>,
    ) -> Result<Self, PublicationError> {
        let reason = reason.into();
        validate_label(&reason, "publication decision reason")?;
        Ok(Self {
            candidate,
            outcome,
            decided_at,
            reason,
        })
    }

    /// Returns the candidate identity.
    #[must_use]
    pub fn candidate(&self) -> &CandidateKey {
        &self.candidate
    }
    /// Returns the recorded outcome.
    #[must_use]
    pub const fn outcome(&self) -> PublicationOutcome {
        self.outcome
    }
    /// Returns the decision timestamp.
    #[must_use]
    pub const fn decided_at(&self) -> Instant {
        self.decided_at
    }
    /// Returns the reason for the decision.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Successful logical publication record and its independent Indexing synchronization status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationRecord {
    decision: PublicationDecision,
    subgraph: CanonicalKnowledgeSubgraph,
    readiness: IndexingReadinessReceipt,
    source_authenticity: SourceAuthenticity,
    synchronization: IndexingSynchronizationRecord,
}

impl PublicationRecord {
    fn new(
        decision: PublicationDecision,
        subgraph: CanonicalKnowledgeSubgraph,
        readiness: IndexingReadinessReceipt,
        source_authenticity: SourceAuthenticity,
    ) -> Self {
        let synchronization = IndexingSynchronizationRecord::new(
            readiness.assigned_id().clone(),
            readiness.object_reference().clone(),
            IndexingSynchronizationStatus::Pending,
        );
        Self {
            decision,
            subgraph,
            readiness,
            source_authenticity,
            synchronization,
        }
    }

    /// Returns the logical publication decision.
    #[must_use]
    pub fn decision(&self) -> &PublicationDecision {
        &self.decision
    }
    /// Returns the complete accepted semantic subgraph.
    #[must_use]
    pub fn subgraph(&self) -> &CanonicalKnowledgeSubgraph {
        &self.subgraph
    }
    /// Returns the readiness acknowledgement used as a publication precondition.
    #[must_use]
    pub fn readiness(&self) -> &IndexingReadinessReceipt {
        &self.readiness
    }
    /// Returns the source-authenticity status retained with this publication.
    #[must_use]
    pub const fn source_authenticity(&self) -> SourceAuthenticity {
        self.source_authenticity
    }
    /// Returns the latest observed post-publication synchronization state.
    #[must_use]
    pub fn synchronization(&self) -> &IndexingSynchronizationRecord {
        &self.synchronization
    }

    /// Returns a new record carrying the latest synchronization state. It does
    /// not revise whether logical KG publication succeeded.
    #[must_use]
    pub fn with_synchronization_status(mut self, status: IndexingSynchronizationStatus) -> Self {
        self.synchronization = IndexingSynchronizationRecord::new(
            self.readiness.assigned_id().clone(),
            self.readiness.object_reference().clone(),
            status,
        );
        self
    }
}

/// Immutable record of a governed withdrawal of an earlier publication.
///
/// The original publication decision is preserved. This record does not delete
/// or rewrite the published subgraph; physical/history storage is a later phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalRecord {
    prior_publication: PublicationDecision,
    decision: PublicationDecision,
    withdrawn_by: AgentId,
}

impl WithdrawalRecord {
    /// Returns the original publication decision being withdrawn.
    #[must_use]
    pub fn prior_publication(&self) -> &PublicationDecision {
        &self.prior_publication
    }
    /// Returns the separate withdrawal decision.
    #[must_use]
    pub fn decision(&self) -> &PublicationDecision {
        &self.decision
    }
    /// Returns the agent responsible for the withdrawal.
    #[must_use]
    pub fn withdrawn_by(&self) -> &AgentId {
        &self.withdrawn_by
    }
}

/// Immutable link recording a correction as a governed successor candidate.
///
/// It does not mutate the predecessor or publish the successor. The successor
/// must pass the regular validation, approval, and publication gates itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectionRecord {
    prior_publication: PublicationDecision,
    successor_candidate: CandidateKey,
    recorded_at: Instant,
    recorded_by: AgentId,
    reason: String,
}

impl CorrectionRecord {
    /// Returns the earlier canonical publication decision being corrected.
    #[must_use]
    pub fn prior_publication(&self) -> &PublicationDecision {
        &self.prior_publication
    }
    /// Returns the new candidate that must pass its own governance path.
    #[must_use]
    pub fn successor_candidate(&self) -> &CandidateKey {
        &self.successor_candidate
    }
    /// Returns when the correction relationship was recorded.
    #[must_use]
    pub const fn recorded_at(&self) -> Instant {
        self.recorded_at
    }
    /// Returns the actor who recorded the correction.
    #[must_use]
    pub fn recorded_by(&self) -> &AgentId {
        &self.recorded_by
    }
    /// Returns the correction rationale.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Inputs required to decide whether one candidate can be logically published.
///
/// Grouping these related gate inputs into one request keeps the coordinator's
/// API explicit without an oversized method signature. References are borrowed
/// for the duration of the decision; the subgraph is consumed into the result.
pub struct PublicationRequest<'a, T> {
    /// Candidate being considered for publication.
    pub candidate: &'a MappedCandidate<T>,
    /// Complete semantic subgraph proposed for canonical acceptance.
    pub subgraph: CanonicalKnowledgeSubgraph,
    /// Validation outcome for this candidate.
    pub validation: &'a ValidationResult,
    /// Source-class governance policy.
    pub approval_policy: &'a ApprovalPolicy,
    /// Recorded human approval decision.
    pub approval: &'a ApprovalDecision,
    /// Source authenticity classification used by governance.
    pub authenticity: SourceAuthenticity,
    /// Indexing readiness acknowledgement required before publication.
    pub readiness: &'a IndexingReadinessReceipt,
    /// Time the logical publication decision is recorded.
    pub published_at: Instant,
}

/// Phase 5 logical publication gate. It checks semantics and prerequisites but
/// writes no physical storage and does not activate an index.
#[derive(Clone, Copy, Debug, Default)]
pub struct PublicationCoordinator;

impl PublicationCoordinator {
    /// Validates and logically publishes one mapped candidate's semantic subgraph.
    pub fn publish<T>(
        &self,
        request: PublicationRequest<'_, T>,
    ) -> Result<PublicationRecord, PublicationError> {
        let PublicationRequest {
            candidate,
            subgraph,
            validation,
            approval_policy,
            approval,
            authenticity,
            readiness,
            published_at,
        } = request;

        if validation.candidate() != candidate.key() {
            return Err(PublicationError::CandidateMismatch);
        }
        subgraph
            .validate()
            .map_err(PublicationError::InvalidSubgraph)?;

        if !validation
            .publication_revision()
            .is_some_and(|revision| revision.matches(&subgraph))
            || !approval
                .publication_revision()
                .is_some_and(|revision| revision.matches(&subgraph))
        {
            return Err(PublicationError::ReviewedSubgraphMismatch);
        }

        let source_id = candidate.source().source_id();
        let recorded_authenticity =
            subgraph.source_authenticity_for(source_id).ok_or_else(|| {
                PublicationError::MissingSourceAuthenticityMetadata {
                    source_id: source_id.clone(),
                }
            })?;
        if recorded_authenticity != authenticity {
            return Err(PublicationError::SourceAuthenticityMismatch {
                source_id: source_id.clone(),
                expected: authenticity,
                recorded: recorded_authenticity,
            });
        }

        approval_policy
            .validate_for_publication(candidate, validation, approval, authenticity)
            .map_err(PublicationError::Approval)?;
        readiness
            .require_ready()
            .map_err(PublicationError::Indexing)?;

        if readiness.object_reference().source() != candidate.source().source_id().as_str()
            || readiness.object_reference().object_reference()
                != candidate.source().source_record_key()
        {
            return Err(PublicationError::IndexingObjectReferenceMismatch);
        }
        if !matches!(readiness.readiness(), IndexingPublicationReadiness::Ready) {
            return Err(PublicationError::Indexing(
                IndexingIntegrationError::PublicationNotReady,
            ));
        }

        let decision = PublicationDecision::new(
            candidate.key().clone(),
            PublicationOutcome::Published,
            published_at,
            "candidate passed validation, approval, semantic-subgraph, and Indexing readiness gates",
        )?;
        Ok(PublicationRecord::new(
            decision,
            subgraph,
            readiness.clone(),
            authenticity,
        ))
    }

    /// Records a governed withdrawal without mutating the prior publication.
    pub fn withdraw(
        &self,
        prior: &PublicationRecord,
        withdrawn_by: AgentId,
        withdrawn_at: Instant,
        reason: impl Into<String>,
    ) -> Result<WithdrawalRecord, PublicationError> {
        if prior.decision().outcome() != PublicationOutcome::Published {
            return Err(PublicationError::PriorPublicationNotPublished);
        }
        let decision = PublicationDecision::new(
            prior.decision().candidate().clone(),
            PublicationOutcome::Withdrawn,
            withdrawn_at,
            reason,
        )?;
        Ok(WithdrawalRecord {
            prior_publication: prior.decision().clone(),
            decision,
            withdrawn_by,
        })
    }

    /// Records that a new candidate is a governed correction to an earlier
    /// publication. The successor still has to pass the ordinary publication gate.
    pub fn record_correction<T>(
        &self,
        prior: &PublicationRecord,
        successor: &MappedCandidate<T>,
        recorded_by: AgentId,
        recorded_at: Instant,
        reason: impl Into<String>,
    ) -> Result<CorrectionRecord, PublicationError> {
        if prior.decision().outcome() != PublicationOutcome::Published {
            return Err(PublicationError::PriorPublicationNotPublished);
        }
        if prior.decision().candidate() == successor.key() {
            return Err(PublicationError::CorrectionCandidateMustDiffer);
        }
        let reason = reason.into();
        validate_label(&reason, "correction reason")?;
        Ok(CorrectionRecord {
            prior_publication: prior.decision().clone(),
            successor_candidate: successor.key().clone(),
            recorded_at,
            recorded_by,
            reason,
        })
    }
}

/// Semantic-subgraph consistency errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubgraphValidationError {
    /// A publication must contain at least one semantic object or relationship.
    EmptySubgraph,
    /// Two candidate-local values have the same identity within one type.
    DuplicateIdentity { object_type: &'static str },
    /// A source's own Phase 4 metadata is invalid.
    InvalidSource(SourceError),
    /// A canonical assertion fails its own structural validation.
    InvalidAssertion(KnowledgeAssertionValidationError),
    /// An evidence record fails its own structural validation.
    InvalidEvidence(EvidenceValidationError),
    /// An evidence support points to an unknown assertion.
    UnknownEvidenceSupportAssertion { id: KnowledgeAssertionId },
    /// An evidence support points to unknown evidence.
    UnknownEvidenceSupportEvidence { id: EvidenceId },
    /// An assertion refers to an entity absent from this subgraph and canonical state.
    UnknownAssertionEntity { id: EntityId },
    /// An assertion refers to a concept absent from this subgraph and canonical state.
    UnknownAssertionConcept { id: ConceptId },
    /// An assertion refers to a mention absent from this subgraph and canonical state.
    UnknownAssertionMention { id: MentionId },
    /// An assertion refers to a source absent from this subgraph and canonical state.
    UnknownAssertionSource { id: SourceId },
    /// An assertion refers to a reference absent from this subgraph and canonical state.
    UnknownAssertionReference { id: ReferenceId },
    /// An assertion refers to a lexical form not declared in canonical state.
    UnknownAssertionLexicalForm { id: LexicalFormId },
    /// Authenticity metadata refers to a source absent from local or declared state.
    UnknownSourceAuthenticitySource { id: SourceId },
    /// A crosswalk points to an entity absent from this subgraph and canonical state.
    UnknownCrosswalkEntity { id: EntityId },
    /// One external identifier maps to multiple canonical entities in this subgraph.
    ConflictingExternalCrosswalk,
    /// A contradiction points to an assertion absent from this subgraph and canonical state.
    UnknownContradictionAssertion { id: KnowledgeAssertionId },
}

impl fmt::Display for SubgraphValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySubgraph => formatter.write_str("publication subgraph must not be empty"),
            Self::DuplicateIdentity { object_type } => write!(
                formatter,
                "duplicate {object_type} identity in publication subgraph"
            ),
            Self::InvalidSource(error) => write!(formatter, "invalid publication source: {error}"),
            Self::InvalidAssertion(error) => {
                write!(formatter, "invalid publication assertion: {error}")
            }
            Self::InvalidEvidence(error) => {
                write!(formatter, "invalid publication evidence: {error}")
            }
            Self::UnknownEvidenceSupportAssertion { id } => write!(
                formatter,
                "evidence support references unknown assertion {id}"
            ),
            Self::UnknownEvidenceSupportEvidence { id } => write!(
                formatter,
                "evidence support references unknown evidence {id}"
            ),
            Self::UnknownAssertionEntity { id } => {
                write!(formatter, "assertion references unknown entity {id}")
            }
            Self::UnknownAssertionConcept { id } => {
                write!(formatter, "assertion references unknown concept {id}")
            }
            Self::UnknownAssertionMention { id } => {
                write!(formatter, "assertion references unknown mention {id}")
            }
            Self::UnknownAssertionSource { id } => {
                write!(formatter, "assertion references unknown source {id}")
            }
            Self::UnknownAssertionReference { id } => {
                write!(formatter, "assertion references unknown reference {id}")
            }
            Self::UnknownAssertionLexicalForm { id } => {
                write!(formatter, "assertion references unknown lexical form {id}")
            }
            Self::UnknownSourceAuthenticitySource { id } => {
                write!(
                    formatter,
                    "source authenticity metadata references unknown source {id}"
                )
            }
            Self::UnknownCrosswalkEntity { id } => write!(
                formatter,
                "external crosswalk references unknown entity {id}"
            ),
            Self::ConflictingExternalCrosswalk => formatter.write_str(
                "one source-scoped external identifier maps to multiple canonical entities",
            ),
            Self::UnknownContradictionAssertion { id } => {
                write!(formatter, "contradiction references unknown assertion {id}")
            }
        }
    }
}
impl std::error::Error for SubgraphValidationError {}

/// Publication gate failures. Index synchronization failure is deliberately not
/// represented as a publication failure because it occurs after logical publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublicationError {
    /// Validation or approval refers to a different candidate.
    CandidateMismatch,
    /// The candidate did not satisfy governance policy.
    Approval(ApprovalError),
    /// Validation and approval were not bound to the exact proposed semantic subgraph.
    ReviewedSubgraphMismatch,
    /// Source authenticity must be explicitly retained in the proposed subgraph.
    MissingSourceAuthenticityMetadata { source_id: SourceId },
    /// The passed authenticity classification differs from the subgraph's explicit status.
    SourceAuthenticityMismatch {
        source_id: SourceId,
        expected: SourceAuthenticity,
        recorded: SourceAuthenticity,
    },
    /// The proposed semantic subgraph is invalid.
    InvalidSubgraph(SubgraphValidationError),
    /// Indexing did not acknowledge a publishable readiness state.
    Indexing(IndexingIntegrationError),
    /// The Indexing readiness receipt belongs to a different source record.
    IndexingObjectReferenceMismatch,
    /// Withdrawal or correction may reference only an accepted prior publication.
    PriorPublicationNotPublished,
    /// A correction must identify a distinct successor candidate.
    CorrectionCandidateMustDiffer,
    /// A publication/correction reason is empty or contains control characters.
    InvalidLabel { field: &'static str },
}

impl fmt::Display for PublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CandidateMismatch => {
                formatter.write_str("publication validation result targets a different candidate")
            }
            Self::Approval(error) => write!(formatter, "publication approval gate failed: {error}"),
            Self::ReviewedSubgraphMismatch => formatter
                .write_str("validation and approval must be bound to the exact proposed subgraph"),
            Self::MissingSourceAuthenticityMetadata { source_id } => write!(
                formatter,
                "publication subgraph is missing explicit authenticity metadata for source {source_id}",
            ),
            Self::SourceAuthenticityMismatch {
                source_id,
                expected,
                recorded,
            } => write!(
                formatter,
                "source authenticity metadata for {source_id} does not match: requested={expected:?}, recorded={recorded:?}",
            ),
            Self::InvalidSubgraph(error) => {
                write!(formatter, "publication subgraph is invalid: {error}")
            }
            Self::Indexing(error) => write!(
                formatter,
                "Indexing publication precondition failed: {error}"
            ),
            Self::IndexingObjectReferenceMismatch => formatter
                .write_str("Indexing readiness receipt does not match the candidate source record"),
            Self::PriorPublicationNotPublished => {
                formatter.write_str("operation requires a previously published candidate")
            }
            Self::CorrectionCandidateMustDiffer => formatter
                .write_str("correction successor must differ from the predecessor candidate"),
            Self::InvalidLabel { field } => write!(formatter, "{field} is invalid"),
        }
    }
}
impl std::error::Error for PublicationError {}

fn ensure_unique<'a, T, I>(ids: I, object_type: &'static str) -> Result<(), SubgraphValidationError>
where
    T: Ord + 'a,
    I: IntoIterator<Item = &'a T>,
{
    let values = ids.into_iter().collect::<Vec<_>>();
    if values.iter().copied().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(SubgraphValidationError::DuplicateIdentity { object_type });
    }
    Ok(())
}

fn validate_label(value: &str, field: &'static str) -> Result<(), PublicationError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(PublicationError::InvalidLabel { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalKnowledgeSubgraph, PublicationCoordinator, PublicationError, PublicationOutcome,
        PublicationRecord, PublicationRequest, SubgraphRevision,
    };
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::concept::Concept;
    use crate::entity::{Entity, ExternalIdentifier, Name};
    use crate::identity::{ConceptId, EntityId, LexicalFormId, MentionId, ReferenceId, SourceId};
    use crate::ingestion::approval::{
        ApprovalDecision, ApprovalOutcome, ApprovalPolicy, CurationOutcome, CurationRequirement,
        SourceApprovalRule, SourceAuthenticity,
    };
    use crate::ingestion::mapping::{MappedCandidate, MappingMetadata};
    use crate::ingestion::normalize::{NormalizationMetadata, NormalizedRecord};
    use crate::ingestion::raw::{SourceClass, SourceRecordMetadata};
    use crate::ingestion::validation::ValidationResult;
    use crate::integration::indexing::{
        IndexingPublicationReadiness, IndexingReadinessReceipt, IndexingSynchronizationStatus,
    };
    use crate::temporal::Instant;
    use nizaam_indexing::IndexAssignedId;
    use nizaam_indexing::TargetReferenceType;
    use nizaam_indexing::index::reference::ObjectReference;

    fn candidate_for(record_key: &str, source_version: &str) -> MappedCandidate<()> {
        let record = NormalizedRecord::new(
            SourceRecordMetadata::new(
                SourceId::new("source-publish").unwrap(),
                SourceClass::structured_data(),
                record_key,
                Some(source_version.to_owned()),
                None,
                [],
            )
            .unwrap(),
            (),
            NormalizationMetadata::new("normalize-v1", "config-v1", Instant::from_unix_seconds(1))
                .unwrap(),
        )
        .unwrap();
        MappedCandidate::from_normalized(
            record,
            0,
            (),
            MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
        )
        .unwrap()
    }

    fn candidate() -> MappedCandidate<()> {
        candidate_for("record-1", "snapshot-1")
    }

    fn readiness_for(record_key: &str) -> IndexingReadinessReceipt {
        let object_reference = ObjectReference::new("source-publish", record_key).unwrap();
        let target_type = TargetReferenceType::new("entity").expect("valid target reference type");
        let assigned_id = IndexAssignedId::generate(&target_type, &object_reference);
        IndexingReadinessReceipt::new(
            assigned_id,
            object_reference,
            IndexingPublicationReadiness::Ready,
        )
    }

    fn readiness() -> IndexingReadinessReceipt {
        readiness_for("record-1")
    }

    fn publish_candidate(
        candidate: &MappedCandidate<()>,
        subgraph: CanonicalKnowledgeSubgraph,
        validation: &ValidationResult,
        approval: &ApprovalDecision,
        readiness: &IndexingReadinessReceipt,
        published_at: Instant,
    ) -> Result<PublicationRecord, PublicationError> {
        let approval_policy = ApprovalPolicy::phase5_default();
        PublicationCoordinator.publish(PublicationRequest {
            candidate,
            subgraph,
            validation,
            approval_policy: &approval_policy,
            approval,
            authenticity: SourceAuthenticity::Authentic,
            readiness,
            published_at,
        })
    }

    fn subgraph_for(
        source_id: &SourceId,
        authenticity: SourceAuthenticity,
    ) -> CanonicalKnowledgeSubgraph {
        let entity_id = EntityId::new("entity-publication").unwrap();
        let concept_id = ConceptId::new("concept-publication").unwrap();
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Entity(entity_id.clone()),
            AssertionPredicate::new("describes").unwrap(),
            AssertionObject::Concept(concept_id.clone()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Known,
            AssertionPolarity::Positive,
        );
        CanonicalKnowledgeSubgraph::new()
            .with_entities([Entity::new(
                entity_id,
                Name::new("Entity", "en"),
                Vec::new(),
            )])
            .with_concepts([Concept::new(concept_id, "concept")])
            .with_assertions([assertion])
            .with_existing_source_ids([source_id.clone()])
            .with_source_authenticity(source_id.clone(), authenticity)
    }

    fn subgraph() -> CanonicalKnowledgeSubgraph {
        subgraph_for(
            &SourceId::new("source-publish").unwrap(),
            SourceAuthenticity::Authentic,
        )
    }

    fn bind_review(
        subgraph: &CanonicalKnowledgeSubgraph,
        validation: ValidationResult,
        approval: ApprovalDecision,
    ) -> (ValidationResult, ApprovalDecision) {
        let revision = SubgraphRevision::capture(subgraph);
        (
            validation.with_publication_revision(revision.clone()),
            approval.with_publication_revision(revision),
        )
    }

    fn subgraph_with_assertion_subject(subject: AssertionObject) -> CanonicalKnowledgeSubgraph {
        let known_entity_id = EntityId::new("endpoint-known-entity").unwrap();
        let assertion = KnowledgeAssertion::new(
            subject,
            AssertionPredicate::new("points-to").unwrap(),
            AssertionObject::Entity(known_entity_id.clone()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Known,
            AssertionPolarity::Positive,
        );
        CanonicalKnowledgeSubgraph::new()
            .with_entities([Entity::new(
                known_entity_id,
                Name::new("Known target", "en"),
                Vec::new(),
            )])
            .with_assertions([assertion])
    }

    #[test]
    fn publication_requires_nonempty_valid_subgraph() {
        assert!(CanonicalKnowledgeSubgraph::new().validate().is_err());
        assert!(subgraph().validate().is_ok());
    }

    #[test]
    fn assertion_concept_references_must_be_local_or_declared_existing() {
        let entity_id = EntityId::new("entity-reference-check").unwrap();
        let concept_id = ConceptId::new("concept-canonical-existing").unwrap();
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Entity(entity_id.clone()),
            AssertionPredicate::new("describes").unwrap(),
            AssertionObject::Concept(concept_id.clone()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Known,
            AssertionPolarity::Positive,
        );
        let incomplete = CanonicalKnowledgeSubgraph::new()
            .with_entities([Entity::new(
                entity_id,
                Name::new("Entity", "en"),
                Vec::new(),
            )])
            .with_assertions([assertion.clone()]);
        assert_eq!(
            incomplete.validate(),
            Err(super::SubgraphValidationError::UnknownAssertionConcept {
                id: concept_id.clone()
            })
        );

        let with_existing = incomplete.with_existing_concept_ids([concept_id]);
        assert!(with_existing.validate().is_ok());
    }

    #[test]
    fn assertions_reject_entity_mention_source_reference_and_lexical_form_endpoints_that_do_not_exist()
     {
        let missing_entity = EntityId::new("endpoint-missing-entity").unwrap();
        assert_eq!(
            subgraph_with_assertion_subject(AssertionObject::Entity(missing_entity.clone()))
                .validate(),
            Err(super::SubgraphValidationError::UnknownAssertionEntity { id: missing_entity })
        );

        let missing_mention = MentionId::new("endpoint-missing-mention").unwrap();
        assert_eq!(
            subgraph_with_assertion_subject(AssertionObject::Mention(missing_mention.clone()))
                .validate(),
            Err(super::SubgraphValidationError::UnknownAssertionMention {
                id: missing_mention,
            })
        );

        let missing_source = SourceId::new("endpoint-missing-source").unwrap();
        assert_eq!(
            subgraph_with_assertion_subject(AssertionObject::Source(missing_source.clone()))
                .validate(),
            Err(super::SubgraphValidationError::UnknownAssertionSource { id: missing_source })
        );

        let missing_reference = ReferenceId::new("endpoint-missing-reference").unwrap();
        assert_eq!(
            subgraph_with_assertion_subject(AssertionObject::Reference(missing_reference.clone()))
                .validate(),
            Err(super::SubgraphValidationError::UnknownAssertionReference {
                id: missing_reference,
            })
        );

        let missing_lexical_form = LexicalFormId::new("endpoint-missing-lexical-form").unwrap();
        assert_eq!(
            subgraph_with_assertion_subject(AssertionObject::LexicalForm(
                missing_lexical_form.clone()
            ))
            .validate(),
            Err(
                super::SubgraphValidationError::UnknownAssertionLexicalForm {
                    id: missing_lexical_form,
                }
            )
        );
    }

    #[test]
    fn assertion_endpoints_can_resolve_through_explicit_existing_identity_sets() {
        let source_id = SourceId::new("endpoint-existing-source").unwrap();
        let mention_id = MentionId::new("endpoint-existing-mention").unwrap();
        let reference_id = ReferenceId::new("endpoint-existing-reference").unwrap();
        let lexical_form_id = LexicalFormId::new("endpoint-existing-lexical-form").unwrap();
        let entity_id = EntityId::new("endpoint-existing-entity").unwrap();
        let concept_id = ConceptId::new("endpoint-existing-concept").unwrap();

        let assertion_for = |subject| {
            KnowledgeAssertion::new(
                subject,
                AssertionPredicate::new("points-to").unwrap(),
                AssertionObject::Entity(entity_id.clone()),
                AssertionContext::new(),
                Qualifiers::new(),
                AssertionStatus::Known,
                AssertionPolarity::Positive,
            )
        };
        let subgraph = CanonicalKnowledgeSubgraph::new()
            .with_assertions([
                assertion_for(AssertionObject::Entity(entity_id.clone())),
                assertion_for(AssertionObject::Concept(concept_id.clone())),
                assertion_for(AssertionObject::Mention(mention_id.clone())),
                assertion_for(AssertionObject::Source(source_id.clone())),
                assertion_for(AssertionObject::Reference(reference_id.clone())),
                assertion_for(AssertionObject::LexicalForm(lexical_form_id.clone())),
            ])
            .with_existing_entity_ids([entity_id])
            .with_existing_concept_ids([concept_id])
            .with_existing_mention_ids([mention_id])
            .with_existing_source_ids([source_id])
            .with_existing_reference_ids([reference_id])
            .with_existing_lexical_form_ids([lexical_form_id]);

        assert!(subgraph.validate().is_ok());
    }

    #[test]
    fn validation_and_approval_cannot_be_reused_for_changed_subgraph_content() {
        let candidate = candidate();
        let reviewed = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::Authentic,
        );
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-stale-subgraph").unwrap(),
            Instant::from_unix_seconds(2),
            CurationOutcome::NotRequired,
            Some("reviewed the original subgraph".to_owned()),
        )
        .unwrap();
        let (validation, approval) = bind_review(
            &reviewed,
            ValidationResult::valid(candidate.key().clone()),
            approval,
        );

        let changed = reviewed.clone().with_concepts([Concept::new(
            ConceptId::new("concept-added-after-review").unwrap(),
            "newly mapped concept",
        )]);
        let result = publish_candidate(
            &candidate,
            changed,
            &validation,
            &approval,
            &readiness(),
            Instant::from_unix_seconds(3),
        );

        assert_eq!(result, Err(PublicationError::ReviewedSubgraphMismatch));
    }

    #[test]
    fn source_authenticity_must_match_subgraph_metadata_and_is_retained() {
        let candidate = candidate();
        let authentic = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::Authentic,
        );
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-authenticity").unwrap(),
            Instant::from_unix_seconds(2),
            CurationOutcome::NotRequired,
            Some("reviewed".to_owned()),
        )
        .unwrap();
        let (validation, approval) = bind_review(
            &authentic,
            ValidationResult::valid(candidate.key().clone()),
            approval,
        );
        let default_policy = ApprovalPolicy::phase5_default();
        let mismatch = PublicationCoordinator.publish(PublicationRequest {
            candidate: &candidate,
            subgraph: authentic,
            validation: &validation,
            approval_policy: &default_policy,
            approval: &approval,
            authenticity: SourceAuthenticity::NonAuthentic,
            readiness: &readiness(),
            published_at: Instant::from_unix_seconds(3),
        });
        assert!(matches!(
            mismatch,
            Err(PublicationError::SourceAuthenticityMismatch {
                expected: SourceAuthenticity::NonAuthentic,
                recorded: SourceAuthenticity::Authentic,
                ..
            })
        ));

        let unlabelled = CanonicalKnowledgeSubgraph::new()
            .with_entities([Entity::new(
                EntityId::new("entity-unlabelled-authenticity").unwrap(),
                Name::new("Unlabelled source material", "en"),
                Vec::new(),
            )])
            .with_existing_source_ids([candidate.source().source_id().clone()]);
        let revision = SubgraphRevision::capture(&unlabelled);
        let unlabelled_validation = ValidationResult::valid(candidate.key().clone())
            .with_publication_revision(revision.clone());
        let unlabelled_approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-unlabelled-authenticity").unwrap(),
            Instant::from_unix_seconds(3),
            CurationOutcome::NotRequired,
            Some("reviewed without an authenticity label".to_owned()),
        )
        .unwrap()
        .with_publication_revision(revision);
        let mut permissive_policy = ApprovalPolicy::phase5_default();
        permissive_policy.set_rule(
            candidate.source().source_class().clone(),
            SourceApprovalRule::new(CurationRequirement::Optional, false, false, true),
        );
        let missing_status = PublicationCoordinator.publish(PublicationRequest {
            candidate: &candidate,
            subgraph: unlabelled,
            validation: &unlabelled_validation,
            approval_policy: &permissive_policy,
            approval: &unlabelled_approval,
            authenticity: SourceAuthenticity::NonAuthentic,
            readiness: &readiness(),
            published_at: Instant::from_unix_seconds(4),
        });
        assert!(matches!(
            missing_status,
            Err(PublicationError::MissingSourceAuthenticityMetadata { .. })
        ));

        let non_authentic = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::NonAuthentic,
        );
        let revision = SubgraphRevision::capture(&non_authentic);
        let validation = ValidationResult::valid(candidate.key().clone())
            .with_publication_revision(revision.clone());
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-non-authentic").unwrap(),
            Instant::from_unix_seconds(4),
            CurationOutcome::NotRequired,
            Some("non-authentic status explicitly reviewed".to_owned()),
        )
        .unwrap()
        .with_publication_revision(revision);
        let mut policy = ApprovalPolicy::phase5_default();
        policy.set_rule(
            candidate.source().source_class().clone(),
            SourceApprovalRule::new(CurationRequirement::Optional, false, false, true),
        );
        let record = PublicationCoordinator
            .publish(PublicationRequest {
                candidate: &candidate,
                subgraph: non_authentic,
                validation: &validation,
                approval_policy: &policy,
                approval: &approval,
                authenticity: SourceAuthenticity::NonAuthentic,
                readiness: &readiness(),
                published_at: Instant::from_unix_seconds(5),
            })
            .expect("explicitly permitted authenticity metadata is retained");

        assert_eq!(
            record.source_authenticity(),
            SourceAuthenticity::NonAuthentic
        );
        assert_eq!(
            record
                .subgraph()
                .source_authenticity_for(candidate.source().source_id()),
            Some(SourceAuthenticity::NonAuthentic)
        );
    }

    #[test]
    fn publication_requires_validation_approval_and_index_readiness() {
        let candidate = candidate();
        let proposed = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::Authentic,
        );
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-publish").unwrap(),
            Instant::from_unix_seconds(2),
            CurationOutcome::NotRequired,
            Some("reviewed".to_owned()),
        )
        .unwrap();
        let (validation, approval) = bind_review(
            &proposed,
            ValidationResult::valid(candidate.key().clone()),
            approval,
        );
        let ready = readiness();
        let result = publish_candidate(
            &candidate,
            proposed,
            &validation,
            &approval,
            &ready,
            Instant::from_unix_seconds(3),
        )
        .unwrap();

        assert_eq!(result.decision().outcome(), PublicationOutcome::Published);
        assert_eq!(result.source_authenticity(), SourceAuthenticity::Authentic);
        assert_eq!(result.subgraph().object_count(), 3);
        assert_eq!(
            result.synchronization().status(),
            IndexingSynchronizationStatus::Pending
        );
    }

    #[test]
    fn publication_rejects_indexing_receipt_for_another_source_record() {
        let candidate = candidate();
        let proposed = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::Authentic,
        );
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-publish").unwrap(),
            Instant::from_unix_seconds(2),
            CurationOutcome::NotRequired,
            None,
        )
        .unwrap();
        let (validation, approval) = bind_review(
            &proposed,
            ValidationResult::valid(candidate.key().clone()),
            approval,
        );
        let wrong_receipt = readiness_for("other-record");
        let result = publish_candidate(
            &candidate,
            proposed,
            &validation,
            &approval,
            &wrong_receipt,
            Instant::from_unix_seconds(3),
        );
        assert_eq!(
            result,
            Err(PublicationError::IndexingObjectReferenceMismatch)
        );
    }

    #[test]
    fn entity_external_identifiers_and_crosswalks_must_not_map_one_source_id_to_multiple_entities()
    {
        let external =
            ExternalIdentifier::new(SourceId::new("source-external-id").unwrap(), "person-42")
                .unwrap();
        let first = Entity::new(
            EntityId::new("entity-external-1").unwrap(),
            Name::new("First", "en"),
            Vec::new(),
        )
        .with_external_identifier(external.clone());
        let second = Entity::new(
            EntityId::new("entity-external-2").unwrap(),
            Name::new("Second", "en"),
            Vec::new(),
        )
        .with_external_identifier(external);

        let subgraph = CanonicalKnowledgeSubgraph::new().with_entities([first, second]);
        assert_eq!(
            subgraph.validate(),
            Err(super::SubgraphValidationError::ConflictingExternalCrosswalk)
        );
    }

    #[test]
    fn withdrawal_and_correction_preserve_the_original_publication_decision() {
        let candidate = candidate();
        let proposed = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::Authentic,
        );
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-publish").unwrap(),
            Instant::from_unix_seconds(2),
            CurationOutcome::NotRequired,
            None,
        )
        .unwrap();
        let (validation, approval) = bind_review(
            &proposed,
            ValidationResult::valid(candidate.key().clone()),
            approval,
        );
        let ready = readiness();
        let published = publish_candidate(
            &candidate,
            proposed,
            &validation,
            &approval,
            &ready,
            Instant::from_unix_seconds(3),
        )
        .unwrap();

        let actor = crate::identity::AgentId::new("reviewer-withdraw").unwrap();
        let withdrawn = PublicationCoordinator
            .withdraw(
                &published,
                actor.clone(),
                Instant::from_unix_seconds(4),
                "source withdrawn",
            )
            .unwrap();
        assert_eq!(
            withdrawn.decision().outcome(),
            PublicationOutcome::Withdrawn
        );
        assert_eq!(
            withdrawn.prior_publication().outcome(),
            PublicationOutcome::Published
        );
        assert_eq!(withdrawn.withdrawn_by(), &actor);
        assert_eq!(
            published.decision().outcome(),
            PublicationOutcome::Published
        );

        let successor = candidate_for("record-1", "snapshot-2");
        let correction = PublicationCoordinator
            .record_correction(
                &published,
                &successor,
                actor,
                Instant::from_unix_seconds(5),
                "source correction",
            )
            .unwrap();
        assert_eq!(correction.prior_publication().candidate(), candidate.key());
        assert_eq!(correction.successor_candidate(), successor.key());
        assert_ne!(
            correction.prior_publication().candidate(),
            correction.successor_candidate()
        );
    }

    #[test]
    fn post_publication_index_sync_failure_does_not_rewrite_publication_outcome() {
        let candidate = candidate();
        let proposed = subgraph_for(
            candidate.source().source_id(),
            SourceAuthenticity::Authentic,
        );
        let approval = ApprovalDecision::new(
            candidate.key().clone(),
            candidate.source().source_class().clone(),
            ApprovalOutcome::Approved,
            crate::identity::AgentId::new("reviewer-publish").unwrap(),
            Instant::from_unix_seconds(2),
            CurationOutcome::NotRequired,
            None,
        )
        .unwrap();
        let (validation, approval) = bind_review(
            &proposed,
            ValidationResult::valid(candidate.key().clone()),
            approval,
        );
        let ready = readiness();
        let published = publish_candidate(
            &candidate,
            proposed,
            &validation,
            &approval,
            &ready,
            Instant::from_unix_seconds(3),
        )
        .unwrap();
        let sync_failed =
            published.with_synchronization_status(IndexingSynchronizationStatus::RetryableFailure);
        assert_eq!(
            sync_failed.decision().outcome(),
            PublicationOutcome::Published
        );
        assert_eq!(
            sync_failed.synchronization().status(),
            IndexingSynchronizationStatus::RetryableFailure
        );
    }
}
