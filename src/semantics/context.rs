//! Semantic context representation for Phase 3.
//!
//! Phase 2 provides an intentionally generic [`crate::assertion::AssertionContext`].
//! Phase 3 adds a semantic-context layer with explicit reusable/ephemeral
//! semantics and typed dimensions. The two types are deliberately kept
//! separate: an assertion context is part of assertion identity, while this
//! type describes reusable semantic context that later resolution and
//! interpretation systems can consume.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use nizaam_core::identity;

use crate::identity::SourceId;
use crate::ontology::ClassId;

identity!(
    /// Identifies a reusable semantic context.
    ///
    /// Ephemeral contexts intentionally do not receive a separate identity.
    ContextId
);

/// Indicates whether a semantic context is reusable or ephemeral.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContextKind {
    /// A reusable context identified by a Core-backed identity.
    Reusable(ContextId),

    /// A short-lived context represented only by its value.
    Ephemeral,
}

/// A typed semantic-context dimension.
///
/// Known dimensions receive explicit Rust representations. Additional
/// dimensions belong in [`Context::extensions`] rather than requiring the
/// semantic context model to become an untyped map.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContextDimension {
    /// Language metadata supplied by a language-aware producer.
    Language(String),

    /// Source associated with the context.
    Source(SourceId),

    /// Ontology class describing the domain of the context.
    Domain(ClassId),

    /// Intended audience or audience class expressed as opaque metadata.
    Audience(String),

    /// Time metadata. Temporal semantics are intentionally deferred to the
    /// Phase 4 temporal subsystem.
    Time(String),

    /// Location metadata. Geographic semantics are intentionally deferred.
    Location(String),

    /// Interpretive framework supplied by an external semantic/linguistic
    /// producer.
    InterpretiveFramework(String),

    /// Scholarly context metadata supplied by the caller or a later authority
    /// subsystem.
    ScholarlyContext(String),
}

impl ContextDimension {
    fn key(&self) -> ContextDimensionKey {
        match self {
            Self::Language(_) => ContextDimensionKey::Language,
            Self::Source(_) => ContextDimensionKey::Source,
            Self::Domain(_) => ContextDimensionKey::Domain,
            Self::Audience(_) => ContextDimensionKey::Audience,
            Self::Time(_) => ContextDimensionKey::Time,
            Self::Location(_) => ContextDimensionKey::Location,
            Self::InterpretiveFramework(_) => ContextDimensionKey::InterpretiveFramework,
            Self::ScholarlyContext(_) => ContextDimensionKey::ScholarlyContext,
        }
    }
}

/// Stable keys for the built-in typed context dimensions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContextDimensionKey {
    Language,
    Source,
    Domain,
    Audience,
    Time,
    Location,
    InterpretiveFramework,
    ScholarlyContext,
}

impl fmt::Display for ContextDimensionKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Language => "language",
            Self::Source => "source",
            Self::Domain => "domain",
            Self::Audience => "audience",
            Self::Time => "time",
            Self::Location => "location",
            Self::InterpretiveFramework => "interpretive-framework",
            Self::ScholarlyContext => "scholarly-context",
        })
    }
}

