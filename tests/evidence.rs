//! Level 3 integration tests for Phase 4 evidence and verification.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::evidence::{
    Evidence, EvidenceMechanism, EvidenceRole, EvidenceSourceReference, EvidenceSupport,
    TextOffsetUnit, TextSpan, VerificationOutcome, VerificationPerformer, VerificationRecord,
    VerificationTarget,
};
use nizaam_knowledge_graph::identity::{
    AgentId, ConceptId, EntityId, EvidenceId, KnowledgeAssertionId, ReferenceId, SourceId,
    VerificationId,
};
use nizaam_knowledge_graph::temporal::Instant;

fn assertion(object_label: &str, polarity: AssertionPolarity) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-evidence-subject").unwrap()),
        AssertionPredicate::new("has-description").unwrap(),
        AssertionObject::Concept(ConceptId::new(object_label).unwrap()),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Known,
        polarity,
    )
}

#[test]
fn evidence_supports_structured_locations_without_requiring_text_offsets() {
    let source_id = SourceId::new("source-evidence-test").unwrap();
    let reference_id = ReferenceId::new("reference-evidence-test").unwrap();
    let source_location = EvidenceSourceReference::source(source_id.clone());
    let verse_location =
        EvidenceSourceReference::verse(reference_id.clone(), "2:255").expect("valid verse locator");
    let page_location =
        EvidenceSourceReference::page(reference_id.clone(), "7").expect("valid page locator");
    let span = TextSpan::new(0, 12, TextOffsetUnit::Utf8Byte).expect("valid text span");
    let text_location = EvidenceSourceReference::text_span(reference_id, span);

    let evidence = Evidence::direct_source(
        EvidenceId::new("evidence-structured-locations").unwrap(),
        [
            source_location.clone(),
            verse_location.clone(),
            page_location,
            text_location.clone(),
        ],
    )
    .expect("direct evidence with explicit source locations");

    assert_eq!(evidence.source_references().len(), 4);
    assert!(evidence.source_references().contains(&source_location));
    assert!(evidence.source_references().contains(&verse_location));
    assert!(evidence.source_references().contains(&text_location));
    assert!(matches!(
        evidence.mechanism(),
        EvidenceMechanism::DirectSource
    ));
    assert!(evidence.validate().is_ok());
}

#[test]
fn direct_and_derived_evidence_preserve_their_distinct_mechanisms() {
    let reference = EvidenceSourceReference::source(
        SourceId::new("source-derived-evidence").expect("valid source identity"),
    );
    let upstream = Evidence::direct_source(
        EvidenceId::new("evidence-upstream").unwrap(),
        [reference.clone()],
    )
    .expect("valid upstream evidence");
    let derived = Evidence::derived_support(
        EvidenceId::new("evidence-derived").unwrap(),
        [upstream.id().clone()],
        [reference],
    )
    .expect("derived evidence with explicit upstream dependency");

    assert!(matches!(
        upstream.mechanism(),
        EvidenceMechanism::DirectSource
    ));
    let inputs = derived
        .mechanism()
        .inputs()
        .expect("derived mechanism has inputs");
    assert_eq!(inputs.len(), 1);
    assert!(inputs.contains(upstream.id()));
}

#[test]
fn one_evidence_item_can_support_one_assertion_and_refute_another() {
    let evidence = Evidence::direct_source(
        EvidenceId::new("evidence-multi-role").unwrap(),
        [EvidenceSourceReference::source(
            SourceId::new("source-multi-role").unwrap(),
        )],
    )
    .expect("valid evidence");
    let first = assertion("concept-supported", AssertionPolarity::Positive);
    let second = assertion("concept-refuted", AssertionPolarity::Positive);

    let support = EvidenceSupport::new(&first, &evidence, EvidenceRole::Supports)
        .expect("support relation should be valid");
    let refutation = EvidenceSupport::new(&second, &evidence, EvidenceRole::Refutes)
        .expect("refutation relation should be valid");

    assert_eq!(support.evidence_id(), refutation.evidence_id());
    assert_ne!(support.assertion_id(), refutation.assertion_id());
    assert_eq!(support.role(), &EvidenceRole::Supports);
    assert_eq!(refutation.role(), &EvidenceRole::Refutes);
}

#[test]
fn verification_retains_target_performer_time_evidence_and_source_locations() {
    let evidence_id = EvidenceId::new("evidence-verified").unwrap();
    let reference_id = ReferenceId::new("reference-verification").unwrap();
    let source_reference =
        EvidenceSourceReference::section(reference_id, "Chapter 1").expect("valid section locator");
    let performer = VerificationPerformer::Agent(AgentId::new("agent-reviewer").unwrap());
    let instant = Instant::from_unix_seconds(1_750_000_000);

    let record = VerificationRecord::new(
        VerificationId::new("verification-confirmed").unwrap(),
        VerificationTarget::Evidence(evidence_id.clone()),
        performer.clone(),
        VerificationOutcome::Confirmed,
    )
    .expect("valid verification record")
    .with_performed_at(instant)
    .with_evidence_considered([evidence_id.clone(), evidence_id.clone()])
    .with_source_references([source_reference.clone()])
    .with_notes("The source location was checked directly.");

    assert_eq!(
        record.target(),
        &VerificationTarget::Evidence(evidence_id.clone())
    );
    assert_eq!(record.performer(), &performer);
    assert_eq!(record.outcome(), VerificationOutcome::Confirmed);
    assert_eq!(record.performed_at(), Some(instant));
    assert_eq!(record.evidence_considered().len(), 1);
    assert!(record.evidence_considered().contains(&evidence_id));
    assert!(record.source_references().contains(&source_reference));
    assert_eq!(
        record.notes(),
        Some("The source location was checked directly.")
    );
}

#[test]
fn multiple_verification_records_coexist_for_the_same_target() {
    let target = VerificationTarget::Assertion(
        KnowledgeAssertionId::new("assertion-with-verification-history").unwrap(),
    );
    let performer =
        VerificationPerformer::automated_system("validator-v1").expect("valid automated verifier");
    let inconclusive = VerificationRecord::new(
        VerificationId::new("verification-inconclusive").unwrap(),
        target.clone(),
        performer.clone(),
        VerificationOutcome::Inconclusive,
    )
    .unwrap();
    let confirmed = VerificationRecord::new(
        VerificationId::new("verification-confirmed-later").unwrap(),
        target.clone(),
        performer,
        VerificationOutcome::Confirmed,
    )
    .unwrap();

    assert_eq!(inconclusive.target(), &target);
    assert_eq!(confirmed.target(), &target);
    assert_ne!(inconclusive.id(), confirmed.id());
    assert_eq!(inconclusive.outcome(), VerificationOutcome::Inconclusive);
    assert_eq!(confirmed.outcome(), VerificationOutcome::Confirmed);
}
