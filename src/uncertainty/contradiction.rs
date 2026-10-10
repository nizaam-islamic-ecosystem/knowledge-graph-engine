//! First-class contradictions and conservative deterministic detection.
//!
//! This module records explicit conflicts and allows callers to inspect a full
//! conflict set or request a simple current view. It does not choose a winning
//! assertion, erase either side, or perform advanced semantic reasoning.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, KnowledgeAssertion,
    Qualifiers,
};
use crate::identity::{ContradictionId, EvidenceId, KnowledgeAssertionId};

/// Structural kind of a contradiction.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContradictionKind {
    /// The same proposition is asserted with both positive and negative polarity.
    OpposedPolarity,
    /// Two positive values disagree for a predicate declared functional.
    FunctionalValueConflict,
    /// A domain-specific contradiction type.
    Custom(String),
}

impl ContradictionKind {
    /// Creates a validated domain-specific kind label.
    pub fn custom(name: impl Into<String>) -> Result<Self, ContradictionError> {
        let name = name.into();
        validate_label(&name, "contradiction kind")?;
        Ok(Self::Custom(name))
    }

    /// Validates the public custom variant as well as constructed values.
    pub fn validate(&self) -> Result<(), ContradictionError> {
        if let Self::Custom(name) = self {
            validate_label(name, "contradiction kind")?;
        }
        Ok(())
    }
}

/// Current review state of a contradiction record.
///
/// `Dismissed` means the conflict record was dismissed as a detected conflict;
/// it does not erase the assertions or claim that a general reasoning engine
/// has resolved their underlying semantic disagreement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub enum ContradictionStatus {
    /// The contradiction has been recorded.
    #[default]
    Detected,
    /// The contradiction is being reviewed.
    UnderReview,
    /// The contradiction was acknowledged but remains inspectable.
    Acknowledged,
    /// The contradiction finding was dismissed.
    Dismissed,
}

/// A structural finding produced by deterministic comparison of two assertions.
///
/// It has no identity yet. The caller supplies a Core-backed `ContradictionId`
/// when converting it into a persistent-domain value, keeping detection pure and
/// making repeated detection deterministic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContradictionFinding {
    assertion_ids: BTreeSet<KnowledgeAssertionId>,
    involved_objects: BTreeSet<AssertionObject>,
    kind: ContradictionKind,
    context: AssertionContext,
    qualifiers: Qualifiers,
}

impl ContradictionFinding {
    /// Returns the distinct canonical assertion identities involved.
    #[must_use]
    pub fn assertion_ids(&self) -> &BTreeSet<KnowledgeAssertionId> {
        &self.assertion_ids
    }

    /// Returns the distinct typed object values involved in the finding.
    ///
    /// Opposed-polarity findings normally contain one value, while functional
    /// value conflicts contain the distinct values asserted by both sides.
    #[must_use]
    pub fn involved_objects(&self) -> &BTreeSet<AssertionObject> {
        &self.involved_objects
    }

    /// Returns the detected structural conflict kind.
    #[must_use]
    pub fn kind(&self) -> &ContradictionKind {
        &self.kind
    }

    /// Creates a contradiction record using a caller-supplied Core identity.
    pub fn into_contradiction(
        self,
        id: ContradictionId,
    ) -> Result<Contradiction, ContradictionError> {
        let Self {
            assertion_ids,
            involved_objects,
            kind,
            context,
            qualifiers,
        } = self;

        Contradiction::new(id, assertion_ids, kind, context, qualifiers)
            .map(|contradiction| contradiction.with_involved_objects(involved_objects))
    }
}

/// A first-class, inspectable contradiction between canonical assertions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contradiction {
    id: ContradictionId,
    assertion_ids: BTreeSet<KnowledgeAssertionId>,
    involved_objects: BTreeSet<AssertionObject>,
    kind: ContradictionKind,
    context: AssertionContext,
    qualifiers: Qualifiers,
    status: ContradictionStatus,
    supporting_evidence: BTreeSet<EvidenceId>,
    notes: Option<String>,
}

