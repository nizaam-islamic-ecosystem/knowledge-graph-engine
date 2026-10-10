//! Level 3 tests for source-class approval policy and checked governance transitions.

use nizaam_knowledge_graph::identity::{AgentId, SourceId};
use nizaam_knowledge_graph::ingestion::{
    ApprovalDecision, ApprovalOutcome, ApprovalPolicy, CandidateKey, CurationOutcome,
    CurationRequirement, GenericTextNormalizer, GovernanceActor, GovernanceRecord, GovernanceState,
    MappedCandidate, MappingMetadata, NormalizationStage, SourceApprovalRule, SourceAuthenticity,
    SourceClass, SourceRecord, SourceRecordMetadata, TextNormalizationMode, ValidationResult,
};
use nizaam_knowledge_graph::temporal::Instant;

fn candidate() -> MappedCandidate<()> {
    let metadata = SourceRecordMetadata::new(
        SourceId::new("source-governance").unwrap(),
        SourceClass::structured_data(),
        "record-1",
        Some("snapshot-1".to_owned()),
        None,
        [],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "record text".to_owned()).unwrap();
    let normalizer =
        GenericTextNormalizer::new("normalizer-v1", "config-v1", TextNormalizationMode::Trim)
            .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(1))
        .unwrap()
        .remove(0);
    MappedCandidate::from_normalized(
        normalized,
        0,
        (),
        MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
    )
    .unwrap()
}

fn approval(
    candidate: &MappedCandidate<()>,
    outcome: ApprovalOutcome,
    curation: CurationOutcome,
) -> ApprovalDecision {
    ApprovalDecision::new(
        candidate.key().clone(),
        candidate.source().source_class().clone(),
        outcome,
        AgentId::new("reviewer-governance").unwrap(),
        Instant::from_unix_seconds(5),
        curation,
        Some("review decision recorded".to_owned()),
    )
    .unwrap()
}

#[test]
fn human_approval_is_required_even_for_authentic_content() {
    let candidate = candidate();
    let validation = ValidationResult::valid(candidate.key().clone());
    let policy = ApprovalPolicy::phase5_default();
    let rejected = approval(
        &candidate,
        ApprovalOutcome::Rejected,
        CurationOutcome::NotRequired,
    );
    let approved = approval(
        &candidate,
        ApprovalOutcome::Approved,
        CurationOutcome::Deferred,
    );

    assert!(
        policy
            .validate_for_publication(
                &candidate,
                &validation,
                &rejected,
                SourceAuthenticity::Authentic
            )
            .is_err()
    );
    assert!(
        policy
            .validate_for_publication(
                &candidate,
                &validation,
                &approved,
                SourceAuthenticity::Authentic
            )
            .is_ok()
    );
}

#[test]
fn non_authenticity_states_require_explicit_source_class_policy_allowance() {
    let candidate = candidate();
    let validation = ValidationResult::valid(candidate.key().clone());
    let approval = approval(
        &candidate,
        ApprovalOutcome::Approved,
        CurationOutcome::Deferred,
    );
    let default_policy = ApprovalPolicy::phase5_default();
    assert!(
        default_policy
            .validate_for_publication(
                &candidate,
                &validation,
                &approval,
                SourceAuthenticity::Unverified
            )
            .is_err()
    );

    let mut permissive_policy = ApprovalPolicy::phase5_default();
    permissive_policy.set_rule(
        SourceClass::structured_data(),
        SourceApprovalRule::new(CurationRequirement::Optional, true, false, false),
    );
    assert!(
        permissive_policy
            .validate_for_publication(
                &candidate,
                &validation,
                &approval,
                SourceAuthenticity::Unverified
            )
            .is_ok()
    );
}

#[test]
fn required_curation_cannot_be_deferred() {
    let candidate = candidate();
    let validation = ValidationResult::valid(candidate.key().clone());
    let mut policy = ApprovalPolicy::phase5_default();
    policy.set_rule(
        SourceClass::structured_data(),
        SourceApprovalRule::new(CurationRequirement::Required, false, false, false),
    );
    let deferred = approval(
        &candidate,
        ApprovalOutcome::Approved,
        CurationOutcome::Deferred,
    );
    let performed = approval(
        &candidate,
        ApprovalOutcome::Approved,
        CurationOutcome::Performed,
    );

    assert!(
        policy
            .validate_for_publication(
                &candidate,
                &validation,
                &deferred,
                SourceAuthenticity::Authentic
            )
            .is_err()
    );
    assert!(
        policy
            .validate_for_publication(
                &candidate,
                &validation,
                &performed,
                SourceAuthenticity::Authentic
            )
            .is_ok()
    );
}

#[test]
fn governance_history_rejects_invalid_transition_without_mutating_state() {
    let key = CandidateKey::new(
        SourceId::new("source-governance-history").unwrap(),
        "record-1",
        None,
        0,
    )
    .unwrap();
    let mut record = GovernanceRecord::new(key);
    record
        .transition(
            GovernanceState::AwaitingApproval,
            Instant::from_unix_seconds(1),
            GovernanceActor::System("validation-stage".to_owned()),
            "validation passed",
        )
        .unwrap();
    record
        .transition(
            GovernanceState::Approved,
            Instant::from_unix_seconds(2),
            GovernanceActor::Human(AgentId::new("reviewer-1").unwrap()),
            "human approved",
        )
        .unwrap();

    assert!(
        record
            .transition(
                GovernanceState::Quarantined,
                Instant::from_unix_seconds(3),
                GovernanceActor::System("publication-stage".to_owned()),
                "illegal backwards transition",
            )
            .is_err()
    );
    assert_eq!(record.state(), GovernanceState::Approved);
    assert_eq!(record.history().len(), 2);
}
