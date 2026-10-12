//! Compatibility boundaries for semantic and physical Phase 7 evolution.
//!
//! Compatibility is modeled per dimension so a storage-schema change does not
//! automatically become a semantic/API change. This module deliberately does
//! not expose provider-specific types or equate KG versions with schema
//! versions.

use core::fmt;

/// Compatibility dimensions that can evolve independently.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompatibilityKind {
    Api,
    SemanticModel,
    StorageSchema,
    StorageProvider,
    Index,
    Version,
}

impl CompatibilityKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::SemanticModel => "semantic_model",
            Self::StorageSchema => "storage_schema",
            Self::StorageProvider => "storage_provider",
            Self::Index => "index",
            Self::Version => "version",
        }
    }
}

/// Compatibility level for one independent dimension.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompatibilityLevel {
    Compatible,
    CompatibleWithMigration,
    Incompatible,
}

impl CompatibilityLevel {
    #[must_use]
    pub const fn is_compatible(self) -> bool {
        !matches!(self, Self::Incompatible)
    }

    #[must_use]
    pub const fn requires_migration(self) -> bool {
        matches!(self, Self::CompatibleWithMigration)
    }
}

/// One compatibility observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityResult {
    kind: CompatibilityKind,
    level: CompatibilityLevel,
    reason: String,
}

impl CompatibilityResult {
    /// Creates one compatibility observation.
    pub fn new(
        kind: CompatibilityKind,
        level: CompatibilityLevel,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            level,
            reason: reason.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> CompatibilityKind {
        self.kind
    }

    #[must_use]
    pub const fn level(&self) -> CompatibilityLevel {
        self.level
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Aggregate compatibility decision across all required dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityReport {
    results: Vec<CompatibilityResult>,
}

impl CompatibilityReport {
    /// Creates an empty report.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Adds one compatibility observation.
    pub fn push(&mut self, result: CompatibilityResult) {
        self.results.push(result);
    }

    #[must_use]
    pub fn results(&self) -> &[CompatibilityResult] {
        &self.results
    }

    /// Returns false if any dimension is incompatible.
    #[must_use]
    pub fn is_compatible(&self) -> bool {
        self.results
            .iter()
            .all(|result| result.level().is_compatible())
    }

    /// Returns true when at least one compatible dimension requires migration.
    #[must_use]
    pub fn requires_migration(&self) -> bool {
        self.results
            .iter()
            .any(|result| result.level().requires_migration())
    }

    /// Returns the first incompatible dimension, if any.
    #[must_use]
    pub fn first_incompatibility(&self) -> Option<&CompatibilityResult> {
        self.results
            .iter()
            .find(|result| matches!(result.level(), CompatibilityLevel::Incompatible))
    }
}

impl Default for CompatibilityReport {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CompatibilityResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {:?} ({})",
            self.kind.as_str(),
            self.level,
            self.reason
        )
    }
}

/// Declared compatibility between two storage/schema states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityMatrix {
    api: CompatibilityLevel,
    semantic_model: CompatibilityLevel,
    storage_schema: CompatibilityLevel,
    storage_provider: CompatibilityLevel,
    index: CompatibilityLevel,
    version: CompatibilityLevel,
}

impl CompatibilityMatrix {
    /// Creates a matrix with all dimensions compatible by default.
    #[must_use]
    pub const fn compatible() -> Self {
        Self {
            api: CompatibilityLevel::Compatible,
            semantic_model: CompatibilityLevel::Compatible,
            storage_schema: CompatibilityLevel::Compatible,
            storage_provider: CompatibilityLevel::Compatible,
            index: CompatibilityLevel::Compatible,
            version: CompatibilityLevel::Compatible,
        }
    }

    #[must_use]
    pub const fn with_api(mut self, level: CompatibilityLevel) -> Self {
        self.api = level;
        self
    }
    #[must_use]
    pub const fn with_semantic_model(mut self, level: CompatibilityLevel) -> Self {
        self.semantic_model = level;
        self
    }
    #[must_use]
    pub const fn with_storage_schema(mut self, level: CompatibilityLevel) -> Self {
        self.storage_schema = level;
        self
    }
    #[must_use]
    pub const fn with_storage_provider(mut self, level: CompatibilityLevel) -> Self {
        self.storage_provider = level;
        self
    }
    #[must_use]
    pub const fn with_index(mut self, level: CompatibilityLevel) -> Self {
        self.index = level;
        self
    }
    #[must_use]
    pub const fn with_version(mut self, level: CompatibilityLevel) -> Self {
        self.version = level;
        self
    }

    /// Converts the declared matrix into an explicit compatibility report.
    #[must_use]
    pub fn report(&self) -> CompatibilityReport {
        let mut report = CompatibilityReport::new();
        report.push(CompatibilityResult::new(
            CompatibilityKind::Api,
            self.api,
            "declared API compatibility",
        ));
        report.push(CompatibilityResult::new(
            CompatibilityKind::SemanticModel,
            self.semantic_model,
            "declared semantic-model compatibility",
        ));
        report.push(CompatibilityResult::new(
            CompatibilityKind::StorageSchema,
            self.storage_schema,
            "declared storage-schema compatibility",
        ));
        report.push(CompatibilityResult::new(
            CompatibilityKind::StorageProvider,
            self.storage_provider,
            "declared storage-provider compatibility",
        ));
        report.push(CompatibilityResult::new(
            CompatibilityKind::Index,
            self.index,
            "declared derived-index compatibility",
        ));
        report.push(CompatibilityResult::new(
            CompatibilityKind::Version,
            self.version,
            "declared version compatibility",
        ));
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_schema_migration_does_not_imply_api_breakage() {
        let report = CompatibilityMatrix::compatible()
            .with_storage_schema(CompatibilityLevel::CompatibleWithMigration)
            .report();

        assert!(report.is_compatible());
        assert!(report.requires_migration());
        assert_eq!(
            report
                .results()
                .iter()
                .find(|result| result.kind() == CompatibilityKind::Api)
                .expect("API result should exist")
                .level(),
            CompatibilityLevel::Compatible
        );
    }

    #[test]
    fn incompatible_semantic_model_is_not_hidden_by_other_compatible_dimensions() {
        let report = CompatibilityMatrix::compatible()
            .with_semantic_model(CompatibilityLevel::Incompatible)
            .with_storage_schema(CompatibilityLevel::CompatibleWithMigration)
            .report();

        assert!(!report.is_compatible());
        assert!(report.requires_migration());
        assert_eq!(
            report
                .first_incompatibility()
                .expect("incompatibility should exist")
                .kind(),
            CompatibilityKind::SemanticModel
        );
    }

    #[test]
    fn compatibility_dimension_names_are_stable() {
        assert_eq!(CompatibilityKind::Api.as_str(), "api");
        assert_eq!(CompatibilityKind::SemanticModel.as_str(), "semantic_model");
        assert_eq!(CompatibilityKind::StorageSchema.as_str(), "storage_schema");
        assert_eq!(
            CompatibilityKind::StorageProvider.as_str(),
            "storage_provider"
        );
        assert_eq!(CompatibilityKind::Index.as_str(), "index");
        assert_eq!(CompatibilityKind::Version.as_str(), "version");
    }
}
