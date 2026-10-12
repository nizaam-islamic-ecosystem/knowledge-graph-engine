//! Deterministic rule-evaluation primitives for Phase 8.
//!
//! This module performs local premise matching, binding, condition checks,
//! conclusion grounding, and deduplication. It deliberately does not choose
//! profiles, traverse storage, or orchestrate a reasoning run.

use core::fmt;
use std::collections::BTreeSet;

use crate::assertion::{AssertionObject, AssertionPolarity, AssertionStatus, KnowledgeAssertion};

use super::rule::{Rule, RuleCondition, RuleKind, RulePattern, RuleTerm, VariableBindings};

/// One successful application of a rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleApplication {
    rule_id: crate::identity::RuleId,
    rule_version: super::rule::RuleVersion,
    premises: Vec<KnowledgeAssertion>,
    conclusion: KnowledgeAssertion,
    bindings: VariableBindings,
    depth: usize,
}

impl RuleApplication {
    /// Creates a rule application record.
    #[must_use]
    pub const fn new(
        rule_id: crate::identity::RuleId,
        rule_version: super::rule::RuleVersion,
        premises: Vec<KnowledgeAssertion>,
        conclusion: KnowledgeAssertion,
        bindings: VariableBindings,
        depth: usize,
    ) -> Self {
        Self {
            rule_id,
            rule_version,
            premises,
            conclusion,
            bindings,
            depth,
        }
    }

    /// Returns the rule identity.
    #[must_use]
    pub const fn rule_id(&self) -> &crate::identity::RuleId {
        &self.rule_id
    }

    /// Returns the rule version.
    #[must_use]
    pub const fn rule_version(&self) -> super::rule::RuleVersion {
        self.rule_version
    }

    /// Returns matched premises in deterministic rule order.
    #[must_use]
    pub fn premises(&self) -> &[KnowledgeAssertion] {
        &self.premises
    }

    /// Returns the grounded conclusion.
    #[must_use]
    pub const fn conclusion(&self) -> &KnowledgeAssertion {
        &self.conclusion
    }

    /// Returns runtime variable bindings.
    #[must_use]
    pub const fn bindings(&self) -> &VariableBindings {
        &self.bindings
    }

    /// Returns derivation depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}

/// Local inference failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceError {
    /// A rule failed its structural validation.
    InvalidRule(String),
    /// A pattern cannot be grounded with the available bindings.
    UngroundedConclusion(String),
    /// A rule condition cannot be evaluated.
    InvalidCondition(String),
    /// A conclusion could not be represented by the current semantic model.
    InvalidConclusion(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRule(message) => {
                write!(formatter, "invalid rule during inference: {message}")
            }
            Self::UngroundedConclusion(message) => {
                write!(
                    formatter,
                    "rule conclusion could not be grounded: {message}"
                )
            }
            Self::InvalidCondition(message) => {
                write!(
                    formatter,
                    "rule condition could not be evaluated: {message}"
                )
            }
            Self::InvalidConclusion(message) => {
                write!(
                    formatter,
                    "rule conclusion could not be represented: {message}"
                )
            }
        }
    }
}

impl std::error::Error for InferenceError {}

/// Attempts to match one rule premise against one semantic assertion.
pub fn match_premise(
    pattern: &RulePattern,
    assertion: &KnowledgeAssertion,
    bindings: &VariableBindings,
) -> Option<VariableBindings> {
    let mut next = bindings.clone();
    if !match_term(
        pattern.subject(),
        &RuleTerm::Object(assertion.subject().clone()),
        &mut next,
    ) {
        return None;
    }
    if !match_term(
        pattern.predicate(),
        &RuleTerm::Text(assertion.predicate().name().to_owned()),
        &mut next,
    ) {
        return None;
    }
    if !match_term(
        pattern.object(),
        &RuleTerm::Object(assertion.object().clone()),
        &mut next,
    ) {
        return None;
    }
    Some(next)
}

/// Finds a deterministic set of premise matches for a rule.
pub fn match_premises(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
) -> Vec<(Vec<KnowledgeAssertion>, VariableBindings)> {
    let mut matches = Vec::new();
    let mut selected = Vec::with_capacity(rule.premises().len());
    match_recursive(
        rule,
        assertions,
        0,
        VariableBindings::new(),
        &mut selected,
        &mut matches,
    );
    matches
}

