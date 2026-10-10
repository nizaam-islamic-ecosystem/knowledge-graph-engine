//! Level 3 Phase 6 negative-boundary tests.
//!
//! These tests ensure malformed query requests, impossible budgets, invalid
//! cursors, unavailable reasoning, and missing graph starts remain explicit
//! failures instead of becoming implicit fallbacks.

use std::cell::Cell;

use nizaam_core::identity::{CorrelationId, OperationId};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_knowledge_graph::assertion::AssertionObject;
use nizaam_knowledge_graph::graph::{
    Graph, TraversalBounds, TraversalDirection, TraversalError, traverse_bounded,
};
use nizaam_knowledge_graph::identity::EntityId;
use nizaam_knowledge_graph::ingestion::PublicationOutcome;
use nizaam_knowledge_graph::query::{
    LookupRequest, Pagination, QueryAccess, QueryExecutionError, QueryMatchCandidate,
    QueryMatchType, QueryReference, QueryRequest, QueryValidationError, ReasoningProfile,
    TraversalBudget, TraversalRequest, execute, plan,
};

#[derive(Debug)]
struct AccessError;

impl std::fmt::Display for AccessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("negative boundary access error")
    }
}

impl std::error::Error for AccessError {}

fn context() -> OperationContext {
    OperationContext::new(Operation::new(
        OperationId::new("nizaam.kg.phase6.negative.operation").unwrap(),
        CorrelationId::new("nizaam.kg.phase6.negative.correlation").unwrap(),
    ))
}

fn entity(value: &str) -> AssertionObject {
    AssertionObject::Entity(EntityId::new(value).expect("valid entity id"))
}

fn published_candidate(value: &str) -> QueryMatchCandidate {
    QueryMatchCandidate::new(
        QueryReference::Object(entity(value)),
        QueryMatchType::Lookup,
    )
    .with_publication(PublicationOutcome::Published)
}

#[test]
fn traversal_request_rejects_zero_work_budgets() {
    let request = QueryRequest::Traversal(
        TraversalRequest::from(entity("budget"), 2)
            .with_budget(TraversalBudget::new(2).with_max_edges(0)),
    );

    assert!(matches!(
        request.validate(),
        Err(QueryValidationError::ZeroBudget("max_edges"))
    ));
}

#[test]
fn lookup_request_rejects_mixed_cursor_and_offset() {
    let request = QueryRequest::Lookup(
        LookupRequest::object(entity("pagination")).with_options(
            nizaam_knowledge_graph::query::QueryOptions::new()
                .with_pagination(Pagination::new(10).with_cursor("cursor").with_offset(1)),
        ),
    );

    assert!(matches!(
        request.validate(),
        Err(QueryValidationError::CursorAndOffsetConflict)
    ));
}

#[test]
fn empty_filter_groups_are_rejected_before_execution() {
    let filter = nizaam_knowledge_graph::query::Filter::and([]);
    assert!(filter.is_err());
}

#[test]
fn invalid_cursor_never_falls_back_to_offset_zero() {
    let request = QueryRequest::Lookup(
        LookupRequest::object(entity("cursor")).with_options(
            nizaam_knowledge_graph::query::QueryOptions::new()
                .with_pagination(Pagination::new(1).with_cursor("invalid-cursor")),
        ),
    );
    let plan = plan(&request).expect("request should plan");
    let access = SingleCandidateAccess {
        candidate: published_candidate("cursor"),
        calls: Cell::new(0),
    };

    assert!(matches!(
        execute(&access, &plan, &context()),
        Err(QueryExecutionError::InvalidCursor(value)) if value == "invalid-cursor"
    ));
    assert_eq!(access.calls.get(), 1);
}

struct SingleCandidateAccess {
    candidate: QueryMatchCandidate,
    calls: Cell<usize>,
}

impl QueryAccess for SingleCandidateAccess {
    type Error = AccessError;

    fn lookup(
        &self,
        _target: &nizaam_knowledge_graph::query::LookupTarget,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        Ok(vec![self.candidate.clone()])
    }

    fn traverse(
        &self,
        _plan: &nizaam_knowledge_graph::query::TraversalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        Ok(vec![self.candidate.clone()])
    }

    fn retrieve(
        &self,
        _plan: &nizaam_knowledge_graph::query::RetrievalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        Ok(vec![self.candidate.clone()])
    }
}

#[test]
fn reasoning_unavailability_is_rejected_before_access() {
    let request = QueryRequest::Lookup(LookupRequest::object(entity("reasoning")).with_options(
        nizaam_knowledge_graph::query::QueryOptions::new().with_reasoning(ReasoningProfile::Basic),
    ));
    let plan = plan(&request).expect("request should plan");
    let access = SingleCandidateAccess {
        candidate: published_candidate("reasoning"),
        calls: Cell::new(0),
    };

    assert!(matches!(
        execute(&access, &plan, &context()),
        Err(QueryExecutionError::ReasoningUnavailable {
            profile: ReasoningProfile::Basic
        })
    ));
    assert_eq!(access.calls.get(), 0);
}

#[test]
fn missing_graph_start_is_rejected_instead_of_becoming_an_empty_result() {
    let graph = Graph::new();
    let missing_node = nizaam_knowledge_graph::graph::GraphNode::new(entity("missing"));
    let missing = missing_node.id().clone();

    let error = traverse_bounded(
        &graph,
        &missing,
        TraversalDirection::Forward,
        TraversalBounds::new(1),
        |_edge| {
            Err(TraversalError::ResolutionUnavailable {
                assertion_id: nizaam_knowledge_graph::identity::KnowledgeAssertionId::new(
                    "missing-assertion",
                )
                .unwrap(),
            })
        },
    )
    .expect_err("missing start node must be explicit");

    assert!(matches!(error, TraversalError::StartNodeNotFound { .. }));
}
