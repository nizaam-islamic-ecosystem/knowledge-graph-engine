//! Strongly typed identity for an agent participating in knowledge provenance.

use nizaam_core::identity;

identity!(
    /// Identifies an agent responsible for or participating in an activity.
    AgentId
);

#[cfg(test)]
mod tests {
    use super::AgentId;

    #[test]
    fn agent_id_can_be_generated() {
        let id = AgentId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn agent_id_generation_produces_distinct_values() {
        assert_ne!(AgentId::generate(), AgentId::generate());
    }

    #[test]
    fn agent_id_accepts_valid_values_and_rejects_empty_values() {
        let id = AgentId::new("agent-1").expect("valid agent identity");
        assert_eq!(id.as_str(), "agent-1");
        assert!(AgentId::new("").is_err());
        assert!(AgentId::new("   ").is_err());
    }
}
