//! Extensible ontology constraints and validation reporting for Phase 3.
//!
//! Phase 3 keeps the constraint vocabulary deliberately small and structural:
//! cardinality, class disjointness, and an explicit extension escape hatch.
//! Domain and range are represented directly by ontology properties as sets of
//! [`super::class::ClassId`].
//!
//! Constraint construction performs local structural validation. Ontology-wide
//! validation, such as checking that a referenced class is actually registered,
//! remains the responsibility of [`super::model::Ontology`].

use core::fmt;
use std::collections::BTreeSet;

use super::class::ClassId;

/// Deterministically ordered set of ontology class identities.
///
/// Ontology properties use this representation for both domain and range.
pub type ClassSet = BTreeSet<ClassId>;

/// A minimal cardinality constraint.
///
/// `None` represents an unbounded side of the interval.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CardinalityConstraint {
    min: Option<usize>,
    max: Option<usize>,
}

impl CardinalityConstraint {
    /// Creates a cardinality interval.
    ///
    /// The interval is valid only when both bounds, when present, satisfy
    /// `min <= max`.
    pub fn new(min: Option<usize>, max: Option<usize>) -> Result<Self, OntologyConstraintError> {
        if let (Some(min), Some(max)) = (min, max)
            && min > max
        {
            return Err(OntologyConstraintError::InvalidCardinalityBounds {
                min: Some(min),
                max: Some(max),
            });
        }

        Ok(Self { min, max })
    }

    /// Returns the minimum cardinality, when bounded.
    #[must_use]
    pub const fn min(self) -> Option<usize> {
        self.min
    }

    /// Returns the maximum cardinality, when bounded.
    #[must_use]
    pub const fn max(self) -> Option<usize> {
        self.max
    }
}

/// An explicit extensibility payload for constraint kinds that Phase 3 does
/// not standardize yet.
///
/// Extensions are intentionally opaque to the ontology core. A later phase or
/// domain package may define the meaning without changing the closed core
/// constraint variants.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConstraintExtension {
    name: String,
    value: String,
}

impl ConstraintExtension {
    /// Creates an extension constraint payload.
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, OntologyConstraintError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(OntologyConstraintError::EmptyExtensionName);
        }

        if let Some(index) = name.chars().position(char::is_control) {
            return Err(OntologyConstraintError::ExtensionNameControlCharacter { index });
        }

        Ok(Self {
            name,
            value: value.into(),
        })
    }

    /// Returns the extension kind/name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the opaque extension value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// The closed Phase 3 core constraint vocabulary.
///
/// The explicit [`Self::Extension`] variant is the controlled escape hatch for
/// future constraint kinds. It does not turn the ontology model into an
/// untyped dynamic constraint system.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OntologyConstraint {
    /// Minimum/maximum cardinality information.
    Cardinality(CardinalityConstraint),

    /// Declares that the owning class/property context is disjoint with the
    /// listed class identities.
    DisjointWith(ClassSet),

    /// Explicit extension data for a constraint kind not standardized in
    /// Phase 3.
    Extension(ConstraintExtension),
}

impl OntologyConstraint {
    /// Creates a cardinality constraint.
    pub fn cardinality(
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<Self, OntologyConstraintError> {
        Ok(Self::Cardinality(CardinalityConstraint::new(min, max)?))
    }

    /// Creates a disjointness constraint.
    pub fn disjoint_with<I>(classes: I) -> Result<Self, OntologyConstraintError>
    where
        I: IntoIterator<Item = ClassId>,
    {
        let classes = classes.into_iter().collect::<ClassSet>();

        if classes.is_empty() {
            return Err(OntologyConstraintError::EmptyDisjointClassSet);
        }

        Ok(Self::DisjointWith(classes))
    }

    /// Creates an extension constraint.
    pub fn extension(
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, OntologyConstraintError> {
        Ok(Self::Extension(ConstraintExtension::new(name, value)?))
    }
}

/// A deterministic set of ontology constraints.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConstraintSet {
    values: BTreeSet<OntologyConstraint>,
}

impl ConstraintSet {
    /// Creates an empty constraint set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a constraint, returning whether it was newly inserted.
    pub fn insert(&mut self, constraint: OntologyConstraint) -> bool {
        self.values.insert(constraint)
    }

    /// Returns whether the set contains the supplied constraint.
    #[must_use]
    pub fn contains(&self, constraint: &OntologyConstraint) -> bool {
        self.values.contains(constraint)
    }

    /// Returns the number of stored constraints.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the set contains no constraints.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterates over constraints in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &OntologyConstraint> {
        self.values.iter()
    }
}

impl FromIterator<OntologyConstraint> for ConstraintSet {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = OntologyConstraint>,
    {
        let mut set = Self::new();

        for constraint in iter {
            set.insert(constraint);
        }

        set
    }
}

/// Structural failures while constructing a Phase 3 ontology constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OntologyConstraintError {
    /// The supplied minimum cardinality is greater than the maximum.
    InvalidCardinalityBounds {
        /// Supplied minimum bound.
        min: Option<usize>,

        /// Supplied maximum bound.
        max: Option<usize>,
    },

    /// A disjointness constraint must name at least one class.
    EmptyDisjointClassSet,

    /// Extension names must not be empty or whitespace-only.
    EmptyExtensionName,

    /// Extension names must not contain Unicode control characters.
    ExtensionNameControlCharacter {
        /// Character index of the control character.
        index: usize,
    },
}

