//! Canonical knowledge-assertion semantic model.
//!
//! This module owns `KnowledgeAssertion`, its canonical semantic identity,
//! structural validation, and identity-based equality.
//!
//! The assertion model deliberately stops at Phase 2:
//! - no ontology validation;
//! - no domain/range validation;
//! - no evidence/provenance/authority system;
//! - no reasoning;
//! - no persistence;
//! - no graph traversal.
//!
//! `KnowledgeAssertionId` remains the identity type established by the KG
//! identity boundary. Canonical construction delegates to the deterministic
//! constructor provided by `crate::identity::KnowledgeAssertionId`.

use core::fmt;

use crate::assertion::context::AssertionContext;
use crate::assertion::object::AssertionObject;
use crate::assertion::predicate::AssertionPredicate;
use crate::assertion::qualifier::Qualifiers;
use crate::assertion::status::AssertionStatus;
use crate::identity::KnowledgeAssertionId;

/// Semantic polarity of a knowledge assertion.
///
/// The absence of an assertion remains distinct from a negative assertion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub enum AssertionPolarity {
    /// The assertion expresses a positive proposition.
    #[default]
    Positive,

    /// The assertion explicitly expresses a negative proposition.
    Negative,
}

impl AssertionPolarity {
    /// Returns the stable canonical textual representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }
}

impl fmt::Display for AssertionPolarity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Structural validation failures for a knowledge assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KnowledgeAssertionValidationError {
    /// The stored assertion identity does not match the canonical semantic
    /// representation of the assertion.
    NonCanonicalIdentity {
        /// The identity currently stored on the assertion.
        actual: KnowledgeAssertionId,

        /// The identity that canonical construction requires.
        expected: KnowledgeAssertionId,
    },
}

impl fmt::Display for KnowledgeAssertionValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonCanonicalIdentity { actual, expected } => write!(
                formatter,
                "knowledge assertion identity is not canonical: actual={actual}, expected={expected}",
            ),
        }
    }
}

impl std::error::Error for KnowledgeAssertionValidationError {}

/// A first-class semantic knowledge assertion.
///
/// Its semantic identity is derived deterministically from:
///
/// ```text
/// subject
/// predicate
/// object
/// canonical context
/// canonical qualifiers
/// polarity
/// ```
///
/// Epistemic status is intentionally excluded from semantic identity because
/// status represents the current epistemic state of an assertion rather than
/// the proposition itself.
#[derive(Clone, Debug, Eq)]
pub struct KnowledgeAssertion {
    id: KnowledgeAssertionId,
    subject: AssertionObject,
    predicate: AssertionPredicate,
    object: AssertionObject,
    context: AssertionContext,
    qualifiers: Qualifiers,
    status: AssertionStatus,
    polarity: AssertionPolarity,
}

impl KnowledgeAssertion {
    /// Creates a canonical knowledge assertion.
    ///
    /// The identity is derived from the canonical semantic representation.
    ///
    /// Because all component types have already passed their own structural
    /// validation, canonical identity construction cannot be empty.
    pub fn new(
        subject: AssertionObject,
        predicate: AssertionPredicate,
        object: AssertionObject,
        context: AssertionContext,
        qualifiers: Qualifiers,
        status: AssertionStatus,
        polarity: AssertionPolarity,
    ) -> Self {
        let canonical = Self::canonical_representation(
            &subject,
            &predicate,
            &object,
            &context,
            &qualifiers,
            polarity,
        );

        let id = KnowledgeAssertionId::from_canonical(canonical)
            .expect("canonical knowledge assertion representation cannot be empty");

        Self {
            id,
            subject,
            predicate,
            object,
            context,
            qualifiers,
            status,
            polarity,
        }
    }

    /// Returns the deterministic assertion identity.
    #[must_use]
    pub fn id(&self) -> &KnowledgeAssertionId {
        &self.id
    }

    /// Returns the assertion subject.
    #[must_use]
    pub fn subject(&self) -> &AssertionObject {
        &self.subject
    }

    /// Returns the assertion predicate.
    #[must_use]
    pub fn predicate(&self) -> &AssertionPredicate {
        &self.predicate
    }

    /// Returns the assertion object.
    #[must_use]
    pub fn object(&self) -> &AssertionObject {
        &self.object
    }

    /// Returns the assertion context.
    #[must_use]
    pub fn context(&self) -> &AssertionContext {
        &self.context
    }

    /// Returns the assertion qualifiers.
    #[must_use]
    pub fn qualifiers(&self) -> &Qualifiers {
        &self.qualifiers
    }

    /// Returns the assertion epistemic status.
    #[must_use]
    pub fn status(&self) -> AssertionStatus {
        self.status
    }

    /// Returns the assertion polarity.
    #[must_use]
    pub fn polarity(&self) -> AssertionPolarity {
        self.polarity
    }

    /// Validates the assertion's structural identity.
    ///
    /// The semantic components are typed and validated by their respective
    /// modules. The remaining structural invariant is that the stored identity
    /// must equal the identity produced from the canonical semantic form.
    pub fn validate(&self) -> Result<(), KnowledgeAssertionValidationError> {
        let expected = self.canonical_id();

        if self.id != expected {
            return Err(KnowledgeAssertionValidationError::NonCanonicalIdentity {
                actual: self.id.clone(),
                expected,
            });
        }

        Ok(())
    }

