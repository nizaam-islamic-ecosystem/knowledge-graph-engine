//! Main controlled Phase 8 reasoning engine.
//!
//! The reasoner owns orchestration only: profile resolution, governed-rule
//! selection, bounded work-queue execution, fixed-point detection, conflict
//! preservation, and derived-state assembly. It is not a query planner,
//! storage engine, governance engine, or control plane.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crate::assertion::KnowledgeAssertion;
use crate::query::ReasoningProfile;
use crate::storage::Snapshot;

use super::derived::DerivedKnowledgeSet;
use super::inference::{InferenceError, apply_rules_with_budget};
use super::profile::{ProfileValidationError, ReasoningProfileConfig};
use super::registry::{RegistryError, RuleRegistry};
use super::rule::{Rule, RulePattern, RuleTerm};

/// Inputs supplied to a reasoning run.
///
/// Semantic assertions are supplied by the caller because Phase 7's repository
/// intentionally keeps serialized records opaque at the storage boundary. The
/// snapshot remains the authoritative identity/version context for the run.
#[derive(Debug)]
pub struct ReasoningInput<'a> {
    snapshot: &'a Snapshot,
    assertions: &'a [KnowledgeAssertion],
    goal: Option<&'a RulePattern>,
}

impl<'a> ReasoningInput<'a> {
    /// Creates a reasoning input pinned to one immutable snapshot.
    #[must_use]
    pub const fn new(
        snapshot: &'a Snapshot,
        assertions: &'a [KnowledgeAssertion],
        goal: Option<&'a RulePattern>,
    ) -> Self {
        Self {
            snapshot,
            assertions,
            goal,
        }
    }

    /// Returns the pinned snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &Snapshot {
        self.snapshot
    }

    /// Returns semantic assertions available to the reasoning run.
    #[must_use]
    pub fn assertions(&self) -> &[KnowledgeAssertion] {
        self.assertions
    }

    /// Returns the optional query goal used for relevant-subgraph selection.
    #[must_use]
    pub const fn goal(&self) -> Option<&RulePattern> {
        self.goal
    }
}

/// Result status of one controlled reasoning execution.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReasoningExecutionStatus {
    /// Reasoning reached a fixed point within all configured budgets.
    Complete,
    /// Reasoning was safely stopped because a budget or execution limit was hit.
    Incomplete,
    /// Reasoning failed before producing a valid controlled result.
    Failed,
}

impl ReasoningExecutionStatus {
    /// Returns whether the run completed without truncation.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// One explicit conflict preserved by open-world reasoning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReasoningConflict {
    positive: crate::identity::KnowledgeAssertionId,
    negative: crate::identity::KnowledgeAssertionId,
}

impl ReasoningConflict {
    /// Creates a conflict pair.
    #[must_use]
    pub const fn new(
        positive: crate::identity::KnowledgeAssertionId,
        negative: crate::identity::KnowledgeAssertionId,
    ) -> Self {
        Self { positive, negative }
    }

    /// Returns the positive assertion identity.
    #[must_use]
    pub const fn positive(&self) -> &crate::identity::KnowledgeAssertionId {
        &self.positive
    }

    /// Returns the negative assertion identity.
    #[must_use]
    pub const fn negative(&self) -> &crate::identity::KnowledgeAssertionId {
        &self.negative
    }
}

/// Final result of one Phase 8 reasoning run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReasoningResult {
    status: ReasoningExecutionStatus,
    profile: ReasoningProfile,
    derived: DerivedKnowledgeSet,
    conflicts: Vec<ReasoningConflict>,
    rule_applications: usize,
    max_depth_reached: usize,
    expanded_assertions: usize,
    message: Option<String>,
}

impl ReasoningResult {
    /// Returns the execution status.
    #[must_use]
    pub const fn status(&self) -> ReasoningExecutionStatus {
        self.status
    }

    /// Returns the profile used for execution.
    #[must_use]
    pub const fn profile(&self) -> ReasoningProfile {
        self.profile
    }

    /// Returns rebuildable derived knowledge.
    #[must_use]
    pub const fn derived(&self) -> &DerivedKnowledgeSet {
        &self.derived
    }

    /// Returns preserved explicit conflicts.
    #[must_use]
    pub fn conflicts(&self) -> &[ReasoningConflict] {
        &self.conflicts
    }

