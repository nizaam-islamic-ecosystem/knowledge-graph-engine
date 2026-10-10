//! Strongly typed identity for a contradiction record.

use nizaam_core::identity;

identity!(
    /// Identifies a contradiction record involving conflicting assertions.
    ContradictionId
);

#[cfg(test)]
mod tests {
    use super::ContradictionId;

    #[test]
    fn contradiction_id_can_be_generated() {
        let id = ContradictionId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn contradiction_id_generation_produces_distinct_values() {
        assert_ne!(ContradictionId::generate(), ContradictionId::generate());
    }

    #[test]
    fn contradiction_id_accepts_valid_values_and_rejects_empty_values() {
        let id = ContradictionId::new("contradiction-1").expect("valid contradiction identity");
        assert_eq!(id.as_str(), "contradiction-1");
        assert!(ContradictionId::new("").is_err());
        assert!(ContradictionId::new("   ").is_err());
    }
}
