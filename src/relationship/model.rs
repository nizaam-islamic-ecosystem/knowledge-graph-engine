//! Core relationship model and Phase 2 relationship vocabulary.
//!
//! This module defines:
//! - relationship families;
//! - relationship definitions;
//! - explicit inverse information;
//! - relationship characteristics;
//! - composition-rule declarations;
//! - the in-memory relationship vocabulary.
//!
//! Phase 2 does not execute reasoning or composition. Composition rules are
//! represented so later reasoning infrastructure can consume them.
//!
//! This module also intentionally does not perform ontology/domain/range
//! validation. Those semantics belong to later ontology/semantic phases.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::characteristic::RelationshipCharacteristics;
use super::direction::RelationshipDirection;
use super::inverse::{InverseRelationship, InverseRelationshipError};
use super::predicate::RelationshipPredicate;

/// Standard Phase 2 relationship-family names.
///
/// These names establish the initial family vocabulary but do not prevent
/// future families from being introduced through `RelationshipFamily::new`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RelationshipFamily(String);

impl RelationshipFamily {
    /// Standard identity-family name.
    pub const IDENTITY: &'static str = "identity";

    /// Standard lexical-family name.
    pub const LEXICAL: &'static str = "lexical";

    /// Standard linguistic-family name.
    pub const LINGUISTIC: &'static str = "linguistic";

    /// Standard semantic-family name.
    pub const SEMANTIC: &'static str = "semantic";

    /// Standard conceptual-family name.
    pub const CONCEPTUAL: &'static str = "conceptual";

    /// Standard hierarchical-family name.
    pub const HIERARCHICAL: &'static str = "hierarchical";

    /// Standard part-whole-family name.
    pub const PART_WHOLE: &'static str = "part-whole";

    /// Standard reference-family name.
    pub const REFERENCE: &'static str = "reference";

    /// Standard temporal-family name.
    pub const TEMPORAL: &'static str = "temporal";

    /// Standard causal-family name.
    pub const CAUSAL: &'static str = "causal";

    /// Standard logical-family name.
    pub const LOGICAL: &'static str = "logical";

    /// Standard knowledge-family name.
    pub const KNOWLEDGE: &'static str = "knowledge";

    /// Creates a relationship family from its canonical name.
    ///
    /// Family names are intentionally extensible. Only structural validation
    /// is performed here.
    pub fn new(name: impl Into<String>) -> Result<Self, RelationshipFamilyError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(RelationshipFamilyError::EmptyName);
        }

        if let Some(index) = name.chars().position(char::is_control) {
            return Err(RelationshipFamilyError::ControlCharacter { index });
        }

        Ok(Self(name))
    }

    /// Returns the canonical family name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RelationshipFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Structural validation failures for a relationship family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipFamilyError {
    /// The supplied family name is empty or whitespace-only.
    EmptyName,

    /// The family name contains a Unicode control character.
    ControlCharacter { index: usize },
}

impl fmt::Display for RelationshipFamilyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("relationship family name must not be empty"),
            Self::ControlCharacter { index } => write!(
                formatter,
                "relationship family name contains a control character at index {index}",
            ),
        }
    }
}

impl std::error::Error for RelationshipFamilyError {}

/// A declaration describing how two relationship predicates compose into a
/// possible third predicate.
///
/// Phase 2 records the rule only. It does not execute composition or derive
/// assertions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CompositionRule {
    first: RelationshipPredicate,
    second: RelationshipPredicate,
    result: RelationshipPredicate,
}

impl CompositionRule {
    /// Creates a composition rule:
    ///
    /// ```text
    /// first + second -> result
    /// ```
    #[must_use]
    pub fn new(
        first: RelationshipPredicate,
        second: RelationshipPredicate,
        result: RelationshipPredicate,
    ) -> Self {
        Self {
            first,
            second,
            result,
        }
    }

    /// Returns the first relationship in the composition.
    #[must_use]
    pub fn first(&self) -> &RelationshipPredicate {
        &self.first
    }

    /// Returns the second relationship in the composition.
    #[must_use]
    pub fn second(&self) -> &RelationshipPredicate {
        &self.second
    }

    /// Returns the relationship produced by the declared composition.
    #[must_use]
    pub fn result(&self) -> &RelationshipPredicate {
        &self.result
    }
}

