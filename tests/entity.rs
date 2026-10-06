//! Level 3 public-boundary tests for the Phase 1 Entity module.

use nizaam_knowledge_graph::{
    entity::{Alias, Entity, Mention, Name},
    identity::{EntityId, MentionId},
};
use std::any::TypeId;

#[test]
fn entity_public_api_preserves_identity_name_and_aliases() {
    let id = EntityId::generate();
    let name = Name::new("محمد", "ar");
    let aliases = vec![Alias::new("Muhammad", "en"), Alias::new("محمد", "ur")];

    let entity = Entity::new(id.clone(), name.clone(), aliases.clone());

    assert_eq!(entity.id(), &id);
    assert_eq!(entity.name(), &name);
    assert_eq!(entity.aliases(), aliases.as_slice());
}

#[test]
fn entity_public_api_supports_multilingual_representations_without_language_logic() {
    let entity = Entity::new(
        EntityId::generate(),
        Name::new("الله", "ar"),
        vec![
            Alias::new("Allah", "en"),
            Alias::new("اللہ", "ur"),
            Alias::new("Other", "custom"),
        ],
    );

    assert_eq!(entity.name().value(), "الله");
    assert_eq!(entity.name().language(), "ar");
    assert_eq!(entity.aliases()[0].value(), "Allah");
    assert_eq!(entity.aliases()[1].value(), "اللہ");
    assert_eq!(entity.aliases()[2].language(), "custom");
}

#[test]
fn alias_is_representation_data_not_a_separate_identity_type() {
    let name = Name::new("Allah", "en");
    let alias = Alias::new("Allah", "en");

    assert_eq!(name, alias);
    assert_eq!(name.value(), alias.value());
    assert_eq!(name.language(), alias.language());
}

#[test]
fn mention_is_distinct_from_entity_at_the_public_boundary() {
    assert_ne!(TypeId::of::<Entity>(), TypeId::of::<Mention>());
    assert_ne!(TypeId::of::<MentionId>(), TypeId::of::<EntityId>());
}

#[test]
fn entity_can_exist_with_no_aliases() {
    let entity = Entity::new(EntityId::generate(), Name::new("Example", "en"), Vec::new());

    assert!(entity.aliases().is_empty());
}
