//! Level 3 Phase 6 query-layer tests.
//!
//! These tests exercise the public request, filter, plan, planner, ranking,
//! result, and execution boundaries without introducing a second storage layer.

use std::cell::Cell;

use nizaam_core::identity::{CorrelationId, OperationId};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_knowledge_graph::assertion::{AssertionObject, AssertionStatus};
use nizaam_knowledge_graph::identity::EntityId;
use nizaam_knowledge_graph::ingestion::PublicationOutcome;
use nizaam_knowledge_graph::query::{
    DeterministicRankingProvider, Filter, InferenceStatus, LookupRequest, Pagination, QueryAccess,
    QueryExecutionError, QueryKind, QueryMatchCandidate, QueryMatchType, QueryOptions,
    QueryOrdering, QueryReference, QueryRequest, RankingProfile, ReasoningProfile, RetrievalMode,
    RetrievalRequest, RetrievalTarget, VisibilityProfile, execute, plan,
};

#[derive(Debug)]
struct AccessError;

impl std::fmt::Display for AccessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("test access error")
    }
}

impl std::error::Error for AccessError {}

struct TestAccess {
    candidates: Vec<QueryMatchCandidate>,
    calls: Cell<usize>,
}

impl QueryAccess for TestAccess {
    type Error = AccessError;

    fn lookup(
        &self,
        _target: &nizaam_knowledge_graph::query::LookupTarget,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.candidates.clone())
    }

    fn traverse(
        &self,
        _plan: &nizaam_knowledge_graph::query::TraversalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.candidates.clone())
    }

    fn retrieve(
        &self,
        _plan: &nizaam_knowledge_graph::query::RetrievalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.candidates.clone())
    }
}

fn context() -> OperationContext {
    OperationContext::new(Operation::new(
        OperationId::new("nizaam.kg.phase6.query.operation").expect("valid operation"),
        CorrelationId::new("nizaam.kg.phase6.query.correlation").expect("valid correlation"),
    ))
}

fn entity(value: &str) -> AssertionObject {
    AssertionObject::Entity(EntityId::new(value).expect("valid entity id"))
}

fn candidate(id: &str, kind: QueryMatchType) -> QueryMatchCandidate {
    QueryMatchCandidate::new(QueryReference::Object(entity(id)), kind)
        .with_publication(PublicationOutcome::Published)
}

#[test]
fn query_planner_preserves_request_kind_and_index_dependency() {
    let exact = QueryRequest::Retrieval(RetrievalRequest::exact(entity("exact")));
    let lexical = QueryRequest::Retrieval(RetrievalRequest::lexical("knowledge"));
    let exact_plan = plan(&exact).expect("exact request should plan");
    let lexical_plan = plan(&lexical).expect("lexical request should plan");

    assert_eq!(exact_plan.kind(), QueryKind::Retrieval);
    assert_eq!(
        exact_plan.index_requirement(),
        nizaam_knowledge_graph::query::IndexAccessRequirement::Optional
    );
    assert_eq!(
        lexical_plan.index_requirement(),
        nizaam_knowledge_graph::query::IndexAccessRequirement::Preferred
    );
}

#[test]
fn query_filter_tree_and_visibility_are_applied_before_ranking() {
    let source = nizaam_knowledge_graph::identity::SourceId::new("source-phase6").unwrap();
    let visible = candidate("visible", QueryMatchType::Exact)
        .with_sources([source.clone()])
        .with_status(AssertionStatus::Accepted);
    let inferred = candidate("inferred", QueryMatchType::Exact)
        .with_sources([source.clone()])
        .with_inference_status(InferenceStatus::Inferred);
    let filter = Filter::and([
        Filter::Source(source),
        Filter::Epistemic(AssertionStatus::Accepted),
    ])
    .expect("filter tree should be valid");
    let request = QueryRequest::Lookup(
        LookupRequest::object(entity("visible")).with_options(
            QueryOptions::new()
                .with_filter(filter)
                .with_visibility(VisibilityProfile::public()),
        ),
    );
    let planned = plan(&request).expect("lookup should plan");
    let access = TestAccess {
        candidates: vec![inferred, visible.clone()],
        calls: Cell::new(0),
    };

    let result = execute(&access, &planned, &context()).expect("query should execute");

    assert_eq!(result.items().len(), 1);
    assert_eq!(result.items()[0].reference(), visible.reference());
    assert_eq!(access.calls.get(), 1);
}

