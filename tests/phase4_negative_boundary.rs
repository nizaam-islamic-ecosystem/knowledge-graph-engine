//! Level 3 negative and boundary tests for invalid Phase 4 values and attachments.

use std::collections::BTreeSet;

use nizaam_knowledge_graph::assertion::KnowledgeAssertionValidationError;
use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::authority::{Authority, AuthorityTarget};
use nizaam_knowledge_graph::evidence::{
    Evidence, EvidenceSourceReference, TextOffsetUnit, TextSpan, VerificationOutcome,
    VerificationPerformer, VerificationRecord, VerificationTarget,
};
use nizaam_knowledge_graph::identity::{
    ActivityId, AgentId, ConceptId, ContradictionId, EntityId, EvidenceId, KnowledgeAssertionId,
    ReferenceId, SourceId, VerificationId,
};
use nizaam_knowledge_graph::provenance::{
    Activity, ActivityKind, AuditAction, AuditRecord, KnowledgeOrigin, LineageKind, LineageLink,
    OperationId, ProvenanceHistory, ProvenanceRecord, ProvenanceTarget,
};
use nizaam_knowledge_graph::source::{Source, SourceError};
use nizaam_knowledge_graph::temporal::{Instant, Interval, IntervalBoundary, TemporalError};
use nizaam_knowledge_graph::uncertainty::{
    ConfidenceBasis, ConfidenceError, ConfidenceValue, ContradictionError, detect_contradiction,
};

fn assertion(object_label: &str, polarity: AssertionPolarity) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-negative-boundary").unwrap()),
        AssertionPredicate::new("has-description").unwrap(),
        AssertionObject::Concept(ConceptId::new(object_label).unwrap()),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Known,
        polarity,
    )
}

#[test]
fn invalid_temporal_instants_and_intervals_are_rejected() {
    assert!(Instant::new(1, 1_000_000_000).is_err());
    let later = Instant::from_unix_seconds(20);
    let earlier = Instant::from_unix_seconds(10);
    assert!(matches!(
        Interval::new(
            IntervalBoundary::Inclusive(later),
            IntervalBoundary::Inclusive(earlier)
        ),
        Err(TemporalError::ReversedInterval { .. })
    ));
    assert!(matches!(
        Interval::new(
            IntervalBoundary::Exclusive(earlier),
            IntervalBoundary::Exclusive(earlier)
        ),
        Err(TemporalError::EmptyInterval { .. })
    ));
}

#[test]
fn empty_spans_and_malformed_source_locations_are_rejected() {
    assert!(matches!(
        TextSpan::new(4, 4, TextOffsetUnit::Utf8Byte),
        Err(nizaam_knowledge_graph::evidence::TextSpanError::InvalidBounds { .. })
    ));
    assert!(matches!(
        EvidenceSourceReference::section(
            ReferenceId::new("reference-invalid-section").unwrap(),
            "   "
        ),
        Err(nizaam_knowledge_graph::evidence::EvidenceSourceReferenceError::EmptyLocator { .. })
    ));
    assert!(matches!(
        EvidenceSourceReference::verse(
            ReferenceId::new("reference-invalid-verse").unwrap(),
            "2:255\n"
        ),
        Err(
            nizaam_knowledge_graph::evidence::EvidenceSourceReferenceError::ControlCharacter { .. }
        )
    ));
    assert!(
        EvidenceSourceReference::source_version(
            SourceId::new("source-invalid-version").unwrap(),
            " ",
        )
        .is_err()
    );
}

#[test]
fn evidence_requires_a_source_location_or_a_non_self_derived_input() {
    assert!(
        Evidence::direct_source(
            EvidenceId::new("evidence-no-source").unwrap(),
            Vec::<EvidenceSourceReference>::new(),
        )
        .is_err()
    );

    assert!(
        Evidence::derived_support(
            EvidenceId::new("evidence-no-input").unwrap(),
            Vec::<EvidenceId>::new(),
            Vec::<EvidenceSourceReference>::new(),
        )
        .is_err()
    );

    let evidence_id = EvidenceId::new("evidence-self-dependency").unwrap();
    assert!(
        Evidence::derived_support(
            evidence_id.clone(),
            [evidence_id],
            Vec::<EvidenceSourceReference>::new(),
        )
        .is_err()
    );
}

#[test]
fn invalid_verification_performers_and_source_reference_batches_are_rejected() {
    assert!(matches!(
        VerificationPerformer::automated_system("  "),
        Err(nizaam_knowledge_graph::evidence::VerificationError::EmptyPerformerName)
    ));

    let record = VerificationRecord::new(
        VerificationId::new("verification-invalid-source-batch").unwrap(),
        VerificationTarget::Evidence(EvidenceId::new("evidence-verification-target").unwrap()),
        VerificationPerformer::Agent(AgentId::new("agent-verifier").unwrap()),
        VerificationOutcome::Confirmed,
    )
    .unwrap();
    let valid = EvidenceSourceReference::section(
        ReferenceId::new("reference-valid-batch").unwrap(),
        "Chapter 1",
    );
    let invalid =
        EvidenceSourceReference::section(ReferenceId::new("reference-invalid-batch").unwrap(), " ");
    let result = record.try_with_source_references([valid, invalid]);
    assert!(
        result.is_err(),
        "a failed batch must not silently discard an invalid location"
    );
}

