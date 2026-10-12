//! Immutable KG read snapshots.
//!
//! A snapshot represents one coherent read view of canonical KG state.
//! It is pinned to a KG version but is not itself a KG version and does not
//! represent temporal validity.

use core::fmt;

use nizaam_core::identity;

use crate::versioning::{ChangeTarget, KgVersion};

use super::repository::RepositoryRecord;

identity!(
    /// Identifies one immutable read snapshot of canonical KG state.
    SnapshotId
);

/// Errors produced while constructing or accessing a snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// A snapshot cannot be created without a committed KG version.
    MissingVersion,

    /// A snapshot record contains an invalid semantic target.
    InvalidRecord(String),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVersion => formatter.write_str("snapshot requires a committed KG version"),
            Self::InvalidRecord(message) => {
                write!(formatter, "invalid snapshot record: {message}")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Immutable read view of canonical KG state.
///
/// The records are captured when the snapshot is created and cannot be
/// modified through this type. A later repository commit therefore does not
/// mutate an existing snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    id: SnapshotId,
    version: KgVersion,
    records: Vec<RepositoryRecord>,
}

impl Snapshot {
    /// Creates an immutable snapshot pinned to one committed KG version.
    pub fn new(version: KgVersion, records: Vec<RepositoryRecord>) -> Result<Self, SnapshotError> {
        if version.id().as_str().is_empty() {
            return Err(SnapshotError::MissingVersion);
        }

        Ok(Self {
            id: SnapshotId::generate(),
            version,
            records,
        })
    }

    /// Returns the snapshot identity.
    #[must_use]
    pub fn id(&self) -> &SnapshotId {
        &self.id
    }

    /// Returns the KG version represented by this read view.
    #[must_use]
    pub fn version(&self) -> &KgVersion {
        &self.version
    }

    /// Returns all records captured by this snapshot.
    #[must_use]
    pub fn records(&self) -> &[RepositoryRecord] {
        &self.records
    }

    /// Looks up a semantic record inside this stable read view.
    #[must_use]
    pub fn read(&self, target: &ChangeTarget) -> Option<&RepositoryRecord> {
        self.records.iter().find(|record| record.target() == target)
    }

    /// Returns whether the snapshot contains the supplied semantic target.
    #[must_use]
    pub fn contains(&self, target: &ChangeTarget) -> bool {
        self.read(target).is_some()
    }

    /// Returns the number of semantic records captured by this snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether the snapshot contains no semantic records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    use crate::storage::document_store::Document;

    fn target(value: &str) -> ChangeTarget {
        ChangeTarget::Entity(EntityId::new(value).expect("valid entity identity"))
    }

    fn record(value: &str) -> RepositoryRecord {
        RepositoryRecord::new(target(value), Document::from_slice(value.as_bytes()))
    }

    #[test]
    fn snapshot_is_pinned_to_one_kg_version() {
        let version = KgVersion::initial();

        let snapshot =
            Snapshot::new(version.clone(), Vec::new()).expect("snapshot should be created");

        assert_eq!(snapshot.version(), &version);
        assert!(!snapshot.id().as_str().is_empty());
    }

    #[test]
    fn snapshot_reads_records_without_mutating_them() {
        let version = KgVersion::initial();
        let first = record("entity-1");
        let second = record("entity-2");

        let snapshot = Snapshot::new(version, vec![first.clone(), second.clone()])
            .expect("snapshot should be created");

        assert_eq!(snapshot.len(), 2);
        assert!(snapshot.contains(first.target()));
        assert_eq!(snapshot.read(first.target()), Some(&first));
        assert_eq!(snapshot.read(second.target()), Some(&second));
    }

    #[test]
    fn snapshot_missing_target_returns_none() {
        let snapshot = Snapshot::new(KgVersion::initial(), vec![record("entity-1")])
            .expect("snapshot should be created");

        let missing = target("entity-2");

        assert!(!snapshot.contains(&missing));
        assert!(snapshot.read(&missing).is_none());
    }

    #[test]
    fn snapshots_have_distinct_identities() {
        let version = KgVersion::initial();

        let first =
            Snapshot::new(version.clone(), Vec::new()).expect("first snapshot should succeed");

        let second = Snapshot::new(version, Vec::new()).expect("second snapshot should succeed");

        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn snapshot_is_independent_from_later_version_identity() {
        let first_version = KgVersion::initial();

        let snapshot =
            Snapshot::new(first_version.clone(), Vec::new()).expect("snapshot should be created");

        let second_version =
            KgVersion::next(&first_version, crate::versioning::ChangeSetId::generate());

        assert_eq!(snapshot.version(), &first_version);
        assert_ne!(snapshot.version().id(), second_version.id());
    }
}