    /// Returns the number of rule applications performed.
    #[must_use]
    pub const fn rule_applications(&self) -> usize {
        self.rule_applications
    }

    /// Returns the deepest derivation depth reached.
    #[must_use]
    pub const fn max_depth_reached(&self) -> usize {
        self.max_depth_reached
    }

    /// Returns the number of assertions considered during bounded expansion.
    #[must_use]
    pub const fn expanded_assertions(&self) -> usize {
        self.expanded_assertions
    }

    /// Returns the optional incomplete/failure message.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Controlled Phase 8 reasoner.
#[derive(Clone, Debug, Default)]
pub struct Reasoner {
    registry: RuleRegistry,
}

impl Reasoner {
    /// Creates a reasoner backed by one governed rule registry.
    #[must_use]
    pub const fn new(registry: RuleRegistry) -> Self {
        Self { registry }
    }

    /// Returns the governed rule registry.
    #[must_use]
    pub const fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    /// Executes controlled reasoning for one profile and immutable input snapshot.
    pub fn reason(
        &self,
        input: &ReasoningInput<'_>,
        profile: ReasoningProfile,
    ) -> Result<ReasoningResult, ReasonerError> {
        let config =
            ReasoningProfileConfig::for_profile(profile).map_err(ReasonerError::Profile)?;
        if profile == ReasoningProfile::None {
            return Err(ReasonerError::ProfileDisabled);
        }

        let mut working = select_relevant_assertions(input.assertions(), input.goal());
        let initially_truncated = working.len() > config.budgets().max_expansion();
        if initially_truncated {
            working.truncate(config.budgets().max_expansion());
        }

        let mut derived = DerivedKnowledgeSet::new();
        let mut seen_applications = BTreeSet::new();
        let mut rule_applications = 0usize;
        let mut max_depth_reached = 0usize;
        let mut expanded_assertions = working.len();
        let start = Instant::now();
        let deadline = start + config.budgets().execution_limit();
        let mut status = if initially_truncated {
            ReasoningExecutionStatus::Incomplete
        } else {
            ReasoningExecutionStatus::Complete
        };
        let mut message = initially_truncated
            .then(|| "reasoning expansion input truncated to the configured maximum".to_owned());

        for depth in 0..=config.budgets().max_depth() {
            max_depth_reached = depth;
            if Instant::now() >= deadline {
                status = ReasoningExecutionStatus::Incomplete;
                message = Some("reasoning execution time budget exceeded".to_owned());
                break;
            }

            let rules = self
                .registry
                .rules()
                .filter(|entry| entry.is_eligible())
                .filter(|entry| config.active_rule_kinds().contains(&entry.rule().kind()))
                .filter(|entry| {
                    entry
                        .rule()
                        .scope()
                        .max_depth()
                        .is_none_or(|limit| depth <= limit)
                })
                .filter(|entry| {
                    input
                        .goal()
                        .is_none_or(|goal| rule_may_reach_goal(entry.rule(), goal))
                })
                .map(|entry| entry.rule())
                .collect::<Vec<_>>();

            if rules.is_empty() {
                break;
            }

            let remaining_applications = config
                .budgets()
                .max_rule_applications()
                .saturating_sub(rule_applications);
            if remaining_applications == 0 {
                status = ReasoningExecutionStatus::Incomplete;
                message = Some("reasoning rule-application budget exceeded".to_owned());
                break;
            }

            let (applications, _remaining, matching_exhausted) =
                apply_rules_with_budget(&rules, &working, depth, deadline, remaining_applications)
                    .map_err(ReasonerError::Inference)?;

            if applications.is_empty() {
                if matching_exhausted {
                    status = ReasoningExecutionStatus::Incomplete;
                    message = Some(
                        "reasoning execution budget exhausted during premise matching".to_owned(),
                    );
                }
                break;
            }

            let mut new_assertions = BTreeMap::new();
            let mut stop = matching_exhausted;
            for application in applications {
                let premise_ids = application
                    .premises()
                    .iter()
                    .map(|premise| premise.id().clone())
                    .collect::<Vec<_>>();
                let application_key = (
                    application.conclusion().id().clone(),
                    application.rule_id().clone(),
                    application.rule_version(),
                    premise_ids,
                );

                if !seen_applications.insert(application_key) {
                    continue;
                }

                rule_applications = rule_applications.saturating_add(1);
                if rule_applications > config.budgets().max_rule_applications() {
                    status = ReasoningExecutionStatus::Incomplete;
                    message = Some("reasoning rule-application budget exceeded".to_owned());
                    stop = true;
                    break;
                }

                let is_new = !working
                    .iter()
                    .any(|assertion| assertion.id() == application.conclusion().id());

                if is_new && derived.len() >= config.budgets().max_derived_assertions() {
                    status = ReasoningExecutionStatus::Incomplete;
                    message = Some("reasoning derived-assertion budget exceeded".to_owned());
                    stop = true;
                    break;
                }

                if is_new && derived.len() >= config.budgets().result_limit() {
                    status = ReasoningExecutionStatus::Incomplete;
                    message = Some(
                        "reasoning result limit exceeded; derived output truncated".to_owned(),
                    );
                    stop = true;
                    break;
                }

                derived.insert_application(&application, input.snapshot(), profile);
                if is_new {
                    new_assertions.insert(
                        application.conclusion().id().clone(),
                        application.conclusion().clone(),
                    );
                }
            }

            if matching_exhausted && message.is_none() {
                status = ReasoningExecutionStatus::Incomplete;
                message =
                    Some("reasoning execution budget exhausted during premise matching".to_owned());
            }

            if stop {
                break;
            }

            if new_assertions.is_empty() {
                break;
            }

            if expanded_assertions.saturating_add(new_assertions.len())
                > config.budgets().max_expansion()
            {
                status = ReasoningExecutionStatus::Incomplete;
                message = Some("reasoning expansion budget exceeded".to_owned());
                break;
            }

            for assertion in new_assertions.into_values() {
                working.push(assertion);
            }
            expanded_assertions = working.len();

            if depth == config.budgets().max_depth() {
                status = ReasoningExecutionStatus::Incomplete;
                message = Some("reasoning depth budget exhausted".to_owned());
                break;
            }
        }

        let conflicts = detect_conflicts(input.assertions(), derived.values());
        Ok(ReasoningResult {
            status,
            profile,
            derived,
            conflicts,
            rule_applications,
            max_depth_reached,
            expanded_assertions,
            message,
        })
    }
}

fn select_relevant_assertions(
    assertions: &[KnowledgeAssertion],
    goal: Option<&RulePattern>,
) -> Vec<KnowledgeAssertion> {
    match goal {
        None => assertions.to_vec(),
        Some(goal) => assertions
            .iter()
            .filter(|assertion| pattern_may_match(goal, assertion))
            .cloned()
            .collect(),
    }
}

fn rule_may_reach_goal(rule: &Rule, goal: &RulePattern) -> bool {
    pattern_terms_may_overlap(rule.conclusion(), goal)
        || rule
            .premises()
            .iter()
            .any(|premise| pattern_terms_may_overlap(premise, goal))
}

fn pattern_terms_may_overlap(left: &RulePattern, right: &RulePattern) -> bool {
    terms_may_overlap(left.subject(), right.subject())
        && terms_may_overlap(left.predicate(), right.predicate())
        && terms_may_overlap(left.object(), right.object())
}

fn terms_may_overlap(left: &RuleTerm, right: &RuleTerm) -> bool {
    match (left, right) {
        (RuleTerm::Variable(_), _) | (_, RuleTerm::Variable(_)) => true,
        _ => left == right,
    }
}

fn pattern_may_match(pattern: &RulePattern, assertion: &KnowledgeAssertion) -> bool {
    term_may_match(
        pattern.subject(),
        &crate::reasoning::rule::RuleTerm::Object(assertion.subject().clone()),
    ) && term_may_match(
        pattern.predicate(),
        &crate::reasoning::rule::RuleTerm::Text(assertion.predicate().name().to_owned()),
    ) && term_may_match(
        pattern.object(),
        &crate::reasoning::rule::RuleTerm::Object(assertion.object().clone()),
    )
}

fn term_may_match(
    pattern: &crate::reasoning::rule::RuleTerm,
    value: &crate::reasoning::rule::RuleTerm,
) -> bool {
    match pattern {
        crate::reasoning::rule::RuleTerm::Variable(_) => true,
        _ => pattern == value,
    }
}

fn detect_conflicts<'a, I>(explicit: &[KnowledgeAssertion], derived: I) -> Vec<ReasoningConflict>
where
    I: Iterator<Item = &'a super::derived::DerivedKnowledge>,
{
    let mut all = explicit.to_vec();
    all.extend(derived.map(|value| value.assertion().clone()));
    let mut positives = BTreeMap::new();
    let mut negatives = BTreeMap::new();
    for assertion in all {
        let key = (
            format!("{:?}", assertion.subject()),
            assertion.predicate().as_str().to_owned(),
            format!("{:?}", assertion.object()),
            format!("{:?}", assertion.context()),
            format!("{:?}", assertion.qualifiers()),
        );
        match assertion.polarity() {
            crate::assertion::AssertionPolarity::Positive => {
                positives.insert(key, assertion.id().clone());
            }
            crate::assertion::AssertionPolarity::Negative => {
                negatives.insert(key, assertion.id().clone());
            }
        }
    }

    positives
        .into_iter()
        .filter_map(|(key, positive)| {
            negatives
                .get(&key)
                .cloned()
                .map(|negative| ReasoningConflict::new(positive, negative))
        })
        .collect()
}

/// Reasoner-level failures.
#[derive(Debug)]
pub enum ReasonerError {
    /// The requested public profile could not be converted to a safe Phase 8 configuration.
    Profile(ProfileValidationError),
    /// The requested profile explicitly disables reasoning.
    ProfileDisabled,
    /// Governed rule registry failure.
    Registry(RegistryError),
    /// Rule evaluation failure.
    Inference(InferenceError),
}

impl fmt::Display for ReasonerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(formatter, "reasoning profile error: {error}"),
            Self::ProfileDisabled => formatter.write_str("reasoning profile disables execution"),
            Self::Registry(error) => write!(formatter, "reasoning registry error: {error}"),
            Self::Inference(error) => write!(formatter, "reasoning inference error: {error}"),
        }
    }
}

