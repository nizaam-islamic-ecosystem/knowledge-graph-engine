//! Ontology property definitions for Phase 3.
//!
//! An [`OntologyProperty`] formalizes one Phase 2 relationship predicate for
//! ontology use. It does not replace [`crate::relationship::Relationship`] or
//! create a second predicate vocabulary.
//!
//! Ontology-defined properties require explicit domain and range class sets.
//! Generic Phase 2 relationships remain valid without an ontology property.

use core::fmt;

use crate::relationship::RelationshipPredicate;

use super::class::ClassId;
use super::constraint::{ClassSet, ConstraintSet, OntologyConstraint};

/// A formal ontology definition for exactly one relationship predicate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyProperty {
    predicate: RelationshipPredicate,
    domain: ClassSet,
    range: ClassSet,
    constraints: ConstraintSet,
}

impl OntologyProperty {
    /// Creates an ontology property.
    ///
    /// Unlike an unconstrained generic KG relationship, an ontology-defined
    /// property must declare at least one domain class and one range class.
    pub fn new(
        predicate: RelationshipPredicate,
        domain: ClassSet,
        range: ClassSet,
    ) -> Result<Self, OntologyPropertyValidationError> {
        if domain.is_empty() {
            return Err(OntologyPropertyValidationError::EmptyDomain);
        }

        if range.is_empty() {
            return Err(OntologyPropertyValidationError::EmptyRange);
        }

        Ok(Self {
            predicate,
            domain,
            range,
            constraints: ConstraintSet::new(),
        })
    }

    /// Returns the single Phase 2 relationship predicate formalized by this
    /// ontology property.
    #[must_use]
    pub fn predicate(&self) -> &RelationshipPredicate {
        &self.predicate
    }

    /// Returns the property's domain class set.
    #[must_use]
    pub fn domain(&self) -> &ClassSet {
        &self.domain
    }

    /// Returns the property's range class set.
    #[must_use]
    pub fn range(&self) -> &ClassSet {
        &self.range
    }

    /// Returns the structural constraints attached to this property.
    #[must_use]
    pub fn constraints(&self) -> &ConstraintSet {
        &self.constraints
    }

    /// Returns a property with the supplied constraint added.
    #[must_use]
    pub fn with_constraint(mut self, constraint: OntologyConstraint) -> Self {
        self.constraints.insert(constraint);
        self
    }

    /// Performs local property validation.
    pub fn validate(&self) -> Result<(), OntologyPropertyValidationError> {
        if self.domain.is_empty() {
            return Err(OntologyPropertyValidationError::EmptyDomain);
        }

        if self.range.is_empty() {
            return Err(OntologyPropertyValidationError::EmptyRange);
        }

        Ok(())
    }

    /// Returns whether the property declares the supplied domain class.
    #[must_use]
    pub fn applies_to_domain(&self, class_id: &ClassId) -> bool {
        self.domain.contains(class_id)
    }

    /// Returns whether the property declares the supplied range class.
    #[must_use]
    pub fn accepts_range(&self, class_id: &ClassId) -> bool {
        self.range.contains(class_id)
    }
}

/// Structural validation failures for an ontology property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OntologyPropertyValidationError {
    /// An ontology-defined property must have at least one domain class.
    EmptyDomain,

    /// An ontology-defined property must have at least one range class.
    EmptyRange,
}

impl fmt::Display for OntologyPropertyValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDomain => formatter.write_str("ontology property domain must not be empty"),
            Self::EmptyRange => formatter.write_str("ontology property range must not be empty"),
        }
    }
}

impl std::error::Error for OntologyPropertyValidationError {}

#[cfg(test)]
mod tests {
    use super::{OntologyProperty, OntologyPropertyValidationError};
    use crate::ontology::class::ClassId;
    use crate::ontology::constraint::OntologyConstraint;
    use crate::relationship::RelationshipPredicate;
    use std::collections::BTreeSet;

    fn class_id(value: &str) -> ClassId {
        ClassId::new(value).expect("valid class identity")
    }

    fn predicate(value: &str) -> RelationshipPredicate {
        RelationshipPredicate::new(value).expect("valid relationship predicate")
    }

    fn class_set(values: &[&str]) -> BTreeSet<ClassId> {
        values.iter().map(|value| class_id(value)).collect()
    }

    #[test]
    fn ontology_property_preserves_one_phase2_predicate_and_its_domain_range() {
        let property = OntologyProperty::new(
            predicate("has-name"),
            class_set(&["class-person"]),
            class_set(&["class-name"]),
        )
        .expect("valid ontology property");

        assert_eq!(property.predicate().as_str(), "kg.relationship.has-name");
        assert_eq!(property.domain(), &class_set(&["class-person"]));
        assert_eq!(property.range(), &class_set(&["class-name"]));
    }

    #[test]
    fn empty_domain_is_rejected() {
        assert_eq!(
            OntologyProperty::new(
                predicate("has-name"),
                BTreeSet::new(),
                class_set(&["class-name"]),
            ),
            Err(OntologyPropertyValidationError::EmptyDomain)
        );
    }

    #[test]
    fn empty_range_is_rejected() {
        assert_eq!(
            OntologyProperty::new(
                predicate("has-name"),
                class_set(&["class-person"]),
                BTreeSet::new(),
            ),
            Err(OntologyPropertyValidationError::EmptyRange)
        );
    }

    #[test]
    fn ontology_property_can_carry_minimum_constraints() {
        let cardinality = OntologyConstraint::cardinality(Some(1), Some(1))
            .expect("valid cardinality constraint");

        let property = OntologyProperty::new(
            predicate("has-name"),
            class_set(&["class-person"]),
            class_set(&["class-name"]),
        )
        .expect("valid ontology property")
        .with_constraint(cardinality.clone());

        assert!(property.constraints().contains(&cardinality));
    }

    #[test]
    fn domain_and_range_membership_are_structural_queries() {
        let person = class_id("class-person");
        let name = class_id("class-name");
        let other = class_id("class-other");

        let property = OntologyProperty::new(
            predicate("has-name"),
            class_set(&["class-person"]),
            class_set(&["class-name"]),
        )
        .expect("valid ontology property");

        assert!(property.applies_to_domain(&person));
        assert!(!property.applies_to_domain(&other));
        assert!(property.accepts_range(&name));
        assert!(!property.accepts_range(&other));
    }

    #[test]
    fn generic_relationships_remain_separate_from_ontology_properties() {
        let generic = predicate("related-to");
        let property = OntologyProperty::new(
            generic.clone(),
            class_set(&["class-person"]),
            class_set(&["class-person"]),
        )
        .expect("valid ontology property");

        assert_eq!(property.predicate(), &generic);
        assert_eq!(generic.as_str(), "kg.relationship.related-to");
    }
}
