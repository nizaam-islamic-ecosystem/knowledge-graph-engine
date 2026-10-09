//! Level 3 Phase 3 negative-boundary tests.

use std::collections::BTreeSet;

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionStatus, KnowledgeAssertion,
    Qualifiers,
};
use nizaam_knowledge_graph::identity::{ConceptId, EntityId};
use nizaam_knowledge_graph::ontology::{
    Class, ClassId, Ontology, OntologyConstraint, OntologyProperty, OntologyRegistry,
};
use nizaam_knowledge_graph::relationship::RelationshipPredicate;
use nizaam_knowledge_graph::resolution::{
    Candidate, CandidateSignal, ResolutionPolicy, ResolutionState, Resolver, decide, provisional,
};
use nizaam_knowledge_graph::semantics::{
    LexicalConceptMapping, SemanticType, SemanticTypeTarget, SenseReference,
};

fn class(id: &str) -> Class {
    Class::new(ClassId::new(id).unwrap(), id)
}
fn cid(id: &str) -> ClassId {
    ClassId::new(id).unwrap()
}
fn candidate(id: &str, signal: CandidateSignal) -> Candidate {
    Candidate::new(EntityId::new(id).unwrap(), BTreeSet::from([signal]))
}

#[test]
fn invalid_ontology_domain_or_range_is_rejected_by_validation() {
    let mut ontology = Ontology::new();
    ontology.add_class(class("person")).unwrap();
    ontology
        .add_property(
            OntologyProperty::new(
                RelationshipPredicate::new("has-name").unwrap(),
                BTreeSet::from([cid("person")]),
                BTreeSet::from([cid("name")]),
            )
            .unwrap(),
        )
        .unwrap();
    let report = ontology.validate();
    assert!(!report.is_valid());
    assert!(
        report
            .errors()
            .iter()
            .any(|e| e.code() == "ontology.property.range.unknown_class")
    );
}

#[test]
fn illegal_disjointness_reference_is_rejected_by_validation() {
    let mut ontology = Ontology::new();
    ontology.add_class(class("person")).unwrap();
    ontology
        .add_class_constraint(
            cid("person"),
            OntologyConstraint::disjoint_with([cid("unknown")]).unwrap(),
        )
        .unwrap();
    let report = ontology.validate();
    assert!(
        report
            .errors()
            .iter()
            .any(|e| e.code() == "ontology.constraint.disjoint.unknown_class")
    );
}

#[test]
fn invalid_ontology_cannot_be_registered() {
    let mut ontology = Ontology::new();
    ontology.add_class(class("person")).unwrap();
    ontology
        .add_property(
            OntologyProperty::new(
                RelationshipPredicate::new("has-name").unwrap(),
                BTreeSet::from([cid("person")]),
                BTreeSet::from([cid("unknown")]),
            )
            .unwrap(),
        )
        .unwrap();
    let mut registry = OntologyRegistry::new();
    assert!(registry.register(ontology).is_err());
    assert!(registry.is_empty());
}

#[test]
fn below_threshold_candidate_cannot_become_resolved() {
    let decision = decide(
        &[candidate("e1", CandidateSignal::Context)],
        &ResolutionPolicy::default(),
    );
    assert_eq!(decision.state(), ResolutionState::Unresolved);
    assert!(decision.entity_id().is_none());
}

#[test]
fn insufficient_margin_candidate_cannot_become_resolved() {
    let decision = decide(
        &[
            candidate("e1", CandidateSignal::ExactAlias),
            candidate("e2", CandidateSignal::ExactAlias),
        ],
        &ResolutionPolicy::default(),
    );
    assert_eq!(decision.state(), ResolutionState::Ambiguous);
    assert!(decision.entity_id().is_none());
}

#[test]
fn provisional_candidate_is_not_canonical_resolution() {
    let candidate = candidate("e1", CandidateSignal::Normalized);
    let decision = provisional(&candidate);
    assert_eq!(decision.state(), ResolutionState::Provisional);
    assert!(!decision.is_resolved());
}

#[test]
fn candidate_has_no_independent_candidate_identity() {
    let candidate = candidate("e1", CandidateSignal::ExactAlias);
    assert_eq!(candidate.entity_id().as_str(), "e1");
}

#[test]
fn resolver_does_not_accept_circular_graph_evidence() {
    use nizaam_knowledge_graph::identity::KnowledgeAssertionId;
    use nizaam_knowledge_graph::resolution::{
        CircularEvidenceGuard, GraphEvidenceDependency, ResolutionDependencyToken, ResolutionError,
    };
    let resolver = Resolver::phase3_default();
    let token = ResolutionDependencyToken::new(99);
    let dependency =
        GraphEvidenceDependency::new(KnowledgeAssertionId::new("a1").unwrap(), [token]);
    let mut guard = CircularEvidenceGuard::new();
    guard.activate(token);
    assert!(matches!(
        resolver.validate_graph_evidence(&guard, &dependency),
        Err(ResolutionError::CircularEvidence { .. })
    ));
}

#[test]
fn lexical_mapping_rejects_non_lexical_subject() {
    let assertion = KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("e1").unwrap()),
        RelationshipPredicate::new("expresses").unwrap(),
        AssertionObject::Concept(ConceptId::new("c1").unwrap()),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );
    assert!(LexicalConceptMapping::direct(assertion).is_err());
}

#[test]
fn sense_reference_rejects_empty_values() {
    assert!(SenseReference::new("   ").is_err());
}

#[test]
fn class_and_concept_semantic_targets_cannot_be_collapsed() {
    let class_target = SemanticTypeTarget::Class(cid("person"));
    let concept_target = SemanticTypeTarget::Concept(ConceptId::new("person").unwrap());

    let class_type = SemanticType::class(cid("person"));
    let concept_type = SemanticType::concept(ConceptId::new("person").unwrap());

    assert_eq!(class_type.target().clone(), class_target);
    assert_eq!(concept_type.target().clone(), concept_target);
    assert_ne!(class_target, concept_target);
}
