//! Minimal knowledge-graph entity representation for Phase 1.
//!
//! An entity is an identifiable referent with a primary name and zero or more
//! aliases. Phase 1 intentionally does not attach ontology, relationships,
//! evidence, authority, provenance, or entity-resolution behavior here.

use crate::identity::EntityId;

use super::alias::Alias;
use super::name::Name;

/// A minimal identifiable knowledge-graph entity.
///
/// The representation is intentionally small so later phases can extend it
/// without forcing Phase 1 to decide higher-level semantic behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    id: EntityId,
    name: Name,
    aliases: Vec<Alias>,
}

impl Entity {
    /// Constructs an entity from its identity, primary name, and aliases.
    #[must_use]
    pub fn new(id: EntityId, name: Name, aliases: Vec<Alias>) -> Self {
        Self { id, name, aliases }
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
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityId};
    use crate::entity::alias::Alias;
    use crate::entity::name::Name;

    #[test]
    fn entity_preserves_identity_and_primary_name() {
        let id = EntityId::generate();
        let name = Name::new("الله", "ar");
        let entity = Entity::new(id.clone(), name.clone(), Vec::new());

        assert_eq!(entity.id(), &id);
        assert_eq!(entity.name(), &name);
        assert!(entity.aliases().is_empty());
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
        assert_eq!(entity.aliases()[0].language(), "en");
        assert_eq!(entity.aliases()[1].language(), "ur");
    }
}
