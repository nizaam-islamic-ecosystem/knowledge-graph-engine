//! Typed public query requests for Phase 6.
//!
//! This module defines the semantic query boundary. Requests describe what the
//! caller wants from the Knowledge Graph without exposing storage, physical
//! indexes, provider identities, or transport details.

use std::collections::BTreeSet;
use std::fmt;

use crate::assertion::{AssertionObject, AssertionStatus};
use crate::identity::ConceptId;
use crate::ingestion::PublicationOutcome;
use crate::relationship::RelationshipPredicate;

use super::filter::{Filter, FilterError};
use super::ranking::RankingProfile;

/// The top-level typed Phase 6 query request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryRequest {
    /// Retrieve one known semantic object or assertion by identity.
    Lookup(LookupRequest),
    /// Follow known graph relationships from one semantic object.
    Traversal(TraversalRequest),
    /// Retrieve knowledge through exact, lexical, conceptual, relational, or
    /// semantic access.
    Retrieval(RetrievalRequest),
}

impl QueryRequest {
    /// Validates the complete request before planning.
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        match self {
            Self::Lookup(request) => request.validate(),
            Self::Traversal(request) => request.validate(),
            Self::Retrieval(request) => request.validate(),
        }
    }

    /// Returns the semantic category of the request.
    #[must_use]
    pub const fn kind(&self) -> QueryKind {
        match self {
            Self::Lookup(_) => QueryKind::Lookup,
            Self::Traversal(_) => QueryKind::Traversal,
            Self::Retrieval(_) => QueryKind::Retrieval,
        }
    }
}

/// The three initial semantic query categories.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryKind {
    /// Direct identity lookup.
    Lookup,
    /// Bounded graph traversal.
    Traversal,
    /// Search/retrieval over known semantic knowledge.
    Retrieval,
}

impl fmt::Display for QueryKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lookup => "lookup",
            Self::Traversal => "traversal",
            Self::Retrieval => "retrieval",
        })
    }
}

/// Direct lookup target.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LookupTarget {
    /// Lookup one semantic object by its typed reference.
    Object(AssertionObject),
    /// Lookup one canonical assertion by its semantic identity.
    Assertion(crate::identity::KnowledgeAssertionId),
}

/// A direct lookup request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LookupRequest {
    target: LookupTarget,
    options: QueryOptions,
}

impl LookupRequest {
    /// Creates a lookup for a semantic object reference.
    #[must_use]
    pub fn object(target: impl Into<AssertionObject>) -> Self {
        Self {
            target: LookupTarget::Object(target.into()),
            options: QueryOptions::default(),
        }
    }

    /// Creates a lookup for a canonical knowledge assertion.
    #[must_use]
    pub fn assertion(id: crate::identity::KnowledgeAssertionId) -> Self {
        Self {
            target: LookupTarget::Assertion(id),
            options: QueryOptions::default(),
        }
    }

    /// Replaces the shared query options.
    #[must_use]
    pub fn with_options(mut self, options: QueryOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns the lookup target.
    #[must_use]
    pub fn target(&self) -> &LookupTarget {
        &self.target
    }

    /// Returns the shared query options.
    #[must_use]
    pub fn options(&self) -> &QueryOptions {
        &self.options
    }

    fn validate(&self) -> Result<(), QueryValidationError> {
        self.options.validate()
    }
}

/// The initial semantic traversal request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraversalRequest {
    start: AssertionObject,
    predicate: Option<RelationshipPredicate>,
    direction: crate::graph::TraversalDirection,
    budget: TraversalBudget,
    options: QueryOptions,
}

impl TraversalRequest {
    /// Creates a bounded traversal from a semantic object reference.
    #[must_use]
    pub fn from(start: impl Into<AssertionObject>, max_depth: usize) -> Self {
        Self {
            start: start.into(),
            predicate: None,
            direction: crate::graph::TraversalDirection::Forward,
            budget: TraversalBudget::new(max_depth),
            options: QueryOptions::default(),
        }
    }

    /// Restricts traversal to one relationship predicate.
    #[must_use]
    pub fn with_predicate(mut self, predicate: RelationshipPredicate) -> Self {
        self.predicate = Some(predicate);
        self
    }

