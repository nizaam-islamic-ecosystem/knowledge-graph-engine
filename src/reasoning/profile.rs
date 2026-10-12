//! Phase 8 interpretation of the public query reasoning profiles.
//!
//! The public profile enum remains owned by Phase 6. This module maps that
//! vocabulary to controlled rule selection, expansion, explanation, and safety
//! budgets without creating a second public profile enum.

use core::fmt;
use std::collections::BTreeSet;
use std::time::Duration;

use crate::query::ReasoningProfile;

use super::rule::{RuleKind, RuleScope};

/// How far a reasoning run may expand beyond its initial target region.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExpansionPolicy {
    /// Do not expand beyond the supplied reasoning inputs.
    None,
    /// Permit bounded graph expansion under the configured budget.
    Bounded,
}

/// Explainability level for a reasoning run.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExplanationPolicy {
    /// Emit the mandatory compact derivation trace.
    Compact,
    /// Emit the compact trace and allow an expanded derivation graph.
    ExpandedGraph,
}

/// Safety and resource limits for one reasoning profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReasoningBudgets {
    max_depth: usize,
    max_rule_applications: usize,
    max_derived_assertions: usize,
    max_expansion: usize,
    execution_limit: Duration,
    result_limit: usize,
}

impl ReasoningBudgets {
    /// Creates one set of profile budgets.
    #[must_use]
    pub const fn new(
        max_depth: usize,
        max_rule_applications: usize,
        max_derived_assertions: usize,
        max_expansion: usize,
        execution_limit: Duration,
        result_limit: usize,
    ) -> Self {
        Self {
            max_depth,
            max_rule_applications,
            max_derived_assertions,
            max_expansion,
            execution_limit,
            result_limit,
        }
    }

    /// Returns the maximum derivation depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the maximum number of rule applications.
    #[must_use]
    pub const fn max_rule_applications(&self) -> usize {
        self.max_rule_applications
    }

    /// Returns the maximum number of derived assertions.
    #[must_use]
    pub const fn max_derived_assertions(&self) -> usize {
        self.max_derived_assertions
    }

    /// Returns the maximum bounded expansion size.
    #[must_use]
    pub const fn max_expansion(&self) -> usize {
        self.max_expansion
    }

    /// Returns the execution time limit.
    #[must_use]
    pub const fn execution_limit(&self) -> Duration {
        self.execution_limit
    }

    /// Returns the maximum number of result assertions exposed by a run.
    #[must_use]
    pub const fn result_limit(&self) -> usize {
        self.result_limit
    }
}

/// Global hard ceilings that no profile may exceed.
pub const GLOBAL_HARD_CEILINGS: ReasoningBudgets = ReasoningBudgets::new(
    64,
    100_000,
    50_000,
    100_000,
    Duration::from_secs(30),
    10_000,
);

/// Immutable Phase 8 configuration derived from a public query profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReasoningProfileConfig {
    profile: ReasoningProfile,
    active_rule_kinds: BTreeSet<RuleKind>,
    rule_scope: RuleScope,
    expansion: ExpansionPolicy,
    budgets: ReasoningBudgets,
    explanation: ExplanationPolicy,
}

impl ReasoningProfileConfig {
    /// Resolves the Phase 6 profile into its Phase 8 configuration.
    pub fn for_profile(profile: ReasoningProfile) -> Result<Self, ProfileValidationError> {
        let all_kinds = [
            RuleKind::Inverse,
            RuleKind::Symmetric,
            RuleKind::Transitive,
            RuleKind::TypeInheritance,
            RuleKind::PropertyInheritance,
            RuleKind::Composition,
            RuleKind::Conditional,
        ]
        .into_iter()
        .collect();

        let (active_rule_kinds, expansion, budgets, explanation) = match profile {
            ReasoningProfile::None => (
                BTreeSet::new(),
                ExpansionPolicy::None,
                ReasoningBudgets::new(1, 1, 1, 1, Duration::from_millis(1), 1),
                ExplanationPolicy::Compact,
            ),
            ReasoningProfile::Basic => (
                [RuleKind::Inverse, RuleKind::Symmetric]
                    .into_iter()
                    .collect(),
                ExpansionPolicy::Bounded,
                ReasoningBudgets::new(4, 1_000, 500, 1_000, Duration::from_secs(1), 100),
                ExplanationPolicy::Compact,
            ),
            ReasoningProfile::Semantic => (
                [
                    RuleKind::Inverse,
                    RuleKind::Symmetric,
                    RuleKind::Transitive,
                    RuleKind::TypeInheritance,
                    RuleKind::PropertyInheritance,
                    RuleKind::Composition,
                    RuleKind::Conditional,
                ]
                .into_iter()
                .collect(),
                ExpansionPolicy::Bounded,
                ReasoningBudgets::new(16, 10_000, 5_000, 10_000, Duration::from_secs(5), 1_000),
                ExplanationPolicy::Compact,
            ),
            ReasoningProfile::Deep => (
                all_kinds,
                ExpansionPolicy::Bounded,
                ReasoningBudgets::new(32, 50_000, 25_000, 50_000, Duration::from_secs(15), 5_000),
                ExplanationPolicy::ExpandedGraph,
            ),
        };

        let config = Self {
            profile,
            active_rule_kinds,
            rule_scope: RuleScope::unrestricted(),
            expansion,
            budgets,
            explanation,
        };
        config.validate()?;
        Ok(config)
    }

