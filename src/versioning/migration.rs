//! Controlled storage-schema migration orchestration.
//!
//! Migration state is deliberately separate from KG versioning. A migration
//! changes the physical storage schema while preserving the semantic KG and
//! its logical version history. The runner exposes explicit prepare/apply/
//! verify/activate stages so an interrupted or failed migration cannot be
//! mistaken for an activated schema.

use core::fmt;

/// Physical storage schema version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StorageSchemaVersion(u64);

impl StorageSchemaVersion {
    /// Creates a schema version from a non-negative integer.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric schema version.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for StorageSchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Lifecycle state of a migration attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationState {
    Prepared,
    Applied,
    Verified,
    Activated,
    Failed,
}

/// Errors produced by migration planning or execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationError {
    InvalidPath {
        from: StorageSchemaVersion,
        to: StorageSchemaVersion,
    },
    StepMismatch {
        expected: StorageSchemaVersion,
        actual: StorageSchemaVersion,
    },
    ApplyFailed(String),
    VerificationFailed(String),
    ActivationFailed(String),
    Interrupted,
    Incompatible(String),
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { from, to } => {
                write!(formatter, "no migration path from schema {from} to {to}")
            }
            Self::StepMismatch { expected, actual } => {
                write!(
                    formatter,
                    "migration step expected schema {expected}, found {actual}"
                )
            }
            Self::ApplyFailed(message) => write!(formatter, "migration apply failed: {message}"),
            Self::VerificationFailed(message) => {
                write!(formatter, "migration verification failed: {message}")
            }
            Self::ActivationFailed(message) => {
                write!(formatter, "migration activation failed: {message}")
            }
            Self::Interrupted => formatter.write_str("migration was interrupted before activation"),
            Self::Incompatible(message) => {
                write!(formatter, "migration is incompatible: {message}")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

/// One controlled schema migration step.
#[allow(clippy::wrong_self_convention)]
pub trait MigrationStep {
    /// Source storage schema version.
    fn from_version(&self) -> StorageSchemaVersion;

    /// Destination storage schema version.
    fn to_version(&self) -> StorageSchemaVersion;

    /// Stable migration name.
    fn name(&self) -> &str;

    /// Applies the physical transformation.
    fn apply(&self) -> Result<(), MigrationError>;

    /// Validates the transformed physical state before activation.
    fn verify(&self) -> Result<(), MigrationError>;

    /// Activates the verified destination schema.
    fn activate(&self) -> Result<(), MigrationError>;
}

/// Ordered migration path between two storage schema versions.
pub struct MigrationPlan<'a> {
    from: StorageSchemaVersion,
    to: StorageSchemaVersion,
    steps: Vec<&'a dyn MigrationStep>,
}

impl<'a> MigrationPlan<'a> {
    /// Builds a plan and verifies that every step forms a contiguous path.
    pub fn new(
        from: StorageSchemaVersion,
        to: StorageSchemaVersion,
        steps: Vec<&'a dyn MigrationStep>,
    ) -> Result<Self, MigrationError> {
        if from == to {
            if steps.is_empty() {
                return Ok(Self { from, to, steps });
            }
            return Err(MigrationError::InvalidPath { from, to });
        }

        let mut expected = from;
        for step in &steps {
            if step.from_version() != expected {
                return Err(MigrationError::StepMismatch {
                    expected,
                    actual: step.from_version(),
                });
            }
            expected = step.to_version();
        }

        if expected != to {
            return Err(MigrationError::InvalidPath { from, to });
        }

        Ok(Self { from, to, steps })
    }

    #[must_use]
    pub const fn from_version(&self) -> StorageSchemaVersion {
        self.from
    }

    #[must_use]
    pub const fn to_version(&self) -> StorageSchemaVersion {
        self.to
    }

    #[must_use]
    pub fn steps(&self) -> &[&'a dyn MigrationStep] {
        &self.steps
    }
}

/// Result of one controlled migration attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationReport {
    from: StorageSchemaVersion,
    to: StorageSchemaVersion,
    state: MigrationState,
    applied_steps: Vec<String>,
}

impl MigrationReport {
    #[must_use]
    pub const fn from_version(&self) -> StorageSchemaVersion {
        self.from
    }

    #[must_use]
    pub const fn to_version(&self) -> StorageSchemaVersion {
        self.to
    }

    #[must_use]
    pub const fn state(&self) -> MigrationState {
        self.state
    }