    /// Returns the canonical semantic representation used for assertion
    /// identity.
    ///
    /// This method is intentionally crate-private. The canonical format is an
    /// implementation contract of the assertion model rather than a public
    /// serialization format.
    pub(crate) fn canonical_representation(
        subject: &AssertionObject,
        predicate: &AssertionPredicate,
        object: &AssertionObject,
        context: &AssertionContext,
        qualifiers: &Qualifiers,
        polarity: AssertionPolarity,
    ) -> String {
        let mut output = String::new();

        Self::push_component(&mut output, "subject", Self::canonical_object(subject));

        Self::push_component(&mut output, "predicate", predicate.as_str().to_owned());

        Self::push_component(&mut output, "object", Self::canonical_object(object));

        let mut context_value = String::new();

        for (key, value) in context.iter() {
            Self::push_component(&mut context_value, "key", key.to_owned());
            Self::push_component(&mut context_value, "value", value.to_owned());
        }

        Self::push_component(&mut output, "context", context_value);

        let mut qualifiers_value = String::new();

        for qualifier in qualifiers.iter() {
            Self::push_component(&mut qualifiers_value, "key", qualifier.key().to_owned());
            Self::push_component(&mut qualifiers_value, "value", qualifier.value().to_owned());
        }

        Self::push_component(&mut output, "qualifiers", qualifiers_value);

        Self::push_component(&mut output, "polarity", polarity.as_str().to_owned());

        output
    }

    fn canonical_id(&self) -> KnowledgeAssertionId {
        let canonical = Self::canonical_representation(
            &self.subject,
            &self.predicate,
            &self.object,
            &self.context,
            &self.qualifiers,
            self.polarity,
        );

        KnowledgeAssertionId::from_canonical(canonical)
            .expect("canonical knowledge assertion representation cannot be empty")
    }

    fn canonical_object(object: &AssertionObject) -> String {
        match object {
            AssertionObject::Entity(id) => {
                format!("entity:{}", id.as_str())
            }
            AssertionObject::Concept(id) => {
                format!("concept:{}", id.as_str())
            }
            AssertionObject::Source(id) => {
                format!("source:{}", id.as_str())
            }
            AssertionObject::Reference(id) => {
                format!("reference:{}", id.as_str())
            }
            AssertionObject::LexicalForm(id) => {
                format!("lexical-form:{}", id.as_str())
            }
            AssertionObject::Mention(id) => {
                format!("mention:{}", id.as_str())
            }
        }
    }

    fn push_component(output: &mut String, field: &str, value: String) {
        output.push_str(field);
        output.push(':');
        output.push_str(&value.len().to_string());
        output.push(':');
        output.push_str(&value);
        output.push('|');
    }
}

impl PartialEq for KnowledgeAssertion {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for KnowledgeAssertion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::{AssertionPolarity, KnowledgeAssertion};
    use crate::assertion::context::AssertionContext;
    use crate::assertion::object::AssertionObject;
    use crate::assertion::qualifier::{Qualifier, Qualifiers};
    use crate::assertion::status::AssertionStatus;
    use crate::identity::EntityId;

    fn entity(value: &str) -> EntityId {
        EntityId::new(value).expect("valid entity identity")
    }

    fn predicate() -> crate::assertion::predicate::AssertionPredicate {
        crate::assertion::predicate::AssertionPredicate::new("has-name").expect("valid predicate")
    }

    #[test]
    fn constructs_a_canonical_knowledge_assertion() {
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert!(!assertion.id().as_str().is_empty());
        assert_eq!(assertion.polarity(), AssertionPolarity::Positive);
        assert_eq!(assertion.status(), AssertionStatus::Accepted);
        assert!(assertion.validate().is_ok());
    }

    #[test]
    fn identical_semantic_assertions_receive_the_same_identity() {
        let first = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let second = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        );

        assert_eq!(first.id(), second.id());
        assert_eq!(first, second);
    }

    #[test]
    fn different_context_produces_a_different_identity() {
        let first = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let mut context = AssertionContext::new();
        context.insert("source", "quran").expect("valid context");

        let second = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            context,
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn qualifier_order_does_not_change_identity() {
        let first_qualifier = Qualifier::new("a", "one").expect("valid qualifier");
        let second_qualifier = Qualifier::new("b", "two").expect("valid qualifier");

        let first = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::from_iter([first_qualifier.clone(), second_qualifier.clone()]),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let second = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::from_iter([second_qualifier, first_qualifier]),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn negative_and_positive_assertions_have_distinct_identities() {
        let positive = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let negative = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Negative,
        );

        assert_ne!(positive.id(), negative.id());
        assert_ne!(positive, negative);
    }

    #[test]
    fn cross_dimensional_assertions_are_supported() {
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Mention(
                crate::identity::MentionId::new("mention-1").expect("valid mention identity"),
            ),
            crate::assertion::predicate::AssertionPredicate::new("refers-to")
                .expect("valid predicate"),
            AssertionObject::Entity(entity("entity-1")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        );

        assert!(assertion.validate().is_ok());
    }

    #[test]
    fn structural_validation_accepts_a_canonical_assertion() {
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            predicate(),
            AssertionObject::Entity(entity("entity-2")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert!(assertion.validate().is_ok());
    }

    #[test]
    fn canonical_representation_preserves_semantic_types() {
        let entity = AssertionObject::Entity(entity("same"));
        let concept = AssertionObject::Concept(
            crate::identity::ConceptId::new("same").expect("valid concept identity"),
        );

        let context = AssertionContext::new();
        let qualifiers = Qualifiers::new();
        let predicate = predicate();

        let entity_representation = KnowledgeAssertion::canonical_representation(
            &entity,
            &predicate,
            &concept,
            &context,
            &qualifiers,
            AssertionPolarity::Positive,
        );

        let concept_representation = KnowledgeAssertion::canonical_representation(
            &concept,
            &predicate,
            &entity,
            &context,
            &qualifiers,
            AssertionPolarity::Positive,
        );

        assert_ne!(entity_representation, concept_representation);
    }
}
