//! Level 3 tests for public Phase 5 raw capture, normalization, mapping and validation.

use nizaam_core::artifact::ArtifactReference;
use nizaam_core::identity::{ArtifactId, OperationId};
use nizaam_knowledge_graph::entity::ExternalIdentifier;
use nizaam_knowledge_graph::identity::{EntityId, ReferenceId, SourceId};
use nizaam_knowledge_graph::ingestion::{
    CandidateKey, DeduplicationResult, GenericTextNormalizer, MappedCandidate, MappingMetadata,
    NormalizationStage, RawMaterial, SourceAdapter, SourceClass, SourceDeduplicator, SourceRecord,
    SourceRecordMetadata, StructuredJsonAdapter, TextNormalizationMode, ValidationPolicy,
    ValidationStatus, validate_candidate_structure,
};
use nizaam_knowledge_graph::resolution::{
    EntityCandidateProfile, ResolutionInput, ResolutionReference, ResolutionState, Resolver,
};
use nizaam_knowledge_graph::temporal::Instant;

fn raw_material(source: SourceId, version: Option<&str>) -> RawMaterial {
    let raw = RawMaterial::new(
        OperationId::new("nizaam.kg.test.ingestion.operation").unwrap(),
        source,
        SourceClass::structured_data(),
        ArtifactReference::new(ArtifactId::new("artifact-ingestion-test").unwrap(), "v1"),
        "application/json; charset=utf-8",
        Instant::from_unix_seconds(1_750_000_000),
    )
    .unwrap();
    match version {
        Some(version) => raw.with_source_version(version).unwrap(),
        None => raw,
    }
}

#[test]
fn structured_json_ingestion_preserves_source_snapshot_and_external_identity() {
    let source = SourceId::new("source-json-ingestion").unwrap();
    let adapter = StructuredJsonAdapter::new(source.clone())
        .with_record_key_field("id")
        .unwrap()
        .with_external_identifier_field("external_id")
        .unwrap();
    let raw = raw_material(source.clone(), Some("snapshot-5"));

    let records = adapter.decode(
        &raw,
        br#"[{"id":"record-1","external_id":"external-1","name":"Amina"},{"id":2,"name":"Bilal"}]"#,
    ).unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].metadata().source_id(), &source);
    assert_eq!(
        records[0].metadata().source_class(),
        &SourceClass::structured_data()
    );
    assert_eq!(records[0].metadata().source_record_key(), "record-1");
    assert_eq!(records[0].metadata().source_version(), Some("snapshot-5"));
    assert_eq!(records[0].metadata().source_locator(), Some("/0"));
    assert_eq!(records[0].metadata().external_identifiers().len(), 1);
    assert_eq!(
        records[0]
            .metadata()
            .external_identifiers()
            .first()
            .unwrap()
            .value(),
        "external-1"
    );
    assert_eq!(records[1].metadata().source_record_key(), "2");
    assert!(records[1].metadata().external_identifiers().is_empty());
}

#[test]
fn normalizing_and_mapping_keeps_source_key_versions_and_candidate_key_stable() {
    let source = SourceId::new("source-normalize-map").unwrap();
    let external = ExternalIdentifier::new(source.clone(), "catalog-7").unwrap();
    let metadata = SourceRecordMetadata::new(
        source.clone(),
        SourceClass::structured_data(),
        "record-7",
        Some("snapshot-2".to_owned()),
        Some("/items/7".to_owned()),
        [external.clone()],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "  Amina\t bint\n Hassan  ".to_owned()).unwrap();
    let normalizer = GenericTextNormalizer::new(
        "text-normalizer-v1",
        "collapse-whitespace-v1",
        TextNormalizationMode::CollapseWhitespace,
    )
    .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(41))
        .unwrap()
        .remove(0);
    assert_eq!(normalized.payload(), "Amina bint Hassan");

    let candidate = MappedCandidate::from_normalized(
        normalized,
        0,
        "candidate-payload".to_owned(),
        MappingMetadata::new("mapper-v1", "mapping-config-v1").unwrap(),
    )
    .unwrap();
    let expected = CandidateKey::new(source, "record-7", Some("snapshot-2".to_owned()), 0).unwrap();

    assert_eq!(candidate.key(), &expected);
    assert_eq!(candidate.source().source_locator(), Some("/items/7"));
    assert!(candidate.external_identifiers().contains(&external));
    assert_eq!(
        candidate.normalization().stage_version(),
        "text-normalizer-v1"
    );
    assert_eq!(
        candidate.mapping().configuration_version(),
        "mapping-config-v1"
    );
}

