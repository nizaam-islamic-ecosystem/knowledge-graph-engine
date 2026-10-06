//! Phase 2 relationship model boundary.
//!
//! This module exposes the complete public relationship infrastructure for
//! Phase 2:
//!
//! - strongly typed relationship predicates;
//! - semantic relationship direction;
//! - relationship characteristics;
//! - explicit inverse declarations;
//! - relationship families;
//! - relationship definitions;
//! - deferred composition declarations;
//! - the deterministic in-memory relationship vocabulary.
//!
//! The relationship module does not implement ontology validation, query
//! traversal, persistence, or reasoning. Composition is represented only;
//! later reasoning phases are responsible for executing composition.

mod characteristic;
mod direction;
mod inverse;
mod model;
mod predicate;

pub use characteristic::{
    RelationshipCharacteristic, RelationshipCharacteristicError, RelationshipCharacteristics,
};

pub use direction::RelationshipDirection;

pub use inverse::{InverseRelationship, InverseRelationshipError};

pub use model::{
    CompositionRule, Relationship, RelationshipError, RelationshipFamily, RelationshipFamilyError,
    RelationshipVocabulary, RelationshipVocabularyError,
};

pub use predicate::{
    RELATIONSHIP_NAMESPACE, RelationshipPredicate, RelationshipPredicateValidationError,
};

#[cfg(test)]
mod tests {
    use super::{
        CompositionRule, InverseRelationship, Relationship, RelationshipCharacteristic,
        RelationshipCharacteristics, RelationshipDirection, RelationshipFamily,
        RelationshipPredicate, RelationshipVocabulary, RelationshipVocabularyError,
    };

    fn predicate(name: &str) -> RelationshipPredicate {
        RelationshipPredicate::new(name).expect("valid relationship predicate")
    }

    fn family(name: &str) -> RelationshipFamily {
        RelationshipFamily::new(name).expect("valid relationship family")
    }

    #[test]
    fn public_relationship_boundary_composes_the_complete_model() {
        let characteristics =
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Functional])
                .expect("valid relationship characteristics");

        let relationship = Relationship::new(
            predicate("has-name"),
            family(RelationshipFamily::IDENTITY),
            RelationshipDirection::SubjectToObject,
            characteristics,
        );

        assert_eq!(
            relationship.predicate().as_str(),
            "kg.relationship.has-name"
        );
        assert_eq!(relationship.family().as_str(), RelationshipFamily::IDENTITY);
        assert_eq!(
            relationship.direction(),
            RelationshipDirection::SubjectToObject
        );
        assert!(relationship.is_functional());
    }

    #[test]
    fn public_boundary_composes_relationship_with_explicit_inverse() {
        let has_name = predicate("has-name");
        let name_of = predicate("name-of");

        let inverse = InverseRelationship::new(has_name.clone(), name_of.clone())
            .expect("valid inverse relationship");

        let relationship = Relationship::new(
            has_name.clone(),
            family(RelationshipFamily::IDENTITY),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        )
        .with_inverse(inverse)
        .expect("inverse should attach");

        let stored_inverse = relationship.inverse().expect("inverse should be present");

        assert_eq!(stored_inverse.predicate(), &has_name);
        assert_eq!(stored_inverse.inverse_predicate(), &name_of);
    }

    #[test]
    fn public_boundary_exposes_structural_characteristics() {
        let characteristics =
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
                .expect("valid relationship characteristics");

        let relationship = Relationship::new(
            predicate("aliases"),
            family(RelationshipFamily::SEMANTIC),
            RelationshipDirection::SubjectToObject,
            characteristics,
        );

        assert!(relationship.is_symmetric());
        assert!(!relationship.is_asymmetric());
        assert!(
            relationship
                .characteristics()
                .supports_structural_reverse_view()
        );
    }

    #[test]
    fn symmetry_and_inverse_remain_distinct_relationship_mechanics() {
        let symmetric = Relationship::new(
            predicate("aliases"),
            family(RelationshipFamily::IDENTITY),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
                .expect("valid characteristics"),
        );

        let inverse = InverseRelationship::new(predicate("has-name"), predicate("name-of"))
            .expect("valid inverse declaration");

        let inverse_relationship = Relationship::new(
            predicate("has-name"),
            family(RelationshipFamily::IDENTITY),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        )
        .with_inverse(inverse)
        .expect("inverse should attach");

        assert!(symmetric.is_symmetric());
        assert!(symmetric.inverse().is_none());

        assert!(!inverse_relationship.is_symmetric());
        assert!(inverse_relationship.inverse().is_some());
    }

    #[test]
    fn public_boundary_preserves_composition_as_a_declaration_only() {
        let first = predicate("part-of");
        let second = predicate("part-of");
        let result = predicate("part-of");

        let rule = CompositionRule::new(first.clone(), second.clone(), result.clone());

        let relationship = Relationship::new(
            predicate("part-of"),
            family(RelationshipFamily::PART_WHOLE),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        )
        .with_composition_rule(rule);

        let stored_rule = relationship
            .composition_rules()
            .next()
            .expect("composition rule should be present");

        assert_eq!(stored_rule.first(), &first);
        assert_eq!(stored_rule.second(), &second);
        assert_eq!(stored_rule.result(), &result);
        assert_eq!(relationship.composition_rule_count(), 1);
    }

    #[test]
    fn public_boundary_registers_and_resolves_relationships() {
        let relationship = Relationship::new(
            predicate("has-name"),
            family(RelationshipFamily::IDENTITY),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        let relationship_predicate = relationship.predicate().clone();

        let mut vocabulary = RelationshipVocabulary::new();

        vocabulary
            .register(relationship)
            .expect("relationship should register");

        assert_eq!(vocabulary.len(), 1);
        assert!(vocabulary.contains(&relationship_predicate));

        let resolved = vocabulary
            .get(&relationship_predicate)
            .expect("relationship should resolve");

        assert_eq!(resolved.predicate(), &relationship_predicate);
    }

    #[test]
    fn public_boundary_rejects_duplicate_relationship_predicates() {
        let first = Relationship::new(
            predicate("has-name"),
            family(RelationshipFamily::IDENTITY),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        let duplicate = first.clone();

        let mut vocabulary = RelationshipVocabulary::new();

        vocabulary
            .register(first)
            .expect("first registration should succeed");

        let error = vocabulary
            .register(duplicate)
            .expect_err("duplicate registration must fail");

        assert_eq!(
            error,
            RelationshipVocabularyError::PredicateAlreadyRegistered {
                predicate: predicate("has-name"),
            }
        );
    }
}
