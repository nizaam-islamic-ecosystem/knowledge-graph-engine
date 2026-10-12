//! Strongly typed identity for a knowledge-graph reasoning rule.

use nizaam_core::identity;

identity!(
    /// Identifies a reasoning rule used by the knowledge-graph reasoning subsystem.
    RuleId
);

#[cfg(test)]
mod tests {
    use super::RuleId;

    #[test]
    fn rule_id_can_be_generated() {
        let id = RuleId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn rule_id_generation_produces_distinct_values() {
        assert_ne!(RuleId::generate(), RuleId::generate());
    }

    #[test]
    fn rule_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = RuleId::new("rule-1").expect("valid rule identity");
        assert_eq!(id.as_str(), "rule-1");
        assert!(RuleId::new("").is_err());
        assert!(RuleId::new("   ").is_err());
    }
}
