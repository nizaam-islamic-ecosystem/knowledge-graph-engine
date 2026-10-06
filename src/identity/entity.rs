//! Strongly typed identity for a knowledge-graph entity.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph entity.
    EntityId
);

#[cfg(test)]
mod tests {
    use super::EntityId;

    #[test]
    fn entity_id_can_be_generated() {
        let id = EntityId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn entity_id_generation_produces_distinct_values() {
        let first = EntityId::generate();
        let second = EntityId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn entity_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = EntityId::new("entity-1").expect("valid entity identity");
        assert_eq!(id.as_str(), "entity-1");
        assert!(EntityId::new("").is_err());
        assert!(EntityId::new("   ").is_err());
    }
}