    /// Sets the semantic traversal direction.
    #[must_use]
    pub fn with_direction(mut self, direction: crate::graph::TraversalDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Replaces the explicit traversal resource budget.
    #[must_use]
    pub fn with_budget(mut self, budget: TraversalBudget) -> Self {
        self.budget = budget;
        self
    }

    /// Replaces the shared query options.
    #[must_use]
    pub fn with_options(mut self, options: QueryOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns the semantic traversal start reference.
    #[must_use]
    pub fn start(&self) -> &AssertionObject {
        &self.start
    }

    /// Returns the optional relationship predicate constraint.
    #[must_use]
    pub fn predicate(&self) -> Option<&RelationshipPredicate> {
        self.predicate.as_ref()
    }

    /// Returns the requested traversal direction.
    #[must_use]
    pub const fn direction(&self) -> crate::graph::TraversalDirection {
        self.direction
    }

    /// Returns the traversal budget.
    #[must_use]
    pub const fn budget(&self) -> TraversalBudget {
        self.budget
    }

    /// Returns the shared query options.
    #[must_use]
    pub fn options(&self) -> &QueryOptions {
        &self.options
    }

    fn validate(&self) -> Result<(), QueryValidationError> {
        self.budget.validate()?;
        self.options.validate()
    }
}

/// Explicit traversal resource controls.
///
/// There is deliberately no unbounded/default constructor. A query must opt
/// into a maximum depth before graph expansion can occur.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraversalBudget {
    max_depth: usize,
    max_nodes: usize,
    max_edges: usize,
    max_results: usize,
    max_expansion: usize,
}

impl TraversalBudget {
    /// Creates a budget with a required depth bound and conservative work caps.
    #[must_use]
    pub const fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            max_nodes: 10_000,
            max_edges: 20_000,
            max_results: 1_000,
            max_expansion: 10_000,
        }
    }

    /// Replaces the maximum visited-node budget.
    #[must_use]
    pub const fn with_max_nodes(mut self, value: usize) -> Self {
        self.max_nodes = value;
        self
    }

    /// Replaces the maximum traversed-edge budget.
    #[must_use]
    pub const fn with_max_edges(mut self, value: usize) -> Self {
        self.max_edges = value;
        self
    }

    /// Replaces the maximum returned traversal-result budget.
    #[must_use]
    pub const fn with_max_results(mut self, value: usize) -> Self {
        self.max_results = value;
        self
    }

    /// Replaces the maximum expansion budget.
    #[must_use]
    pub const fn with_max_expansion(mut self, value: usize) -> Self {
        self.max_expansion = value;
        self
    }

    /// Returns the maximum traversal depth.
    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// Returns the maximum number of visited nodes.
    #[must_use]
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum number of traversed edges.
    #[must_use]
    pub const fn max_edges(self) -> usize {
        self.max_edges
    }

    /// Returns the maximum number of traversal results.
    #[must_use]
    pub const fn max_results(self) -> usize {
        self.max_results
    }

    /// Returns the maximum expansion work budget.
    #[must_use]
    pub const fn max_expansion(self) -> usize {
        self.max_expansion
    }

    fn validate(self) -> Result<(), QueryValidationError> {
        if self.max_nodes == 0 {
            return Err(QueryValidationError::ZeroBudget("max_nodes"));
        }
        if self.max_edges == 0 {
            return Err(QueryValidationError::ZeroBudget("max_edges"));
        }
        if self.max_results == 0 {
            return Err(QueryValidationError::ZeroBudget("max_results"));
        }
        if self.max_expansion == 0 {
            return Err(QueryValidationError::ZeroBudget("max_expansion"));
        }
        Ok(())
    }
}

/// Retrieval mode for the initial semantic retrieval surface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetrievalMode {
    /// Exact reference or exact semantic match.
    Exact,
    /// Lexical/text-oriented retrieval.
    Lexical,
    /// Concept-targeted retrieval.
    Conceptual,
    /// Relationship/predicate-oriented retrieval.
    Relational,
    /// Controlled semantic retrieval.
    Semantic,
}

/// Typed target supplied to a retrieval request.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetrievalTarget {
    /// Exact semantic object reference.
    Object(AssertionObject),
    /// Canonical concept target.
    Concept(ConceptId),
    /// Relationship predicate target.
    Relationship(RelationshipPredicate),
    /// User-supplied lexical or semantic text.
    Text(String),
}

