//! Strongly typed identity for a knowledge-graph reference.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph reference.
    ReferenceId
);

#[cfg(test)]
mod tests {
    use super::ReferenceId;

    #[test]
    fn reference_id_can_be_generated() {
        let id = ReferenceId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn reference_id_generation_produces_distinct_values() {
        let first = ReferenceId::generate();
        let second = ReferenceId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn reference_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = ReferenceId::new("reference-1").expect("valid reference identity");
        assert_eq!(id.as_str(), "reference-1");
        assert!(ReferenceId::new("").is_err());
        assert!(ReferenceId::new("   ").is_err());
    }
}
