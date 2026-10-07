//! Level 3 Phase 3 public-boundary tests for semantic behavior.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionStatus, KnowledgeAssertion,
    Qualifiers,
};
use nizaam_knowledge_graph::identity::{ConceptId, LexicalFormId, SourceId};
use nizaam_knowledge_graph::ontology::ClassId;
use nizaam_knowledge_graph::relationship::RelationshipPredicate;
use nizaam_knowledge_graph::semantics::{
    Context, ContextDimension, ContextDimensionKey, ContextKind, Interpretation,
    InterpretationSource, LexicalConceptMapping, LexicalConceptMappingKind, Meaning, SemanticType,
    SemanticTypeTarget, SemanticTypes, SenseReference,
};

fn assertion() -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        AssertionObject::LexicalForm(LexicalFormId::new("lexical-sabr").unwrap()),
        RelationshipPredicate::new("expresses").unwrap(),
        AssertionObject::Concept(ConceptId::new("concept-patience").unwrap()),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    )
}

#[test]
fn reusable_and_ephemeral_contexts_are_distinct() {
    let reusable_id = nizaam_knowledge_graph::semantics::ContextId::new("context-quran").unwrap();
    let reusable = Context::reusable(reusable_id.clone());
    let ephemeral = Context::ephemeral();
    assert_eq!(reusable.kind(), &ContextKind::Reusable(reusable_id));
    assert_eq!(ephemeral.kind(), &ContextKind::Ephemeral);
    assert!(ephemeral.id().is_none());
}

#[test]
fn context_supports_typed_dimensions_and_extension_dimensions() {
    let mut context = Context::ephemeral();
    context
        .insert_dimension(ContextDimension::Language("ar".into()))
        .unwrap();
    context
        .insert_dimension(ContextDimension::Source(SourceId::new("quran").unwrap()))
        .unwrap();
    context
        .insert_dimension(ContextDimension::Domain(
            ClassId::new("class-quran").unwrap(),
        ))
        .unwrap();
    context.insert_extension("custom-domain", "tafsir").unwrap();

    assert!(context.contains_dimension(ContextDimensionKey::Language));
    assert!(context.contains_dimension(ContextDimensionKey::Source));
    assert_eq!(context.extension("custom-domain"), Some("tafsir"));
}

#[test]
fn context_rejects_duplicate_known_dimensions() {
    let mut context = Context::ephemeral();
    context
        .insert_dimension(ContextDimension::Language("ar".into()))
        .unwrap();
    assert!(
        context
            .insert_dimension(ContextDimension::Language("en".into()))
            .is_err()
    );
}

#[test]
fn meaning_is_first_class_and_structurally_independent_from_concept() {
    let meaning = Meaning::new("patience");
    let concept = ConceptId::new("patience").unwrap();

    assert_eq!(meaning.representation(), "patience");
    assert_eq!(concept.as_str(), "patience");
    assert_ne!(
        std::any::TypeId::of::<Meaning>(),
        std::any::TypeId::of::<ConceptId>()
    );
}

#[test]
fn interpretation_preserves_source_and_context() {
    let mut context = Context::ephemeral();
    context
        .insert_dimension(ContextDimension::Language("ar".into()))
        .unwrap();
    let interpretation = Interpretation::new(
        InterpretationSource::LexicalForm(LexicalFormId::new("lexical-sabr").unwrap()),
        context,
    );
    assert!(matches!(
        interpretation.source(),
        InterpretationSource::LexicalForm(_)
    ));
    assert!(
        interpretation
            .context()
            .contains_dimension(ContextDimensionKey::Language)
    );
}

#[test]
fn direct_lexical_to_concept_mapping_wraps_existing_assertion() {
    let assertion = assertion();
    let id = assertion.id().clone();
    let mapping = LexicalConceptMapping::direct(assertion).unwrap();
    assert!(matches!(mapping.kind(), LexicalConceptMappingKind::Direct));
    assert_eq!(mapping.assertion().id(), &id);
}

#[test]
fn sense_mediated_lexical_to_concept_mapping_preserves_sense_reference() {
    let sense = SenseReference::new("sense-sabr-1").unwrap();
    let mapping = LexicalConceptMapping::sense_mediated(assertion(), sense.clone()).unwrap();
    assert!(
        matches!(mapping.kind(), LexicalConceptMappingKind::SenseMediated { sense: value } if value == &sense)
    );
}

#[test]
fn semantic_types_keep_class_and_concept_targets_distinct() {
    let class = SemanticType::class(ClassId::new("class-person").unwrap());
    let concept = SemanticType::concept(ConceptId::new("concept-person").unwrap());
    assert_eq!(
        class.target(),
        &SemanticTypeTarget::Class(ClassId::new("class-person").unwrap())
    );
    assert_eq!(
        concept.target(),
        &SemanticTypeTarget::Concept(ConceptId::new("concept-person").unwrap())
    );
    assert_ne!(class, concept);
}

#[test]
fn semantic_types_support_primary_and_additional_membership() {
    let primary = SemanticType::class(ClassId::new("class-person").unwrap());
    let additional = SemanticType::class(ClassId::new("class-scholar").unwrap());
    let mut types = SemanticTypes::new(primary.clone());
    types.add_additional(additional.clone());
    types.add_additional(primary.clone());
    assert_eq!(types.primary(), &primary);
    assert!(types.contains(&additional));
    assert_eq!(types.additional().count(), 1);
}
