//! Knowledge-graph identity foundations for Phase 1.
//!
//! This module defines the seven strongly typed semantic identities established
//! by the Phase 1 scope. Identity generation, validation, serialization, and
//! comparison behavior remain owned by Nizaam Core's `identity!` mechanism.
//!
//! The semantic objects represented by these identities are intentionally kept
//! outside this module. Phase 1 establishes identity boundaries only.

mod assertion;
mod concept;
mod entity;
mod lexical;
mod mention;
mod reference;
mod source;

pub use assertion::KnowledgeAssertionId;
pub use concept::ConceptId;
pub use entity::EntityId;
pub use lexical::LexicalFormId;
pub use mention::MentionId;
pub use reference::ReferenceId;
pub use source::SourceId;

#[cfg(test)]
mod tests {
    use super::{
        ConceptId, EntityId, KnowledgeAssertionId, LexicalFormId, MentionId, ReferenceId, SourceId,
    };
    use std::any::TypeId;

    #[test]
    fn phase1_identity_types_are_all_exposed() {
        let entity = EntityId::generate();
        let concept = ConceptId::generate();
        let source = SourceId::generate();
        let reference = ReferenceId::generate();
        let lexical_form = LexicalFormId::generate();
        let mention = MentionId::generate();
        let assertion = KnowledgeAssertionId::generate();

        assert!(!entity.as_str().is_empty());
        assert!(!concept.as_str().is_empty());
        assert!(!source.as_str().is_empty());
        assert!(!reference.as_str().is_empty());
        assert!(!lexical_form.as_str().is_empty());
        assert!(!mention.as_str().is_empty());
        assert!(!assertion.as_str().is_empty());
    }

    #[test]
    fn phase1_identity_types_remain_strongly_distinct() {
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<ConceptId>());
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<SourceId>());
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<ReferenceId>());
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<LexicalFormId>());
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<MentionId>());
        assert_ne!(
            TypeId::of::<EntityId>(),
            TypeId::of::<KnowledgeAssertionId>()
        );

        assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<SourceId>());
        assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<ReferenceId>());
        assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<LexicalFormId>());
        assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<MentionId>());
        assert_ne!(
            TypeId::of::<ConceptId>(),
            TypeId::of::<KnowledgeAssertionId>()
        );

        assert_ne!(TypeId::of::<SourceId>(), TypeId::of::<ReferenceId>());
        assert_ne!(TypeId::of::<SourceId>(), TypeId::of::<LexicalFormId>());
        assert_ne!(TypeId::of::<SourceId>(), TypeId::of::<MentionId>());
        assert_ne!(
            TypeId::of::<SourceId>(),
            TypeId::of::<KnowledgeAssertionId>()
        );

        assert_ne!(TypeId::of::<ReferenceId>(), TypeId::of::<LexicalFormId>());
        assert_ne!(TypeId::of::<ReferenceId>(), TypeId::of::<MentionId>());
        assert_ne!(
            TypeId::of::<ReferenceId>(),
            TypeId::of::<KnowledgeAssertionId>()
        );

        assert_ne!(TypeId::of::<LexicalFormId>(), TypeId::of::<MentionId>());
        assert_ne!(
            TypeId::of::<LexicalFormId>(),
            TypeId::of::<KnowledgeAssertionId>()
        );

        assert_ne!(
            TypeId::of::<MentionId>(),
            TypeId::of::<KnowledgeAssertionId>()
        );
    }

    #[test]
    fn phase1_identity_module_preserves_core_value_semantics() {
        let entity = EntityId::new("entity-1").expect("valid entity identity");
        let same_entity = EntityId::new("entity-1").expect("same valid entity identity");
        let concept = ConceptId::new("entity-1").expect("valid concept identity with same value");

        assert_eq!(entity, same_entity);
        assert_eq!(entity.as_str(), "entity-1");
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<ConceptId>());
        assert_eq!(concept.as_str(), "entity-1");
    }

    #[test]
    fn phase1_identity_generation_is_unique_within_each_identity_type() {
        assert_ne!(EntityId::generate(), EntityId::generate());
        assert_ne!(ConceptId::generate(), ConceptId::generate());
        assert_ne!(SourceId::generate(), SourceId::generate());
        assert_ne!(ReferenceId::generate(), ReferenceId::generate());
        assert_ne!(LexicalFormId::generate(), LexicalFormId::generate());
        assert_ne!(MentionId::generate(), MentionId::generate());
        assert_ne!(
            KnowledgeAssertionId::generate(),
            KnowledgeAssertionId::generate()
        );
    }
}
