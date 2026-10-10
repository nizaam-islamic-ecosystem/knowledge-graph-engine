//! Level 3 negative-boundary tests for invalid input, mismatched identities and stage ordering.

use nizaam_core::identity::{ArtifactId, OperationId};
use nizaam_indexing::index::reference::ObjectReference;
use nizaam_indexing::{IndexAssignedId, TargetReferenceType};
use nizaam_knowledge_graph::entity::ExternalIdentifier;
use nizaam_knowledge_graph::identity::{AgentId, SourceId};
use nizaam_knowledge_graph::ingestion::{
    ApprovalDecision, ApprovalOutcome, ApprovalPolicy, CandidateKey, CurationOutcome,
    GenericTextNormalizer, MappedCandidate, MappingError, MappingMetadata, NormalizationStage,
    PipelineError, PipelinePlan, RawMaterial, ReprocessingMode, ReprocessingRequest, SourceAdapter,
    SourceClass, SourceDelta, SourceRecord, SourceRecordMetadata, StageExecutionInput,
    StageExecutionOutcome, StructuredJsonAdapter, TextNormalizationMode, ValidationFinding,
    ValidationSeverity, ValidationStage,
};
use nizaam_knowledge_graph::integration::indexing::{
    IndexingPublicationReadiness, IndexingReadinessBlocker, IndexingReadinessReceipt,
};
use nizaam_knowledge_graph::temporal::Instant;