#[test]
fn ranking_is_deterministic_and_stable_by_reference() {
    let provider = DeterministicRankingProvider;
    let ranked = nizaam_knowledge_graph::query::rank_query_candidates(
        &provider,
        &RankingProfile::General,
        [
            candidate("entity-b", QueryMatchType::Semantic),
            candidate("entity-a", QueryMatchType::Semantic),
        ],
    );

    assert_eq!(ranked.len(), 2);
    assert_eq!(
        ranked[0].candidate.reference().stable_key(),
        "entity:entity-a"
    );
    assert!(ranked[0].ranking.score() > 0);
}

#[test]
fn query_result_preserves_reference_ranking_and_explanation_metadata() {
    let request = QueryRequest::Lookup(
        LookupRequest::object(entity("result"))
            .with_options(QueryOptions::new().with_ranking(RankingProfile::Semantic)),
    );
    let planned = plan(&request).expect("request should plan");
    let access = TestAccess {
        candidates: vec![candidate("result", QueryMatchType::Semantic)],
        calls: Cell::new(0),
    };

    let result = execute(&access, &planned, &context()).expect("query should execute");

    assert_eq!(result.items().len(), 1);
    assert_eq!(result.items()[0].ranking().profile().name(), "semantic");
    assert_eq!(result.explanation().ranking_profile(), "semantic");
    assert!(!result.explanation().reasoning_requested());
}

#[test]
fn cursor_pagination_is_deterministic_across_query_pages() {
    let first_request =
        QueryRequest::Lookup(LookupRequest::object(entity("page")).with_options(
            QueryOptions::new().with_pagination(
                Pagination::new(1).with_ordering(QueryOrdering::ReferenceAscending),
            ),
        ));
    let first_plan = plan(&first_request).expect("first page should plan");
    let access = TestAccess {
        candidates: vec![
            candidate("page-b", QueryMatchType::Exact),
            candidate("page-a", QueryMatchType::Exact),
        ],
        calls: Cell::new(0),
    };
    let first = execute(&access, &first_plan, &context()).expect("first page should execute");
    let cursor = first
        .next_cursor()
        .expect("first page should provide a cursor");

    let second_request = QueryRequest::Lookup(
        LookupRequest::object(entity("page")).with_options(
            QueryOptions::new().with_pagination(
                Pagination::new(1)
                    .with_cursor(cursor)
                    .with_ordering(QueryOrdering::ReferenceAscending),
            ),
        ),
    );
    let second_plan = plan(&second_request).expect("second page should plan");
    let second = execute(&access, &second_plan, &context()).expect("second page should execute");

    assert_eq!(second.items()[0].reference().stable_key(), "entity:page-b");
    assert!(second.pagination().cursor_used());
}

#[test]
fn reasoning_requests_are_explicitly_rejected_at_phase6_execution_boundary() {
    let request = QueryRequest::Lookup(
        LookupRequest::object(entity("reasoning"))
            .with_options(QueryOptions::new().with_reasoning(ReasoningProfile::Basic)),
    );
    let planned = plan(&request).expect("request should plan");
    let access = TestAccess {
        candidates: vec![candidate("reasoning", QueryMatchType::Exact)],
        calls: Cell::new(0),
    };

    assert!(matches!(
        execute(&access, &planned, &context()),
        Err(QueryExecutionError::ReasoningUnavailable { .. })
    ));
    assert_eq!(access.calls.get(), 0);
}

#[test]
fn retrieval_targets_remain_typed_and_provider_neutral() {
    let request = QueryRequest::Retrieval(
        RetrievalRequest::semantic(RetrievalTarget::Text("semantic text".to_owned()))
            .with_expansion(nizaam_knowledge_graph::query::SemanticExpansion::Controlled),
    );
    let planned = plan(&request).expect("semantic retrieval should plan");

    match planned.operator() {
        nizaam_knowledge_graph::query::QueryOperator::Retrieve(retrieval) => {
            assert_eq!(retrieval.mode(), RetrievalMode::Semantic);
            assert_eq!(
                retrieval.expansion(),
                nizaam_knowledge_graph::query::SemanticExpansion::Controlled
            );
        }
        other => panic!("unexpected query operator: {other:?}"),
    }
}