    /// Returns the public Phase 6 profile this configuration interprets.
    #[must_use]
    pub const fn profile(&self) -> ReasoningProfile {
        self.profile
    }

    /// Returns the allowed rule families.
    #[must_use]
    pub fn active_rule_kinds(&self) -> &BTreeSet<RuleKind> {
        &self.active_rule_kinds
    }

    /// Returns the rule-scope constraint.
    #[must_use]
    pub const fn rule_scope(&self) -> &RuleScope {
        &self.rule_scope
    }

    /// Returns the expansion policy.
    #[must_use]
    pub const fn expansion(&self) -> ExpansionPolicy {
        self.expansion
    }

    /// Returns the profile budgets.
    #[must_use]
    pub const fn budgets(&self) -> ReasoningBudgets {
        self.budgets
    }

    /// Returns the explanation policy.
    #[must_use]
    pub const fn explanation(&self) -> ExplanationPolicy {
        self.explanation
    }

    /// Validates all profile budgets against the global hard ceilings.
    pub fn validate(&self) -> Result<(), ProfileValidationError> {
        validate_budget(
            "max_depth",
            self.budgets.max_depth,
            GLOBAL_HARD_CEILINGS.max_depth,
        )?;
        validate_budget(
            "max_rule_applications",
            self.budgets.max_rule_applications,
            GLOBAL_HARD_CEILINGS.max_rule_applications,
        )?;
        validate_budget(
            "max_derived_assertions",
            self.budgets.max_derived_assertions,
            GLOBAL_HARD_CEILINGS.max_derived_assertions,
        )?;
        validate_budget(
            "max_expansion",
            self.budgets.max_expansion,
            GLOBAL_HARD_CEILINGS.max_expansion,
        )?;
        validate_budget(
            "result_limit",
            self.budgets.result_limit,
            GLOBAL_HARD_CEILINGS.result_limit,
        )?;
        if self.budgets.execution_limit > GLOBAL_HARD_CEILINGS.execution_limit {
            return Err(ProfileValidationError::ExceedsGlobalCeiling {
                field: "execution_limit",
            });
        }
        if self.budgets.max_depth == 0
            || self.budgets.max_rule_applications == 0
            || self.budgets.max_derived_assertions == 0
            || self.budgets.max_expansion == 0
            || self.budgets.result_limit == 0
        {
            return Err(ProfileValidationError::ZeroBudget);
        }
        Ok(())
    }
}

fn validate_budget(
    field: &'static str,
    value: usize,
    ceiling: usize,
) -> Result<(), ProfileValidationError> {
    if value > ceiling {
        return Err(ProfileValidationError::ExceedsGlobalCeiling { field });
    }
    Ok(())
}

/// Profile configuration failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileValidationError {
    /// A configured budget exceeds a hard global ceiling.
    ExceedsGlobalCeiling { field: &'static str },
    /// A profile contains a zero budget.
    ZeroBudget,
}

impl fmt::Display for ProfileValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExceedsGlobalCeiling { field } => {
                write!(
                    formatter,
                    "reasoning profile budget exceeds global ceiling: {field}"
                )
            }
            Self::ZeroBudget => {
                formatter.write_str("reasoning profile budgets must be greater than zero")
            }
        }
    }
}

impl std::error::Error for ProfileValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_profile_maps_to_controlled_rule_families() {
        let profile = ReasoningProfileConfig::for_profile(ReasoningProfile::Basic)
            .expect("basic profile should validate");
        assert!(profile.active_rule_kinds().contains(&RuleKind::Inverse));
        assert!(profile.active_rule_kinds().contains(&RuleKind::Symmetric));
        assert!(!profile.active_rule_kinds().contains(&RuleKind::Conditional));
    }

    #[test]
    fn profile_budgets_are_within_global_hard_ceilings() {
        for profile in [
            ReasoningProfile::None,
            ReasoningProfile::Basic,
            ReasoningProfile::Semantic,
            ReasoningProfile::Deep,
        ] {
            assert!(
                ReasoningProfileConfig::for_profile(profile)
                    .expect("profile should validate")
                    .validate()
                    .is_ok()
            );
        }
    }
}
