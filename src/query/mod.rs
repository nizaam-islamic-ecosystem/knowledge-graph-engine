//! Phase 6 typed query, planning, execution, filtering, ranking, and results.
//!
//! The query module is intentionally storage-independent. It owns the semantic
//! request/plan/result boundary while delegating canonical data access to a
//! `QueryAccess` implementation and universal execution context to Nizaam Core.

mod execution;
mod filter;
mod plan;
mod planner;
mod ranking;
mod request;
mod result;

pub use execution::{QueryAccess, QueryExecutionError, execute, execute_with_ranking};
pub use filter::{EntityConstraint, EntityPosition, Filter, FilterError, TemporalFilter};
pub use plan::{
    IndexAccessRequirement, LookupPlan, PlanValidationError, QueryOperator, QueryPlan,
    RetrievalPlan, TraversalPlan,
};
pub use planner::{QueryPlannerError, plan};
pub use ranking::{
    DeterministicRankingProvider, RankedCandidate, RankingMetadata, RankingProfile,
    RankingProvider, RankingSignal, rank_query_candidates,
};
pub use request::{
    LookupRequest, LookupTarget, Pagination, QueryKind, QueryOptions, QueryOrdering, QueryRequest,
    QueryValidationError, ReasoningProfile, RetrievalMode, RetrievalRequest, RetrievalTarget,
    SemanticExpansion, TraversalBudget, TraversalRequest, VisibilityProfile,
};
pub use result::{
    InferenceStatus, MatchExplanation, PaginationMetadata, QueryExplanation, QueryMatchCandidate,
    QueryMatchType, QueryReference, QueryResult, QueryResultItem, ResultExplanationError,
};

#[cfg(test)]
mod tests {
    use super::{
        Filter, IndexAccessRequirement, LookupRequest, QueryKind, QueryOptions, QueryRequest,
        RankingProfile, RetrievalMode, RetrievalRequest, TraversalBudget, TraversalRequest, plan,
    };
    use crate::identity::EntityId;

    fn entity(value: &str) -> crate::assertion::AssertionObject {
        crate::assertion::AssertionObject::Entity(EntityId::new(value).expect("valid entity"))
    }

    #[test]
    fn public_query_boundary_plans_each_initial_query_category() {
        let lookup = plan(&QueryRequest::Lookup(LookupRequest::object(entity(
            "entity-1",
        ))))
        .expect("lookup should plan");
        assert_eq!(lookup.kind(), QueryKind::Lookup);
        assert_eq!(lookup.index_requirement(), IndexAccessRequirement::Optional);

        let traversal = plan(&QueryRequest::Traversal(
            TraversalRequest::from(entity("entity-1"), 2).with_budget(TraversalBudget::new(2)),
        ))
        .expect("traversal should plan");
        assert_eq!(traversal.kind(), QueryKind::Traversal);
        assert_eq!(
            traversal.index_requirement(),
            IndexAccessRequirement::Optional
        );

        let retrieval = plan(&QueryRequest::Retrieval(RetrievalRequest::lexical(
            "patience",
        )))
        .expect("lexical retrieval should plan");
        assert_eq!(retrieval.kind(), QueryKind::Retrieval);
        assert_eq!(
            retrieval.index_requirement(),
            IndexAccessRequirement::Preferred
        );
    }

    #[test]
    fn public_query_options_preserve_typed_filter_and_ranking_profile() {
        let options = QueryOptions::new()
            .with_filter(Filter::all())
            .with_ranking(RankingProfile::Semantic);
        let request =
            QueryRequest::Retrieval(RetrievalRequest::lexical("sabr").with_options(options));

        let plan = plan(&request).expect("request should plan");
        assert_eq!(plan.ranking(), RankingProfile::Semantic);
        assert!(matches!(
            plan.operator(),
            super::QueryOperator::Retrieve(retrieval) if retrieval.mode() == RetrievalMode::Lexical
        ));
    }
}
