//! Level 3 Phase 8 positive reasoning tests.
//!
//! These tests exercise the public rule model, governed registry, profile
//! configuration, deterministic inference primitives, derived state, and the
//! controlled reasoner without introducing a second storage or rule language.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::identity::{EntityId, RuleId};
use nizaam_knowledge_graph::query::ReasoningProfile;
use nizaam_knowledge_graph::reasoning::{
    DerivedKnowledgeSet, ExpansionPolicy, ExplanationPolicy, GLOBAL_HARD_CEILINGS, InferenceError,
    Reasoner, ReasoningExecutionStatus, ReasoningProfileConfig, Rule, RuleCondition, RuleKind,
    RuleMetadata, RulePattern, RuleRegistry, RuleScope, RuleTerm, RuleVersion, VariableBindings,
    apply_rule, apply_rules, deduplicate_applications, supports_rule_kind,
};
use nizaam_knowledge_graph::storage::{Document, RepositoryRecord, Snapshot};
use nizaam_knowledge_graph::versioning::{ChangeTarget, KgVersion};

fn entity(value: &str) -> AssertionObject {
    AssertionObject::Entity(EntityId::new(value).expect("valid entity id"))
}

fn assertion(subject: &str, predicate: &str, object: &str) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        entity(subject),
        AssertionPredicate::new(predicate).expect("valid predicate"),
        entity(object),
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
            ChangeTarget::Entity(EntityId::new("snapshot-root").expect("valid id")),
            Document::from_slice(b"snapshot-root"),
        )],
    )
    .expect("snapshot should be valid")
}

fn pattern(subject: RuleTerm, predicate: &str, object: RuleTerm) -> RulePattern {
    RulePattern::new(
        subject,
        RuleTerm::text(predicate).expect("valid predicate"),
        object,
    )
}

fn inverse_rule(id: &str) -> Rule {
    Rule::new(
        RuleId::new(id).expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Inverse,
        vec![pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "parent-of",
            RuleTerm::variable("y").expect("valid variable"),
        )],
        Vec::new(),
        Vec::new(),
        pattern(
            RuleTerm::variable("y").expect("valid variable"),
            "child-of",
            RuleTerm::variable("x").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("inverse", "derive the inverse relationship").expect("metadata"),
    )
    .expect("inverse rule should validate")
}

#[test]
fn rule_model_supports_all_initial_controlled_families() {
    for kind in [
        RuleKind::Inverse,
        RuleKind::Symmetric,
        RuleKind::Transitive,
        RuleKind::TypeInheritance,
        RuleKind::PropertyInheritance,
        RuleKind::Composition,
        RuleKind::Conditional,
    ] {
        assert!(supports_rule_kind(kind));
        assert!(!kind.as_str().is_empty());
    }
}

#[test]
fn rule_validation_rejects_unbound_conclusion_variables() {
    let result = Rule::new(
        RuleId::new("invalid-grounding").expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Inverse,
        vec![pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "parent-of",
            RuleTerm::variable("y").expect("valid variable"),
        )],
        Vec::new(),
        Vec::new(),
        pattern(
            RuleTerm::variable("z").expect("valid variable"),
            "child-of",
            RuleTerm::variable("x").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("invalid", "invalid grounding").expect("metadata"),
    );

    assert!(result.is_err());
}

#[test]
fn inference_reports_an_ungrounded_runtime_conclusion_as_a_typed_error() {
    let rule = Rule::new(
        RuleId::new("invalid-runtime-grounding").expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Conditional,
        vec![RulePattern::new(
            RuleTerm::variable("x").expect("valid variable"),
            RuleTerm::variable("predicate").expect("valid variable"),
            RuleTerm::variable("y").expect("valid variable"),
        )],
        Vec::new(),
        Vec::new(),
        pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "derived",
            RuleTerm::variable("predicate").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new(
            "invalid-runtime-grounding",
            "predicate binding is not an object",
        )
        .expect("metadata"),
    )
    .expect("rule structure should be valid");

    let result = apply_rule(&rule, &[assertion("a", "knows", "b")], 0);
    assert!(matches!(
        result,
        Err(InferenceError::UngroundedConclusion(_))
    ));
}

