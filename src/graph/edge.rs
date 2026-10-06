//! Structural graph edges for Phase 2.
//!
//! A `GraphEdge` connects two structural graph nodes and references one
//! canonical `KnowledgeAssertion` by identity. The edge does not duplicate
//! the semantic assertion itself.

use core::fmt;

use nizaam_core::identity;

use crate::assertion::{AssertionObject, KnowledgeAssertion};
use crate::identity::KnowledgeAssertionId;

use super::node::{GraphNode, GraphNodeId};

identity!(
    /// Identifies an edge within the in-memory Knowledge Graph structure.
    GraphEdgeId
);

/// Structural validation failures when creating a graph edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphEdgeError {
    /// The source graph node does not represent the assertion subject.
    SourceReferenceMismatch {
        /// Reference required by the canonical assertion.
        expected: AssertionObject,

        /// Reference represented by the supplied graph node.
        actual: AssertionObject,
    },

    /// The target graph node does not represent the assertion object.
    TargetReferenceMismatch {
        /// Reference required by the canonical assertion.
        expected: AssertionObject,

        /// Reference represented by the supplied graph node.
        actual: AssertionObject,
    },
}

impl fmt::Display for GraphEdgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceReferenceMismatch { expected, actual } => write!(
                formatter,
                "graph edge source reference does not match assertion subject: expected={expected:?}, actual={actual:?}",
            ),
            Self::TargetReferenceMismatch { expected, actual } => write!(
                formatter,
                "graph edge target reference does not match assertion object: expected={expected:?}, actual={actual:?}",
            ),
        }
    }
}

impl std::error::Error for GraphEdgeError {}

/// A structural graph edge associated with one canonical knowledge assertion.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GraphEdge {
    id: GraphEdgeId,
    assertion_id: KnowledgeAssertionId,
    source: GraphNodeId,
    target: GraphNodeId,
}

impl GraphEdge {
    /// Creates a structural edge for a canonical assertion and its graph
    /// source/target nodes.
    ///
    /// The source must represent the assertion subject and the target must
    /// represent the assertion object. Semantic relationship direction is
    /// handled separately by the traversal layer.
    pub fn new(
        assertion: &KnowledgeAssertion,
        source: &GraphNode,
        target: &GraphNode,
    ) -> Result<Self, GraphEdgeError> {
        if source.reference() != assertion.subject() {
            return Err(GraphEdgeError::SourceReferenceMismatch {
                expected: assertion.subject().clone(),
                actual: source.reference().clone(),
            });
        }

        if target.reference() != assertion.object() {
            return Err(GraphEdgeError::TargetReferenceMismatch {
                expected: assertion.object().clone(),
                actual: target.reference().clone(),
            });
        }

        Ok(Self {
            id: GraphEdgeId::generate(),
            assertion_id: assertion.id().clone(),
            source: source.id().clone(),
            target: target.id().clone(),
        })
    }

    /// Returns the structural graph-edge identity.
    #[must_use]
    pub fn id(&self) -> &GraphEdgeId {
        &self.id
    }

    /// Returns the canonical assertion identity referenced by this edge.
    #[must_use]
    pub fn assertion_id(&self) -> &KnowledgeAssertionId {
        &self.assertion_id
    }

    /// Returns the structural source node identity.
    #[must_use]
    pub fn source(&self) -> &GraphNodeId {
        &self.source
    }

    /// Returns the structural target node identity.
    #[must_use]
    pub fn target(&self) -> &GraphNodeId {
        &self.target
    }
}

#[cfg(test)]
mod tests {
    use super::{GraphEdge, GraphEdgeError};
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::graph::node::GraphNode;
    use crate::identity::EntityId;

    fn entity(value: &str) -> EntityId {
        EntityId::new(value).expect("valid entity identity")
    }

    fn assertion(subject: &str, object: &str) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(entity(subject)),
            AssertionPredicate::new("related-to").expect("valid predicate"),
            AssertionObject::Entity(entity(object)),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    #[test]
    fn creates_an_edge_reference_to_the_canonical_assertion() {
        let assertion = assertion("entity-1", "entity-2");
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(assertion.object().clone());

        let edge = GraphEdge::new(&assertion, &source, &target)
            .expect("matching graph nodes should produce a valid edge");

        assert!(!edge.id().as_str().is_empty());
        assert_eq!(edge.assertion_id(), assertion.id());
        assert_eq!(edge.source(), source.id());
        assert_eq!(edge.target(), target.id());
    }

    #[test]
    fn rejects_a_source_that_does_not_match_the_assertion_subject() {
        let assertion = assertion("entity-1", "entity-2");
        let source = GraphNode::new(AssertionObject::Entity(entity("other")));
        let target = GraphNode::new(assertion.object().clone());

        let error = GraphEdge::new(&assertion, &source, &target)
            .expect_err("mismatched source must be rejected");

        assert_eq!(
            error,
            GraphEdgeError::SourceReferenceMismatch {
                expected: assertion.subject().clone(),
                actual: source.reference().clone(),
            }
        );
    }

    #[test]
    fn rejects_a_target_that_does_not_match_the_assertion_object() {
        let assertion = assertion("entity-1", "entity-2");
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(AssertionObject::Entity(entity("other")));

        let error = GraphEdge::new(&assertion, &source, &target)
            .expect_err("mismatched target must be rejected");

        assert_eq!(
            error,
            GraphEdgeError::TargetReferenceMismatch {
                expected: assertion.object().clone(),
                actual: target.reference().clone(),
            }
        );
    }
}
