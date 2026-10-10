//! Strongly typed identity for a provenance activity.

use nizaam_core::identity;

identity!(
    /// Identifies an activity recorded in knowledge provenance.
    ActivityId
);

#[cfg(test)]
mod tests {
    use super::ActivityId;

    #[test]
    fn activity_id_can_be_generated() {
        let id = ActivityId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn activity_id_generation_produces_distinct_values() {
        assert_ne!(ActivityId::generate(), ActivityId::generate());
    }

    #[test]
    fn activity_id_accepts_valid_values_and_rejects_empty_values() {
        let id = ActivityId::new("activity-1").expect("valid activity identity");
        assert_eq!(id.as_str(), "activity-1");
        assert!(ActivityId::new("").is_err());
        assert!(ActivityId::new("   ").is_err());
    }
}
