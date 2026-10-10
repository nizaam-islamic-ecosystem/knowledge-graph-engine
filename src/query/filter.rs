//! Typed composable query filters.
//!
//! Filters operate on factual metadata already associated with a query
//! candidate. They do not fetch data, execute reasoning, or inspect physical
//! storage.

use core::fmt;

use crate::assertion::{AssertionObject, AssertionStatus};
use crate::authority::AuthorityDimension;
use crate::identity::{EvidenceId, SourceId};
use crate::ingestion::PublicationOutcome;
use crate::relationship::RelationshipPredicate;
use crate::semantics::SemanticType;
use crate::temporal::{Instant, Interval, IntervalBoundary, TemporalValidity, TemporalValue};

use super::result::QueryMatchCandidate;

/// A storage-independent typed filter tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Filter {
    /// Matches every candidate.
    All,
    /// Matches no candidate.
    None,
    /// All child filters must match.
    And(Vec<Filter>),
    /// At least one child filter must match.
    Or(Vec<Filter>),
    /// Negates one child filter.
    Not(Box<Filter>),
    /// Matches candidates carrying a source reference.
    Source(SourceId),
    /// Matches one exact authority dimension.
    Authority(AuthorityDimension),
    /// Matches candidates carrying one evidence reference.
    Evidence(EvidenceId),
    /// Applies valid-time semantics.
    Temporal(TemporalFilter),
    /// Matches the canonical assertion epistemic status.
    Epistemic(AssertionStatus),
    /// Matches governed publication state.
    Publication(PublicationOutcome),
    /// Matches one exact context key/value pair.
    Context { key: String, value: String },
    /// Matches one exact relationship predicate.
    Relationship(RelationshipPredicate),
    /// Matches one semantic type declaration.
    SemanticType(SemanticType),
    /// Matches an exact subject/object semantic reference.
    Entity(EntityConstraint),
}

impl Filter {
    /// Returns the identity filter that accepts every candidate.
    #[must_use]
    pub const fn all() -> Self {
        Self::All
    }

    /// Returns a filter that accepts no candidate.
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    /// Constructs a conjunction and rejects an empty boolean group.
    pub fn and(filters: impl IntoIterator<Item = Filter>) -> Result<Self, FilterError> {
        let filters = filters.into_iter().collect::<Vec<_>>();
        if filters.is_empty() {
            return Err(FilterError::EmptyComposite);
        }
        Ok(Self::And(filters))
    }

    /// Constructs a disjunction and rejects an empty boolean group.
    pub fn or(filters: impl IntoIterator<Item = Filter>) -> Result<Self, FilterError> {
        let filters = filters.into_iter().collect::<Vec<_>>();
        if filters.is_empty() {
            return Err(FilterError::EmptyComposite);
        }
        Ok(Self::Or(filters))
    }

    /// Constructs a negation.
    #[must_use]
    pub fn negate(filter: Filter) -> Self {
        Self::Not(Box::new(filter))
    }

    /// Validates the filter tree without executing it.
    pub fn validate(&self) -> Result<(), FilterError> {
        match self {
            Self::All | Self::None => Ok(()),
            Self::And(filters) | Self::Or(filters) => {
                if filters.is_empty() {
                    return Err(FilterError::EmptyComposite);
                }
                for filter in filters {
                    filter.validate()?;
                }
                Ok(())
            }
            Self::Not(filter) => filter.validate(),
            Self::Source(_) | Self::Authority(_) | Self::Evidence(_) => Ok(()),
            Self::Temporal(filter) => filter.validate(),
            Self::Epistemic(_) | Self::Publication(_) | Self::Relationship(_) => Ok(()),
            Self::Context { key, value } => {
                if key.trim().is_empty() {
                    return Err(FilterError::EmptyContextKey);
                }
                if value.contains('\0') {
                    return Err(FilterError::InvalidContextValue);
                }
                Ok(())
            }
            Self::SemanticType(semantic_type) => semantic_type
                .validate()
                .map_err(|_| FilterError::InvalidSemanticType),
            Self::Entity(constraint) => constraint.validate(),
        }
    }

