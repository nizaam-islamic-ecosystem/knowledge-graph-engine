//! KG versioning, historical change tracking, migration, and compatibility.
//!
//! KG version identity remains a semantic version of canonical knowledge. It
//! is intentionally separate from physical storage schema versions, source
//! versions, temporal validity, and Indexing-native versions.

mod change;
mod compatibility;
mod migration;
mod version;

pub use change::{Change, ChangeKind, ChangeSet, ChangeSetId, ChangeTarget};
pub use compatibility::{
    CompatibilityKind, CompatibilityLevel, CompatibilityMatrix, CompatibilityReport,
    CompatibilityResult,
};
pub use migration::{
    MigrationError, MigrationPlan, MigrationReport, MigrationRunner, MigrationState, MigrationStep,
    StorageSchemaVersion,
};
pub use version::{KgVersion, KgVersionId};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    use std::cell::Cell;

    #[derive(Debug)]
    struct Migration {
        from: StorageSchemaVersion,
        to: StorageSchemaVersion,
        activated: Cell<bool>,
    }

    impl Migration {
        fn new(from: u64, to: u64) -> Self {
            Self {
                from: StorageSchemaVersion::new(from),
                to: StorageSchemaVersion::new(to),
                activated: Cell::new(false),
            }
        }
    }

    impl MigrationStep for Migration {
        fn from_version(&self) -> StorageSchemaVersion {
            self.from
        }

        fn to_version(&self) -> StorageSchemaVersion {
            self.to
        }

        fn name(&self) -> &str {
            "level2-schema-migration"
        }

        fn apply(&self) -> Result<(), MigrationError> {
            Ok(())
        }

        fn verify(&self) -> Result<(), MigrationError> {
            Ok(())
        }

        fn activate(&self) -> Result<(), MigrationError> {
            self.activated.set(true);
            Ok(())
        }
    }

    #[test]
    fn versioning_public_boundary_connects_change_history_migration_and_compatibility() {
        let initial = KgVersion::initial();
        let target = ChangeTarget::Entity(
            EntityId::new("entity-1").expect("test entity identity should be valid"),
        );
        let mut change_set = ChangeSet::new(Some(initial.id().clone()));
        change_set.push(Change::new(ChangeKind::Created, target.clone()));

        let next = KgVersion::next(&initial, change_set.id().clone());
        assert_eq!(next.parent(), Some(initial.id()));
        assert_eq!(next.change_set(), Some(change_set.id()));

        let migration = Migration::new(1, 2);
        let plan = MigrationPlan::new(
            StorageSchemaVersion::new(1),
            StorageSchemaVersion::new(2),
            vec![&migration],
        )
        .expect("schema migration path should be valid");
        let report = MigrationRunner::run(&plan).expect("migration should activate");

        let compatibility = CompatibilityMatrix::compatible()
            .with_storage_schema(CompatibilityLevel::CompatibleWithMigration)
            .report();

        assert!(report.is_activated());
        assert!(migration.activated.get());
        assert!(compatibility.is_compatible());
        assert!(compatibility.requires_migration());

        // A physical schema migration does not create a new semantic KG
        // version by itself.
        assert_eq!(next.sequence(), 1);
        assert_ne!(next.id().as_str(), "2");
        assert_eq!(plan.to_version().value(), 2);
    }

    #[test]
    fn versioning_public_boundary_keeps_kg_version_distinct_from_schema_version() {
        let version = KgVersion::initial();
        let schema = StorageSchemaVersion::new(1);

        assert_eq!(version.sequence(), 0);
        assert_eq!(schema.value(), 1);
        assert_ne!(version.sequence(), schema.value());
    }

    #[test]
    fn incompatible_semantic_change_blocks_compatibility_even_when_schema_can_migrate() {
        let report = CompatibilityMatrix::compatible()
            .with_semantic_model(CompatibilityLevel::Incompatible)
            .with_storage_schema(CompatibilityLevel::CompatibleWithMigration)
            .report();

        assert!(!report.is_compatible());
        assert!(report.requires_migration());
        assert_eq!(
            report
                .first_incompatibility()
                .expect("semantic incompatibility should be present")
                .kind(),
            CompatibilityKind::SemanticModel
        );
    }
}
