//! Ontology-level coordination model for Phase 3.
//!
//! [`Ontology`] coordinates first-class classes, formal ontology properties,
//! class taxonomy, and class-scoped constraints. It is a logical semantic
//! model only. It does not own persistence, query planning, graph traversal,
//! full inference, or ontology version management.
//!
//! Phase 2 remains authoritative for generic relationship predicates and
//! knowledge assertions. This module formalizes selected predicates for
//! ontology use without replacing the Phase 2 relationship vocabulary.

use core::fmt;
use std::collections::BTreeMap;

use crate::relationship::RelationshipPredicate;

use super::class::{Class, ClassId};
use super::constraint::{ConstraintSet, OntologyConstraint, ValidationReport};
use super::property::{OntologyProperty, OntologyPropertyValidationError};
use super::taxonomy::{ClassTaxonomy, TaxonomyError};

/// The in-memory ontology model.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ontology {
    classes: BTreeMap<ClassId, Class>,
    properties: BTreeMap<RelationshipPredicate, OntologyProperty>,
    class_taxonomy: ClassTaxonomy,
    class_constraints: BTreeMap<ClassId, ConstraintSet>,
}

impl Ontology {
    /// Creates an empty ontology.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a first-class ontology class.
    ///
    /// The class is also inserted into the structural class taxonomy so later
    /// multiple-inheritance relations can reference it immediately.
    pub fn add_class(&mut self, class: Class) -> Result<(), OntologyError> {
        let id = class.id().clone();

        if self.classes.contains_key(&id) {
            return Err(OntologyError::ClassAlreadyRegistered { class_id: id });
        }

        self.class_taxonomy.insert_node(id.clone());
        self.classes.insert(id, class);

        Ok(())
    }

    /// Returns a class by identity.
    #[must_use]
    pub fn class(&self, class_id: &ClassId) -> Option<&Class> {
        self.classes.get(class_id)
    }

    /// Returns whether a class is registered.
    #[must_use]
    pub fn contains_class(&self, class_id: &ClassId) -> bool {
        self.classes.contains_key(class_id)
    }

    /// Returns the number of registered classes.
    #[must_use]
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Iterates over classes in deterministic identity order.
    pub fn classes(&self) -> impl Iterator<Item = (&ClassId, &Class)> {
        self.classes.iter()
    }

    /// Registers an ontology property.
    ///
    /// The property is structurally validated before insertion. References to
    /// class identities are intentionally checked by [`Self::validate`] rather
    /// than rejected here so the ontology can be assembled before a complete
    /// validation pass.
    pub fn add_property(&mut self, property: OntologyProperty) -> Result<(), OntologyError> {
        property
            .validate()
            .map_err(|error| OntologyError::InvalidProperty {
                predicate: property.predicate().clone(),
                error,
            })?;

        let predicate = property.predicate().clone();

        if self.properties.contains_key(&predicate) {
            return Err(OntologyError::PropertyAlreadyRegistered { predicate });
        }

        self.properties.insert(predicate, property);

        Ok(())
    }

    /// Returns an ontology property by its Phase 2 relationship predicate.
    #[must_use]
    pub fn property(&self, predicate: &RelationshipPredicate) -> Option<&OntologyProperty> {
        self.properties.get(predicate)
    }

    /// Returns whether an ontology property is registered for a predicate.
    #[must_use]
    pub fn contains_property(&self, predicate: &RelationshipPredicate) -> bool {
        self.properties.contains_key(predicate)
    }

    /// Returns the number of ontology-defined properties.
    #[must_use]
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// Iterates over ontology properties in deterministic predicate order.
    pub fn properties(&self) -> impl Iterator<Item = (&RelationshipPredicate, &OntologyProperty)> {
        self.properties.iter()
    }

    /// Adds a structural class-parent relation.
    ///
    /// The relation permits multiple parents but rejects cycles. No inherited
    /// assertions or transitive closure are generated.
    pub fn add_class_parent(
        &mut self,
        child: ClassId,
        parent: ClassId,
    ) -> Result<bool, OntologyError> {
        if !self.contains_class(&child) {
            return Err(OntologyError::UnknownClass { class_id: child });
        }

        if !self.contains_class(&parent) {
            return Err(OntologyError::UnknownClass { class_id: parent });
        }

        self.class_taxonomy
            .add_parent(child, parent)
            .map_err(OntologyError::Taxonomy)
    }

    /// Returns the structural class taxonomy.
    #[must_use]
    pub fn class_taxonomy(&self) -> &ClassTaxonomy {
        &self.class_taxonomy
    }

    /// Attaches a structural constraint to a registered class.
    pub fn add_class_constraint(
        &mut self,
        class_id: ClassId,
        constraint: OntologyConstraint,
    ) -> Result<bool, OntologyError> {
        if !self.contains_class(&class_id) {
            return Err(OntologyError::UnknownClass { class_id });
        }

        Ok(self
            .class_constraints
            .entry(class_id)
            .or_default()
            .insert(constraint))
    }

