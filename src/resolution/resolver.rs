//! Phase 3 deterministic entity-resolution pipeline and circular-evidence
//! dependency protection.
//!
//! The resolver composes normalization/candidate generation and disambiguation
//! without owning persistence, entity merging, graph traversal, evidence
//! provenance, or IndexAssignedId generation.

use core::fmt;
use std::collections::BTreeSet;

use crate::identity::{EntityId, KnowledgeAssertionId};

use super::candidate::{
    Candidate, EntityCandidateProfile, ResolutionInput, ResolutionReference, generate_candidates,
};
use super::disambiguation::{ResolutionDecision, ResolutionPolicy, ResolutionState, decide};

/// A lightweight dependency token for one resolution hypothesis.
///
/// It is intentionally not a Core identity and is not persisted. It only
/// identifies a resolution attempt within the dependency-protection boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolutionDependencyToken(u64);

impl ResolutionDependencyToken {
    /// Creates a deterministic token from a caller-owned cycle identifier.
    ///
    /// The resolver does not generate or persist a semantic identity here. The
    /// value is only a local dependency marker.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the opaque dependency marker value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Minimal dependency relation between a resolution hypothesis and graph
/// evidence used during that same attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEvidenceDependency {
    assertion_id: KnowledgeAssertionId,
    depends_on: BTreeSet<ResolutionDependencyToken>,
}

impl GraphEvidenceDependency {
    /// Creates a graph-evidence dependency record.
    #[must_use]
    pub fn new(
        assertion_id: KnowledgeAssertionId,
        depends_on: impl IntoIterator<Item = ResolutionDependencyToken>,
    ) -> Self {
        Self {
            assertion_id,
            depends_on: depends_on.into_iter().collect(),
        }
    }

    /// Returns the graph assertion represented by this dependency record.
    #[must_use]
    pub fn assertion_id(&self) -> &KnowledgeAssertionId {
        &self.assertion_id
    }

    /// Returns the resolution hypotheses on which this graph evidence depends.
    #[must_use]
    pub fn depends_on(&self) -> &BTreeSet<ResolutionDependencyToken> {
        &self.depends_on
    }
}

/// Lightweight dependency guard used to prevent circular graph evidence from
/// being counted as independent support.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CircularEvidenceGuard {
    active_hypotheses: BTreeSet<ResolutionDependencyToken>,
}

impl CircularEvidenceGuard {
    /// Creates an empty guard.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an active resolution hypothesis.
    pub fn activate(&mut self, token: ResolutionDependencyToken) {
        self.active_hypotheses.insert(token);
    }

    /// Removes an active hypothesis.
    pub fn deactivate(&mut self, token: ResolutionDependencyToken) {
        self.active_hypotheses.remove(&token);
    }

    /// Returns whether graph evidence may count independently.
    ///
    /// Evidence depending on any active hypothesis is not independent and must
    /// therefore be excluded from candidate support for that same cycle.
    #[must_use]
    pub fn allows_independent_evidence(&self, dependency: &GraphEvidenceDependency) -> bool {
        self.active_hypotheses.is_disjoint(dependency.depends_on())
    }
}

/// Errors produced by the resolver boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionError {
    /// A graph-evidence dependency was rejected because it participates in the
    /// same active resolution cycle.
    CircularEvidence { assertion_id: KnowledgeAssertionId },
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircularEvidence { assertion_id } => write!(
                formatter,
                "graph evidence is circular for active resolution hypothesis: assertion={assertion_id}"
            ),
        }
    }
}

impl std::error::Error for ResolutionError {}

/// The result of one deterministic resolution attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionResult {
    reference: ResolutionReference,
    decision: ResolutionDecision,
    candidates: Vec<Candidate>,
}

impl ResolutionResult {
    /// Returns the source-level reference being resolved.
    #[must_use]
    pub fn reference(&self) -> &ResolutionReference {
        &self.reference
    }

    /// Returns the minimal resolution decision.
    #[must_use]
    pub fn decision(&self) -> &ResolutionDecision {
        &self.decision
    }

    /// Returns the ephemeral candidates considered by the decision.
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// Returns the selected canonical entity if the decision is resolved.
    #[must_use]
    pub fn resolved_entity(&self) -> Option<&EntityId> {
        if self.decision.state() == ResolutionState::Resolved {
            self.decision.entity_id()
        } else {
            None
        }
    }
}

/// Stateless deterministic resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolver {
    policy: ResolutionPolicy,
}

impl Resolver {
    /// Creates a resolver with the supplied deterministic policy.
    #[must_use]
    pub fn new(policy: ResolutionPolicy) -> Self {
        Self { policy }
    }

