//! Strongly typed identity for a knowledge-graph source.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph source.
    SourceId
);

#[cfg(test)]
mod tests {
    use super::SourceId;

    #[test]
    fn source_id_can_be_generated() {
        let id = SourceId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn source_id_generation_produces_distinct_values() {
        let first = SourceId::generate();
        let second = SourceId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn source_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = SourceId::new("source-1").expect("valid source identity");
        assert_eq!(id.as_str(), "source-1");
        assert!(SourceId::new("").is_err());
        assert!(SourceId::new("   ").is_err());
    }
}
