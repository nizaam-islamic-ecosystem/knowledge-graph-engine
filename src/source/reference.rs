//! Minimal generic reference representation for Phase 1.
//!
//! A reference has its own identity and an opaque reference value. Phase 1
//! deliberately does not resolve, hydrate, parse, or look up the referenced
//! source/document/passage.

use crate::identity::ReferenceId;

/// A generic source/reference value owned by the knowledge-graph model.
///
/// The value is intentionally opaque. Domain-specific reference parsing and
/// lookup are later-phase responsibilities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reference {
    id: ReferenceId,
    value: String,
}

impl Reference {
    /// Constructs a reference from its identity and opaque value.
    #[must_use]
    pub fn new(id: ReferenceId, value: impl Into<String>) -> Self {
        Self {
            id,
            value: value.into(),
        }
    }

    /// Returns the reference identity.
    #[must_use]
    pub fn id(&self) -> &ReferenceId {
        &self.id
    }

    /// Returns the opaque reference value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::Reference;
    use crate::identity::ReferenceId;

    #[test]
    fn reference_preserves_identity_and_opaque_value() {
        let id = ReferenceId::generate();
        let reference = Reference::new(id.clone(), "opaque-reference-1");

        assert_eq!(reference.id(), &id);
        assert_eq!(reference.value(), "opaque-reference-1");
    }

    #[test]
    fn reference_value_is_preserved_without_domain_specific_parsing() {
        let reference = Reference::new(ReferenceId::generate(), "quran://example/reference");

        assert_eq!(reference.value(), "quran://example/reference");
    }

    #[test]
    fn reference_generation_produces_distinct_identities() {
        let first = ReferenceId::generate();
        let second = ReferenceId::generate();

        assert_ne!(first, second);
    }
}
