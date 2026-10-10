//! Generic, vocabulary-extensible scholarly status values.
//!
//! Domain-specific classification systems remain data rather than being
//! hard-coded into the universal Knowledge Graph authority vocabulary.

use core::fmt;

/// Errors raised while constructing a scholarly status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScholarlyStatusError {
    /// A required text field is empty or whitespace-only.
    EmptyField {
        /// Name of the field that failed validation.
        field: &'static str,
    },
    /// A text field contains a Unicode control character.
    ControlCharacter {
        /// Name of the field that failed validation.
        field: &'static str,
        /// Character index of the invalid value.
        index: usize,
    },
}

impl fmt::Display for ScholarlyStatusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => {
                write!(formatter, "scholarly status {field} must not be empty")
            }
            Self::ControlCharacter { field, index } => write!(
                formatter,
                "scholarly status {field} contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for ScholarlyStatusError {}

fn validate_text(value: &str, field: &'static str) -> Result<(), ScholarlyStatusError> {
    if value.trim().is_empty() {
        return Err(ScholarlyStatusError::EmptyField { field });
    }

    if let Some(index) = value.chars().position(char::is_control) {
        return Err(ScholarlyStatusError::ControlCharacter { field, index });
    }

    Ok(())
}

/// A generic scholarly status label, optionally qualified by a vocabulary.
///
/// Values such as `peer-reviewed`, `disputed`, or a domain-specific status are
/// preserved as provided. The Knowledge Graph does not infer scholarly
/// acceptance from the presence of a status or impose a single global taxonomy.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScholarlyStatus {
    vocabulary: Option<String>,
    value: String,
}

impl ScholarlyStatus {
    /// Creates an unqualified scholarly status value.
    pub fn new(value: impl Into<String>) -> Result<Self, ScholarlyStatusError> {
        let value = value.into();
        validate_text(&value, "value")?;

        Ok(Self {
            vocabulary: None,
            value,
        })
    }

    /// Creates a scholarly status from an explicit domain vocabulary and value.
    pub fn in_vocabulary(
        vocabulary: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ScholarlyStatusError> {
        let vocabulary = vocabulary.into();
        let value = value.into();
        validate_text(&vocabulary, "vocabulary")?;
        validate_text(&value, "value")?;

        Ok(Self {
            vocabulary: Some(vocabulary),
            value,
        })
    }

    /// Returns the vocabulary identifier, if supplied.
    #[must_use]
    pub fn vocabulary(&self) -> Option<&str> {
        self.vocabulary.as_deref()
    }

    /// Returns the status label exactly as stored.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::{ScholarlyStatus, ScholarlyStatusError};

    #[test]
    fn scholarly_status_supports_domain_specific_vocabularies() {
        let status = ScholarlyStatus::in_vocabulary("islamic-scholarship-v1", "disputed")
            .expect("generic model should preserve a domain-specific status");

        assert_eq!(status.vocabulary(), Some("islamic-scholarship-v1"));
        assert_eq!(status.value(), "disputed");
    }

    #[test]
    fn unqualified_scholarly_status_is_supported() {
        let status = ScholarlyStatus::new("peer-reviewed").expect("valid status");
        assert_eq!(status.vocabulary(), None);
        assert_eq!(status.value(), "peer-reviewed");
    }

    #[test]
    fn invalid_scholarly_status_text_is_rejected() {
        assert_eq!(
            ScholarlyStatus::new(" "),
            Err(ScholarlyStatusError::EmptyField { field: "value" })
        );
        assert!(matches!(
            ScholarlyStatus::in_vocabulary("review\ncontext", "accepted"),
            Err(ScholarlyStatusError::ControlCharacter {
                field: "vocabulary",
                ..
            })
        ));
        assert!(ScholarlyStatus::in_vocabulary("domain", "\0invalid").is_err());
    }
}