impl std::error::Error for ReasonerError {}

impl From<RegistryError> for ReasonerError {
    fn from(error: RegistryError) -> Self {
        Self::Registry(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::identity::{EntityId, RuleId};
    use crate::reasoning::rule::{
        RuleKind, RuleMetadata, RulePattern, RuleScope, RuleTerm, RuleVersion,
    };
    use crate::storage::RepositoryRecord;
    use crate::versioning::{ChangeTarget, KgVersion};

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

    fn snapshot() -> Snapshot {
        Snapshot::new(
            KgVersion::initial(),
            vec![RepositoryRecord::new(
                ChangeTarget::Entity(EntityId::new("entity-1").expect("valid id")),
                crate::storage::Document::from_slice(b"entity-1"),
            )],
        )
        .expect("snapshot should succeed")
    }

    #[test]
    fn reasoner_reaches_fixed_point_for_a_simple_inverse_rule() {
        let mut registry = RuleRegistry::new();
        let rule = inverse_rule();
        let id = rule.id().clone();
        registry
            .register_built_in(rule)
            .expect("register should succeed");
        registry.approve(&id).expect("approve should succeed");
        registry
            .version(&id, RuleVersion::default())
            .expect("version should succeed");
        registry.activate(&id).expect("activate should succeed");

        let reasoner = Reasoner::new(registry);
        let source = assertion("a", "parent-of", "b");
        let snapshot = snapshot();
        let assertions = [source];
        let input = ReasoningInput::new(&snapshot, &assertions, None);
        let result = reasoner
            .reason(&input, ReasoningProfile::Basic)
            .expect("reasoning should succeed");

        assert!(
            result
                .derived()
                .values()
                .any(|value| value.assertion().predicate().name() == "child-of")
        );
        assert!(matches!(
            result.status(),
            ReasoningExecutionStatus::Complete | ReasoningExecutionStatus::Incomplete
        ));
    }

    #[test]
    fn disabled_profile_is_rejected_at_phase8_boundary() {
        let reasoner = Reasoner::default();
        let assertions: Vec<KnowledgeAssertion> = Vec::new();
        let snapshot = snapshot();
        let input = ReasoningInput::new(&snapshot, &assertions, None);
        assert!(matches!(
            reasoner.reason(&input, ReasoningProfile::None),
            Err(ReasonerError::ProfileDisabled)
        ));
    }
}
