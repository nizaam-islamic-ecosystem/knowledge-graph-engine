//! Basic in-memory graph structure and adjacency for Phase 2.
//!
//! The graph stores structural nodes and edges only. A graph edge references a
//! canonical `KnowledgeAssertion` by identity; the graph does not become a
//! second assertion store.
//!
//! Full query planning, multi-hop traversal, filtering, ranking, persistence,
//! and other later-phase behavior are intentionally out of scope here.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::assertion::{AssertionObject, KnowledgeAssertion};
use crate::identity::KnowledgeAssertionId;

use super::edge::{GraphEdge, GraphEdgeError, GraphEdgeId};
use super::node::{GraphNode, GraphNodeId};

/// Structural graph-operation failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphError {
    /// An edge references a source node that is not registered in the graph.
    SourceNodeNotFound { node_id: GraphNodeId },

    /// An edge references a target node that is not registered in the graph.
    TargetNodeNotFound { node_id: GraphNodeId },

    /// The canonical assertion is already represented by an edge in the graph.
    AssertionAlreadyAssociated {
        /// Canonical assertion identity already present in the graph.
        assertion_id: KnowledgeAssertionId,

        /// Existing structural edge identity.
        edge_id: GraphEdgeId,
    },

    /// The supplied edge identity is already present in the graph.
    EdgeAlreadyExists { edge_id: GraphEdgeId },

    /// A structural node with the same semantic reference is already registered
    /// under a different graph-node identity.
    NodeReferenceAlreadyRegistered {
        /// Semantic reference already associated with a graph node.
        reference: AssertionObject,

        /// Existing graph-node identity for the reference.
        node_id: GraphNodeId,
    },

    /// Edge construction failed its assertion/node structural checks.
    InvalidEdge(GraphEdgeError),
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNodeNotFound { node_id } => {
                write!(formatter, "graph source node not found: {node_id}")
            }
            Self::TargetNodeNotFound { node_id } => {
                write!(formatter, "graph target node not found: {node_id}")
            }
            Self::AssertionAlreadyAssociated {
                assertion_id,
                edge_id,
            } => write!(
                formatter,
                "knowledge assertion {assertion_id} is already associated with graph edge {edge_id}",
            ),
            Self::EdgeAlreadyExists { edge_id } => {
                write!(formatter, "graph edge already exists: {edge_id}")
            }
            Self::NodeReferenceAlreadyRegistered { reference, node_id } => write!(
                formatter,
                "graph node reference is already registered under node {node_id}: {reference:?}"
            ),
            Self::InvalidEdge(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for GraphError {}

/// Basic in-memory Knowledge Graph structure for Phase 2.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    nodes: BTreeMap<GraphNodeId, GraphNode>,
    node_ids_by_reference: BTreeMap<AssertionObject, GraphNodeId>,
    edges: BTreeMap<GraphEdgeId, GraphEdge>,
    edge_ids_by_assertion: BTreeMap<KnowledgeAssertionId, GraphEdgeId>,
    outgoing: BTreeMap<GraphNodeId, BTreeSet<GraphEdgeId>>,
    incoming: BTreeMap<GraphNodeId, BTreeSet<GraphEdgeId>>,
}

impl Graph {
    /// Creates an empty in-memory graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a structural node for a semantic reference.
    ///
    /// If the semantic reference is already represented by a graph node, the
    /// existing node identity is returned instead of creating a duplicate.
    #[must_use]
    pub fn add_node(&mut self, reference: AssertionObject) -> GraphNodeId {
        if let Some(existing) = self.node_ids_by_reference.get(&reference) {
            return existing.clone();
        }

        let node = GraphNode::new(reference.clone());
        let node_id = node.id().clone();

        self.node_ids_by_reference
            .insert(reference, node_id.clone());
        self.nodes.insert(node_id.clone(), node);
        self.outgoing.entry(node_id.clone()).or_default();
        self.incoming.entry(node_id.clone()).or_default();

        node_id
    }

    /// Adds a graph edge for a canonical knowledge assertion.
    ///
    /// The required subject/object nodes are created or reused automatically.
    pub fn add_assertion(
        &mut self,
        assertion: &KnowledgeAssertion,
    ) -> Result<GraphEdgeId, GraphError> {
        if let Some(existing_edge_id) = self.edge_ids_by_assertion.get(assertion.id()) {
            return Err(GraphError::AssertionAlreadyAssociated {
                assertion_id: assertion.id().clone(),
                edge_id: existing_edge_id.clone(),
            });
        }

        let source_id = self.add_node(assertion.subject().clone());
        let target_id = self.add_node(assertion.object().clone());

        let source = self
            .nodes
            .get(&source_id)
            .expect("newly added graph source node must exist");
        let target = self
            .nodes
            .get(&target_id)
            .expect("newly added graph target node must exist");

        let edge = GraphEdge::new(assertion, source, target).map_err(GraphError::InvalidEdge)?;

        let edge_id = edge.id().clone();
        self.insert_edge(edge)?;

        Ok(edge_id)
    }

