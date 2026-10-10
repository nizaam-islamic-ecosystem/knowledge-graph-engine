//! Engine-local logical query plan and extensible query AST boundary.
//!
//! The plan records semantic operators and execution requirements without
//! selecting a physical storage provider. Planning and execution remain
//! separate so Phase 7 can replace physical access without changing queries.

use crate::assertion::AssertionObject;
use crate::graph::{TraversalBounds, TraversalDirection};
use crate::relationship::RelationshipPredicate;

use super::filter::Filter;
use super::ranking::RankingProfile;
use super::request::{
    LookupTarget, Pagination, QueryKind, QueryOptions, QueryOrdering, ReasoningProfile,
    RetrievalMode, RetrievalTarget, SemanticExpansion, TraversalBudget, VisibilityProfile,
};

/// Logical index dependency strength selected by the KG planner.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexAccessRequirement {
    /// Query semantics cannot be satisfied without the requested indexed path.
    Required,
    /// Indexing is the normal accelerator, but a valid KG fallback may satisfy
    /// the same semantic query.
    Preferred,
    /// Index access is merely an optional optimization.
    Optional,
}

/// General logical query operator.
///
/// The AST is intentionally extensible. Phase 6 only emits the initial
/// operators below; future pattern composition can add variants without
/// changing the public `QueryRequest` boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryOperator {
    /// Direct identity lookup.
    Lookup(LookupPlan),
    /// Bounded relationship traversal.
    Traverse(TraversalPlan),
    /// Exact/lexical/conceptual/relational/semantic retrieval.
    Retrieve(RetrievalPlan),
    /// Future composite operator boundary.
    Composite(Vec<QueryOperator>),
    /// Future filter operator boundary.
    Filter {
        /// Child logical operator.
        input: Box<QueryOperator>,
        /// Typed filter tree.
        filter: Filter,
    },
    /// Future ranking operator boundary.
    Rank {
        /// Child logical operator.
        input: Box<QueryOperator>,
        /// KG-owned ranking profile.
        profile: RankingProfile,
    },
    /// Future pagination operator boundary.
    Paginate {
        /// Child logical operator.
        input: Box<QueryOperator>,
        /// Logical pagination semantics.
        pagination: Pagination,
    },
}

/// Planned direct lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LookupPlan {
    target: LookupTarget,
}

impl LookupPlan {
    /// Creates a direct lookup plan.
    #[must_use]
    pub fn new(target: LookupTarget) -> Self {
        Self { target }
    }

    /// Returns the lookup target.
    #[must_use]
    pub fn target(&self) -> &LookupTarget {
        &self.target
    }
}

/// Planned bounded traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraversalPlan {
    start: AssertionObject,
    predicate: Option<RelationshipPredicate>,
    direction: TraversalDirection,
    budget: TraversalBudget,
    graph_bounds: TraversalBounds,
}

impl TraversalPlan {
    /// Creates a traversal plan from the explicit Phase 6 budget.
    #[must_use]
    pub const fn new(
        start: AssertionObject,
        predicate: Option<RelationshipPredicate>,
        direction: TraversalDirection,
        budget: TraversalBudget,
    ) -> Self {
        let graph_bounds = TraversalBounds::new(budget.max_depth());
        Self {
            start,
            predicate,
            direction,
            budget,
            graph_bounds,
        }
    }

    /// Returns the semantic traversal start reference.
    #[must_use]
    pub fn start(&self) -> &AssertionObject {
        &self.start
    }

    /// Returns the optional relationship predicate.
    #[must_use]
    pub fn predicate(&self) -> Option<&RelationshipPredicate> {
        self.predicate.as_ref()
    }

    /// Returns traversal direction.
    #[must_use]
    pub const fn direction(&self) -> TraversalDirection {
        self.direction
    }

    /// Returns the complete query traversal budget.
    #[must_use]
    pub const fn budget(&self) -> TraversalBudget {
        self.budget
    }

    /// Returns the graph-layer depth bound.
    #[must_use]
    pub const fn graph_bounds(&self) -> TraversalBounds {
        self.graph_bounds
    }
}

/// Planned semantic/search retrieval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalPlan {
    mode: RetrievalMode,
    target: RetrievalTarget,
    expansion: SemanticExpansion,
}

impl RetrievalPlan {
    /// Creates a retrieval plan.
    #[must_use]
    pub const fn new(
        mode: RetrievalMode,
        target: RetrievalTarget,
        expansion: SemanticExpansion,
    ) -> Self {
        Self {
            mode,
            target,
            expansion,
        }
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
}

/// Validated engine-local logical query plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    kind: QueryKind,
    operator: QueryOperator,
    filter: Filter,
    pagination: Pagination,
    ranking: RankingProfile,
    visibility: VisibilityProfile,
    reasoning: ReasoningProfile,
    index_requirement: IndexAccessRequirement,
}

