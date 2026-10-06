//! Explicit inverse relationship declarations.
//!
//! Phase 2 stores inverse semantics as a declaration between two typed
//! relationship predicates. It does not duplicate canonical relationships or
//! knowledge assertions.
//!
//! Symmetry and inverse relationships remain separate concepts.

use core::fmt;

use super::predicate::RelationshipPredicate;

/// Structural validation failures for an inverse relationship declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InverseRelationshipError {
    /// A predicate cannot be declared as the inverse of itself in the Phase 2
    /// relationship model.
    SelfInverse {
        /// The predicate involved in the invalid declaration.
        predicate: RelationshipPredicate,
    },
}

impl fmt::Display for InverseRelationshipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfInverse { predicate } => write!(
                formatter,
                "relationship predicate cannot be declared as its own inverse: {predicate}",
            ),
        }
    }
}

impl std::error::Error for InverseRelationshipError {}

/// Explicit inverse declaration between two relationship predicates.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InverseRelationship {
    predicate: RelationshipPredicate,
    inverse_predicate: RelationshipPredicate,
}

impl InverseRelationship {
    /// Creates an explicit inverse declaration.
    pub fn new(
        predicate: RelationshipPredicate,
        inverse_predicate: RelationshipPredicate,
    ) -> Result<Self, InverseRelationshipError> {
        if predicate == inverse_predicate {
            return Err(InverseRelationshipError::SelfInverse { predicate });
        }

        Ok(Self {
            predicate,
            inverse_predicate,
        })
    }

    /// Returns the canonical predicate for the declaration.
    #[must_use]
    pub fn predicate(&self) -> &RelationshipPredicate {
        &self.predicate
    }

    /// Returns the explicitly declared inverse predicate.
    #[must_use]
    pub fn inverse_predicate(&self) -> &RelationshipPredicate {
        &self.inverse_predicate
    }

    /// Returns the same declaration viewed from the inverse side.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            predicate: self.inverse_predicate.clone(),
            inverse_predicate: self.predicate.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InverseRelationship, InverseRelationshipError};
    use crate::relationship::predicate::RelationshipPredicate;

    #[test]
    fn creates_an_explicit_inverse_declaration() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        let inverse = RelationshipPredicate::new("name-of").expect("valid predicate");

        let declaration = InverseRelationship::new(predicate.clone(), inverse.clone())
            .expect("valid inverse declaration");

        assert_eq!(declaration.predicate(), &predicate);
        assert_eq!(declaration.inverse_predicate(), &inverse);
    }

    #[test]
    fn inverse_declaration_does_not_equal_the_inverse_predicate_itself() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        let inverse = RelationshipPredicate::new("name-of").expect("valid predicate");

        let declaration = InverseRelationship::new(predicate.clone(), inverse.clone())
            .expect("valid inverse declaration");

        assert_ne!(declaration.predicate(), declaration.inverse_predicate());
    }

    #[test]
    fn self_inverse_declarations_are_rejected() {
        let predicate = RelationshipPredicate::new("aliases").expect("valid predicate");

        let error = InverseRelationship::new(predicate.clone(), predicate.clone())
            .expect_err("self-inverse declaration must be rejected");

        assert_eq!(error, InverseRelationshipError::SelfInverse { predicate });
    }

    #[test]
    fn reversed_declaration_swaps_the_predicates() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        let inverse = RelationshipPredicate::new("name-of").expect("valid predicate");

        let declaration = InverseRelationship::new(predicate.clone(), inverse.clone())
            .expect("valid inverse declaration");

        let reversed = declaration.reversed();

        assert_eq!(reversed.predicate(), &inverse);
        assert_eq!(reversed.inverse_predicate(), &predicate);
    }

    #[test]
    fn reversing_twice_restores_the_original_declaration() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        let inverse = RelationshipPredicate::new("name-of").expect("valid predicate");

        let declaration =
            InverseRelationship::new(predicate, inverse).expect("valid inverse declaration");

        assert_eq!(declaration, declaration.reversed().reversed());
    }
}
