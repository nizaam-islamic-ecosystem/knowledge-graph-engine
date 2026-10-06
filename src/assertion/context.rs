//! Generic assertion context representation for Phase 2.
//!
//! Context is intentionally opaque and structurally simple in this phase.
//! It provides deterministic key/value metadata that can participate in
//! canonical assertion identity.
//!
//! Semantic interpretation of context belongs to later phases.

use std::collections::BTreeMap;
use std::fmt;

/// Structural validation errors for an assertion-context entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssertionContextError {
    /// A context key is empty or whitespace-only.
    EmptyKey,

    /// A context key contains a Unicode control character.
    ControlCharacter { index: usize },
}

impl fmt::Display for AssertionContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("assertion context key must not be empty"),
            Self::ControlCharacter { index } => write!(
                formatter,
                "assertion context key contains a control character at index {index}",
            ),
        }
    }
}

impl std::error::Error for AssertionContextError {}

/// Generic deterministic assertion context.
///
/// Context entries are ordered by key so their canonical representation is
/// stable regardless of insertion order.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssertionContext {
    entries: BTreeMap<String, String>,
}

impl AssertionContext {
    /// Creates an empty assertion context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an assertion context from key/value pairs.
    ///
    /// Keys are validated structurally and stored in deterministic order.
    pub fn from_entries<I, K, V>(entries: I) -> Result<Self, AssertionContextError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut context = Self::new();

        for (key, value) in entries {
            context.insert(key, value)?;
        }

        Ok(context)
    }

    /// Inserts or replaces one context entry.
    ///
    /// Returns the previous value when the key already existed.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Option<String>, AssertionContextError> {
        let key = key.into();
        Self::validate_key(&key)?;

        Ok(self.entries.insert(key, value.into()))
    }

    /// Returns the value associated with a context key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    /// Returns the number of context entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the context contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over context entries in canonical key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    fn validate_key(key: &str) -> Result<(), AssertionContextError> {
        if key.trim().is_empty() {
            return Err(AssertionContextError::EmptyKey);
        }

        if let Some(index) = key.chars().position(char::is_control) {
            return Err(AssertionContextError::ControlCharacter { index });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AssertionContext, AssertionContextError};

    #[test]
    fn creates_an_empty_context() {
        let context = AssertionContext::new();

        assert!(context.is_empty());
        assert_eq!(context.len(), 0);
    }

    #[test]
    fn inserts_and_reads_context_entries() {
        let mut context = AssertionContext::new();

        context
            .insert("source", "quran")
            .expect("valid context key");

        assert_eq!(context.get("source"), Some("quran"));
        assert_eq!(context.len(), 1);
    }

    #[test]
    fn duplicate_keys_replace_the_previous_value() {
        let mut context = AssertionContext::new();

        assert_eq!(
            context
                .insert("source", "quran")
                .expect("valid context key"),
            None
        );

        assert_eq!(
            context
                .insert("source", "hadith")
                .expect("valid context key"),
            Some("quran".to_owned())
        );

        assert_eq!(context.get("source"), Some("hadith"));
        assert_eq!(context.len(), 1);
    }

    #[test]
    fn iteration_is_deterministically_sorted_by_key() {
        let context =
            AssertionContext::from_entries([("z", "last"), ("a", "first"), ("m", "middle")])
                .expect("valid context entries");

        let entries = context.iter().collect::<Vec<_>>();

        assert_eq!(
            entries,
            vec![("a", "first"), ("m", "middle"), ("z", "last"),]
        );
    }

    #[test]
    fn empty_keys_are_rejected() {
        assert_eq!(
            AssertionContext::new().insert("", "value"),
            Err(AssertionContextError::EmptyKey)
        );

        assert_eq!(
            AssertionContext::new().insert("   ", "value"),
            Err(AssertionContextError::EmptyKey)
        );
    }

    #[test]
    fn control_characters_in_keys_are_rejected() {
        assert_eq!(
            AssertionContext::new().insert("sou\nrce", "quran"),
            Err(AssertionContextError::ControlCharacter { index: 3 })
        );
    }

    #[test]
    fn values_remain_opaque() {
        let context =
            AssertionContext::from_entries([("language", "العربية"), ("note", "any opaque value")])
                .expect("valid context entries");

        assert_eq!(context.get("language"), Some("العربية"));
        assert_eq!(context.get("note"), Some("any opaque value"));
    }
}
