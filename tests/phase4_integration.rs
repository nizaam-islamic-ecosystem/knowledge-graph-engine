//! Level 3 cross-module integration test for the Phase 4 semantic foundations.
//!
//! Exercises one canonical assertion through temporal validity, source/evidence
//! attachment, authority, verification, provenance, audit, confidence, and a
//! conservative contradiction finding. No storage or reasoning engine is used.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::authority::{
    Authority, AuthorityDimension, AuthorityTarget, AuthorityValue,
};
use nizaam_knowledge_graph::evidence::{
    Evidence, EvidenceRole, EvidenceSourceReference, EvidenceSupport, VerificationOutcome,
    VerificationPerformer, VerificationRecord, VerificationTarget,
};
use nizaam_knowledge_graph::identity::{
    ActivityId, AgentId, ConceptId, ContradictionId, EntityId, EvidenceId, ReferenceId, SourceId,
    VerificationId,
};
use nizaam_knowledge_graph::provenance::{
    Activity, ActivityKind, Agent, AgentType, AuditAction, AuditRecord, AuditTrail,
    KnowledgeOrigin, Lineage, LineageKind, LineageLink, OperationId, ProvenanceHistory,
    ProvenanceRecord, ProvenanceTarget,
};
use nizaam_knowledge_graph::source::Source;
use nizaam_knowledge_graph::temporal::{Instant, Interval, IntervalBoundary, TemporalValidity};
use nizaam_knowledge_graph::uncertainty::{
    ConfidenceAssessment, ConfidenceBasis, ConfidenceContext, ConfidenceTarget, ConfidenceValue,
    ContradictionSet, ContradictionStatus, CurrentViewPolicy, detect_contradiction,
};
use std::collections::BTreeSet;

fn assertion(polarity: AssertionPolarity) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-phase4-subject").unwrap()),
        AssertionPredicate::new("has-description").unwrap(),
        AssertionObject::Concept(ConceptId::new("concept-phase4-object").unwrap()),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Known,
        polarity,
    )
}

