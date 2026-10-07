//! Phase 3 ontology boundary.
//!
//! The ontology module establishes the schema and semantic-structure layer
//! above the Phase 2 relationship and assertion model:
//!
//! - [`Class`] and [`ClassId`] define first-class ontology classes;
//! - [`OntologyProperty`] formalizes one existing Phase 2 relationship
//!   predicate with domain/range constraints;
//! - [`Taxonomy`] provides shared structural hierarchy infrastructure while
//!   keeping class and concept hierarchies type-distinct;
//! - [`Ontology`] coordinates the logical ontology model;
//! - [`OntologyRegistry`] owns controlled immutable snapshot registration and
//!   activation.
//!
//! The module does not replace the Phase 2 relationship vocabulary and does not
//! implement persistence, query planning, full reasoning, ingestion, or
//! version management.

mod class;
mod constraint;
mod model;
mod property;
mod registry;
mod seed;
mod taxonomy;

pub use class::{Class, ClassId};
pub use constraint::{
    CardinalityConstraint, ClassSet, ConstraintExtension, ConstraintSet, OntologyConstraint,
    OntologyConstraintError, ValidationIssue, ValidationReport,
};
pub use model::{Ontology, OntologyError};
pub use property::{OntologyProperty, OntologyPropertyValidationError};
pub use registry::{OntologyRegistry, OntologyRegistryError, OntologySnapshot, OntologySnapshotId};
pub use seed::{IslamicSeed, IslamicSeedError, load_islamic_seed, load_islamic_seed_from_str};
pub use taxonomy::{ClassTaxonomy, ConceptTaxonomy, Taxonomy, TaxonomyError};

#[cfg(test)]
mod tests {
    use super::{Class, ClassId, Ontology, OntologyProperty, OntologyRegistry, OntologySnapshotId};
    use crate::relationship::RelationshipPredicate;
    use std::collections::BTreeSet;

    fn class(value: &str, label: &str) -> Class {
        Class::new(ClassId::new(value).expect("valid class identity"), label)
    }

    #[test]
    fn public_ontology_boundary_composes_class_property_and_registry_layers() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");
        ontology
            .add_class(class("class-name", "Name"))
            .expect("class registration should succeed");

        let property = OntologyProperty::new(
            RelationshipPredicate::new("has-name").expect("valid predicate"),
            BTreeSet::from([ClassId::new("class-person").expect("valid class identity")]),
            BTreeSet::from([ClassId::new("class-name").expect("valid class identity")]),
        )
        .expect("valid ontology property");

        ontology
            .add_property(property)
            .expect("property registration should succeed");

        assert!(ontology.validate().is_valid());

        let mut registry = OntologyRegistry::new();
        let snapshot_id = registry
            .register(ontology)
            .expect("valid ontology should register");

        registry
            .activate(&snapshot_id)
            .expect("registered snapshot should activate");

        assert_eq!(registry.active_snapshot_id(), Some(&snapshot_id));
        assert_eq!(
            registry.active_snapshot().unwrap().ontology().class_count(),
            2
        );
    }

    #[test]
    fn public_boundary_exposes_core_backed_snapshot_identity() {
        let generated = OntologySnapshotId::generate();

        assert!(!generated.as_str().is_empty());
    }
}
