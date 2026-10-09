//! Interpretation and semantic-connection foundations for Phase 3.
//!
//! Interpretation is kept distinct from canonical semantic objects. A semantic
//! relation wraps the existing Phase 2 `KnowledgeAssertion` rather than
//! creating a second assertion or relationship system.

use std::fmt;

use crate::assertion::KnowledgeAssertion;
use crate::identity::{LexicalFormId, MentionId, ReferenceId, SourceId};

use super::context::Context;

/// An opaque reference to a lexical sense.
///
/// Phase 1 did not establish a `SenseId`. This reference therefore deliberately
/// does not introduce a second identity framework. It is an integration seam
/// for a future concrete lexical-sense model owned by the lexical subsystem.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SenseReference(String);

impl SenseReference {
    /// Creates an opaque sense reference.
    pub fn new(value: impl Into<String>) -> Result<Self, SenseReferenceError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SenseReferenceError::Empty);
        }
        if let Some(index) = value.chars().position(char::is_control) {
            return Err(SenseReferenceError::ControlCharacter { index });
        }
        Ok(Self(value))
    }

    /// Returns the opaque sense reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validation failures for an opaque sense reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SenseReferenceError {
    /// The reference is empty or whitespace-only.
    Empty,

    /// The reference contains a Unicode control character.
    ControlCharacter { index: usize },
}

impl fmt::Display for SenseReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("sense reference must not be empty"),
            Self::ControlCharacter { index } => write!(
                formatter,
                "sense reference contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for SenseReferenceError {}

/// Source material that an interpretation describes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InterpretationSource {
    /// A lexical-form representation.
    LexicalForm(LexicalFormId),

    /// A lexical sense reference supplied by a linguistic/lexical subsystem.
    Sense(SenseReference),

    /// A source-level reference.
    Reference(ReferenceId),

    /// A source document/passage identifier.
    Source(SourceId),

    /// A mention occurring in source material.
    Mention(MentionId),
}

/// A first-class interpretation of source or linguistic material in context.
///
/// Meaning is deliberately not embedded here. The selected Phase 3
/// architecture keeps `Meaning` structurally independent and uses semantic
/// assertions/connections to relate it to other semantic objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interpretation {
    source: InterpretationSource,
    context: Context,
}

impl Interpretation {
    /// Creates an interpretation from source material and semantic context.
    #[must_use]
    pub fn new(source: InterpretationSource, context: Context) -> Self {
        Self { source, context }
    }

    /// Returns the interpreted source material.
    #[must_use]
    pub fn source(&self) -> &InterpretationSource {
        &self.source
    }

    /// Returns the semantic context of the interpretation.
    #[must_use]
    pub fn context(&self) -> &Context {
        &self.context
    }
}

/// A semantic connection backed by an existing Phase 2 knowledge assertion.
///
/// This is intentionally a wrapper/value object. It does not create a new
/// assertion identity, predicate vocabulary, or relationship storage model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticRelation {
    assertion: KnowledgeAssertion,
}

impl SemanticRelation {
    /// Wraps an existing knowledge assertion.
    #[must_use]
    pub fn new(assertion: KnowledgeAssertion) -> Self {
        Self { assertion }
    }

    /// Returns the wrapped assertion.
    #[must_use]
    pub fn assertion(&self) -> &KnowledgeAssertion {
        &self.assertion
    }

    /// Consumes the wrapper and returns the underlying assertion.
    #[must_use]
    pub fn into_assertion(self) -> KnowledgeAssertion {
        self.assertion
    }
}

/// Describes whether a lexical-to-concept mapping is direct or mediated by a
/// lexical sense.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LexicalConceptMappingKind {
    /// `LexicalForm → Concept` without an intermediate sense reference.
    Direct,

    /// `LexicalForm → Sense → Concept`, where the sense is referenced by an
    /// opaque lexical-subsystem value.
    SenseMediated {
        /// Opaque reference to the mediating sense.
        sense: SenseReference,
    },
}

/// A lexical-to-concept semantic relation that wraps a `KnowledgeAssertion`.
///
/// The wrapped assertion remains the authoritative semantic relationship. The
/// mapping kind is additional semantic metadata and therefore does not create
/// a competing mapping/assertion system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexicalConceptMapping {
    relation: SemanticRelation,
    kind: LexicalConceptMappingKind,
}

/// Validation failures for a lexical-to-concept semantic relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LexicalConceptMappingError {
    /// The wrapped assertion does not have a lexical form as its subject.
    InvalidSubject,

    /// The wrapped assertion does not have a concept as its object.
    InvalidObject,
}

impl fmt::Display for LexicalConceptMappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSubject => formatter.write_str(
                "lexical-to-concept mapping assertion must use a lexical form as its subject",
            ),
            Self::InvalidObject => formatter
                .write_str("lexical-to-concept mapping assertion must use a concept as its object"),
        }
    }
}

impl std::error::Error for LexicalConceptMappingError {}

impl LexicalConceptMapping {
    /// Creates a direct lexical-to-concept mapping around an existing
    /// assertion.
    pub fn direct(assertion: KnowledgeAssertion) -> Result<Self, LexicalConceptMappingError> {
        Self::new(assertion, LexicalConceptMappingKind::Direct)
    }

