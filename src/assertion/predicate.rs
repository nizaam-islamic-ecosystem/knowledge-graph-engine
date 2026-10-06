//! Predicate boundary for knowledge assertions.
//!
//! The assertion layer does not define a second predicate representation.
//! `AssertionPredicate` is an assertion-facing alias of the canonical
//! relationship predicate owned by the relationship module.
//!
//! Relationship namespace validation and vocabulary semantics remain owned by
//! `RelationshipPredicate` and the relationship module.

pub use crate::relationship::RelationshipPredicate as AssertionPredicate;

#[cfg(test)]
mod tests {
    use super::AssertionPredicate;

    #[test]
    fn assertion_predicate_uses_the_canonical_relationship_predicate() {
        let predicate = AssertionPredicate::new("has-name").expect("valid relationship predicate");

        assert_eq!(predicate.as_str(), "kg.relationship.has-name");
        assert_eq!(predicate.name(), "has-name");
    }

    #[test]
    fn assertion_predicate_preserves_relationship_predicate_equality() {
        let first = AssertionPredicate::new("has-name").expect("valid relationship predicate");
        let second = AssertionPredicate::new("has-name").expect("valid relationship predicate");

        assert_eq!(first, second);
    }

    #[test]
    fn assertion_predicate_does_not_create_a_second_identity_value() {
        let predicate = AssertionPredicate::new("aliases").expect("valid relationship predicate");

        assert_eq!(predicate.to_string(), "kg.relationship.aliases");
    }
}
