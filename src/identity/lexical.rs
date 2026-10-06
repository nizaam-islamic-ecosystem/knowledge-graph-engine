//! Strongly typed identity for a knowledge-graph lexical form.

use nizaam_core::identity;

identity!(
    /// Identifies a knowledge-graph lexical form.
    LexicalFormId
);

#[cfg(test)]
mod tests {
    use super::LexicalFormId;

    #[test]
    fn lexical_form_id_can_be_generated() {
        let id = LexicalFormId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn lexical_form_id_generation_produces_distinct_values() {
        let first = LexicalFormId::generate();
        let second = LexicalFormId::generate();
        assert_ne!(first, second);
    }

    #[test]
    fn lexical_form_id_accepts_valid_explicit_values_and_rejects_empty_values() {
        let id = LexicalFormId::new("lexical-form-1").expect("valid lexical identity");
        assert_eq!(id.as_str(), "lexical-form-1");
        assert!(LexicalFormId::new("").is_err());
        assert!(LexicalFormId::new("   ").is_err());
    }
}
