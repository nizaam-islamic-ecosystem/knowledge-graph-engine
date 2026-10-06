//! Level 3 public-boundary tests for the KG identity layer.
//!
//! Phase 1 established the strongly typed semantic identities. Phase 2 adds
//! the deterministic construction path for `KnowledgeAssertionId` while
//! preserving Core ownership of the underlying identity implementation.

use nizaam_knowledge_graph::identity::{
    ConceptId, EntityId, KnowledgeAssertionId, LexicalFormId, MentionId, ReferenceId, SourceId,
};
use std::any::TypeId;

#[test]
fn all_kg_identity_types_generate_non_empty_values() {
    assert!(!EntityId::generate().as_str().is_empty());
    assert!(!ConceptId::generate().as_str().is_empty());
    assert!(!SourceId::generate().as_str().is_empty());
    assert!(!ReferenceId::generate().as_str().is_empty());
    assert!(!LexicalFormId::generate().as_str().is_empty());
    assert!(!MentionId::generate().as_str().is_empty());
    assert!(!KnowledgeAssertionId::generate().as_str().is_empty());
}

#[test]
fn generated_identity_values_remain_distinct_within_each_type() {
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
fn identity_types_are_strongly_distinct() {
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
fn explicit_core_identity_values_preserve_value_semantics() {
    let first = EntityId::new("entity-1").expect("valid identity");
    let second = EntityId::new("entity-1").expect("valid identity");

    assert_eq!(first, second);
    assert_eq!(first.as_str(), "entity-1");
}

#[test]
fn different_identity_types_do_not_collapse_shared_text_values() {
    let entity = EntityId::new("shared-value").expect("valid identity");
    let concept = ConceptId::new("shared-value").expect("valid identity");

    assert_eq!(entity.as_str(), concept.as_str());
    assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<ConceptId>());
}

#[test]
fn knowledge_assertion_identity_is_deterministic_from_canonical_input() {
    let canonical = "subject:9:entity:1|predicate:25:kg.relationship.has-name|object:10:entity:2|context:0:|qualifiers:0:|polarity:8:positive|";

    let first = KnowledgeAssertionId::from_canonical(canonical)
        .expect("valid canonical assertion identity");
    let second = KnowledgeAssertionId::from_canonical(canonical)
        .expect("same canonical assertion identity should be valid");

    assert_eq!(first, second);
    assert_eq!(first.as_str(), canonical);
}

#[test]
fn different_canonical_assertion_inputs_produce_distinct_identities() {
    let first = KnowledgeAssertionId::from_canonical(
        "subject:9:entity:1|predicate:25:kg.relationship.has-name|object:10:entity:2|context:0:|qualifiers:0:|polarity:8:positive|",
    )
    .expect("valid canonical assertion identity");

    let second = KnowledgeAssertionId::from_canonical(
        "subject:9:entity:1|predicate:25:kg.relationship.has-name|object:10:entity:3|context:0:|qualifiers:0:|polarity:8:positive|",
    )
    .expect("valid canonical assertion identity");

    assert_ne!(first, second);
}