impl fmt::Display for OntologyConstraintError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCardinalityBounds { min, max } => {
                write!(
                    formatter,
                    "invalid cardinality bounds: min={min:?}, max={max:?}"
                )
            }
            Self::EmptyDisjointClassSet => {
                formatter.write_str("disjointness constraint must reference at least one class")
            }
            Self::EmptyExtensionName => {
                formatter.write_str("constraint extension name must not be empty")
            }
            Self::ExtensionNameControlCharacter { index } => write!(
                formatter,
                "constraint extension name contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for OntologyConstraintError {}

/// One ontology validation issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    code: String,
    message: String,
}

impl ValidationIssue {
    /// Creates a validation issue with a stable caller-defined code.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Returns the validation issue code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the human-readable validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Aggregated ontology validation result.
///
/// Phase 3 distinguishes warnings from errors and makes validity explicit.
/// It is intentionally a small reporting mechanism rather than a full
/// validation framework.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    warnings: Vec<ValidationIssue>,
    errors: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Creates an empty valid report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a warning to the report.
    pub fn add_warning(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ValidationIssue::new(code, message));
    }

    /// Adds an error to the report.
    pub fn add_error(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationIssue::new(code, message));
    }

    /// Merges another report into this report.
    pub fn merge(&mut self, other: Self) {
        self.warnings.extend(other.warnings);
        self.errors.extend(other.errors);
    }

    /// Returns all warnings in insertion order.
    #[must_use]
    pub fn warnings(&self) -> &[ValidationIssue] {
        &self.warnings
    }

    /// Returns all errors in insertion order.
    #[must_use]
    pub fn errors(&self) -> &[ValidationIssue] {
        &self.errors
    }

    /// Returns whether the report contains no validation errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns whether the report contains at least one warning.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CardinalityConstraint, ConstraintExtension, ConstraintSet, OntologyConstraint,
        OntologyConstraintError, ValidationReport,
    };
    use crate::ontology::class::ClassId;

    fn class_id(value: &str) -> ClassId {
        ClassId::new(value).expect("valid class identity")
    }

    #[test]
    fn cardinality_constraints_preserve_both_bounds() {
        let constraint =
            CardinalityConstraint::new(Some(1), Some(3)).expect("valid cardinality bounds");

        assert_eq!(constraint.min(), Some(1));
        assert_eq!(constraint.max(), Some(3));
    }

    #[test]
    fn unbounded_cardinality_sides_are_supported() {
        let lower_bounded =
            CardinalityConstraint::new(Some(1), None).expect("valid lower-bounded cardinality");
        let upper_bounded =
            CardinalityConstraint::new(None, Some(3)).expect("valid upper-bounded cardinality");

        assert_eq!(lower_bounded.min(), Some(1));
        assert_eq!(lower_bounded.max(), None);
        assert_eq!(upper_bounded.min(), None);
        assert_eq!(upper_bounded.max(), Some(3));
    }

    #[test]
    fn invalid_cardinality_bounds_are_rejected() {
        assert_eq!(
            CardinalityConstraint::new(Some(4), Some(2)),
            Err(OntologyConstraintError::InvalidCardinalityBounds {
                min: Some(4),
                max: Some(2),
            })
        );
    }

    #[test]
    fn disjointness_requires_at_least_one_class() {
        assert_eq!(
            OntologyConstraint::disjoint_with(Vec::<ClassId>::new()),
            Err(OntologyConstraintError::EmptyDisjointClassSet)
        );

        let constraint = OntologyConstraint::disjoint_with([class_id("class-person")])
            .expect("valid disjointness constraint");

        assert!(matches!(constraint, OntologyConstraint::DisjointWith(_)));
    }

    #[test]
    fn extension_constraints_are_explicit_and_opaque() {
        let extension =
            ConstraintExtension::new("future.constraint", "opaque-value").expect("valid extension");

        assert_eq!(extension.name(), "future.constraint");
        assert_eq!(extension.value(), "opaque-value");

        let constraint = OntologyConstraint::extension("future.constraint", "opaque-value")
            .expect("valid extension constraint");

        assert!(matches!(constraint, OntologyConstraint::Extension(_)));
    }

    #[test]
    fn extension_names_are_structurally_validated() {
        assert_eq!(
            ConstraintExtension::new("   ", "value"),
            Err(OntologyConstraintError::EmptyExtensionName)
        );

        assert_eq!(
            ConstraintExtension::new("future\nconstraint", "value"),
            Err(OntologyConstraintError::ExtensionNameControlCharacter { index: 6 })
        );
    }

    #[test]
    fn constraint_sets_are_deterministic_and_deduplicated() {
        let first = OntologyConstraint::cardinality(Some(1), Some(1)).expect("valid cardinality");
        let second = OntologyConstraint::disjoint_with([class_id("class-person")])
            .expect("valid disjointness");

        let mut constraints = ConstraintSet::new();
        assert!(constraints.insert(second.clone()));
        assert!(constraints.insert(first.clone()));
        assert!(!constraints.insert(first.clone()));

        assert_eq!(constraints.len(), 2);
        assert!(constraints.contains(&first));
        assert!(constraints.contains(&second));
    }

    #[test]
    fn validation_report_tracks_warnings_errors_and_validity() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid());
        assert!(!report.has_warnings());

        report.add_warning("ontology.warning", "example warning");
        assert!(report.is_valid());
        assert!(report.has_warnings());

        report.add_error("ontology.error", "example error");
        assert!(!report.is_valid());
        assert_eq!(report.warnings()[0].code(), "ontology.warning");
        assert_eq!(report.errors()[0].message(), "example error");
    }

    #[test]
    fn validation_reports_can_be_merged() {
        let mut first = ValidationReport::new();
        first.add_warning("warning.one", "one");

        let mut second = ValidationReport::new();
        second.add_error("error.one", "one");

        first.merge(second);

        assert_eq!(first.warnings().len(), 1);
        assert_eq!(first.errors().len(), 1);
        assert!(!first.is_valid());
    }
}