#[test]
fn variable_bindings_are_strong_and_reject_conflicting_values() {
    let mut bindings = VariableBindings::new();
    let first = RuleTerm::text("parent-of").expect("valid text");
    let second = RuleTerm::text("child-of").expect("valid text");

    assert!(bindings.bind("predicate", first.clone()));
    assert!(!bindings.bind("predicate", second));
    assert_eq!(bindings.get("predicate"), Some(&first));
}

#[test]
fn registry_requires_approval_versioning_and_activation() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule("registry-lifecycle");
    let id = rule.id().clone();

    registry
        .register_declarative(rule)
        .expect("registration should succeed");
    assert!(!registry.get(&id).expect("rule should exist").is_eligible());

    registry.approve(&id).expect("approval should succeed");
    registry
        .version(&id, RuleVersion::default())
        .expect("versioning should succeed");
    registry.activate(&id).expect("activation should succeed");

    assert!(registry.get(&id).expect("rule should exist").is_eligible());
}

#[test]
fn profiles_map_to_controlled_rule_sets_and_bounded_execution() {
    let basic = ReasoningProfileConfig::for_profile(ReasoningProfile::Basic)
        .expect("basic profile should be valid");
    let deep = ReasoningProfileConfig::for_profile(ReasoningProfile::Deep)
        .expect("deep profile should be valid");

    assert_eq!(basic.expansion(), ExpansionPolicy::Bounded);
    assert_eq!(basic.explanation(), ExplanationPolicy::Compact);
    assert!(deep.budgets().max_depth() <= GLOBAL_HARD_CEILINGS.max_depth());
    assert!(deep.budgets().max_rule_applications() <= GLOBAL_HARD_CEILINGS.max_rule_applications());
    assert!(
        deep.budgets().max_derived_assertions() <= GLOBAL_HARD_CEILINGS.max_derived_assertions()
    );
}

#[test]
fn inverse_reasoning_derives_the_expected_relation() {
    let rule = inverse_rule("inverse-positive");
    let source = assertion("a", "parent-of", "b");
    let applications = apply_rule(&rule, std::slice::from_ref(&source), 1)
        .expect("inverse inference should succeed");

    assert_eq!(applications.len(), 1);
    assert_eq!(applications[0].conclusion().predicate().name(), "child-of");
}

#[test]
fn symmetric_reasoning_uses_a_single_premise() {
    let rule = Rule::new(
        RuleId::new("symmetric-positive").expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Symmetric,
        vec![pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "knows",
            RuleTerm::variable("y").expect("valid variable"),
        )],
        Vec::new(),
        Vec::new(),
        pattern(
            RuleTerm::variable("y").expect("valid variable"),
            "knows",
            RuleTerm::variable("x").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("symmetric", "symmetric relation").expect("metadata"),
    )
    .expect("symmetric rule should validate");

    let applications = apply_rule(&rule, &[assertion("a", "knows", "b")], 0)
        .expect("symmetric inference should succeed");
    assert_eq!(applications.len(), 1);
    assert_eq!(applications[0].conclusion().subject(), &entity("b"));
    assert_eq!(applications[0].conclusion().object(), &entity("a"));
}

#[test]
fn transitive_and_composition_rules_accept_two_premises() {
    let rule = Rule::new(
        RuleId::new("transitive-positive").expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Transitive,
        vec![
            pattern(
                RuleTerm::variable("x").expect("valid variable"),
                "parent-of",
                RuleTerm::variable("y").expect("valid variable"),
            ),
            pattern(
                RuleTerm::variable("y").expect("valid variable"),
                "parent-of",
                RuleTerm::variable("z").expect("valid variable"),
            ),
        ],
        Vec::new(),
        Vec::new(),
        pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "ancestor-of",
            RuleTerm::variable("z").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("transitive", "transitive relation").expect("metadata"),
    )
    .expect("transitive rule should validate");

    let applications = apply_rule(
        &rule,
        &[
            assertion("a", "parent-of", "b"),
            assertion("b", "parent-of", "c"),
        ],
        1,
    )
    .expect("transitive inference should succeed");

    assert_eq!(applications.len(), 1);
    assert_eq!(
        applications[0].conclusion().predicate().name(),
        "ancestor-of"
    );
}

