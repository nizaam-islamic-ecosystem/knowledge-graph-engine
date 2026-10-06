//! Minimal canonical/inverse graph traversal primitives for Phase 2.
//!
//! This module provides one-step structural traversal views only. It does not
//! implement multi-hop traversal, path execution, filtering, ranking, query
//! planning, or reasoning.
//!
//! Traversal direction is intentionally separate from relationship semantic
//! direction.

use core::fmt;

use crate::assertion::KnowledgeAssertion;
use crate::relationship::{Relationship, RelationshipDirection, RelationshipPredicate};

use super::edge::GraphEdge;
use super::node::GraphNodeId;

/// Direction in which a graph edge is being traversed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TraversalDirection {
    /// Traverse using the relationship's canonical semantic orientation.
    Forward,

    /// Traverse the structural reverse of the relationship.
    Inverse,
}

impl TraversalDirection {
    /// Returns the stable textual representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Inverse => "inverse",
        }
    }
}

impl fmt::Display for TraversalDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Structural traversal failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraversalError {
    /// The graph edge does not reference the supplied canonical assertion.
    AssertionMismatch,

    /// A reverse traversal was requested but the relationship provides neither
    /// an explicit inverse predicate nor symmetric semantics.
    ReverseTraversalUnavailable,
}

impl fmt::Display for TraversalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssertionMismatch => formatter
                .write_str("graph edge does not reference the supplied canonical assertion"),
            Self::ReverseTraversalUnavailable => {
                formatter.write_str("relationship does not provide reverse traversal semantics")
            }
        }
    }
}

impl std::error::Error for TraversalError {}

/// A one-step structural traversal view over a canonical graph edge.
///
/// The traversal view may reverse the displayed endpoints without changing the
/// relationship's stored semantic direction.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraversalStep {
    edge_id: super::edge::GraphEdgeId,
    assertion_id: crate::identity::KnowledgeAssertionId,
    source: GraphNodeId,
    target: GraphNodeId,
    predicate: RelationshipPredicate,
    semantic_direction: RelationshipDirection,
    traversal_direction: TraversalDirection,
}

impl TraversalStep {
    /// Returns the graph edge being traversed.
    #[must_use]
    pub fn edge_id(&self) -> &super::edge::GraphEdgeId {
        &self.edge_id
    }

    /// Returns the canonical assertion identity.
    #[must_use]
    pub fn assertion_id(&self) -> &crate::identity::KnowledgeAssertionId {
        &self.assertion_id
    }

    /// Returns the traversal-view source node.
    #[must_use]
    pub fn source(&self) -> &GraphNodeId {
        &self.source
    }

    /// Returns the traversal-view target node.
    #[must_use]
    pub fn target(&self) -> &GraphNodeId {
        &self.target
    }

    /// Returns the predicate visible from this traversal view.
    #[must_use]
    pub fn predicate(&self) -> &RelationshipPredicate {
        &self.predicate
    }

    /// Returns the stored semantic direction of the relationship definition.
    #[must_use]
    pub fn semantic_direction(&self) -> RelationshipDirection {
        self.semantic_direction
    }

    /// Returns the direction used for this traversal view.
    #[must_use]
    pub fn traversal_direction(&self) -> TraversalDirection {
        self.traversal_direction
    }
}

/// Traverses one canonical graph edge using the requested structural view.
///
/// Forward traversal follows the relationship's semantic direction. Inverse
/// traversal reverses the displayed endpoints and uses an explicit inverse
/// predicate when one is declared; otherwise a symmetric relationship uses
/// the same predicate.
pub fn traverse(
    edge: &GraphEdge,
    assertion: &KnowledgeAssertion,
    relationship: &Relationship,
    direction: TraversalDirection,
) -> Result<TraversalStep, TraversalError> {
    if edge.assertion_id() != assertion.id() {
        return Err(TraversalError::AssertionMismatch);
    }

    let (canonical_source, canonical_target) = match relationship.direction() {
        RelationshipDirection::SubjectToObject => (edge.source().clone(), edge.target().clone()),
        RelationshipDirection::ObjectToSubject => (edge.target().clone(), edge.source().clone()),
    };

    let (source, target, predicate) = match direction {
        TraversalDirection::Forward => (
            canonical_source,
            canonical_target,
            relationship.predicate().clone(),
        ),
        TraversalDirection::Inverse => {
            let predicate = relationship
                .inverse()
                .map(|inverse| inverse.inverse_predicate().clone())
                .or_else(|| {
                    relationship
                        .is_symmetric()
                        .then(|| relationship.predicate().clone())
                })
                .ok_or(TraversalError::ReverseTraversalUnavailable)?;

            (canonical_target, canonical_source, predicate)
        }
    };

    Ok(TraversalStep {
        edge_id: edge.id().clone(),
        assertion_id: edge.assertion_id().clone(),
        source,
        target,
        predicate,
        semantic_direction: relationship.direction(),
        traversal_direction: direction,
    })
}

#[cfg(test)]
mod tests {
    use super::{TraversalDirection, TraversalError, traverse};
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::graph::GraphEdge;
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

