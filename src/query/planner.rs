//! KG-local query planner.
//!
//! The planner validates typed requests and converts them into an engine-local
//! logical plan. It does not route engines, select global providers, schedule
//! work, or expose physical storage details.

use core::fmt;

use super::plan::{
    IndexAccessRequirement, LookupPlan, QueryOperator, QueryPlan, RetrievalPlan, TraversalPlan,
};
use super::request::{QueryKind, QueryRequest, QueryValidationError, RetrievalMode};

/// Plans one typed KG query request.
pub fn plan(request: &QueryRequest) -> Result<QueryPlan, QueryPlannerError> {
    request
        .validate()
        .map_err(QueryPlannerError::InvalidRequest)?;

    let query_plan = match request {
        QueryRequest::Lookup(request) => QueryPlan::new(
            QueryKind::Lookup,
            QueryOperator::Lookup(LookupPlan::new(request.target().clone())),
            request.options(),
            IndexAccessRequirement::Optional,
        ),
        QueryRequest::Traversal(request) => QueryPlan::new(
            QueryKind::Traversal,
            QueryOperator::Traverse(TraversalPlan::new(
                request.start().clone(),
                request.predicate().cloned(),
                request.direction(),
                request.budget(),
            )),
            request.options(),
            IndexAccessRequirement::Optional,
        ),
        QueryRequest::Retrieval(request) => QueryPlan::new(
            QueryKind::Retrieval,
            QueryOperator::Retrieve(RetrievalPlan::new(
                request.mode(),
                request.target().clone(),
                request.expansion(),
            )),
            request.options(),
            retrieval_index_requirement(request.mode()),
        ),
    };

    query_plan
        .validate()
        .map_err(QueryPlannerError::InvalidPlan)?;

    Ok(query_plan)
}

fn retrieval_index_requirement(mode: RetrievalMode) -> IndexAccessRequirement {
    match mode {
        RetrievalMode::Exact => IndexAccessRequirement::Optional,
        RetrievalMode::Lexical
        | RetrievalMode::Conceptual
        | RetrievalMode::Relational
        | RetrievalMode::Semantic => IndexAccessRequirement::Preferred,
    }
}

/// Typed planner failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryPlannerError {
    /// The caller supplied an invalid semantic request.
    InvalidRequest(QueryValidationError),
    /// The validated request could not be represented as a valid logical plan.
    InvalidPlan(super::plan::PlanValidationError),
}

impl fmt::Display for QueryPlannerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error) => {
                write!(formatter, "query request validation failed: {error}")
            }
            Self::InvalidPlan(error) => write!(formatter, "query plan validation failed: {error}"),
        }
    }
}

impl std::error::Error for QueryPlannerError {}

#[cfg(test)]
mod tests {
    use super::{QueryPlannerError, plan};
    use crate::assertion::AssertionObject;
    use crate::identity::EntityId;
    use crate::query::{IndexAccessRequirement, QueryRequest, RetrievalRequest};

    #[test]
    fn retrieval_planner_prefers_indexing_without_making_it_canonical() {
        let request = QueryRequest::Retrieval(RetrievalRequest::lexical("sabr"));
        let plan = plan(&request).expect("retrieval should plan");
        assert_eq!(plan.index_requirement(), IndexAccessRequirement::Preferred);
    }

    #[test]
    fn planner_rejects_invalid_requests_before_creating_a_plan() {
        let request = QueryRequest::Retrieval(RetrievalRequest::lexical("   "));
        let error = plan(&request).expect_err("empty retrieval target must fail");
        assert!(matches!(error, QueryPlannerError::InvalidRequest(_)));
    }

    #[test]
    fn equivalent_lookup_requests_produce_equal_plans() {
        let object = AssertionObject::Entity(EntityId::new("entity-deterministic").unwrap());
        let first = plan(&QueryRequest::Lookup(crate::query::LookupRequest::object(
            object.clone(),
        )))
        .expect("first lookup should plan");
        let second = plan(&QueryRequest::Lookup(crate::query::LookupRequest::object(
            object,
        )))
        .expect("second lookup should plan");
        assert_eq!(first, second);
    }
}
