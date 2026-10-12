//! Controlled Knowledge Graph reasoning subsystem for Phase 8.
//!
//! The module wires the semantic rule model, governed registry, profile
//! interpretation, deterministic inference primitives, rebuildable derived
//! state, and main reasoner. Canonical storage remains owned by Phase 7 and
//! query orchestration remains owned by Phase 6.

mod derived;
mod inference;
mod profile;
mod reasoner;
mod registry;
mod rule;

pub use derived::{
    DerivationPath, DerivationStep, DerivationTrace, DerivedKnowledge, DerivedKnowledgeSet,
};
pub use inference::{
    InferenceError, RuleApplication, apply_rule, apply_rules, deduplicate_applications,
    match_premise, match_premises, supports_rule_kind,
};
pub use profile::{
    ExpansionPolicy, ExplanationPolicy, GLOBAL_HARD_CEILINGS, ProfileValidationError,
    ReasoningBudgets, ReasoningProfileConfig,
};
pub use reasoner::{
    Reasoner, ReasonerError, ReasoningConflict, ReasoningExecutionStatus, ReasoningInput,
    ReasoningResult,
};
pub use registry::{RegisteredRule, RegistryError, RuleLifecycle, RuleOrigin, RuleRegistry};
pub use rule::{
    Rule, RuleBinding, RuleCondition, RuleKind, RuleMetadata, RulePattern, RuleScope, RuleTerm,
    RuleValidationError, RuleVersion, VariableBindings,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::identity::{EntityId, RuleId};
    use crate::storage::{RepositoryRecord, Snapshot};
    use crate::versioning::{ChangeTarget, KgVersion};

    fn inverse_rule() -> Rule {
        Rule::new(
            RuleId::new("phase8-level2-inverse").expect("valid rule id"),
            RuleVersion::default(),
            RuleKind::Inverse,
            vec![RulePattern::new(
                RuleTerm::variable("x").expect("valid variable"),
                RuleTerm::text("parent-of").expect("valid predicate"),
                RuleTerm::variable("y").expect("valid variable"),
            )],
            Vec::new(),
            Vec::new(),
            RulePattern::new(
                RuleTerm::variable("y").expect("valid variable"),
                RuleTerm::text("child-of").expect("valid predicate"),
                RuleTerm::variable("x").expect("valid variable"),
            ),
            RuleScope::unrestricted(),
            RuleMetadata::new("inverse", "level 2 integration rule").expect("valid metadata"),
        )
        .expect("rule should validate")
    }

    fn assertion() -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-a").expect("valid id")),
            AssertionPredicate::new("parent-of").expect("valid predicate"),
            AssertionObject::Entity(EntityId::new("entity-b").expect("valid id")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    fn snapshot() -> Snapshot {
        Snapshot::new(
            KgVersion::initial(),
            vec![RepositoryRecord::new(
                ChangeTarget::Entity(EntityId::new("entity-a").expect("valid id")),
                crate::storage::Document::from_slice(b"entity-a"),
            )],
        )
        .expect("snapshot should succeed")
    }

    #[test]
    fn reasoning_module_level2_connects_rule_registry_inference_derived_and_reasoner() {
        let rule = inverse_rule();
        let id = rule.id().clone();

        let mut registry = RuleRegistry::new();
        registry
            .register_built_in(rule)
            .expect("rule registration should succeed");
        registry.approve(&id).expect("rule approval should succeed");
        registry
            .version(&id, RuleVersion::default())
            .expect("rule versioning should succeed");
        registry
            .activate(&id)
            .expect("rule activation should succeed");

        let source = assertion();
        let direct = apply_rules(
            &[registry.get(&id).expect("registered rule").rule()],
            std::slice::from_ref(&source),
            1,
        )
        .expect("inference should succeed");
        assert_eq!(direct.len(), 1);

        let snapshot = snapshot();
        let input = ReasoningInput::new(&snapshot, std::slice::from_ref(&source), None);
        let result = Reasoner::new(registry)
            .reason(&input, crate::query::ReasoningProfile::Basic)
            .expect("reasoner should execute");

        assert!(
            result
                .derived()
                .values()
                .any(|value| { value.assertion().predicate().name() == "child-of" })
        );
        assert_eq!(result.profile(), crate::query::ReasoningProfile::Basic);
    }

    #[test]
    fn reasoning_profile_and_result_status_remain_separate() {
        let profile = ReasoningProfileConfig::for_profile(crate::query::ReasoningProfile::Deep)
            .expect("deep profile should validate");
        assert_eq!(profile.profile(), crate::query::ReasoningProfile::Deep);
        assert!(matches!(
            ReasoningExecutionStatus::Complete,
            ReasoningExecutionStatus::Complete
        ));
        assert!(matches!(
            ReasoningExecutionStatus::Incomplete,
            ReasoningExecutionStatus::Incomplete
        ));
        assert!(matches!(
            ReasoningExecutionStatus::Failed,
            ReasoningExecutionStatus::Failed
        ));
    }
}