/// Structural validation failures for a relationship definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipError {
    /// The inverse declaration does not describe the relationship's own
    /// predicate as its canonical side.
    InversePredicateMismatch {
        /// Predicate owned by the relationship definition.
        relationship: RelationshipPredicate,

        /// Canonical predicate stored in the inverse declaration.
        declared: RelationshipPredicate,
    },

    /// An inverse declaration is structurally invalid.
    InvalidInverse(InverseRelationshipError),
}

impl fmt::Display for RelationshipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InversePredicateMismatch {
                relationship,
                declared,
            } => write!(
                formatter,
                "inverse declaration predicate {declared} does not match relationship predicate {relationship}",
            ),
            Self::InvalidInverse(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RelationshipError {}

/// A first-class Phase 2 relationship definition.
///
/// A relationship combines its strongly typed predicate with structural
/// semantics such as family, direction, characteristics, inverse information,
/// and deferred composition rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relationship {
    predicate: RelationshipPredicate,
    family: RelationshipFamily,
    direction: RelationshipDirection,
    characteristics: RelationshipCharacteristics,
    inverse: Option<InverseRelationship>,
    composition_rules: BTreeSet<CompositionRule>,
}

impl Relationship {
    /// Creates a relationship definition.
    #[must_use]
    pub fn new(
        predicate: RelationshipPredicate,
        family: RelationshipFamily,
        direction: RelationshipDirection,
        characteristics: RelationshipCharacteristics,
    ) -> Self {
        Self {
            predicate,
            family,
            direction,
            characteristics,
            inverse: None,
            composition_rules: BTreeSet::new(),
        }
    }

    /// Returns the relationship predicate.
    #[must_use]
    pub fn predicate(&self) -> &RelationshipPredicate {
        &self.predicate
    }

    /// Returns the relationship family.
    #[must_use]
    pub fn family(&self) -> &RelationshipFamily {
        &self.family
    }

    /// Returns the semantic direction.
    #[must_use]
    pub fn direction(&self) -> RelationshipDirection {
        self.direction
    }

    /// Returns the relationship characteristics.
    #[must_use]
    pub fn characteristics(&self) -> &RelationshipCharacteristics {
        &self.characteristics
    }

    /// Returns the explicit inverse declaration, when one exists.
    #[must_use]
    pub fn inverse(&self) -> Option<&InverseRelationship> {
        self.inverse.as_ref()
    }

    /// Attaches an explicit inverse declaration.
    ///
    /// The declaration must identify this relationship's own predicate as its
    /// canonical predicate.
    pub fn with_inverse(mut self, inverse: InverseRelationship) -> Result<Self, RelationshipError> {
        if inverse.predicate() != &self.predicate {
            return Err(RelationshipError::InversePredicateMismatch {
                relationship: self.predicate.clone(),
                declared: inverse.predicate().clone(),
            });
        }

        self.inverse = Some(inverse);

        Ok(self)
    }

    /// Convenience constructor for an inverse predicate.
    pub fn with_inverse_predicate(
        self,
        inverse_predicate: RelationshipPredicate,
    ) -> Result<Self, RelationshipError> {
        let inverse = InverseRelationship::new(self.predicate.clone(), inverse_predicate)
            .map_err(RelationshipError::InvalidInverse)?;

        self.with_inverse(inverse)
    }

    /// Returns whether this relationship is symmetric.
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        self.characteristics.is_symmetric()
    }

    /// Returns whether this relationship is asymmetric.
    #[must_use]
    pub fn is_asymmetric(&self) -> bool {
        self.characteristics.is_asymmetric()
    }

    /// Returns whether this relationship is transitive.
    ///
    /// This is a declaration only. No transitive assertions are derived.
    #[must_use]
    pub fn is_transitive(&self) -> bool {
        self.characteristics.is_transitive()
    }

    /// Returns whether this relationship is reflexive.
    ///
    /// This is a declaration only. No reflexive assertions are derived.
    #[must_use]
    pub fn is_reflexive(&self) -> bool {
        self.characteristics.is_reflexive()
    }

    /// Returns whether this relationship is functional.
    ///
    /// This is a declaration only. No graph constraint is executed here.
    #[must_use]
    pub fn is_functional(&self) -> bool {
        self.characteristics.is_functional()
    }

    /// Adds a composition rule without executing it.
    pub fn with_composition_rule(mut self, rule: CompositionRule) -> Self {
        self.composition_rules.insert(rule);
        self
    }

    /// Returns all declared composition rules.
    pub fn composition_rules(&self) -> impl Iterator<Item = &CompositionRule> {
        self.composition_rules.iter()
    }

    /// Returns the number of composition rules.
    #[must_use]
    pub fn composition_rule_count(&self) -> usize {
        self.composition_rules.len()
    }
}

