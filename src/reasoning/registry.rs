//! Governed registry for executable Phase 8 reasoning rules.
//!
//! Rule existence is deliberately distinct from executability. A rule becomes
//! eligible only after validation, approval, versioning, and activation.

use core::fmt;
use std::collections::BTreeMap;

use crate::identity::RuleId;

use super::rule::{Rule, RuleKind, RuleScope, RuleValidationError, RuleVersion};

/// Origin of a governed rule definition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleOrigin {
    /// Foundational rule supplied by the implementation.
    BuiltIn,
    /// Declarative rule supplied through the governed rule boundary.
    Declarative,
}

/// Lifecycle state of a registered rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleLifecycle {
    /// Rule exists but has not passed validation.
    Defined,
    /// Rule passed structural validation.
    Validated,
    /// Rule was explicitly approved for activation.
    Approved,
    /// Rule has a governed version and is eligible for activation.
    Versioned,
    /// Rule is active and may be selected by a profile.
    Activated,
    /// Rule was explicitly deactivated.
    Deactivated,
}

impl RuleLifecycle {
    /// Returns whether the lifecycle state is executable.
    #[must_use]
    pub const fn is_eligible(self) -> bool {
        matches!(self, Self::Activated)
    }
}

/// One governed rule registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRule {
    rule: Rule,
    origin: RuleOrigin,
    lifecycle: RuleLifecycle,
}

impl RegisteredRule {
    /// Returns the underlying rule.
    #[must_use]
    pub const fn rule(&self) -> &Rule {
        &self.rule
    }

    /// Returns the rule origin.
    #[must_use]
    pub const fn origin(&self) -> RuleOrigin {
        self.origin
    }

    /// Returns the lifecycle state.
    #[must_use]
    pub const fn lifecycle(&self) -> RuleLifecycle {
        self.lifecycle
    }

    /// Returns whether the rule may execute.
    #[must_use]
    pub const fn is_eligible(&self) -> bool {
        self.lifecycle.is_eligible()
    }
}

/// Governed rule registry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuleRegistry {
    rules: BTreeMap<RuleId, RegisteredRule>,
}

impl RuleRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a built-in rule after structural validation.
    pub fn register_built_in(&mut self, rule: Rule) -> Result<(), RegistryError> {
        self.register(rule, RuleOrigin::BuiltIn)
    }

    /// Registers a declarative rule after structural validation.
    pub fn register_declarative(&mut self, rule: Rule) -> Result<(), RegistryError> {
        self.register(rule, RuleOrigin::Declarative)
    }

    fn register(&mut self, rule: Rule, origin: RuleOrigin) -> Result<(), RegistryError> {
        rule.validate().map_err(RegistryError::Validation)?;
        if self.rules.contains_key(rule.id()) {
            return Err(RegistryError::AlreadyRegistered(rule.id().clone()));
        }
        let id = rule.id().clone();
        self.rules.insert(
            id,
            RegisteredRule {
                rule,
                origin,
                lifecycle: RuleLifecycle::Validated,
            },
        );
        Ok(())
    }

    /// Approves a validated rule.
    pub fn approve(&mut self, id: &RuleId) -> Result<(), RegistryError> {
        let entry = self.entry_mut(id)?;
        if !matches!(
            entry.lifecycle,
            RuleLifecycle::Validated | RuleLifecycle::Versioned
        ) {
            return Err(RegistryError::InvalidTransition {
                id: id.clone(),
                state: entry.lifecycle,
            });
        }
        entry.lifecycle = RuleLifecycle::Approved;
        Ok(())
    }

    /// Marks an approved rule as versioned.
    pub fn version(&mut self, id: &RuleId, version: RuleVersion) -> Result<(), RegistryError> {
        let entry = self.entry_mut(id)?;
        if entry.rule.version() != version {
            return Err(RegistryError::VersionMismatch {
                id: id.clone(),
                expected: entry.rule.version(),
                actual: version,
            });
        }
        if !matches!(
            entry.lifecycle,
            RuleLifecycle::Approved | RuleLifecycle::Versioned
        ) {
            return Err(RegistryError::InvalidTransition {
                id: id.clone(),
                state: entry.lifecycle,
            });
        }
        entry.lifecycle = RuleLifecycle::Versioned;
        Ok(())
    }

    /// Activates a versioned rule.
    pub fn activate(&mut self, id: &RuleId) -> Result<(), RegistryError> {
        let entry = self.entry_mut(id)?;
        if !matches!(
            entry.lifecycle,
            RuleLifecycle::Versioned | RuleLifecycle::Deactivated
        ) {
            return Err(RegistryError::InvalidTransition {
                id: id.clone(),
                state: entry.lifecycle,
            });
        }
        entry.lifecycle = RuleLifecycle::Activated;
        Ok(())
    }

    /// Deactivates an active rule without deleting its definition.
    pub fn deactivate(&mut self, id: &RuleId) -> Result<(), RegistryError> {
        let entry = self.entry_mut(id)?;
        if entry.lifecycle != RuleLifecycle::Activated {
            return Err(RegistryError::InvalidTransition {
                id: id.clone(),
                state: entry.lifecycle,
            });
        }
        entry.lifecycle = RuleLifecycle::Deactivated;
        Ok(())
    }

    /// Looks up a registered rule by identity.
    #[must_use]
    pub fn get(&self, id: &RuleId) -> Option<&RegisteredRule> {
        self.rules.get(id)
    }

    /// Looks up an exact rule version by identity.
    #[must_use]
    pub fn get_version(&self, id: &RuleId, version: RuleVersion) -> Option<&Rule> {
        self.rules
            .get(id)
            .filter(|entry| entry.rule.version() == version)
            .map(RegisteredRule::rule)
    }

    /// Returns all registered rules in deterministic identity order.
    pub fn rules(&self) -> impl Iterator<Item = &RegisteredRule> {
        self.rules.values()
    }

    /// Returns active rules matching the supplied kind and scope constraints.
    #[must_use]
    pub fn eligible(&self, kind: Option<RuleKind>, scope: Option<&RuleScope>) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|entry| entry.is_eligible())
            .filter(|entry| kind.is_none_or(|expected| entry.rule.kind() == expected))
            .filter(|entry| {
                scope.is_none_or(|requested| scope_compatible(entry.rule.scope(), requested))
            })
            .map(RegisteredRule::rule)
            .collect()
    }

    fn entry_mut(&mut self, id: &RuleId) -> Result<&mut RegisteredRule, RegistryError> {
        self.rules
            .get_mut(id)
            .ok_or_else(|| RegistryError::NotFound(id.clone()))
    }
}

