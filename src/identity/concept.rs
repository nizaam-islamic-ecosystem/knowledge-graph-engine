//! Strongly typed identity for a knowledge-graph concept.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph concept.
    ConceptId
);

#[cfg(test)]
mod tests {
    use super::ConceptId;

    #[test]
    fn concept_id_can_be_generated() {
        let id = ConceptId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn concept_id_generation_produces_distinct_values() {
        let first = ConceptId::generate();
        let second = ConceptId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn concept_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = ConceptId::new("concept-1").expect("valid concept identity");
        assert_eq!(id.as_str(), "concept-1");
        assert!(ConceptId::new("").is_err());
        assert!(ConceptId::new("   ").is_err());
    }
}
