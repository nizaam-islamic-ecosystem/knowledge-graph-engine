//! Level 3 Phase 3 public-boundary tests for deterministic entity resolution.

use nizaam_knowledge_graph::identity::{EntityId, MentionId, SourceId};
use nizaam_knowledge_graph::resolution::{
    CandidateSignal, EntityCandidateProfile, ExternalIdentifier, ExternalIdentifierCrosswalk,
    ResolutionInput, ResolutionPolicy, ResolutionReference, ResolutionState, Resolver, decide,
    normalize, normalized_match, rank_candidates, transliteration_match,
};

fn candidate(
    id: &str,
    signals: &[CandidateSignal],
) -> nizaam_knowledge_graph::resolution::Candidate {
    nizaam_knowledge_graph::resolution::Candidate::new(
        EntityId::new(id).unwrap(),
        signals.iter().copied().collect(),
    )
}

#[test]
fn normalization_is_deterministic() {
    assert_eq!(
        normalize("  Muhammad   ibn  Abdullah ").unwrap(),
        "muhammad ibn abdullah"
    );
    assert!(normalized_match(" Muhammad ", "muhammad").unwrap());
}

#[test]
fn exact_alias_normalized_and_transliteration_matching_are_distinct_supported_strategies() {
    assert!(nizaam_knowledge_graph::resolution::exact_match(
        "Muhammad", "Muhammad"
    ));
    assert!(normalized_match("Muhammad", " muhammad ").unwrap());
    assert!(transliteration_match("Muhammad", " muhammad ").unwrap());
}

#[test]
fn candidate_generation_supports_identifier_alias_source_and_context_signals() {
    let input = ResolutionInput::new(
        ResolutionReference::Mention(MentionId::new("mention-1").unwrap()),
        "Muhammad",
    )
    .with_identifier("person-42")
    .with_source(SourceId::new("source-a").unwrap())
    .with_context_key("quran");
    let profile = EntityCandidateProfile::new(EntityId::new("entity-1").unwrap())
        .with_alias("Muhammad")
        .with_identifier("person-42")
        .with_context_key("quran");
    let candidates = nizaam_knowledge_graph::resolution::generate_candidates(&input, &[profile]);
    assert_eq!(candidates.len(), 1);
    assert!(
        candidates[0]
            .signals()
            .contains(&CandidateSignal::ExactIdentifier)
    );
    assert!(
        candidates[0]
            .signals()
            .contains(&CandidateSignal::ExactAlias)
    );
    assert!(candidates[0].signals().contains(&CandidateSignal::Context));
}

#[test]
fn candidate_generation_supports_lexical_semantic_and_graph_assisted_signals() {
    let input = ResolutionInput::new(
        ResolutionReference::Mention(MentionId::new("mention-1").unwrap()),
        "sabr",
    )
    .with_lexical_key("lexical-sabr")
    .with_semantic_relationship_key("concept-patience")
    .with_graph_neighborhood_key("entity-scholar");
    let profile = EntityCandidateProfile::new(EntityId::new("entity-1").unwrap())
        .with_lexical_key("lexical-sabr")
        .with_semantic_relationship_key("concept-patience")
        .with_graph_neighborhood_key("entity-scholar");
    let candidates = nizaam_knowledge_graph::resolution::generate_candidates(&input, &[profile]);
    assert!(
        candidates[0]
            .signals()
            .contains(&CandidateSignal::LexicalMapping)
    );
    assert!(
        candidates[0]
            .signals()
            .contains(&CandidateSignal::SemanticRelationship)
    );
    assert!(
        candidates[0]
            .signals()
            .contains(&CandidateSignal::GraphNeighborhood)
    );
}

#[test]
fn deterministic_ranking_prefers_stronger_signal() {
    let ranked = rank_candidates(&[
        candidate("entity-2", &[CandidateSignal::Normalized]),
        candidate("entity-1", &[CandidateSignal::ExactAlias]),
    ]);
    assert_eq!(ranked[0].entity_id().as_str(), "entity-1");
}

#[test]
fn threshold_and_margin_control_resolution() {
    let policy = ResolutionPolicy::default();
    let weak = decide(
        &[candidate("entity-1", &[CandidateSignal::Context])],
        &policy,
    );
    assert_eq!(weak.state(), ResolutionState::Unresolved);

    let tied = decide(
        &[
            candidate("entity-1", &[CandidateSignal::ExactAlias]),
            candidate("entity-2", &[CandidateSignal::ExactAlias]),
        ],
        &policy,
    );
    assert_eq!(tied.state(), ResolutionState::Ambiguous);

    let resolved = decide(
        &[
            candidate("entity-1", &[CandidateSignal::ExactIdentifier]),
            candidate("entity-2", &[CandidateSignal::Normalized]),
        ],
        &policy,
    );
    assert_eq!(resolved.state(), ResolutionState::Resolved);
    assert_eq!(resolved.entity_id().unwrap().as_str(), "entity-1");
}

#[test]
fn resolver_supports_all_non_ml_phase3_states() {
    let resolver = Resolver::phase3_default();
    let input = ResolutionInput::new(
        ResolutionReference::Mention(MentionId::new("m1").unwrap()),
        "Muhammad",
    );
    let profiles =
        [EntityCandidateProfile::new(EntityId::new("e1").unwrap()).with_alias("Muhammad")];
    assert_eq!(
        resolver.resolve(&input, &profiles).decision().state(),
        ResolutionState::Resolved
    );
    assert_eq!(
        decide(&[], resolver.policy()).state(),
        ResolutionState::Unknown
    );
    assert_eq!(
        decide(
            &[candidate("e1", &[CandidateSignal::Context])],
            resolver.policy()
        )
        .state(),
        ResolutionState::Unresolved
    );
}

#[test]
fn external_crosswalk_remains_separate_from_canonical_entity_identity() {
    let external =
        ExternalIdentifier::new(SourceId::new("source-a").unwrap(), "person-42").unwrap();
    let entity = EntityId::new("entity-1").unwrap();
    let crosswalk = ExternalIdentifierCrosswalk::new(external.clone(), entity.clone());
    assert_eq!(crosswalk.external_identifier(), &external);
    assert_eq!(crosswalk.canonical_entity(), &entity);
}

#[test]
fn resolution_result_does_not_create_or_merge_entities() {
    let resolver = Resolver::phase3_default();
    let input = ResolutionInput::new(
        ResolutionReference::Mention(MentionId::new("m1").unwrap()),
        "Muhammad",
    );
    let profile = EntityCandidateProfile::new(EntityId::new("e1").unwrap()).with_alias("Muhammad");
    let result = resolver.resolve(&input, &[profile]);
    assert_eq!(result.resolved_entity().unwrap().as_str(), "e1");
    assert_eq!(result.candidates().len(), 1);
}

#[test]
fn circular_graph_evidence_is_not_treated_as_independent() {
    use nizaam_knowledge_graph::identity::KnowledgeAssertionId;
    use nizaam_knowledge_graph::resolution::{
        CircularEvidenceGuard, GraphEvidenceDependency, ResolutionDependencyToken, ResolutionError,
    };
    let resolver = Resolver::phase3_default();
    let token = ResolutionDependencyToken::new(7);
    let dependency =
        GraphEvidenceDependency::new(KnowledgeAssertionId::new("assertion-1").unwrap(), [token]);
    let mut guard = CircularEvidenceGuard::new();
    guard.activate(token);
    assert!(matches!(
        resolver.validate_graph_evidence(&guard, &dependency),
        Err(ResolutionError::CircularEvidence { .. })
    ));
}
