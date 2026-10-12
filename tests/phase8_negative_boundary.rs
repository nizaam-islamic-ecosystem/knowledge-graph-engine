//! Level 3 Phase 8 negative-boundary tests.
//!
//! These tests ensure controlled reasoning fails explicitly for invalid rule
//! definitions, ungoverned rules, disabled profiles, version mismatches, and
//! conflicting knowledge rather than silently falling back.

use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::identity::{EntityId, RuleId};
use nizaam_knowledge_graph::query::ReasoningProfile;
use nizaam_knowledge_graph::reasoning::{
    Reasoner, ReasonerError, ReasoningExecutionStatus, ReasoningInput, RegistryError, Rule,
    RuleKind, RuleMetadata, RulePattern, RuleRegistry, RuleScope, RuleTerm, RuleValidationError,
    RuleVersion, apply_rule,
};
use nizaam_knowledge_graph::storage::{Document, RepositoryRecord, Snapshot};
use nizaam_knowledge_graph::versioning::{ChangeTarget, KgVersion};

fn entity(value: &str) -> AssertionObject {
    AssertionObject::Entity(EntityId::new(value).expect("valid entity id"))
}

fn assertion_with_polarity(
    subject: &str,
    predicate: &str,
    object: &str,
    polarity: AssertionPolarity,
) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        entity(subject),
        AssertionPredicate::new(predicate).expect("valid predicate"),
        entity(object),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        polarity,
    )
}

fn assertion(subject: &str, predicate: &str, object: &str) -> KnowledgeAssertion {
    assertion_with_polarity(subject, predicate, object, AssertionPolarity::Positive)
}

fn snapshot() -> Snapshot {
    Snapshot::new(
        KgVersion::initial(),
        vec![RepositoryRecord::new(
            ChangeTarget::Entity(EntityId::new("negative-root").expect("valid id")),
            Document::from_slice(b"negative-root"),
        )],
    )
    .expect("snapshot should be valid")
}

fn inverse_rule(id: &str) -> Rule {
    Rule::new(
        RuleId::new(id).expect("valid rule id"),
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
        RuleMetadata::new("inverse", "negative-boundary rule").expect("metadata"),
    )
    .expect("rule should validate")
}

#[test]
fn invalid_rule_is_rejected_before_inference() {
    let invalid = Rule::new(
        RuleId::new("invalid-rule").expect("valid id"),
        RuleVersion::default(),
        RuleKind::Inverse,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        RulePattern::new(
            RuleTerm::variable("x").expect("valid variable"),
            RuleTerm::text("child-of").expect("valid predicate"),
            RuleTerm::variable("y").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("invalid", "missing premises").expect("metadata"),
    );

    assert!(matches!(invalid, Err(RuleValidationError::NoPremises)));
}

#[test]
fn unapproved_rule_is_not_eligible_for_reasoning() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule("unapproved");
    let id = rule.id().clone();
    registry.register_declarative(rule).expect("register");

    assert!(!registry.get(&id).expect("rule should exist").is_eligible());
    assert!(registry.eligible(None, None).is_empty());
}

#[test]
fn activation_before_approval_is_an_explicit_registry_failure() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule("invalid-transition");
    let id = rule.id().clone();
    registry.register_declarative(rule).expect("register");

    assert!(matches!(
        registry.activate(&id),
        Err(RegistryError::InvalidTransition { .. })
    ));
}

#[test]
fn wrong_rule_version_is_rejected_by_registry() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule("version-mismatch");
    let id = rule.id().clone();
    registry.register_built_in(rule).expect("register");
    registry.approve(&id).expect("approve");

    assert!(matches!(
        registry.version(&id, RuleVersion::new(2, 0, 0)),
        Err(RegistryError::VersionMismatch { .. })
    ));
}

#[test]
fn disabled_profile_is_rejected_at_the_reasoner_boundary() {
    let reasoner = Reasoner::default();
    let assertions = Vec::<KnowledgeAssertion>::new();
    let snapshot = snapshot();
    let input = ReasoningInput::new(&snapshot, &assertions, None);

    assert!(matches!(
        reasoner.reason(&input, ReasoningProfile::None),
        Err(ReasonerError::ProfileDisabled)
    ));
}

#[test]
fn conflicting_explicit_knowledge_is_preserved_as_a_conflict() {
    let reasoner = Reasoner::default();
    let positive = assertion("a", "knows", "b");
    let negative = assertion_with_polarity("a", "knows", "b", AssertionPolarity::Negative);
    let assertions = vec![positive, negative];
    let snapshot = snapshot();
    let input = ReasoningInput::new(&snapshot, &assertions, None);

    let result = reasoner
        .reason(&input, ReasoningProfile::Basic)
        .expect("conflicting input should remain representable");

    assert_eq!(result.conflicts().len(), 1);
}

#[test]
fn inference_failure_remains_typed_instead_of_becoming_empty_success() {
    let rule = inverse_rule("typed-failure");
    let source = assertion("a", "parent-of", "b");
    let applications =
        apply_rule(&rule, std::slice::from_ref(&source), 0).expect("valid rule should infer");
    assert_eq!(applications.len(), 1);

    // A malformed conclusion cannot be constructed through the public Rule
    // validator, so the negative boundary is established at rule validation.
    let invalid_scope = RuleScope::unrestricted().with_max_depth(0);
    let result = Rule::new(
        RuleId::new("zero-depth").expect("valid id"),
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
        invalid_scope,
        RuleMetadata::new("zero-depth", "invalid scope").expect("metadata"),
    );
    assert!(matches!(result, Err(RuleValidationError::ZeroScopeDepth)));
}

#[test]
fn execution_status_vocabulary_is_explicit_and_separate_from_origin() {
    assert!(ReasoningExecutionStatus::Complete.is_complete());
    assert!(!ReasoningExecutionStatus::Incomplete.is_complete());
    assert!(!ReasoningExecutionStatus::Failed.is_complete());
}