impl Contradiction {
    /// Creates a contradiction record relating at least two distinct assertions.
    pub fn new<I>(
        id: ContradictionId,
        assertion_ids: I,
        kind: ContradictionKind,
        context: AssertionContext,
        qualifiers: Qualifiers,
    ) -> Result<Self, ContradictionError>
    where
        I: IntoIterator<Item = KnowledgeAssertionId>,
    {
        kind.validate()?;
        let assertion_ids = assertion_ids.into_iter().collect::<BTreeSet<_>>();
        if assertion_ids.len() < 2 {
            return Err(ContradictionError::RequiresTwoDistinctAssertions);
        }

        Ok(Self {
            id,
            assertion_ids,
            involved_objects: BTreeSet::new(),
            kind,
            context,
            qualifiers,
            status: ContradictionStatus::Detected,
            supporting_evidence: BTreeSet::new(),
            notes: None,
        })
    }

    /// Returns the Core-backed contradiction identity.
    #[must_use]
    pub fn id(&self) -> &ContradictionId {
        &self.id
    }

    /// Returns the canonical assertions involved in the contradiction.
    #[must_use]
    pub fn assertion_ids(&self) -> &BTreeSet<KnowledgeAssertionId> {
        &self.assertion_ids
    }

    /// Returns the distinct typed object values involved in this contradiction.
    ///
    /// Findings produced by [`detect_contradiction`] carry these values through
    /// conversion. Manually created records can populate them with
    /// [`Self::with_involved_objects`].
    #[must_use]
    pub fn involved_objects(&self) -> &BTreeSet<AssertionObject> {
        &self.involved_objects
    }

    /// Returns the structural contradiction kind.
    #[must_use]
    pub fn kind(&self) -> &ContradictionKind {
        &self.kind
    }

    /// Returns a new value that preserves the typed object values involved.
    ///
    /// This is useful for manually curated contradictions whose assertions are
    /// identified by ID but whose involved values should also be directly
    /// inspectable. Duplicate objects are stored once, in deterministic order.
    #[must_use]
    pub fn with_involved_objects<I>(mut self, objects: I) -> Self
    where
        I: IntoIterator<Item = AssertionObject>,
    {
        self.involved_objects.extend(objects);
        self
    }

    /// Returns the shared assertion context preserved by detection or curation.
    #[must_use]
    pub fn context(&self) -> &AssertionContext {
        &self.context
    }

    /// Returns the shared qualifier scope preserved by detection or curation.
    #[must_use]
    pub fn qualifiers(&self) -> &Qualifiers {
        &self.qualifiers
    }

    /// Returns the current record status.
    #[must_use]
    pub const fn status(&self) -> ContradictionStatus {
        self.status
    }

    /// Returns supporting evidence identities attached to the conflict record.
    #[must_use]
    pub fn supporting_evidence(&self) -> &BTreeSet<EvidenceId> {
        &self.supporting_evidence
    }

    /// Returns the optional explanatory note.
    #[must_use]
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    /// Returns a new value with additional supporting evidence references.
    #[must_use]
    pub fn with_supporting_evidence<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.supporting_evidence.extend(ids);
        self
    }

    /// Returns a new value with a changed current review status.
    #[must_use]
    pub fn with_status(mut self, status: ContradictionStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns a new value with an explanatory note.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Result<Self, ContradictionError> {
        let notes = notes.into();
        validate_label(&notes, "contradiction note")?;
        self.notes = Some(notes);
        Ok(self)
    }
}

/// Conservative policy for choosing a simple current assertion view.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub enum CurrentViewPolicy {
    /// Keep every supplied assertion visible, including conflicts.
    #[default]
    IncludeAll,
    /// Hide every assertion involved in a non-dismissed contradiction; never
    /// select a winner on behalf of the caller.
    ExcludeActiveConflicts,
}

