//! Ontology registry and immutable snapshot access for Phase 3.
//!
//! The registry provides controlled registration and activation of complete
//! ontology snapshots. Snapshot identity is a Core-backed identity and is not
//! an ontology version number. Version management remains a later-phase
//! concern.
//!
//! Registered snapshots are exposed only through shared references. The
//! registry intentionally provides no mutable access to a stored snapshot.

use core::fmt;
use std::collections::BTreeMap;

use nizaam_core::identity;

use super::constraint::ValidationReport;
use super::model::Ontology;

identity!(
    /// Identifies one immutable ontology snapshot held by the registry.
    OntologySnapshotId
);

/// One immutable ontology snapshot stored by [`OntologyRegistry`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologySnapshot {
    id: OntologySnapshotId,
    ontology: Ontology,
}

impl OntologySnapshot {
    fn new(id: OntologySnapshotId, ontology: Ontology) -> Self {
        Self { id, ontology }
    }

    /// Returns the snapshot identity.
    #[must_use]
    pub fn id(&self) -> &OntologySnapshotId {
        &self.id
    }

    /// Returns the immutable ontology model contained in this snapshot.
    #[must_use]
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }
}

/// Controlled in-memory ontology snapshot registry.
#[derive(Clone, Debug, Default)]
pub struct OntologyRegistry {
    snapshots: BTreeMap<OntologySnapshotId, OntologySnapshot>,
    active: Option<OntologySnapshotId>,
}

impl OntologyRegistry {
    /// Creates an empty ontology registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and registers a new immutable ontology snapshot.
    pub fn register(
        &mut self,
        ontology: Ontology,
    ) -> Result<OntologySnapshotId, OntologyRegistryError> {
        let report = ontology.validate();

        if !report.is_valid() {
            return Err(OntologyRegistryError::InvalidOntology { report });
        }

        let id = OntologySnapshotId::generate();
        let snapshot = OntologySnapshot::new(id.clone(), ontology);

        self.snapshots.insert(id.clone(), snapshot);

        Ok(id)
    }

    /// Returns a registered immutable snapshot by identity.
    #[must_use]
    pub fn snapshot(&self, id: &OntologySnapshotId) -> Option<&OntologySnapshot> {
        self.snapshots.get(id)
    }

    /// Returns whether a snapshot is registered.
    #[must_use]
    pub fn contains(&self, id: &OntologySnapshotId) -> bool {
        self.snapshots.contains_key(id)
    }

    /// Returns the number of registered snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns whether no snapshots are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Iterates over registered snapshots in deterministic identity order.
    pub fn iter(&self) -> impl Iterator<Item = (&OntologySnapshotId, &OntologySnapshot)> {
        self.snapshots.iter()
    }

    /// Activates one already registered snapshot.
    pub fn activate(&mut self, id: &OntologySnapshotId) -> Result<(), OntologyRegistryError> {
        if !self.contains(id) {
            return Err(OntologyRegistryError::SnapshotNotFound { id: id.clone() });
        }

        self.active = Some(id.clone());
        Ok(())
    }

    /// Returns the identity of the currently active snapshot, when one exists.
    #[must_use]
    pub fn active_snapshot_id(&self) -> Option<&OntologySnapshotId> {
        self.active.as_ref()
    }

    /// Returns the currently active immutable snapshot, when one exists.
    #[must_use]
    pub fn active_snapshot(&self) -> Option<&OntologySnapshot> {
        self.active.as_ref().and_then(|id| self.snapshots.get(id))
    }
}

/// Registry operation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyRegistryError {
    /// Registration was rejected because ontology validation reported errors.
    InvalidOntology {
        /// Complete validation report produced by the ontology model.
        report: ValidationReport,
    },

    /// The requested snapshot identity is not registered.
    SnapshotNotFound {
        /// Unknown snapshot identity.
        id: OntologySnapshotId,
    },
}

impl fmt::Display for OntologyRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOntology { report } => write!(
                formatter,
                "ontology registration rejected because validation failed ({} error(s))",
                report.errors().len()
            ),
            Self::SnapshotNotFound { id } => {
                write!(formatter, "ontology snapshot is not registered: {id}")
            }
        }
    }
}

impl std::error::Error for OntologyRegistryError {}