    /// Registers an already constructed structural graph node without replacing
    /// the node identity.
    ///
    /// This is the registration path for callers that construct `GraphNode`
    /// values themselves and later build a `GraphEdge` from those exact nodes.
    ///
    /// Registering the same node value again is idempotent. Registering a
    /// different node identity for an already registered semantic reference is
    /// rejected so one semantic reference continues to map to one structural
    /// graph node.
    pub fn register_node(&mut self, node: GraphNode) -> Result<(), GraphError> {
        let node_id = node.id().clone();
        let reference = node.reference().clone();

        if let Some(existing) = self.node_ids_by_reference.get(&reference) {
            if existing == &node_id {
                return Ok(());
            }

            return Err(GraphError::NodeReferenceAlreadyRegistered {
                reference,
                node_id: existing.clone(),
            });
        }

        if self.nodes.contains_key(&node_id) {
            return Ok(());
        }

        self.nodes.insert(node_id.clone(), node);
        self.node_ids_by_reference
            .insert(reference, node_id.clone());
        self.outgoing.entry(node_id.clone()).or_default();
        self.incoming.entry(node_id).or_default();

        Ok(())
    }

    /// Adds an already constructed structural graph edge.
    ///
    /// The edge must reference nodes that have already been registered with
    /// [`Graph::register_node`]. This preserves the edge's exact structural node
    /// identities rather than silently replacing them.
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<(), GraphError> {
        self.insert_edge(edge)
    }

    /// Registers the supplied source/target nodes and then adds the already
    /// constructed edge.
    ///
    /// This is the convenience path for callers that construct a
    /// `GraphEdge` from public `GraphNode` values before handing the complete
    /// structure to the graph.
    pub fn add_edge_with_nodes(
        &mut self,
        source: GraphNode,
        target: GraphNode,
        edge: GraphEdge,
    ) -> Result<(), GraphError> {
        self.register_node(source)?;
        self.register_node(target)?;
        self.insert_edge(edge)
    }

    /// Returns a node by structural identity.
    #[must_use]
    pub fn node(&self, node_id: &GraphNodeId) -> Option<&GraphNode> {
        self.nodes.get(node_id)
    }

    /// Returns a node by its typed semantic reference.
    #[must_use]
    pub fn node_for_reference(&self, reference: &AssertionObject) -> Option<&GraphNode> {
        self.node_ids_by_reference
            .get(reference)
            .and_then(|node_id| self.nodes.get(node_id))
    }

    /// Returns an edge by structural identity.
    #[must_use]
    pub fn edge(&self, edge_id: &GraphEdgeId) -> Option<&GraphEdge> {
        self.edges.get(edge_id)
    }

    /// Returns the edge associated with a canonical assertion identity.
    #[must_use]
    pub fn edge_for_assertion(&self, assertion_id: &KnowledgeAssertionId) -> Option<&GraphEdge> {
        self.edge_ids_by_assertion
            .get(assertion_id)
            .and_then(|edge_id| self.edges.get(edge_id))
    }