    /// Evaluates the filter against one already-retrieved candidate.
    #[must_use]
    pub fn matches(&self, candidate: &QueryMatchCandidate) -> bool {
        match self {
            Self::All => true,
            Self::None => false,
            Self::And(filters) => filters.iter().all(|filter| filter.matches(candidate)),
            Self::Or(filters) => filters.iter().any(|filter| filter.matches(candidate)),
            Self::Not(filter) => !filter.matches(candidate),
            Self::Source(source_id) => candidate.source_ids().contains(source_id),
            Self::Authority(dimension) => candidate.authority_dimensions().contains(dimension),
            Self::Evidence(evidence_id) => candidate.evidence_ids().contains(evidence_id),
            Self::Temporal(filter) => filter.matches(candidate.validity()),
            Self::Epistemic(status) => candidate.status() == Some(*status),
            Self::Publication(outcome) => candidate.publication() == Some(*outcome),
            Self::Context { key, value } => candidate
                .context()
                .and_then(|context| context.get(key))
                .is_some_and(|actual| actual == value),
            Self::Relationship(predicate) => candidate.relationship_predicate() == Some(predicate),
            Self::SemanticType(semantic_type) => candidate.semantic_types().contains(semantic_type),
            Self::Entity(constraint) => constraint.matches(candidate),
        }
    }
}

/// Temporal query predicates supported by Phase 6.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TemporalFilter {
    /// Candidate validity must definitely contain this instant.
    ValidAt(Instant),
    /// Candidate validity must overlap this requested interval.
    ValidDuring(Interval),
    /// Candidate validity must be explicitly unknown.
    Unknown,
}

impl TemporalFilter {
    fn validate(self) -> Result<(), FilterError> {
        match self {
            Self::ValidAt(_) | Self::ValidDuring(_) | Self::Unknown => Ok(()),
        }
    }

    fn matches(self, validity: Option<TemporalValidity>) -> bool {
        let Some(validity) = validity else {
            return matches!(self, Self::Unknown);
        };

        match self {
            Self::Unknown => validity.is_unknown(),
            Self::ValidAt(instant) => match validity.value() {
                TemporalValue::Instant(value) => value == instant,
                TemporalValue::Interval(interval) => interval.contains(instant) == Some(true),
                TemporalValue::Approximate(_) => false,
                TemporalValue::OpenEnded(value) => value.interval().contains(instant) == Some(true),
                TemporalValue::Unknown => false,
            },
            Self::ValidDuring(requested) => validity_overlaps(validity.value(), requested),
        }
    }
}

/// Position used by an entity/reference constraint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EntityPosition {
    /// Match the assertion subject/reference.
    Subject,
    /// Match the assertion object/reference.
    Object,
    /// Match either assertion side.
    Either,
}

/// Typed subject/object reference constraint.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityConstraint {
    position: EntityPosition,
    reference: AssertionObject,
}

impl EntityConstraint {
    /// Creates an exact semantic-reference constraint.
    #[must_use]
    pub fn new(position: EntityPosition, reference: AssertionObject) -> Self {
        Self {
            position,
            reference,
        }
    }

    /// Returns the constrained position.
    #[must_use]
    pub const fn position(&self) -> EntityPosition {
        self.position
    }

    /// Returns the constrained semantic reference.
    #[must_use]
    pub fn reference(&self) -> &AssertionObject {
        &self.reference
    }

    fn validate(&self) -> Result<(), FilterError> {
        Ok(())
    }

    fn matches(&self, candidate: &QueryMatchCandidate) -> bool {
        match self.position {
            EntityPosition::Subject => candidate.subject() == Some(&self.reference),
            EntityPosition::Object => candidate.object() == Some(&self.reference),
            EntityPosition::Either => {
                candidate.subject() == Some(&self.reference)
                    || candidate.object() == Some(&self.reference)
            }
        }
    }
}

/// Structural filter-validation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilterError {
    /// An `AND` or `OR` group must contain at least one child.
    EmptyComposite,
    /// A context key cannot be empty.
    EmptyContextKey,
    /// A context value contains a forbidden NUL character.
    InvalidContextValue,
    /// A semantic type could not be structurally validated.
    InvalidSemanticType,
}

impl fmt::Display for FilterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyComposite => formatter.write_str("filter boolean group must not be empty"),
            Self::EmptyContextKey => formatter.write_str("filter context key must not be empty"),
            Self::InvalidContextValue => formatter.write_str("filter context value is invalid"),
            Self::InvalidSemanticType => formatter.write_str("filter semantic type is invalid"),
        }
    }
}

impl std::error::Error for FilterError {}

fn validity_overlaps(value: TemporalValue, requested: Interval) -> bool {
    let Some(value_interval) = as_interval(value) else {
        return false;
    };

    intervals_overlap(value_interval, requested)
}

