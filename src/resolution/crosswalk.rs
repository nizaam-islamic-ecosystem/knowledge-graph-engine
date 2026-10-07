//! External identifier crosswalk records for Phase 3.
//!
//! A crosswalk preserves the distinction between a source-owned external
//! identifier and the canonical Knowledge Graph entity it currently maps to.
//! It is not an entity identity, is not an IndexAssignedId, and does not own
//! persistence or historical versioning.

use core::fmt;

use crate::identity::{EntityId, SourceId};

/// An opaque external identifier supplied by one source system.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExternalIdentifier {
    source_id: SourceId,
    value: String,
}

impl ExternalIdentifier {
    /// Creates a source-scoped external identifier.
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
    ControlCharacter { index: usize },
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

/// A source-to-canonical crosswalk record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExternalIdentifierCrosswalk {
    external_identifier: ExternalIdentifier,
    canonical_entity: EntityId,
}

impl ExternalIdentifierCrosswalk {
    /// Creates a crosswalk record without changing either identity.
    #[must_use]
    pub fn new(external_identifier: ExternalIdentifier, canonical_entity: EntityId) -> Self {
        Self {
            external_identifier,
            canonical_entity,
        }
    }

    /// Returns the source-owned external identifier.
    #[must_use]
    pub fn external_identifier(&self) -> &ExternalIdentifier {
        &self.external_identifier
    }

    /// Returns the canonical KG entity identity.
    #[must_use]
    pub fn canonical_entity(&self) -> &EntityId {
        &self.canonical_entity
    }
}

#[cfg(test)]
mod tests {
    use super::{ExternalIdentifier, ExternalIdentifierCrosswalk, ExternalIdentifierError};
    use crate::identity::{EntityId, SourceId};

    #[test]
    fn external_identifier_preserves_source_and_value() {
        let source = SourceId::new("source-a").expect("valid source");
        let identifier = ExternalIdentifier::new(source.clone(), "person-42")
            .expect("valid external identifier");

        assert_eq!(identifier.source_id(), &source);
        assert_eq!(identifier.value(), "person-42");
    }

    #[test]
    fn crosswalk_preserves_canonical_entity_separately() {
        let external = ExternalIdentifier::new(
            SourceId::new("source-a").expect("valid source"),
            "person-42",
        )
        .expect("valid external identifier");
        let entity = EntityId::new("entity-1").expect("valid entity");
        let crosswalk = ExternalIdentifierCrosswalk::new(external, entity.clone());

        assert_eq!(crosswalk.canonical_entity(), &entity);
        assert_eq!(crosswalk.external_identifier().value(), "person-42");
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
}
