//! Level 3 tests for per-candidate logical publication and its independent Indexing status.

use nizaam_indexing::index::reference::ObjectReference;
use nizaam_indexing::{IndexAssignedId, TargetReferenceType};
use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::entity::{Entity, Name};
use nizaam_knowledge_graph::identity::{AgentId, ConceptId, EntityId, SourceId};
use nizaam_knowledge_graph::ingestion::{
    ApprovalDecision, ApprovalOutcome, ApprovalPolicy, CanonicalKnowledgeSubgraph,
    CorrectionRecord, CurationOutcome, GenericTextNormalizer, MappedCandidate, MappingMetadata,
    NormalizationStage, PublicationCoordinator, PublicationOutcome, PublicationRequest,
    SourceAuthenticity, SourceClass, SourceRecord, SourceRecordMetadata, TextNormalizationMode,
    ValidationResult,
};
use nizaam_knowledge_graph::integration::indexing::{
    IndexingPublicationReadiness, IndexingReadinessReceipt, IndexingSynchronizationStatus,
};
use nizaam_knowledge_graph::temporal::Instant;

fn candidate_for(record_key: &str) -> MappedCandidate<()> {
    let metadata = SourceRecordMetadata::new(
        SourceId::new("source-publication-level3").unwrap(),
        SourceClass::structured_data(),
        record_key,
        Some("snapshot-1".to_owned()),
        None,
        [],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "semantic candidate".to_owned()).unwrap();
    let normalizer =
        GenericTextNormalizer::new("normalizer-v1", "config-v1", TextNormalizationMode::Trim)
            .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(10))
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

fn readiness(record_key: &str, state: IndexingPublicationReadiness) -> IndexingReadinessReceipt {
    let reference = ObjectReference::new("source-publication-level3", record_key).unwrap();
    let target = TargetReferenceType::new("entity").unwrap();
    IndexingReadinessReceipt::new(
        IndexAssignedId::generate(&target, &reference),
        reference,
        state,
    )
}

fn approved(candidate: &MappedCandidate<()>) -> ApprovalDecision {
    ApprovalDecision::new(
        candidate.key().clone(),
        candidate.source().source_class().clone(),
        ApprovalOutcome::Approved,
        AgentId::new("reviewer-publication").unwrap(),
        Instant::from_unix_seconds(20),
        CurationOutcome::Deferred,
        Some("reviewed and approved".to_owned()),
    )
    .unwrap()
}

fn subgraph() -> CanonicalKnowledgeSubgraph {
    let entity = Entity::new(
        EntityId::new("entity-publication-level3").unwrap(),
        Name::new("Amina", "en"),
        Vec::new(),
    );
    CanonicalKnowledgeSubgraph::new().with_entities([entity])
}

fn publish(
    candidate: &MappedCandidate<()>,
    subgraph: CanonicalKnowledgeSubgraph,
    validation: &ValidationResult,
    approval: &ApprovalDecision,
    readiness: &IndexingReadinessReceipt,
) -> Result<
    nizaam_knowledge_graph::ingestion::PublicationRecord,
    nizaam_knowledge_graph::ingestion::PublicationError,
> {
    let policy = ApprovalPolicy::phase5_default();
    PublicationCoordinator.publish(PublicationRequest {
        candidate,
        subgraph,
        validation,
        approval_policy: &policy,
        approval,
        authenticity: SourceAuthenticity::Authentic,
        readiness,
        published_at: Instant::from_unix_seconds(30),
    })
}

#[test]
fn publication_requires_validation_human_approval_and_explicit_indexing_readiness() {
    let candidate = candidate_for("record-1");
    let validation = ValidationResult::valid(candidate.key().clone());
    let approval = approved(&candidate);
    let ready = readiness("record-1", IndexingPublicationReadiness::Ready);
    let record = publish(&candidate, subgraph(), &validation, &approval, &ready).unwrap();

    assert_eq!(record.decision().outcome(), PublicationOutcome::Published);
    assert_eq!(record.subgraph().entities().len(), 1);
    assert_eq!(record.readiness().assigned_id(), ready.assigned_id());
    assert_eq!(
        record.synchronization().status(),
        IndexingSynchronizationStatus::Pending
    );

    let blocked = readiness(
        "record-1",
        IndexingPublicationReadiness::Blocked(
            nizaam_knowledge_graph::integration::indexing::IndexingReadinessBlocker::Stale,
        ),
    );
    assert!(publish(&candidate, subgraph(), &validation, &approval, &blocked).is_err());
}

#[test]
fn indexing_sync_failure_does_not_rewrite_successful_logical_publication() {
    let candidate = candidate_for("record-2");
    let validation = ValidationResult::valid(candidate.key().clone());
    let approval = approved(&candidate);
    let ready = readiness("record-2", IndexingPublicationReadiness::Ready);
    let published = publish(&candidate, subgraph(), &validation, &approval, &ready).unwrap();
    let retryable = published
        .clone()
        .with_synchronization_status(IndexingSynchronizationStatus::RetryableFailure);

    assert_eq!(
        retryable.decision().outcome(),
        PublicationOutcome::Published
    );
    assert_eq!(
        retryable.synchronization().status(),
        IndexingSynchronizationStatus::RetryableFailure
    );
    assert_eq!(
        published.synchronization().status(),
        IndexingSynchronizationStatus::Pending
    );
}

#[test]
fn withdrawal_and_correction_are_separate_records_from_the_original_publication() {
    let candidate = candidate_for("record-3");
    let validation = ValidationResult::valid(candidate.key().clone());
    let approval = approved(&candidate);
    let ready = readiness("record-3", IndexingPublicationReadiness::Ready);
    let published = publish(&candidate, subgraph(), &validation, &approval, &ready).unwrap();

    let withdrawal = PublicationCoordinator
        .withdraw(
            &published,
            AgentId::new("reviewer-withdrawal").unwrap(),
            Instant::from_unix_seconds(40),
            "source later withdrawn",
        )
        .unwrap();
    assert_eq!(
        withdrawal.prior_publication().outcome(),
        PublicationOutcome::Published
    );
    assert_eq!(
        withdrawal.decision().outcome(),
        PublicationOutcome::Withdrawn
    );
    assert_eq!(
        published.decision().outcome(),
        PublicationOutcome::Published
    );

    let successor = candidate_for("record-3-corrected");
    let correction: CorrectionRecord = PublicationCoordinator
        .record_correction(
            &published,
            &successor,
            AgentId::new("reviewer-correction").unwrap(),
            Instant::from_unix_seconds(41),
            "corrected source record",
        )
        .unwrap();
    assert_eq!(correction.prior_publication().candidate(), candidate.key());
    assert_eq!(correction.successor_candidate(), successor.key());
}

#[test]
fn assertion_concept_references_must_be_local_or_explicitly_existing() {
    let entity_id = EntityId::new("entity-with-assertion").unwrap();
    let concept_id = ConceptId::new("concept-existing").unwrap();
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
        .with_assertions([assertion]);
    assert!(incomplete.validate().is_err());
    assert!(
        incomplete
            .with_existing_concept_ids([concept_id])
            .validate()
            .is_ok()
    );
}