impl RetrievalTarget {
    fn validate(&self) -> Result<(), QueryValidationError> {
        if let Self::Text(value) = self
            && value.trim().is_empty()
        {
            return Err(QueryValidationError::EmptyTextTarget);
        }
        Ok(())
    }
}

/// A semantic retrieval request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalRequest {
    mode: RetrievalMode,
    target: RetrievalTarget,
    expansion: SemanticExpansion,
    options: QueryOptions,
}

impl RetrievalRequest {
    /// Creates an exact object/reference retrieval.
    #[must_use]
    pub fn exact(target: impl Into<AssertionObject>) -> Self {
        Self {
            mode: RetrievalMode::Exact,
            target: RetrievalTarget::Object(target.into()),
            expansion: SemanticExpansion::None,
            options: QueryOptions::default(),
        }
    }

    /// Creates lexical retrieval from text.
    #[must_use]
    pub fn lexical(text: impl Into<String>) -> Self {
        Self {
            mode: RetrievalMode::Lexical,
            target: RetrievalTarget::Text(text.into()),
            expansion: SemanticExpansion::None,
            options: QueryOptions::default(),
        }
    }

    /// Creates conceptual retrieval for one concept identity.
    #[must_use]
    pub fn conceptual(concept: ConceptId) -> Self {
        Self {
            mode: RetrievalMode::Conceptual,
            target: RetrievalTarget::Concept(concept),
            expansion: SemanticExpansion::None,
            options: QueryOptions::default(),
        }
    }

    /// Creates relational retrieval for one relationship predicate.
    #[must_use]
    pub fn relational(predicate: RelationshipPredicate) -> Self {
        Self {
            mode: RetrievalMode::Relational,
            target: RetrievalTarget::Relationship(predicate),
            expansion: SemanticExpansion::None,
            options: QueryOptions::default(),
        }
    }

    /// Creates controlled semantic retrieval from text or another typed target.
    #[must_use]
    pub fn semantic(target: RetrievalTarget) -> Self {
        Self {
            mode: RetrievalMode::Semantic,
            target,
            expansion: SemanticExpansion::Controlled,
            options: QueryOptions::default(),
        }
    }

    /// Sets the semantic expansion policy.
    #[must_use]
    pub const fn with_expansion(mut self, expansion: SemanticExpansion) -> Self {
        self.expansion = expansion;
        self
    }

    /// Replaces the shared query options.
    #[must_use]
    pub fn with_options(mut self, options: QueryOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns the retrieval mode.
    #[must_use]
    pub const fn mode(&self) -> RetrievalMode {
        self.mode
    }

    /// Returns the typed retrieval target.
    #[must_use]
    pub fn target(&self) -> &RetrievalTarget {
        &self.target
    }

    /// Returns the semantic-expansion policy.
    #[must_use]
    pub const fn expansion(&self) -> SemanticExpansion {
        self.expansion
    }

    /// Returns shared query options.
    #[must_use]
    pub fn options(&self) -> &QueryOptions {
        &self.options
    }

    fn validate(&self) -> Result<(), QueryValidationError> {
        self.target.validate()?;
        if self.mode == RetrievalMode::Conceptual
            && !matches!(self.target, RetrievalTarget::Concept(_))
        {
            return Err(QueryValidationError::TargetModeMismatch);
        }
        if self.mode == RetrievalMode::Relational
            && !matches!(self.target, RetrievalTarget::Relationship(_))
        {
            return Err(QueryValidationError::TargetModeMismatch);
        }
        self.options.validate()
    }
}

/// Controlled semantic-expansion policy.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticExpansion {
    /// Do not expand beyond the supplied target.
    None,
    /// Allow only KG-approved, bounded semantic expansion.
    Controlled,
}

/// Shared options applied to all query variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOptions {
    filter: Filter,
    pagination: Pagination,
    ranking: RankingProfile,
    visibility: VisibilityProfile,
    reasoning: ReasoningProfile,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            filter: Filter::all(),
            pagination: Pagination::default(),
            ranking: RankingProfile::General,
            visibility: VisibilityProfile::public(),
            reasoning: ReasoningProfile::None,
        }
    }
}

