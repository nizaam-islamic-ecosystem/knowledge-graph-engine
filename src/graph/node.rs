//! Structural graph nodes for Phase 2.
//!
//! A `GraphNode` represents the graph-level presence of one typed semantic
//! reference. It does not create or replace the identity of the underlying
//! semantic object.
//!
//! `GraphNodeId` is a structural graph identity and therefore uses Core's
//! `identity!` mechanism. Generated identity contents are intentionally not
//! part of the semantic contract.

use nizaam_core::identity;

use crate::assertion::AssertionObject;

identity!(
    /// Identifies a node within the in-memory Knowledge Graph structure.
    GraphNodeId
);

/// A structural graph node associated with a typed semantic reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GraphNode {
    id: GraphNodeId,
    reference: AssertionObject,
}

impl GraphNode {
    /// Creates a new graph node for the supplied semantic reference.
    #[must_use]
    pub fn new(reference: AssertionObject) -> Self {
        Self {
            id: GraphNodeId::generate(),
            reference,
        }
    }

    /// Returns the structural graph-node identity.
    #[must_use]
    pub fn id(&self) -> &GraphNodeId {
        &self.id
    }

    /// Returns the typed semantic reference represented by this node.
    #[must_use]
    pub fn reference(&self) -> &AssertionObject {
        &self.reference
    }
}

#[cfg(test)]
mod tests {
    use super::GraphNode;
    use crate::assertion::AssertionObject;
    use crate::identity::EntityId;

    #[test]
    fn creates_a_graph_node_for_a_typed_reference() {
        let entity_id = EntityId::new("entity-1").expect("valid entity identity");
        let reference = AssertionObject::Entity(entity_id.clone());
        let node = GraphNode::new(reference.clone());

        assert!(!node.id().as_str().is_empty());
        assert_eq!(node.reference(), &reference);
    }

    #[test]
    fn graph_nodes_for_the_same_reference_have_distinct_structural_ids() {
        let entity_id = EntityId::new("entity-1").expect("valid entity identity");
        let reference = AssertionObject::Entity(entity_id);

        let first = GraphNode::new(reference.clone());
        let second = GraphNode::new(reference);

        assert_ne!(first.id(), second.id());
        assert_eq!(first.reference(), second.reference());
    }

    #[test]
    fn graph_node_does_not_change_the_underlying_semantic_reference() {
        let entity_id = EntityId::new("entity-1").expect("valid entity identity");
        let node = GraphNode::new(AssertionObject::Entity(entity_id.clone()));

        assert_eq!(node.reference(), &AssertionObject::Entity(entity_id));
    }
}
