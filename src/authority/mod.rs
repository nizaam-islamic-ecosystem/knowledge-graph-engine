//! Phase 4 authority metadata for sources and knowledge assertions.
//!
//! Authority is multi-dimensional semantic metadata. It is distinct from
//! runtime authorization, provenance, verification records, confidence, and
//! any universal notion of trust.

mod model;
mod reliability;
mod scholarly;

pub use model::{
    Authority, AuthorityDimension, AuthorityError, AuthorityEvaluationProfile, AuthorityTarget,
    AuthorityValue,
};
pub use reliability::{
    ProcessReliability, ReliabilityAssessment, ReliabilityError, SourceReliability,
};
pub use scholarly::{ScholarlyStatus, ScholarlyStatusError};

#[cfg(test)]
mod tests {
    use super::{
        Authority, AuthorityDimension, AuthorityEvaluationProfile, AuthorityTarget, AuthorityValue,
        ProcessReliability, ScholarlyStatus, SourceReliability,
    };
    use crate::identity::{KnowledgeAssertionId, SourceId};

    #[test]
    fn public_authority_boundary_exposes_typed_multidimensional_metadata() {
        let target = AuthorityTarget::Assertion(
            KnowledgeAssertionId::new("assertion-public-api").expect("valid assertion identity"),
        );
        let source_reliability =
            SourceReliability::new("trusted-source").expect("valid source reliability");
        let process_reliability =
            ProcessReliability::new("needs-review").expect("valid process reliability");
        let scholarly_status = ScholarlyStatus::in_vocabulary("example-domain", "disputed")
            .expect("valid scholarly status");
        let profile = AuthorityEvaluationProfile::new("example-profile-v1", "authority-review")
            .expect("valid profile");

        let authority = Authority::new(target.clone())
            .with_dimension(AuthorityDimension::SourceReliability(source_reliability))
            .expect("valid source reliability dimension")
            .with_dimension(AuthorityDimension::ProcessReliability(process_reliability))
            .expect("valid process reliability dimension")
            .with_dimension(AuthorityDimension::ScholarlyStatus(scholarly_status))
            .expect("valid scholarly status dimension")
            .with_dimension(AuthorityDimension::Authentication(
                AuthorityValue::new("not-yet-authenticated").expect("valid authority value"),
            ))
            .expect("valid authentication dimension")
            .with_evaluation_profile(profile);

        assert_eq!(authority.target(), &target);
        assert_eq!(authority.dimension_count(), 4);
        assert_eq!(authority.evaluation_profiles().count(), 1);
    }

    #[test]
    fn source_and_assertion_authority_use_their_existing_typed_identities() {
        let source = Authority::new(AuthorityTarget::Source(
            SourceId::new("source-public-api").expect("valid source identity"),
        ));
        let assertion = Authority::new(AuthorityTarget::Assertion(
            KnowledgeAssertionId::new("assertion-public-api").expect("valid assertion identity"),
        ));

        assert!(matches!(source.target(), AuthorityTarget::Source(_)));
        assert!(matches!(assertion.target(), AuthorityTarget::Assertion(_)));
        assert!(source.is_empty());
        assert!(assertion.is_empty());
    }

    #[test]
    fn authority_extension_supports_domain_specific_dimensions() {
        let extension = AuthorityDimension::extension(
            "hadith-authentication-method",
            AuthorityValue::in_vocabulary("hadith-methods-v1", "isnad-review")
                .expect("valid domain value"),
        )
        .expect("valid extension dimension");
        let authority = Authority::new(AuthorityTarget::Source(SourceId::generate()))
            .with_dimension(extension)
            .expect("extension should be retained");

        assert_eq!(
            authority.dimensions().next().unwrap().name(),
            "hadith-authentication-method"
        );
    }
}