    /// Returns constraints attached to a class.
    #[must_use]
    pub fn class_constraints(&self, class_id: &ClassId) -> Option<&ConstraintSet> {
        self.class_constraints.get(class_id)
    }

    /// Validates ontology-wide structural consistency.
    ///
    /// This is a schema validation pass. It does not validate graph instances,
    /// execute inference, or evaluate evidence/provenance.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        for (predicate, property) in &self.properties {
            if let Err(error) = property.validate() {
                report.add_error(
                    "ontology.property.invalid",
                    format!("property {predicate} is structurally invalid: {error}"),
                );
            }

            for class_id in property.domain() {
                if !self.contains_class(class_id) {
                    report.add_error(
                        "ontology.property.domain.unknown_class",
                        format!("property {predicate} references unknown domain class {class_id}"),
                    );
                }
            }

            for class_id in property.range() {
                if !self.contains_class(class_id) {
                    report.add_error(
                        "ontology.property.range.unknown_class",
                        format!("property {predicate} references unknown range class {class_id}"),
                    );
                }
            }
        }

        for (class_id, constraints) in &self.class_constraints {
            if !self.contains_class(class_id) {
                report.add_error(
                    "ontology.constraint.unknown_class",
                    format!("constraints reference unknown class {class_id}"),
                );
            }

            self.validate_constraint_class_references(constraints, &mut report);
        }

        for property in self.properties.values() {
            self.validate_constraint_class_references(property.constraints(), &mut report);
        }

        for class_id in self.class_taxonomy.nodes() {
            if !self.contains_class(class_id) {
                report.add_error(
                    "ontology.taxonomy.unknown_class",
                    format!("class taxonomy contains unknown class {class_id}"),
                );
            }
        }

        report
    }

    fn validate_constraint_class_references(
        &self,
        constraints: &ConstraintSet,
        report: &mut ValidationReport,
    ) {
        for constraint in constraints.iter() {
            if let OntologyConstraint::DisjointWith(classes) = constraint {
                for class_id in classes {
                    if !self.contains_class(class_id) {
                        report.add_error(
                            "ontology.constraint.disjoint.unknown_class",
                            format!("disjointness constraint references unknown class {class_id}"),
                        );
                    }
                }
            }
        }
    }
}

/// Structural failures while mutating the ontology model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyError {
    /// A class identity is already registered.
    ClassAlreadyRegistered {
        /// Duplicate class identity.
        class_id: ClassId,
    },

    /// An ontology property predicate is already registered.
    PropertyAlreadyRegistered {
        /// Duplicate relationship predicate.
        predicate: RelationshipPredicate,
    },

    /// A property failed its local structural validation.
    InvalidProperty {
        /// Predicate of the invalid property.
        predicate: RelationshipPredicate,

        /// Property validation failure.
        error: OntologyPropertyValidationError,
    },

    /// A hierarchy or constraint operation referenced an unknown class.
    UnknownClass {
        /// Missing class identity.
        class_id: ClassId,
    },

    /// The class taxonomy rejected the requested structural relation.
    Taxonomy(TaxonomyError<ClassId>),
}

