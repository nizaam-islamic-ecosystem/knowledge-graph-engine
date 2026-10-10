//! Traversal path value objects for Phase 6.
//!
//! A `TraversalPath` is an ordered structural/semantic traversal result. It
//! stores the one-step traversal views produced by `traverse` and keeps path
//! construction separate from traversal execution.
//!
//! Paths have no generated identity. Their value is completely determined by
//! their starting node and ordered traversal steps.

use core::fmt;

use super::node::GraphNodeId;
use super::traversal::TraversalStep;

/// Structural failures while constructing a traversal path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathError {
    /// The next step does not begin at the current path endpoint.
    Disconnected {
        /// Current endpoint of the path.
        expected: GraphNodeId,

        /// Source node supplied by the next traversal step.
        actual: GraphNodeId,
    },
}

impl fmt::Display for PathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected { expected, actual } => write!(
                formatter,
                "traversal path is disconnected: expected source {expected}, got {actual}",
            ),
        }
    }
}

impl std::error::Error for PathError {}

/// An ordered traversal path starting at one graph node.
///
/// An empty path is valid and represents the starting node itself. A non-empty
/// path contains contiguous [`TraversalStep`] values whose endpoints form one
/// connected sequence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraversalPath {
    start: GraphNodeId,
    steps: Vec<TraversalStep>,
}

impl TraversalPath {
    /// Creates an empty path rooted at the supplied graph node.
    #[must_use]
    pub fn new(start: GraphNodeId) -> Self {
        Self {
            start,
            steps: Vec::new(),
        }
    }

    /// Creates a path from an ordered sequence of traversal steps.
    ///
    /// The first step must start at `start`, and every later step must start at
    /// the preceding step's target.
    pub fn from_steps(
        start: GraphNodeId,
        steps: impl IntoIterator<Item = TraversalStep>,
    ) -> Result<Self, PathError> {
        let mut path = Self::new(start);

        for step in steps {
            path.push(step)?;
        }

        Ok(path)
    }

    /// Appends one traversal step after validating endpoint continuity.
    pub fn push(&mut self, step: TraversalStep) -> Result<(), PathError> {
        let expected = self.end().clone();

        if step.source() != &expected {
            return Err(PathError::Disconnected {
                expected,
                actual: step.source().clone(),
            });
        }

        self.steps.push(step);
        Ok(())
    }

    /// Returns the starting node of the path.
    #[must_use]
    pub fn start(&self) -> &GraphNodeId {
        &self.start
    }

    /// Returns the current endpoint of the path.
    #[must_use]
    pub fn end(&self) -> &GraphNodeId {
        self.steps.last().map_or(&self.start, TraversalStep::target)
    }

    /// Returns the ordered traversal steps.
    #[must_use]
    pub fn steps(&self) -> &[TraversalStep] {
        &self.steps
    }

    /// Returns the number of traversed edges.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Returns whether the path contains no traversal steps.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{PathError, TraversalPath};
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::graph::{GraphEdge, GraphNode, TraversalDirection, traverse};
    use crate::identity::EntityId;
    use crate::relationship::{
        Relationship, RelationshipCharacteristics, RelationshipDirection, RelationshipFamily,
        RelationshipPredicate,
    };

    fn entity(value: &str) -> EntityId {
        EntityId::new(value).expect("valid entity identity")
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

    fn step(subject: &str, object: &str, predicate: &str) -> crate::graph::TraversalStep {
        let assertion = assertion(subject, object, predicate);
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(assertion.object().clone());
        let edge = GraphEdge::new(&assertion, &source, &target).expect("valid graph edge");
        let relationship = Relationship::new(
            RelationshipPredicate::new(predicate).expect("valid predicate"),
            RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Forward,
        )
        .expect("valid traversal step")
    }

    fn contiguous_steps() -> (crate::graph::TraversalStep, crate::graph::TraversalStep) {
        let first_assertion = assertion("entity-1", "entity-2", "first");
        let second_assertion = assertion("entity-2", "entity-3", "second");

        let source = GraphNode::new(first_assertion.subject().clone());
        let middle = GraphNode::new(first_assertion.object().clone());
        let target = GraphNode::new(second_assertion.object().clone());

        let first_edge =
            GraphEdge::new(&first_assertion, &source, &middle).expect("valid first graph edge");
        let second_edge =
            GraphEdge::new(&second_assertion, &middle, &target).expect("valid second graph edge");

        let first_relationship = Relationship::new(
            RelationshipPredicate::new("first").expect("valid predicate"),
            RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );
        let second_relationship = Relationship::new(
            RelationshipPredicate::new("second").expect("valid predicate"),
            RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::new(),
        );

        let first = traverse(
            &first_edge,
            &first_assertion,
            &first_relationship,
            TraversalDirection::Forward,
        )
        .expect("valid first traversal step");
        let second = traverse(
            &second_edge,
            &second_assertion,
            &second_relationship,
            TraversalDirection::Forward,
        )
        .expect("valid second traversal step");

        (first, second)
    }

    #[test]
    fn empty_path_ends_at_its_start() {
        let node = GraphNode::new(AssertionObject::Entity(entity("entity-1")));

        let path = TraversalPath::new(node.id().clone());

        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
        assert_eq!(path.start(), node.id());
        assert_eq!(path.end(), node.id());
    }

    #[test]
    fn path_accepts_contiguous_steps() {
        let (first, second) = contiguous_steps();

        let path =
            TraversalPath::from_steps(first.source().clone(), [first.clone(), second.clone()])
                .expect("contiguous steps should form a path");

        assert_eq!(path.len(), 2);
        assert_eq!(path.start(), first.source());
        assert_eq!(path.end(), second.target());
        assert_eq!(path.steps(), &[first, second]);
    }

    #[test]
    fn path_rejects_disconnected_steps() {
        let first = step("entity-1", "entity-2", "first");
        let disconnected = step("entity-9", "entity-10", "second");

        let expected = first.target().clone();
        let error =
            TraversalPath::from_steps(first.source().clone(), [first, disconnected.clone()])
                .expect_err("disconnected steps must be rejected");

        assert_eq!(
            error,
            PathError::Disconnected {
                expected,
                actual: disconnected.source().clone(),
            }
        );
    }
}