fn as_interval(value: TemporalValue) -> Option<Interval> {
    match value {
        TemporalValue::Instant(instant) => Interval::new(
            IntervalBoundary::Inclusive(instant),
            IntervalBoundary::Inclusive(instant),
        )
        .ok(),
        TemporalValue::Interval(interval) => Some(interval),
        TemporalValue::Approximate(approximate) => Interval::new(
            IntervalBoundary::Inclusive(approximate.center()),
            IntervalBoundary::Inclusive(approximate.center()),
        )
        .ok(),
        TemporalValue::OpenEnded(open_ended) => Some(*open_ended.interval()),
        TemporalValue::Unknown => None,
    }
}

fn intervals_overlap(left: Interval, right: Interval) -> bool {
    if let (Some(left_end), Some(right_start)) = (left.end().instant(), right.start().instant())
        && (left_end < right_start || (left_end == right_start && excludes_right(left, right)))
    {
        return false;
    }

    if let (Some(right_end), Some(left_start)) = (right.end().instant(), left.start().instant())
        && (right_end < left_start || (right_end == left_start && excludes_right(right, left)))
    {
        return false;
    }

    if left.end().is_unknown()
        || left.start().is_unknown()
        || right.end().is_unknown()
        || right.start().is_unknown()
    {
        return false;
    }

    true
}

fn excludes_right(left: Interval, right: Interval) -> bool {
    matches!(
        (left.end(), right.start()),
        (IntervalBoundary::Exclusive(_), _) | (_, IntervalBoundary::Exclusive(_))
    )
}

#[cfg(test)]
mod tests {
    use super::{EntityConstraint, EntityPosition, Filter, TemporalFilter};
    use crate::assertion::{AssertionObject, AssertionStatus};
    use crate::identity::{EntityId, EvidenceId, SourceId};
    use crate::ingestion::PublicationOutcome;
    use crate::query::{InferenceStatus, QueryMatchCandidate, QueryMatchType, QueryReference};
    use crate::temporal::{Instant, TemporalValidity};

    fn entity(value: &str) -> AssertionObject {
        AssertionObject::Entity(EntityId::new(value).expect("valid entity"))
    }

    fn candidate() -> QueryMatchCandidate {
        let subject = entity("subject");
        let object = entity("object");
        QueryMatchCandidate::new(
            QueryReference::Assertion(
                crate::identity::KnowledgeAssertionId::new("assertion-filter")
                    .expect("valid assertion"),
            ),
            QueryMatchType::Semantic,
        )
        .with_assertion_ends(subject, object)
        .with_sources([SourceId::new("source-filter").expect("valid source")])
        .with_evidence([EvidenceId::new("evidence-filter").expect("valid evidence")])
        .with_status(AssertionStatus::Known)
        .with_publication(PublicationOutcome::Published)
        .with_validity(TemporalValidity::at(Instant::from_unix_seconds(100)))
        .with_inference_status(InferenceStatus::Observed)
    }

    #[test]
    fn boolean_filter_tree_composes_typed_leaf_filters() {
        let filter = Filter::and([
            Filter::Source(SourceId::new("source-filter").unwrap()),
            Filter::or([
                Filter::Epistemic(AssertionStatus::Known),
                Filter::Publication(PublicationOutcome::Published),
            ])
            .expect("non-empty disjunction"),
        ])
        .expect("non-empty conjunction");

        let candidate = candidate();
        assert!(filter.matches(&candidate));
        assert!(filter.validate().is_ok());
    }

    #[test]
    fn entity_and_temporal_filters_match_existing_candidate_facts() {
        let candidate = candidate();
        let entity_filter = Filter::Entity(EntityConstraint::new(
            EntityPosition::Subject,
            entity("subject"),
        ));
        let temporal = Filter::Temporal(TemporalFilter::ValidAt(Instant::from_unix_seconds(100)));

        assert!(entity_filter.matches(&candidate));
        assert!(temporal.matches(&candidate));
    }

    #[test]
    fn valid_at_does_not_treat_approximate_time_as_exact() {
        use crate::temporal::{Approximate, TemporalValue};

        let validity = TemporalValidity::new(TemporalValue::Approximate(Approximate::new(
            Instant::from_unix_seconds(100),
        )));
        let filter = Filter::Temporal(TemporalFilter::ValidAt(Instant::from_unix_seconds(100)));

        assert!(
            !filter.matches(
                &QueryMatchCandidate::new(
                    QueryReference::Assertion(
                        crate::identity::KnowledgeAssertionId::new("assertion-approximate")
                            .unwrap(),
                    ),
                    QueryMatchType::Semantic,
                )
                .with_validity(validity),
            )
        );
    }

    #[test]
    fn empty_boolean_groups_are_rejected() {
        assert!(Filter::and([]).is_err());
        assert!(Filter::or([]).is_err());
    }
}
