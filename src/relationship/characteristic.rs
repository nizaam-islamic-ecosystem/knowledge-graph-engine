//! Relationship characteristics for Phase 2.
//!
//! Characteristics describe structural or semantic properties of a
//! relationship. Phase 2 may expose safe structural behavior derived from
//! these characteristics, but it does not perform knowledge derivation.
//!
//! In particular, marking a relationship as transitive does not cause new
//! assertions to be generated. Controlled inference belongs to later
//! reasoning phases.

use core::fmt;
use std::collections::BTreeSet;

/// Phase 2 relationship characteristic.
///
/// Additional characteristics may be introduced by later phases as the
/// relationship model expands.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelationshipCharacteristic {
    /// The relationship permits a semantic reverse view using the same
    /// predicate.
    Symmetric,

    /// The relationship is explicitly non-symmetric.
    Asymmetric,

    /// The relationship has transitive semantic behavior available to later
    /// reasoning.
    Transitive,

    /// The relationship conceptually permits self-related instances.
    Reflexive,

    /// The relationship permits at most one target for a given source under
    /// its semantic interpretation.
    Functional,
}

impl RelationshipCharacteristic {
    /// Returns the stable textual representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Symmetric => "symmetric",
            Self::Asymmetric => "asymmetric",
            Self::Transitive => "transitive",
            Self::Reflexive => "reflexive",
            Self::Functional => "functional",
        }
    }
}

impl fmt::Display for RelationshipCharacteristic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Structural validation errors for relationship characteristics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipCharacteristicError {
    /// A relationship cannot simultaneously be symmetric and asymmetric.
    ConflictingSymmetryCharacteristics,
}

impl fmt::Display for RelationshipCharacteristicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingSymmetryCharacteristics => {
                formatter.write_str("relationship cannot be both symmetric and asymmetric")
            }
        }
    }
}

impl std::error::Error for RelationshipCharacteristicError {}

/// Deterministically ordered collection of relationship characteristics.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RelationshipCharacteristics {
    values: BTreeSet<RelationshipCharacteristic>,
}

impl RelationshipCharacteristics {
    /// Creates an empty characteristic set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a validated characteristic set.
    ///
    /// This constructor intentionally uses the `from_iter` name while being
    /// fallible because the standard `FromIterator` trait cannot represent
    /// structural validation errors. Implementing that trait here would either
    /// require panicking on invalid input or permit an invalid characteristic
    /// set, so the fallible constructor remains explicit.
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<I>(characteristics: I) -> Result<Self, RelationshipCharacteristicError>
    where
        I: IntoIterator<Item = RelationshipCharacteristic>,
    {
        let mut values = BTreeSet::new();

        for characteristic in characteristics {
            values.insert(characteristic);
        }

        Self::validate_values(&values)?;

        Ok(Self { values })
    }

    /// Inserts a characteristic while preserving structural validity.
    pub fn insert(
        &mut self,
        characteristic: RelationshipCharacteristic,
    ) -> Result<bool, RelationshipCharacteristicError> {
        let mut candidate = self.values.clone();
        let inserted = candidate.insert(characteristic);

        Self::validate_values(&candidate)?;

        self.values = candidate;

        Ok(inserted)
    }

    /// Returns whether the relationship has the requested characteristic.
    #[must_use]
    pub fn contains(&self, characteristic: RelationshipCharacteristic) -> bool {
        self.values.contains(&characteristic)
    }

