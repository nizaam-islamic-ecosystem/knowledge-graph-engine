//! Level 3 cross-module integration: raw source data through logical publication.

use nizaam_core::artifact::ArtifactReference;
use nizaam_core::identity::{ArtifactId, OperationId};
use nizaam_indexing::index::reference::ObjectReference;
use nizaam_indexing::{IndexAssignedId, TargetReferenceType};
use nizaam_knowledge_graph::entity::{Entity, ExternalIdentifier, Name};
use nizaam_knowledge_graph::identity::{AgentId, EntityId, SourceId};
use nizaam_knowledge_graph::ingestion::{
    ApprovalDecision, ApprovalOutcome, ApprovalPolicy, CanonicalKnowledgeSubgraph, CurationOutcome,
    GenericTextNormalizer, MappedCandidate, MappingMetadata, NormalizationStage,
    PublicationCoordinator, PublicationOutcome, PublicationRequest, RawMaterial, SourceAdapter,
    SourceAuthenticity, SourceClass, SourceRecord, StructuredJsonAdapter, TextNormalizationMode,
    ValidationPolicy, validate_candidate_structure,
};
use nizaam_knowledge_graph::integration::indexing::{
    IndexingPublicationReadiness, IndexingReadinessReceipt, IndexingSynchronizationStatus,
};
use nizaam_knowledge_graph::resolution::ExternalIdentifierCrosswalk;
use nizaam_knowledge_graph::temporal::Instant;

#[test]
fn source_record_can_be_normalized_mapped_validated_approved_and_published() {
    let source_id = SourceId::new("source-phase5-integration").unwrap();
    let raw = RawMaterial::new(
        OperationId::new("nizaam.kg.phase5.integration.run").unwrap(),
        source_id.clone(),
        SourceClass::structured_data(),
        ArtifactReference::new(
            ArtifactId::new("artifact-phase5-integration").unwrap(),
            "v1",
        ),
        "application/json",
        Instant::from_unix_seconds(1_750_000_100),
    )
    .unwrap()
    .with_source_version("snapshot-1")
    .unwrap();
    let adapter = StructuredJsonAdapter::new(source_id.clone())
        .with_record_key_field("id")
        .unwrap()
        .with_external_identifier_field("external_id")
        .unwrap();
    let decoded = adapter
        .decode(
            &raw,
            br#"{"id":"record-1","external_id":"person-ext-1","name":"Amina"}"#,
        )
        .unwrap();
    assert_eq!(decoded.len(), 1);

    let decoded_record = decoded.into_iter().next().unwrap();
    let name = decoded_record.payload()["name"]
        .as_str()
        .unwrap()
        .to_owned();
    let record = SourceRecord::new(decoded_record.metadata().clone(), name).unwrap();
    let normalizer =
        GenericTextNormalizer::new("text-normalizer-v1", "trim-v1", TextNormalizationMode::Trim)
            .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(1_750_000_101))
        .unwrap()
        .remove(0);
    let candidate = MappedCandidate::from_normalized(
        normalized,
        0,
        (),
        MappingMetadata::new("person-mapper-v1", "person-mapping-v1").unwrap(),
    )
    .unwrap();

    assert_eq!(candidate.payload(), &());
    assert_eq!(candidate.key().source_record_key(), "record-1");
    assert_eq!(candidate.source().source_version(), Some("snapshot-1"));
    let external: ExternalIdentifier = candidate
        .external_identifiers()
        .iter()
        .next()
        .unwrap()
        .clone();
    assert_eq!(external.source_id(), &source_id);
    assert_eq!(external.value(), "person-ext-1");

    let validation = validate_candidate_structure(&candidate, &ValidationPolicy::new());
    assert!(validation.is_publishable());
    let approval_policy = ApprovalPolicy::phase5_default();
    let approval = ApprovalDecision::new(
        candidate.key().clone(),
        candidate.source().source_class().clone(),
        ApprovalOutcome::Approved,
        AgentId::new("reviewer-phase5-integration").unwrap(),
        Instant::from_unix_seconds(1_750_000_102),
        CurationOutcome::Deferred,
        Some("source record reviewed".to_owned()),
    )
    .unwrap();

    let entity_id = EntityId::new("entity-phase5-integration").unwrap();
    let entity = Entity::new(entity_id.clone(), Name::new("Amina", "en"), Vec::new())
        .with_external_identifier(external.clone());
    let crosswalk = ExternalIdentifierCrosswalk::new(external, entity_id.clone());
    let subgraph = CanonicalKnowledgeSubgraph::new()
        .with_entities([entity])
        .with_external_identifier_crosswalks([crosswalk]);
    assert!(subgraph.validate().is_ok());

    let object_reference = ObjectReference::new("source-phase5-integration", "record-1").unwrap();
    let target_type = TargetReferenceType::new("entity").unwrap();
    let assigned_id = IndexAssignedId::generate(&target_type, &object_reference);
    let readiness = IndexingReadinessReceipt::new(
        assigned_id.clone(),
        object_reference,
        IndexingPublicationReadiness::Ready,
    );
    let published = PublicationCoordinator
        .publish(PublicationRequest {
            candidate: &candidate,
            subgraph,
            validation: &validation,
            approval_policy: &approval_policy,
            approval: &approval,
            authenticity: SourceAuthenticity::Authentic,
            readiness: &readiness,
            published_at: Instant::from_unix_seconds(1_750_000_103),
        })
        .unwrap();

    assert_eq!(
        published.decision().outcome(),
        PublicationOutcome::Published
    );
    assert_eq!(published.subgraph().entities()[0].id(), &entity_id);
    assert_eq!(
        published.subgraph().external_identifier_crosswalks()[0].canonical_entity(),
        &entity_id
    );
    assert_eq!(published.readiness().assigned_id(), &assigned_id);
    assert_eq!(
        published.synchronization().status(),
        IndexingSynchronizationStatus::Pending
    );
}

