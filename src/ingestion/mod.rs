//! Phase 5 governed ingestion boundary.
//!
//! The module assembles source capture, deterministic normalization, semantic
//! mapping/deduplication, layered validation, approval/governance, logical
//! publication, and pipeline/reprocessing contracts. It reuses Phase 0–4 models,
//! Core operation/artifact identities, and the separate Indexing readiness boundary.
//! It does not implement physical KG storage, index activation, a generic workflow
//! engine, advanced text extraction, or language-specific morphology.

mod approval;
mod mapping;
mod normalize;
mod pipeline;
mod publication;
mod raw;
mod validation;

pub use approval::{
    ApprovalDecision, ApprovalError, ApprovalOutcome, ApprovalPolicy, CurationOutcome,
    CurationRequirement, GovernanceActor, GovernanceRecord, GovernanceState, GovernanceTransition,
    SourceApprovalRule, SourceAuthenticity,
};
pub use mapping::{
    CandidateKey, CanonicalSemanticIdentity, DeduplicationResult, MappedCandidate, MappingError,
    MappingMetadata, SemanticDeduplicationKey, SemanticDeduplicator, SemanticMapper,
    SourceDeduplicationKey, SourceDeduplicator,
};
pub use normalize::{
    GenericTextNormalizer, NormalizationError, NormalizationMetadata, NormalizationStage,
    NormalizedRecord, TextNormalizationMode,
};
pub use pipeline::{
    CandidateProgress, CandidateState, CandidateTransition, IngestionPipeline, IngestionRun,
    IngestionRunState, PipelineError, PipelinePlan, PipelineStage, PipelineStageHook,
    PipelineStageVersion, PipelineStepVersion, ReprocessingMode, ReprocessingRequest,
    RunCompletion, SourceDelta, StageExecution, StageExecutionInput, StageExecutionOutcome,
};
pub use publication::{
    CanonicalKnowledgeSubgraph, CorrectionRecord, PublicationCoordinator, PublicationDecision,
    PublicationError, PublicationOutcome, PublicationRecord, PublicationRequest,
    SubgraphValidationError, WithdrawalRecord,
};
pub use raw::{
    RawMaterial, RawMaterialError, SourceAdapter, SourceAdapterError, SourceClass,
    SourceMetadataError, SourceRecord, SourceRecordError, SourceRecordMetadata,
    SourceRecordMetadataError, StructuredJsonAdapter,
};
pub use validation::{
    ValidationError, ValidationFinding, ValidationPolicy, ValidationResult, ValidationSeverity,
    ValidationStage, ValidationStatus, validate_candidate_structure,
};

#[cfg(test)]
mod tests {
    use super::{
        ApprovalPolicy, CandidateKey, GenericTextNormalizer, IngestionPipeline, MappingMetadata,
        NormalizationStage, PipelineStage, PipelineStageHook, PipelineStageVersion, SourceClass,
        SourceRecord, SourceRecordMetadata, TextNormalizationMode,
    };
    use crate::identity::SourceId;
    use crate::temporal::Instant;

    #[test]
    fn public_ingestion_api_exposes_the_fixed_pipeline_and_source_classes() {
        let pipeline = IngestionPipeline::phase5_default();
        assert_eq!(
            pipeline.plan().ordered_steps().first().unwrap().name(),
            "raw-capture"
        );
        assert_eq!(
            pipeline.plan().ordered_steps().last().unwrap().name(),
            "index-synchronization"
        );
        assert_eq!(SourceClass::structured_data().as_str(), "structured-data");
        assert_eq!(
            ApprovalPolicy::phase5_default().configuration_version(),
            "phase5-approval-v1"
        );
    }

    #[test]
    fn public_normalization_and_mapping_contracts_preserve_candidate_keys() {
        let metadata = SourceRecordMetadata::new(
            SourceId::new("source-ingestion-public").unwrap(),
            SourceClass::structured_data(),
            "record-42",
            Some("snapshot-7".to_owned()),
            None,
            [],
        )
        .unwrap();
        let record = SourceRecord::new(metadata, "  one   two  ".to_owned()).unwrap();
        let normalizer = GenericTextNormalizer::new(
            "text-v1",
            "whitespace-v1",
            TextNormalizationMode::CollapseWhitespace,
        )
        .unwrap();
        let normalized = normalizer
            .normalize(record, Instant::from_unix_seconds(42))
            .unwrap()
            .remove(0);
        let candidate_key = CandidateKey::new(
            normalized.source().source_id().clone(),
            normalized.source().source_record_key(),
            normalized.source().source_version().map(str::to_owned),
            0,
        )
        .unwrap();
        let mapping = MappingMetadata::new("mapping-v1", "mapping-config-v1").unwrap();
        assert_eq!(candidate_key.source_record_key(), "record-42");
        assert_eq!(mapping.stage_version(), "mapping-v1");
    }

    #[test]
    fn source_specific_hooks_do_not_replace_mandatory_pipeline_stages() {
        let base = IngestionPipeline::phase5_default();
        let hook = PipelineStageHook::new(
            "record-enrichment",
            PipelineStage::Mapping,
            "hook-v1",
            "config-v1",
        )
        .unwrap();
        let hooked = base.plan().with_hook(hook).unwrap();
        assert!(
            hooked
                .ordered_steps()
                .iter()
                .any(|step| step.name() == "record-enrichment")
        );
        for stage in PipelineStage::CORE_ORDER {
            assert!(
                hooked
                    .ordered_steps()
                    .iter()
                    .any(|step| step.name() == stage.as_str())
            );
        }
        assert!(PipelineStageVersion::new(PipelineStage::Normalization, "v1", "config-v1").is_ok());
    }
}