/// Errors produced while constructing or mutating semantic context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextError {
    /// A textual context value is empty or whitespace-only.
    EmptyValue {
        /// The dimension receiving the invalid value.
        dimension: ContextDimensionKey,
    },

    /// The same known typed dimension was supplied more than once.
    DuplicateDimension {
        /// The duplicated dimension.
        dimension: ContextDimensionKey,
    },

    /// An extension key is empty or whitespace-only.
    EmptyExtensionKey,

    /// An extension key contains a control character.
    ExtensionControlCharacter {
        /// Character index of the invalid character.
        index: usize,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyValue { dimension } => {
                write!(formatter, "context value for {dimension} must not be empty")
            }
            Self::DuplicateDimension { dimension } => {
                write!(formatter, "context dimension {dimension} was already set")
            }
            Self::EmptyExtensionKey => {
                formatter.write_str("context extension key must not be empty")
            }
            Self::ExtensionControlCharacter { index } => write!(
                formatter,
                "context extension key contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for ContextError {}

/// A reusable or ephemeral semantic context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    kind: ContextKind,
    dimensions: BTreeSet<ContextDimension>,
    extensions: BTreeMap<String, String>,
}

impl Context {
    /// Creates an ephemeral semantic context.
    #[must_use]
    pub fn ephemeral() -> Self {
        Self {
            kind: ContextKind::Ephemeral,
            dimensions: BTreeSet::new(),
            extensions: BTreeMap::new(),
        }
    }

    /// Creates a reusable semantic context from its Core-backed identity.
    #[must_use]
    pub fn reusable(id: ContextId) -> Self {
        Self {
            kind: ContextKind::Reusable(id),
            dimensions: BTreeSet::new(),
            extensions: BTreeMap::new(),
        }
    }

    /// Returns the context kind.
    #[must_use]
    pub fn kind(&self) -> &ContextKind {
        &self.kind
    }

    /// Returns the reusable context identity, if this is a reusable context.
    #[must_use]
    pub fn id(&self) -> Option<&ContextId> {
        match &self.kind {
            ContextKind::Reusable(id) => Some(id),
            ContextKind::Ephemeral => None,
        }
    }

    /// Inserts one typed known dimension.
    pub fn insert_dimension(&mut self, dimension: ContextDimension) -> Result<(), ContextError> {
        Self::validate_dimension(&dimension)?;

        let key = dimension.key();
        if self.dimensions.iter().any(|existing| existing.key() == key) {
            return Err(ContextError::DuplicateDimension { dimension: key });
        }

        self.dimensions.insert(dimension);
        Ok(())
    }

    /// Returns whether a known dimension of the supplied kind exists.
    #[must_use]
    pub fn contains_dimension(&self, key: ContextDimensionKey) -> bool {
        self.dimensions
            .iter()
            .any(|dimension| dimension.key() == key)
    }

    /// Iterates over known typed dimensions in deterministic order.
    pub fn dimensions(&self) -> impl Iterator<Item = &ContextDimension> {
        self.dimensions.iter()
    }

    /// Inserts or replaces an extension dimension.
    ///
    /// Extension dimensions are deliberately opaque key/value data. Known
    /// semantic dimensions should use [`ContextDimension`] instead.
    pub fn insert_extension(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Option<String>, ContextError> {
        let key = key.into();
        Self::validate_extension_key(&key)?;

        Ok(self.extensions.insert(key, value.into()))
    }

    /// Returns an extension value.
    #[must_use]
    pub fn extension(&self, key: &str) -> Option<&str> {
        self.extensions.get(key).map(String::as_str)
    }

    /// Iterates over extension dimensions in deterministic key order.
    pub fn extensions(&self) -> impl Iterator<Item = (&str, &str)> {
        self.extensions
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    fn validate_dimension(dimension: &ContextDimension) -> Result<(), ContextError> {
        let textual = match dimension {
            ContextDimension::Language(value)
            | ContextDimension::Audience(value)
            | ContextDimension::Time(value)
            | ContextDimension::Location(value)
            | ContextDimension::InterpretiveFramework(value)
            | ContextDimension::ScholarlyContext(value) => Some((dimension.key(), value)),
            ContextDimension::Source(_) | ContextDimension::Domain(_) => None,
        };

        if let Some((key, value)) = textual
            && value.trim().is_empty()
        {
            return Err(ContextError::EmptyValue { dimension: key });
        }

        Ok(())
    }

    fn validate_extension_key(key: &str) -> Result<(), ContextError> {
        if key.trim().is_empty() {
            return Err(ContextError::EmptyExtensionKey);
        }

        if let Some(index) = key.chars().position(char::is_control) {
            return Err(ContextError::ExtensionControlCharacter { index });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Context, ContextDimension, ContextDimensionKey, ContextError, ContextId, ContextKind,
    };
    use crate::identity::SourceId;
    use crate::ontology::ClassId;

    #[test]
    fn reusable_context_is_identity_backed() {
        let id = ContextId::new("context-quran").expect("valid context identity");
        let context = Context::reusable(id.clone());

        assert_eq!(context.id(), Some(&id));
        assert_eq!(context.kind(), &ContextKind::Reusable(id));
    }

    #[test]
    fn ephemeral_context_has_no_identity() {
        let context = Context::ephemeral();

        assert_eq!(context.id(), None);
        assert_eq!(context.kind(), &ContextKind::Ephemeral);
    }

    #[test]
    fn known_dimensions_are_typed() {
        let mut context = Context::ephemeral();
        context
            .insert_dimension(ContextDimension::Language("ar".to_owned()))
            .expect("language should be accepted");
        context
            .insert_dimension(ContextDimension::Source(
                SourceId::new("source-quran").expect("valid source identity"),
            ))
            .expect("source should be accepted");
        context
            .insert_dimension(ContextDimension::Domain(
                ClassId::new("class-religious-text").expect("valid class identity"),
            ))
            .expect("domain should be accepted");

        assert!(context.contains_dimension(ContextDimensionKey::Language));
        assert!(context.contains_dimension(ContextDimensionKey::Source));
        assert!(context.contains_dimension(ContextDimensionKey::Domain));
    }

    #[test]
    fn duplicate_known_dimensions_are_rejected() {
        let mut context = Context::ephemeral();
        context
            .insert_dimension(ContextDimension::Language("ar".to_owned()))
            .expect("first language should be accepted");

        assert_eq!(
            context.insert_dimension(ContextDimension::Language("en".to_owned())),
            Err(ContextError::DuplicateDimension {
                dimension: ContextDimensionKey::Language,
            })
        );
    }

    #[test]
    fn extensions_support_future_dimensions_without_erasing_typed_ones() {
        let mut context = Context::ephemeral();
        context
            .insert_extension("future.dimension", "value")
            .expect("valid extension");

        assert_eq!(context.extension("future.dimension"), Some("value"));
    }

    #[test]
    fn empty_textual_dimensions_are_rejected() {
        let mut context = Context::ephemeral();

        assert_eq!(
            context.insert_dimension(ContextDimension::Language("   ".to_owned())),
            Err(ContextError::EmptyValue {
                dimension: ContextDimensionKey::Language,
            })
        );
    }
}
