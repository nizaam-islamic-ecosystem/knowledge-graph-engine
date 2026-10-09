//! Semantic typing foundations for Phase 3.
//!
//! Semantic typing supports both ontology classes and semantic concepts while
//! keeping those targets strongly distinct. The type model is a structural
//! representation; full ontology semantic validation and inference remain
//! outside this module.

use std::collections::BTreeSet;
use std::fmt;

use crate::identity::ConceptId;
use crate::ontology::ClassId;

/// A semantic type target can refer to either a formal ontology class or an
/// existing semantic concept.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticTypeTarget {
    /// Formal ontology class target.
    Class(ClassId),

    /// Existing semantic concept target.
    Concept(ConceptId),
}

impl SemanticTypeTarget {
    /// Validates the target identity representation.
    ///
    /// Core identity types are already strongly typed, so Phase 3 only needs
    /// to guard the representation boundary here. An empty rendered identity
    /// is rejected rather than being treated as a valid semantic target.
    pub fn validate(&self) -> Result<(), SemanticTypeError> {
        let identity = match self {
            Self::Class(class_id) => class_id.to_string(),
            Self::Concept(concept_id) => concept_id.to_string(),
        };

        if identity.is_empty() {
            return Err(SemanticTypeError::InvalidTarget);
        }

        Ok(())
    }
}

/// A semantic type declaration.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticType {
    target: SemanticTypeTarget,
}

impl SemanticType {
    /// Creates a class-backed semantic type.
    #[must_use]
    pub fn class(class: ClassId) -> Self {
        Self {
            target: SemanticTypeTarget::Class(class),
        }
    }

    /// Creates a concept-backed semantic type.
    #[must_use]
    pub fn concept(concept: ConceptId) -> Self {
        Self {
            target: SemanticTypeTarget::Concept(concept),
        }
    }

    /// Creates a class-backed semantic type after validating its target.
    pub fn try_class(class: ClassId) -> Result<Self, SemanticTypeError> {
        let semantic_type = Self::class(class);
        semantic_type.validate()?;
        Ok(semantic_type)
    }

    /// Creates a concept-backed semantic type after validating its target.
    pub fn try_concept(concept: ConceptId) -> Result<Self, SemanticTypeError> {
        let semantic_type = Self::concept(concept);
        semantic_type.validate()?;
        Ok(semantic_type)
    }

    /// Returns the semantic type target.
    #[must_use]
    pub fn target(&self) -> &SemanticTypeTarget {
        &self.target
    }

    /// Validates the semantic type target.
    pub fn validate(&self) -> Result<(), SemanticTypeError> {
        self.target.validate()
    }
}

/// A primary semantic type plus zero or more additional types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticTypes {
    primary: SemanticType,
    additional: BTreeSet<SemanticType>,
}

impl SemanticTypes {
    /// Creates a semantic-type set with one required primary type.
    #[must_use]
    pub fn new(primary: SemanticType) -> Self {
        Self {
            primary,
            additional: BTreeSet::new(),
        }
    }

    /// Returns the primary semantic type.
    #[must_use]
    pub fn primary(&self) -> &SemanticType {
        &self.primary
    }

    /// Adds an additional semantic type.
    ///
    /// If the supplied type is identical to the primary type, it is ignored
    /// because the primary type already represents that membership.
    pub fn add_additional(&mut self, semantic_type: SemanticType) {
        if semantic_type != self.primary {
            self.additional.insert(semantic_type);
        }
    }

    /// Returns additional semantic types in deterministic order.
    pub fn additional(&self) -> impl Iterator<Item = &SemanticType> {
        self.additional.iter()
    }

    /// Returns whether the supplied semantic type is present as primary or
    /// additional type.
    #[must_use]
    pub fn contains(&self, semantic_type: &SemanticType) -> bool {
        &self.primary == semantic_type || self.additional.contains(semantic_type)
    }
}

/// A structural type-membership declaration.
///
/// This type deliberately does not replace `KnowledgeAssertion`. It records
/// the relationship between an assertion subject and a semantic type target so
/// the semantic layer can preserve the primary/additional type distinction.
/// Once the assertion-object boundary is extended for ontology classes, the
/// declaration can be materialized as a normal Phase 2 assertion without
/// creating another assertion system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticTypeMembership {
    subject: crate::assertion::AssertionObject,
    semantic_type: SemanticType,
}

