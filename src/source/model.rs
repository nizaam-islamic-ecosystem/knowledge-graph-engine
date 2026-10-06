//! Minimal knowledge-graph source representation for Phase 1.
//!
//! A source is a minimal semantic representation of an origin from which
//! knowledge may later be derived. Phase 1 intentionally does not define
//! authority, reliability, provenance, document hierarchy, passage structure,
//! ingestion, lookup, or persistence.

use crate::identity::SourceId;

/// A minimal identifiable knowledge-graph source.
///
/// The source label is intentionally opaque identifying metadata. More
/// detailed source semantics belong to later phases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    id: SourceId,
    label: String,
}

impl Source {
    /// Constructs a source from its identity and minimal identifying label.
    #[must_use]
    pub fn new(id: SourceId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Returns the source identity.
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Returns the opaque identifying label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::Source;
    use crate::identity::SourceId;

    #[test]
    fn source_preserves_identity_and_label() {
        let id = SourceId::generate();
        let source = Source::new(id.clone(), "Example source");

        assert_eq!(source.id(), &id);
        assert_eq!(source.label(), "Example source");
    }

    #[test]
    fn source_accepts_multilingual_identifying_metadata() {
        let arabic = Source::new(SourceId::generate(), "مصدر");
        let english = Source::new(SourceId::generate(), "Source");
        let urdu = Source::new(SourceId::generate(), "ماخذ");

        assert_eq!(arabic.label(), "مصدر");
        assert_eq!(english.label(), "Source");
        assert_eq!(urdu.label(), "ماخذ");
    }

    #[test]
    fn source_generation_produces_distinct_identities() {
        let first = SourceId::generate();
        let second = SourceId::generate();

        assert_ne!(first, second);
    }
}
