//! Canonical/inverse and bounded multi-hop graph traversal for Phase 2 and Phase 6.
//!
//! The one-step traversal primitive preserves the Phase 2 semantic contract.
//! Phase 6 adds explicitly bounded multi-hop traversal and a path result model.
//! Traversal does not infer new assertions, execute reasoning, or own semantic
//! relationship storage. Callers resolve the canonical assertion and relationship
//! definition for each structural edge.

use core::fmt;

use crate::assertion::KnowledgeAssertion;
use crate::relationship::{Relationship, RelationshipDirection, RelationshipPredicate};

use super::edge::GraphEdge;
use super::model::Graph;
use super::node::GraphNodeId;
use super::path::TraversalPath;

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

/// Explicit upper bound for multi-hop traversal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraversalBounds {
    max_depth: usize,
    max_edges: usize,
    max_results: usize,
}

impl TraversalBounds {
    /// Creates an explicit traversal bound.
    ///
    /// A zero depth is valid and returns only the empty path rooted at the
    /// starting node. There is deliberately no unbounded/default constructor.
    #[must_use]
    pub const fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            max_edges: usize::MAX,
            max_results: usize::MAX,
        }
    }

    /// Replaces the maximum number of edge expansions performed by bounded traversal.
    #[must_use]
    pub const fn with_max_edges(mut self, max_edges: usize) -> Self {
        self.max_edges = max_edges;
        self
    }

    /// Replaces the maximum number of paths returned by bounded traversal.
    #[must_use]
    pub const fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }

    /// Returns the maximum traversal depth.
    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// Returns the maximum number of edge expansions.
    #[must_use]
    pub const fn max_edges(self) -> usize {
        self.max_edges
    }

    /// Returns the maximum number of paths returned.
    #[must_use]
    pub const fn max_results(self) -> usize {
        self.max_results
    }
}

/// Structural traversal failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraversalError {
    /// The graph edge does not reference the supplied canonical assertion.
    AssertionMismatch,

    /// The relationship predicate does not match the predicate of the canonical
    /// assertion referenced by the graph edge.
    PredicateMismatch {
        /// Predicate carried by the canonical assertion.
        assertion_predicate: RelationshipPredicate,

        /// Predicate supplied by the relationship definition.
        relationship_predicate: RelationshipPredicate,
    },

    /// A reverse traversal was requested but the relationship provides neither
    /// an explicit inverse predicate nor symmetric semantics.
    ReverseTraversalUnavailable,

    /// The starting graph node is not registered in the graph.
    StartNodeNotFound {
        /// Missing graph-node identity.
        node_id: GraphNodeId,
    },

    /// A bounded traversal resolver could not provide semantic data for an edge.
    ResolutionUnavailable {
        /// Canonical assertion identity referenced by the edge.
        assertion_id: crate::identity::KnowledgeAssertionId,
    },
}

