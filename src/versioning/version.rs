//! Knowledge-graph version identity and state.
//!
//! A KG version identifies a committed logical state of the Knowledge Graph.
//! It is intentionally distinct from source versions, temporal validity,
//! storage schema versions, and Indexing Engine version identities.

use nizaam_core::identity;

use super::ChangeSetId;

identity!(
    /// Identifies a committed logical version of the Knowledge Graph.
    KgVersionId
);

/// Metadata describing one committed logical KG state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KgVersion {
    id: KgVersionId,
    sequence: u64,
    parent: Option<KgVersionId>,
    change_set: Option<ChangeSetId>,
}

impl KgVersion {
    /// Creates the initial KG version.
    pub fn initial() -> Self {
        Self {
            id: KgVersionId::generate(),
            sequence: 0,
            parent: None,
            change_set: None,
        }
    }

    /// Creates a new committed KG version from a previous version and the
    /// Change Set that produced it.
    pub fn next(parent: &Self, change_set: ChangeSetId) -> Self {
        Self {
            id: KgVersionId::generate(),
            sequence: parent.sequence + 1,
            parent: Some(parent.id.clone()),
            change_set: Some(change_set),
        }
    }

    /// Reconstructs persisted version metadata inside the KG crate.
    pub(crate) fn from_parts(
        id: KgVersionId,
        sequence: u64,
        parent: Option<KgVersionId>,
        change_set: Option<ChangeSetId>,
    ) -> Self {
        Self {
            id,
            sequence,
            parent,
            change_set,
        }
    }

    /// Returns the version identity.
    pub fn id(&self) -> &KgVersionId {
        &self.id
    }

    /// Returns the logical sequence number of this version.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the parent version, if this version was produced from one.
    pub fn parent(&self) -> Option<&KgVersionId> {
        self.parent.as_ref()
    }

    /// Returns the Change Set that produced this version, if one exists.
    pub fn change_set(&self) -> Option<&ChangeSetId> {
        self.change_set.as_ref()
    }

    /// Returns whether the supplied identity identifies this version.
    pub fn has_id(&self, id: &KgVersionId) -> bool {
        &self.id == id
    }

    /// Returns whether this is the initial KG version.
    pub fn is_initial(&self) -> bool {
        self.parent.is_none() && self.change_set.is_none() && self.sequence == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_version_has_no_parent_or_change_set() {
        let version = KgVersion::initial();

        assert!(version.is_initial());
        assert_eq!(version.sequence(), 0);
        assert!(version.parent().is_none());
        assert!(version.change_set().is_none());
        assert!(!version.id().as_str().is_empty());
        assert!(version.has_id(version.id()));
    }

    #[test]
    fn next_version_preserves_parent_and_increments_sequence() {
        let initial = KgVersion::initial();
        let change_set = ChangeSetId::generate();

        let next = KgVersion::next(&initial, change_set.clone());

        assert_eq!(next.sequence(), 1);
        assert_eq!(next.parent(), Some(initial.id()));
        assert_eq!(next.change_set(), Some(&change_set));
        assert_ne!(next.id(), initial.id());
        assert!(!next.is_initial());
    }

    #[test]
    fn persisted_version_parts_round_trip() {
        let initial = KgVersion::initial();
        let change_set = ChangeSetId::generate();
        let next = KgVersion::next(&initial, change_set.clone());

        let restored = KgVersion::from_parts(
            next.id().clone(),
            next.sequence(),
            next.parent().cloned(),
            next.change_set().cloned(),
        );

        assert_eq!(restored, next);
    }
}
