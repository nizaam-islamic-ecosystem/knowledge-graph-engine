//! Strongly typed identity for a knowledge assertion.
//!
//! Phase 1 establishes the assertion identity boundary only. The
//! `KnowledgeAssertion` semantic object is intentionally deferred to a later
//! phase.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph assertion.
    KnowledgeAssertionId
);

#[cfg(test)]
mod tests {
    use super::KnowledgeAssertionId;

    #[test]
    fn knowledge_assertion_id_can_be_generated() {
        let id = KnowledgeAssertionId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn knowledge_assertion_id_generation_produces_distinct_values() {
        let first = KnowledgeAssertionId::generate();
        let second = KnowledgeAssertionId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn knowledge_assertion_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id =
            KnowledgeAssertionId::new("assertion-1").expect("valid knowledge assertion identity");
        assert_eq!(id.as_str(), "assertion-1");
        assert!(KnowledgeAssertionId::new("").is_err());
        assert!(KnowledgeAssertionId::new("   ").is_err());
    }
}
