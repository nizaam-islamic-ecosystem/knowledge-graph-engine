//! Nizaam Knowledge Graph
//!
//! Top-level public library boundary for the Knowledge Graph engine.
//!
//! The crate root declares the complete current module tree up front. This keeps
//! the top-level module boundary stable as later phases populate the existing
//! scaffolded directories.
//!
//! The module declarations below intentionally include both implemented and
//! future-phase modules. Declaring a module does not mean that all behavior
//! associated with that module is implemented or part of the current contract.
//!
//! The public `use`/re-export surface below exposes the currently established
//! Phase 0 through Phase 4 contracts without recreating Core or Indexing types.
//!
//! Core remains authoritative for universal runtime, lifecycle, request
//! admission, execution context, cancellation, deadlines, capability dispatch,
//! registration, Control Plane coordination, and universal request/response
//! infrastructure. The Knowledge Graph adapts to those mechanisms rather than
//! recreating them.

pub mod assertion;
pub mod authority;
pub mod concept;
pub mod contract;
pub mod engine;
pub mod entity;
pub mod evidence;
pub mod graph;
pub mod identity;
pub mod index;
pub mod ingestion;
pub mod integration;
pub mod lexical;
pub mod ml;
pub mod ontology;
pub mod provenance;
pub mod query;
pub mod reasoning;
pub mod relationship;
pub mod resolution;
pub mod semantics;
pub mod source;
pub mod storage;
pub mod temporal;
pub mod uncertainty;
pub mod versioning;

// -----------------------------------------------------------------------------
// Phase 0 public surface
// -----------------------------------------------------------------------------

// The engine module remains the primary namespace for the Phase 0 engine
// facade. Its existing public items are consumed through `engine::...`.
// No Core runtime/registry/capability implementation is duplicated here.

// -----------------------------------------------------------------------------
// Phase 1 / Phase 4 identity public surface
// -----------------------------------------------------------------------------

pub use identity::{
    ActivityId, AgentId, ConceptId, ContradictionId, EntityId, EvidenceId, KnowledgeAssertionId,
    LexicalFormId, MentionId, ReferenceId, SourceId, VerificationId,
};

// -----------------------------------------------------------------------------
// Phase 1 entity public surface
// -----------------------------------------------------------------------------

pub use entity::{Alias, Entity, Mention, Name};

// -----------------------------------------------------------------------------
// Phase 1 concept public surface
// -----------------------------------------------------------------------------

pub use concept::Concept;

// -----------------------------------------------------------------------------
// Phase 1 source model and Phase 4 authority integration
// -----------------------------------------------------------------------------

pub use source::{Reference, Source, SourceError};

// -----------------------------------------------------------------------------
// Phase 1 lexical public surface
// -----------------------------------------------------------------------------

// `LexicalFormId` is re-exported above from the canonical identity module.
// There is intentionally no concrete `LexicalForm` root export in the current
// implementation boundary.

// -----------------------------------------------------------------------------
// Phase 2 assertion model with Phase 4 metadata
// -----------------------------------------------------------------------------

pub use assertion::{
    AssertionContext, AssertionContextError, AssertionObject, AssertionPolarity,
    AssertionPredicate, AssertionStatus, KnowledgeAssertion, KnowledgeAssertionValidationError,
    Qualifier, QualifierError, Qualifiers,
};

// `KnowledgeAssertionId` remains re-exported above from the canonical identity
// module. The assertion module also exposes it through `assertion::...`.

// -----------------------------------------------------------------------------
// Phase 2 relationship public surface
// -----------------------------------------------------------------------------

pub use relationship::{
    CompositionRule, InverseRelationship, InverseRelationshipError, RELATIONSHIP_NAMESPACE,
    Relationship, RelationshipCharacteristic, RelationshipCharacteristicError,
    RelationshipCharacteristics, RelationshipDirection, RelationshipError, RelationshipFamily,
    RelationshipFamilyError, RelationshipPredicate, RelationshipPredicateValidationError,
    RelationshipVocabulary, RelationshipVocabularyError,
};

// -----------------------------------------------------------------------------
// Phase 2 graph public surface
// -----------------------------------------------------------------------------

pub use graph::{
    Graph, GraphEdge, GraphEdgeError, GraphEdgeId, GraphError, GraphNode, GraphNodeId,
    TraversalDirection, TraversalError, TraversalStep, traverse,
};

// -----------------------------------------------------------------------------
// Phase 3 ontology public surface
// -----------------------------------------------------------------------------

