//! Source and source-version origin metadata for knowledge provenance.
//!
//! An origin describes where knowledge came from. It is not evidence that the
//! knowledge is true, and it is not a Core operation/audit record. Structured
//! source references use the existing typed `SourceId` and `ReferenceId`.

use core::fmt;

use crate::identity::{ReferenceId, SourceId};

/// Origin details known for a knowledge object.
///
/// A source may be specified with an optional source-version label and an
/// optional generic reference/locator. Version labels are source metadata, not
/// Phase 7 KG storage versions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeOrigin {
    source_id: Option<SourceId>,
    source_version: Option<String>,
    reference_id: Option<ReferenceId>,
    locator: Option<String>,
}

impl KnowledgeOrigin {
    /// Creates origin metadata from explicitly supplied fields.
    pub fn new(
        source_id: Option<SourceId>,
        source_version: Option<String>,
        reference_id: Option<ReferenceId>,
        locator: Option<String>,
    ) -> Result<Self, KnowledgeOriginError> {
        if source_id.is_none() && reference_id.is_none() {
            return Err(KnowledgeOriginError::RequiresSourceOrReference);
        }
        if source_version.is_some() && source_id.is_none() {
            return Err(KnowledgeOriginError::SourceVersionRequiresSource);
        }
        if locator.is_some() && reference_id.is_none() {
            return Err(KnowledgeOriginError::LocatorRequiresReference);
        }
        if let Some(version) = &source_version {
            validate_label(version, "source version")?;
        }
        if let Some(locator) = &locator {
            validate_label(locator, "source locator")?;
        }

        Ok(Self {
            source_id,
            source_version,
            reference_id,
            locator,
        })
    }

    /// Creates an origin that identifies a source without asserting a version.
    #[must_use]
    pub fn from_source(source_id: SourceId) -> Self {
        Self {
            source_id: Some(source_id),
            source_version: None,
            reference_id: None,
            locator: None,
        }
    }

    /// Creates an origin for a specific source-version label.
    pub fn source_version(
        source_id: SourceId,
        version: impl Into<String>,
    ) -> Result<Self, KnowledgeOriginError> {
        Self::new(Some(source_id), Some(version.into()), None, None)
    }

    /// Creates an origin associated with a generic source reference.
    pub fn source_reference(source_id: Option<SourceId>, reference_id: ReferenceId) -> Self {
        Self {
            source_id,
            source_version: None,
            reference_id: Some(reference_id),
            locator: None,
        }
    }

    /// Creates an origin with a structured reference and opaque locator.
    pub fn source_location(
        source_id: Option<SourceId>,
        reference_id: ReferenceId,
        locator: impl Into<String>,
    ) -> Result<Self, KnowledgeOriginError> {
        Self::new(source_id, None, Some(reference_id), Some(locator.into()))
    }

    /// Explicitly represents an origin that is not known.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            source_id: None,
            source_version: None,
            reference_id: None,
            locator: None,
        }
    }

    /// Returns the source identity, if known.
    #[must_use]
    pub fn source_id(&self) -> Option<&SourceId> {
        self.source_id.as_ref()
    }

    /// Returns the source-version label, if supplied.
    #[must_use]
    pub fn source_version_label(&self) -> Option<&str> {
        self.source_version.as_deref()
    }

    /// Returns the generic source/reference identity, if available.
    #[must_use]
    pub fn reference_id(&self) -> Option<&ReferenceId> {
        self.reference_id.as_ref()
    }

    /// Returns the opaque locator within the reference, if supplied.
    #[must_use]
    pub fn locator(&self) -> Option<&str> {
        self.locator.as_deref()
    }

    /// Returns whether the origin is explicitly unknown.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.source_id.is_none()
            && self.source_version.is_none()
            && self.reference_id.is_none()
            && self.locator.is_none()
    }

    /// Revalidates public fields/invariants at attachment boundaries.
    pub fn validate(&self) -> Result<(), KnowledgeOriginError> {
        if self.is_unknown() {
            return Ok(());
        }
        if self.source_id.is_none() && self.reference_id.is_none() {
            return Err(KnowledgeOriginError::RequiresSourceOrReference);
        }
        if self.source_version.is_some() && self.source_id.is_none() {
            return Err(KnowledgeOriginError::SourceVersionRequiresSource);
        }
        if self.locator.is_some() && self.reference_id.is_none() {
            return Err(KnowledgeOriginError::LocatorRequiresReference);
        }
        if let Some(version) = &self.source_version {
            validate_label(version, "source version")?;
        }
        if let Some(locator) = &self.locator {
            validate_label(locator, "source locator")?;
        }
        Ok(())
    }
}