impl QueryOptions {
    /// Creates the default query options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the typed filter tree.
    #[must_use]
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    /// Replaces pagination and ordering.
    #[must_use]
    pub fn with_pagination(mut self, pagination: Pagination) -> Self {
        self.pagination = pagination;
        self
    }

    /// Selects a KG-owned ranking profile.
    #[must_use]
    pub fn with_ranking(mut self, ranking: RankingProfile) -> Self {
        self.ranking = ranking;
        self
    }

    /// Selects a visibility/governance profile.
    #[must_use]
    pub fn with_visibility(mut self, visibility: VisibilityProfile) -> Self {
        self.visibility = visibility;
        self
    }

    /// Selects a reasoning profile. Actual reasoning remains a Phase 8 concern.
    #[must_use]
    pub const fn with_reasoning(mut self, reasoning: ReasoningProfile) -> Self {
        self.reasoning = reasoning;
        self
    }

    /// Returns the typed filter tree.
    #[must_use]
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// Returns pagination and ordering.
    #[must_use]
    pub const fn pagination(&self) -> &Pagination {
        &self.pagination
    }

    /// Returns the ranking profile.
    #[must_use]
    pub fn ranking(&self) -> RankingProfile {
        self.ranking.clone()
    }

    /// Returns the visibility profile.
    #[must_use]
    pub const fn visibility(&self) -> &VisibilityProfile {
        &self.visibility
    }

    /// Returns the reasoning profile.
    #[must_use]
    pub const fn reasoning(&self) -> ReasoningProfile {
        self.reasoning
    }

    fn validate(&self) -> Result<(), QueryValidationError> {
        self.filter
            .validate()
            .map_err(QueryValidationError::InvalidFilter)?;
        self.pagination.validate()?;
        self.visibility.validate()
    }
}

/// Hybrid cursor-first pagination options.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pagination {
    limit: usize,
    cursor: Option<String>,
    offset: Option<usize>,
    ordering: QueryOrdering,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            cursor: None,
            offset: None,
            ordering: QueryOrdering::RelevanceDescending,
        }
    }
}

impl Pagination {
    /// Creates cursor-first pagination with the supplied limit.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            ..Self::default()
        }
    }

    /// Sets an opaque continuation cursor.
    #[must_use]
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Sets offset pagination for small/static result sets.
    #[must_use]
    pub const fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets deterministic result ordering.
    #[must_use]
    pub const fn with_ordering(mut self, ordering: QueryOrdering) -> Self {
        self.ordering = ordering;
        self
    }

    /// Returns the result limit.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Returns the opaque continuation cursor.
    #[must_use]
    pub fn cursor(&self) -> Option<&str> {
        self.cursor.as_deref()
    }

    /// Returns the optional offset.
    #[must_use]
    pub const fn offset(&self) -> Option<usize> {
        self.offset
    }

    /// Returns the deterministic ordering strategy.
    #[must_use]
    pub const fn ordering(&self) -> QueryOrdering {
        self.ordering
    }

    fn validate(&self) -> Result<(), QueryValidationError> {
        if self.limit == 0 {
            return Err(QueryValidationError::ZeroLimit);
        }
        if self.cursor.is_some() && self.offset.is_some() {
            return Err(QueryValidationError::CursorAndOffsetConflict);
        }
        if self.cursor.as_deref().is_some_and(str::is_empty) {
            return Err(QueryValidationError::EmptyCursor);
        }
        Ok(())
    }
}

/// Deterministic ordering of query results.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryOrdering {
    /// Highest relevance first, then stable reference order.
    RelevanceDescending,
    /// Shortest matched path first, then stable reference order.
    PathLengthAscending,
    /// Stable semantic reference order.
    ReferenceAscending,
}

/// Query/governance visibility policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibilityProfile {
    name: String,
    required_publication: Option<PublicationOutcome>,
    include_inferred: bool,
    include_machine_generated: bool,
    include_disputed: bool,
    allowed_statuses: BTreeSet<AssertionStatus>,
}

impl VisibilityProfile {
    /// Conservative public-facing profile.
    #[must_use]
    pub fn public() -> Self {
        Self {
            name: "public".to_owned(),
            required_publication: Some(PublicationOutcome::Published),
            include_inferred: false,
            include_machine_generated: false,
            include_disputed: false,
            allowed_statuses: BTreeSet::new(),
        }
    }

