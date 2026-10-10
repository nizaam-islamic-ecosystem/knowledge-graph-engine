//! KG-owned deterministic ranking profiles and explainable ranking metadata.
//!
//! Ranking is a retrieval-order concern. It is deliberately distinct from
//! confidence, authority, evidence quality, and semantic truth.

use std::collections::BTreeMap;
use std::fmt;

use super::result::{QueryMatchCandidate, QueryMatchType};

/// Initial KG-owned ranking profiles.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RankingProfile {
    /// General deterministic relevance ordering.
    General,
    /// Semantic-match-oriented relevance ordering.
    Semantic,
    /// Evidence/authority-aware scholarly ordering.
    Scholarly,
    /// Temporal/path-aware historical ordering.
    Historical,
    /// Extension point for a named future KG ranking policy.
    Custom(String),
}

impl RankingProfile {
    /// Returns a stable profile label.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::General => "general",
            Self::Semantic => "semantic",
            Self::Scholarly => "scholarly",
            Self::Historical => "historical",
            Self::Custom(name) => name.as_str(),
        }
    }
}

/// Explainable ranking signals.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RankingSignal {
    /// Base score associated with the match type.
    MatchQuality,
    /// Bonus/penalty associated with matched path length.
    PathRelevance,
    /// Presence/quantity of evidence references.
    EvidenceSupport,
    /// Presence/quantity of authority metadata.
    AuthorityMetadata,
    /// Presence of valid-time metadata.
    TemporalRelevance,
}

/// Ranking metadata carried with every final result item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankingMetadata {
    profile: RankingProfile,
    score: u64,
    signals: BTreeMap<RankingSignal, u64>,
}

impl RankingMetadata {
    /// Creates deterministic ranking metadata.
    #[must_use]
    pub fn new(profile: RankingProfile, score: u64, signals: BTreeMap<RankingSignal, u64>) -> Self {
        Self {
            profile,
            score,
            signals,
        }
    }

    /// Returns the selected ranking profile.
    #[must_use]
    pub fn profile(&self) -> &RankingProfile {
        &self.profile
    }

    /// Returns the opaque relevance score.
    #[must_use]
    pub const fn score(&self) -> u64 {
        self.score
    }

    /// Returns explainable score components.
    #[must_use]
    pub fn signals(&self) -> &BTreeMap<RankingSignal, u64> {
        &self.signals
    }
}

/// A ranked candidate used by the query execution layer before result shaping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedCandidate {
    /// Original factual candidate.
    pub candidate: QueryMatchCandidate,
    /// Deterministic ranking metadata.
    pub ranking: RankingMetadata,
}

/// Pluggable KG-owned ranking policy.
pub trait RankingProvider {
    /// Produces deterministic ranking metadata for one candidate.
    fn rank(&self, profile: &RankingProfile, candidate: &QueryMatchCandidate) -> RankingMetadata;
}

/// Default deterministic ranking implementation used by Phase 6.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicRankingProvider;