/// Structural failures while constructing source-origin metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KnowledgeOriginError {
    /// A known origin must identify either a source or a generic reference.
    RequiresSourceOrReference,
    /// Source-version metadata cannot exist without a source identity.
    SourceVersionRequiresSource,
    /// A locator must be scoped to a generic source reference.
    LocatorRequiresReference,
    /// A source-version or locator label was empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// A source-version or locator label contained a control character.
    ControlCharacter { field: &'static str, index: usize },
}

impl fmt::Display for KnowledgeOriginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiresSourceOrReference => {
                formatter.write_str("knowledge origin must identify a source or reference")
            }
            Self::SourceVersionRequiresSource => {
                formatter.write_str("source-version metadata requires a source identity")
            }
            Self::LocatorRequiresReference => {
                formatter.write_str("a source locator requires a reference identity")
            }
            Self::EmptyLabel { field } => write!(formatter, "{field} must not be empty"),
            Self::ControlCharacter { field, index } => {
                write!(
                    formatter,
                    "{field} contains a control character at index {index}"
                )
            }
        }
    }
}

impl std::error::Error for KnowledgeOriginError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), KnowledgeOriginError> {
    if value.trim().is_empty() {
        return Err(KnowledgeOriginError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(KnowledgeOriginError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{KnowledgeOrigin, KnowledgeOriginError};
    use crate::identity::{ReferenceId, SourceId};

    #[test]
    fn source_and_source_version_origins_preserve_their_typed_identifiers() {
        let source_id = SourceId::new("source-hadith").expect("valid source identity");
        let source = KnowledgeOrigin::from_source(source_id.clone());
        let version = KnowledgeOrigin::source_version(source_id.clone(), "edition-2024")
            .expect("valid source version");

        assert_eq!(source.source_id(), Some(&source_id));
        assert_eq!(source.source_version_label(), None);
        assert_eq!(version.source_id(), Some(&source_id));
        assert_eq!(version.source_version_label(), Some("edition-2024"));
    }

    #[test]
    fn references_and_locators_remain_structured_and_generic() {
        let reference_id = ReferenceId::new("reference-book-1").expect("valid reference");
        let origin =
            KnowledgeOrigin::source_location(None, reference_id.clone(), "volume-2/page-7")
                .expect("valid reference locator");

        assert_eq!(origin.reference_id(), Some(&reference_id));
        assert_eq!(origin.locator(), Some("volume-2/page-7"));
    }

    #[test]
    fn invalid_source_version_combinations_are_rejected() {
        assert_eq!(
            KnowledgeOrigin::new(None, Some("v1".to_owned()), None, None),
            Err(KnowledgeOriginError::RequiresSourceOrReference)
        );
        assert!(KnowledgeOrigin::source_version(SourceId::new("source-1").unwrap(), "  ").is_err());
    }

    #[test]
    fn unknown_origin_is_explicit_and_distinct_from_source_origin() {
        let unknown = KnowledgeOrigin::unknown();
        let known = KnowledgeOrigin::from_source(SourceId::new("source-1").unwrap());

        assert!(unknown.is_unknown());
        assert!(!known.is_unknown());
        assert!(unknown.validate().is_ok());
    }
}
