//! Phase 3 semantic foundation.
//!
//! This module adds semantic context, meaning, interpretation, lexical-to-
//! concept semantic connections, and semantic typing on top of the Phase 1
//! identities and Phase 2 assertion model.
//!
//! The module deliberately does not implement Arabic linguistic analysis,
//! ingestion, persistence, ML inference, or a competing assertion/relationship
//! system.

mod context;
mod interpretation;
mod meaning;
mod semantic_type;

pub use context::{
    Context, ContextDimension, ContextDimensionKey, ContextError, ContextId, ContextKind,
};
pub use interpretation::{
    Interpretation, InterpretationSource, LexicalConceptMapping, LexicalConceptMappingError,
    LexicalConceptMappingKind, SemanticRelation, SenseReference, SenseReferenceError,
};
pub use meaning::Meaning;
pub use semantic_type::{SemanticType, SemanticTypeMembership, SemanticTypeTarget, SemanticTypes};

#[cfg(test)]
mod tests {
    use super::{
        Context, ContextDimension, ContextDimensionKey, Interpretation, InterpretationSource,
        LexicalConceptMapping, Meaning, SemanticType, SemanticTypeTarget, SemanticTypes,
    };
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionStatus, KnowledgeAssertion,
        Qualifiers,
    };
    use crate::identity::{ConceptId, LexicalFormId};
    use crate::ontology::ClassId;
    use crate::relationship::RelationshipPredicate;

    #[test]
    fn public_semantics_boundary_exposes_all_phase3_foundations() {
        let mut context = Context::ephemeral();
        context
            .insert_dimension(ContextDimension::Language("ar".to_owned()))
            .expect("valid language dimension");

        let interpretation = Interpretation::new(
            InterpretationSource::LexicalForm(
                LexicalFormId::new("lexical-sabr").expect("valid lexical identity"),
            ),
            context,
        );
        assert!(
            interpretation
                .context()
                .contains_dimension(ContextDimensionKey::Language)
        );

        let meaning = Meaning::new("patience");
        assert_eq!(meaning.representation(), "patience");

        let mut types = SemanticTypes::new(SemanticType::class(
            ClassId::new("class-conceptual-object").expect("valid class identity"),
        ));
        types.add_additional(SemanticType::concept(
            ConceptId::new("concept-patience").expect("valid concept identity"),
        ));
        assert_eq!(types.additional().count(), 1);

        let assertion = KnowledgeAssertion::new(
            AssertionObject::LexicalForm(
                LexicalFormId::new("lexical-sabr").expect("valid lexical identity"),
            ),
            RelationshipPredicate::new("expresses").expect("valid predicate"),
            AssertionObject::Concept(
                ConceptId::new("concept-patience").expect("valid concept identity"),
            ),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        );
        let mapping =
            LexicalConceptMapping::direct(assertion).expect("valid lexical-to-concept mapping");

        assert!(matches!(
            mapping.kind(),
            super::LexicalConceptMappingKind::Direct
        ));
        assert_eq!(
            types.primary().target(),
            &SemanticTypeTarget::Class(
                ClassId::new("class-conceptual-object").expect("valid class identity")
            )
        );
    }
}
