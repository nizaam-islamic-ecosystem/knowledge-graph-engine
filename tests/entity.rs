//! Level 3 public-boundary tests for the Phase 1 Entity module.

use nizaam_knowledge_graph::{
    entity::{Alias, Entity, ExternalIdentifier, Mention, Name},
    identity::{EntityId, MentionId, SourceId},
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

#[test]
fn entity_owns_source_scoped_external_identifiers_without_rewriting_values() {
    let source = SourceId::new("source-catalog").unwrap();
    let external = ExternalIdentifier::new(source.clone(), " Person-0042 ").unwrap();
    let entity = Entity::new(
        EntityId::new("entity-external-identity").unwrap(),
        Name::new("Person", "en"),
        Vec::new(),
    )
    .with_external_identifier(external.clone());

    assert!(entity.has_external_identifier(&external));
    assert_eq!(entity.external_identifiers().len(), 1);
    assert_eq!(
        entity.external_identifiers().first().unwrap().source_id(),
        &source
    );
    assert_eq!(
        entity.external_identifiers().first().unwrap().value(),
        " Person-0042 "
    );
}

#[test]
fn same_external_value_from_two_sources_remains_distinct_on_one_entity() {
    let first = ExternalIdentifier::new(SourceId::new("source-a").unwrap(), "record-7").unwrap();
    let second = ExternalIdentifier::new(SourceId::new("source-b").unwrap(), "record-7").unwrap();
    let entity = Entity::new(
        EntityId::new("entity-multi-source").unwrap(),
        Name::new("Example", "en"),
        Vec::new(),
    )
    .with_external_identifiers([first.clone(), second.clone(), first.clone()]);

    assert_eq!(entity.external_identifiers().len(), 2);
    assert!(entity.has_external_identifier(&first));
    assert!(entity.has_external_identifier(&second));
    assert_eq!(
        entity
            .external_identifiers()
            .iter()
            .filter(|id| id.value() == "record-7")
            .count(),
        2
    );
}