    fn relationship(predicate: &str, direction: RelationshipDirection) -> Relationship {
        Relationship::new(
            RelationshipPredicate::new(predicate).expect("valid predicate"),
            RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
            direction,
            RelationshipCharacteristics::new(),
        )
    }

    fn edge_for(assertion: &KnowledgeAssertion) -> GraphEdge {
        let source = crate::graph::GraphNode::new(assertion.subject().clone());
        let target = crate::graph::GraphNode::new(assertion.object().clone());

        GraphEdge::new(assertion, &source, &target).expect("valid graph edge")
    }

    #[test]
    fn forward_traversal_follows_subject_to_object_semantics() {
        let assertion = assertion("has-name");
        let edge = edge_for(&assertion);
        let relationship = relationship("has-name", RelationshipDirection::SubjectToObject);

        let step = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Forward,
        )
        .expect("forward traversal should succeed");

        assert_eq!(step.source(), edge.source());
        assert_eq!(step.target(), edge.target());
        assert_eq!(step.predicate().as_str(), "kg.relationship.has-name");
        assert_eq!(
            step.semantic_direction(),
            RelationshipDirection::SubjectToObject
        );
    }

    #[test]
    fn forward_traversal_respects_object_to_subject_semantics() {
        let assertion = assertion("name-of");
        let edge = edge_for(&assertion);
        let relationship = relationship("name-of", RelationshipDirection::ObjectToSubject);

        let step = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Forward,
        )
        .expect("forward traversal should succeed");

        assert_eq!(step.source(), edge.target());
        assert_eq!(step.target(), edge.source());
        assert_eq!(
            step.semantic_direction(),
            RelationshipDirection::ObjectToSubject
        );
    }

    #[test]
    fn inverse_traversal_uses_the_declared_inverse_without_new_assertion() {
        let assertion = assertion("has-name");
        let edge = edge_for(&assertion);

        let relationship = relationship("has-name", RelationshipDirection::SubjectToObject)
            .with_inverse_predicate(
                RelationshipPredicate::new("name-of").expect("valid inverse predicate"),
            )
            .expect("valid inverse");

        let step = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Inverse,
        )
        .expect("inverse traversal should succeed");

        assert_eq!(step.source(), edge.target());
        assert_eq!(step.target(), edge.source());
        assert_eq!(step.predicate().as_str(), "kg.relationship.name-of");
        assert_eq!(step.assertion_id(), assertion.id());
        assert_eq!(
            step.semantic_direction(),
            RelationshipDirection::SubjectToObject
        );
    }

    #[test]
    fn symmetric_reverse_traversal_reuses_the_same_predicate() {
        let assertion = assertion("aliases");
        let edge = edge_for(&assertion);

        let relationship = Relationship::new(
            RelationshipPredicate::new("aliases").expect("valid predicate"),
            RelationshipFamily::new(RelationshipFamily::IDENTITY).expect("valid family"),
            RelationshipDirection::SubjectToObject,
            RelationshipCharacteristics::from_iter([RelationshipCharacteristic::Symmetric])
                .expect("valid characteristics"),
        );

        let step = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Inverse,
        )
        .expect("symmetric reverse traversal should succeed");

        assert_eq!(step.source(), edge.target());
        assert_eq!(step.target(), edge.source());
        assert_eq!(step.predicate().as_str(), "kg.relationship.aliases");
    }

    #[test]
    fn reverse_traversal_without_inverse_or_symmetry_is_rejected() {
        let assertion = assertion("related-to");
        let edge = edge_for(&assertion);
        let relationship = relationship("related-to", RelationshipDirection::SubjectToObject);

        let error = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Inverse,
        )
        .expect_err("reverse traversal must require declared reverse semantics");

        assert_eq!(error, TraversalError::ReverseTraversalUnavailable);
    }

    #[test]
    fn traversal_rejects_an_unrelated_assertion() {
        let kgassertion = assertion("has-name");
        let other_assertion = assertion("aliases");
        let edge = edge_for(&kgassertion);
        let relationship = relationship("has-name", RelationshipDirection::SubjectToObject);

        let error = traverse(
            &edge,
            &other_assertion,
            &relationship,
            TraversalDirection::Forward,
        )
        .expect_err("edge/assertion mismatch must be rejected");

        assert_eq!(error, TraversalError::AssertionMismatch);
    }

    #[test]
    fn traversal_direction_is_not_the_relationship_semantic_direction() {
        let assertion = assertion("has-name");
        let edge = edge_for(&assertion);
        let relationship = relationship("has-name", RelationshipDirection::SubjectToObject)
            .with_inverse_predicate(
                RelationshipPredicate::new("name-of").expect("valid inverse predicate"),
            )
            .expect("valid inverse");

        let inverse = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Inverse,
        )
        .expect("inverse traversal should succeed");

        assert_eq!(inverse.traversal_direction(), TraversalDirection::Inverse);
        assert_eq!(
            inverse.semantic_direction(),
            RelationshipDirection::SubjectToObject
        );
    }
}
