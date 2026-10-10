//! Level 3 integration tests for Phase 4 multidimensional authority.

use nizaam_knowledge_graph::KnowledgeAssertionId;
use nizaam_knowledge_graph::assertion::KnowledgeAssertionValidationError;
use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::authority::{
    Authority, AuthorityDimension, AuthorityEvaluationProfile, AuthorityTarget, AuthorityValue,
    ProcessReliability, ScholarlyStatus, SourceReliability,
};
use nizaam_knowledge_graph::identity::{ConceptId, EntityId, SourceId};
use nizaam_knowledge_graph::source::{Source, SourceError};
use nizaam_knowledge_graph::temporal::{Instant, TemporalValidity};

fn assertion() -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-authority-subject").unwrap()),
        AssertionPredicate::new("has-description").unwrap(),
        AssertionObject::Concept(ConceptId::new("concept-authority-object").unwrap()),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Known,
        AssertionPolarity::Positive,
    )
}

#[test]
fn source_and_process_reliability_are_separate_dimensions_not_a_global_score() {
    let evidence_id = nizaam_knowledge_graph::identity::EvidenceId::new("evidence-authority")
        .expect("valid evidence identity");
    let source_reliability = SourceReliability::new("well-attested")
        .expect("valid source reliability")
        .with_basis("independent source review")
        .expect("valid source reliability basis")
        .with_evidence([evidence_id.clone()]);
    let process_reliability = ProcessReliability::new("needs-review")
        .expect("valid process reliability")
        .with_basis("extraction confidence not independently checked")
        .expect("valid process basis")
        .with_evidence([evidence_id]);
    let target = AuthorityTarget::Source(SourceId::new("source-authority").unwrap());
    let authority = Authority::new(target.clone())
        .with_dimension(AuthorityDimension::SourceReliability(
            source_reliability.clone(),
        ))
        .expect("source reliability dimension")
        .with_dimension(AuthorityDimension::ProcessReliability(
            process_reliability.clone(),
        ))
        .expect("process reliability dimension");

    assert_eq!(authority.target(), &target);
    assert_eq!(authority.dimension_count(), 2);
    assert_eq!(source_reliability.assessment().label(), "well-attested");
    assert_eq!(process_reliability.assessment().label(), "needs-review");
    assert_ne!(
        std::any::TypeId::of::<SourceReliability>(),
        std::any::TypeId::of::<ProcessReliability>()
    );
}

#[test]
fn scholarly_vocabularies_and_domain_evaluation_profiles_remain_extensible() {
    let scholarly = ScholarlyStatus::in_vocabulary("hadith-status-v1", "requires-review")
        .expect("valid domain scholarly status");
    let profile = AuthorityEvaluationProfile::new("authority-profile-v1", "scholarly-assessment")
        .expect("valid authority profile")
        .with_domain("hadith")
        .expect("valid profile domain")
        .with_version("v2")
        .expect("valid profile version");
    let target = AuthorityTarget::Assertion(
        KnowledgeAssertionId::new("assertion-authority-profile").unwrap(),
    );
    let authority = Authority::new(target)
        .with_dimension(AuthorityDimension::ScholarlyStatus(scholarly.clone()))
        .expect("scholarly status dimension")
        .with_evaluation_profile(profile.clone());

    assert_eq!(scholarly.vocabulary(), Some("hadith-status-v1"));
    assert_eq!(scholarly.value(), "requires-review");
    assert_eq!(profile.domain(), Some("hadith"));
    assert_eq!(profile.version(), Some("v2"));
    assert_eq!(authority.evaluation_profiles().next(), Some(&profile));
}

#[test]
fn authority_can_be_attached_to_its_matching_source_and_assertion() {
    let source_id = SourceId::new("source-authority-attached").unwrap();
    let source_authority = Authority::new(AuthorityTarget::Source(source_id.clone()))
        .with_dimension(AuthorityDimension::SourceAuthority(
            AuthorityValue::new("curated").expect("valid source authority value"),
        ))
        .expect("valid authority dimension");
    let source = Source::new(source_id.clone(), "Curated source")
        .with_authority(source_authority)
        .expect("source authority target must match source");

    assert_eq!(source.id(), &source_id);
    assert_eq!(
        source.authority().unwrap().target(),
        &AuthorityTarget::Source(source_id)
    );
    assert!(source.validate().is_ok());

    let original = assertion();
    let original_id = original.id().clone();
    let assertion_authority = Authority::new(AuthorityTarget::Assertion(original_id.clone()))
        .with_dimension(AuthorityDimension::HumanReview(
            AuthorityValue::new("reviewed").expect("valid review designation"),
        ))
        .expect("valid assertion authority");
    let enriched = original
        .with_validity(TemporalValidity::at(Instant::from_unix_seconds(100)))
        .with_authority(assertion_authority)
        .expect("authority target must match canonical assertion");

    assert_eq!(enriched.id(), &original_id);
    assert!(enriched.validity().is_some());
    assert!(enriched.authority().is_some());
    assert!(enriched.validate().is_ok());

    let status_updated = enriched.with_status(AssertionStatus::Uncertain);
    assert_eq!(status_updated.id(), &original_id);
    assert_eq!(status_updated.status(), AssertionStatus::Uncertain);
    assert!(status_updated.validate().is_ok());
}

#[test]
fn authority_attachment_rejects_a_target_that_does_not_match_the_owner() {
    let source = Source::new(SourceId::new("source-owner").unwrap(), "Owner source");
    let wrong_source_authority = Authority::new(AuthorityTarget::Source(
        SourceId::new("source-not-owner").unwrap(),
    ));
    assert!(matches!(
        source.with_authority(wrong_source_authority),
        Err(SourceError::AuthorityTargetMismatch { .. })
    ));

    let knowledge_assertion = assertion();
    let wrong_assertion_authority = Authority::new(AuthorityTarget::Source(
        SourceId::new("source-not-assertion").unwrap(),
    ));
    assert!(matches!(
        knowledge_assertion.with_authority(wrong_assertion_authority),
        Err(KnowledgeAssertionValidationError::AuthorityTargetMismatch { .. })
    ));
}