    #[must_use]
    pub fn applied_steps(&self) -> &[String] {
        &self.applied_steps
    }

    #[must_use]
    pub const fn is_activated(&self) -> bool {
        matches!(self.state, MigrationState::Activated)
    }
}

/// Executes a previously validated migration plan.
#[derive(Debug, Default)]
pub struct MigrationRunner;

impl MigrationRunner {
    /// Runs every step through apply and verify, then activates the destination.
    ///
    /// Activation is represented by the final report state rather than being
    /// implied by successful transformation. This keeps interrupted/failed
    /// migrations explicitly distinguishable from an active schema.
    pub fn run(plan: &MigrationPlan<'_>) -> Result<MigrationReport, MigrationError> {
        if plan.from_version() == plan.to_version() {
            return Ok(MigrationReport {
                from: plan.from_version(),
                to: plan.to_version(),
                state: MigrationState::Activated,
                applied_steps: Vec::new(),
            });
        }

        let mut applied_steps = Vec::with_capacity(plan.steps().len());
        for step in plan.steps() {
            step.apply()?;
            applied_steps.push(step.name().to_owned());
            step.verify()?;
            step.activate()?;
        }

        Ok(MigrationReport {
            from: plan.from_version(),
            to: plan.to_version(),
            state: MigrationState::Activated,
            applied_steps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Step {
        from: StorageSchemaVersion,
        to: StorageSchemaVersion,
        name: &'static str,
        applied: Cell<bool>,
        verified: Cell<bool>,
        fail_apply: bool,
        activated: Cell<bool>,
    }

    impl Step {
        fn new(from: u64, to: u64, name: &'static str) -> Self {
            Self {
                from: StorageSchemaVersion::new(from),
                to: StorageSchemaVersion::new(to),
                name,
                applied: Cell::new(false),
                verified: Cell::new(false),
                fail_apply: false,
                activated: Cell::new(false),
            }
        }
    }

    impl MigrationStep for Step {
        fn from_version(&self) -> StorageSchemaVersion {
            self.from
        }
        fn to_version(&self) -> StorageSchemaVersion {
            self.to
        }
        fn name(&self) -> &str {
            self.name
        }

        fn apply(&self) -> Result<(), MigrationError> {
            if self.fail_apply {
                return Err(MigrationError::Interrupted);
            }
            self.applied.set(true);
            Ok(())
        }

        fn verify(&self) -> Result<(), MigrationError> {
            assert!(self.applied.get());
            self.verified.set(true);
            Ok(())
        }

        fn activate(&self) -> Result<(), MigrationError> {
            assert!(self.verified.get());
            self.activated.set(true);
            Ok(())
        }
    }

    #[test]
    fn migration_plan_requires_contiguous_schema_path() {
        let first = Step::new(1, 2, "one-to-two");
        let second = Step::new(2, 3, "two-to-three");

        let plan = MigrationPlan::new(
            StorageSchemaVersion::new(1),
            StorageSchemaVersion::new(3),
            vec![&first, &second],
        )
        .expect("migration path should be valid");

        assert_eq!(plan.steps().len(), 2);
        assert_eq!(plan.from_version().value(), 1);
        assert_eq!(plan.to_version().value(), 3);
    }

    #[test]
    fn migration_runner_verifies_before_activation() {
        let first = Step::new(1, 2, "one-to-two");
        let plan = MigrationPlan::new(
            StorageSchemaVersion::new(1),
            StorageSchemaVersion::new(2),
            vec![&first],
        )
        .expect("migration path should be valid");

        let report = MigrationRunner::run(&plan).expect("migration should succeed");

        assert_eq!(report.state(), MigrationState::Activated);
        assert!(report.is_activated());
        assert!(first.verified.get());
        assert!(first.activated.get());
        assert_eq!(report.applied_steps(), &["one-to-two".to_owned()]);
    }

    #[test]
    fn interrupted_migration_is_not_reported_as_activated() {
        let mut first = Step::new(1, 2, "one-to-two");
        first.fail_apply = true;

        let plan = MigrationPlan::new(
            StorageSchemaVersion::new(1),
            StorageSchemaVersion::new(2),
            vec![&first],
        )
        .expect("migration path should be valid");

        let error = MigrationRunner::run(&plan).expect_err("interruption should fail");
        assert_eq!(error, MigrationError::Interrupted);
        assert!(!first.applied.get());
    }
}