/// In-memory contradiction collection for deterministic inspection.
///
/// This is a domain collection, not persistence or a full query engine.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContradictionSet {
    values: BTreeMap<ContradictionId, Contradiction>,
}

impl ContradictionSet {
    /// Creates an empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a contradiction while rejecting duplicate record identities.
    pub fn insert(&mut self, contradiction: Contradiction) -> Result<bool, ContradictionError> {
        let id = contradiction.id().clone();
        if self.values.contains_key(&id) {
            return Err(ContradictionError::DuplicateIdentity);
        }
        self.values.insert(id, contradiction);
        Ok(true)
    }

    /// Returns a contradiction by identity.
    #[must_use]
    pub fn get(&self, id: &ContradictionId) -> Option<&Contradiction> {
        self.values.get(id)
    }

    /// Iterates through all contradictions in deterministic identity order.
    pub fn iter(&self) -> impl Iterator<Item = &Contradiction> {
        self.values.values()
    }

    /// Returns all records involving an assertion, including dismissed findings.
    pub fn for_assertion(&self, id: &KnowledgeAssertionId) -> Vec<&Contradiction> {
        self.values
            .values()
            .filter(|value| value.assertion_ids().contains(id))
            .collect()
    }

    /// Returns whether an assertion participates in any non-dismissed conflict.
    #[must_use]
    pub fn has_active_conflict(&self, id: &KnowledgeAssertionId) -> bool {
        self.values.values().any(|contradiction| {
            contradiction.status() != ContradictionStatus::Dismissed
                && contradiction.assertion_ids().contains(id)
        })
    }

    /// Selects a conservative current view from already-loaded assertions.
    ///
    /// `ExcludeActiveConflicts` hides all sides of an active conflict rather than
    /// implicitly resolving the conflict or ranking one side over another.
    pub fn current_view<'a>(
        &self,
        assertions: &'a [KnowledgeAssertion],
        policy: CurrentViewPolicy,
    ) -> Vec<&'a KnowledgeAssertion> {
        match policy {
            CurrentViewPolicy::IncludeAll => assertions.iter().collect(),
            CurrentViewPolicy::ExcludeActiveConflicts => assertions
                .iter()
                .filter(|assertion| !self.has_active_conflict(assertion.id()))
                .collect(),
        }
    }

    /// Returns the number of contradiction records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Detects a simple, structurally explicit contradiction between two assertions.
///
/// Detection only occurs when subject, predicate, context, and qualifiers match.
/// Two different positive objects contradict each other only when their
/// predicate is listed as functional by the caller's relationship vocabulary.
/// The helper does not infer that missing values are false and does not resolve
/// any detected conflict.
#[must_use]
pub fn detect_contradiction(
    left: &KnowledgeAssertion,
    right: &KnowledgeAssertion,
    functional_predicates: &BTreeSet<AssertionPredicate>,
) -> Option<ContradictionFinding> {
    if left.id() == right.id()
        || left.subject() != right.subject()
        || left.predicate() != right.predicate()
        || left.context() != right.context()
        || left.qualifiers() != right.qualifiers()
    {
        return None;
    }

    let kind = if left.object() == right.object() && left.polarity() != right.polarity() {
        ContradictionKind::OpposedPolarity
    } else if left.polarity() == AssertionPolarity::Positive
        && right.polarity() == AssertionPolarity::Positive
        && left.object() != right.object()
        && functional_predicates.contains(left.predicate())
    {
        ContradictionKind::FunctionalValueConflict
    } else {
        return None;
    };

    Some(ContradictionFinding {
        assertion_ids: BTreeSet::from([left.id().clone(), right.id().clone()]),
        involved_objects: BTreeSet::from([left.object().clone(), right.object().clone()]),
        kind,
        context: left.context().clone(),
        qualifiers: left.qualifiers().clone(),
    })
}

