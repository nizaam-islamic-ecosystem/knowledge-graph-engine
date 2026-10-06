//! Level 3 public-boundary tests for the Phase 1 Concept module.

use nizaam_knowledge_graph::{
    concept::Concept,
    identity::{ConceptId, EntityId, LexicalFormId},
};
use std::any::TypeId;

#[test]
fn concept_public_api_preserves_identity_and_representation() {
    let id = ConceptId::generate();
    let concept = Concept::new(id.clone(), "patience");

    assert_eq!(concept.id(), &id);
    assert_eq!(concept.representation(), "patience");
}

#[test]
fn concept_representation_remains_opaque_and_multilingual() {
    let arabic = Concept::new(ConceptId::generate(), "صبر");
    let english = Concept::new(ConceptId::generate(), "patience");
    let urdu = Concept::new(ConceptId::generate(), "صبر");

    assert_eq!(arabic.representation(), "صبر");
    assert_eq!(english.representation(), "patience");
    assert_eq!(urdu.representation(), "صبر");
}

#[test]
fn concept_identity_is_distinct_from_entity_and_lexical_identity() {
    assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<EntityId>());
    assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<LexicalFormId>());
    assert_ne!(
        TypeId::of::<Concept>(),
        TypeId::of::<nizaam_knowledge_graph::Entity>()
    );
}