#[test]
fn phase4_semantic_object_round_trips_through_public_models() {
    let source_id = SourceId::new("source-phase4-integration").unwrap();
    let reference_id = ReferenceId::new("reference-phase4-integration").unwrap();
    let source_authority = Authority::new(AuthorityTarget::Source(source_id.clone()))
        .with_dimension(AuthorityDimension::SourceAuthority(
            AuthorityValue::new("curated").unwrap(),
        ))
        .expect("valid source authority");
    let source = Source::new(source_id.clone(), "Integration test source")
        .with_authority(source_authority)
        .expect("authority belongs to source");

    let source_location = EvidenceSourceReference::verse(reference_id.clone(), "2:255")
        .expect("valid verse location");
    let evidence = Evidence::direct_source(
        EvidenceId::new("evidence-phase4-integration").unwrap(),
        [source_location.clone()],
    )
    .expect("valid source-backed evidence");

    let original = assertion(AssertionPolarity::Positive);
    let assertion_id = original.id().clone();
    let valid_start = Instant::from_unix_seconds(1_000);
    let valid_end = Instant::from_unix_seconds(2_000);
    let validity_interval = Interval::new(
        IntervalBoundary::Inclusive(valid_start),
        IntervalBoundary::Exclusive(valid_end),
    )
    .expect("valid semantic interval");
    let assertion_authority = Authority::new(AuthorityTarget::Assertion(assertion_id.clone()))
        .with_dimension(AuthorityDimension::HumanReview(
            AuthorityValue::new("reviewed").unwrap(),
        ))
        .expect("valid assertion authority");
    let assertion = original
        .with_validity(TemporalValidity::during(validity_interval))
        .with_authority(assertion_authority)
        .expect("authority target matches canonical assertion");

    assert_eq!(assertion.id(), &assertion_id);
    assert_eq!(
        assertion.validity().unwrap(),
        TemporalValidity::during(validity_interval)
    );
    assert!(assertion.authority().is_some());
    assert!(assertion.validate().is_ok());

    let support = EvidenceSupport::new(&assertion, &evidence, EvidenceRole::Supports)
        .expect("valid evidence/assertion association");
    assert_eq!(support.assertion_id(), &assertion_id);
    assert_eq!(support.evidence_id(), evidence.id());

    let agent = Agent::new(
        AgentId::new("agent-phase4-reviewer").unwrap(),
        AgentType::Human,
        "reviewer",
    )
    .expect("valid provenance agent");
    let activity_id = ActivityId::new("activity-phase4-review").unwrap();
    let operation_id = OperationId::new("nizaam.kg.phase4.integration-operation")
        .expect("Core operation identity remains authoritative");
    let activity = Activity::new(activity_id.clone(), ActivityKind::Review)
        .expect("valid activity")
        .with_agent(agent.id().clone())
        .with_core_operation_id(operation_id.clone())
        .with_description("Reviewed the assertion against its source")
        .expect("valid activity description");
    let origin = KnowledgeOrigin::source_reference(Some(source_id.clone()), reference_id.clone());
    let recorded_at = Instant::from_unix_seconds(3_000);
    let target = ProvenanceTarget::Assertion(assertion_id.clone());
    let provenance_record =
        ProvenanceRecord::new(target.clone(), activity, recorded_at, Some(origin.clone()))
            .expect("valid provenance record");
    let mut history = ProvenanceHistory::new(target.clone()).expect("valid target history");
    history
        .append(provenance_record)
        .expect("history append succeeds");

    assert_eq!(history.len(), 1);
    assert_eq!(
        history.records()[0].activity().core_operation_id(),
        Some(&operation_id)
    );
    assert_eq!(history.records()[0].origin(), Some(&origin));
    assert_eq!(validity_interval.contains(recorded_at), Some(false));

    let verification_id = VerificationId::new("verification-phase4-integration").unwrap();
    let verification = VerificationRecord::new(
        verification_id.clone(),
        VerificationTarget::Assertion(assertion_id.clone()),
        VerificationPerformer::Agent(agent.id().clone()),
        VerificationOutcome::Confirmed,
    )
    .expect("valid verification")
    .with_performed_at(recorded_at)
    .with_evidence_considered([evidence.id().clone()])
    .with_source_references([source_location.clone()]);
    assert_eq!(
        verification.target(),
        &VerificationTarget::Assertion(assertion_id.clone())
    );

    let confidence_target = ConfidenceTarget::new(
        assertion_id.clone(),
        evidence.id().clone(),
        EvidenceRole::Supports,
    )
    .expect("valid confidence target");
    let confidence_basis = ConfidenceBasis::new(
        [evidence.id().clone()],
        [verification_id],
        Some("Source location reviewed and confirmed.".to_owned()),
    )
    .expect("valid confidence basis");
    let confidence = ConfidenceAssessment::new(
        confidence_target,
        ConfidenceValue::numeric(0.9).expect("valid numeric confidence"),
        confidence_basis,
        ConfidenceContext::new()
            .with_domain("phase4-integration")
            .expect("valid confidence domain")
            .with_evaluated_at(recorded_at),
    );
    assert_eq!(confidence.target().assertion_id(), &assertion_id);
    assert_eq!(confidence.value().as_numeric().unwrap().value(), 0.9);

    let lineage_link = LineageLink::new(
        ProvenanceTarget::Source(source_id.clone()),
        target.clone(),
        LineageKind::DerivedFrom,
    )
    .expect("valid lineage link")
    .with_activity(activity_id)
    .with_recorded_at(recorded_at);
    let mut lineage = Lineage::new();
    assert!(lineage.append(lineage_link).expect("append lineage"));

    let audit_record = AuditRecord::new(
        operation_id,
        target,
        AuditAction::Attached,
        recorded_at,
        "Attached evidence and review metadata",
    )
    .expect("valid audit record")
    .with_actor(agent.id().clone());
    let mut audit = AuditTrail::new();
    assert!(audit.append(audit_record));

    assert_eq!(source.id(), &source_id);
    assert!(source.validate().is_ok());
    assert_eq!(lineage.len(), 1);
    assert_eq!(audit.len(), 1);
}

#[test]
fn phase4_conflict_detection_preserves_both_assertion_objects() {
    let positive = assertion(AssertionPolarity::Positive);
    let negative = assertion(AssertionPolarity::Negative);
    let functional_predicates = BTreeSet::new();
    let finding = detect_contradiction(&positive, &negative, &functional_predicates)
        .expect("opposed polarity should be detected");
    let contradiction = finding
        .into_contradiction(ContradictionId::new("contradiction-phase4-integration").unwrap())
        .expect("valid contradiction record")
        .with_supporting_evidence([EvidenceId::new("evidence-phase4-conflict").unwrap()])
        .with_status(ContradictionStatus::UnderReview);
    let mut contradictions = ContradictionSet::new();
    contradictions
        .insert(contradiction)
        .expect("record contradiction");

    let current = [positive.clone(), negative.clone()];
    assert_eq!(
        contradictions
            .current_view(&current, CurrentViewPolicy::IncludeAll)
            .len(),
        2
    );
    assert!(
        contradictions
            .current_view(&current, CurrentViewPolicy::ExcludeActiveConflicts)
            .is_empty()
    );
    assert!(contradictions.has_active_conflict(positive.id()));
    assert!(contradictions.has_active_conflict(negative.id()));
}