/// Structural errors while representing or collecting contradiction records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContradictionError {
    /// A contradiction must reference at least two different assertions.
    RequiresTwoDistinctAssertions,
    /// A contradiction ID is already present in the set.
    DuplicateIdentity,
    /// A required label is empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// A label contains a Unicode control character.
    ControlCharacter { field: &'static str, index: usize },
}

impl fmt::Display for ContradictionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiresTwoDistinctAssertions => {
                formatter.write_str("a contradiction requires at least two distinct assertions")
            }
            Self::DuplicateIdentity => {
                formatter.write_str("contradiction identity is already registered")
            }
            Self::EmptyLabel { field } => write!(formatter, "{field} must not be empty"),
            Self::ControlCharacter { field, index } => {
                write!(
                    formatter,
                    "{field} contains a control character at index {index}"
                )
            }
        }
    }
}

impl std::error::Error for ContradictionError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), ContradictionError> {
    if value.trim().is_empty() {
        return Err(ContradictionError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(ContradictionError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Contradiction, ContradictionError, ContradictionKind, ContradictionSet,
        ContradictionStatus, CurrentViewPolicy, detect_contradiction,
    };
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::identity::{ConceptId, ContradictionId, EntityId, EvidenceId};
    use std::collections::BTreeSet;

    fn assertion(object_id: &str, polarity: AssertionPolarity) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("subject-1").unwrap()),
            AssertionPredicate::new("has-name").unwrap(),
            AssertionObject::Concept(ConceptId::new(object_id).unwrap()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Known,
            polarity,
        )
    }

    fn contradiction_id(value: &str) -> ContradictionId {
        ContradictionId::new(value).expect("valid contradiction identity")
    }

    #[test]
    fn opposite_polarity_detects_same_proposition_conflict() {
        let positive = assertion("name-a", AssertionPolarity::Positive);
        let negative = assertion("name-a", AssertionPolarity::Negative);
        let finding = detect_contradiction(&positive, &negative, &BTreeSet::new())
            .expect("opposite polarity is an explicit contradiction");

        assert_eq!(finding.kind(), &ContradictionKind::OpposedPolarity);
        assert_eq!(finding.assertion_ids().len(), 2);
        assert_eq!(
            finding.involved_objects(),
            &BTreeSet::from([positive.object().clone()]),
            "opposed polarity applies to one shared typed object value"
        );
    }

    #[test]
    fn different_positive_values_conflict_only_for_functional_predicates() {
        let first = assertion("name-a", AssertionPolarity::Positive);
        let second = assertion("name-b", AssertionPolarity::Positive);
        assert!(detect_contradiction(&first, &second, &BTreeSet::new()).is_none());

        let functional = BTreeSet::from([AssertionPredicate::new("has-name").unwrap()]);
        let finding = detect_contradiction(&first, &second, &functional)
            .expect("functional values conflict structurally");
        assert_eq!(finding.kind(), &ContradictionKind::FunctionalValueConflict);
        assert_eq!(finding.involved_objects().len(), 2);
        assert!(finding.involved_objects().contains(first.object()));
        assert!(finding.involved_objects().contains(second.object()));
    }

    #[test]
    fn detection_requires_matching_context_and_qualifier_scope() {
        let first = assertion("name-a", AssertionPolarity::Positive);
        let second = KnowledgeAssertion::new(
            first.subject().clone(),
            first.predicate().clone(),
            AssertionObject::Concept(ConceptId::new("name-b").unwrap()),
            AssertionContext::from_entries([("region", "historical")]).unwrap(),
            Qualifiers::new(),
            AssertionStatus::Known,
            AssertionPolarity::Positive,
        );
        let functional = BTreeSet::from([first.predicate().clone()]);

        assert!(detect_contradiction(&first, &second, &functional).is_none());
    }

