//! Phase 4 evidence and verification foundations.
//!
//! Evidence records why knowledge may be supported, refuted, or qualified;
//! support associations explicitly connect evidence to the canonical Phase 2
//! `KnowledgeAssertion`; verification records capture individual evaluations.
//! This module does not own provenance, authority, confidence calculation,
//! persistence, ingestion governance, or reasoning execution.

mod model;
mod support;
mod verification;

pub use model::{
    Evidence, EvidenceMechanism, EvidenceRole, EvidenceRoleError, EvidenceSourceKind,
    EvidenceSourceReference, EvidenceSourceReferenceError, EvidenceValidationError, TextOffsetUnit,
    TextSpan, TextSpanError,
};
pub use support::{EvidenceSupport, EvidenceSupportError};
pub use verification::{
    VerificationError, VerificationOutcome, VerificationPerformer, VerificationRecord,
    VerificationTarget,
};

#[cfg(test)]
mod tests {
    use super::{
        Evidence, EvidenceMechanism, EvidenceRole, EvidenceSourceReference, EvidenceSupport,
        TextOffsetUnit, TextSpan, VerificationOutcome, VerificationPerformer, VerificationRecord,
        VerificationTarget,
    };
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::identity::{
        ConceptId, EntityId, EvidenceId, KnowledgeAssertionId, ReferenceId, SourceId,
        VerificationId,
    };

    fn assertion() -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("public-entity").unwrap()),
            AssertionPredicate::new("related-to").unwrap(),
            AssertionObject::Concept(ConceptId::new("public-concept").unwrap()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        )
    }

    #[test]
    fn public_boundary_exposes_evidence_support_and_verification() {
        let assertion = assertion();
        let source = EvidenceSourceReference::source(SourceId::new("public-source").unwrap());
        let evidence =
            Evidence::direct_source(EvidenceId::new("public-evidence").unwrap(), [source])
                .expect("valid evidence");
        let support = EvidenceSupport::new(&assertion, &evidence, EvidenceRole::Supports)
            .expect("valid support association");
        let verification = VerificationRecord::new(
            VerificationId::new("public-verification").unwrap(),
            VerificationTarget::Evidence(evidence.id().clone()),
            VerificationPerformer::automated_system("public-test").unwrap(),
            VerificationOutcome::Confirmed,
        )
        .expect("valid verification record");

        assert_eq!(support.assertion_id(), assertion.id());
        assert_eq!(
            verification.target(),
            &VerificationTarget::Evidence(evidence.id().clone())
        );
        assert_eq!(verification.outcome(), VerificationOutcome::Confirmed);
    }

    #[test]
    fn direct_and_derived_evidence_mechanisms_are_distinct() {
        let direct = EvidenceMechanism::direct_source();
        let derived =
            EvidenceMechanism::derived_support([EvidenceId::new("input-evidence").unwrap()])
                .expect("valid derived mechanism");

        assert_ne!(direct, derived);
        assert!(derived.inputs().is_some());
    }

    #[test]
    fn different_location_kinds_remain_distinct() {
        let reference = ReferenceId::new("location-ref").unwrap();
        let span = TextSpan::new(0, 4, TextOffsetUnit::Utf8Byte).unwrap();
        let page = EvidenceSourceReference::page(reference.clone(), "7").unwrap();
        let text = EvidenceSourceReference::text_span(reference, span);

        assert_ne!(page, text);
    }

    #[test]
    fn public_identity_types_remain_separate() {
        assert_ne!(
            std::any::TypeId::of::<EvidenceId>(),
            std::any::TypeId::of::<KnowledgeAssertionId>()
        );
        assert_ne!(
            std::any::TypeId::of::<EvidenceId>(),
            std::any::TypeId::of::<VerificationId>()
        );
    }
}
