//! External identifier crosswalk records for Phase 3 and later ingestion.
//!
//! The canonical `ExternalIdentifier` type is owned by the entity layer.
//! A crosswalk preserves the distinction between a source-owned external
//! identifier and the canonical Knowledge Graph entity it currently maps to.
//! It is not an entity identity, is not an `IndexAssignedId`, and does not own
//! persistence or historical versioning.

use crate::entity::ExternalIdentifier;
use crate::identity::EntityId;

/// A source-to-canonical crosswalk record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExternalIdentifierCrosswalk {
    external_identifier: ExternalIdentifier,
    canonical_entity: EntityId,
}

impl ExternalIdentifierCrosswalk {
    /// Creates a crosswalk record without changing either identity.
    #[must_use]
    pub fn new(external_identifier: ExternalIdentifier, canonical_entity: EntityId) -> Self {
        Self {
            external_identifier,
            canonical_entity,
        }
    }

    /// Returns the source-owned external identifier.
    #[must_use]
    pub fn external_identifier(&self) -> &ExternalIdentifier {
        &self.external_identifier
    }

    /// Returns the canonical KG entity identity.
    #[must_use]
    pub fn canonical_entity(&self) -> &EntityId {
        &self.canonical_entity
    }
}

#[cfg(test)]
mod tests {
    use super::ExternalIdentifierCrosswalk;
    use crate::entity::ExternalIdentifier;
    use crate::identity::{EntityId, SourceId};

    #[test]
    fn crosswalk_preserves_external_source_and_value_separately_from_entity() {
        let source = SourceId::new("source-a").expect("valid source");
        let external = ExternalIdentifier::new(source.clone(), "person-42")
            .expect("valid external identifier");
        let entity = EntityId::new("entity-1").expect("valid entity");
        let crosswalk = ExternalIdentifierCrosswalk::new(external, entity.clone());

        assert_eq!(crosswalk.external_identifier().source_id(), &source);
        assert_eq!(crosswalk.external_identifier().value(), "person-42");
        assert_eq!(crosswalk.canonical_entity(), &entity);
    }

    #[test]
    fn crosswalk_does_not_rewrite_source_identifier_or_canonical_identity() {
        let external = ExternalIdentifier::new(
            SourceId::new("source-catalog").expect("valid source"),
            " Person-0042 ",
        )
        .expect("valid external identifier");
        let entity = EntityId::new("canonical-person-4").expect("valid entity");
        let crosswalk = ExternalIdentifierCrosswalk::new(external.clone(), entity.clone());

        assert_eq!(crosswalk.external_identifier(), &external);
        assert_eq!(crosswalk.external_identifier().value(), " Person-0042 ");
        assert_eq!(crosswalk.canonical_entity(), &entity);
    }
}
