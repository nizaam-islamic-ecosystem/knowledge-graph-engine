//! First-class meaning representation for Phase 3.
//!
//! `Meaning` is intentionally independent from `Concept`, `Sense`, and
//! `Entity`. Semantic connections are represented through the Phase 2
//! assertion model rather than by embedding a concept identity into meaning.

/// A first-class semantic meaning representation.
///
/// The representation is deliberately opaque. Phase 3 records the semantic
/// object but does not perform linguistic analysis, morphology, parsing, or
/// semantic inference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Meaning {
    representation: String,
}

impl Meaning {
    /// Creates a meaning from an opaque semantic representation.
    #[must_use]
    pub fn new(representation: impl Into<String>) -> Self {
        Self {
            representation: representation.into(),
        }
    }

    /// Returns the opaque semantic representation exactly as supplied.
    #[must_use]
    pub fn representation(&self) -> &str {
        &self.representation
    }

    /// Returns whether the representation is empty after trimming whitespace.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.representation.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Meaning;

    #[test]
    fn meaning_is_a_first_class_value() {
        let meaning = Meaning::new("patience");

        assert_eq!(meaning.representation(), "patience");
        assert!(!meaning.is_empty());
    }

    #[test]
    fn meaning_remains_independent_from_concept() {
        let meaning = Meaning::new("patience");

        assert_eq!(meaning.representation(), "patience");
        assert_eq!(
            std::mem::size_of_val(&meaning),
            std::mem::size_of::<Meaning>()
        );
    }

    #[test]
    fn meaning_preserves_multilingual_opaque_data() {
        let arabic = Meaning::new("صبر");
        let english = Meaning::new("patience");

        assert_eq!(arabic.representation(), "صبر");
        assert_eq!(english.representation(), "patience");
    }
}
