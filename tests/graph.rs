//! Level 3 public-boundary tests for the Phase 2 graph model.
//!
//! These tests verify structural graph behavior, canonical assertion
//! association, and one-step canonical/inverse traversal.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::graph::{
    Graph, GraphNode, PathError, TraversalBounds, TraversalDirection, TraversalError,
    TraversalPath, traverse, traverse_bounded,
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

#[test]
fn traversal_path_requires_connected_steps_and_preserves_endpoints() {
    let first = assertion("first", "entity-1", "entity-2");
    let second = assertion("second", "entity-2", "entity-3");
    let shared_middle = GraphNode::new(first.object().clone());
    let first_edge = {
        let source = GraphNode::new(first.subject().clone());
        nizaam_knowledge_graph::graph::GraphEdge::new(&first, &source, &shared_middle)
            .expect("first edge should be valid")
    };
    let second_edge = {
        let target = GraphNode::new(second.object().clone());
        nizaam_knowledge_graph::graph::GraphEdge::new(&second, &shared_middle, &target)
            .expect("second edge should be valid")
    };
    let first_relationship = relationship("first", RelationshipCharacteristics::new());
    let second_relationship = relationship("second", RelationshipCharacteristics::new());
    let first_step = traverse(
        &first_edge,
        &first,
        &first_relationship,
        TraversalDirection::Forward,
    )
    .expect("first step should be valid");
    let second_step = traverse(
        &second_edge,
        &second,
        &second_relationship,
        TraversalDirection::Forward,
    )
    .expect("second step should be valid");

    let mut path = TraversalPath::new(first_step.source().clone());
    path.push(first_step).expect("first step should connect");
    path.push(second_step).expect("second step should connect");

    assert_eq!(path.len(), 2);
    assert_eq!(path.start(), path.steps()[0].source());
    assert_eq!(path.end(), path.steps()[1].target());

    let disconnected = assertion("other", "entity-9", "entity-10");
    let disconnected_edge = {
        let source = GraphNode::new(disconnected.subject().clone());
        let target = GraphNode::new(disconnected.object().clone());
        nizaam_knowledge_graph::graph::GraphEdge::new(&disconnected, &source, &target)
            .expect("disconnected edge should be valid")
    };
    let disconnected_step = traverse(
        &disconnected_edge,
        &disconnected,
        &relationship("other", RelationshipCharacteristics::new()),
        TraversalDirection::Forward,
    )
    .expect("disconnected step should be semantically valid");

    assert!(matches!(
        path.clone().push(disconnected_step),
        Err(PathError::Disconnected { .. })
    ));
}

#[test]
fn bounded_traversal_is_explicitly_depth_limited_and_cycle_safe() {
    let first = assertion("first", "entity-1", "entity-2");
    let second = assertion("second", "entity-2", "entity-3");
    let cycle = assertion("cycle", "entity-3", "entity-1");
    let mut graph = Graph::new();
    graph.add_assertion(&first).expect("first edge");
    graph.add_assertion(&second).expect("second edge");
    graph.add_assertion(&cycle).expect("cycle edge");

    let semantics = [
        (
            first.id().clone(),
            first.clone(),
            relationship("first", RelationshipCharacteristics::new()),
        ),
        (
            second.id().clone(),
            second.clone(),
            relationship("second", RelationshipCharacteristics::new()),
        ),
        (
            cycle.id().clone(),
            cycle.clone(),
            relationship("cycle", RelationshipCharacteristics::new()),
        ),
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
                .ok_or(TraversalError::ResolutionUnavailable {
                    assertion_id: edge.assertion_id().clone(),
                })
        },
    )
    .expect("bounded traversal should succeed");

    assert_eq!(paths[0].len(), 0);
    assert!(paths.iter().any(|path| path.len() == 1));
    assert!(paths.iter().any(|path| path.len() == 2));
    assert!(paths.iter().all(|path| path.len() <= 2));
    assert_eq!(paths.iter().filter(|path| path.len() == 3).count(), 0);
}

#[test]
fn incident_edge_order_is_deterministic_for_bounded_traversal_inputs() {
    let first = assertion("first", "entity-1", "entity-3");
    let second = assertion("second", "entity-1", "entity-2");
    let mut graph = Graph::new();
    graph.add_assertion(&first).expect("first edge");
    graph.add_assertion(&second).expect("second edge");

    let node = graph
        .node_for_reference(first.subject())
        .expect("shared source node should exist");
    let first_observation = graph
        .incident_edges(node.id())
        .iter()
        .map(|edge| edge.assertion_id().as_str().to_owned())
        .collect::<Vec<_>>();
    let second_observation = graph
        .incident_edges(node.id())
        .iter()
        .map(|edge| edge.assertion_id().as_str().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(first_observation, second_observation);
}
