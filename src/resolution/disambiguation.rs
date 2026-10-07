//! Deterministic candidate ranking, threshold/margin decisions, and resolution
//! state transitions for Phase 3.
//!
//! The initial policy uses ordered deterministic signal priorities rather than
//! weighted numerical scoring. The numeric values exposed by `CandidateSignal`
//! are ordinal ranks only. This leaves a clean path toward configurable
//! scoring in a later phase without prematurely freezing weights.

use core::fmt;

use crate::identity::EntityId;

use super::candidate::{Candidate, CandidateSignal};

/// The six explicit Phase 3 resolution states.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResolutionState {
    /// The reference has been associated with a canonical identity.
    Resolved,
    /// Multiple candidates remain sufficiently plausible.
    Ambiguous,
    /// An attempt occurred but no acceptable candidate was established.
    Unresolved,
    /// There is not enough information to determine whether a known identity exists.
    Unknown,
    /// A useful working hypothesis that must not become canonical identity.
    Provisional,
    /// A candidate or decision has been explicitly rejected.
    Rejected,
}

impl ResolutionState {
    /// Returns the stable textual representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Ambiguous => "ambiguous",
            Self::Unresolved => "unresolved",
            Self::Unknown => "unknown",
            Self::Provisional => "provisional",
            Self::Rejected => "rejected",
        }
    }
}

impl fmt::Display for ResolutionState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Initial deterministic ranking configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankingPolicy {
    order: Vec<CandidateSignal>,
}

impl RankingPolicy {
    /// Returns the frozen Phase 3 deterministic ordering.
    #[must_use]
    pub fn phase3_default() -> Self {
        Self {
            order: vec![
                CandidateSignal::ExactIdentifier,
                CandidateSignal::ExactAlias,
                CandidateSignal::SourceIdentifier,
                CandidateSignal::Normalized,
                CandidateSignal::Transliteration,
                CandidateSignal::LexicalMapping,
                CandidateSignal::Language,
                CandidateSignal::SemanticRelationship,
                CandidateSignal::Context,
                CandidateSignal::GraphNeighborhood,
            ],
        }
    }

    /// Returns the configured signal order.
    #[must_use]
    pub fn order(&self) -> &[CandidateSignal] {
        &self.order
    }
}

impl Default for RankingPolicy {
    fn default() -> Self {
        Self::phase3_default()
    }
}

/// The deterministic Phase 3 acceptance policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionPolicy {
    threshold: u8,
    margin: u8,
    ranking: RankingPolicy,
}

impl ResolutionPolicy {
    /// Creates a policy from an ordinal minimum signal priority and margin.
    ///
    /// `threshold` is an ordinal signal rank, not a weighted score. A margin
    /// is measured in ordinal rank steps between the strongest candidates.
    #[must_use]
    pub fn new(threshold: u8, margin: u8) -> Self {
        Self {
            threshold,
            margin,
            ranking: RankingPolicy::default(),
        }
    }

    /// Returns the Phase 3 default policy.
    ///
    /// The default accepts at least an exact/strong deterministic match and
    /// requires one ordinal rank of separation from the runner-up.
    #[must_use]
    pub fn phase3_default() -> Self {
        Self::new(CandidateSignal::Normalized.priority(), 1)
    }

    /// Returns the absolute acceptance threshold.
    #[must_use]
    pub const fn threshold(&self) -> u8 {
        self.threshold
    }

    /// Returns the minimum candidate margin.
    #[must_use]
    pub const fn margin(&self) -> u8 {
        self.margin
    }

    /// Returns the deterministic ranking configuration.
    #[must_use]
    pub fn ranking(&self) -> &RankingPolicy {
        &self.ranking
    }
}

impl Default for ResolutionPolicy {
    fn default() -> Self {
        Self::phase3_default()
    }
}

/// The minimal reversible resolution decision represented in Phase 3.
///
/// This is intentionally a value object. It has no `ResolutionId` and no
/// historical version chain. Full revision history belongs to later
/// provenance/versioning architecture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionDecision {
    state: ResolutionState,
    entity_id: Option<EntityId>,
    primary_priority: u8,
    margin: u8,
}

impl ResolutionDecision {
    /// Constructs a resolution decision value.
    #[must_use]
    pub fn new(
        state: ResolutionState,
        entity_id: Option<EntityId>,
        primary_priority: u8,
        margin: u8,
    ) -> Self {
        Self {
            state,
            entity_id,
            primary_priority,
            margin,
        }
    }

    /// Returns the decision state.
    #[must_use]
    pub const fn state(&self) -> ResolutionState {
        self.state
    }

    /// Returns the selected canonical entity, when one is present.
    #[must_use]
    pub fn entity_id(&self) -> Option<&EntityId> {
        self.entity_id.as_ref()
    }

    /// Returns the strongest ordinal candidate priority considered.
    #[must_use]
    pub const fn primary_priority(&self) -> u8 {
        self.primary_priority
    }

    /// Returns the observed ordinal candidate margin.
    #[must_use]
    pub const fn margin(&self) -> u8 {
        self.margin
    }

    /// Returns whether the decision is canonical resolution.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        matches!(self.state, ResolutionState::Resolved)
    }
}

