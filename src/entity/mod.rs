//! Foundational knowledge-graph entity representations for Phase 1.
//!
//! This module provides the minimal entity vocabulary established by the
//! Phase 1 scope: [`Entity`], [`Name`], [`Alias`], and [`Mention`].
//!
//! Entity identity is supplied by the Core-backed [`crate::identity::EntityId`]
//! type. Names and aliases remain representation data, while mentions remain
//! distinct occurrence-level objects. Higher-level concerns such as ontology,
//! relationships, evidence, provenance, authority, and entity resolution are
//! intentionally outside this module.

mod alias;
mod mention;
mod model;
mod name;

pub use alias::Alias;
pub use mention::Mention;
pub use model::Entity;
pub use name::Name;

#[cfg(test)]
mod tests {
    use super::{Alias, Entity, Mention, Name};
    use crate::identity::{EntityId, MentionId};
    use std::any::TypeId;

    #[test]
    fn entity_module_exposes_the_complete_phase1_vocabulary() {
        let entity_id = EntityId::generate();
        let mention_id = MentionId::generate();

        let name = Name::new("محمد", "ar");
        let aliases = vec![Alias::new("Muhammad", "en"), Alias::new("محمد", "ur")];
        let entity = Entity::new(entity_id.clone(), name.clone(), aliases.clone());
        let mention = Mention::new(mention_id.clone(), "محمد");

        assert_eq!(entity.id(), &entity_id);
        assert_eq!(entity.name(), &name);
        assert_eq!(entity.aliases(), aliases.as_slice());
        assert_eq!(mention.id(), &mention_id);
        assert_eq!(mention.representation(), "محمد");
    }

    #[test]
    fn entity_and_mention_remain_strongly_distinct_types() {
        assert_ne!(TypeId::of::<Entity>(), TypeId::of::<Mention>());
    }

    #[test]
    fn name_and_alias_share_the_representation_boundary() {
        let name = Name::new("Allah", "en");
        let alias = Alias::new("Allah", "en");

        assert_eq!(name, alias);
        assert_eq!(name.value(), alias.value());
        assert_eq!(name.language(), alias.language());
    }

    #[test]
    fn entity_module_preserves_multilingual_representation_without_linguistic_logic() {
        let entity = Entity::new(
            EntityId::generate(),
            Name::new("الله", "ar"),
            vec![Alias::new("Allah", "en"), Alias::new("اللہ", "ur")],
        );

        assert_eq!(entity.name().value(), "الله");
        assert_eq!(entity.name().language(), "ar");
        assert_eq!(entity.aliases()[0].value(), "Allah");
        assert_eq!(entity.aliases()[0].language(), "en");
        assert_eq!(entity.aliases()[1].value(), "اللہ");
        assert_eq!(entity.aliases()[1].language(), "ur");
    }

    #[test]
    fn entity_module_does_not_require_an_entity_type_for_phase1() {
        let entity = Entity::new(EntityId::generate(), Name::new("Example", "en"), Vec::new());

        assert_eq!(entity.name().value(), "Example");
        assert!(entity.aliases().is_empty());
    }
}