#[test]
fn authority_and_provenance_attachments_reject_wrong_targets_or_time_ranges() {
    let source = Source::new(SourceId::new("source-owner-negative").unwrap(), "Source");
    let authority_for_other_source = Authority::new(AuthorityTarget::Source(
        SourceId::new("source-not-owner-negative").unwrap(),
    ));
    assert!(matches!(
        source.with_authority(authority_for_other_source),
        Err(SourceError::AuthorityTargetMismatch { .. })
    ));

    let knowledge_assertion = assertion("concept-assertion-owner", AssertionPolarity::Positive);
    let wrong_authority = Authority::new(AuthorityTarget::Source(
        SourceId::new("source-not-assertion-negative").unwrap(),
    ));
    assert!(matches!(
        knowledge_assertion.with_authority(wrong_authority),
        Err(KnowledgeAssertionValidationError::AuthorityTargetMismatch { .. })
    ));

    let invalid_activity = Activity::new(
        ActivityId::new("activity-reversed-time").unwrap(),
        ActivityKind::Extraction,
    )
    .unwrap()
    .with_ended_at(Instant::from_unix_seconds(10))
    .unwrap()
    .with_started_at(Instant::from_unix_seconds(20));
    assert!(invalid_activity.is_err());

    assert!(KnowledgeOrigin::new(None, None, None, None).is_err());
    assert!(
        KnowledgeOrigin::source_version(SourceId::new("source-origin-version").unwrap(), "\n",)
            .is_err()
    );
}

#[test]
fn provenance_history_rejects_target_mismatch_and_lineage_rejects_self_links() {
    let target = ProvenanceTarget::Source(SourceId::new("source-history-target").unwrap());
    let other = ProvenanceTarget::Assertion(
        KnowledgeAssertionId::new("assertion-wrong-history-target").unwrap(),
    );
    let mut history = ProvenanceHistory::new(target.clone()).unwrap();
    let mismatched_record = ProvenanceRecord::new(
        other,
        Activity::new(
            ActivityId::new("activity-wrong-history-target").unwrap(),
            ActivityKind::Import,
        )
        .unwrap(),
        Instant::from_unix_seconds(10),
        Some(KnowledgeOrigin::from_source(
            SourceId::new("source-origin-target").unwrap(),
        )),
    )
    .unwrap();
    assert!(history.append(mismatched_record).is_err());

    assert!(matches!(
        LineageLink::new(target.clone(), target, LineageKind::DerivedFrom),
        Err(nizaam_knowledge_graph::provenance::LineageError::SelfLink)
    ));
}

#[test]
fn invalid_audit_values_and_empty_confidence_basis_are_rejected() {
    let target = ProvenanceTarget::Source(SourceId::new("source-audit-negative").unwrap());
    assert!(
        AuditRecord::new(
            OperationId::new("nizaam.kg.phase4.invalid-audit").unwrap(),
            target.clone(),
            AuditAction::Updated,
            Instant::from_unix_seconds(100),
            "   ",
        )
        .is_err()
    );

    let record = AuditRecord::new(
        OperationId::new("nizaam.kg.phase4.invalid-fields").unwrap(),
        target,
        AuditAction::Updated,
        Instant::from_unix_seconds(100),
        "Updated source metadata",
    )
    .unwrap();
    assert!(record.with_changed_fields([String::from(" ")]).is_err());

    assert!(matches!(
        ConfidenceBasis::new(Vec::<EvidenceId>::new(), Vec::<VerificationId>::new(), None,),
        Err(ConfidenceError::EmptyBasis)
    ));
    assert!(ConfidenceValue::numeric(f64::NAN).is_err());
    assert!(ConfidenceValue::numeric(1.01).is_err());
}

#[test]
fn contradiction_detection_respects_context_and_rejects_invalid_records() {
    let positive = assertion("concept-negative-conflict", AssertionPolarity::Positive);
    let same_context_negative = assertion("concept-negative-conflict", AssertionPolarity::Negative);
    assert!(detect_contradiction(&positive, &same_context_negative, &BTreeSet::new()).is_some());

    let changed_context = KnowledgeAssertion::new(
        positive.subject().clone(),
        positive.predicate().clone(),
        same_context_negative.object().clone(),
        AssertionContext::from_entries([("time-frame", "different")]).unwrap(),
        positive.qualifiers().clone(),
        AssertionStatus::Known,
        AssertionPolarity::Negative,
    );
    assert!(detect_contradiction(&positive, &changed_context, &BTreeSet::new()).is_none());

    let assertion_id = KnowledgeAssertionId::new("assertion-one-only").unwrap();
    let result = nizaam_knowledge_graph::uncertainty::Contradiction::new(
        ContradictionId::new("contradiction-one-assertion").unwrap(),
        [assertion_id.clone(), assertion_id],
        nizaam_knowledge_graph::uncertainty::ContradictionKind::OpposedPolarity,
        AssertionContext::new(),
        Qualifiers::new(),
    );
    assert_eq!(
        result,
        Err(ContradictionError::RequiresTwoDistinctAssertions)
    );
}
