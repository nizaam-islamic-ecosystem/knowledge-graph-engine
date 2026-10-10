//! Strongly typed identity for a knowledge-graph evidence record.

use nizaam_core::identity;

identity!(
    /// Identifies an evidence record associated with knowledge in the graph.
    EvidenceId
);

#[cfg(test)]
mod tests {
    use super::EvidenceId;

    #[test]
    fn evidence_id_can_be_generated() {
        let id = EvidenceId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn evidence_id_generation_produces_distinct_values() {
        assert_ne!(EvidenceId::generate(), EvidenceId::generate());
    }

    #[test]
    fn evidence_id_accepts_valid_values_and_rejects_empty_values() {
        let id = EvidenceId::new("evidence-1").expect("valid evidence identity");
        assert_eq!(id.as_str(), "evidence-1");
        assert!(EvidenceId::new("").is_err());
        assert!(EvidenceId::new("   ").is_err());
    }
}