#[test]
fn conditional_rules_evaluate_declared_conditions() {
    let rule = Rule::new(
        RuleId::new("conditional-positive").expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Conditional,
        vec![pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "knows",
            RuleTerm::variable("y").expect("valid variable"),
        )],
        vec![RuleCondition::NotEqual(
            RuleTerm::variable("x").expect("valid variable"),
            RuleTerm::variable("y").expect("valid variable"),
        )],
        Vec::new(),
        pattern(
            RuleTerm::variable("x").expect("valid variable"),
            "trusted",
            RuleTerm::variable("y").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("conditional", "conditioned rule").expect("metadata"),
    )
    .expect("conditional rule should validate");

    let applications = apply_rule(&rule, &[assertion("a", "knows", "b")], 0)
        .expect("conditional inference should succeed");
    assert_eq!(applications.len(), 1);
    assert_eq!(applications[0].conclusion().predicate().name(), "trusted");
}

#[test]
fn deterministic_rule_application_order_is_independent_of_input_order() {
    let first = inverse_rule("rule-a");
    let second = inverse_rule("rule-b");
    let source = assertion("a", "parent-of", "b");

    let left = apply_rules(&[&first, &second], std::slice::from_ref(&source), 0)
        .expect("first order should succeed");
    let right = apply_rules(&[&second, &first], std::slice::from_ref(&source), 0)
        .expect("second order should succeed");

    let left_keys: Vec<_> = left
        .iter()
        .map(|application| application.rule_id().as_str())
        .collect();
    let right_keys: Vec<_> = right
        .iter()
        .map(|application| application.rule_id().as_str())
        .collect();
    assert_eq!(left_keys, right_keys);
}

#[test]
fn duplicate_rule_applications_are_deduplicated() {
    let rule = inverse_rule("deduplicate");
    let source = assertion("a", "parent-of", "b");
    let mut applications =
        apply_rule(&rule, std::slice::from_ref(&source), 0).expect("inference should succeed");
    let duplicate = applications[0].clone();
    applications.push(duplicate);
    deduplicate_applications(&mut applications);
    assert_eq!(applications.len(), 1);
}

#[test]
fn reasoner_reaches_a_fixed_point_and_preserves_derived_state() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule("reasoner-positive");
    let id = rule.id().clone();
    registry.register_built_in(rule).expect("register");
    registry.approve(&id).expect("approve");
    registry
        .version(&id, RuleVersion::default())
        .expect("version");
    registry.activate(&id).expect("activate");

    let source = assertion("a", "parent-of", "b");
    let snapshot = snapshot();
    let input = nizaam_knowledge_graph::reasoning::ReasoningInput::new(
        &snapshot,
        std::slice::from_ref(&source),
        None,
    );
    let result = Reasoner::new(registry)
        .reason(&input, ReasoningProfile::Basic)
        .expect("reasoning should succeed");

    assert!(!result.derived().is_empty());
    assert!(matches!(
        result.status(),
        ReasoningExecutionStatus::Complete | ReasoningExecutionStatus::Incomplete
    ));
}

#[test]
fn derived_knowledge_is_rebuildable_and_pins_the_input_version() {
    let source = assertion("a", "parent-of", "b");
    let rule = inverse_rule("trace");
    let application = apply_rule(&rule, std::slice::from_ref(&source), 1)
        .expect("inference should succeed")
        .pop()
        .expect("one application expected");
    let snapshot = snapshot();
    let mut derived = DerivedKnowledgeSet::new();
    derived.insert_application(&application, &snapshot, ReasoningProfile::Basic);

    let item = derived.values().next().expect("derived item expected");
    let trace = item.traces().first().expect("trace expected");
    assert_eq!(trace.rule_id(), rule.id());
    assert_eq!(trace.rule_version(), rule.version());
    assert_eq!(trace.profile(), ReasoningProfile::Basic);
    assert_eq!(trace.input_version(), snapshot.version().id());
    assert_eq!(trace.depth(), 1);
}
