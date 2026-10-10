//! Level 3 Phase 6 cross-module integration tests.
//!
//! These tests connect bounded graph traversal, logical query planning, and
//! reference-oriented result shaping without introducing physical storage.

use std::cell::Cell;

use nizaam_core::identity::{CorrelationId, OperationId};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::graph::{Graph, TraversalBounds, TraversalDirection, traverse_bounded};
use nizaam_knowledge_graph::identity::EntityId;
use nizaam_knowledge_graph::ingestion::PublicationOutcome;
use nizaam_knowledge_graph::query::{
    LookupRequest, QueryAccess, QueryMatchCandidate, QueryMatchType, QueryReference, QueryRequest,
    execute, plan,
};
use nizaam_knowledge_graph::relationship::{
    Relationship, RelationshipCharacteristics, RelationshipDirection, RelationshipFamily,
    RelationshipPredicate,
};

#[derive(Debug)]
struct AccessError;

impl std::fmt::Display for AccessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("integration access error")
    }
}

impl std::error::Error for AccessError {}

struct GraphBackedAccess {
    candidate: QueryMatchCandidate,
    calls: Cell<usize>,
}

impl QueryAccess for GraphBackedAccess {
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

fn entity(value: &str) -> EntityId {
    EntityId::new(value).expect("valid entity id")
}

fn assertion(subject: &str, object: &str, predicate: &str) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::Entity(entity(subject)),
        AssertionPredicate::new(predicate).expect("valid predicate"),
        AssertionObject::Entity(entity(object)),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    )
}

fn relationship(predicate: &str) -> Relationship {
    Relationship::new(
        RelationshipPredicate::new(predicate).expect("valid predicate"),
        RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
        RelationshipDirection::SubjectToObject,
        RelationshipCharacteristics::new(),
    )
}

fn context() -> OperationContext {
    OperationContext::new(Operation::new(
        OperationId::new("nizaam.kg.phase6.integration.operation").unwrap(),
        CorrelationId::new("nizaam.kg.phase6.integration.correlation").unwrap(),
    ))
}

#[test]
fn bounded_graph_path_can_flow_into_query_reference_results() {
    let first = assertion("entity-1", "entity-2", "knows");
    let second = assertion("entity-2", "entity-3", "knows");
    let mut graph = Graph::new();
    graph.add_assertion(&first).expect("first edge");
    graph.add_assertion(&second).expect("second edge");

    let semantics = [
        (first.id().clone(), first.clone(), relationship("knows")),
        (second.id().clone(), second.clone(), relationship("knows")),
    ];
    let start = graph
        .node_for_reference(first.subject())
        .expect("start node should exist")
        .id()
        .clone();

    let paths = traverse_bounded(
        &graph,
        &start,
        TraversalDirection::Forward,
        TraversalBounds::new(2),
        |edge| {
            semantics
                .iter()
                .find(|(id, _, _)| id == edge.assertion_id())
                .map(|(_, assertion, relationship)| (assertion.clone(), relationship.clone()))
                .ok_or(
                    nizaam_knowledge_graph::graph::TraversalError::ResolutionUnavailable {
                        assertion_id: edge.assertion_id().clone(),
                    },
                )
        },
    )
    .expect("bounded graph traversal should succeed");

    let path = paths
        .iter()
        .find(|path| path.len() == 2)
        .expect("two-hop path should exist")
        .clone();
    let candidate = QueryMatchCandidate::new(
        QueryReference::Object(AssertionObject::Entity(entity("entity-3"))),
        QueryMatchType::Traversal,
    )
    .with_path(path.clone())
    .with_publication(PublicationOutcome::Published);

    let access = GraphBackedAccess {
        candidate,
        calls: Cell::new(0),
    };
    let request = QueryRequest::Lookup(LookupRequest::object(entity("entity-3")));
    let plan = plan(&request).expect("lookup request should plan");
    let result = execute(&access, &plan, &context()).expect("query should execute");

    assert_eq!(result.items().len(), 1);
    assert_eq!(result.items()[0].path(), Some(&path));
    assert_eq!(result.items()[0].match_type(), QueryMatchType::Traversal);
    assert_eq!(access.calls.get(), 1);
}

#[test]
fn query_public_surface_keeps_execution_context_and_reference_identity_separate() {
    let candidate = QueryMatchCandidate::new(
        QueryReference::Object(AssertionObject::Entity(entity("entity-context"))),
        QueryMatchType::Lookup,
    )
    .with_publication(PublicationOutcome::Published);
    let access = GraphBackedAccess {
        candidate,
        calls: Cell::new(0),
    };
    let request = QueryRequest::Lookup(LookupRequest::object(entity("entity-context")));
    let plan = plan(&request).expect("lookup request should plan");

    let result = execute(&access, &plan, &context()).expect("query should execute");

    assert_eq!(
        result.items()[0].reference().stable_key(),
        "entity:entity-context"
    );
    assert_eq!(result.explanation().ranking_profile(), "general");
    assert_eq!(access.calls.get(), 1);
}
