//! Foundational knowledge-graph source boundaries for Phase 1.
//!
//! Phase 1 exposes only the minimal [`Source`] and generic [`Reference`]
//! representations together with their strongly typed identities. Collection,
//! document, and passage scaffolds remain deferred and are intentionally not
//! wired into the Phase 1 public API.

mod model;
mod reference;

pub use crate::identity::{ReferenceId, SourceId};
pub use model::Source;
pub use reference::Reference;

#[cfg(test)]
mod tests {
    use super::{Reference, ReferenceId, Source, SourceId};
    use std::any::TypeId;

    #[test]
    fn source_module_exposes_the_phase1_source_boundary() {
        let source_id = SourceId::generate();
        let reference_id = ReferenceId::generate();

        let source = Source::new(source_id.clone(), "Example source");
        let reference = Reference::new(reference_id.clone(), "opaque-reference-1");

        assert_eq!(source.id(), &source_id);
        assert_eq!(source.label(), "Example source");
        assert_eq!(reference.id(), &reference_id);
        assert_eq!(reference.value(), "opaque-reference-1");
    }

    #[test]
    fn source_and_reference_remain_strongly_distinct_types() {
        assert_ne!(TypeId::of::<Source>(), TypeId::of::<Reference>());
        assert_ne!(TypeId::of::<SourceId>(), TypeId::of::<ReferenceId>());
    }

    #[test]
    fn source_reference_composition_remains_generic() {
        let source = Source::new(SourceId::generate(), "Quran");
        let reference = Reference::new(
            ReferenceId::generate(),
            "opaque:collection/document/passage",
        );

        assert_eq!(source.label(), "Quran");
        assert_eq!(reference.value(), "opaque:collection/document/passage");
    }

    #[test]
    fn source_module_does_not_require_document_collection_or_passage_objects() {
        let source = Source::new(SourceId::generate(), "Minimal source");

        assert_eq!(source.label(), "Minimal source");
    }
}