/// Errors produced by the relationship vocabulary registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipVocabularyError {
    /// A predicate is already registered.
    PredicateAlreadyRegistered {
        /// Predicate that was already present.
        predicate: RelationshipPredicate,
    },
}

impl fmt::Display for RelationshipVocabularyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PredicateAlreadyRegistered { predicate } => write!(
                formatter,
                "relationship predicate is already registered: {predicate}",
            ),
        }
    }
}

impl std::error::Error for RelationshipVocabularyError {}

/// In-memory relationship vocabulary registry.
///
/// The registry is intentionally small and deterministic. It is a logical
/// semantic registry, not a persistence system or general application-logic
/// container.
#[derive(Clone, Debug, Default)]
pub struct RelationshipVocabulary {
    relationships: BTreeMap<RelationshipPredicate, Relationship>,
}

impl RelationshipVocabulary {
    /// Creates an empty relationship vocabulary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a relationship definition.
    ///
    /// Duplicate predicates are rejected so one predicate has one canonical
    /// relationship definition within a vocabulary.
    pub fn register(
        &mut self,
        relationship: Relationship,
    ) -> Result<(), RelationshipVocabularyError> {
        let predicate = relationship.predicate().clone();

        if self.relationships.contains_key(&predicate) {
            return Err(RelationshipVocabularyError::PredicateAlreadyRegistered { predicate });
        }

        self.relationships.insert(predicate, relationship);

        Ok(())
    }

    /// Returns a relationship definition by predicate.
    #[must_use]
    pub fn get(&self, predicate: &RelationshipPredicate) -> Option<&Relationship> {
        self.relationships.get(predicate)
    }

    /// Returns whether a predicate is registered.
    #[must_use]
    pub fn contains(&self, predicate: &RelationshipPredicate) -> bool {
        self.relationships.contains_key(predicate)
    }

    /// Returns the number of registered relationships.
    #[must_use]
    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    /// Returns whether the vocabulary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    /// Iterates over definitions in deterministic predicate order.
    pub fn iter(&self) -> impl Iterator<Item = (&RelationshipPredicate, &Relationship)> {
        self.relationships.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompositionRule, Relationship, RelationshipFamily, RelationshipVocabulary,
        RelationshipVocabularyError,
    };
    use crate::relationship::characteristic::RelationshipCharacteristics;
    use crate::relationship::direction::RelationshipDirection;
    use crate::relationship::predicate::RelationshipPredicate;

    fn predicate(name: &str) -> RelationshipPredicate {
        RelationshipPredicate::new(name).expect("valid relationship predicate")
    }

    fn identity_family() -> RelationshipFamily {
        RelationshipFamily::new(RelationshipFamily::IDENTITY).expect("valid relationship family")
    }

    #[test]
    fn relationship_family_is_extensible() {
        let family =
            RelationshipFamily::new("future-family").expect("custom family should be valid");

        assert_eq!(family.as_str(), "future-family");
    }

    #[test]
    fn empty_relationship_family_is_rejected() {
        assert!(RelationshipFamily::new("").is_err());
        assert!(RelationshipFamily::new("   ").is_err());
    }

