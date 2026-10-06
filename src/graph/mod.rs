//! Phase 2 graph module boundary.
//!
//! The graph layer provides structural nodes and edges, a minimal in-memory
//! graph with adjacency, and one-step canonical/inverse traversal views.
//! Full query traversal, path execution, filtering, ranking, and persistence
//! remain later-phase responsibilities.

mod edge;
mod model;
mod node;
mod path;
mod traversal;

pub use edge::{GraphEdge, GraphEdgeError, GraphEdgeId};
pub use model::{Graph, GraphError};
pub use node::{GraphNode, GraphNodeId};
pub use traversal::{TraversalDirection, TraversalError, TraversalStep, traverse};

#[cfg(test)]
mod tests {
    use super::{Graph, GraphNode, TraversalDirection, traverse};
    use crate::assertion::KnowledgeAssertion;
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        Qualifiers,
    };
    use crate::identity::EntityId;
    use crate::relationship::{
        Relationship, RelationshipCharacteristic, RelationshipCharacteristics,
        RelationshipDirection, RelationshipFamily, RelationshipPredicate,
    };

    fn entity(value: &str) -> EntityId {
        EntityId::new(value).expect("valid entity identity")
    }

    fn assertion(predicate: &str) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            AssertionPredicate::new(predicate).expect("valid predicate"),
            AssertionObject::Entity(entity("entity-2")),
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
    fn public_graph_boundary_composes_nodes_and_edges() {
        let assertion = assertion("has-name");
        let mut graph = Graph::new();

        let edge_id = graph
            .add_assertion(&assertion)
            .expect("assertion should be represented structurally");

        let source = graph
            .node_for_reference(assertion.subject())
            .expect("source node should exist");
        let target = graph
            .node_for_reference(assertion.object())
            .expect("target node should exist");

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.edge(&edge_id).unwrap().source(), source.id());
        assert_eq!(graph.edge(&edge_id).unwrap().target(), target.id());
    }

    #[test]
    fn public_graph_boundary_supports_multiple_assertions_between_the_same_nodes() {
        let mut graph = Graph::new();
        let has_name = assertion("has-name");
        let aliases = assertion("aliases");

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
    fn public_graph_boundary_supports_canonical_and_inverse_views() {
        let assertion = assertion("has-name");
        let mut graph = Graph::new();
        let edge_id = graph
            .add_assertion(&assertion)
            .expect("assertion should be added");

        let edge = graph.edge(&edge_id).expect("edge should be available");

        let relationship = relationship("has-name", RelationshipCharacteristics::new())
            .with_inverse_predicate(
                RelationshipPredicate::new("name-of").expect("valid inverse predicate"),
            )
            .expect("valid inverse");

        let forward = traverse(edge, &assertion, &relationship, TraversalDirection::Forward)
            .expect("forward traversal should succeed");

        let inverse = traverse(edge, &assertion, &relationship, TraversalDirection::Inverse)
            .expect("inverse traversal should succeed");

        assert_eq!(forward.source(), edge.source());
        assert_eq!(forward.target(), edge.target());
        assert_eq!(inverse.source(), edge.target());
        assert_eq!(inverse.target(), edge.source());

        assert_eq!(forward.assertion_id(), inverse.assertion_id());
        assert_eq!(forward.predicate().as_str(), "kg.relationship.has-name");
        assert_eq!(inverse.predicate().as_str(), "kg.relationship.name-of");
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn public_graph_boundary_distinguishes_symmetric_behavior_from_inverse_behavior() {
        let assertion = assertion("aliases");
        let mut graph = Graph::new();
        let edge_id = graph
            .add_assertion(&assertion)
            .expect("assertion should be added");
        let edge = graph.edge(&edge_id).unwrap();

        let relationship = relationship(
            "aliases",
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
                .expect("valid characteristics"),
        );

        let reverse = traverse(edge, &assertion, &relationship, TraversalDirection::Inverse)
            .expect("symmetric reverse traversal should succeed");

        assert_eq!(reverse.predicate().as_str(), "kg.relationship.aliases");
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn graph_nodes_preserve_typed_semantic_references() {
        let reference = AssertionObject::Entity(entity("entity-1"));
        let node = GraphNode::new(reference.clone());

        assert_eq!(node.reference(), &reference);
    }
}
