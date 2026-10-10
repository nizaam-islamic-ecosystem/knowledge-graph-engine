//! Level 3 integration tests for knowledge provenance, origins, lineage, and audit.

use nizaam_knowledge_graph::identity::{
    ActivityId, AgentId, ConceptId, EntityId, KnowledgeAssertionId, SourceId,
};
use nizaam_knowledge_graph::provenance::{
    Activity, ActivityKind, Agent, AgentType, AuditAction, AuditRecord, AuditTrail,
    KnowledgeOrigin, Lineage, LineageKind, LineageLink, OperationId, ProvenanceHistory,
    ProvenanceRecord, ProvenanceTarget,
};
use nizaam_knowledge_graph::temporal::Instant;

#[test]
fn provenance_links_source_activity_agent_and_canonical_target() {
    let source_id = SourceId::new("source-provenance").unwrap();
    let target = ProvenanceTarget::Entity(EntityId::new("entity-provenance").unwrap());
    let agent = Agent::new(
        AgentId::new("agent-extractor").unwrap(),
        AgentType::Pipeline,
        "quran-import-pipeline",
    )
    .expect("valid agent");
    let started = Instant::from_unix_seconds(100);
    let ended = Instant::from_unix_seconds(120);
    let activity = Activity::new(
        ActivityId::new("activity-extraction").unwrap(),
        ActivityKind::Extraction,
    )
    .expect("valid activity")
    .with_agent(agent.id().clone())
    .with_started_at(started)
    .expect("valid start time")
    .with_ended_at(ended)
    .expect("valid end time")
    .with_description("Extracted a canonical entity mention")
    .expect("valid activity description");
    let origin = KnowledgeOrigin::source_version(source_id.clone(), "dataset-v2")
        .expect("valid source-version origin");
    let recorded_at = Instant::from_unix_seconds(130);
    let record = ProvenanceRecord::new(target.clone(), activity, recorded_at, Some(origin.clone()))
        .expect("valid provenance record");

    let mut history = ProvenanceHistory::new(target.clone()).expect("valid history target");
    history
        .append(record)
        .expect("append first provenance record");

    assert_eq!(history.target(), &target);
    assert_eq!(history.len(), 1);
    assert_eq!(history.records()[0].recorded_at(), recorded_at);
    assert_eq!(history.records()[0].origin(), Some(&origin));
    assert_eq!(history.records()[0].activity().agents().len(), 1);
    assert!(
        history.records()[0]
            .activity()
            .agents()
            .contains(agent.id())
    );
    assert_eq!(origin.source_id(), Some(&source_id));
    assert_eq!(origin.source_version_label(), Some("dataset-v2"));
}

#[test]
fn provenance_history_appends_records_without_overwriting_prior_history() {
    let target = ProvenanceTarget::Concept(ConceptId::new("concept-provenance-history").unwrap());
    let first_activity = Activity::new(
        ActivityId::new("activity-provenance-first").unwrap(),
        ActivityKind::Import,
    )
    .expect("valid first activity");
    let second_activity = Activity::new(
        ActivityId::new("activity-provenance-second").unwrap(),
        ActivityKind::Review,
    )
    .expect("valid second activity");
    let first = ProvenanceRecord::new(
        target.clone(),
        first_activity,
        Instant::from_unix_seconds(100),
        Some(KnowledgeOrigin::unknown()),
    )
    .expect("valid first record");
    let second = ProvenanceRecord::new(
        target.clone(),
        second_activity,
        Instant::from_unix_seconds(200),
        None,
    )
    .expect("valid second record");

    let mut history = ProvenanceHistory::new(target.clone()).unwrap();
    history.append(first.clone()).unwrap();
    history.append(second).unwrap();

    assert_eq!(history.len(), 2);
    assert_eq!(history.records()[0], first);
    assert_eq!(
        history.records()[0].recorded_at(),
        Instant::from_unix_seconds(100)
    );
    assert_eq!(
        history.records()[1].recorded_at(),
        Instant::from_unix_seconds(200)
    );
    assert!(
        history.append(first).is_err(),
        "an activity must not overwrite a prior record"
    );

    let mismatch = ProvenanceRecord::new(
        ProvenanceTarget::Assertion(KnowledgeAssertionId::new("different-target").unwrap()),
        Activity::new(
            ActivityId::new("activity-wrong-target").unwrap(),
            ActivityKind::Modification,
        )
        .unwrap(),
        Instant::from_unix_seconds(300),
        None,
    )
    .unwrap();
    assert!(
        history.append(mismatch).is_err(),
        "history is scoped to one target"
    );
}

#[test]
fn lineage_is_directed_deduplicated_and_distinct_from_audit() {
    let source = ProvenanceTarget::Source(SourceId::new("source-lineage").unwrap());
    let entity = ProvenanceTarget::Entity(EntityId::new("entity-derived").unwrap());
    let activity_id = ActivityId::new("activity-lineage").unwrap();
    let recorded_at = Instant::from_unix_seconds(400);
    let link = LineageLink::new(source.clone(), entity.clone(), LineageKind::DerivedFrom)
        .expect("valid lineage link")
        .with_activity(activity_id.clone())
        .with_recorded_at(recorded_at)
        .with_note("Entity was derived from source material")
        .expect("valid lineage note");
    let mut lineage = Lineage::new();

    assert!(lineage.append(link.clone()).expect("first append succeeds"));
    assert!(
        !lineage
            .append(link.clone())
            .expect("duplicate append is idempotent")
    );
    assert_eq!(lineage.len(), 1);
    assert_eq!(lineage.links()[0].from(), &source);
    assert_eq!(lineage.links()[0].to(), &entity);
    assert_eq!(lineage.links()[0].activity_id(), Some(&activity_id));
    assert_eq!(lineage.links()[0].recorded_at(), Some(recorded_at));

    assert!(LineageLink::new(source.clone(), source, LineageKind::DerivedFrom).is_err());

    let operation_id = OperationId::new("nizaam.kg.phase4.audit-lineage-test")
        .expect("valid Core operation identity");
    let audit = AuditRecord::new(
        operation_id.clone(),
        entity,
        AuditAction::Updated,
        recorded_at,
        "Updated metadata after review",
    )
    .expect("valid audit record")
    .with_actor(AgentId::new("agent-auditor").unwrap())
    .with_changed_fields([String::from("status"), String::from("authority")])
    .expect("valid changed field names");
    let mut audit_trail = AuditTrail::new();

    assert!(audit_trail.append(audit.clone()));
    assert!(!audit_trail.append(audit));
    assert_eq!(audit_trail.len(), 1);
    assert_eq!(audit_trail.records()[0].operation_id(), &operation_id);
    assert_eq!(audit_trail.records()[0].changed_fields().len(), 2);
    assert_ne!(
        std::any::TypeId::of::<Lineage>(),
        std::any::TypeId::of::<AuditTrail>()
    );
}
