//! Deterministic, strongly typed identity for a knowledge assertion.
//!
//! `KnowledgeAssertionId` remains a Core identity type, but its semantic
//! construction path is deterministic: the already-canonical assertion
//! representation is passed through Core's validated identity constructor.
//!
//! The Core-generated `generate()` method remains present because it is part
//! of the Core `identity!` contract. It must not be used for semantic
//! assertion identity. Semantic assertion construction uses
//! `KnowledgeAssertionId::from_canonical(...)` instead.

use nizaam_core::identity;

/// Wraps Core's identity macro for the one KG identity whose semantic
/// construction must be deterministic.
///
/// This is intentionally local to this module. It is not a second identity
/// system and does not reimplement Core identity generation.
macro_rules! knowledge_assertion_identity {
    ($(#[$meta:meta])* $name:ident) => {
        identity!($(#[$meta])* $name);

        impl $name {
            /// Constructs a knowledge-assertion identity from an already
            /// canonicalized semantic representation.
            ///
            /// The canonicalization itself is owned by the assertion model.
            /// This method only delegates the resulting stable representation
            /// to Core's validated identity constructor.
            ///
            /// The KG does not implement a second hash, UUID, ULID, timestamp,
            /// counter, random, or other identity-generation mechanism.
            pub fn from_canonical(
                canonical: impl Into<String>,
            ) -> Result<Self, nizaam_core::identity::InvalidIdentity> {
                Self::new(canonical)
            }
        }
    };
}

knowledge_assertion_identity!(
    /// Identifies a knowledge-graph assertion by its canonical semantic identity.
    KnowledgeAssertionId
);

#[cfg(test)]
mod tests {
    use super::KnowledgeAssertionId;

    #[test]
    fn canonical_identity_is_deterministic() {
        let canonical = "subject=entity:1|predicate=kg.relationship.has-name|object=entity:2";

        let first = KnowledgeAssertionId::from_canonical(canonical)
            .expect("canonical assertion identity should be valid");

        let second = KnowledgeAssertionId::from_canonical(canonical)
            .expect("same canonical assertion identity should be valid");

        assert_eq!(first, second);
        assert_eq!(first.as_str(), canonical);
    }

    #[test]
    fn different_canonical_representations_produce_distinct_identities() {
        let first = KnowledgeAssertionId::from_canonical(
            "subject=entity:1|predicate=kg.relationship.has-name|object=entity:2",
        )
        .expect("first canonical assertion identity should be valid");

        let second = KnowledgeAssertionId::from_canonical(
            "subject=entity:1|predicate=kg.relationship.has-name|object=entity:3",
        )
        .expect("second canonical assertion identity should be valid");

        assert_ne!(first, second);
    }

    #[test]
    fn canonical_identity_rejects_empty_values() {
        assert!(KnowledgeAssertionId::from_canonical("").is_err());
        assert!(KnowledgeAssertionId::from_canonical("   ").is_err());
    }

    #[test]
    fn core_explicit_constructor_remains_available() {
        let id = KnowledgeAssertionId::new("assertion-1")
            .expect("Core identity constructor should remain available");

        assert_eq!(id.as_str(), "assertion-1");
    }
}