/// Ranks candidates using the fixed deterministic ordered-rule policy.
#[must_use]
pub fn rank_candidates(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut ranked = candidates.to_vec();

    ranked.sort_by(|left, right| {
        right
            .primary_priority()
            .cmp(&left.primary_priority())
            .then_with(|| {
                let left_secondary = secondary_priorities(left);
                let right_secondary = secondary_priorities(right);
                right_secondary.cmp(&left_secondary)
            })
            .then_with(|| left.entity_id().cmp(right.entity_id()))
    });

    ranked
}

fn secondary_priorities(candidate: &Candidate) -> Vec<u8> {
    let mut priorities = candidate
        .signals()
        .iter()
        .map(|signal| signal.priority())
        .collect::<Vec<_>>();
    priorities.sort_unstable_by(|left, right| right.cmp(left));
    priorities
}

/// Converts ranked candidates into the minimal Phase 3 resolution decision.
#[must_use]
pub fn decide(candidates: &[Candidate], policy: &ResolutionPolicy) -> ResolutionDecision {
    if candidates.is_empty() {
        return ResolutionDecision::new(ResolutionState::Unknown, None, 0, 0);
    }

    let ranked = rank_candidates(candidates);
    let best = &ranked[0];
    let second_priority = ranked.get(1).map(Candidate::primary_priority).unwrap_or(0);
    let margin = best.primary_priority().saturating_sub(second_priority);

    if best.primary_priority() < policy.threshold() {
        return ResolutionDecision::new(
            ResolutionState::Unresolved,
            None,
            best.primary_priority(),
            margin,
        );
    }

    if ranked.len() > 1 && margin < policy.margin() {
        return ResolutionDecision::new(
            ResolutionState::Ambiguous,
            None,
            best.primary_priority(),
            margin,
        );
    }

    ResolutionDecision::new(
        ResolutionState::Resolved,
        Some(best.entity_id().clone()),
        best.primary_priority(),
        margin,
    )
}

/// Converts a useful working hypothesis into a provisional decision.
///
/// This operation never creates a canonical entity identity.
#[must_use]
pub fn provisional(candidate: &Candidate) -> ResolutionDecision {
    ResolutionDecision::new(
        ResolutionState::Provisional,
        Some(candidate.entity_id().clone()),
        candidate.primary_priority(),
        0,
    )
}

/// Explicitly rejects a candidate without mutating the canonical entity.
#[must_use]
pub fn rejected(candidate: &Candidate) -> ResolutionDecision {
    ResolutionDecision::new(
        ResolutionState::Rejected,
        Some(candidate.entity_id().clone()),
        candidate.primary_priority(),
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::super::candidate::{Candidate, CandidateSignal};
    use super::{
        ResolutionPolicy, ResolutionState, decide, provisional, rank_candidates, rejected,
    };
    use crate::identity::EntityId;

    fn candidate(id: &str, signals: &[CandidateSignal]) -> Candidate {
        Candidate::new(
            EntityId::new(id).expect("valid entity"),
            signals.iter().copied().collect(),
        )
    }

    #[test]
    fn deterministic_order_prefers_stronger_signals() {
        let candidates = vec![
            candidate("entity-2", &[CandidateSignal::Normalized]),
            candidate("entity-1", &[CandidateSignal::ExactAlias]),
        ];

        let ranked = rank_candidates(&candidates);

        assert_eq!(ranked[0].entity_id().as_str(), "entity-1");
        assert_eq!(ranked[1].entity_id().as_str(), "entity-2");
    }

    #[test]
    fn default_policy_requires_absolute_threshold() {
        let candidates = vec![candidate("entity-1", &[CandidateSignal::Context])];
        let decision = decide(&candidates, &ResolutionPolicy::default());

        assert_eq!(decision.state(), ResolutionState::Unresolved);
        assert!(decision.entity_id().is_none());
    }

    #[test]
    fn default_policy_requires_candidate_margin() {
        let candidates = vec![
            candidate("entity-1", &[CandidateSignal::ExactAlias]),
            candidate("entity-2", &[CandidateSignal::ExactAlias]),
        ];
        let decision = decide(&candidates, &ResolutionPolicy::default());

        assert_eq!(decision.state(), ResolutionState::Ambiguous);
        assert_eq!(decision.margin(), 0);
    }

    #[test]
    fn sufficient_threshold_and_margin_resolve_to_the_best_candidate() {
        let candidates = vec![
            candidate("entity-1", &[CandidateSignal::ExactIdentifier]),
            candidate("entity-2", &[CandidateSignal::Normalized]),
        ];
        let decision = decide(&candidates, &ResolutionPolicy::default());

        assert_eq!(decision.state(), ResolutionState::Resolved);
        assert_eq!(
            decision.entity_id().expect("resolved entity").as_str(),
            "entity-1"
        );
    }

    #[test]
    fn provisional_decisions_do_not_create_a_new_identity() {
        let candidate = candidate("entity-1", &[CandidateSignal::Normalized]);
        let decision = provisional(&candidate);

        assert_eq!(decision.state(), ResolutionState::Provisional);
        assert_eq!(
            decision.entity_id().expect("candidate identity").as_str(),
            "entity-1"
        );
    }

    #[test]
    fn rejection_is_explicit_and_reversible_as_a_value() {
        let candidate = candidate("entity-1", &[CandidateSignal::ExactAlias]);
        let decision = rejected(&candidate);

        assert_eq!(decision.state(), ResolutionState::Rejected);
        assert_eq!(
            decision.entity_id().expect("rejected candidate").as_str(),
            "entity-1"
        );
    }
}
