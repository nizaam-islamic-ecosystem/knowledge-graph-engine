//! Level 3 Phase 6 Indexing-boundary tests.
//!
//! These tests verify that the KG index module remains a thin logical boundary
//! over Indexing-owned identities and caller-provided adapters.

use nizaam_indexing::index::reference::ObjectReference;
use nizaam_indexing::{IndexAssignedId, TargetReferenceType};
use nizaam_knowledge_graph::index::{
    ForwardIndexAccess, IndexQueryability, IndexStateObservation, IndexVersionObservation,
    ReverseIndexAccess, lookup_forward, lookup_reverse,
};

#[derive(Debug)]
struct AdapterError;

impl std::fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("test adapter error")
    }
}

impl std::error::Error for AdapterError {}

struct ForwardAdapter {
    expected: ObjectReference,
    assigned: IndexAssignedId,
}

impl ForwardIndexAccess for ForwardAdapter {
    type Error = AdapterError;

    fn lookup_forward(
        &self,
        object_reference: &ObjectReference,
    ) -> Result<Option<IndexAssignedId>, Self::Error> {
        Ok((object_reference == &self.expected).then(|| self.assigned.clone()))
    }
}

struct ReverseAdapter {
    expected: IndexAssignedId,
    reference: ObjectReference,
}

impl ReverseIndexAccess for ReverseAdapter {
    type Error = AdapterError;

    fn lookup_reverse(
        &self,
        assigned_id: &IndexAssignedId,
    ) -> Result<Option<ObjectReference>, Self::Error> {
        Ok((assigned_id == &self.expected).then(|| self.reference.clone()))
    }
}

fn reference() -> ObjectReference {
    ObjectReference::new("phase6-source", "record-1").expect("valid object reference")
}

fn assigned_id(reference: &ObjectReference) -> IndexAssignedId {
    let target = TargetReferenceType::new("entity").expect("valid target type");
    IndexAssignedId::generate(&target, reference)
}

#[test]
fn index_state_observation_remains_factual_and_opaque() {
    let reference = reference();
    let assigned = assigned_id(&reference);
    let observation = IndexStateObservation::new(
        assigned.clone(),
        reference.clone(),
        IndexQueryability::Stale,
    )
    .with_version(
        IndexVersionObservation::new("index-version-2")
            .expect("valid version")
            .with_source_version("source-version-7")
            .with_schema_version("schema-version-3"),
    )
    .with_source_update_sequence(7)
    .with_indexed_update_sequence(6);

    assert_eq!(observation.assigned_id(), &assigned);
    assert_eq!(observation.object_reference(), &reference);
    assert!(observation.queryability().is_queryable());
    assert_eq!(
        observation
            .version()
            .expect("version should exist")
            .version(),
        "index-version-2"
    );
    assert_eq!(observation.source_update_sequence(), Some(7));
    assert_eq!(observation.indexed_update_sequence(), Some(6));
}

#[test]
fn forward_and_reverse_adapters_preserve_indexing_identity_without_owning_storage() {
    let reference = reference();
    let assigned = assigned_id(&reference);

    let forward = ForwardAdapter {
        expected: reference.clone(),
        assigned: assigned.clone(),
    };
    let reverse = ReverseAdapter {
        expected: assigned.clone(),
        reference: reference.clone(),
    };

    assert_eq!(
        lookup_forward(&forward, &reference).expect("forward lookup should succeed"),
        Some(assigned.clone())
    );
    assert_eq!(
        lookup_reverse(&reverse, &assigned).expect("reverse lookup should succeed"),
        Some(reference.clone())
    );
    assert_eq!(
        lookup_forward(
            &forward,
            &ObjectReference::new("phase6-source", "missing").unwrap()
        )
        .expect("missing lookup should be representable"),
        None
    );
}

#[test]
fn stale_is_queryable_but_unavailable_and_rebuilding_are_not() {
    assert!(IndexQueryability::Queryable.is_queryable());
    assert!(IndexQueryability::Stale.is_queryable());
    assert!(!IndexQueryability::Unavailable.is_queryable());
    assert!(!IndexQueryability::Rebuilding.is_queryable());
}
