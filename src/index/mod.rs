//! Phase 6 Knowledge Graph access boundary over Nizaam Indexing.
//!
//! This module is intentionally thin. Indexing remains authoritative for
//! physical index construction, query planning, provider selection, retrieval,
//! consistency policy, publication, and version lifecycle. The Knowledge Graph
//! exposes only the logical lookup/search adapters and factual state observations
//! required by its own query layer.

mod forward;
mod index_state;
mod reverse;
mod search;

pub use forward::{ForwardIndexAccess, lookup_forward};
pub use index_state::{
    IndexQueryability, IndexStateError, IndexStateObservation, IndexVersionObservation,
};
pub use reverse::{ReverseIndexAccess, lookup_reverse};
pub use search::{IndexSearchAccess, search};

#[cfg(test)]
mod tests {
    use super::{IndexQueryability, IndexStateObservation, IndexVersionObservation};
    use nizaam_indexing::index::reference::ObjectReference;
    use nizaam_indexing::{IndexAssignedId, TargetReferenceType};

    #[test]
    fn public_index_boundary_exposes_only_logical_observations_and_adapters() {
        let reference = ObjectReference::new("kg", "entity-1").expect("valid object reference");
        let target_type = TargetReferenceType::new("entity").expect("valid target reference type");
        let assigned_id = IndexAssignedId::generate(&target_type, &reference);
        let observation = IndexStateObservation::new(
            assigned_id.clone(),
            reference.clone(),
            IndexQueryability::Queryable,
        )
        .with_version(IndexVersionObservation::new("v1").expect("valid version"));

        assert_eq!(observation.assigned_id(), &assigned_id);
        assert_eq!(observation.object_reference(), &reference);
        assert_eq!(observation.version().unwrap().version(), "v1");
    }
}
