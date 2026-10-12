//! Deterministic rule-evaluation primitives for Phase 8.
//!
//! This module performs local premise matching, binding, condition checks,
//! conclusion grounding, and deduplication. It deliberately does not choose
//! profiles, traverse storage, or orchestrate a reasoning run.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

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
    if assertion.polarity() != AssertionPolarity::Positive
        || !matches!(
            assertion.status(),
            AssertionStatus::Accepted | AssertionStatus::Known
        )
    {
        return None;
    }

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
    let predicate_index = build_predicate_index(assertions);
    match_recursive(
        rule,
        assertions,
        &predicate_index,
        0,
        VariableBindings::new(),
        &mut selected,
        &mut matches,
        None,
    );
    matches
}

#[derive(Clone, Copy, Debug)]
struct InferenceBudget {
    deadline: Instant,
    remaining_applications: usize,
}

impl InferenceBudget {
    fn exhausted(self) -> bool {
        Instant::now() >= self.deadline || self.remaining_applications == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MatchOutcome {
    exhausted: bool,
}

fn build_predicate_index(assertions: &[KnowledgeAssertion]) -> BTreeMap<String, Vec<usize>> {
    let mut index = BTreeMap::new();
    for (position, assertion) in assertions.iter().enumerate() {
        if assertion.polarity() == AssertionPolarity::Positive
            && matches!(
                assertion.status(),
                AssertionStatus::Accepted | AssertionStatus::Known
            )
        {
            index
                .entry(assertion.predicate().name().to_owned())
                .or_insert_with(Vec::new)
                .push(position);
        }
    }
    index
}

#[allow(clippy::too_many_arguments)]
fn match_recursive(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
    predicate_index: &BTreeMap<String, Vec<usize>>,
    premise_index: usize,
    bindings: VariableBindings,
    selected: &mut Vec<KnowledgeAssertion>,
    matches: &mut Vec<(Vec<KnowledgeAssertion>, VariableBindings)>,
    budget: Option<&mut InferenceBudget>,
) -> MatchOutcome {
    if let Some(budget) = budget.as_ref()
        && budget.exhausted()
    {
        return MatchOutcome { exhausted: true };
    }

    if premise_index == rule.premises().len() {
        if conditions_match(rule, selected, &bindings) {
            if let Some(budget) = budget {
                if budget.exhausted() {
                    return MatchOutcome { exhausted: true };
                }
                budget.remaining_applications = budget.remaining_applications.saturating_sub(1);
            }
            matches.push((selected.clone(), bindings));
        }
        return MatchOutcome { exhausted: false };
    }

    let pattern = &rule.premises()[premise_index];
    let positions = match pattern.predicate() {
        RuleTerm::Text(predicate) => predicate_index.get(predicate).cloned().unwrap_or_default(),
        RuleTerm::Object(_) => Vec::new(),
        RuleTerm::Variable(_) => (0..assertions.len()).collect(),
    };

    match_recursive_positions(
        rule,
        assertions,
        predicate_index,
        premise_index,
        bindings,
        selected,
        matches,
        positions,
        budget,
    )
}

#[allow(clippy::too_many_arguments)]
fn match_recursive_positions(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
    predicate_index: &BTreeMap<String, Vec<usize>>,
    premise_index: usize,
    bindings: VariableBindings,
    selected: &mut Vec<KnowledgeAssertion>,
    matches: &mut Vec<(Vec<KnowledgeAssertion>, VariableBindings)>,
    positions: Vec<usize>,
    mut budget: Option<&mut InferenceBudget>,
) -> MatchOutcome {
    let pattern = &rule.premises()[premise_index];
    for position in positions {
        if let Some(ref mut budget) = budget
            && budget.exhausted()
        {
            return MatchOutcome { exhausted: true };
        }

        let assertion = &assertions[position];
        if let Some(next) = match_premise(pattern, assertion, &bindings) {
            selected.push(assertion.clone());
            let outcome = match_recursive(
                rule,
                assertions,
                predicate_index,
                premise_index + 1,
                next,
                selected,
                matches,
                budget.as_deref_mut(),
            );
            selected.pop();
            if outcome.exhausted {
                return outcome;
            }
        }
    }
    MatchOutcome { exhausted: false }
}

fn match_premises_bounded(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
    budget: &mut InferenceBudget,
) -> (Vec<(Vec<KnowledgeAssertion>, VariableBindings)>, bool) {
    let mut matches = Vec::new();
    let mut selected = Vec::with_capacity(rule.premises().len());
    let predicate_index = build_predicate_index(assertions);
    let outcome = match_recursive(
        rule,
        assertions,
        &predicate_index,
        0,
        VariableBindings::new(),
        &mut selected,
        &mut matches,
        Some(budget),
    );
    (matches, outcome.exhausted)
}

fn apply_rule_with_budget(
    rule: &Rule,
    assertions: &[KnowledgeAssertion],
    depth: usize,
    budget: &mut InferenceBudget,
) -> Result<(Vec<RuleApplication>, bool), InferenceError> {
    rule.validate()
        .map_err(|error| InferenceError::InvalidRule(error.to_string()))?;

    let (matches, exhausted) = match_premises_bounded(rule, assertions, budget);
    let mut applications = Vec::new();
    for (premises, bindings) in matches {
        let conclusion = ground_conclusion(rule.conclusion(), &bindings, &premises)?;
        if !rule.scope().allows_predicate(conclusion.predicate().name())
            || !conclusion_object_allowed(rule, &conclusion)
            || !bindings_object_kinds_allowed(rule, &bindings)
            || !bindings_match_declared_types(rule, &bindings)
        {
            continue;
        }
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
    Ok((applications, exhausted))
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
        let conclusion = ground_conclusion(rule.conclusion(), &bindings, &premises)?;
        if !rule.scope().allows_predicate(conclusion.predicate().name())
            || !conclusion_object_allowed(rule, &conclusion)
            || !bindings_object_kinds_allowed(rule, &bindings)
            || !bindings_match_declared_types(rule, &bindings)
        {
            continue;
        }
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

/// Executes rules while enforcing the caller's inference work budget.
pub(crate) fn apply_rules_with_budget(
    rules: &[&Rule],
    assertions: &[KnowledgeAssertion],
    depth: usize,
    deadline: Instant,
    remaining_applications: usize,
) -> Result<(Vec<RuleApplication>, usize, bool), InferenceError> {
    let mut ordered = rules.to_vec();
    ordered.sort_by_key(|left| left.structural_key());

    let mut budget = InferenceBudget {
        deadline,
        remaining_applications,
    };
    let mut applications = Vec::new();
    let mut exhausted = false;
    for rule in ordered {
        if budget.exhausted() {
            exhausted = true;
            break;
        }
        let (mut rule_applications, rule_exhausted) =
            apply_rule_with_budget(rule, assertions, depth, &mut budget)?;
        applications.append(&mut rule_applications);
        if rule_exhausted {
            exhausted = true;
            break;
        }
    }
    deduplicate_applications(&mut applications);
    Ok((applications, budget.remaining_applications, exhausted))
}

fn conclusion_object_allowed(rule: &Rule, conclusion: &KnowledgeAssertion) -> bool {
    object_kind(&RuleTerm::Object(conclusion.object().clone()))
        .is_none_or(|kind| rule.scope().allows_object_kind(kind))
}

fn bindings_object_kinds_allowed(rule: &Rule, bindings: &VariableBindings) -> bool {
    bindings
        .values()
        .values()
        .all(|value| object_kind(value).is_none_or(|kind| rule.scope().allows_object_kind(kind)))
}

fn bindings_match_declared_types(rule: &Rule, bindings: &VariableBindings) -> bool {
    rule.bindings().iter().all(|binding| {
        binding.required_type().is_none_or(|required| {
            bindings
                .get(binding.variable())
                .and_then(object_kind)
                .is_some_and(|actual| actual == required)
        })
    })
}

/// Deduplicates applications by conclusion, rule identity, and premise path.
pub fn deduplicate_applications(applications: &mut Vec<RuleApplication>) {
    let mut seen = BTreeSet::new();
    applications.retain(|application| {
        seen.insert((
            application.conclusion().id().clone(),
            application.rule_id().clone(),
            application.rule_version(),
            application
                .premises()
                .iter()
                .map(|premise| premise.id().clone())
                .collect::<Vec<_>>(),
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

fn conditions_match(
    rule: &Rule,
    selected: &[KnowledgeAssertion],
    bindings: &VariableBindings,
) -> bool {
    rule.conditions()
        .iter()
        .all(|condition| evaluate_condition(condition, selected, bindings))
}

fn evaluate_condition(
    condition: &RuleCondition,
    selected: &[KnowledgeAssertion],
    bindings: &VariableBindings,
) -> bool {
    match condition {
        RuleCondition::Equal(left, right) => {
            resolve_term(left, bindings) == resolve_term(right, bindings)
        }
        RuleCondition::NotEqual(left, right) => {
            resolve_term(left, bindings) != resolve_term(right, bindings)
        }
        RuleCondition::PredicateIs(expected) => selected
            .iter()
            .any(|assertion| assertion.predicate().name() == expected),
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
    premises: &[KnowledgeAssertion],
) -> Result<KnowledgeAssertion, InferenceError> {
    let subject = ground_object(pattern.subject(), bindings)?;
    let predicate = ground_text(pattern.predicate(), bindings)?;
    let object = ground_object(pattern.object(), bindings)?;
    let predicate = crate::assertion::AssertionPredicate::new(predicate)
        .map_err(|error| InferenceError::InvalidConclusion(error.to_string()))?;

    let (context, qualifiers, polarity) = if premises.is_empty() {
        (
            crate::assertion::AssertionContext::new(),
            crate::assertion::Qualifiers::new(),
            AssertionPolarity::Positive,
        )
    } else {
        (
            combine_premise_contexts(premises),
            combine_premise_qualifiers(premises),
            premises[0].polarity(),
        )
    };
    let status = AssertionStatus::Provisional;

    Ok(KnowledgeAssertion::new(
        subject, predicate, object, context, qualifiers, status, polarity,
    ))
}

fn combine_premise_contexts(premises: &[KnowledgeAssertion]) -> crate::assertion::AssertionContext {
    let mut context = crate::assertion::AssertionContext::new();
    for (index, premise) in premises.iter().enumerate() {
        for (key, value) in premise.context().iter() {
            if context.get(key).is_none() || context.get(key) == Some(value) {
                let _ = context.insert(key, value);
            } else {
                let _ = context.insert(format!("premise.{index}.{key}"), value);
            }
        }
    }
    context
}

fn combine_premise_qualifiers(premises: &[KnowledgeAssertion]) -> crate::assertion::Qualifiers {
    premises
        .iter()
        .flat_map(|premise| premise.qualifiers().iter().cloned())
        .collect()
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
