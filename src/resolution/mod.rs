//! Phase 3 entity-resolution public boundary.
//!
//! Resolution is intentionally limited to deterministic normalization,
//! candidate generation, ranking, disambiguation, minimal reversible
//! decisions, external identifier crosswalks, and structural circular-evidence
//! protection. Persistence, entity merging, full provenance/evidence,
//! `IndexAssignedId` generation, and ML-assisted canonicalization remain outside
//! this module.
//!
//! `ExternalIdentifier` is canonically defined by the entity layer and
//! re-exported here so existing `resolution::ExternalIdentifier` callers remain
//! source-compatible. Crosswalks remain resolution-owned mappings from an
//! external identifier to a canonical entity identity.

mod candidate;
mod crosswalk;
mod disambiguation;
mod matching;
mod resolver;

pub use crate::entity::{ExternalIdentifier, ExternalIdentifierError};
pub use candidate::{
    Candidate, CandidateSignal, EntityCandidateProfile, ResolutionInput, ResolutionReference,
    generate_candidates,
};
pub use crosswalk::ExternalIdentifierCrosswalk;
pub use disambiguation::{
    RankingPolicy, ResolutionDecision, ResolutionPolicy, ResolutionState, decide, provisional,
    rank_candidates, rejected,
};
pub use matching::{
    NormalizationError, exact_match, normalize, normalized_match, transliteration_match,
};
pub use resolver::{
    CircularEvidenceGuard, GraphEvidenceDependency, ResolutionDependencyToken, ResolutionError,
    ResolutionResult, Resolver,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{EntityId, MentionId};
    use std::any::TypeId;

    #[test]
    fn public_resolution_boundary_supports_the_complete_phase3_flow() {
        let resolver = Resolver::phase3_default();
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            "Muhammad",
        );
        let profile = EntityCandidateProfile::new(EntityId::new("entity-1").expect("valid entity"))
            .with_alias("Muhammad");

        let result = resolver.resolve(&input, &[profile]);

        assert_eq!(result.decision().state(), ResolutionState::Resolved);
        assert_eq!(
            result.resolved_entity().expect("resolved entity").as_str(),
            "entity-1"
        );
    }

    #[test]
    fn public_boundary_keeps_candidate_identity_distinct_from_resolution_state() {
        let candidate = Candidate::new(
            EntityId::new("entity-1").expect("valid entity"),
            [CandidateSignal::ExactAlias].into_iter().collect(),
        );
        let decision = provisional(&candidate);

        assert_eq!(
            candidate.entity_id(),
            decision.entity_id().expect("candidate reference")
        );
        assert_eq!(decision.state(), ResolutionState::Provisional);
    }

    #[test]
    fn resolution_external_identifier_is_the_entity_canonical_type() {
        assert_eq!(
            TypeId::of::<ExternalIdentifier>(),
            TypeId::of::<crate::entity::ExternalIdentifier>()
        );
        assert_eq!(
            TypeId::of::<ExternalIdentifierError>(),
            TypeId::of::<crate::entity::ExternalIdentifierError>()
        );
    }

    #[test]
    fn public_boundary_exposes_external_crosswalk_without_indexing_identity() {
        let external = ExternalIdentifier::new(
            crate::identity::SourceId::new("source-a").expect("valid source"),
            "external-42",
        )
        .expect("valid external identifier");
        let entity = EntityId::new("entity-1").expect("valid entity");
        let crosswalk = ExternalIdentifierCrosswalk::new(external, entity.clone());

        assert_eq!(crosswalk.canonical_entity(), &entity);
        assert_eq!(crosswalk.external_identifier().value(), "external-42");
    }
}