fn mapped_candidate() -> MappedCandidate<()> {
    let metadata = SourceRecordMetadata::new(
        SourceId::new("source-negative-boundary").unwrap(),
        SourceClass::structured_data(),
        "record-1",
        Some("snapshot-1".to_owned()),
        None,
        [],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "payload".to_owned()).unwrap();
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

#[test]
fn raw_adapter_rejects_invalid_json_roots_and_wrong_source_identity() {
    let source = SourceId::new("source-negative-raw").unwrap();
    let adapter = StructuredJsonAdapter::new(source.clone());
    let raw = RawMaterial::new(
        OperationId::new("nizaam.kg.negative.raw").unwrap(),
        source.clone(),
        SourceClass::structured_data(),
        nizaam_core::artifact::ArtifactReference::new(
            ArtifactId::new("artifact-negative-raw").unwrap(),
            "v1",
        ),
        "application/json",
        Instant::from_unix_seconds(1),
    )
    .unwrap();
    assert!(adapter.decode(&raw, b"null").is_err());
    assert!(adapter.decode(&raw, br#"[1,2,3]"#).is_err());

    let wrong_source = RawMaterial::new(
        OperationId::new("nizaam.kg.negative.raw-other").unwrap(),
        SourceId::new("other-source").unwrap(),
        SourceClass::structured_data(),
        nizaam_core::artifact::ArtifactReference::new(
            ArtifactId::new("artifact-negative-other").unwrap(),
            "v1",
        ),
        "application/json",
        Instant::from_unix_seconds(1),
    )
    .unwrap();
    assert!(adapter.decode(&wrong_source, br#"{"id":"1"}"#).is_err());
}

#[test]
fn source_metadata_and_mapping_reject_cross_source_external_identifiers() {
    let source = SourceId::new("source-negative-metadata").unwrap();
    let foreign =
        ExternalIdentifier::new(SourceId::new("foreign-source").unwrap(), "record-x").unwrap();
    assert!(
        SourceRecordMetadata::new(
            source.clone(),
            SourceClass::structured_data(),
            "record-x",
            None,
            None,
            [foreign.clone()],
        )
        .is_err()
    );

    let candidate = mapped_candidate();
    assert!(matches!(
        candidate.with_external_identifier(foreign),
        Err(MappingError::ExternalIdentifierSourceMismatch { .. })
    ));
}

#[test]
fn invalid_stage_labels_and_validation_codes_are_rejected() {
    assert!(GenericTextNormalizer::new(" ", "config-v1", TextNormalizationMode::Trim).is_err());
    assert!(SourceClass::new("\n").is_err());
    assert!(CandidateKey::new(SourceId::new("source-key").unwrap(), " ", None, 0).is_err());
    assert!(
        ValidationFinding::new(
            " ",
            ValidationSeverity::Warning,
            ValidationStage::Semantic,
            "message",
        )
        .is_err()
    );
}

#[test]
fn pipeline_rejects_stage_skips_and_reversed_attempt_times() {
    let candidate = mapped_candidate();
    let key = candidate.key().clone();
    let plan = PipelinePlan::phase5_default();
    let mut run = nizaam_knowledge_graph::ingestion::IngestionPipeline::new(plan.clone())
        .begin_run(
            OperationId::new("nizaam.kg.negative.pipeline").unwrap(),
            SourceId::new("source-negative-boundary").unwrap(),
            SourceClass::structured_data(),
            Some("snapshot-1".to_owned()),
            Instant::from_unix_seconds(1),
        )
        .unwrap();
    run.start().unwrap();
    run.register_candidate(&candidate).unwrap();
    let steps = plan.ordered_steps();
    let skipped = StageExecutionInput::new(
        steps[1].clone(),
        1,
        StageExecutionOutcome::Succeeded,
        Instant::from_unix_seconds(2),
        Instant::from_unix_seconds(3),
        None,
    );
    assert!(matches!(
        run.record_stage_execution(&key, skipped),
        Err(PipelineError::StageOrderMismatch { .. })
    ));

    let reversed = StageExecutionInput::new(
        steps[0].clone(),
        1,
        StageExecutionOutcome::Succeeded,
        Instant::from_unix_seconds(3),
        Instant::from_unix_seconds(2),
        None,
    );
    assert_eq!(
        run.record_stage_execution(&key, reversed),
        Err(PipelineError::InvalidStageTimeRange)
    );
}

#[test]
fn reprocessing_rejects_empty_or_mismatched_scopes() {
    assert_eq!(
        SourceDelta::new("v1", "v2", Vec::<String>::new()),
        Err(PipelineError::EmptySourceDelta)
    );
    let result = ReprocessingRequest::new(
        OperationId::new("nizaam.kg.negative.reprocessing").unwrap(),
        SourceId::new("source-one").unwrap(),
        Some("v1".to_owned()),
        ReprocessingMode::TargetedCandidates(
            [CandidateKey::new(
                SourceId::new("source-two").unwrap(),
                "record-1",
                Some("v1".to_owned()),
                0,
            )
            .unwrap()]
            .into_iter()
            .collect(),
        ),
        Instant::from_unix_seconds(5),
        "mismatched source",
    );
    assert_eq!(result, Err(PipelineError::ReprocessingTargetMismatch));
}

#[test]
fn unknown_or_blocked_indexing_readiness_never_opens_publication_gate() {
    let reference = ObjectReference::new("source-one", "record-1").unwrap();
    let target = TargetReferenceType::new("entity").unwrap();
    let id = IndexAssignedId::generate(&target, &reference);
    let blocked = IndexingReadinessReceipt::new(
        id.clone(),
        reference.clone(),
        IndexingPublicationReadiness::Blocked(IndexingReadinessBlocker::DependencyUnavailable),
    );
    let unknown =
        IndexingReadinessReceipt::new(id, reference, IndexingPublicationReadiness::Unknown);
    assert!(blocked.require_ready().is_err());
    assert!(unknown.require_ready().is_err());
}

#[test]
fn approval_decision_must_target_the_same_candidate() {
    let candidate = mapped_candidate();
    let other = CandidateKey::new(
        SourceId::new("source-negative-boundary").unwrap(),
        "different-record",
        Some("snapshot-1".to_owned()),
        0,
    )
    .unwrap();
    let decision = ApprovalDecision::new(
        other,
        candidate.source().source_class().clone(),
        ApprovalOutcome::Approved,
        AgentId::new("reviewer-negative").unwrap(),
        Instant::from_unix_seconds(4),
        CurationOutcome::Deferred,
        Some("wrong target".to_owned()),
    )
    .unwrap();
    let validation =
        nizaam_knowledge_graph::ingestion::ValidationResult::valid(candidate.key().clone());
    assert!(
        ApprovalPolicy::phase5_default()
            .validate_for_publication(
                &candidate,
                &validation,
                &decision,
                nizaam_knowledge_graph::ingestion::SourceAuthenticity::Authentic
            )
            .is_err()
    );
}