fn match_recursive(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
    premise_index: usize,
    bindings: VariableBindings,
    selected: &mut Vec<KnowledgeAssertion>,
    matches: &mut Vec<(Vec<KnowledgeAssertion>, VariableBindings)>,
) {
    if premise_index == rule.premises().len() {
        if conditions_match(rule, &bindings) {
            matches.push((selected.clone(), bindings));
        }
        return;
    }

    let pattern = &rule.premises()[premise_index];
    for assertion in assertions {
        if let Some(next) = match_premise(pattern, assertion, &bindings) {
            selected.push(assertion.clone());
            match_recursive(rule, assertions, premise_index + 1, next, selected, matches);
            selected.pop();
        }
    }
}

/// Grounds all successful premise matches into rule applications.
pub fn apply_rule(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
    depth: usize,
) -> Result<Vec<RuleApplication>, InferenceError> {
    rule.validate()
        .map_err(|error| InferenceError::InvalidRule(error.to_string()))?;

    let mut applications = Vec::new();
    let matches = match_premises(rule, assertions);
    for (premises, bindings) in matches {
        let conclusion = ground_conclusion(rule.conclusion(), &bindings, premises.first())?;
        applications.push(RuleApplication::new(
            rule.id().clone(),
            rule.version(),
            premises,
            conclusion,
            bindings,
            depth,
        ));
    }
    deduplicate_applications(&mut applications);
    Ok(applications)
}

/// Executes all supplied rules in deterministic rule order.
pub fn apply_rules(
    rules: &[&Rule],
    assertions: &[KnowledgeAssertion],
    depth: usize,
) -> Result<Vec<RuleApplication>, InferenceError> {
    let mut ordered = rules.to_vec();
    ordered.sort_by_key(|left| left.structural_key());

    let mut applications = Vec::new();
    for rule in ordered {
        applications.extend(apply_rule(rule, assertions, depth)?);
    }
    deduplicate_applications(&mut applications);
    Ok(applications)
}

/// Deduplicates applications by semantic conclusion and rule identity.
pub fn deduplicate_applications(applications: &mut Vec<RuleApplication>) {
    let mut seen = BTreeSet::new();
    applications.retain(|application| {
        seen.insert((
            application.conclusion().id().clone(),
            application.rule_id().clone(),
            application.rule_version(),
        ))
    });
}

fn match_term(pattern: &RuleTerm, value: &RuleTerm, bindings: &mut VariableBindings) -> bool {
    match pattern {
        RuleTerm::Variable(variable) => bindings.bind(variable.clone(), value.clone()),
        RuleTerm::Object(expected) => value == &RuleTerm::Object(expected.clone()),
        RuleTerm::Text(expected) => value == &RuleTerm::Text(expected.clone()),
    }
}

fn conditions_match(rule: &Rule, bindings: &VariableBindings) -> bool {
    rule.conditions()
        .iter()
        .all(|condition| evaluate_condition(condition, bindings))
}

fn evaluate_condition(condition: &RuleCondition, bindings: &VariableBindings) -> bool {
    match condition {
        RuleCondition::Equal(left, right) => {
            resolve_term(left, bindings) == resolve_term(right, bindings)
        }
        RuleCondition::NotEqual(left, right) => {
            resolve_term(left, bindings) != resolve_term(right, bindings)
        }
        RuleCondition::PredicateIs(expected) => bindings
            .values()
            .values()
            .any(|value| matches!(value, RuleTerm::Text(actual) if actual == expected)),
        RuleCondition::ObjectKind { variable, kind } => bindings
            .get(variable)
            .and_then(object_kind)
            .is_some_and(|actual| actual == kind),
    }
}

fn resolve_term(term: &RuleTerm, bindings: &VariableBindings) -> Option<RuleTerm> {
    match term {
        RuleTerm::Variable(variable) => bindings.get(variable).cloned(),
        other => Some(other.clone()),
    }
}

fn object_kind(value: &RuleTerm) -> Option<&'static str> {
    match value {
        RuleTerm::Object(AssertionObject::Entity(_)) => Some("entity"),
        RuleTerm::Object(AssertionObject::Concept(_)) => Some("concept"),
        RuleTerm::Object(AssertionObject::Source(_)) => Some("source"),
        RuleTerm::Object(AssertionObject::Reference(_)) => Some("reference"),
        RuleTerm::Object(AssertionObject::LexicalForm(_)) => Some("lexical_form"),
        RuleTerm::Object(AssertionObject::Mention(_)) => Some("mention"),
        _ => None,
    }
}

