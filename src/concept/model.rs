//! Minimal knowledge-graph concept representation for Phase 1.
//!
//! A concept represents an abstract semantic idea in the knowledge model.
//! Phase 1 keeps the representation deliberately small and does not establish
//! ontology, concept hierarchies, relationships, semantic inference, or
//! lexical-to-concept mapping.

use crate::identity::ConceptId;

/// A minimal identifiable knowledge-graph concept.
///
/// `representation` is opaque semantic representation data. The Knowledge
/// Graph does not interpret it linguistically in Phase 1.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Concept {
    id: ConceptId,
    representation: String,
}

impl Concept {
    /// Constructs a concept from its identity and minimal representation.
    #[must_use]
    pub fn new(id: ConceptId, representation: impl Into<String>) -> Self {
        Self {
            id,
            representation: representation.into(),
        }
    }

    /// Returns the concept identity.
    #[must_use]
    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    /// Returns the opaque semantic representation.
    #[must_use]
    pub fn representation(&self) -> &str {
        &self.representation
    }
}

#[cfg(test)]
mod tests {
    use super::Concept;
    use crate::identity::ConceptId;

    #[test]
    fn concept_preserves_identity_and_representation() {
        let id = ConceptId::generate();
        let concept = Concept::new(id.clone(), "patience");

        assert_eq!(concept.id(), &id);
        assert_eq!(concept.representation(), "patience");
    }

    #[test]
    fn concept_representation_is_preserved_as_opaque_data() {
        let arabic = Concept::new(ConceptId::generate(), "صبر");
        let english = Concept::new(ConceptId::generate(), "patience");
        let urdu = Concept::new(ConceptId::generate(), "صبر");

        assert_eq!(arabic.representation(), "صبر");
        assert_eq!(english.representation(), "patience");
        assert_eq!(urdu.representation(), "صبر");
    }

    #[test]
    fn distinct_concept_id_values_remain_distinct() {
        let first = ConceptId::generate();
        let second = ConceptId::generate();

        assert_ne!(first, second);
    }
}
