//! Typed semantic references used by knowledge assertions.
//!
//! `AssertionObject` represents a reference to one of the semantic object
//! categories currently established by the Knowledge Graph Engine.
//!
//! The same type is used for both the `subject` and `object` positions of a
//! `KnowledgeAssertion`.
//!
//! The representation is intentionally a closed enum in Phase 2. Later phases
//! may add new variants when new first-class KG semantic object categories are
//! introduced.
//!
//! This module does not create identities, perform semantic validation, resolve
//! entities, or interpret ontology/domain constraints. It only preserves the
//! relationship between a semantic-object category and its strongly typed Core
//! identity.

use crate::identity::{ConceptId, EntityId, LexicalFormId, MentionId, ReferenceId, SourceId};

/// A strongly typed reference to a Phase 1 Knowledge Graph semantic object.
///
/// `AssertionObject` is used for both assertion subjects and assertion
/// objects. Each enum variant is permanently paired with its corresponding
/// semantic identity type, preventing incompatible type/identity
/// combinations at compile time.
///
/// The enum is intentionally closed for Phase 2. Additional semantic-object
/// categories may be introduced by later phases as explicit new variants.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssertionObject {
    /// Reference to an `Entity`.
    Entity(EntityId),

    /// Reference to a `Concept`.
    Concept(ConceptId),

    /// Reference to a `Source`.
    Source(SourceId),

    /// Reference to a `Reference`.
    Reference(ReferenceId),

    /// Reference to a `LexicalForm`.
    LexicalForm(LexicalFormId),

    /// Reference to a `Mention`.
    Mention(MentionId),
}

impl AssertionObject {
    /// Creates an entity reference.
    #[must_use]
    pub fn entity(id: EntityId) -> Self {
        Self::Entity(id)
    }

    /// Creates a concept reference.
    #[must_use]
    pub fn concept(id: ConceptId) -> Self {
        Self::Concept(id)
    }

    /// Creates a source reference.
    #[must_use]
    pub fn source(id: SourceId) -> Self {
        Self::Source(id)
    }

    /// Creates a reference-object reference.
    #[must_use]
    pub fn reference(id: ReferenceId) -> Self {
        Self::Reference(id)
    }

    /// Creates a lexical-form reference.
    #[must_use]
    pub fn lexical_form(id: LexicalFormId) -> Self {
        Self::LexicalForm(id)
    }

    /// Creates a mention reference.
    #[must_use]
    pub fn mention(id: MentionId) -> Self {
        Self::Mention(id)
    }
}

impl From<EntityId> for AssertionObject {
    fn from(id: EntityId) -> Self {
        Self::Entity(id)
    }
}

impl From<ConceptId> for AssertionObject {
    fn from(id: ConceptId) -> Self {
        Self::Concept(id)
    }
}

impl From<SourceId> for AssertionObject {
    fn from(id: SourceId) -> Self {
        Self::Source(id)
    }
}

impl From<ReferenceId> for AssertionObject {
    fn from(id: ReferenceId) -> Self {
        Self::Reference(id)
    }
}

impl From<LexicalFormId> for AssertionObject {
    fn from(id: LexicalFormId) -> Self {
        Self::LexicalForm(id)
    }
}

impl From<MentionId> for AssertionObject {
    fn from(id: MentionId) -> Self {
        Self::Mention(id)
    }
}

#[cfg(test)]
mod tests {
    use super::AssertionObject;
    use crate::identity::{ConceptId, EntityId, LexicalFormId, MentionId, ReferenceId, SourceId};

    #[test]
    fn entity_identity_is_preserved_as_entity_variant() {
        let id = EntityId::new("entity-1").expect("valid entity identity");
        let object = AssertionObject::entity(id.clone());

        assert_eq!(object, AssertionObject::Entity(id));
    }

    #[test]
    fn concept_identity_is_preserved_as_concept_variant() {
        let id = ConceptId::new("concept-1").expect("valid concept identity");
        let object = AssertionObject::concept(id.clone());

        assert_eq!(object, AssertionObject::Concept(id));
    }

    #[test]
    fn source_identity_is_preserved_as_source_variant() {
        let id = SourceId::new("source-1").expect("valid source identity");
        let object = AssertionObject::source(id.clone());

        assert_eq!(object, AssertionObject::Source(id));
    }

    #[test]
    fn reference_identity_is_preserved_as_reference_variant() {
        let id = ReferenceId::new("reference-1").expect("valid reference identity");
        let object = AssertionObject::reference(id.clone());

        assert_eq!(object, AssertionObject::Reference(id));
    }

    #[test]
    fn lexical_form_identity_is_preserved_as_lexical_form_variant() {
        let id = LexicalFormId::new("lexical-form-1").expect("valid lexical-form identity");
        let object = AssertionObject::lexical_form(id.clone());

        assert_eq!(object, AssertionObject::LexicalForm(id));
    }

    #[test]
    fn mention_identity_is_preserved_as_mention_variant() {
        let id = MentionId::new("mention-1").expect("valid mention identity");
        let object = AssertionObject::mention(id.clone());

        assert_eq!(object, AssertionObject::Mention(id));
    }

    #[test]
    fn same_identity_value_in_different_variants_remains_distinct() {
        let entity_id = EntityId::new("shared-value").expect("valid entity identity");
        let concept_id = ConceptId::new("shared-value").expect("valid concept identity");

        let entity = AssertionObject::Entity(entity_id);
        let concept = AssertionObject::Concept(concept_id);

        assert_ne!(entity, concept);
    }

    #[test]
    fn from_conversions_preserve_the_correct_variant() {
        let entity_id = EntityId::new("entity-1").expect("valid entity identity");
        let concept_id = ConceptId::new("concept-1").expect("valid concept identity");
        let source_id = SourceId::new("source-1").expect("valid source identity");
        let reference_id = ReferenceId::new("reference-1").expect("valid reference identity");
        let lexical_form_id =
            LexicalFormId::new("lexical-form-1").expect("valid lexical-form identity");
        let mention_id = MentionId::new("mention-1").expect("valid mention identity");

        assert_eq!(
            AssertionObject::from(entity_id.clone()),
            AssertionObject::Entity(entity_id)
        );

        assert_eq!(
            AssertionObject::from(concept_id.clone()),
            AssertionObject::Concept(concept_id)
        );

        assert_eq!(
            AssertionObject::from(source_id.clone()),
            AssertionObject::Source(source_id)
        );

        assert_eq!(
            AssertionObject::from(reference_id.clone()),
            AssertionObject::Reference(reference_id)
        );

        assert_eq!(
            AssertionObject::from(lexical_form_id.clone()),
            AssertionObject::LexicalForm(lexical_form_id)
        );

        assert_eq!(
            AssertionObject::from(mention_id.clone()),
            AssertionObject::Mention(mention_id)
        );
    }
}
