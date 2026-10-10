//! Foundational source and reference boundaries with Phase 4 source authority.
//!
//! `Source` retains its Phase 1 strongly typed identity and identifying label;
//! Phase 4 adds target-checked authority metadata. Generic references remain
//! opaque, and collection, document, and passage scaffolds stay deferred.

mod model;
mod reference;

pub use crate::identity::{ReferenceId, SourceId};
pub use model::{Source, SourceError};
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

    #[test]
    fn public_source_boundary_exposes_target_checked_authority_metadata() {
        use crate::authority::{Authority, AuthorityDimension, AuthorityTarget, AuthorityValue};

        let id = SourceId::new("source-public-authority").expect("valid source identity");
        let authority = Authority::new(AuthorityTarget::Source(id.clone()))
            .with_dimension(AuthorityDimension::SourceAuthority(
                AuthorityValue::new("curated").expect("valid authority value"),
            ))
            .expect("valid authority metadata");
        let source = Source::new(id.clone(), "Curated source")
            .with_authority(authority)
            .expect("authority must point to this source");

        assert_eq!(source.id(), &id);
        assert!(source.authority().is_some());
        assert!(source.validate().is_ok());
    }
}