impl QueryPlan {
    /// Creates a validated logical plan boundary.
    #[must_use]
    pub fn new(
        kind: QueryKind,
        operator: QueryOperator,
        options: &QueryOptions,
        index_requirement: IndexAccessRequirement,
    ) -> Self {
        Self {
            kind,
            operator,
            filter: options.filter().clone(),
            pagination: options.pagination().clone(),
            ranking: options.ranking(),
            visibility: options.visibility().clone(),
            reasoning: options.reasoning(),
            index_requirement,
        }
    }

    /// Returns the semantic query kind.
    #[must_use]
    pub const fn kind(&self) -> QueryKind {
        self.kind
    }

    /// Returns the root logical operator.
    #[must_use]
    pub fn operator(&self) -> &QueryOperator {
        &self.operator
    }

    /// Returns the typed filter tree.
    #[must_use]
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// Returns logical pagination.
    #[must_use]
    pub fn pagination(&self) -> &Pagination {
        &self.pagination
    }

    /// Returns the ranking profile.
    #[must_use]
    pub fn ranking(&self) -> RankingProfile {
        self.ranking.clone()
    }

    /// Returns the visibility profile.
    #[must_use]
    pub fn visibility(&self) -> &VisibilityProfile {
        &self.visibility
    }

    /// Returns the requested reasoning profile.
    #[must_use]
    pub const fn reasoning(&self) -> ReasoningProfile {
        self.reasoning
    }

    /// Returns the planner-selected index dependency strength.
    #[must_use]
    pub const fn index_requirement(&self) -> IndexAccessRequirement {
        self.index_requirement
    }

    /// Returns the configured deterministic ordering.
    #[must_use]
    pub const fn ordering(&self) -> QueryOrdering {
        self.pagination.ordering()
    }

    /// Validates the logical plan's structural invariants.
    pub fn validate(&self) -> Result<(), PlanValidationError> {
        self.filter
            .validate()
            .map_err(PlanValidationError::InvalidFilter)?;
        if self.pagination.limit() == 0 {
            return Err(PlanValidationError::ZeroLimit);
        }
        validate_operator(&self.operator)
    }
}

/// Plan-validation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanValidationError {
    /// The plan contains an invalid filter tree.
    InvalidFilter(super::filter::FilterError),
    /// The plan contains an impossible zero result limit.
    ZeroLimit,
    /// A composite operator has no children.
    EmptyComposite,
}

impl core::fmt::Display for PlanValidationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFilter(error) => write!(formatter, "invalid plan filter: {error}"),
            Self::ZeroLimit => formatter.write_str("query plan limit must be greater than zero"),
            Self::EmptyComposite => formatter.write_str("query plan composite must not be empty"),
        }
    }
}

impl std::error::Error for PlanValidationError {}

fn validate_operator(operator: &QueryOperator) -> Result<(), PlanValidationError> {
    match operator {
        QueryOperator::Lookup(_) | QueryOperator::Traverse(_) | QueryOperator::Retrieve(_) => {
            Ok(())
        }
        QueryOperator::Composite(children) => {
            if children.is_empty() {
                return Err(PlanValidationError::EmptyComposite);
            }
            for child in children {
                validate_operator(child)?;
            }
            Ok(())
        }
        QueryOperator::Filter { input, filter } => {
            filter
                .validate()
                .map_err(PlanValidationError::InvalidFilter)?;
            validate_operator(input)
        }
        QueryOperator::Rank { input, .. } | QueryOperator::Paginate { input, .. } => {
            validate_operator(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexAccessRequirement, LookupPlan, QueryOperator, QueryPlan};
    use crate::assertion::AssertionObject;
    use crate::identity::EntityId;
    use crate::query::{LookupRequest, QueryRequest, plan};

    #[test]
    fn logical_plan_keeps_physical_indexing_out_of_the_operator_tree() {
        let object = AssertionObject::Entity(EntityId::new("entity-plan").unwrap());
        let request = QueryRequest::Lookup(LookupRequest::object(object));
        let planned = plan(&request).expect("lookup should plan");

        assert!(matches!(planned.operator(), QueryOperator::Lookup(_)));
        assert_eq!(
            planned.index_requirement(),
            IndexAccessRequirement::Optional
        );
    }

    #[test]
    fn lookup_plan_preserves_typed_target() {
        let object = AssertionObject::Entity(EntityId::new("entity-plan-target").unwrap());
        let lookup = LookupPlan::new(crate::query::LookupTarget::Object(object.clone()));
        assert_eq!(lookup.target(), &crate::query::LookupTarget::Object(object));
    }

    #[test]
    fn query_plan_validate_rejects_zero_limit() {
        let object = AssertionObject::Entity(EntityId::new("entity-plan-limit").unwrap());
        let options =
            crate::query::QueryOptions::new().with_pagination(crate::query::Pagination::new(0));
        let plan = QueryPlan::new(
            crate::query::QueryKind::Lookup,
            QueryOperator::Lookup(LookupPlan::new(crate::query::LookupTarget::Object(object))),
            &options,
            IndexAccessRequirement::Optional,
        );
        assert!(matches!(
            plan.validate(),
            Err(super::PlanValidationError::ZeroLimit)
        ));
    }
}
