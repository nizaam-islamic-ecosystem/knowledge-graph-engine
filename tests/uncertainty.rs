//! Level 3 integration tests for epistemic status, confidence, and contradiction.

use std::any::TypeId;
use std::collections::BTreeSet;

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::evidence::{
    Evidence, EvidenceRole, EvidenceSourceReference, VerificationOutcome, VerificationPerformer,
    VerificationRecord, VerificationTarget,
};
use nizaam_knowledge_graph::identity::{
    AgentId, ConceptId, ContradictionId, EntityId, EvidenceId, VerificationId,
};
use nizaam_knowledge_graph::resolution::ResolutionState;
use nizaam_knowledge_graph::temporal::Instant;
use nizaam_knowledge_graph::uncertainty::{
    ConfidenceAssessment, ConfidenceBasis, ConfidenceContext, ConfidenceTarget, ConfidenceValue,
    ContradictionSet, ContradictionStatus, CurrentViewPolicy, EPISTEMIC_STATES, EpistemicStatus,
    detect_contradiction, is_phase4_epistemic_status,
};

fn assertion(
    object_label: &str,
    predicate: &str,
    polarity: AssertionPolarity,
    context: AssertionContext,
) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-uncertainty-subject").unwrap()),
        AssertionPredicate::new(predicate).unwrap(),
        AssertionObject::Concept(ConceptId::new(object_label).unwrap()),
        context,
        Qualifiers::new(),
        AssertionStatus::Known,
        polarity,
    )
}

#[test]
fn epistemic_status_reuses_assertion_status_and_remains_distinct_from_resolution_state() {
    assert_eq!(EPISTEMIC_STATES.len(), 6);
    for status in EPISTEMIC_STATES {
        assert!(is_phase4_epistemic_status(status));
    }
    assert!(!is_phase4_epistemic_status(AssertionStatus::Provisional));
    assert!(!is_phase4_epistemic_status(AssertionStatus::Accepted));
    assert!(!is_phase4_epistemic_status(AssertionStatus::Rejected));

    assert_eq!(
        TypeId::of::<EpistemicStatus>(),
        TypeId::of::<AssertionStatus>()
    );
    assert_ne!(
        TypeId::of::<EpistemicStatus>(),
        TypeId::of::<ResolutionState>()
    );
    assert_eq!(AssertionStatus::Conflicting.as_str(), "conflicting");
    assert_eq!(ResolutionState::Ambiguous.as_str(), "ambiguous");
}

#[test]
fn confidence_preserves_value_basis_and_evaluation_context() {
    let assertion = assertion(
        "concept-confidence",
        "has-description",
        AssertionPolarity::Positive,
        AssertionContext::new(),
    );
    let evidence_id = EvidenceId::new("evidence-confidence").unwrap();
    let verification_id = VerificationId::new("verification-confidence").unwrap();
    let target = ConfidenceTarget::new(
        assertion.id().clone(),
        evidence_id.clone(),
        EvidenceRole::Supports,
    )
    .expect("valid confidence target");
    let basis = ConfidenceBasis::new(
        [evidence_id.clone()],
        [verification_id.clone()],
        Some("Evidence was reviewed against the cited source.".to_owned()),
    )
    .expect("nonempty confidence basis");
    let evaluated_at = Instant::from_unix_seconds(1_750_000_000);
    let context = ConfidenceContext::new()
        .with_domain("knowledge-assertion-review")
        .expect("valid domain")
        .with_profile("conservative-review-v1")
        .expect("valid profile")
        .with_evaluator("phase4-test")
        .expect("valid evaluator")
        .with_method("manual-review")
        .expect("valid method")
        .with_evaluated_at(evaluated_at);
    let assessment = ConfidenceAssessment::new(
        target,
        ConfidenceValue::numeric(0.85).expect("confidence is within [0, 1]"),
        basis,
        context,
    );

    assert_eq!(assessment.target().assertion_id(), assertion.id());
    assert_eq!(assessment.target().evidence_id(), &evidence_id);
    assert_eq!(assessment.value().as_numeric().unwrap().value(), 0.85);
    assert!(assessment.basis().evidence_ids().contains(&evidence_id));
    assert!(
        assessment
            .basis()
            .verification_ids()
            .contains(&verification_id)
    );
    assert_eq!(
        assessment.basis().explanation(),
        Some("Evidence was reviewed against the cited source.")
    );
    assert_eq!(
        assessment.context().domain(),
        Some("knowledge-assertion-review")
    );
    assert_eq!(assessment.context().evaluated_at(), Some(evaluated_at));

    let unknown = ConfidenceValue::unknown();
    let qualitative = ConfidenceValue::qualitative("moderate").expect("valid qualitative value");
    assert!(unknown.is_unknown());
    assert_eq!(qualitative.as_qualitative(), Some("moderate"));
}

