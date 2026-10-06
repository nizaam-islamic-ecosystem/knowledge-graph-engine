//! Level 3 public-boundary tests for the Phase 2 relationship model.
//!
//! These tests verify relationship mechanics without exercising ontology
//! validation or reasoning.

use nizaam_knowledge_graph::relationship::{
    CompositionRule, InverseRelationship, Relationship, RelationshipCharacteristic,
    RelationshipCharacteristics, RelationshipDirection, RelationshipFamily, RelationshipPredicate,
    RelationshipVocabulary, RelationshipVocabularyError,
};

fn predicate(name: &str) -> RelationshipPredicate {
    RelationshipPredicate::new(name).expect("valid relationship predicate")
}

fn semantic_family() -> RelationshipFamily {
    RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid relationship family")
}

fn empty_characteristics() -> RelationshipCharacteristics {
    RelationshipCharacteristics::new()
}

fn relationship(name: &str) -> Relationship {
    Relationship::new(
        predicate(name),
        semantic_family(),
        RelationshipDirection::SubjectToObject,
        empty_characteristics(),
    )
}

#[test]
fn relationship_predicates_have_the_canonical_namespace() {
    let predicate = predicate("has-name");

    assert_eq!(predicate.as_str(), "kg.relationship.has-name");
    assert_eq!(predicate.name(), "has-name");
}

#[test]
fn relationship_definition_preserves_predicate_family_and_direction() {
    let relationship = relationship("has-name");

    assert_eq!(
        relationship.predicate().as_str(),
        "kg.relationship.has-name"
    );
    assert_eq!(relationship.family().as_str(), RelationshipFamily::SEMANTIC);
    assert_eq!(
        relationship.direction(),
        RelationshipDirection::SubjectToObject
    );
}

#[test]
fn explicit_inverse_is_attached_to_the_canonical_relationship() {
    let has_name = predicate("has-name");
    let name_of = predicate("name-of");

    let inverse = InverseRelationship::new(has_name.clone(), name_of.clone())
        .expect("valid inverse declaration");

    let relationship = Relationship::new(
        has_name.clone(),
        semantic_family(),
        RelationshipDirection::SubjectToObject,
        empty_characteristics(),
    )
    .with_inverse(inverse)
    .expect("inverse should attach");

    let stored = relationship.inverse().expect("inverse should be present");

    assert_eq!(stored.predicate(), &has_name);
    assert_eq!(stored.inverse_predicate(), &name_of);
}

#[test]
fn symmetric_relationships_expose_structural_reverse_behavior() {
    let characteristics =
        RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
            .expect("valid characteristics");

    let relationship = Relationship::new(
        predicate("aliases"),
        semantic_family(),
        RelationshipDirection::SubjectToObject,
        characteristics,
    );

    assert!(relationship.is_symmetric());
    assert!(
        relationship
            .characteristics()
            .supports_structural_reverse_view()
    );
    assert!(relationship.inverse().is_none());
}

#[test]
fn composition_is_stored_as_a_declaration_without_execution() {
    let first = predicate("part-of");
    let second = predicate("part-of");
    let result = predicate("part-of");

    let rule = CompositionRule::new(first.clone(), second.clone(), result.clone());

    let relationship = Relationship::new(
        predicate("part-of"),
        RelationshipFamily::new(RelationshipFamily::PART_WHOLE).expect("valid family"),
        RelationshipDirection::SubjectToObject,
        empty_characteristics(),
    )
    .with_composition_rule(rule);

    let stored = relationship
        .composition_rules()
        .next()
        .expect("composition rule should be stored");

    assert_eq!(stored.first(), &first);
    assert_eq!(stored.second(), &second);
    assert_eq!(stored.result(), &result);
    assert_eq!(relationship.composition_rule_count(), 1);
}

#[test]
fn relationship_vocabulary_registers_and_resolves_relationships() {
    let definition = relationship("has-name");
    let predicate = definition.predicate().clone();
    let mut vocabulary = RelationshipVocabulary::new();

    vocabulary
        .register(definition)
        .expect("relationship should register");

    assert_eq!(vocabulary.len(), 1);
    assert!(vocabulary.contains(&predicate));
    assert_eq!(
        vocabulary
            .get(&predicate)
            .expect("relationship should resolve")
            .predicate(),
        &predicate
    );
}

#[test]
fn relationship_vocabulary_rejects_duplicate_predicates() {
    let definition = relationship("has-name");
    let duplicate = definition.clone();
    let mut vocabulary = RelationshipVocabulary::new();

    vocabulary
        .register(definition)
        .expect("first relationship should register");

    let error = vocabulary
        .register(duplicate)
        .expect_err("duplicate predicate must be rejected");

    assert_eq!(
        error,
        RelationshipVocabularyError::PredicateAlreadyRegistered {
            predicate: predicate("has-name"),
        }
    );
}