impl RankingProvider for DeterministicRankingProvider {
    fn rank(&self, profile: &RankingProfile, candidate: &QueryMatchCandidate) -> RankingMetadata {
        let base = match (profile, candidate.match_type()) {
            (_, QueryMatchType::Lookup) => 1_000,
            (_, QueryMatchType::Exact) => 950,
            (RankingProfile::Semantic, QueryMatchType::Semantic) => 1_000,
            (_, QueryMatchType::Semantic) => 900,
            (RankingProfile::Scholarly, QueryMatchType::Conceptual) => 930,
            (_, QueryMatchType::Conceptual) => 880,
            (RankingProfile::Scholarly, QueryMatchType::Relational) => 910,
            (_, QueryMatchType::Relational) => 860,
            (_, QueryMatchType::Lexical) => 820,
            (_, QueryMatchType::Traversal) => 780,
        };

        let path_len = candidate.path().map_or(0, |path| path.len());
        let path_relevance = 100u64.saturating_sub((path_len as u64).saturating_mul(10));
        let evidence_support = (candidate.evidence_ids().len().min(10) as u64) * 10;
        let authority_metadata = (candidate.authority_dimensions().len().min(10) as u64) * 8;
        let temporal_relevance = u64::from(candidate.validity().is_some());

        let mut score = base + path_relevance + evidence_support + authority_metadata;

        if matches!(profile, RankingProfile::Historical) {
            score = score.saturating_add(temporal_relevance.saturating_mul(20));
        } else if matches!(profile, RankingProfile::Scholarly) {
            score = score.saturating_add(evidence_support + authority_metadata);
        } else if matches!(profile, RankingProfile::Semantic) {
            score = score.saturating_add(match candidate.match_type() {
                QueryMatchType::Semantic | QueryMatchType::Conceptual => 50,
                _ => 0,
            });
        }

        let mut signals = BTreeMap::new();
        signals.insert(RankingSignal::MatchQuality, base);
        signals.insert(RankingSignal::PathRelevance, path_relevance);
        signals.insert(RankingSignal::EvidenceSupport, evidence_support);
        signals.insert(RankingSignal::AuthorityMetadata, authority_metadata);
        signals.insert(RankingSignal::TemporalRelevance, temporal_relevance);

        RankingMetadata::new(profile.clone(), score, signals)
    }
}

/// Ranks candidates using the supplied KG-owned ranking provider.
#[must_use]
pub fn rank_query_candidates<P>(
    provider: &P,
    profile: &RankingProfile,
    candidates: impl IntoIterator<Item = QueryMatchCandidate>,
) -> Vec<RankedCandidate>
where
    P: RankingProvider,
{
    let mut ranked = candidates
        .into_iter()
        .map(|candidate| {
            let ranking = provider.rank(profile, &candidate);
            RankedCandidate { candidate, ranking }
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .ranking
            .score()
            .cmp(&left.ranking.score())
            .then_with(|| {
                left.candidate
                    .reference()
                    .stable_key()
                    .cmp(&right.candidate.reference().stable_key())
            })
            .then_with(|| {
                left.candidate
                    .path()
                    .map_or(0, |path| path.len())
                    .cmp(&right.candidate.path().map_or(0, |path| path.len()))
            })
    });

    ranked
}

impl fmt::Display for RankingProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DeterministicRankingProvider, RankingProfile, RankingProvider, rank_query_candidates,
    };
    use crate::identity::KnowledgeAssertionId;
    use crate::query::{InferenceStatus, QueryMatchCandidate, QueryMatchType, QueryReference};

    fn candidate(id: &str, kind: QueryMatchType) -> QueryMatchCandidate {
        QueryMatchCandidate::new(
            QueryReference::Assertion(KnowledgeAssertionId::new(id).unwrap()),
            kind,
        )
        .with_inference_status(InferenceStatus::Observed)
    }

    #[test]
    fn ranking_profiles_produce_deterministic_metadata() {
        let provider = DeterministicRankingProvider;
        let candidate = candidate("assertion-ranking", QueryMatchType::Semantic);
        let first = provider.rank(&RankingProfile::Semantic, &candidate);
        let second = provider.rank(&RankingProfile::Semantic, &candidate);

        assert_eq!(first, second);
        assert!(first.score() > 0);
        assert!(
            first
                .signals()
                .contains_key(&super::RankingSignal::MatchQuality)
        );
    }

    #[test]
    fn ranking_ties_use_stable_reference_order() {
        let provider = DeterministicRankingProvider;
        let ranked = rank_query_candidates(
            &provider,
            &RankingProfile::General,
            [
                candidate("assertion-b", QueryMatchType::Semantic),
                candidate("assertion-a", QueryMatchType::Semantic),
            ],
        );

        assert_eq!(
            ranked[0].candidate.reference().stable_key(),
            "assertion:assertion-a"
        );
    }
}
