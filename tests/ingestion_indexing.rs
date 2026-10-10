//! Level 3 contract tests for the Knowledge Graph-to-Indexing boundary.

use nizaam_core::contracts::{UniversalRequest, UniversalResponse};
use nizaam_indexing::index::reference::ObjectReference;
use nizaam_indexing::{IndexAssignedId, TargetReferenceType};
use nizaam_knowledge_graph::integration::indexing::{
    CoreIndexingRequest, CoreIndexingResponse, IndexingIntegrationError,
    IndexingPublicationReadiness, IndexingReadinessBlocker, IndexingReadinessReceipt,
    IndexingSynchronizationRecord, IndexingSynchronizationStatus, TypedIndexingEvent,
    TypedIndexingEventResponse,
};
use std::any::TypeId;

fn assigned(reference: &ObjectReference) -> IndexAssignedId {
    let target = TargetReferenceType::new("source-object").unwrap();
    IndexAssignedId::generate(&target, reference)
}

#[test]
fn core_envelopes_remain_aliases_and_do_not_create_a_second_transport_contract() {
    assert_eq!(
        TypeId::of::<CoreIndexingRequest>(),
        TypeId::of::<UniversalRequest>()
    );
    assert_eq!(
        TypeId::of::<CoreIndexingResponse>(),
        TypeId::of::<UniversalResponse>()
    );
    assert_eq!(
        TypeId::of::<TypedIndexingEvent>(),
        TypeId::of::<nizaam_indexing::event::index_event::IndexEvent>()
    );
    assert_eq!(
        TypeId::of::<TypedIndexingEventResponse>(),
        TypeId::of::<nizaam_indexing::event::index_event_response::IndexEventResponse>()
    );
}

#[test]
fn readiness_gate_fails_closed_and_returns_the_external_assigned_id() {
    let reference = ObjectReference::new("source-system", "record-42").unwrap();
    let assigned_id = assigned(&reference);
    let ready = IndexingReadinessReceipt::new(
        assigned_id.clone(),
        reference.clone(),
        IndexingPublicationReadiness::Ready,
    );
    assert_eq!(ready.require_ready().unwrap(), &assigned_id);
    assert_eq!(ready.assigned_id(), &assigned_id);

    let blocked = IndexingReadinessReceipt::new(
        assigned(&reference),
        reference.clone(),
        IndexingPublicationReadiness::Blocked(IndexingReadinessBlocker::IntegrityFailure),
    );
    assert_eq!(
        blocked.require_ready(),
        Err(IndexingIntegrationError::PublicationNotReady)
    );

    let unknown = IndexingReadinessReceipt::new(
        assigned(&reference),
        reference,
        IndexingPublicationReadiness::Unknown,
    );
    assert_eq!(
        unknown.require_ready(),
        Err(IndexingIntegrationError::ReadinessUnknown)
    );
}

#[test]
fn synchronization_records_keep_assigned_identity_separate_from_internal_index_identity() {
    let reference = ObjectReference::new("source-system", "record-99").unwrap();
    let assigned_id = assigned(&reference);
    let pending = IndexingSynchronizationRecord::new(
        assigned_id.clone(),
        reference.clone(),
        IndexingSynchronizationStatus::Pending,
    );
    assert_eq!(pending.assigned_id(), &assigned_id);
    assert_eq!(pending.object_reference(), &reference);
    assert_eq!(pending.status(), IndexingSynchronizationStatus::Pending);
    assert!(!pending.status().is_synchronized());
    assert!(pending.status().requires_retry());

    let synced = IndexingSynchronizationRecord::new(
        assigned_id.clone(),
        reference.clone(),
        IndexingSynchronizationStatus::Synchronized,
    );
    assert!(synced.status().is_synchronized());
    let terminal = IndexingSynchronizationRecord::new(
        assigned_id,
        reference,
        IndexingSynchronizationStatus::TerminalFailure,
    );
    assert!(!terminal.status().requires_retry());
}
