//! Knowledge-graph identity foundations for Phase 1 and Phase 4.
//!
//! This module exposes the strongly typed identities used by Phase 1 semantic
//! objects and the Phase 4 evidence, provenance, verification, and contradiction
//! records. Identity generation, validation, serialization, and comparison
//! behavior remain owned by Nizaam Core's `identity!` mechanism.
//!
//! The semantic objects represented by these identities are intentionally kept
//! outside this module. This module establishes identity boundaries only.

mod activity;
mod agent;
mod assertion;
mod concept;
mod contradiction;
mod entity;
mod evidence;
mod lexical;
mod mention;
mod reference;
mod source;
mod verification;

pub use activity::ActivityId;
pub use agent::AgentId;
pub use assertion::KnowledgeAssertionId;
pub use concept::ConceptId;
pub use contradiction::ContradictionId;
pub use entity::EntityId;
pub use evidence::EvidenceId;
pub use lexical::LexicalFormId;
pub use mention::MentionId;
pub use reference::ReferenceId;
pub use source::SourceId;
pub use verification::VerificationId;

#[cfg(test)]
mod tests {
    use super::{
        ActivityId, AgentId, ConceptId, ContradictionId, EntityId, EvidenceId,
        KnowledgeAssertionId, LexicalFormId, MentionId, ReferenceId, SourceId, VerificationId,
    };
    use std::any::TypeId;

    #[test]
    fn phase1_and_phase4_identity_types_are_all_exposed() {
        let entity = EntityId::generate();
        let concept = ConceptId::generate();
        let source = SourceId::generate();
        let reference = ReferenceId::generate();
        let lexical_form = LexicalFormId::generate();
        let mention = MentionId::generate();
        let assertion = KnowledgeAssertionId::generate();
        let evidence = EvidenceId::generate();
        let activity = ActivityId::generate();
        let agent = AgentId::generate();
        let verification = VerificationId::generate();
        let contradiction = ContradictionId::generate();

        for value in [
            entity.as_str(),
            concept.as_str(),
            source.as_str(),
            reference.as_str(),
            lexical_form.as_str(),
            mention.as_str(),
            assertion.as_str(),
            evidence.as_str(),
            activity.as_str(),
            agent.as_str(),
            verification.as_str(),
            contradiction.as_str(),
        ] {
            assert!(!value.is_empty());
        }
    }

    #[test]
    fn phase1_and_phase4_identity_types_remain_strongly_distinct() {
        let identity_types = [
            TypeId::of::<EntityId>(),
            TypeId::of::<ConceptId>(),
            TypeId::of::<SourceId>(),
            TypeId::of::<ReferenceId>(),
            TypeId::of::<LexicalFormId>(),
            TypeId::of::<MentionId>(),
            TypeId::of::<KnowledgeAssertionId>(),
            TypeId::of::<EvidenceId>(),
            TypeId::of::<ActivityId>(),
            TypeId::of::<AgentId>(),
            TypeId::of::<VerificationId>(),
            TypeId::of::<ContradictionId>(),
        ];

        for (index, current) in identity_types.iter().enumerate() {
            assert!(
                identity_types[index + 1..]
                    .iter()
                    .all(|other| current != other),
                "identity type at index {index} must be distinct from all later types"
            );
        }
    }

    #[test]
    fn identity_module_preserves_core_value_semantics() {
        let entity = EntityId::new("entity-1").expect("valid entity identity");
        let same_entity = EntityId::new("entity-1").expect("same valid entity identity");
        let concept = ConceptId::new("entity-1").expect("valid concept identity with same value");

        assert_eq!(entity, same_entity);
        assert_eq!(entity.as_str(), "entity-1");
        assert_ne!(TypeId::of::<EntityId>(), TypeId::of::<ConceptId>());
        assert_eq!(concept.as_str(), "entity-1");
    }

    #[test]
    fn phase1_and_phase4_identity_generation_is_unique_within_each_identity_type() {
        assert_ne!(EntityId::generate(), EntityId::generate());
        assert_ne!(ConceptId::generate(), ConceptId::generate());
        assert_ne!(SourceId::generate(), SourceId::generate());
        assert_ne!(ReferenceId::generate(), ReferenceId::generate());
        assert_ne!(LexicalFormId::generate(), LexicalFormId::generate());
        assert_ne!(MentionId::generate(), MentionId::generate());
        assert_ne!(
            KnowledgeAssertionId::generate(),
            KnowledgeAssertionId::generate()
        );
        assert_ne!(EvidenceId::generate(), EvidenceId::generate());
        assert_ne!(ActivityId::generate(), ActivityId::generate());
        assert_ne!(AgentId::generate(), AgentId::generate());
        assert_ne!(VerificationId::generate(), VerificationId::generate());
        assert_ne!(ContradictionId::generate(), ContradictionId::generate());
    }
}
