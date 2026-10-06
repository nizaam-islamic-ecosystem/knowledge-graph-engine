//! Level 3 public-boundary tests for the Phase 1 Source module.

use nizaam_knowledge_graph::{
    identity::{ReferenceId, SourceId},
    source::{Reference, Source},
};
use std::any::TypeId;

#[test]
fn source_public_api_preserves_identity_and_minimal_metadata() {
    let id = SourceId::generate();
    let source = Source::new(id.clone(), "Example source");

    assert_eq!(source.id(), &id);
    assert_eq!(source.label(), "Example source");
}

#[test]
fn source_supports_opaque_multilingual_identifying_metadata() {
    let arabic = Source::new(SourceId::generate(), "مصدر");
    let english = Source::new(SourceId::generate(), "Source");
    let urdu = Source::new(SourceId::generate(), "ماخذ");

    assert_eq!(arabic.label(), "مصدر");
    assert_eq!(english.label(), "Source");
    assert_eq!(urdu.label(), "ماخذ");
}

#[test]
fn reference_public_api_preserves_identity_and_opaque_value() {
    let id = ReferenceId::generate();
    let reference = Reference::new(id.clone(), "opaque-reference");

    assert_eq!(reference.id(), &id);
    assert_eq!(reference.value(), "opaque-reference");
}

#[test]
fn source_and_reference_remain_distinct_semantic_objects() {
    assert_ne!(TypeId::of::<Source>(), TypeId::of::<Reference>());
    assert_ne!(TypeId::of::<SourceId>(), TypeId::of::<ReferenceId>());
}

#[test]
fn reference_value_remains_generic_in_phase1() {
    let reference = Reference::new(
        ReferenceId::generate(),
        "quran://future-domain-specific-resolution",
    );

    assert_eq!(
        reference.value(),
        "quran://future-domain-specific-resolution"
    );
}
