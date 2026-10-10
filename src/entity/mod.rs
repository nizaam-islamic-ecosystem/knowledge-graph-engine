//! Foundational knowledge-graph entity representations.
//!
//! The module provides [`Entity`], [`Name`], [`Alias`], and [`Mention`], plus
//! source-scoped external identity metadata. Entity-owned external identifiers
//! reuse the canonical Phase 3 `ExternalIdentifier` type, so the entity and
//! resolution APIs cannot drift into incompatible identifier representations.
//! Source-to-canonical correspondences remain separate crosswalk records in the
//! resolution module. This module does not perform entity resolution.

mod alias;
mod external_identifier;
mod mention;
mod model;
mod name;

pub use alias::Alias;
pub use external_identifier::{ExternalIdentifier, ExternalIdentifierError};
pub use mention::Mention;
pub use model::Entity;
pub use name::Name;

#[cfg(test)]
mod tests {
    use super::{Alias, Entity, ExternalIdentifier, Mention, Name};
    use crate::identity::{EntityId, MentionId, SourceId};
    use std::any::TypeId;

    #[test]
    fn entity_module_exposes_the_complete_entity_and_external_identity_api() {
        let entity_id = EntityId::generate();
        let mention_id = MentionId::generate();
        let source_id = SourceId::new("source-a").expect("valid source ID");
        let external =
            ExternalIdentifier::new(source_id, "upstream-123").expect("valid external identifier");

        let name = Name::new("محمد", "ar");
        let aliases = vec![Alias::new("Muhammad", "en"), Alias::new("محمد", "ur")];
        let entity = Entity::new(entity_id.clone(), name.clone(), aliases.clone())
            .with_external_identifier(external.clone());
        let mention = Mention::new(mention_id.clone(), "محمد");

        assert_eq!(entity.id(), &entity_id);
        assert_eq!(entity.name(), &name);
        assert_eq!(entity.aliases(), aliases.as_slice());
        assert!(entity.has_external_identifier(&external));
        assert_eq!(mention.id(), &mention_id);
        assert_eq!(mention.representation(), "محمد");
    }

    #[test]
    fn entity_and_mention_remain_strongly_distinct_types() {
        assert_ne!(TypeId::of::<Entity>(), TypeId::of::<Mention>());
    }

    #[test]
    fn entity_external_identifier_reexport_is_the_same_canonical_type() {
        assert_eq!(
            TypeId::of::<ExternalIdentifier>(),
            TypeId::of::<crate::resolution::ExternalIdentifier>()
        );
        assert_eq!(
            TypeId::of::<super::ExternalIdentifierError>(),
            TypeId::of::<crate::resolution::ExternalIdentifierError>()
        );
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
        assert!(entity.external_identifiers().is_empty());
    }
}
