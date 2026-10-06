//! Level 3 public-boundary tests for the Phase 2 assertion model.
//!
//! These tests exercise `KnowledgeAssertion` through the public Knowledge
//! Graph API rather than reaching into private assertion implementation
//! details.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifier, Qualifiers,
};
use nizaam_knowledge_graph::identity::{ConceptId, EntityId, MentionId};

fn entity(value: &str) -> EntityId {
    EntityId::new(value).expect("valid entity identity")
}

fn concept(value: &str) -> ConceptId {
    ConceptId::new(value).expect("valid concept identity")
}

fn has_name() -> AssertionPredicate {
    AssertionPredicate::new("has-name").expect("valid relationship predicate")
}

fn assertion(
    subject: AssertionObject,
    predicate: AssertionPredicate,
    object: AssertionObject,
    context: AssertionContext,
    qualifiers: Qualifiers,
    status: AssertionStatus,
    polarity: AssertionPolarity,
) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        subject, predicate, object, context, qualifiers, status, polarity,
    )
}

#[test]
fn knowledge_assertion_is_a_first_class_public_semantic_object() {
    let mut context = AssertionContext::new();
    context
        .insert("source", "quran")
        .expect("valid assertion context");

    let qualifier = Qualifier::new("scope", "primary").expect("valid assertion qualifier");

    let assertion = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Concept(concept("concept-1")),
        context,
        Qualifiers::from_iter([qualifier]),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    assert!(assertion.validate().is_ok());
    assert_eq!(assertion.status(), AssertionStatus::Accepted);
    assert_eq!(assertion.polarity(), AssertionPolarity::Positive);
}

#[test]
fn typed_subject_and_object_references_support_cross_dimensional_assertions() {
    let assertion = assertion(
        AssertionObject::Mention(MentionId::new("mention-1").expect("valid mention identity")),
        AssertionPredicate::new("refers-to").expect("valid predicate"),
        AssertionObject::Entity(entity("entity-1")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Provisional,
        AssertionPolarity::Positive,
    );

    assert!(assertion.validate().is_ok());
    assert!(matches!(assertion.subject(), AssertionObject::Mention(_)));
    assert!(matches!(assertion.object(), AssertionObject::Entity(_)));
}

#[test]
fn identical_semantic_assertions_receive_the_same_knowledge_assertion_id() {
    let first = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    let second = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Provisional,
        AssertionPolarity::Positive,
    );

    assert_eq!(first.id(), second.id());
    assert_eq!(first, second);
}

#[test]
fn assertion_context_and_qualifiers_participate_in_identity() {
    let base = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    let context =
        AssertionContext::from_entries([("source", "quran")]).expect("valid assertion context");
    let qualifier = Qualifier::new("scope", "primary").expect("valid assertion qualifier");

    let enriched = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        context,
        Qualifiers::from_iter([qualifier]),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    assert_ne!(base.id(), enriched.id());
}

#[test]
fn qualifier_insertion_order_does_not_change_assertion_identity() {
    let first_qualifier = Qualifier::new("a", "one").expect("valid qualifier");
    let second_qualifier = Qualifier::new("b", "two").expect("valid qualifier");

    let first = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::from_iter([first_qualifier.clone(), second_qualifier.clone()]),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    let second = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::from_iter([second_qualifier, first_qualifier]),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    assert_eq!(first.id(), second.id());
}

#[test]
fn polarity_is_part_of_semantic_assertion_identity() {
    let positive = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    let negative = assertion(
        AssertionObject::Entity(entity("entity-1")),
        has_name(),
        AssertionObject::Entity(entity("entity-2")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Negative,
    );

    assert_ne!(positive.id(), negative.id());
    assert_ne!(positive, negative);
}

#[test]
fn semantic_types_are_preserved_even_when_identity_text_matches() {
    let entity_object = AssertionObject::Entity(entity("shared-value"));
    let concept_object = AssertionObject::Concept(concept("shared-value"));

    assert_ne!(entity_object, concept_object);
}