    /// Creates a resolver with the Phase 3 default policy.
    #[must_use]
    pub fn phase3_default() -> Self {
        Self::new(ResolutionPolicy::default())
    }

    /// Returns the active resolution policy.
    #[must_use]
    pub fn policy(&self) -> &ResolutionPolicy {
        &self.policy
    }

    /// Runs deterministic candidate generation and disambiguation.
    ///
    /// The supplied profiles are caller-owned inputs. The resolver does not
    /// persist, merge, create, or mutate entities.
    #[must_use]
    pub fn resolve(
        &self,
        input: &ResolutionInput,
        profiles: &[EntityCandidateProfile],
    ) -> ResolutionResult {
        let candidates = generate_candidates(input, profiles);
        let decision = decide(&candidates, &self.policy);

        ResolutionResult {
            reference: input.reference().clone(),
            decision,
            candidates,
        }
    }

    /// Accepts graph evidence only when its dependency set is independent from
    /// the currently active resolution hypotheses.
    pub fn validate_graph_evidence(
        &self,
        guard: &CircularEvidenceGuard,
        dependency: &GraphEvidenceDependency,
    ) -> Result<(), ResolutionError> {
        if guard.allows_independent_evidence(dependency) {
            Ok(())
        } else {
            Err(ResolutionError::CircularEvidence {
                assertion_id: dependency.assertion_id().clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::candidate::{EntityCandidateProfile, ResolutionInput, ResolutionReference};
    use super::{
        CircularEvidenceGuard, GraphEvidenceDependency, ResolutionDependencyToken, ResolutionError,
        Resolver,
    };
    use crate::identity::{EntityId, KnowledgeAssertionId, MentionId};

    #[test]
    fn resolver_returns_a_resolved_canonical_entity_without_creating_one() {
        let resolver = Resolver::phase3_default();
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            "Muhammad",
        );
        let profile = EntityCandidateProfile::new(EntityId::new("entity-1").expect("valid entity"))
            .with_alias("Muhammad");

        let result = resolver.resolve(&input, &[profile]);

        assert_eq!(
            result.resolved_entity().expect("resolved entity").as_str(),
            "entity-1"
        );
        assert_eq!(result.candidates().len(), 1);
    }

    #[test]
    fn resolver_does_not_resolve_weak_context_only_candidates_by_default() {
        let resolver = Resolver::phase3_default();
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            "unknown",
        )
        .with_context_key("quran");
        let profile = EntityCandidateProfile::new(EntityId::new("entity-1").expect("valid entity"))
            .with_context_key("quran");

        let result = resolver.resolve(&input, &[profile]);

        assert_eq!(result.decision().state().as_str(), "unresolved");
        assert!(result.resolved_entity().is_none());
    }

    #[test]
    fn circular_graph_evidence_is_rejected() {
        let resolver = Resolver::phase3_default();
        let token = ResolutionDependencyToken::new(7);
        let assertion = KnowledgeAssertionId::new("assertion-1").expect("valid assertion");
        let dependency = GraphEvidenceDependency::new(assertion.clone(), [token]);
        let mut guard = CircularEvidenceGuard::new();
        guard.activate(token);

        let error = resolver
            .validate_graph_evidence(&guard, &dependency)
            .expect_err("same-cycle evidence must be rejected");

        assert_eq!(
            error,
            ResolutionError::CircularEvidence {
                assertion_id: assertion
            }
        );
    }

    #[test]
    fn independent_graph_evidence_is_allowed() {
        let resolver = Resolver::phase3_default();
        let active = ResolutionDependencyToken::new(7);
        let independent = ResolutionDependencyToken::new(8);
        let assertion = KnowledgeAssertionId::new("assertion-1").expect("valid assertion");
        let dependency = GraphEvidenceDependency::new(assertion, [independent]);
        let mut guard = CircularEvidenceGuard::new();
        guard.activate(active);

        assert!(
            resolver
                .validate_graph_evidence(&guard, &dependency)
                .is_ok()
        );
    }

    #[test]
    fn deactivated_hypothesis_allows_previous_dependency_to_be_reused() {
        let resolver = Resolver::phase3_default();
        let token = ResolutionDependencyToken::new(7);
        let assertion = KnowledgeAssertionId::new("assertion-1").expect("valid assertion");
        let dependency = GraphEvidenceDependency::new(assertion, [token]);
        let mut guard = CircularEvidenceGuard::new();
        guard.activate(token);
        guard.deactivate(token);

        assert!(
            resolver
                .validate_graph_evidence(&guard, &dependency)
                .is_ok()
        );
    }
}
