//! Explicit semantic associations between evidence and knowledge assertions.
//!
//! This module stores links to the canonical Phase 2 `KnowledgeAssertion`; it
//! does not define a replacement assertion model and does not implement a
//! provenance chain or a reasoning engine.

use core::fmt;

use crate::assertion::{KnowledgeAssertion, KnowledgeAssertionValidationError};
use crate::identity::{EvidenceId, KnowledgeAssertionId};

use super::model::{Evidence, EvidenceRole, EvidenceRoleError, EvidenceValidationError};

/// One role-qualified association between evidence and a knowledge assertion.
///
/// The association stores identities rather than duplicating either semantic
/// object. A piece of evidence can participate in multiple associations with
/// different assertions and roles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSupport {
    assertion_id: KnowledgeAssertionId,
    evidence_id: EvidenceId,
    role: EvidenceRole,
}

impl EvidenceSupport {
    /// Constructs an association after validating both referenced objects and
    /// the evidence role.
    pub fn new(
        assertion: &KnowledgeAssertion,
        evidence: &Evidence,
        role: EvidenceRole,
    ) -> Result<Self, EvidenceSupportError> {
        assertion
            .validate()
            .map_err(EvidenceSupportError::InvalidAssertion)?;
        evidence
            .validate()
            .map_err(EvidenceSupportError::InvalidEvidence)?;
        role.validate().map_err(EvidenceSupportError::InvalidRole)?;

        Ok(Self {
            assertion_id: assertion.id().clone(),
            evidence_id: evidence.id().clone(),
            role,
        })
    }

    /// Returns the canonical assertion identity being qualified.
    #[must_use]
    pub fn assertion_id(&self) -> &KnowledgeAssertionId {
        &self.assertion_id
    }

    /// Returns the supporting evidence identity.
    #[must_use]
    pub fn evidence_id(&self) -> &EvidenceId {
        &self.evidence_id
    }

    /// Returns the role played by this evidence for this assertion.
    #[must_use]
    pub fn role(&self) -> &EvidenceRole {
        &self.role
    }
}

/// Construction errors for an evidence/assertion association.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceSupportError {
    /// The assertion did not satisfy its own structural identity invariant.
    InvalidAssertion(KnowledgeAssertionValidationError),
    /// The evidence object was structurally invalid.
    InvalidEvidence(EvidenceValidationError),
    /// The supplied role was structurally invalid.
    InvalidRole(EvidenceRoleError),
}

impl fmt::Display for EvidenceSupportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAssertion(error) => {
                write!(formatter, "invalid assertion for evidence support: {error}")
            }
            Self::InvalidEvidence(error) => write!(
                formatter,
                "invalid evidence for support association: {error}"
            ),
            Self::InvalidRole(error) => write!(formatter, "invalid evidence support role: {error}"),
        }
    }
}

impl std::error::Error for EvidenceSupportError {}

#[cfg(test)]
mod tests {
    use super::{EvidenceSupport, EvidenceSupportError};
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::evidence::{Evidence, EvidenceRole, EvidenceSourceReference};
    use crate::identity::{ConceptId, EntityId, EvidenceId, SourceId};

    fn assertion() -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-subject").expect("valid entity")),
            AssertionPredicate::new("relates-to").expect("valid predicate"),
            AssertionObject::Concept(ConceptId::new("concept-object").expect("valid concept")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        )
    }

    fn evidence() -> Evidence {
        Evidence::direct_source(
            EvidenceId::new("evidence-source-1").expect("valid evidence identity"),
            [EvidenceSourceReference::source(
                SourceId::new("source-1").expect("valid source identity"),
            )],
        )
        .expect("valid evidence")
    }

    #[test]
    fn support_association_preserves_assertion_evidence_and_role() {
        let assertion = assertion();
        let evidence = evidence();
        let support = EvidenceSupport::new(&assertion, &evidence, EvidenceRole::Supports)
            .expect("valid evidence/assertion association");

        assert_eq!(support.assertion_id(), assertion.id());
        assert_eq!(support.evidence_id(), evidence.id());
        assert_eq!(support.role(), &EvidenceRole::Supports);
    }

    #[test]
    fn the_same_evidence_can_have_different_roles_for_different_assertions() {
        let first_assertion = assertion();
        let mut second_assertion = assertion();
        // Create a second canonical assertion with a different semantic object.
        second_assertion = KnowledgeAssertion::new(
            second_assertion.subject().clone(),
            second_assertion.predicate().clone(),
            AssertionObject::Concept(ConceptId::new("concept-other").expect("valid concept")),
            second_assertion.context().clone(),
            second_assertion.qualifiers().clone(),
            second_assertion.status(),
            second_assertion.polarity(),
        );
        let evidence = evidence();

        let supports = EvidenceSupport::new(&first_assertion, &evidence, EvidenceRole::Supports)
            .expect("support association");
        let refutes = EvidenceSupport::new(&second_assertion, &evidence, EvidenceRole::Refutes)
            .expect("refutation association");

        assert_eq!(supports.evidence_id(), refutes.evidence_id());
        assert_ne!(supports.assertion_id(), refutes.assertion_id());
        assert_ne!(supports.role(), refutes.role());
    }

    #[test]
    fn invalid_custom_role_is_rejected_at_association_boundary() {
        let assertion = assertion();
        let evidence = evidence();
        let error =
            EvidenceSupport::new(&assertion, &evidence, EvidenceRole::Custom("  ".to_owned()))
                .expect_err("empty custom role must be rejected");

        assert!(matches!(error, EvidenceSupportError::InvalidRole(_)));
    }

    #[test]
    fn direct_source_evidence_without_a_source_is_rejected_before_attachment() {
        use crate::evidence::EvidenceValidationError;

        let error = Evidence::direct_source(
            EvidenceId::new("evidence-missing-source").expect("valid identity"),
            [],
        )
        .expect_err("direct evidence requires a source");

        assert_eq!(
            error,
            EvidenceValidationError::DirectSourceRequiresSourceReference
        );
    }
}
