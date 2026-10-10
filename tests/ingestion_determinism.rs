//! Level 3 tests for deterministic normalization, candidate keys and pipeline plans.

use nizaam_knowledge_graph::identity::SourceId;
use nizaam_knowledge_graph::ingestion::{
    CandidateKey, GenericTextNormalizer, MappedCandidate, MappingMetadata, NormalizationStage,
    PipelinePlan, SourceClass, SourceRecord, SourceRecordMetadata, TextNormalizationMode,
};
use nizaam_knowledge_graph::temporal::Instant;

fn candidate() -> MappedCandidate<String> {
    let metadata = SourceRecordMetadata::new(
        SourceId::new("source-determinism").unwrap(),
        SourceClass::structured_data(),
        "record-42",
        Some("snapshot-3".to_owned()),
        None,
        [],
    )
    .unwrap();
    let record = SourceRecord::new(metadata, "  one\t two  ".to_owned()).unwrap();
    let normalizer = GenericTextNormalizer::new(
        "normalizer-v1",
        "collapse-whitespace-v2",
        TextNormalizationMode::CollapseWhitespace,
    )
    .unwrap();
    let normalized = normalizer
        .normalize(record, Instant::from_unix_seconds(100))
        .unwrap()
        .remove(0);
    MappedCandidate::from_normalized(
        normalized,
        2,
        "same-semantic-output".to_owned(),
        MappingMetadata::new("mapper-v4", "mapping-config-v1").unwrap(),
    )
    .unwrap()
}

#[test]
fn fixed_source_snapshot_versions_and_configuration_produce_equal_candidates() {
    let first = candidate();
    let second = candidate();
    assert_eq!(first.key(), second.key());
    assert_eq!(first.payload(), second.payload());
    assert_eq!(first.source(), second.source());
    assert_eq!(first.normalization(), second.normalization());
    assert_eq!(first.mapping(), second.mapping());
    assert_eq!(first.source().source_record_key(), "record-42");
}

#[test]
fn candidate_key_changes_only_when_source_identity_version_or_output_ordinal_changes() {
    let base = CandidateKey::new(
        SourceId::new("source-determinism-key").unwrap(),
        "record-1",
        Some("v1".to_owned()),
        0,
    )
    .unwrap();
    let same = CandidateKey::new(
        SourceId::new("source-determinism-key").unwrap(),
        "record-1",
        Some("v1".to_owned()),
        0,
    )
    .unwrap();
    let different_version = CandidateKey::new(
        SourceId::new("source-determinism-key").unwrap(),
        "record-1",
        Some("v2".to_owned()),
        0,
    )
    .unwrap();
    let different_ordinal = CandidateKey::new(
        SourceId::new("source-determinism-key").unwrap(),
        "record-1",
        Some("v1".to_owned()),
        1,
    )
    .unwrap();

    assert_eq!(base, same);
    assert_ne!(base, different_version);
    assert_ne!(base, different_ordinal);
}

#[test]
fn default_pipeline_plan_is_stable_and_hook_order_is_deterministic() {
    let first = PipelinePlan::phase5_default();
    let second = PipelinePlan::phase5_default();
    assert_eq!(first, second);
    let names_a = first
        .ordered_steps()
        .iter()
        .map(|step| step.name().to_owned())
        .collect::<Vec<_>>();
    let names_b = second
        .ordered_steps()
        .iter()
        .map(|step| step.name().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names_a, names_b);
    assert_eq!(names_a.first().map(String::as_str), Some("raw-capture"));
    assert_eq!(
        names_a.last().map(String::as_str),
        Some("index-synchronization")
    );
}
