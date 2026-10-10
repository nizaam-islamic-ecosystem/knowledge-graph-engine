//! Level 3 tests for explicit targeted, source-delta and full-source reprocessing.

use nizaam_core::identity::OperationId;
use nizaam_knowledge_graph::identity::SourceId;
use nizaam_knowledge_graph::ingestion::{
    CandidateKey, PipelineError, ReprocessingMode, ReprocessingRequest, SourceDelta,
};
use nizaam_knowledge_graph::temporal::Instant;

fn key(source: &str, record: &str, version: Option<&str>) -> CandidateKey {
    CandidateKey::new(
        SourceId::new(source).unwrap(),
        record,
        version.map(str::to_owned),
        0,
    )
    .unwrap()
}

#[test]
fn targeted_reprocessing_is_scoped_to_source_and_snapshot() {
    let source = SourceId::new("source-reprocess").unwrap();
    let target = key("source-reprocess", "record-2", Some("snapshot-2"));
    let request = ReprocessingRequest::new(
        OperationId::new("nizaam.kg.reprocess.targeted").unwrap(),
        source.clone(),
        Some("snapshot-2".to_owned()),
        ReprocessingMode::TargetedCandidates([target.clone()].into_iter().collect()),
        Instant::from_unix_seconds(10),
        "retry quarantined candidate",
    )
    .unwrap();
    assert_eq!(request.source_id(), &source);
    assert!(
        matches!(request.mode(), ReprocessingMode::TargetedCandidates(keys) if keys.contains(&target))
    );

    let mismatched = ReprocessingRequest::new(
        OperationId::new("nizaam.kg.reprocess.wrong-target").unwrap(),
        source,
        Some("snapshot-2".to_owned()),
        ReprocessingMode::TargetedCandidates(
            [key("other-source", "record-2", Some("snapshot-2"))]
                .into_iter()
                .collect(),
        ),
        Instant::from_unix_seconds(11),
        "invalid target",
    );
    assert_eq!(mismatched, Err(PipelineError::ReprocessingTargetMismatch));
}

#[test]
fn source_delta_requires_different_versions_and_changed_keys() {
    let delta = SourceDelta::new(
        "snapshot-1",
        "snapshot-2",
        ["record-a".to_owned(), "record-b".to_owned()],
    )
    .unwrap();
    assert_eq!(delta.previous_version(), "snapshot-1");
    assert_eq!(delta.current_version(), "snapshot-2");
    assert_eq!(delta.changed_record_keys().len(), 2);

    assert_eq!(
        SourceDelta::new("same", "same", ["record-a".to_owned()]),
        Err(PipelineError::UnchangedSourceVersion)
    );
    assert_eq!(
        SourceDelta::new("v1", "v2", Vec::<String>::new()),
        Err(PipelineError::EmptySourceDelta)
    );
    assert!(SourceDelta::new("v1", "v2", [" ".to_owned()]).is_err());
}

#[test]
fn source_delta_request_must_target_its_current_snapshot() {
    let delta =
        SourceDelta::new("snapshot-1", "snapshot-2", ["changed-record".to_owned()]).unwrap();
    let wrong_snapshot = ReprocessingRequest::new(
        OperationId::new("nizaam.kg.reprocess.delta-wrong-version").unwrap(),
        SourceId::new("source-reprocess-delta").unwrap(),
        Some("snapshot-1".to_owned()),
        ReprocessingMode::SourceDelta(delta.clone()),
        Instant::from_unix_seconds(20),
        "delta mismatch",
    );
    assert_eq!(
        wrong_snapshot,
        Err(PipelineError::ReprocessingTargetMismatch)
    );

    let accepted = ReprocessingRequest::new(
        OperationId::new("nizaam.kg.reprocess.delta-valid").unwrap(),
        SourceId::new("source-reprocess-delta").unwrap(),
        Some("snapshot-2".to_owned()),
        ReprocessingMode::SourceDelta(delta),
        Instant::from_unix_seconds(21),
        "apply source delta",
    )
    .unwrap();
    assert!(matches!(accepted.mode(), ReprocessingMode::SourceDelta(_)));
}

#[test]
fn full_source_reprocessing_is_explicit_not_an_empty_delta() {
    let request = ReprocessingRequest::new(
        OperationId::new("nizaam.kg.reprocess.full").unwrap(),
        SourceId::new("source-reprocess-full").unwrap(),
        Some("snapshot-8".to_owned()),
        ReprocessingMode::FullSource,
        Instant::from_unix_seconds(30),
        "source schema changed",
    )
    .unwrap();
    assert!(matches!(request.mode(), ReprocessingMode::FullSource));
}