#[test]
fn post_publication_index_failure_is_recoverable_without_rewriting_publication_decision() {
    let source_id = SourceId::new("source-phase5-sync").unwrap();
    let metadata = nizaam_knowledge_graph::ingestion::SourceRecordMetadata::new(
        source_id.clone(),
        SourceClass::structured_data(),
        "record-sync",
        Some("v1".to_owned()),
        None,
        [],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "name".to_owned()).unwrap();
    let normalizer =
        GenericTextNormalizer::new("normalizer-v1", "config-v1", TextNormalizationMode::Trim)
            .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(1))
        .unwrap()
        .remove(0);
    let candidate = MappedCandidate::from_normalized(
        normalized,
        0,
        (),
        MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
    )
    .unwrap();
    let validation = validate_candidate_structure(&candidate, &ValidationPolicy::new());
    let approval_policy = ApprovalPolicy::phase5_default();
    let approval = ApprovalDecision::new(
        candidate.key().clone(),
        candidate.source().source_class().clone(),
        ApprovalOutcome::Approved,
        AgentId::new("reviewer-sync").unwrap(),
        Instant::from_unix_seconds(2),
        CurationOutcome::Deferred,
        None,
    )
    .unwrap();
    let entity_id = EntityId::new("entity-sync").unwrap();
    let subgraph = CanonicalKnowledgeSubgraph::new().with_entities([Entity::new(
        entity_id,
        Name::new("Sync entity", "en"),
        Vec::new(),
    )]);
    let reference = ObjectReference::new(source_id.as_str(), "record-sync").unwrap();
    let assigned_id =
        IndexAssignedId::generate(&TargetReferenceType::new("entity").unwrap(), &reference);
    let readiness =
        IndexingReadinessReceipt::new(assigned_id, reference, IndexingPublicationReadiness::Ready);
    let published = PublicationCoordinator
        .publish(PublicationRequest {
            candidate: &candidate,
            subgraph,
            validation: &validation,
            approval_policy: &approval_policy,
            approval: &approval,
            authenticity: SourceAuthenticity::Authentic,
            readiness: &readiness,
            published_at: Instant::from_unix_seconds(3),
        })
        .unwrap();
    let retry = published
        .clone()
        .with_synchronization_status(IndexingSynchronizationStatus::RetryableFailure);
    assert_eq!(retry.decision().outcome(), PublicationOutcome::Published);
    assert_eq!(
        retry.synchronization().status(),
        IndexingSynchronizationStatus::RetryableFailure
    );
    assert_eq!(
        published.synchronization().status(),
        IndexingSynchronizationStatus::Pending
    );
}