    /// Returns whether the relationship is symmetric.
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        self.contains(RelationshipCharacteristic::Symmetric)
    }

    /// Returns whether the relationship is asymmetric.
    #[must_use]
    pub fn is_asymmetric(&self) -> bool {
        self.contains(RelationshipCharacteristic::Asymmetric)
    }

    /// Returns whether the relationship is transitive.
    ///
    /// This is metadata only. It does not execute transitive inference.
    #[must_use]
    pub fn is_transitive(&self) -> bool {
        self.contains(RelationshipCharacteristic::Transitive)
    }

    /// Returns whether the relationship is reflexive.
    ///
    /// This is metadata only. It does not generate reflexive assertions.
    #[must_use]
    pub fn is_reflexive(&self) -> bool {
        self.contains(RelationshipCharacteristic::Reflexive)
    }

    /// Returns whether the relationship is functional.
    ///
    /// This is metadata only. It does not enforce functional constraints
    /// against graph contents.
    #[must_use]
    pub fn is_functional(&self) -> bool {
        self.contains(RelationshipCharacteristic::Functional)
    }

    /// Returns whether structural reverse interpretation may use the same
    /// predicate.
    ///
    /// This is the Phase 2 structural behavior for symmetric relationships.
    /// Named inverse predicates remain the responsibility of the inverse
    /// relationship model.
    #[must_use]
    pub fn supports_structural_reverse_view(&self) -> bool {
        self.is_symmetric()
    }

    /// Returns the number of characteristics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no characteristics are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterates over characteristics in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = &RelationshipCharacteristic> {
        self.values.iter()
    }

    fn validate_values(
        values: &BTreeSet<RelationshipCharacteristic>,
    ) -> Result<(), RelationshipCharacteristicError> {
        let symmetric = values.contains(&RelationshipCharacteristic::Symmetric);
        let asymmetric = values.contains(&RelationshipCharacteristic::Asymmetric);

        if symmetric && asymmetric {
            return Err(RelationshipCharacteristicError::ConflictingSymmetryCharacteristics);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RelationshipCharacteristic, RelationshipCharacteristicError, RelationshipCharacteristics,
    };

    #[test]
    fn creates_empty_characteristics() {
        let characteristics = RelationshipCharacteristics::new();

        assert!(characteristics.is_empty());
    }

    #[test]
    fn characteristics_are_deterministically_ordered() {
        let characteristics = RelationshipCharacteristics::from_iter([
            RelationshipCharacteristic::Transitive,
            RelationshipCharacteristic::Symmetric,
            RelationshipCharacteristic::Functional,
        ])
        .expect("valid characteristics");

        let values = characteristics.iter().copied().collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                RelationshipCharacteristic::Symmetric,
                RelationshipCharacteristic::Transitive,
                RelationshipCharacteristic::Functional,
            ]
        );
    }

    #[test]
    fn symmetric_relationships_support_structural_reverse_views() {
        let characteristics =
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
                .expect("valid characteristics");

        assert!(characteristics.is_symmetric());
        assert!(characteristics.supports_structural_reverse_view());
    }

    #[test]
    fn transitivity_does_not_create_inference_behavior() {
        let characteristics =
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Transitive])
                .expect("valid characteristics");

        assert!(characteristics.is_transitive());
        assert!(!characteristics.supports_structural_reverse_view());
    }

    #[test]
    fn symmetric_and_asymmetric_are_structurally_incompatible() {
        let error = RelationshipCharacteristics::from_iter([
            RelationshipCharacteristic::Symmetric,
            RelationshipCharacteristic::Asymmetric,
        ])
        .expect_err("conflicting characteristics must be rejected");

        assert_eq!(
            error,
            RelationshipCharacteristicError::ConflictingSymmetryCharacteristics
        );
    }

    #[test]
    fn duplicate_characteristics_are_not_stored_twice() {
        let characteristics = RelationshipCharacteristics::from_iter([
            RelationshipCharacteristic::Functional,
            RelationshipCharacteristic::Functional,
        ])
        .expect("valid characteristics");

        assert_eq!(characteristics.len(), 1);
    }

    #[test]
    fn individual_characteristic_flags_are_available() {
        let characteristics = RelationshipCharacteristics::from_iter([
            RelationshipCharacteristic::Transitive,
            RelationshipCharacteristic::Reflexive,
            RelationshipCharacteristic::Functional,
        ])
        .expect("valid characteristics");

        assert!(characteristics.is_transitive());
        assert!(characteristics.is_reflexive());
        assert!(characteristics.is_functional());
        assert!(!characteristics.is_symmetric());
        assert!(!characteristics.is_asymmetric());
    }
}