#[test]
fn contradiction_detection_is_conservative_and_keeps_conflicting_assertions_inspectable() {
    let context = AssertionContext::new();
    let positive = assertion(
        "concept-same-proposition",
        "has-description",
        AssertionPolarity::Positive,
        context.clone(),
    );
    let negative = assertion(
        "concept-same-proposition",
        "has-description",
        AssertionPolarity::Negative,
        context.clone(),
    );
    let no_functional_predicates = BTreeSet::new();
    let finding = detect_contradiction(&positive, &negative, &no_functional_predicates)
        .expect("opposite polarity on the same proposition is explicit conflict");

    assert_eq!(finding.assertion_ids().len(), 2);
    assert_eq!(finding.involved_objects().len(), 1);

    let contradiction = finding
        .into_contradiction(ContradictionId::new("contradiction-explicit").unwrap())
        .expect("finding converts to a contradiction record")
        .with_status(ContradictionStatus::UnderReview);
    let mut conflicts = ContradictionSet::new();
    conflicts
        .insert(contradiction.clone())
        .expect("first contradiction insertion should succeed");
    assert!(
        conflicts.insert(contradiction).is_err(),
        "duplicate contradiction identity must be rejected"
    );
    assert!(conflicts.has_active_conflict(positive.id()));
    assert!(conflicts.has_active_conflict(negative.id()));

    let all = [positive.clone(), negative.clone()];
    assert_eq!(
        conflicts
            .current_view(&all, CurrentViewPolicy::IncludeAll)
            .len(),
        2
    );
    assert!(
        conflicts
            .current_view(&all, CurrentViewPolicy::ExcludeActiveConflicts)
            .is_empty()
    );

    let unrelated = assertion(
        "concept-unrelated",
        "has-description",
        AssertionPolarity::Positive,
        context,
    );
    assert!(detect_contradiction(&positive, &unrelated, &no_functional_predicates).is_none());
}

#[test]
fn functional_value_conflicts_require_explicit_functional_predicate_and_matching_context() {
    let first = assertion(
        "concept-value-a",
        "has-single-value",
        AssertionPolarity::Positive,
        AssertionContext::new(),
    );
    let second = assertion(
        "concept-value-b",
        "has-single-value",
        AssertionPolarity::Positive,
        AssertionContext::new(),
    );
    assert!(detect_contradiction(&first, &second, &BTreeSet::new()).is_none());

    let functional = BTreeSet::from([AssertionPredicate::new("has-single-value").unwrap()]);
    let finding = detect_contradiction(&first, &second, &functional)
        .expect("different positive values conflict for a declared functional predicate");
    assert_eq!(finding.involved_objects().len(), 2);

    let other_context = AssertionContext::from_entries([("jurisdiction", "other")]).unwrap();
    let context_mismatch = assertion(
        "concept-value-b",
        "has-single-value",
        AssertionPolarity::Positive,
        other_context,
    );
    assert!(detect_contradiction(&first, &context_mismatch, &functional).is_none());
}

#[test]
fn verification_record_retains_its_explicit_target_and_outcome() {
    let evidence = Evidence::direct_source(
        EvidenceId::new("evidence-uncertainty-verification").unwrap(),
        [EvidenceSourceReference::source(
            nizaam_knowledge_graph::identity::SourceId::new("source-uncertainty-verification")
                .unwrap(),
        )],
    )
    .unwrap();
    let verification = VerificationRecord::new(
        VerificationId::new("verification-uncertainty").unwrap(),
        VerificationTarget::Evidence(evidence.id().clone()),
        VerificationPerformer::Agent(AgentId::new("agent-uncertainty").unwrap()),
        VerificationOutcome::Inconclusive,
    )
    .unwrap();
    assert_eq!(verification.outcome(), VerificationOutcome::Inconclusive);
    assert_eq!(
        verification.target(),
        &VerificationTarget::Evidence(evidence.id().clone())
    );
}