fn scope_compatible(rule: &RuleScope, requested: &RuleScope) -> bool {
    if let Some(rule_depth) = rule.max_depth()
        && let Some(requested_depth) = requested.max_depth()
    {
        return rule_depth <= requested_depth;
    }
    true
}

/// Registry failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryError {
    /// Rule validation failed.
    Validation(RuleValidationError),
    /// A rule identity is already registered.
    AlreadyRegistered(RuleId),
    /// A requested rule identity is absent.
    NotFound(RuleId),
    /// A lifecycle transition is not permitted.
    InvalidTransition { id: RuleId, state: RuleLifecycle },
    /// A supplied version does not match the registered rule version.
    VersionMismatch {
        id: RuleId,
        expected: RuleVersion,
        actual: RuleVersion,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => write!(formatter, "rule validation failed: {error}"),
            Self::AlreadyRegistered(id) => write!(formatter, "rule is already registered: {id}"),
            Self::NotFound(id) => write!(formatter, "rule was not found: {id}"),
            Self::InvalidTransition { id, state } => {
                write!(
                    formatter,
                    "invalid lifecycle transition for {id} from {state:?}"
                )
            }
            Self::VersionMismatch {
                id,
                expected,
                actual,
            } => write!(
                formatter,
                "rule version mismatch for {id}: expected {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::RuleId;
    use crate::reasoning::rule::{RuleMetadata, RulePattern, RuleScope, RuleTerm};

    fn rule() -> Rule {
        Rule::new(
            RuleId::new("rule-1").expect("valid rule id"),
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
            RuleMetadata::new("inverse", "inverse test").expect("valid metadata"),
        )
        .expect("rule should be valid")
    }

    #[test]
    fn registry_requires_governed_lifecycle_before_execution() {
        let mut registry = RuleRegistry::new();
        let id = rule().id().clone();
        registry
            .register_declarative(rule())
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
    fn registry_keeps_declarative_and_builtin_origins_distinct() {
        let mut registry = RuleRegistry::new();
        let built_in = rule();
        registry
            .register_built_in(built_in.clone())
            .expect("built-in should register");
        assert_eq!(
            registry
                .get(built_in.id())
                .expect("rule should exist")
                .origin(),
            RuleOrigin::BuiltIn
        );
    }
}