    /// Creates a Sense-mediated lexical-to-concept mapping around an existing
    /// assertion.
    pub fn sense_mediated(
        assertion: KnowledgeAssertion,
        sense: SenseReference,
    ) -> Result<Self, LexicalConceptMappingError> {
        Self::new(
            assertion,
            LexicalConceptMappingKind::SenseMediated { sense },
        )
    }

    fn new(
        assertion: KnowledgeAssertion,
        kind: LexicalConceptMappingKind,
    ) -> Result<Self, LexicalConceptMappingError> {
        if !matches!(
            assertion.subject(),
            crate::assertion::AssertionObject::LexicalForm(_)
        ) {
            return Err(LexicalConceptMappingError::InvalidSubject);
        }

        if !matches!(
            assertion.object(),
            crate::assertion::AssertionObject::Concept(_)
        ) {
            return Err(LexicalConceptMappingError::InvalidObject);
        }

        Ok(Self {
            relation: SemanticRelation::new(assertion),
            kind,
        })
    }

    /// Returns the underlying semantic relation wrapper.
    #[must_use]
    pub fn relation(&self) -> &SemanticRelation {
        &self.relation
    }

    /// Returns the wrapped knowledge assertion.
    #[must_use]
    pub fn assertion(&self) -> &KnowledgeAssertion {
        self.relation.assertion()
    }

    /// Returns whether this mapping is direct or Sense-mediated.
    #[must_use]
    pub fn kind(&self) -> &LexicalConceptMappingKind {
        &self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Interpretation, InterpretationSource, LexicalConceptMapping, LexicalConceptMappingError,
        LexicalConceptMappingKind, SemanticRelation, SenseReference,
    };
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionStatus, KnowledgeAssertion,
        Qualifiers,
    };
    use crate::identity::{ConceptId, LexicalFormId};
    use crate::relationship::RelationshipPredicate;
    use crate::semantics::Context;

    fn mapping_assertion() -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::LexicalForm(
                LexicalFormId::new("lexical-sabr").expect("valid lexical identity"),
            ),
            RelationshipPredicate::new("expresses").expect("valid relationship predicate"),
            AssertionObject::Concept(
                ConceptId::new("concept-sabr").expect("valid concept identity"),
            ),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    #[test]
    fn interpretation_is_first_class_and_preserves_source_context() {
        let interpretation = Interpretation::new(
            InterpretationSource::LexicalForm(
                LexicalFormId::new("lexical-sabr").expect("valid lexical identity"),
            ),
            Context::ephemeral(),
        );

        assert!(matches!(
            interpretation.source(),
            InterpretationSource::LexicalForm(_)
        ));
        assert!(interpretation.context().id().is_none());
    }

    #[test]
    fn semantic_relation_wraps_existing_assertion_without_replacing_it() {
        let assertion = mapping_assertion();
        let relation = SemanticRelation::new(assertion.clone());

        assert_eq!(relation.assertion(), &assertion);
        assert_eq!(relation.clone().into_assertion(), assertion);
    }

    #[test]
    fn direct_lexical_to_concept_mapping_is_supported() {
        let mapping = LexicalConceptMapping::direct(mapping_assertion())
            .expect("valid direct lexical-to-concept mapping");

        assert_eq!(mapping.kind(), &LexicalConceptMappingKind::Direct);
        assert!(matches!(
            mapping.assertion().subject(),
            AssertionObject::LexicalForm(_)
        ));
        assert!(matches!(
            mapping.assertion().object(),
            AssertionObject::Concept(_)
        ));
    }

    #[test]
    fn sense_mediated_mapping_preserves_the_sense_reference_as_metadata() {
        let sense = SenseReference::new("sense-sabr-1").expect("valid sense reference");
        let mapping = LexicalConceptMapping::sense_mediated(mapping_assertion(), sense.clone())
            .expect("valid Sense-mediated mapping");

        assert_eq!(
            mapping.kind(),
            &LexicalConceptMappingKind::SenseMediated { sense }
        );
    }

    #[test]
    fn invalid_mapping_subject_is_rejected() {
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Concept(ConceptId::new("concept-a").expect("valid concept identity")),
            RelationshipPredicate::new("expresses").expect("valid predicate"),
            AssertionObject::Concept(ConceptId::new("concept-b").expect("valid concept identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert_eq!(
            LexicalConceptMapping::direct(assertion),
            Err(LexicalConceptMappingError::InvalidSubject)
        );
    }

    #[test]
    fn invalid_mapping_object_is_rejected() {
        let assertion = KnowledgeAssertion::new(
            AssertionObject::LexicalForm(
                LexicalFormId::new("lexical-a").expect("valid lexical identity"),
            ),
            RelationshipPredicate::new("expresses").expect("valid predicate"),
            AssertionObject::LexicalForm(
                LexicalFormId::new("lexical-b").expect("valid lexical identity"),
            ),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert_eq!(
            LexicalConceptMapping::direct(assertion),
            Err(LexicalConceptMappingError::InvalidObject)
        );
    }
}
