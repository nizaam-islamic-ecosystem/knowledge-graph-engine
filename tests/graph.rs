//! Level 3 public-boundary tests for the Phase 2 graph model.
//!
//! These tests verify structural graph behavior, canonical assertion
//! association, and one-step canonical/inverse traversal.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::graph::{
    Graph, GraphNode, TraversalDirection, TraversalError, traverse,
};
use nizaam_knowledge_graph::identity::EntityId;
use nizaam_knowledge_graph::relationship::{
    Relationship, RelationshipCharacteristic, RelationshipCharacteristics, RelationshipDirection,
    RelationshipFamily, RelationshipPredicate,
};

fn entity(value: &str) -> EntityId {
    EntityId::new(value).expect("valid entity identity")
}

fn assertion(predicate: &str, subject: &str, object: &str) -> KnowledgeAssertion {
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

fn relationship(predicate: &str, characteristics: RelationshipCharacteristics) -> Relationship {
    Relationship::new(
        RelationshipPredicate::new(predicate).expect("valid predicate"),
        RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
        RelationshipDirection::SubjectToObject,
        characteristics,
    )
}

#[test]
fn graph_reuses_one_node_for_the_same_typed_semantic_reference() {
    let mut graph = Graph::new();
    let reference = AssertionObject::Entity(entity("entity-1"));

    let first = graph.add_node(reference.clone());
    let second = graph.add_node(reference);

    assert_eq!(first, second);
    assert_eq!(graph.node_count(), 1);
}

#[test]
fn graph_assertion_creates_two_nodes_and_one_structural_edge() {
    let mut graph = Graph::new();
    let assertion = assertion("has-name", "entity-1", "entity-2");

    let edge_id = graph
        .add_assertion(&assertion)
        .expect("assertion should be represented structurally");

    let source = graph
        .node_for_reference(assertion.subject())
        .expect("source node should exist");
    let target = graph
        .node_for_reference(assertion.object())
        .expect("target node should exist");
    let edge = graph.edge(&edge_id).expect("edge should exist");

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(edge.assertion_id(), assertion.id());
    assert_eq!(edge.source(), source.id());
    assert_eq!(edge.target(), target.id());
}

#[test]
fn graph_rejects_duplicate_canonical_assertion_association() {
    let mut graph = Graph::new();
    let assertion = assertion("has-name", "entity-1", "entity-2");

    graph
        .add_assertion(&assertion)
        .expect("first assertion should be added");

    let error = graph
        .add_assertion(&assertion)
        .expect_err("same canonical assertion must not be duplicated");

    match error {
        nizaam_knowledge_graph::graph::GraphError::AssertionAlreadyAssociated {
            assertion_id,
            ..
        } => assert_eq!(assertion_id, *assertion.id()),
        other => panic!("unexpected graph error: {other:?}"),
    }
}

#[test]
fn graph_supports_multiple_relationships_between_the_same_nodes() {
    let mut graph = Graph::new();
    let has_name = assertion("has-name", "entity-1", "entity-2");
    let aliases = assertion("aliases", "entity-1", "entity-2");

    graph
        .add_assertion(&has_name)
        .expect("first assertion should be added");
    graph
        .add_assertion(&aliases)
        .expect("second assertion should be added");

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn graph_exposes_basic_outgoing_and_incoming_adjacency() {
    let mut graph = Graph::new();
    let assertion = assertion("has-name", "entity-1", "entity-2");

    let edge_id = graph
        .add_assertion(&assertion)
        .expect("assertion should be added");

    let source = graph
        .node_for_reference(assertion.subject())
        .expect("source node should exist");
    let target = graph
        .node_for_reference(assertion.object())
        .expect("target node should exist");
    let outgoing = graph.outgoing_edges(source.id());
    let incoming = graph.incoming_edges(target.id());

    assert_eq!(outgoing.len(), 1);
    assert_eq!(incoming.len(), 1);
    assert_eq!(outgoing[0].id(), &edge_id);
    assert_eq!(incoming[0].id(), &edge_id);
}

#[test]
fn inverse_traversal_reuses_the_same_canonical_edge_and_assertion() {
    let assertion = assertion("has-name", "entity-1", "entity-2");
    let mut graph = Graph::new();
    let edge_id = graph
        .add_assertion(&assertion)
        .expect("assertion should be added");
    let edge = graph.edge(&edge_id).expect("edge should exist");

    let relationship = relationship("has-name", RelationshipCharacteristics::new())
        .with_inverse_predicate(
            RelationshipPredicate::new("name-of").expect("valid inverse predicate"),
        )
        .expect("inverse should attach");

    let forward = traverse(edge, &assertion, &relationship, TraversalDirection::Forward)
        .expect("forward traversal should succeed");
    let inverse = traverse(edge, &assertion, &relationship, TraversalDirection::Inverse)
        .expect("inverse traversal should succeed");

    assert_eq!(forward.edge_id(), inverse.edge_id());
    assert_eq!(forward.assertion_id(), inverse.assertion_id());
    assert_eq!(forward.source(), edge.source());
    assert_eq!(forward.target(), edge.target());
    assert_eq!(inverse.source(), edge.target());
    assert_eq!(inverse.target(), edge.source());
    assert_eq!(forward.predicate().as_str(), "kg.relationship.has-name");
    assert_eq!(inverse.predicate().as_str(), "kg.relationship.name-of");
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn symmetric_traversal_reverses_endpoints_without_changing_the_predicate() {
    let assertion = assertion("aliases", "entity-1", "entity-2");
    let mut graph = Graph::new();
    let edge_id = graph
        .add_assertion(&assertion)
        .expect("assertion should be added");
    let edge = graph.edge(&edge_id).expect("edge should exist");

    let characteristics =
        RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
            .expect("valid characteristics");
    let relationship = relationship("aliases", characteristics);

    let reverse = traverse(edge, &assertion, &relationship, TraversalDirection::Inverse)
        .expect("symmetric reverse traversal should succeed");

    assert_eq!(reverse.source(), edge.target());
    assert_eq!(reverse.target(), edge.source());
    assert_eq!(reverse.predicate().as_str(), "kg.relationship.aliases");
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn inverse_traversal_requires_inverse_or_symmetric_relationship_semantics() {
    let assertion = assertion("has-name", "entity-1", "entity-2");
    let source = GraphNode::new(assertion.subject().clone());
    let target = GraphNode::new(assertion.object().clone());
    let edge = nizaam_knowledge_graph::graph::GraphEdge::new(&assertion, &source, &target)
        .expect("matching nodes should create a valid edge");
    let relationship = relationship("has-name", RelationshipCharacteristics::new());

    let error = traverse(
        &edge,
        &assertion,
        &relationship,
        TraversalDirection::Inverse,
    )
    .expect_err("reverse traversal should be unavailable");

    assert_eq!(error, TraversalError::ReverseTraversalUnavailable);
}