    #[test]
    fn creates_a_relationship_definition() {
        let relationship = Relationship::new(
            predicate("has-name"),
            identity_family(),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        assert_eq!(
            relationship.predicate().as_str(),
            "kg.relationship.has-name"
        );

        assert_eq!(relationship.family().as_str(), RelationshipFamily::IDENTITY);

        assert_eq!(
            relationship.direction(),
            RelationshipDirection::SubjectToObject
        );

        assert!(relationship.inverse().is_none());
    }

    #[test]
    fn attaches_an_explicit_inverse_relationship() {
        let relationship = Relationship::new(
            predicate("has-name"),
            identity_family(),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        )
        .with_inverse_predicate(predicate("name-of"))
        .expect("valid inverse");

        let inverse = relationship.inverse().expect("inverse should be present");

        assert_eq!(inverse.predicate().as_str(), "kg.relationship.has-name");

        assert_eq!(
            inverse.inverse_predicate().as_str(),
            "kg.relationship.name-of"
        );
    }

    #[test]
    fn rejects_an_inverse_for_a_different_canonical_predicate() {
        let relationship = Relationship::new(
            predicate("has-name"),
            identity_family(),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        let inverse = crate::relationship::inverse::InverseRelationship::new(
            predicate("aliases"),
            predicate("name-of"),
        )
        .expect("valid inverse pair");

        let error = relationship
            .with_inverse(inverse)
            .expect_err("mismatched canonical predicate must be rejected");

        match error {
            super::RelationshipError::InversePredicateMismatch {
                relationship,
                declared,
            } => {
                assert_eq!(relationship.as_str(), "kg.relationship.has-name");
                assert_eq!(declared.as_str(), "kg.relationship.aliases");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn composition_rules_are_declared_without_execution() {
        let first = predicate("part-of");
        let second = predicate("part-of");
        let result = predicate("part-of");

        let rule = CompositionRule::new(first.clone(), second.clone(), result.clone());

        let relationship = Relationship::new(
            first,
            RelationshipFamily::new(RelationshipFamily::PART_WHOLE).expect("valid family"),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        )
        .with_composition_rule(rule);

        let rule = relationship
            .composition_rules()
            .next()
            .expect("composition rule should be present");

        assert_eq!(rule.first(), &second);
        assert_eq!(rule.second(), &second);
        assert_eq!(rule.result(), &result);
        assert_eq!(relationship.composition_rule_count(), 1);
    }

    #[test]
    fn relationship_characteristics_are_exposed_structurally() {
        let characteristics = RelationshipCharacteristics::from_iter([
            crate::relationship::characteristic::RelationshipCharacteristic::Symmetric,
        ])
        .expect("valid characteristics");

        let relationship = Relationship::new(
            predicate("aliases"),
            identity_family(),
            RelationshipDirection::SubjectToObject,
            characteristics,
        );

        assert!(relationship.is_symmetric());
        assert!(!relationship.is_asymmetric());
    }

    #[test]
    fn relationship_vocabulary_registers_and_resolves_definitions() {
        let relationship = Relationship::new(
            predicate("has-name"),
            identity_family(),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        let predicate = relationship.predicate().clone();

        let mut vocabulary = RelationshipVocabulary::new();

        vocabulary
            .register(relationship)
            .expect("relationship should register");

        assert_eq!(vocabulary.len(), 1);
        assert!(vocabulary.contains(&predicate));
        assert!(vocabulary.get(&predicate).is_some());
    }

    #[test]
    fn duplicate_relationship_predicates_are_rejected() {
        let relationship = Relationship::new(
            predicate("has-name"),
            identity_family(),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        let duplicate = relationship.clone();

        let mut vocabulary = RelationshipVocabulary::new();

        vocabulary
            .register(relationship)
            .expect("first registration should succeed");

        let error = vocabulary
            .register(duplicate)
            .expect_err("duplicate predicate must be rejected");

        assert_eq!(
            error,
            RelationshipVocabularyError::PredicateAlreadyRegistered {
                predicate: predicate("has-name"),
            }
        );
    }

    #[test]
    fn vocabulary_iteration_is_deterministic() {
        let mut vocabulary = RelationshipVocabulary::new();

        vocabulary
            .register(Relationship::new(
                predicate("z"),
                identity_family(),
                RelationshipDirection::SubjectToObject,
                RelationshipCharacteristics::new(),
            ))
            .expect("valid relationship");

        vocabulary
            .register(Relationship::new(
                predicate("a"),
                identity_family(),
                RelationshipDirection::SubjectToObject,
                RelationshipCharacteristics::new(),
            ))
            .expect("valid relationship");

        let names = vocabulary
            .iter()
            .map(|(predicate, _)| predicate.name())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["a", "z"]);
    }
}