    /// Returns structural outgoing edges for a node.
    #[must_use]
    pub fn outgoing_edges(&self, node_id: &GraphNodeId) -> Vec<&GraphEdge> {
        self.outgoing
            .get(node_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|edge_id| self.edges.get(edge_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns structural incoming edges for a node.
    #[must_use]
    pub fn incoming_edges(&self, node_id: &GraphNodeId) -> Vec<&GraphEdge> {
        self.incoming
            .get(node_id)
            .map(|edge_ids| {
                edge_ids
                    .iter()
                    .filter_map(|edge_id| self.edges.get(edge_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the number of graph nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of graph edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns whether the graph contains no nodes and no edges.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    fn insert_edge(&mut self, edge: GraphEdge) -> Result<(), GraphError> {
        let source_id = edge.source().clone();
        let target_id = edge.target().clone();
        let edge_id = edge.id().clone();
        let assertion_id = edge.assertion_id().clone();

        if !self.nodes.contains_key(&source_id) {
            return Err(GraphError::SourceNodeNotFound { node_id: source_id });
        }

        if !self.nodes.contains_key(&target_id) {
            return Err(GraphError::TargetNodeNotFound { node_id: target_id });
        }

        if self.edges.contains_key(&edge_id) {
            return Err(GraphError::EdgeAlreadyExists { edge_id });
        }

        if let Some(existing_edge_id) = self.edge_ids_by_assertion.get(&assertion_id) {
            return Err(GraphError::AssertionAlreadyAssociated {
                assertion_id,
                edge_id: existing_edge_id.clone(),
            });
        }

        self.edges.insert(edge_id.clone(), edge);
        self.edge_ids_by_assertion
            .insert(assertion_id, edge_id.clone());
        self.outgoing
            .entry(source_id)
            .or_default()
            .insert(edge_id.clone());
        self.incoming.entry(target_id).or_default().insert(edge_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Graph, GraphError};
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::graph::edge::GraphEdge;
    use crate::graph::node::GraphNode;
    use crate::identity::EntityId;

    fn entity(value: &str) -> EntityId {
        EntityId::new(value).expect("valid entity identity")
    }

    fn assertion(predicate: &str, object: &str) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(entity("entity-1")),
            AssertionPredicate::new(predicate).expect("valid predicate"),
            AssertionObject::Entity(entity(object)),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    #[test]
    fn graph_reuses_the_same_node_for_the_same_semantic_reference() {
        let mut graph = Graph::new();
        let reference = AssertionObject::Entity(entity("entity-1"));

        let first = graph.add_node(reference.clone());
        let second = graph.add_node(reference);

        assert_eq!(first, second);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn graph_adds_assertion_as_one_structural_edge() {
        let mut graph = Graph::new();
        let assertion = assertion("has-name", "entity-2");

        let edge_id = graph
            .add_assertion(&assertion)
            .expect("assertion should become a graph edge");

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.edge(&edge_id).unwrap().assertion_id(), assertion.id());
    }

    #[test]
    fn graph_supports_multiple_distinct_relationships_between_the_same_nodes() {
        let mut graph = Graph::new();
        let first = assertion("has-name", "entity-2");
        let second = assertion("aliases", "entity-2");

        graph
            .add_assertion(&first)
            .expect("first assertion should be added");
        graph
            .add_assertion(&second)
            .expect("second assertion should be added");

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn graph_rejects_duplicate_canonical_assertion_edges() {
        let mut graph = Graph::new();
        let assertion = assertion("has-name", "entity-2");

        graph
            .add_assertion(&assertion)
            .expect("first assertion should be added");

        let error = graph
            .add_assertion(&assertion)
            .expect_err("same canonical assertion must not be duplicated");

        match error {
            GraphError::AssertionAlreadyAssociated { assertion_id, .. } => {
                assert_eq!(assertion_id, *assertion.id())
            }
            other => panic!("unexpected graph error: {other:?}"),
        }
    }

    #[test]
    fn structural_adjacency_is_available_in_both_directions() {
        let mut graph = Graph::new();
        let assertion = assertion("has-name", "entity-2");

        let edge_id = graph
            .add_assertion(&assertion)
            .expect("assertion should be added");

        let source = graph
            .node_for_reference(assertion.subject())
            .expect("source node should exist")
            .id()
            .clone();
        let target = graph
            .node_for_reference(assertion.object())
            .expect("target node should exist")
            .id()
            .clone();

        assert_eq!(graph.outgoing_edges(&source).len(), 1);
        assert_eq!(graph.incoming_edges(&target).len(), 1);
        assert_eq!(graph.outgoing_edges(&source)[0].id(), &edge_id);
        assert_eq!(graph.incoming_edges(&target)[0].id(), &edge_id);
    }

    #[test]
    fn manually_constructed_nodes_can_be_registered_and_used_by_add_edge() {
        let mut graph = Graph::new();
        let assertion = assertion("has-name", "entity-2");
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(assertion.object().clone());
        let edge = GraphEdge::new(&assertion, &source, &target)
            .expect("edge should be structurally valid");

        graph
            .register_node(source.clone())
            .expect("source node should register");
        graph
            .register_node(target.clone())
            .expect("target node should register");

        graph
            .add_edge(edge.clone())
            .expect("registered nodes should allow the constructed edge");

        assert!(graph.node(source.id()).is_some());
        assert!(graph.node(target.id()).is_some());
        assert!(graph.edge(edge.id()).is_some());
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn add_edge_still_rejects_an_edge_with_unregistered_nodes() {
        let mut graph = Graph::new();
        let assertion = assertion("has-name", "entity-2");
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(assertion.object().clone());
        let edge = GraphEdge::new(&assertion, &source, &target)
            .expect("edge should be structurally valid");

        let error = graph
            .add_edge(edge)
            .expect_err("unregistered nodes must still be rejected");

        match error {
            GraphError::SourceNodeNotFound { node_id } => {
                assert_eq!(node_id, *source.id());
            }
            other => panic!("unexpected graph error: {other:?}"),
        }
    }

    #[test]
    fn add_edge_with_nodes_registers_the_supplied_nodes() {
        let mut graph = Graph::new();
        let assertion = assertion("has-name", "entity-2");
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(assertion.object().clone());
        let edge = GraphEdge::new(&assertion, &source, &target)
            .expect("edge should be structurally valid");

        graph
            .add_edge_with_nodes(source.clone(), target.clone(), edge.clone())
            .expect("complete constructed graph structure should be accepted");

        assert_eq!(graph.node(source.id()).unwrap().id(), source.id());
        assert_eq!(graph.node(target.id()).unwrap().id(), target.id());
        assert_eq!(graph.edge(edge.id()).unwrap().id(), edge.id());
    }
}