impl fmt::Display for OntologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClassAlreadyRegistered { class_id } => {
                write!(
                    formatter,
                    "ontology class is already registered: {class_id}"
                )
            }
            Self::PropertyAlreadyRegistered { predicate } => {
                write!(
                    formatter,
                    "ontology property is already registered: {predicate}"
                )
            }
            Self::InvalidProperty { predicate, error } => {
                write!(
                    formatter,
                    "ontology property {predicate} is invalid: {error}"
                )
            }
            Self::UnknownClass { class_id } => {
                write!(formatter, "ontology class is not registered: {class_id}")
            }
            Self::Taxonomy(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for OntologyError {}

#[cfg(test)]
mod tests {
    use super::{Ontology, OntologyError};
    use crate::concept::Concept;
    use crate::identity::ConceptId;
    use crate::ontology::class::{Class, ClassId};
    use crate::ontology::constraint::OntologyConstraint;
    use crate::ontology::property::OntologyProperty;
    use crate::relationship::RelationshipPredicate;
    use std::collections::BTreeSet;

    fn class(value: &str, label: &str) -> Class {
        Class::new(ClassId::new(value).expect("valid class identity"), label)
    }

    fn class_id(value: &str) -> ClassId {
        ClassId::new(value).expect("valid class identity")
    }

    fn predicate(value: &str) -> RelationshipPredicate {
        RelationshipPredicate::new(value).expect("valid relationship predicate")
    }

    fn property(predicate_name: &str, domain: &str, range: &str) -> OntologyProperty {
        OntologyProperty::new(
            predicate(predicate_name),
            BTreeSet::from([class_id(domain)]),
            BTreeSet::from([class_id(range)]),
        )
        .expect("valid ontology property")
    }

    #[test]
    fn ontology_registers_first_class_classes() {
        let mut ontology = Ontology::new();
        let person = class("class-person", "Person");
        let id = person.id().clone();

        ontology
            .add_class(person)
            .expect("class registration should succeed");

        assert_eq!(ontology.class(&id).unwrap().label(), "Person");
        assert_eq!(ontology.class_count(), 1);
        assert!(ontology.class_taxonomy().contains(&id));
    }

    #[test]
    fn duplicate_class_registration_is_rejected() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("first class registration should succeed");

        let error = ontology
            .add_class(class("class-person", "Person duplicate"))
            .expect_err("duplicate class should be rejected");

        assert_eq!(
            error,
            OntologyError::ClassAlreadyRegistered {
                class_id: class_id("class-person")
            }
        );
    }

    #[test]
    fn ontology_properties_reference_exactly_one_phase2_predicate() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");
        ontology
            .add_class(class("class-name", "Name"))
            .expect("class registration should succeed");

        let property = property("has-name", "class-person", "class-name");
        let predicate = property.predicate().clone();
        ontology
            .add_property(property)
            .expect("property registration should succeed");

        assert_eq!(
            ontology.property(&predicate).unwrap().predicate(),
            &predicate
        );
        assert_eq!(ontology.property_count(), 1);
    }

    #[test]
    fn duplicate_property_predicates_are_rejected() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");
        ontology
            .add_class(class("class-name", "Name"))
            .expect("class registration should succeed");

        ontology
            .add_property(property("has-name", "class-person", "class-name"))
            .expect("first property should succeed");

        let error = ontology
            .add_property(property("has-name", "class-person", "class-name"))
            .expect_err("duplicate predicate should be rejected");

        assert_eq!(
            error,
            OntologyError::PropertyAlreadyRegistered {
                predicate: predicate("has-name")
            }
        );
    }

    #[test]
    fn ontology_validation_reports_unknown_domain_and_range_classes() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");

        ontology
            .add_property(property("has-name", "class-person", "class-name"))
            .expect("property can be assembled before complete validation");

        let report = ontology.validate();

        assert!(!report.is_valid());
        assert_eq!(report.errors().len(), 1);
        assert_eq!(
            report.errors()[0].code(),
            "ontology.property.range.unknown_class"
        );
    }

    #[test]
    fn ontology_supports_multiple_class_parents_without_inference() {
        let mut ontology = Ontology::new();
        for class_name in [
            ("class-object", "Object"),
            ("class-agent", "Agent"),
            ("class-person", "Person"),
        ] {
            ontology
                .add_class(class(class_name.0, class_name.1))
                .expect("class registration should succeed");
        }

        assert!(
            ontology
                .add_class_parent(class_id("class-person"), class_id("class-object"))
                .expect("first parent should succeed")
        );
        assert!(
            ontology
                .add_class_parent(class_id("class-person"), class_id("class-agent"))
                .expect("second parent should succeed")
        );

        let parents = ontology
            .class_taxonomy()
            .parents_of(&class_id("class-person"))
            .expect("person should exist");

        assert_eq!(parents.len(), 2);
    }

    #[test]
    fn ontology_class_constraints_are_typed_and_structural() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");

        let constraint = OntologyConstraint::disjoint_with([class_id("class-animal")])
            .expect("valid disjointness constraint");

        assert!(
            ontology
                .add_class_constraint(class_id("class-person"), constraint.clone())
                .expect("constraint should be added")
        );

        assert!(
            ontology
                .class_constraints(&class_id("class-person"))
                .unwrap()
                .contains(&constraint)
        );
    }

    #[test]
    fn ontology_validation_reports_unknown_classes_inside_disjointness_constraints() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");

        let constraint = OntologyConstraint::disjoint_with([class_id("class-animal")])
            .expect("valid disjointness constraint");
        ontology
            .add_class_constraint(class_id("class-person"), constraint)
            .expect("constraint should be attached");

        let report = ontology.validate();

        assert!(!report.is_valid());
        assert!(
            report
                .errors()
                .iter()
                .any(|error| error.code() == "ontology.constraint.disjoint.unknown_class")
        );
    }

    #[test]
    fn class_and_concept_remain_distinct_semantic_categories() {
        let class = class("class-person", "Person");
        let concept = Concept::new(
            ConceptId::new("concept-person").expect("valid concept identity"),
            "person",
        );

        assert_ne!(
            std::any::TypeId::of::<Class>(),
            std::any::TypeId::of::<Concept>()
        );
        assert_eq!(class.label(), "Person");
        assert_eq!(concept.representation(), "person");
    }

    #[test]
    fn generic_relationships_can_exist_without_ontology_formalization() {
        let generic = predicate("related-to");
        let ontology = Ontology::new();

        assert!(!ontology.contains_property(&generic));
    }

    #[test]
    fn ontology_rejects_hierarchy_relations_to_unknown_classes() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");

        let error = ontology
            .add_class_parent(class_id("class-person"), class_id("class-agent"))
            .expect_err("unknown parent should be rejected");

        assert_eq!(
            error,
            OntologyError::UnknownClass {
                class_id: class_id("class-agent")
            }
        );
    }
}