impl fmt::Display for TraversalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssertionMismatch => formatter
                .write_str("graph edge does not reference the supplied canonical assertion"),
            Self::PredicateMismatch {
                assertion_predicate,
                relationship_predicate,
            } => write!(
                formatter,
                "relationship predicate does not match assertion predicate: assertion={assertion_predicate}, relationship={relationship_predicate}"
            ),
            Self::ReverseTraversalUnavailable => {
                formatter.write_str("relationship does not provide reverse traversal semantics")
            }
            Self::StartNodeNotFound { node_id } => {
                write!(formatter, "traversal start node not found: {node_id}")
            }
            Self::ResolutionUnavailable { assertion_id } => {
                write!(
                    formatter,
                    "traversal semantics could not be resolved for assertion {assertion_id}"
                )
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

    if relationship.predicate() != assertion.predicate() {
        return Err(TraversalError::PredicateMismatch {
            assertion_predicate: assertion.predicate().clone(),
            relationship_predicate: relationship.predicate().clone(),
        });
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

/// Resolves semantic data for a structural edge during bounded traversal.
///
/// The graph intentionally stores only structural edge data. A caller supplies
/// the canonical assertion and relationship definition without making the graph
/// a second assertion or relationship store.
pub fn traverse_bounded<F>(
    graph: &Graph,
    start: &GraphNodeId,
    direction: TraversalDirection,
    bounds: TraversalBounds,
    mut resolve: F,
) -> Result<Vec<TraversalPath>, TraversalError>
where
    F: FnMut(&GraphEdge) -> Result<(KnowledgeAssertion, Relationship), TraversalError>,
{
    if graph.node(start).is_none() {
        return Err(TraversalError::StartNodeNotFound {
            node_id: start.clone(),
        });
    }

    let mut results = Vec::new();
    let mut edge_work = 0;
    let root = TraversalPath::new(start.clone());

    visit_paths(
        graph,
        direction,
        bounds.max_depth(),
        bounds.max_edges(),
        bounds.max_results(),
        &mut resolve,
        root,
        &mut results,
        &mut edge_work,
    )?;

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn visit_paths<F>(
    graph: &Graph,
    direction: TraversalDirection,
    max_depth: usize,
    max_edges: usize,
    max_results: usize,
    resolve: &mut F,
    path: TraversalPath,
    results: &mut Vec<TraversalPath>,
    edge_work: &mut usize,
) -> Result<(), TraversalError>
where
    F: FnMut(&GraphEdge) -> Result<(KnowledgeAssertion, Relationship), TraversalError>,
{
    if results.len() >= max_results {
        return Ok(());
    }

    results.push(path.clone());

    if path.len() >= max_depth || results.len() >= max_results || *edge_work >= max_edges {
        return Ok(());
    }

    let current = path.end().clone();

    for edge in graph.incident_edges(&current) {
        if *edge_work >= max_edges || results.len() >= max_results {
            break;
        }
        *edge_work += 1;

        let (assertion, relationship) = resolve(edge)?;
        let step = match traverse(edge, &assertion, &relationship, direction) {
            Ok(step) => step,
            Err(TraversalError::ReverseTraversalUnavailable)
                if direction == TraversalDirection::Inverse =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };

        if step.source() != &current
            || step.target() == path.start()
            || path.steps().iter().any(|previous| {
                previous.source() == step.target() || previous.target() == step.target()
            })
        {
            continue;
        }

        let mut next = path.clone();
        next.push(step)
            .map_err(|_error| TraversalError::ResolutionUnavailable {
                assertion_id: edge.assertion_id().clone(),
            })?;
        visit_paths(
            graph,
            direction,
            max_depth,
            max_edges,
            max_results,
            resolve,
            next,
            results,
            edge_work,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TraversalBounds, TraversalDirection, TraversalError, traverse, traverse_bounded};
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::graph::{Graph, GraphEdge, GraphNode};
    use crate::identity::EntityId;
    use crate::relationship::{
        Relationship, RelationshipCharacteristic, RelationshipCharacteristics,
        RelationshipDirection, RelationshipFamily, RelationshipPredicate,
    };
    use std::collections::BTreeMap;

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

    fn relationship(predicate: &str, direction: RelationshipDirection) -> Relationship {
        Relationship::new(
            RelationshipPredicate::new(predicate).expect("valid predicate"),
            RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid family"),
            direction,
            RelationshipCharacteristics::new(),
        )
    }

    fn edge_for(assertion: &KnowledgeAssertion) -> GraphEdge {
        let source = GraphNode::new(assertion.subject().clone());
        let target = GraphNode::new(assertion.object().clone());

        GraphEdge::new(assertion, &source, &target).expect("valid graph edge")
    }

    fn graph_with_chain() -> (
        Graph,
        BTreeMap<crate::identity::KnowledgeAssertionId, (KnowledgeAssertion, Relationship)>,
    ) {
        let mut graph = Graph::new();
        let first = assertion("entity-1", "entity-2", "first");
        let second = assertion("entity-2", "entity-3", "second");
        let third = assertion("entity-3", "entity-4", "third");

        graph.add_assertion(&first).expect("first edge");
        graph.add_assertion(&second).expect("second edge");
        graph.add_assertion(&third).expect("third edge");

        let mut semantics = BTreeMap::new();
        semantics.insert(
            first.id().clone(),
            (
                first,
                relationship("first", RelationshipDirection::SubjectToObject),
            ),
        );
        semantics.insert(
            second.id().clone(),
            (
                second,
                relationship("second", RelationshipDirection::SubjectToObject),
            ),
        );
        semantics.insert(
            third.id().clone(),
            (
                third,
                relationship("third", RelationshipDirection::SubjectToObject),
            ),
        );

        (graph, semantics)
    }

    #[test]
    fn forward_traversal_follows_subject_to_object_semantics() {
        let assertion = assertion("entity-1", "entity-2", "has-name");
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
        let assertion = assertion("entity-1", "entity-2", "name-of");
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
        let assertion = assertion("entity-1", "entity-2", "has-name");
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
        let assertion = assertion("entity-1", "entity-2", "aliases");
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
        let assertion = assertion("entity-1", "entity-2", "related-to");
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
        let kgassertion = assertion("entity-1", "entity-2", "has-name");
        let other_assertion = assertion("entity-1", "entity-2", "aliases");
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
    fn traversal_rejects_a_relationship_with_a_different_predicate() {
        let assertion = assertion("entity-1", "entity-2", "has-name");
        let edge = edge_for(&assertion);
        let relationship = relationship("aliases", RelationshipDirection::SubjectToObject);

        let error = traverse(
            &edge,
            &assertion,
            &relationship,
            TraversalDirection::Forward,
        )
        .expect_err("relationship/assertion predicate mismatch must be rejected");

        assert_eq!(
            error,
            TraversalError::PredicateMismatch {
                assertion_predicate: RelationshipPredicate::new("has-name")
                    .expect("valid predicate"),
                relationship_predicate: RelationshipPredicate::new("aliases")
                    .expect("valid predicate"),
            }
        );
    }

    #[test]
    fn traversal_direction_is_not_the_relationship_semantic_direction() {
        let assertion = assertion("entity-1", "entity-2", "has-name");
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

    #[test]
    fn bounded_traversal_is_explicitly_depth_limited_and_deterministic_within_a_graph() {
        let (graph, semantics) = graph_with_chain();
        let start = graph
            .node_for_reference(&AssertionObject::Entity(entity("entity-1")))
            .expect("start node")
            .id()
            .clone();

        let results = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Forward,
            TraversalBounds::new(2),
            |edge| {
                semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                    TraversalError::ResolutionUnavailable {
                        assertion_id: edge.assertion_id().clone(),
                    }
                })
            },
        )
        .expect("bounded traversal should succeed");

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].len(), 0);
        assert_eq!(results[1].len(), 1);
        assert_eq!(results[2].len(), 2);
        let entity_3 = graph
            .node_for_reference(&AssertionObject::Entity(entity("entity-3")))
            .expect("entity-3 node")
            .id()
            .clone();
        assert_eq!(results[2].end(), &entity_3);
    }

    #[test]
    fn bounded_traversal_rejects_unknown_start_nodes() {
        let graph = Graph::new();
        let start = GraphNode::new(AssertionObject::Entity(entity("missing")));

        let error = traverse_bounded(
            &graph,
            start.id(),
            TraversalDirection::Forward,
            TraversalBounds::new(1),
            |_edge| {
                Err(TraversalError::ResolutionUnavailable {
                    assertion_id: crate::identity::KnowledgeAssertionId::generate(),
                })
            },
        )
        .expect_err("unknown start must be rejected");

        assert_eq!(
            error,
            TraversalError::StartNodeNotFound {
                node_id: start.id().clone(),
            }
        );
    }

    #[test]
    fn bounded_traversal_respects_result_and_edge_work_limits_when_branches_rejoin() {
        let mut graph = Graph::new();
        let first = assertion("entity-1", "entity-2", "first");
        let second = assertion("entity-1", "entity-3", "second");
        let third = assertion("entity-2", "entity-4", "third");
        let fourth = assertion("entity-3", "entity-4", "fourth");

        for item in [&first, &second, &third, &fourth] {
            graph.add_assertion(item).expect("edge should be added");
        }

        let mut semantics = BTreeMap::new();
        for item in [
            (first, "first"),
            (second, "second"),
            (third, "third"),
            (fourth, "fourth"),
        ] {
            semantics.insert(
                item.0.id().clone(),
                (
                    item.0,
                    relationship(item.1, RelationshipDirection::SubjectToObject),
                ),
            );
        }

        let start = graph
            .node_for_reference(&AssertionObject::Entity(entity("entity-1")))
            .expect("start node")
            .id()
            .clone();

        let result_limited = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Forward,
            TraversalBounds::new(2).with_max_results(3),
            |edge| {
                semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                    TraversalError::ResolutionUnavailable {
                        assertion_id: edge.assertion_id().clone(),
                    }
                })
            },
        )
        .expect("result-limited traversal should succeed");
        assert_eq!(result_limited.len(), 3);

        let edge_work = std::cell::Cell::new(0);
        let edge_limited = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Forward,
            TraversalBounds::new(2).with_max_edges(3),
            |edge| {
                edge_work.set(edge_work.get() + 1);
                semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                    TraversalError::ResolutionUnavailable {
                        assertion_id: edge.assertion_id().clone(),
                    }
                })
            },
        )
        .expect("edge-limited traversal should succeed");
        assert_eq!(edge_work.get(), 3);
        assert!(edge_limited.len() <= 4);
        assert!(edge_limited.iter().all(|path| path.len() <= 2));
    }

    #[test]
    fn inverse_bounded_traversal_skips_non_invertible_edges_but_propagates_other_errors() {
        let mut graph = Graph::new();
        let invertible = assertion("entity-1", "entity-2", "has-name");
        let non_invertible = assertion("entity-3", "entity-2", "related-to");
        graph.add_assertion(&invertible).expect("invertible edge");
        graph
            .add_assertion(&non_invertible)
            .expect("non-invertible edge");

        let invertible_relationship =
            relationship("has-name", RelationshipDirection::SubjectToObject)
                .with_inverse_predicate(
                    RelationshipPredicate::new("name-of").expect("valid inverse predicate"),
                )
                .expect("valid inverse");
        let non_invertible_relationship =
            relationship("related-to", RelationshipDirection::SubjectToObject);

        let mut semantics = BTreeMap::new();
        semantics.insert(
            invertible.id().clone(),
            (invertible.clone(), invertible_relationship),
        );
        semantics.insert(
            non_invertible.id().clone(),
            (non_invertible.clone(), non_invertible_relationship),
        );

        let start = graph
            .node_for_reference(&AssertionObject::Entity(entity("entity-2")))
            .expect("start node")
            .id()
            .clone();
        let results = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Inverse,
            TraversalBounds::new(1),
            |edge| {
                semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                    TraversalError::ResolutionUnavailable {
                        assertion_id: edge.assertion_id().clone(),
                    }
                })
            },
        )
        .expect("non-invertible edge should be skipped");

        assert_eq!(results.len(), 2);
        assert_eq!(results[1].steps()[0].assertion_id(), invertible.id());

        let error = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Inverse,
            TraversalBounds::new(1),
            |edge| {
                if edge.assertion_id() == invertible.id() {
                    Err(TraversalError::AssertionMismatch)
                } else {
                    semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                        TraversalError::ResolutionUnavailable {
                            assertion_id: edge.assertion_id().clone(),
                        }
                    })
                }
            },
        )
        .expect_err("assertion mismatch must propagate");
        assert_eq!(error, TraversalError::AssertionMismatch);

        let error = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Inverse,
            TraversalBounds::new(1),
            |edge| {
                if edge.assertion_id() == invertible.id() {
                    Ok((
                        invertible.clone(),
                        relationship("other-predicate", RelationshipDirection::SubjectToObject),
                    ))
                } else {
                    semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                        TraversalError::ResolutionUnavailable {
                            assertion_id: edge.assertion_id().clone(),
                        }
                    })
                }
            },
        )
        .expect_err("predicate mismatch must propagate");
        assert!(matches!(error, TraversalError::PredicateMismatch { .. }));
    }

    #[test]
    fn bounded_traversal_does_not_revisit_a_node_on_the_same_path() {
        let (mut graph, mut semantics) = graph_with_chain();
        let cycle = assertion("entity-4", "entity-2", "cycle");
        graph.add_assertion(&cycle).expect("cycle edge");
        semantics.insert(
            cycle.id().clone(),
            (
                cycle,
                relationship("cycle", RelationshipDirection::SubjectToObject),
            ),
        );

        let start = graph
            .node_for_reference(&AssertionObject::Entity(entity("entity-1")))
            .expect("start node")
            .id()
            .clone();

        let results = traverse_bounded(
            &graph,
            &start,
            TraversalDirection::Forward,
            TraversalBounds::new(5),
            |edge| {
                semantics.get(edge.assertion_id()).cloned().ok_or_else(|| {
                    TraversalError::ResolutionUnavailable {
                        assertion_id: edge.assertion_id().clone(),
                    }
                })
            },
        )
        .expect("cycle should be handled");

        assert!(results.iter().all(|path| path.len() <= 4));
        assert!(results.iter().all(|path| {
            let mut nodes = std::collections::BTreeSet::new();
            nodes.insert(path.start().clone());
            path.steps()
                .iter()
                .all(|step| nodes.insert(step.target().clone()))
        }));
    }
}