    /// Scholarly profile retaining disputed material while excluding machine
    /// candidates by default.
    #[must_use]
    pub fn scholarly() -> Self {
        Self {
            name: "scholarly".to_owned(),
            required_publication: Some(PublicationOutcome::Published),
            include_inferred: false,
            include_machine_generated: false,
            include_disputed: true,
            allowed_statuses: BTreeSet::new(),
        }
    }

    /// Historical profile that permits disputed material but still requires
    /// governed publication.
    #[must_use]
    pub fn historical() -> Self {
        Self {
            name: "historical".to_owned(),
            required_publication: Some(PublicationOutcome::Published),
            include_inferred: false,
            include_machine_generated: false,
            include_disputed: true,
            allowed_statuses: BTreeSet::new(),
        }
    }

    /// Research profile with the broadest explicit visibility of governed data.
    #[must_use]
    pub fn research() -> Self {
        Self {
            name: "research".to_owned(),
            required_publication: Some(PublicationOutcome::Published),
            include_inferred: true,
            include_machine_generated: true,
            include_disputed: true,
            allowed_statuses: BTreeSet::new(),
        }
    }

    /// Creates a named custom visibility profile.
    pub fn custom(name: impl Into<String>) -> Result<Self, QueryValidationError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(QueryValidationError::EmptyVisibilityProfile);
        }
        Ok(Self {
            name,
            required_publication: Some(PublicationOutcome::Published),
            include_inferred: false,
            include_machine_generated: false,
            include_disputed: false,
            allowed_statuses: BTreeSet::new(),
        })
    }

    /// Requires a publication outcome in addition to the other policy rules.
    #[must_use]
    pub const fn with_required_publication(mut self, outcome: PublicationOutcome) -> Self {
        self.required_publication = Some(outcome);
        self
    }

    /// Enables or disables inferred knowledge.
    #[must_use]
    pub const fn with_inferred(mut self, enabled: bool) -> Self {
        self.include_inferred = enabled;
        self
    }

    /// Enables or disables machine-generated candidates.
    #[must_use]
    pub const fn with_machine_generated(mut self, enabled: bool) -> Self {
        self.include_machine_generated = enabled;
        self
    }

    /// Enables or disables disputed knowledge.
    #[must_use]
    pub const fn with_disputed(mut self, enabled: bool) -> Self {
        self.include_disputed = enabled;
        self
    }

    /// Adds an allowed assertion status.
    #[must_use]
    pub fn allowing_status(mut self, status: AssertionStatus) -> Self {
        self.allowed_statuses.insert(status);
        self
    }

    /// Returns the profile name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the required publication outcome, if any.
    #[must_use]
    pub const fn required_publication(&self) -> Option<PublicationOutcome> {
        self.required_publication
    }

    /// Returns whether inferred knowledge is visible.
    #[must_use]
    pub const fn include_inferred(&self) -> bool {
        self.include_inferred
    }

    /// Returns whether machine-generated candidates are visible.
    #[must_use]
    pub const fn include_machine_generated(&self) -> bool {
        self.include_machine_generated
    }

    /// Returns whether disputed knowledge is visible.
    #[must_use]
    pub const fn include_disputed(&self) -> bool {
        self.include_disputed
    }

    /// Returns the explicit status allow-list.
    #[must_use]
    pub fn allowed_statuses(&self) -> &BTreeSet<AssertionStatus> {
        &self.allowed_statuses
    }

    fn validate(&self) -> Result<(), QueryValidationError> {
        if self.name.trim().is_empty() {
            return Err(QueryValidationError::EmptyVisibilityProfile);
        }
        Ok(())
    }
}

/// Reasoning orchestration profile. The actual inference engine belongs to Phase 8.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReasoningProfile {
    /// Do not request reasoning.
    None,
    /// Request the basic controlled reasoning profile.
    Basic,
    /// Request semantic reasoning orchestration.
    Semantic,
    /// Request the deepest approved reasoning profile.
    Deep,
}

/// Structural request-validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryValidationError {
    /// A traversal budget field was set to zero.
    ZeroBudget(&'static str),
    /// A result limit was set to zero.
    ZeroLimit,
    /// Cursor and offset cannot be used together.
    CursorAndOffsetConflict,
    /// A supplied cursor was empty.
    EmptyCursor,
    /// A visibility profile has no valid name.
    EmptyVisibilityProfile,
    /// A text retrieval target is empty.
    EmptyTextTarget,
    /// A retrieval mode was paired with the wrong typed target.
    TargetModeMismatch,
    /// The filter tree failed structural validation.
    InvalidFilter(FilterError),
}

