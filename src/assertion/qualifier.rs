//! Generic assertion qualifiers for Phase 2.
//!
//! Qualifiers are intentionally distinct from assertion context.
//! Phase 2 provides only a generic key/value qualifier representation.
//!
//! Specialized temporal, evidence, provenance, authority, confidence, and
//! other domain-specific qualifiers belong to later phases.

use std::collections::BTreeSet;
use std::fmt;

/// Structural validation errors for an assertion qualifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualifierError {
    /// The qualifier key is empty or whitespace-only.
    EmptyKey,

    /// The qualifier key contains a Unicode control character.
    ControlCharacter { index: usize },
}

impl fmt::Display for QualifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("qualifier key must not be empty"),
            Self::ControlCharacter { index } => write!(
                formatter,
                "qualifier key contains a control character at index {index}",
            ),
        }
    }
}

impl std::error::Error for QualifierError {}

/// One generic key/value assertion qualifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Qualifier {
    key: String,
    value: String,
}

impl Qualifier {
    /// Creates a structurally valid qualifier.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, QualifierError> {
        let key = key.into();
        Self::validate_key(&key)?;

        Ok(Self {
            key,
            value: value.into(),
        })
    }

    /// Returns the qualifier key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the opaque qualifier value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    fn validate_key(key: &str) -> Result<(), QualifierError> {
        if key.trim().is_empty() {
            return Err(QualifierError::EmptyKey);
        }

        if let Some(index) = key.chars().position(char::is_control) {
            return Err(QualifierError::ControlCharacter { index });
        }

        Ok(())
    }
}

/// Deterministically ordered collection of assertion qualifiers.
///
/// Exact duplicate qualifiers are represented only once. Different values
/// for the same key remain distinct.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Qualifiers {
    values: BTreeSet<Qualifier>,
}

impl Qualifiers {
    /// Creates an empty qualifier collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a qualifier.
    ///
    /// Returns `true` when the qualifier was not already present.
    pub fn insert(&mut self, qualifier: Qualifier) -> bool {
        self.values.insert(qualifier)
    }

    /// Returns the number of unique qualifiers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no qualifiers are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterates in deterministic canonical order.
    pub fn iter(&self) -> impl Iterator<Item = &Qualifier> {
        self.values.iter()
    }
}

impl std::iter::FromIterator<Qualifier> for Qualifiers {
    fn from_iter<I>(qualifiers: I) -> Self
    where
        I: IntoIterator<Item = Qualifier>,
    {
        Self {
            values: qualifiers.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Qualifier, QualifierError, Qualifiers};

    #[test]
    fn creates_a_valid_qualifier() {
        let qualifier = Qualifier::new("scope", "primary").expect("valid qualifier");

        assert_eq!(qualifier.key(), "scope");
        assert_eq!(qualifier.value(), "primary");
    }

    #[test]
    fn empty_qualifier_keys_are_rejected() {
        assert_eq!(Qualifier::new("", "value"), Err(QualifierError::EmptyKey));

        assert_eq!(
            Qualifier::new("   ", "value"),
            Err(QualifierError::EmptyKey)
        );
    }

    #[test]
    fn control_characters_in_qualifier_keys_are_rejected() {
        assert_eq!(
            Qualifier::new("sc\npe", "primary"),
            Err(QualifierError::ControlCharacter { index: 2 })
        );
    }

    #[test]
    fn qualifier_collection_is_deterministically_ordered() {
        let first = Qualifier::new("z", "last").expect("valid qualifier");
        let second = Qualifier::new("a", "first").expect("valid qualifier");
        let third = Qualifier::new("m", "middle").expect("valid qualifier");

        let qualifiers = Qualifiers::from_iter([first, second, third]);

        let values = qualifiers
            .iter()
            .map(|qualifier| (qualifier.key(), qualifier.value()))
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![("a", "first"), ("m", "middle"), ("z", "last"),]
        );
    }

    #[test]
    fn duplicate_qualifiers_do_not_change_the_collection() {
        let qualifier = Qualifier::new("source", "quran").expect("valid qualifier");

        let mut qualifiers = Qualifiers::new();

        assert!(qualifiers.insert(qualifier.clone()));
        assert!(!qualifiers.insert(qualifier));

        assert_eq!(qualifiers.len(), 1);
    }

    #[test]
    fn different_values_for_the_same_key_remain_distinct() {
        let first = Qualifier::new("tag", "one").expect("valid qualifier");
        let second = Qualifier::new("tag", "two").expect("valid qualifier");

        let qualifiers = Qualifiers::from_iter([first, second]);

        assert_eq!(qualifiers.len(), 2);
    }
}
