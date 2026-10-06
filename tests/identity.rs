//! Level 3 public-boundary tests for the Phase 1 KG identity layer.
//!
//! These tests exercise the public identity API exposed by the Knowledge Graph
//! crate. Identity generation remains owned by Nizaam Core through the
//! `identity!` mechanism; the KG only owns the semantic identity types.

use nizaam_knowledge_graph::identity::{
    ConceptId, EntityId, KnowledgeAssertionId, LexicalFormId, MentionId, ReferenceId, SourceId,
};
use std::any::TypeId;

#[test]
fn all_phase1_identity_types_generate_non_empty_values() {
    assert!(!EntityId::generate().as_str().is_empty());
    assert!(!ConceptId::generate().as_str().is_empty());
    assert!(!SourceId::generate().as_str().is_empty());
    assert!(!ReferenceId::generate().as_str().is_empty());
    assert!(!LexicalFormId::generate().as_str().is_empty());
    assert!(!MentionId::generate().as_str().is_empty());
    assert!(!KnowledgeAssertionId::generate().as_str().is_empty());
}

#[test]
fn phase1_identity_generation_produces_distinct_values_per_type() {
    assert_ne!(EntityId::generate(), EntityId::generate());
    assert_ne!(ConceptId::generate(), ConceptId::generate());
    assert_ne!(SourceId::generate(), SourceId::generate());
    assert_ne!(ReferenceId::generate(), ReferenceId::generate());
    assert_ne!(LexicalFormId::generate(), LexicalFormId::generate());
    assert_ne!(MentionId::generate(), MentionId::generate());
    assert_ne!(
        KnowledgeAssertionId::generate(),
        KnowledgeAssertionId::generate()
    );
}

#[test]
fn phase1_identity_types_are_strongly_distinct() {
    let types = [
        TypeId::of::<EntityId>(),
        TypeId::of::<ConceptId>(),
        TypeId::of::<SourceId>(),
        TypeId::of::<ReferenceId>(),
        TypeId::of::<LexicalFormId>(),
        TypeId::of::<MentionId>(),
        TypeId::of::<KnowledgeAssertionId>(),
    ];

    for (index, left) in types.iter().enumerate() {
        for right in types.iter().skip(index + 1) {
            assert_ne!(left, right);
        }
    }
}

#[test]
fn phase1_identity_explicit_values_preserve_core_value_semantics() {
    let first = EntityId::new("entity-1").expect("valid identity");
    let second = EntityId::new("entity-1").expect("valid identity");

    assert_eq!(first, second);
    assert_eq!(first.as_str(), "entity-1");
}

#[test]
fn phase1_identity_types_do_not_require_shared_textual_equality() {
    let entity = EntityId::new("shared-value").expect("valid identity");
    let concept = ConceptId::new("shared-value").expect("valid identity");

    assert_eq!(entity.as_str(), concept.as_str());
    assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<ConceptId>());
}