pub use ontology::{
    CardinalityConstraint, Class, ClassId, ClassSet, ClassTaxonomy, ConceptTaxonomy,
    ConstraintExtension, ConstraintSet, IslamicSeed, IslamicSeedError, Ontology,
    OntologyConstraint, OntologyConstraintError, OntologyError, OntologyProperty,
    OntologyPropertyValidationError, OntologyRegistry, OntologyRegistryError, OntologySnapshot,
    OntologySnapshotId, Taxonomy, TaxonomyError, ValidationIssue, ValidationReport,
    load_islamic_seed, load_islamic_seed_from_str,
};

// -----------------------------------------------------------------------------
// Phase 3 semantics public surface
// -----------------------------------------------------------------------------

pub use semantics::{
    Context, ContextDimension, ContextDimensionKey, ContextError, ContextId, ContextKind,
    Interpretation, InterpretationSource, LexicalConceptMapping, LexicalConceptMappingError,
    LexicalConceptMappingKind, Meaning, SemanticRelation, SemanticType, SemanticTypeError,
    SemanticTypeMembership, SemanticTypeTarget, SemanticTypes, SenseReference, SenseReferenceError,
};

// -----------------------------------------------------------------------------
// Phase 3 entity-resolution public surface
// -----------------------------------------------------------------------------

pub use resolution::{
    Candidate, CandidateSignal, CircularEvidenceGuard, EntityCandidateProfile, ExternalIdentifier,
    ExternalIdentifierCrosswalk, ExternalIdentifierError, GraphEvidenceDependency,
    NormalizationError, RankingPolicy, ResolutionDecision, ResolutionDependencyToken,
    ResolutionError, ResolutionInput, ResolutionPolicy, ResolutionReference, ResolutionResult,
    ResolutionState, Resolver, decide, exact_match, generate_candidates, normalize,
    normalized_match, provisional, rank_candidates, rejected, transliteration_match,
};

// -----------------------------------------------------------------------------
// Phase 4 temporal public surface
// -----------------------------------------------------------------------------

pub use temporal::{
    Approximate, Instant, Interval, IntervalBoundary, OpenEnded, OpenEndedDirection, TemporalError,
    TemporalRelation, TemporalRelationship, TemporalValidity, TemporalValue,
};

// -----------------------------------------------------------------------------
// Phase 4 evidence and verification public surface
// -----------------------------------------------------------------------------

pub use evidence::{
    Evidence, EvidenceMechanism, EvidenceRole, EvidenceRoleError, EvidenceSourceKind,
    EvidenceSourceReference, EvidenceSourceReferenceError, EvidenceSupport, EvidenceSupportError,
    EvidenceValidationError, TextOffsetUnit, TextSpan, TextSpanError, VerificationError,
    VerificationOutcome, VerificationPerformer, VerificationRecord, VerificationTarget,
};

// -----------------------------------------------------------------------------
// Phase 4 knowledge provenance and audit public surface
// -----------------------------------------------------------------------------

pub use provenance::{
    Activity, ActivityKind, Agent, AgentType, AuditAction, AuditError, AuditRecord, AuditTrail,
    KnowledgeOrigin, KnowledgeOriginError, Lineage, LineageError, LineageKind, LineageLink,
    ProvenanceError, ProvenanceHistory, ProvenanceRecord, ProvenanceTarget,
};

// -----------------------------------------------------------------------------
// Phase 4 authority public surface
// -----------------------------------------------------------------------------

pub use authority::{
    Authority, AuthorityDimension, AuthorityError, AuthorityEvaluationProfile, AuthorityTarget,
    AuthorityValue, ProcessReliability, ReliabilityAssessment, ReliabilityError, ScholarlyStatus,
    ScholarlyStatusError, SourceReliability,
};

// -----------------------------------------------------------------------------
// Phase 4 uncertainty, confidence, and contradiction public surface
// -----------------------------------------------------------------------------

pub use uncertainty::{
    ConfidenceAssessment, ConfidenceBasis, ConfidenceContext, ConfidenceError, ConfidenceLabel,
    ConfidenceScore, ConfidenceTarget, ConfidenceValue, Contradiction, ContradictionError,
    ContradictionFinding, ContradictionKind, ContradictionSet, ContradictionStatus,
    CurrentViewPolicy, EPISTEMIC_STATES, EpistemicStatus, detect_contradiction,
    is_phase4_epistemic_status,
};