#[cfg(test)]
mod tests {
    use super::{OntologyRegistry, OntologyRegistryError, OntologySnapshotId};
    use crate::ontology::class::{Class, ClassId};
    use crate::ontology::model::Ontology;
    use crate::ontology::property::OntologyProperty;
    use crate::relationship::RelationshipPredicate;
    use std::collections::BTreeSet;

    fn class(value: &str, label: &str) -> Class {
        Class::new(ClassId::new(value).expect("valid class identity"), label)
    }

    fn valid_ontology() -> Ontology {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");
        ontology
            .add_class(class("class-name", "Name"))
            .expect("class registration should succeed");

        ontology
            .add_property(
                OntologyProperty::new(
                    RelationshipPredicate::new("has-name").expect("valid predicate"),
                    BTreeSet::from([ClassId::new("class-person").expect("valid class identity")]),
                    BTreeSet::from([ClassId::new("class-name").expect("valid class identity")]),
                )
                .expect("valid ontology property"),
            )
            .expect("property registration should succeed");

        ontology
    }

    #[test]
    fn registry_registers_valid_ontology_as_an_immutable_snapshot() {
        let mut registry = OntologyRegistry::new();
        let id = registry
            .register(valid_ontology())
            .expect("valid ontology should register");

        assert!(!id.as_str().is_empty());
        assert!(registry.contains(&id));
        assert_eq!(registry.len(), 1);

        let snapshot = registry.snapshot(&id).expect("snapshot should exist");
        assert_eq!(snapshot.id(), &id);
        assert_eq!(snapshot.ontology().class_count(), 2);
    }

    #[test]
    fn generated_snapshot_id_is_core_backed_and_not_semantic_ontology_data() {
        let generated = OntologySnapshotId::generate();

        assert!(!generated.as_str().is_empty());
    }

    #[test]
    fn invalid_ontology_is_rejected_during_registry_registration() {
        let mut ontology = Ontology::new();
        ontology
            .add_class(class("class-person", "Person"))
            .expect("class registration should succeed");
        ontology
            .add_property(
                OntologyProperty::new(
                    RelationshipPredicate::new("has-name").expect("valid predicate"),
                    BTreeSet::from([ClassId::new("class-person").expect("valid class identity")]),
                    BTreeSet::from([ClassId::new("class-name").expect("valid class identity")]),
                )
                .expect("property can be assembled before validation"),
            )
            .expect("property registration should succeed");

        let mut registry = OntologyRegistry::new();
        let error = registry
            .register(ontology)
            .expect_err("invalid ontology must not become a registry snapshot");

        assert!(matches!(
            error,
            OntologyRegistryError::InvalidOntology { .. }
        ));
        assert!(registry.is_empty());
    }

    #[test]
    fn activation_requires_a_registered_snapshot() {
        let mut registry = OntologyRegistry::new();
        let unknown = OntologySnapshotId::new("unknown-snapshot").expect("valid snapshot identity");

        assert_eq!(
            registry.activate(&unknown),
            Err(OntologyRegistryError::SnapshotNotFound { id: unknown })
        );
        assert!(registry.active_snapshot().is_none());
    }

    #[test]
    fn activation_returns_the_selected_immutable_snapshot() {
        let mut registry = OntologyRegistry::new();
        let id = registry
            .register(valid_ontology())
            .expect("valid ontology should register");

        registry
            .activate(&id)
            .expect("registered snapshot should activate");

        assert_eq!(registry.active_snapshot_id(), Some(&id));
        assert_eq!(
            registry.active_snapshot().unwrap().ontology().class_count(),
            2
        );
    }

    #[test]
    fn mutating_a_clone_does_not_mutate_the_registered_snapshot() {
        let mut registry = OntologyRegistry::new();
        let id = registry
            .register(valid_ontology())
            .expect("valid ontology should register");

        let mut clone = registry
            .snapshot(&id)
            .expect("snapshot should exist")
            .ontology()
            .clone();

        clone
            .add_class(class("class-animal", "Animal"))
            .expect("clone should remain independently mutable");

        assert_eq!(registry.snapshot(&id).unwrap().ontology().class_count(), 2);
        assert_eq!(clone.class_count(), 3);
    }
}