#[test]
fn source_deduplication_is_scoped_to_source_record_and_snapshot() {
    let source = SourceId::new("source-dedup").unwrap();
    let first = SourceRecordMetadata::new(
        source.clone(),
        SourceClass::structured_data(),
        "same-record",
        Some("v1".to_owned()),
        None,
        [],
    )
    .unwrap();
    let mut deduplicator = SourceDeduplicator::new();
    assert_eq!(
        deduplicator.insert(&first).unwrap(),
        DeduplicationResult::FirstSeen
    );
    assert!(matches!(
        deduplicator.insert(&first).unwrap(),
        DeduplicationResult::Duplicate { .. }
    ));

    let next_snapshot = SourceRecordMetadata::new(
        source,
        SourceClass::structured_data(),
        "same-record",
        Some("v2".to_owned()),
        None,
        [],
    )
    .unwrap();
    assert_eq!(
        deduplicator.insert(&next_snapshot).unwrap(),
        DeduplicationResult::FirstSeen
    );
    assert_eq!(deduplicator.len(), 2);
}

#[test]
fn structural_validation_returns_a_candidate_scoped_result() {
    let metadata = SourceRecordMetadata::new(
        SourceId::new("source-validation-flow").unwrap(),
        SourceClass::structured_data(),
        "record-1",
        None,
        None,
        [],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "payload".to_owned()).unwrap();
    let normalizer =
        GenericTextNormalizer::new("normalizer-v1", "config-v1", TextNormalizationMode::Trim)
            .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(11))
        .unwrap()
        .remove(0);
    let candidate = MappedCandidate::from_normalized(
        normalized,
        0,
        (),
        MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
    )
    .unwrap();

    let result = validate_candidate_structure(&candidate, &ValidationPolicy::new());
    assert_eq!(result.candidate(), candidate.key());
    assert_eq!(result.status(), ValidationStatus::Valid);
    assert!(result.is_publishable());
}

#[test]
fn ambiguous_mapping_delegates_source_scoped_resolution_to_phase3() {
    let source = SourceId::new("source-mapping-resolution").unwrap();
    let external = ExternalIdentifier::new(source.clone(), "external-person-7").unwrap();
    let metadata = SourceRecordMetadata::new(
        source.clone(),
        SourceClass::structured_data(),
        "record-7",
        Some("snapshot-1".to_owned()),
        None,
        [external.clone()],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "Amina".to_owned()).unwrap();
    let normalizer =
        GenericTextNormalizer::new("normalizer-v1", "config-v1", TextNormalizationMode::Trim)
            .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(77))
        .unwrap()
        .remove(0);
    let input = ResolutionInput::new(
        ResolutionReference::Reference(ReferenceId::new("reference-mapping-resolution").unwrap()),
        "Amina",
    )
    .with_source(source)
    .with_external_identifier(external.clone());
    let candidate = MappedCandidate::from_normalized(
        normalized,
        0,
        (),
        MappingMetadata::new("mapper-v1", "config-v1").unwrap(),
    )
    .unwrap()
    .with_resolution_input(input)
    .unwrap();
    let profiles = [
        EntityCandidateProfile::new(EntityId::new("entity-person-7").unwrap())
            .with_name("Amina")
            .with_source_identifier(external),
    ];

    let result = candidate
        .resolve_with(&Resolver::phase3_default(), &profiles)
        .expect("resolution was requested");
    assert_eq!(result.decision().state(), ResolutionState::Resolved);
    assert_eq!(
        result.resolved_entity().unwrap().as_str(),
        "entity-person-7"
    );
}