    #[test]
    fn contradiction_preserves_status_scope_and_supporting_evidence() {
        let first = assertion("name-a", AssertionPolarity::Positive);
        let second = assertion("name-a", AssertionPolarity::Negative);
        let finding = detect_contradiction(&first, &second, &BTreeSet::new()).unwrap();
        let evidence_id = EvidenceId::new("evidence-support").unwrap();
        let contradiction = finding
            .into_contradiction(contradiction_id("conflict-1"))
            .unwrap()
            .with_supporting_evidence([evidence_id.clone()])
            .with_status(ContradictionStatus::UnderReview)
            .with_notes("Both assertion records are retained.")
            .unwrap();

        assert_eq!(contradiction.assertion_ids().len(), 2);
        assert_eq!(
            contradiction.involved_objects(),
            &BTreeSet::from([first.object().clone()]),
            "detected typed object values survive conversion into the contradiction record"
        );
        assert_eq!(contradiction.status(), ContradictionStatus::UnderReview);
        assert!(contradiction.supporting_evidence().contains(&evidence_id));
        assert_eq!(
            contradiction.notes(),
            Some("Both assertion records are retained.")
        );
        assert_eq!(contradiction.context(), first.context());
    }

    #[test]
    fn contradiction_requires_two_distinct_assertions_and_custom_kinds_are_validated() {
        let one = crate::identity::KnowledgeAssertionId::new("assertion-1").unwrap();
        let result = Contradiction::new(
            contradiction_id("conflict-invalid"),
            [one.clone(), one],
            ContradictionKind::OpposedPolarity,
            AssertionContext::new(),
            Qualifiers::new(),
        );
        assert_eq!(
            result,
            Err(ContradictionError::RequiresTwoDistinctAssertions)
        );
        assert!(ContradictionKind::custom(" ").is_err());
    }

    #[test]
    fn current_view_can_preserve_or_hide_conflicting_assertions_without_selecting_a_winner() {
        let first = assertion("name-a", AssertionPolarity::Positive);
        let second = assertion("name-a", AssertionPolarity::Negative);
        let unrelated = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("other-subject").unwrap()),
            AssertionPredicate::new("has-name").unwrap(),
            AssertionObject::Concept(ConceptId::new("other-name").unwrap()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Known,
            AssertionPolarity::Positive,
        );
        let finding = detect_contradiction(&first, &second, &BTreeSet::new()).unwrap();
        let conflict = finding
            .into_contradiction(contradiction_id("conflict-view"))
            .unwrap();
        let mut conflicts = ContradictionSet::new();
        conflicts.insert(conflict).unwrap();
        let assertions = [first.clone(), second.clone(), unrelated.clone()];

        assert_eq!(
            conflicts
                .current_view(&assertions, CurrentViewPolicy::IncludeAll)
                .len(),
            3
        );
        let filtered =
            conflicts.current_view(&assertions, CurrentViewPolicy::ExcludeActiveConflicts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id(), unrelated.id());
        assert_eq!(conflicts.for_assertion(first.id()).len(), 1);
        assert!(conflicts.has_active_conflict(first.id()));
    }

    #[test]
    fn dismissing_a_finding_does_not_erase_the_record_or_assertions() {
        let first = assertion("name-a", AssertionPolarity::Positive);
        let second = assertion("name-a", AssertionPolarity::Negative);
        let conflict = detect_contradiction(&first, &second, &BTreeSet::new())
            .unwrap()
            .into_contradiction(contradiction_id("conflict-dismissed"))
            .unwrap()
            .with_status(ContradictionStatus::Dismissed);
        let mut conflicts = ContradictionSet::new();
        conflicts.insert(conflict).unwrap();
        let assertions = [first.clone(), second.clone()];

        assert_eq!(conflicts.len(), 1);
        assert!(!conflicts.has_active_conflict(first.id()));
        assert_eq!(
            conflicts
                .current_view(&assertions, CurrentViewPolicy::ExcludeActiveConflicts)
                .len(),
            2
        );
        assert_eq!(conflicts.for_assertion(first.id()).len(), 1);
    }
}