impl fmt::Display for QueryValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroBudget(field) => {
                write!(formatter, "query budget {field} must be greater than zero")
            }
            Self::ZeroLimit => {
                formatter.write_str("query pagination limit must be greater than zero")
            }
            Self::CursorAndOffsetConflict => {
                formatter.write_str("query pagination cursor and offset cannot be combined")
            }
            Self::EmptyCursor => formatter.write_str("query pagination cursor must not be empty"),
            Self::EmptyVisibilityProfile => {
                formatter.write_str("query visibility profile name must not be empty")
            }
            Self::EmptyTextTarget => formatter.write_str("query text target must not be empty"),
            Self::TargetModeMismatch => {
                formatter.write_str("retrieval mode and typed retrieval target do not match")
            }
            Self::InvalidFilter(error) => write!(formatter, "invalid query filter: {error}"),
        }
    }
}

impl std::error::Error for QueryValidationError {}

#[cfg(test)]
mod tests {
    use super::{
        LookupRequest, Pagination, QueryOptions, QueryOrdering, QueryRequest, QueryValidationError,
        ReasoningProfile, RetrievalRequest, RetrievalTarget, TraversalBudget, TraversalRequest,
        VisibilityProfile,
    };
    use crate::assertion::AssertionObject;
    use crate::identity::{ConceptId, EntityId};
    use crate::relationship::RelationshipPredicate;

    fn entity(value: &str) -> AssertionObject {
        AssertionObject::Entity(EntityId::new(value).expect("valid entity"))
    }

    #[test]
    fn typed_lookup_traversal_and_retrieval_requests_validate() {
        assert!(
            QueryRequest::Lookup(LookupRequest::object(entity("entity-1")))
                .validate()
                .is_ok()
        );
        assert!(
            QueryRequest::Traversal(TraversalRequest::from(entity("entity-1"), 2))
                .validate()
                .is_ok()
        );
        assert!(
            QueryRequest::Retrieval(RetrievalRequest::lexical("sabr"))
                .validate()
                .is_ok()
        );
        assert!(
            QueryRequest::Retrieval(RetrievalRequest::conceptual(
                ConceptId::new("concept-sabr").unwrap(),
            ))
            .validate()
            .is_ok()
        );
        assert!(
            QueryRequest::Retrieval(RetrievalRequest::relational(
                RelationshipPredicate::new("related-to").unwrap(),
            ))
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn pagination_rejects_mixed_cursor_and_offset() {
        let options = QueryOptions::new()
            .with_pagination(Pagination::new(10).with_cursor("cursor").with_offset(2));
        let error =
            QueryRequest::Lookup(LookupRequest::object(entity("entity-1")).with_options(options))
                .validate()
                .expect_err("cursor and offset must not be combined");
        assert_eq!(error, QueryValidationError::CursorAndOffsetConflict);
    }

    #[test]
    fn traversal_budget_requires_positive_work_caps() {
        let budget = TraversalBudget::new(2).with_max_nodes(0);
        let request = QueryRequest::Traversal(
            TraversalRequest::from(entity("entity-1"), 2).with_budget(budget),
        );
        assert!(matches!(
            request.validate(),
            Err(QueryValidationError::ZeroBudget("max_nodes"))
        ));
    }

    #[test]
    fn query_options_preserve_reasoning_visibility_and_ordering() {
        let options = QueryOptions::new()
            .with_reasoning(ReasoningProfile::Basic)
            .with_visibility(VisibilityProfile::research())
            .with_pagination(Pagination::new(5).with_ordering(QueryOrdering::ReferenceAscending));

        assert_eq!(options.reasoning(), ReasoningProfile::Basic);
        assert!(options.visibility().include_inferred());
        assert_eq!(
            options.pagination().ordering(),
            QueryOrdering::ReferenceAscending
        );
        assert!(matches!(
            RetrievalRequest::semantic(RetrievalTarget::Text("sabr".to_owned())),
            RetrievalRequest { .. }
        ));
    }
}
