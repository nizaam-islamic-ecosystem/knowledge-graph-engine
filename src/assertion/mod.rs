//! Canonical knowledge-assertion module with Phase 4 metadata integration.
//!
//! This module exposes the assertion's typed proposition, canonical identity,
//! context, qualifiers, polarity, and shared epistemic status. Phase 4 adds
//! target-checked authority metadata and valid-time qualification without
//! changing canonical assertion identity.
//!
//! The assertion module intentionally does not define a separate subject
//! abstraction. `AssertionObject` is used for both assertion subjects and
//! objects. Evidence support, provenance, and contradiction remain explicit
//! typed associations in their owning modules rather than untyped assertion
//! metadata. Ontology-wide validation, reasoning, traversal, persistence, and
//! query execution remain outside this module.

mod context;
mod model;
mod object;
mod predicate;
mod qualifier;
mod status;

pub use context::{AssertionContext, AssertionContextError};
pub use model::{AssertionPolarity, KnowledgeAssertion, KnowledgeAssertionValidationError};
pub use object::AssertionObject;
pub use predicate::AssertionPredicate;
pub use qualifier::{Qualifier, QualifierError, Qualifiers};
pub use status::AssertionStatus;

pub use crate::identity::KnowledgeAssertionId;

#[cfg(test)]
mod tests {
    use super::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifier, Qualifiers,
    };
    use crate::identity::{ConceptId, EntityId, MentionId};

    #[test]
    fn public_assertion_boundary_exposes_complete_phase2_model() {
        let subject =
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity"));

        let object =
            AssertionObject::Concept(ConceptId::new("concept-1").expect("valid concept identity"));

        let predicate = AssertionPredicate::new("has-name").expect("valid relationship predicate");

        let context =
            AssertionContext::from_entries([("source", "quran")]).expect("valid assertion context");

        let qualifier = Qualifier::new("scope", "primary").expect("valid assertion qualifier");

        let qualifiers = Qualifiers::from_iter([qualifier]);

        let assertion = KnowledgeAssertion::new(
            subject,
            predicate,
            object,
            context,
            qualifiers,
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert!(assertion.validate().is_ok());
        assert_eq!(assertion.status(), AssertionStatus::Accepted);
        assert_eq!(assertion.polarity(), AssertionPolarity::Positive);
    }

    #[test]
    fn public_boundary_supports_cross_dimensional_references() {
        let subject =
            AssertionObject::Mention(MentionId::new("mention-1").expect("valid mention identity"));

        let object =
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity"));

        let predicate = AssertionPredicate::new("refers-to").expect("valid relationship predicate");

        let assertion = KnowledgeAssertion::new(
            subject,
            predicate,
            object,
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        );

        assert!(assertion.validate().is_ok());
    }

    #[test]
    fn identical_semantic_assertions_share_identity_across_the_module_boundary() {
        let first = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Concept(ConceptId::new("concept-1").expect("valid concept identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let second = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Concept(ConceptId::new("concept-1").expect("valid concept identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        );

        assert_eq!(first.id(), second.id());
        assert_eq!(first, second);
    }

    #[test]
    fn epistemic_status_does_not_change_semantic_identity() {
        let accepted = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let disputed = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Disputed,
            AssertionPolarity::Positive,
        );

        assert_eq!(accepted.id(), disputed.id());
    }

    #[test]
    fn polarity_is_part_of_semantic_identity() {
        let positive = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let negative = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Negative,
        );

        assert_ne!(positive.id(), negative.id());
        assert_ne!(positive, negative);
    }

    #[test]
    fn context_and_qualifiers_participate_in_assertion_identity() {
        let base = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        let context =
            AssertionContext::from_entries([("source", "quran")]).expect("valid assertion context");

        let qualifier = Qualifier::new("scope", "primary").expect("valid assertion qualifier");

        let qualified = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            context,
            Qualifiers::from_iter([qualifier]),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert_ne!(base.id(), qualified.id());
    }

    #[test]
    fn public_assertion_boundary_preserves_structural_validation() {
        let assertion = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );

        assert!(assertion.validate().is_ok());
    }

    #[test]
    fn phase4_metadata_does_not_change_canonical_assertion_identity() {
        use crate::authority::{Authority, AuthorityDimension, AuthorityTarget, AuthorityValue};
        use crate::temporal::{Instant, TemporalValidity};

        let base = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-phase4").unwrap()),
            AssertionPredicate::new("has-name").unwrap(),
            AssertionObject::Concept(ConceptId::new("concept-phase4").unwrap()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        );
        let canonical_id = base.id().clone();
        let authority = Authority::new(AuthorityTarget::Assertion(canonical_id.clone()))
            .with_dimension(AuthorityDimension::Authentication(
                AuthorityValue::new("reviewed").unwrap(),
            ))
            .unwrap();
        let enriched = base
            .with_status(AssertionStatus::Known)
            .with_validity(TemporalValidity::at(Instant::from_unix_seconds(
                1_750_000_000,
            )))
            .with_authority(authority)
            .expect("authority target matches canonical assertion identity");

        assert_eq!(enriched.id(), &canonical_id);
        assert_eq!(enriched.status(), AssertionStatus::Known);
        assert!(enriched.validity().is_some());
        assert!(enriched.authority().is_some());
        assert!(enriched.validate().is_ok());
    }

    #[test]
    fn assertion_rejects_authority_metadata_for_another_target() {
        use crate::authority::{Authority, AuthorityTarget};
        use crate::identity::SourceId;

        let assertion = KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-target-check").unwrap()),
            AssertionPredicate::new("related-to").unwrap(),
            AssertionObject::Concept(ConceptId::new("concept-target-check").unwrap()),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Provisional,
            AssertionPolarity::Positive,
        );
        let authority = Authority::new(AuthorityTarget::Source(
            SourceId::new("not-this-assertion").unwrap(),
        ));

        assert!(assertion.with_authority(authority).is_err());
    }
}
