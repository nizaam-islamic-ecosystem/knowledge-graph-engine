//! Strongly typed relationship predicates for the Knowledge Graph.
//!
//! A `RelationshipPredicate` represents one canonical relationship-vocabulary
//! term in the `kg.relationship.<name>` namespace.
//!
//! This type is a semantic vocabulary value, not a generated KG object
//! identity. It therefore does not use Core's `identity!` mechanism.
//!
//! Relationship definitions, families, characteristics, directions, inverse
//! declarations, and vocabulary registration belong to the surrounding
//! relationship model rather than this module.

use core::fmt;
use std::str::FromStr;

/// Canonical namespace for Knowledge Graph relationship predicates.
pub const RELATIONSHIP_NAMESPACE: &str = "kg.relationship.";

/// A strongly typed Knowledge Graph relationship predicate.
///
/// The stored value always contains the complete canonical namespace:
///
/// ```text
/// kg.relationship.<name>
/// ```
///
/// The local relationship name is supplied to [`RelationshipPredicate::new`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RelationshipPredicate(String);

impl RelationshipPredicate {
    /// Creates a relationship predicate from its local vocabulary name.
    ///
    /// The namespace is added automatically:
    ///
    /// ```text
    /// "has-name"
    ///     ↓
    /// "kg.relationship.has-name"
    /// ```
    ///
    /// The input must be non-empty, must not contain Unicode control
    /// characters, and must not already contain the canonical relationship
    /// namespace.
    pub fn new(name: impl Into<String>) -> Result<Self, RelationshipPredicateValidationError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(RelationshipPredicateValidationError::EmptyName);
        }

        if let Some(index) = name.chars().position(char::is_control) {
            return Err(RelationshipPredicateValidationError::ControlCharacter { index });
        }

        if name.starts_with(RELATIONSHIP_NAMESPACE) {
            return Err(RelationshipPredicateValidationError::AlreadyNamespaced);
        }

        Ok(Self(format!("{RELATIONSHIP_NAMESPACE}{name}")))
    }

    /// Returns the complete canonical predicate representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the local relationship vocabulary name.
    ///
    /// For:
    ///
    /// ```text
    /// kg.relationship.has-name
    /// ```
    ///
    /// this returns:
    ///
    /// ```text
    /// has-name
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0[RELATIONSHIP_NAMESPACE.len()..]
    }

    /// Returns the canonical relationship namespace.
    #[must_use]
    pub const fn namespace() -> &'static str {
        RELATIONSHIP_NAMESPACE
    }
}

impl AsRef<str> for RelationshipPredicate {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RelationshipPredicate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RelationshipPredicate {
    type Err = RelationshipPredicateValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<String> for RelationshipPredicate {
    type Error = RelationshipPredicateValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for RelationshipPredicate {
    type Error = RelationshipPredicateValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Structural validation failures for a relationship predicate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipPredicateValidationError {
    /// The supplied local relationship name is empty or whitespace-only.
    EmptyName,

    /// The supplied relationship name contains a Unicode control character.
    ControlCharacter {
        /// Character index of the control character.
        index: usize,
    },

    /// The caller supplied the canonical namespace even though `new` expects
    /// only the local relationship name.
    AlreadyNamespaced,
}

impl fmt::Display for RelationshipPredicateValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => {
                formatter.write_str("relationship predicate name must not be empty")
            }
            Self::ControlCharacter { index } => write!(
                formatter,
                "relationship predicate name contains a control character at index {index}"
            ),
            Self::AlreadyNamespaced => formatter.write_str(
                "relationship predicate constructor expects a local name, not an already namespaced value",
            ),
        }
    }
}

impl std::error::Error for RelationshipPredicateValidationError {}

#[cfg(test)]
mod tests {
    use super::{
        RELATIONSHIP_NAMESPACE, RelationshipPredicate, RelationshipPredicateValidationError,
    };

    #[test]
    fn constructs_canonical_relationship_predicate() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        assert_eq!(predicate.as_str(), "kg.relationship.has-name");
    }

    #[test]
    fn exposes_the_local_relationship_name() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        assert_eq!(predicate.name(), "has-name");
    }

    #[test]
    fn exposes_the_relationship_namespace() {
        assert_eq!(RelationshipPredicate::namespace(), RELATIONSHIP_NAMESPACE);
    }

    #[test]
    fn display_uses_the_canonical_predicate_representation() {
        let predicate = RelationshipPredicate::new("aliases").expect("valid predicate");

        assert_eq!(predicate.to_string(), "kg.relationship.aliases");
    }

    #[test]
    fn predicates_with_different_names_are_distinct() {
        let has_name = RelationshipPredicate::new("has-name").expect("valid predicate");
        let aliases = RelationshipPredicate::new("aliases").expect("valid predicate");

        assert_ne!(has_name, aliases);
    }

    #[test]
    fn predicates_with_the_same_name_are_equal() {
        let first = RelationshipPredicate::new("has-name").expect("valid predicate");
        let second = RelationshipPredicate::new("has-name").expect("valid predicate");

        assert_eq!(first, second);
    }

    #[test]
    fn empty_names_are_rejected() {
        assert_eq!(
            RelationshipPredicate::new(""),
            Err(RelationshipPredicateValidationError::EmptyName)
        );

        assert_eq!(
            RelationshipPredicate::new("   "),
            Err(RelationshipPredicateValidationError::EmptyName)
        );
    }

    #[test]
    fn control_characters_are_rejected() {
        let error = RelationshipPredicate::new("has-\n-name")
            .expect_err("control character should be rejected");

        assert_eq!(
            error,
            RelationshipPredicateValidationError::ControlCharacter { index: 4 }
        );
    }

    #[test]
    fn already_namespaced_values_are_rejected() {
        assert_eq!(
            RelationshipPredicate::new("kg.relationship.has-name"),
            Err(RelationshipPredicateValidationError::AlreadyNamespaced)
        );
    }

    #[test]
    fn from_str_uses_the_same_validation_rules() {
        let predicate = "has-name"
            .parse::<RelationshipPredicate>()
            .expect("valid predicate");

        assert_eq!(predicate.as_str(), "kg.relationship.has-name");

        assert_eq!(
            "kg.relationship.has-name".parse::<RelationshipPredicate>(),
            Err(RelationshipPredicateValidationError::AlreadyNamespaced)
        );
    }

    #[test]
    fn try_from_conversions_use_the_same_constructor() {
        let from_string =
            RelationshipPredicate::try_from("has-name".to_owned()).expect("valid predicate");

        let from_str = RelationshipPredicate::try_from("aliases").expect("valid predicate");

        assert_eq!(from_string.as_str(), "kg.relationship.has-name");

        assert_eq!(from_str.as_str(), "kg.relationship.aliases");
    }

    #[test]
    fn predicate_is_string_reference_compatible() {
        let predicate = RelationshipPredicate::new("has-name").expect("valid predicate");

        let value: &str = predicate.as_ref();

        assert_eq!(value, "kg.relationship.has-name");
    }
}
