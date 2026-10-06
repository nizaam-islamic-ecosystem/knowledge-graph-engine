//! Strongly typed identity for a knowledge-graph mention.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph mention.
    MentionId
);

#[cfg(test)]
mod tests {
    use super::MentionId;

    #[test]
    fn mention_id_can_be_generated() {
        let id = MentionId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn mention_id_generation_produces_distinct_values() {
        let first = MentionId::generate();
        let second = MentionId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn mention_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = MentionId::new("mention-1").expect("valid mention identity");
        assert_eq!(id.as_str(), "mention-1");
        assert!(MentionId::new("").is_err());
        assert!(MentionId::new("   ").is_err());
    }
}
