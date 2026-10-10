//! Minimal knowledge-graph source representation for Phase 1.
//!
//! A source is a minimal semantic representation of an origin from which
//! knowledge may be derived. Phase 4 adds typed authority metadata while leaving
//! provenance, evidence, document hierarchy, ingestion, lookup, and persistence
//! to their owning modules or later phases.

use core::fmt;

use crate::authority::{Authority, AuthorityTarget};
use crate::identity::SourceId;

/// A minimal identifiable knowledge-graph source.
///
/// The source label is intentionally opaque identifying metadata. More
/// detailed source semantics belong to later phases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    id: SourceId,
    label: String,
    authority: Option<Authority>,
}

impl Source {
    /// Constructs a source from its identity and minimal identifying label.
    #[must_use]
    pub fn new(id: SourceId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            authority: None,
        }
    }

    /// Returns the source identity.
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Returns the opaque identifying label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns authority metadata attached to this source, if any.
    #[must_use]
    pub fn authority(&self) -> Option<&Authority> {
        self.authority.as_ref()
    }

    /// Attaches authority metadata whose target must match this source identity.
    pub fn with_authority(mut self, authority: Authority) -> Result<Self, SourceError> {
        let expected = AuthorityTarget::Source(self.id.clone());
        let actual = authority.target().clone();
        if actual != expected {
            return Err(SourceError::AuthorityTargetMismatch { expected, actual });
        }

        self.authority = Some(authority);
        Ok(self)
    }

    /// Returns a new source value without attached authority metadata.
    #[must_use]
    pub fn without_authority(mut self) -> Self {
        self.authority = None;
        self
    }

    /// Validates source-level structural metadata.
    pub fn validate(&self) -> Result<(), SourceError> {
        if let Some(authority) = &self.authority {
            let expected = AuthorityTarget::Source(self.id.clone());
            let actual = authority.target().clone();
            if actual != expected {
                return Err(SourceError::AuthorityTargetMismatch { expected, actual });
            }
        }

        Ok(())
    }
}

/// Structural errors while attaching authority metadata to a source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceError {
    /// Authority metadata refers to a different source or assertion.
    AuthorityTargetMismatch {
        /// The source target required by this record.
        expected: AuthorityTarget,
        /// The actual target declared by the attached authority metadata.
        actual: AuthorityTarget,
    },
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorityTargetMismatch { expected, actual } => write!(
                formatter,
                "source authority target does not match: expected={expected:?}, actual={actual:?}",
            ),
        }
    }
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::Source;
    use crate::identity::SourceId;

    #[test]
    fn source_preserves_identity_and_label() {
        let id = SourceId::generate();
        let source = Source::new(id.clone(), "Example source");

        assert_eq!(source.id(), &id);
        assert_eq!(source.label(), "Example source");
    }

    #[test]
    fn source_accepts_multilingual_identifying_metadata() {
        let arabic = Source::new(SourceId::generate(), "مصدر");
        let english = Source::new(SourceId::generate(), "Source");
        let urdu = Source::new(SourceId::generate(), "ماخذ");

        assert_eq!(arabic.label(), "مصدر");
        assert_eq!(english.label(), "Source");
        assert_eq!(urdu.label(), "ماخذ");
    }

    #[test]
    fn source_generation_produces_distinct_identities() {
        let first = SourceId::generate();
        let second = SourceId::generate();

        assert_ne!(first, second);
    }

    #[test]
    fn source_accepts_authority_only_for_its_own_typed_identity() {
        use crate::authority::{Authority, AuthorityDimension, AuthorityTarget, AuthorityValue};

        let id = SourceId::new("source-authority-target").expect("valid source identity");
        let authority = Authority::new(AuthorityTarget::Source(id.clone()))
            .with_dimension(AuthorityDimension::SourceAuthority(
                AuthorityValue::new("primary-source").expect("valid authority value"),
            ))
            .expect("valid authority dimension");
        let source = Source::new(id.clone(), "Primary source")
            .with_authority(authority)
            .expect("matching source authority target");

        assert_eq!(source.id(), &id);
        assert!(source.authority().is_some());
        assert!(source.validate().is_ok());
    }

    #[test]
    fn source_rejects_authority_targeted_at_another_object() {
        use crate::authority::{Authority, AuthorityTarget};
        use crate::identity::KnowledgeAssertionId;

        let source = Source::new(
            SourceId::new("source-wrong-authority").expect("valid source identity"),
            "Source",
        );
        let authority = Authority::new(AuthorityTarget::Assertion(
            KnowledgeAssertionId::new("assertion-not-source").expect("valid assertion identity"),
        ));

        assert!(source.with_authority(authority).is_err());
    }
}
