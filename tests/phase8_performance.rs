//! Level 3 Phase 8 performance-baseline tests.
//!
//! These tests intentionally establish small deterministic baselines rather
//! than imposing optimization-oriented throughput targets.

use std::time::Instant;

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::identity::{EntityId, RuleId};
use nizaam_knowledge_graph::query::ReasoningProfile;
use nizaam_knowledge_graph::reasoning::{
    Reasoner, ReasoningExecutionStatus, ReasoningInput, Rule, RuleKind, RuleMetadata, RulePattern,
    RuleRegistry, RuleScope, RuleTerm, RuleVersion, apply_rule,
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

fn inverse_rule() -> Rule {
    Rule::new(
        RuleId::new("performance-inverse").expect("valid rule id"),
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
        RuleMetadata::new("performance", "small baseline rule").expect("metadata"),
    )
    .expect("rule should validate")
}

fn snapshot() -> Snapshot {
    Snapshot::new(
        KgVersion::initial(),
        vec![RepositoryRecord::new(
            ChangeTarget::Entity(EntityId::new("performance-root").expect("valid id")),
            Document::from_slice(b"performance-root"),
        )],
    )
    .expect("snapshot should be valid")
}

#[test]
fn rule_matching_small_baseline_is_finite_and_produces_expected_work() {
    let rule = inverse_rule();
    let assertions: Vec<_> = (0..100)
        .map(|index| assertion(&format!("entity-{index}"), "parent-of", "root"))
        .collect();

    let start = Instant::now();
    let applications = apply_rule(&rule, &assertions, 0).expect("matching should succeed");
    let elapsed = start.elapsed();

    assert_eq!(applications.len(), 100);
    assert!(elapsed < std::time::Duration::from_secs(5));
}

#[test]
fn bounded_reasoning_small_baseline_remains_within_a_finite_execution_window() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule();
    let id = rule.id().clone();
    registry.register_built_in(rule).expect("register");
    registry.approve(&id).expect("approve");
    registry
        .version(&id, RuleVersion::default())
        .expect("version");
    registry.activate(&id).expect("activate");

    let assertions: Vec<_> = (0..50)
        .map(|index| assertion(&format!("entity-{index}"), "parent-of", "root"))
        .collect();
    let snapshot = snapshot();
    let input = ReasoningInput::new(&snapshot, &assertions, None);

    let start = Instant::now();
    let result = Reasoner::new(registry)
        .reason(&input, ReasoningProfile::Basic)
        .expect("reasoning should succeed");
    let elapsed = start.elapsed();

    assert!(!result.derived().is_empty());
    assert!(result.rule_applications() <= 1_000);
    assert!(result.expanded_assertions() <= 1_000);
    assert!(matches!(
        result.status(),
        ReasoningExecutionStatus::Complete | ReasoningExecutionStatus::Incomplete
    ));
    assert!(elapsed < std::time::Duration::from_secs(5));
}

#[test]
fn derived_result_generation_has_a_small_deterministic_baseline() {
    let rule = inverse_rule();
    let source = assertion("a", "parent-of", "b");
    let snapshot = snapshot();

    let start = Instant::now();
    let applications =
        apply_rule(&rule, std::slice::from_ref(&source), 1).expect("inference should succeed");
    let elapsed = start.elapsed();

    let mut derived = nizaam_knowledge_graph::reasoning::DerivedKnowledgeSet::new();
    for application in &applications {
        derived.insert_application(application, &snapshot, ReasoningProfile::Basic);
    }

    assert_eq!(derived.len(), 1);
    assert_eq!(
        derived
            .values()
            .next()
            .expect("derived value")
            .traces()
            .len(),
        1
    );
    assert!(elapsed < std::time::Duration::from_secs(1));
}
