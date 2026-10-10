//! Minimal knowledge-graph entity representation, extended with external IDs.
//!
//! An entity is an identifiable referent with a primary name, zero or more
//! aliases, and zero or more source-scoped external identifiers. The external
//! identifiers preserve source-provided identity values; canonical KG identity
//! remains represented by `EntityId` and source-to-canonical mapping remains a
//! separate resolution/crosswalk concern.

use std::collections::BTreeSet;

use crate::identity::EntityId;

use super::alias::Alias;
use super::external_identifier::ExternalIdentifier;
use super::name::Name;

/// A minimal identifiable knowledge-graph entity.
///
/// External identifiers are retained exactly as supplied by their source.
/// The same external value from two different sources remains two distinct
/// identifiers because the source identity is part of `ExternalIdentifier`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    id: EntityId,
    name: Name,
    aliases: Vec<Alias>,
    external_identifiers: BTreeSet<ExternalIdentifier>,
}

impl Entity {
    /// Constructs an entity from its identity, primary name, and aliases.
    ///
    /// This Phase 1 constructor remains unchanged for compatibility. External
    /// identifiers can be added through the builder methods below.
    #[must_use]
    pub fn new(id: EntityId, name: Name, aliases: Vec<Alias>) -> Self {
        Self {
            id,
            name,
            aliases,
            external_identifiers: BTreeSet::new(),
        }
    }

    /// Returns the entity identity.
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the primary name/representation.
    #[must_use]
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the entity's aliases.
    #[must_use]
    pub fn aliases(&self) -> &[Alias] {
        &self.aliases
    }

    /// Returns all source-scoped external identifiers in deterministic order.
    #[must_use]
    pub fn external_identifiers(&self) -> &BTreeSet<ExternalIdentifier> {
        &self.external_identifiers
    }

    /// Returns whether this exact source-scoped external identifier is attached.
    #[must_use]
    pub fn has_external_identifier(&self, identifier: &ExternalIdentifier) -> bool {
        self.external_identifiers.contains(identifier)
    }

    /// Inserts a source-scoped external identifier.
    ///
    /// Returns `true` if it was newly inserted. Re-inserting the same source
    /// and value is idempotent and returns `false`; equal values owned by
    /// different sources remain distinct identifiers.
    pub fn insert_external_identifier(&mut self, identifier: ExternalIdentifier) -> bool {
        self.external_identifiers.insert(identifier)
    }

    /// Returns a new entity value with an external identifier attached.
    ///
    /// An exact duplicate is ignored, preserving idempotent construction.
    #[must_use]
    pub fn with_external_identifier(mut self, identifier: ExternalIdentifier) -> Self {
        self.insert_external_identifier(identifier);
        self
    }

    /// Returns a new entity value with all supplied external identifiers attached.
    ///
    /// Exact duplicates are collapsed by the set; the source-owned values are
    /// not normalized or rewritten.
    #[must_use]
    pub fn with_external_identifiers<I>(mut self, identifiers: I) -> Self
    where
        I: IntoIterator<Item = ExternalIdentifier>,
    {
        self.external_identifiers.extend(identifiers);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityId};
    use crate::entity::alias::Alias;
    use crate::entity::external_identifier::ExternalIdentifier;
    use crate::entity::name::Name;
    use crate::identity::SourceId;

    #[test]
    fn entity_preserves_identity_and_primary_name() {
        let id = EntityId::generate();
        let name = Name::new("الله", "ar");
        let entity = Entity::new(id.clone(), name.clone(), Vec::new());

        assert_eq!(entity.id(), &id);
        assert_eq!(entity.name(), &name);
        assert!(entity.aliases().is_empty());
        assert!(entity.external_identifiers().is_empty());
    }

    #[test]
    fn entity_preserves_multiple_alias_representations() {
        let name = Name::new("الله", "ar");
        let aliases = vec![Name::new("Allah", "en"), Name::new("اللہ", "ur")];

        let entity = Entity::new(EntityId::generate(), name, aliases.clone());

        assert_eq!(entity.aliases(), aliases.as_slice());
    }

    #[test]
    fn entity_representation_remains_multilingual_without_linguistic_processing() {
        let entity = Entity::new(
            EntityId::generate(),
            Name::new("محمد", "ar"),
            vec![Alias::new("Muhammad", "en"), Alias::new("محمد", "ur")],
        );

        assert_eq!(entity.name().value(), "محمد");
        assert_eq!(entity.name().language(), "ar");
        assert_eq!(entity.aliases()[0].value(), "Muhammad");
        assert_eq!(entity.aliases()[1].value(), "محمد");
    }

    #[test]
    fn entity_preserves_external_identifiers_without_rewriting_source_values() {
        let source = SourceId::new("source-catalog").expect("valid source ID");
        let external = ExternalIdentifier::new(source.clone(), " Person-0042 ")
            .expect("nonblank external identifier is retained exactly");
        let entity = Entity::new(EntityId::generate(), Name::new("Person", "en"), Vec::new())
            .with_external_identifier(external.clone());

        assert_eq!(entity.external_identifiers().len(), 1);
        assert!(entity.has_external_identifier(&external));
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
    fn same_external_value_from_different_sources_remains_distinct() {
        let first =
            ExternalIdentifier::new(SourceId::new("catalog-a").unwrap(), "record-17").unwrap();
        let second =
            ExternalIdentifier::new(SourceId::new("catalog-b").unwrap(), "record-17").unwrap();
        let entity = Entity::new(EntityId::generate(), Name::new("Example", "en"), Vec::new())
            .with_external_identifiers([first.clone(), second.clone(), first.clone()]);

        assert_eq!(entity.external_identifiers().len(), 2);
        assert!(entity.has_external_identifier(&first));
        assert!(entity.has_external_identifier(&second));
    }

    #[test]
    fn inserting_an_exact_duplicate_external_identifier_is_idempotent() {
        let identifier =
            ExternalIdentifier::new(SourceId::new("source-a").unwrap(), "id-9").unwrap();
        let mut entity = Entity::new(EntityId::generate(), Name::new("Example", "en"), Vec::new());

        assert!(entity.insert_external_identifier(identifier.clone()));
        assert!(!entity.insert_external_identifier(identifier));
        assert_eq!(entity.external_identifiers().len(), 1);
    }
}