fn ground_conclusion(
    pattern: &RulePattern,
    bindings: &VariableBindings,
    template: Option<&KnowledgeAssertion>,
) -> Result<KnowledgeAssertion, InferenceError> {
    let subject = ground_object(pattern.subject(), bindings)?;
    let predicate = ground_text(pattern.predicate(), bindings)?;
    let object = ground_object(pattern.object(), bindings)?;
    let predicate = crate::assertion::AssertionPredicate::new(predicate)
        .map_err(|error| InferenceError::InvalidConclusion(error.to_string()))?;

    let (context, qualifiers, status, polarity) = template
        .map(|assertion| {
            (
                assertion.context().clone(),
                assertion.qualifiers().clone(),
                AssertionStatus::Provisional,
                assertion.polarity(),
            )
        })
        .unwrap_or_else(|| {
            (
                crate::assertion::AssertionContext::new(),
                crate::assertion::Qualifiers::new(),
                AssertionStatus::Provisional,
                AssertionPolarity::Positive,
            )
        });

    Ok(KnowledgeAssertion::new(
        subject, predicate, object, context, qualifiers, status, polarity,
    ))
}

fn ground_object(
    term: &RuleTerm,
    bindings: &VariableBindings,
) -> Result<AssertionObject, InferenceError> {
    match resolve_term(term, bindings) {
        Some(RuleTerm::Object(object)) => Ok(object),
        Some(other) => Err(InferenceError::UngroundedConclusion(format!(
            "expected semantic object, got {other:?}"
        ))),
        None => Err(InferenceError::UngroundedConclusion(
            "missing object binding".to_owned(),
        )),
    }
}

fn ground_text(term: &RuleTerm, bindings: &VariableBindings) -> Result<String, InferenceError> {
    match resolve_term(term, bindings) {
        Some(RuleTerm::Text(value)) => Ok(value),
        Some(other) => Err(InferenceError::UngroundedConclusion(format!(
            "expected textual predicate, got {other:?}"
        ))),
        None => Err(InferenceError::UngroundedConclusion(
            "missing predicate binding".to_owned(),
        )),
    }
}

/// Returns the supported initial rule families.
#[must_use]
pub const fn supports_rule_kind(kind: RuleKind) -> bool {
    matches!(
        kind,
        RuleKind::Inverse
            | RuleKind::Symmetric
            | RuleKind::Transitive
            | RuleKind::TypeInheritance
            | RuleKind::PropertyInheritance
            | RuleKind::Composition
            | RuleKind::Conditional
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertion::{AssertionContext, AssertionObject, AssertionPredicate, Qualifiers};
    use crate::identity::EntityId;
    use crate::identity::RuleId;
    use crate::reasoning::rule::{RuleMetadata, RuleScope, RuleTerm, RuleVersion};

    fn assertion(subject: &str, predicate: &str, object: &str) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new(subject).expect("valid subject")),
            AssertionPredicate::new(predicate).expect("valid predicate"),
            AssertionObject::Entity(EntityId::new(object).expect("valid object")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    fn inverse_rule() -> Rule {
        Rule::new(
            RuleId::new("inverse").expect("valid rule id"),
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
            RuleMetadata::new("inverse", "inverse rule").expect("valid metadata"),
        )
        .expect("rule should be valid")
    }

    #[test]
    fn premise_matching_produces_typed_bindings() {
        let rule = inverse_rule();
        let input = assertion("a", "parent-of", "b");
        let matches = match_premises(&rule, &[input]);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].1.get("x").is_some());
        assert!(matches[0].1.get("y").is_some());
    }

    #[test]
    fn inverse_rule_grounding_is_deterministic() {
        let rule = inverse_rule();
        let input = assertion("a", "parent-of", "b");
        let applications = apply_rule(&rule, &[input], 1).expect("rule should apply");
        assert_eq!(applications.len(), 1);
        assert_eq!(applications[0].conclusion().predicate().name(), "child-of");
        assert_eq!(
            applications[0].conclusion().subject(),
            &AssertionObject::Entity(EntityId::new("b").unwrap())
        );
    }
}
