//! First-class ontology class definitions for Phase 3.
//!
//! A [`Class`] is an ontology-level schema object. It is intentionally distinct
//! from the Phase 1 [`crate::concept::Concept`] model: a class describes a
//! category in the ontology, while a concept is an existing semantic object.
//!
//! `ClassId` is owned by this module because it identifies an ontology class.
//! Identity generation and value semantics remain entirely owned by Nizaam
//! Core's `identity!` mechanism.

use nizaam_core::identity;

identity!(
    /// Identifies one ontology class.
    ///
    /// The generated identity is opaque to the Knowledge Graph. The KG must
    /// not introduce a second class-identity or hashing scheme.
    ClassId
);

/// A first-class ontology class.
///
/// The class label is descriptive metadata only. Hierarchy membership is not
/// stored inside the class itself; [`super::taxonomy::ClassTaxonomy`] owns the
/// structural class hierarchy so multiple inheritance can be represented
/// without making the class object responsible for hierarchy semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Class {
    id: ClassId,
    label: String,
}

impl Class {
    /// Creates an ontology class from its Core-backed identity and label.
    #[must_use]
    pub fn new(id: ClassId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Returns the class identity.
    #[must_use]
    pub fn id(&self) -> &ClassId {
        &self.id
    }

    /// Returns the descriptive class label exactly as supplied.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::{Class, ClassId};
    use crate::concept::Concept;
    use crate::identity::ConceptId;

    #[test]
    fn class_preserves_identity_and_label() {
        let id = ClassId::new("class-person").expect("valid class identity");
        let class = Class::new(id.clone(), "Person");

        assert_eq!(class.id(), &id);
        assert_eq!(class.label(), "Person");
    }

    #[test]
    fn class_identity_is_core_backed_and_can_be_generated() {
        let generated = ClassId::generate();

        assert!(!generated.as_str().is_empty());
    }

    #[test]
    fn class_is_distinct_from_concept() {
        let class = Class::new(
            ClassId::new("class-person").expect("valid class identity"),
            "Person",
        );
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
    fn class_labels_are_opaque_metadata() {
        let class = Class::new(
            ClassId::new("class-arabic").expect("valid class identity"),
            "شخص",
        );

        assert_eq!(class.label(), "شخص");
    }
}