impl SemanticTypeMembership {
    /// Creates a type-membership declaration for an assertion subject.
    #[must_use]
    pub fn new(subject: crate::assertion::AssertionObject, semantic_type: SemanticType) -> Self {
        Self {
            subject,
            semantic_type,
        }
    }

    /// Creates a type-membership declaration after validating the semantic
    /// type target.
    pub fn try_new(
        subject: crate::assertion::AssertionObject,
        semantic_type: SemanticType,
    ) -> Result<Self, SemanticTypeError> {
        semantic_type.validate()?;

        Ok(Self {
            subject,
            semantic_type,
        })
    }

    /// Returns the assertion subject whose type is being described.
    #[must_use]
    pub fn subject(&self) -> &crate::assertion::AssertionObject {
        &self.subject
    }

    /// Returns the semantic type target.
    #[must_use]
    pub fn semantic_type(&self) -> &SemanticType {
        &self.semantic_type
    }
}

/// Errors for semantic-type helper validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticTypeError {
    /// A semantic type representation must have a non-empty target identity.
    InvalidTarget,
}

impl fmt::Display for SemanticTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget => formatter.write_str("semantic type target must be valid"),
        }
    }
}

impl std::error::Error for SemanticTypeError {}

#[cfg(test)]
mod tests {
    use super::{
        SemanticType, SemanticTypeError, SemanticTypeMembership, SemanticTypeTarget, SemanticTypes,
    };
    use crate::Concept;
    use crate::assertion::AssertionObject;
    use crate::identity::{ConceptId, EntityId};
    use crate::ontology::ClassId;

    #[test]
    fn semantic_type_can_target_a_class() {
        let class = ClassId::new("class-person").expect("valid class identity");
        let semantic_type = SemanticType::class(class.clone());

        assert_eq!(semantic_type.target(), &SemanticTypeTarget::Class(class));
        assert_eq!(semantic_type.validate(), Ok(()));
    }

    #[test]
    fn semantic_type_can_target_a_concept_without_collapsing_the_types() {
        let concept_id = ConceptId::new("concept-person").expect("valid concept identity");
        let semantic_type = SemanticType::concept(concept_id.clone());
        let class =
            SemanticType::class(ClassId::new("class-person").expect("valid class identity"));

        assert_eq!(
            semantic_type.target(),
            &SemanticTypeTarget::Concept(concept_id)
        );
        assert_ne!(semantic_type, class);
    }

    #[test]
    fn try_constructors_validate_the_target() {
        let class = ClassId::new("class-person").expect("valid class identity");
        let concept = ConceptId::new("concept-person").expect("valid concept identity");

        assert_eq!(
            SemanticType::try_class(class)
                .expect("valid class semantic type")
                .validate(),
            Ok(())
        );
        assert_eq!(
            SemanticType::try_concept(concept)
                .expect("valid concept semantic type")
                .validate(),
            Ok(())
        );
    }

    #[test]
    fn invalid_target_error_has_stable_display() {
        assert_eq!(
            SemanticTypeError::InvalidTarget.to_string(),
            "semantic type target must be valid"
        );
    }

    #[test]
    fn primary_and_additional_types_are_supported() {
        let primary =
            SemanticType::class(ClassId::new("class-person").expect("valid class identity"));
        let additional =
            SemanticType::class(ClassId::new("class-scholar").expect("valid class identity"));
        let mut types = SemanticTypes::new(primary.clone());

        types.add_additional(additional.clone());
        types.add_additional(primary.clone());

        assert_eq!(types.primary(), &primary);
        assert!(types.contains(&primary));
        assert!(types.contains(&additional));
        assert_eq!(types.additional().count(), 1);
    }

    #[test]
    fn type_membership_preserves_the_assertion_subject() {
        let entity =
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity"));
        let semantic_type =
            SemanticType::class(ClassId::new("class-person").expect("valid class identity"));
        let membership = SemanticTypeMembership::try_new(entity.clone(), semantic_type.clone())
            .expect("valid semantic type membership");

        assert_eq!(membership.subject(), &entity);
        assert_eq!(membership.semantic_type(), &semantic_type);
    }

    #[test]
    fn concept_and_class_type_targets_remain_distinct() {
        let class_target =
            SemanticTypeTarget::Class(ClassId::new("same-text").expect("valid class identity"));
        let concept_target = SemanticTypeTarget::Concept(
            ConceptId::new("same-text").expect("valid concept identity"),
        );

        assert_ne!(class_target, concept_target);

        let _concept = Concept::new(
            ConceptId::new("concept-1").expect("valid concept identity"),
            "person",
        );
    }
}
