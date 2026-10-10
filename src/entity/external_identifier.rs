//! Source-scoped external identifiers owned by the entity layer.
//!
//! An external identifier is assigned by a source system, not by the
//! Knowledge Graph. Its source namespace and original value are preserved
//! exactly. Resolution re-exports this type for compatibility and uses it in
//! source-to-canonical crosswalk records.

use core::fmt;

use crate::identity::SourceId;

/// An opaque external identifier supplied by one source system.
///
/// The pair `(source_id, value)` is the source-scoped identity. The value is
/// not trimmed or otherwise normalized after validation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExternalIdentifier {
    source_id: SourceId,
    value: String,
}

impl ExternalIdentifier {
    /// Creates a source-scoped external identifier.
    ///
    /// Empty/whitespace-only values and Unicode control characters are
    /// rejected. Valid values are otherwise preserved exactly as supplied.
    pub fn new(
        source_id: SourceId,
        value: impl Into<String>,
    ) -> Result<Self, ExternalIdentifierError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(ExternalIdentifierError::EmptyValue);
        }

        if let Some(index) = value.chars().position(char::is_control) {
            return Err(ExternalIdentifierError::ControlCharacter { index });
        }

        Ok(Self { source_id, value })
    }

    /// Returns the source that owns the external identifier namespace.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the external identifier exactly as supplied.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Structural validation errors for an external identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalIdentifierError {
    /// The external identifier is empty or whitespace-only.
    EmptyValue,

    /// The external identifier contains a Unicode control character.
    ControlCharacter {
        /// Character index of the first control character.
        index: usize,
    },
}

impl fmt::Display for ExternalIdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue => formatter.write_str("external identifier must not be empty"),
            Self::ControlCharacter { index } => write!(
                formatter,
                "external identifier contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for ExternalIdentifierError {}

#[cfg(test)]
mod tests {
    use super::{ExternalIdentifier, ExternalIdentifierError};
    use crate::identity::SourceId;

    #[test]
    fn external_identifier_preserves_source_and_original_value() {
        let source = SourceId::new("source-a").expect("valid source");
        let identifier =
            ExternalIdentifier::new(source.clone(), " Person-0042 ").expect("nonblank identifier");

        assert_eq!(identifier.source_id(), &source);
        assert_eq!(identifier.value(), " Person-0042 ");
    }

    #[test]
    fn empty_external_identifiers_are_rejected() {
        let source = SourceId::new("source-a").expect("valid source");

        assert_eq!(
            ExternalIdentifier::new(source.clone(), ""),
            Err(ExternalIdentifierError::EmptyValue)
        );
        assert_eq!(
            ExternalIdentifier::new(source, "   "),
            Err(ExternalIdentifierError::EmptyValue)
        );
    }

    #[test]
    fn control_characters_are_rejected() {
        let source = SourceId::new("source-a").expect("valid source");

        assert_eq!(
            ExternalIdentifier::new(source, "person\n42"),
            Err(ExternalIdentifierError::ControlCharacter { index: 6 })
        );
    }

    #[test]
    fn same_value_in_different_sources_is_a_distinct_identifier() {
        let first = ExternalIdentifier::new(
            SourceId::new("source-a").expect("valid source"),
            "record-42",
        )
        .expect("valid identifier");
        let second = ExternalIdentifier::new(
            SourceId::new("source-b").expect("valid source"),
            "record-42",
        )
        .expect("valid identifier");

        assert_ne!(first, second);
        assert_eq!(first.value(), second.value());
        assert_ne!(first.source_id(), second.source_id());
    }
}
