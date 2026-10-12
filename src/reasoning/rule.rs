//! Declarative rule model for controlled Knowledge Graph reasoning.
//!
//! This module describes semantic rules only. It does not execute rules, own
//! rule governance, persist derived state, or create new semantic identities.
//! Execution belongs to [`super::inference`] and orchestration belongs to
//! [`super::reasoner`].

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::assertion::AssertionObject;
use crate::identity::RuleId;

/// The initial controlled families of semantic rules supported by Phase 8.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleKind {
    /// Derives the inverse relation.
    Inverse,
    /// Derives the symmetric relation.
    Symmetric,
    /// Derives transitive closure for one predicate.
    Transitive,
    /// Derives a type/class inheritance relation.
    TypeInheritance,
    /// Derives controlled property inheritance.
    PropertyInheritance,
    /// Derives a conclusion from a bounded predicate composition.
    Composition,
    /// Applies a declarative condition to produce a conclusion.
    Conditional,
}

impl RuleKind {
    /// Returns the stable textual representation of the rule family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inverse => "inverse",
            Self::Symmetric => "symmetric",
            Self::Transitive => "transitive",
            Self::TypeInheritance => "type_inheritance",
            Self::PropertyInheritance => "property_inheritance",
            Self::Composition => "composition",
            Self::Conditional => "conditional",
        }
    }
}

impl fmt::Display for RuleKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Semantic version of one rule definition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuleVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl RuleVersion {
    /// Creates a semantic rule version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns the major component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Returns the patch component.
    #[must_use]
    pub const fn patch(self) -> u16 {
        self.patch
    }

    /// Returns whether two rule versions are compatible for this initial
    /// reasoning contract. A major-version change is incompatible.
    #[must_use]
    pub const fn compatible_with(self, other: Self) -> bool {
        self.major == other.major
    }
}

impl Default for RuleVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl fmt::Display for RuleVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A typed value used by rule patterns and conclusions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleTerm {
    /// A semantic object reference already present in the KG.
    Object(AssertionObject),
    /// A predicate or controlled textual value.
    Text(String),
    /// A named rule variable.
    Variable(String),
}

impl RuleTerm {
    /// Creates a named rule variable.
    pub fn variable(name: impl Into<String>) -> Result<Self, RuleValidationError> {
        let name = name.into();
        validate_variable(&name)?;
        Ok(Self::Variable(name))
    }

    /// Creates a textual term.
    pub fn text(value: impl Into<String>) -> Result<Self, RuleValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(RuleValidationError::EmptyTextTerm);
        }
        Ok(Self::Text(value))
    }

    /// Returns the variable name when this term is a variable.
    #[must_use]
    pub fn variable_name(&self) -> Option<&str> {
        match self {
            Self::Variable(name) => Some(name),
            _ => None,
        }
    }
}

/// A declarative assertion pattern used as a premise or conclusion.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RulePattern {
    subject: RuleTerm,
    predicate: RuleTerm,
    object: RuleTerm,
}

impl RulePattern {
    /// Creates a rule pattern.
    #[must_use]
    pub const fn new(subject: RuleTerm, predicate: RuleTerm, object: RuleTerm) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }

    /// Returns the subject term.
    #[must_use]
    pub const fn subject(&self) -> &RuleTerm {
        &self.subject
    }

    /// Returns the predicate term.
    #[must_use]
    pub const fn predicate(&self) -> &RuleTerm {
        &self.predicate
    }

    /// Returns the object term.
    #[must_use]
    pub const fn object(&self) -> &RuleTerm {
        &self.object
    }

    fn variables(&self) -> impl Iterator<Item = &str> {
        [
            self.subject.variable_name(),
            self.predicate.variable_name(),
            self.object.variable_name(),
        ]
        .into_iter()
        .flatten()
    }
}

/// A declared variable binding constraint in a rule definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuleBinding {
    variable: String,
    required_type: Option<String>,
}

impl RuleBinding {
    /// Creates an unconstrained binding declaration.
    pub fn new(variable: impl Into<String>) -> Result<Self, RuleValidationError> {
        let variable = variable.into();
        validate_variable(&variable)?;
        Ok(Self {
            variable,
            required_type: None,
        })
    }

    /// Adds a semantic object-kind constraint to the binding.
    pub fn with_required_type(
        mut self,
        required_type: impl Into<String>,
    ) -> Result<Self, RuleValidationError> {
        let required_type = required_type.into();
        if required_type.trim().is_empty() {
            return Err(RuleValidationError::EmptyBindingType);
        }
        self.required_type = Some(required_type);
        Ok(self)
    }

    /// Returns the variable name.
    #[must_use]
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the optional semantic object-kind constraint.
    #[must_use]
    pub fn required_type(&self) -> Option<&str> {
        self.required_type.as_deref()
    }
}

/// Controlled conditions evaluated against a successful premise binding.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleCondition {
    /// Requires two terms to resolve to the same value.
    Equal(RuleTerm, RuleTerm),
    /// Requires two terms to resolve to different values.
    NotEqual(RuleTerm, RuleTerm),
    /// Requires a textual predicate value.
    PredicateIs(String),
    /// Requires a semantic object-kind label for a bound object.
    ObjectKind { variable: String, kind: String },
}

/// Restricts where a rule may be considered eligible.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct RuleScope {
    allowed_predicates: BTreeSet<String>,
    allowed_object_kinds: BTreeSet<String>,
    max_depth: Option<usize>,
}

impl RuleScope {
    /// Creates an unrestricted rule scope subject to profile budgets.
    #[must_use]
    pub fn unrestricted() -> Self {
        Self::default()
    }

    /// Adds an allowed predicate to the scope.
    #[must_use]
    pub fn allowing_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.allowed_predicates.insert(predicate.into());
        self
    }

    /// Adds an allowed assertion-object kind to the scope.
    #[must_use]
    pub fn allowing_object_kind(mut self, kind: impl Into<String>) -> Self {
        self.allowed_object_kinds.insert(kind.into());
        self
    }

    /// Restricts the maximum derivation depth for this rule.
    #[must_use]
    pub const fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Returns the configured maximum derivation depth.
    #[must_use]
    pub const fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    /// Returns whether a predicate is allowed by this scope.
    #[must_use]
    pub fn allows_predicate(&self, predicate: &str) -> bool {
        self.allowed_predicates.is_empty() || self.allowed_predicates.contains(predicate)
    }

    /// Returns whether an object kind is allowed by this scope.
    #[must_use]
    pub fn allows_object_kind(&self, kind: &str) -> bool {
        self.allowed_object_kinds.is_empty() || self.allowed_object_kinds.contains(kind)
    }
}

/// Human-readable and governance metadata attached to a rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleMetadata {
    name: String,
    description: String,
    tags: BTreeSet<String>,
}

impl RuleMetadata {
    /// Creates rule metadata.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, RuleValidationError> {
        let name = name.into();
        let description = description.into();
        if name.trim().is_empty() {
            return Err(RuleValidationError::EmptyMetadataName);
        }
        if description.trim().is_empty() {
            return Err(RuleValidationError::EmptyMetadataDescription);
        }
        Ok(Self {
            name,
            description,
            tags: BTreeSet::new(),
        })
    }

    /// Adds a metadata tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Returns the display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns metadata tags.
    #[must_use]
    pub fn tags(&self) -> &BTreeSet<String> {
        &self.tags
    }
}

/// Declarative semantic rule definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule {
    id: RuleId,
    version: RuleVersion,
    kind: RuleKind,
    premises: Vec<RulePattern>,
    conditions: Vec<RuleCondition>,
    bindings: Vec<RuleBinding>,
    conclusion: RulePattern,
    scope: RuleScope,
    metadata: RuleMetadata,
}

impl Rule {
    /// Creates a declarative rule and validates its structural contract.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RuleId,
        version: RuleVersion,
        kind: RuleKind,
        premises: Vec<RulePattern>,
        conditions: Vec<RuleCondition>,
        bindings: Vec<RuleBinding>,
        conclusion: RulePattern,
        scope: RuleScope,
        metadata: RuleMetadata,
    ) -> Result<Self, RuleValidationError> {
        let rule = Self {
            id,
            version,
            kind,
            premises,
            conditions,
            bindings,
            conclusion,
            scope,
            metadata,
        };
        rule.validate()?;
        Ok(rule)
    }

    /// Validates the complete declarative rule contract.
    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if self.id.as_str().trim().is_empty() {
            return Err(RuleValidationError::EmptyRuleId);
        }
        if self.premises.is_empty() {
            return Err(RuleValidationError::NoPremises);
        }

        let premise_variables = self
            .premises
            .iter()
            .flat_map(RulePattern::variables)
            .collect::<BTreeSet<_>>();
        let declared_variables = self
            .bindings
            .iter()
            .map(|binding| binding.variable.as_str())
            .collect::<BTreeSet<_>>();

        for variable in &declared_variables {
            if !premise_variables.contains(variable) {
                return Err(RuleValidationError::UnboundBinding((*variable).to_owned()));
            }
        }

        for variable in self.conclusion.variables() {
            if !premise_variables.contains(variable) && !declared_variables.contains(variable) {
                return Err(RuleValidationError::UnboundConclusionVariable(
                    variable.to_owned(),
                ));
            }
        }

        for condition in &self.conditions {
            validate_condition(condition, &premise_variables, &declared_variables)?;
        }

        if let Some(max_depth) = self.scope.max_depth
            && max_depth == 0
        {
            return Err(RuleValidationError::ZeroScopeDepth);
        }

        validate_family_shape(self.kind, &self.premises, &self.conclusion)?;
        Ok(())
    }

    /// Returns the rule identity.
    #[must_use]
    pub const fn id(&self) -> &RuleId {
        &self.id
    }

    /// Returns the rule version.
    #[must_use]
    pub const fn version(&self) -> RuleVersion {
        self.version
    }

    /// Returns the rule family.
    #[must_use]
    pub const fn kind(&self) -> RuleKind {
        self.kind
    }

    /// Returns rule premises.
    #[must_use]
    pub fn premises(&self) -> &[RulePattern] {
        &self.premises
    }

    /// Returns rule conditions.
    #[must_use]
    pub fn conditions(&self) -> &[RuleCondition] {
        &self.conditions
    }

    /// Returns declared bindings.
    #[must_use]
    pub fn bindings(&self) -> &[RuleBinding] {
        &self.bindings
    }

    /// Returns the conclusion pattern.
    #[must_use]
    pub const fn conclusion(&self) -> &RulePattern {
        &self.conclusion
    }

    /// Returns the rule scope.
    #[must_use]
    pub const fn scope(&self) -> &RuleScope {
        &self.scope
    }

    /// Returns rule metadata.
    #[must_use]
    pub const fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    /// Returns whether another version can be used as a compatible rule version.
    #[must_use]
    pub fn compatible_with(&self, other: &Rule) -> bool {
        self.id == other.id && self.version.compatible_with(other.version)
    }

    /// Returns a deterministic structural fingerprint for registry/testing use.
    #[must_use]
    pub fn structural_key(&self) -> String {
        let mut key = format!("{}:{}:{}", self.id, self.version, self.kind);
        key.push('|');
        for premise in &self.premises {
            key.push_str(&format!("{:?};", premise));
        }
        key.push_str(&format!("=>{:?}", self.conclusion));
        key
    }
}

/// Structural failures in declarative rule definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleValidationError {
    /// The rule identity is empty.
    EmptyRuleId,
    /// A rule must contain at least one premise.
    NoPremises,
    /// A variable name is empty or whitespace-only.
    EmptyVariable,
    /// A textual term is empty or whitespace-only.
    EmptyTextTerm,
    /// A binding type is empty.
    EmptyBindingType,
    /// A metadata name is empty.
    EmptyMetadataName,
    /// A metadata description is empty.
    EmptyMetadataDescription,
    /// A declared binding is not introduced by any premise.
    UnboundBinding(String),
    /// A conclusion variable is not grounded by premises or declarations.
    UnboundConclusionVariable(String),
    /// A condition refers to an unavailable variable.
    UnboundConditionVariable(String),
    /// A scoped rule cannot use a zero depth.
    ZeroScopeDepth,
    /// The rule family has an invalid structural shape.
    InvalidFamilyShape {
        /// Rule family under validation.
        kind: RuleKind,
        /// Expected premise count when fixed.
        expected: usize,
        /// Actual premise count.
        actual: usize,
    },
    /// A family requires a predicate term with a textual or variable form.
    InvalidPredicateTerm,
}

impl fmt::Display for RuleValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRuleId => formatter.write_str("rule identity must not be empty"),
            Self::NoPremises => formatter.write_str("rule must contain at least one premise"),
            Self::EmptyVariable => formatter.write_str("rule variable must not be empty"),
            Self::EmptyTextTerm => formatter.write_str("rule text term must not be empty"),
            Self::EmptyBindingType => formatter.write_str("rule binding type must not be empty"),
            Self::EmptyMetadataName => formatter.write_str("rule metadata name must not be empty"),
            Self::EmptyMetadataDescription => {
                formatter.write_str("rule metadata description must not be empty")
            }
            Self::UnboundBinding(variable) => write!(
                formatter,
                "rule binding is not introduced by a premise: {variable}"
            ),
            Self::UnboundConclusionVariable(variable) => {
                write!(
                    formatter,
                    "rule conclusion variable is not grounded: {variable}"
                )
            }
            Self::UnboundConditionVariable(variable) => {
                write!(
                    formatter,
                    "rule condition variable is not grounded: {variable}"
                )
            }
            Self::ZeroScopeDepth => {
                formatter.write_str("rule scope max depth must be greater than zero")
            }
            Self::InvalidFamilyShape {
                kind,
                expected,
                actual,
            } => write!(
                formatter,
                "rule family {kind} requires {expected} premise(s), got {actual}"
            ),
            Self::InvalidPredicateTerm => {
                formatter.write_str("rule predicate term must be textual or variable")
            }
        }
    }
}

impl std::error::Error for RuleValidationError {}

fn validate_variable(name: &str) -> Result<(), RuleValidationError> {
    if name.trim().is_empty() {
        return Err(RuleValidationError::EmptyVariable);
    }
    Ok(())
}

fn validate_condition(
    condition: &RuleCondition,
    premise_variables: &BTreeSet<&str>,
    declared_variables: &BTreeSet<&str>,
) -> Result<(), RuleValidationError> {
    let mut variables = BTreeSet::new();
    match condition {
        RuleCondition::Equal(left, right) | RuleCondition::NotEqual(left, right) => {
            variables.extend(left.variable_name());
            variables.extend(right.variable_name());
        }
        RuleCondition::PredicateIs(_) => {}
        RuleCondition::ObjectKind { variable, .. } => {
            variables.insert(variable.as_str());
        }
    }

    for variable in variables {
        if !premise_variables.contains(variable) && !declared_variables.contains(variable) {
            return Err(RuleValidationError::UnboundConditionVariable(
                variable.to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_family_shape(
    kind: RuleKind,
    premises: &[RulePattern],
    conclusion: &RulePattern,
) -> Result<(), RuleValidationError> {
    let fixed = match kind {
        RuleKind::Inverse | RuleKind::Symmetric => Some(1),
        RuleKind::Transitive | RuleKind::Composition | RuleKind::TypeInheritance => Some(2),
        RuleKind::PropertyInheritance | RuleKind::Conditional => None,
    };

    if let Some(expected) = fixed
        && premises.len() != expected
    {
        return Err(RuleValidationError::InvalidFamilyShape {
            kind,
            expected,
            actual: premises.len(),
        });
    }

    if matches!(conclusion.predicate(), RuleTerm::Object(_)) {
        return Err(RuleValidationError::InvalidPredicateTerm);
    }

    Ok(())
}

/// Runtime variable bindings produced by inference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariableBindings {
    values: BTreeMap<String, RuleTerm>,
}

impl VariableBindings {
    /// Creates an empty binding set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Inserts a binding, rejecting conflicting values.
    pub fn bind(&mut self, variable: impl Into<String>, value: RuleTerm) -> bool {
        let variable = variable.into();
        match self.values.get(&variable) {
            Some(existing) => existing == &value,
            None => {
                self.values.insert(variable, value);
                true
            }
        }
    }

    /// Returns a bound value.
    #[must_use]
    pub fn get(&self, variable: &str) -> Option<&RuleTerm> {
        self.values.get(variable)
    }

    /// Returns all bindings.
    #[must_use]
    pub fn values(&self) -> &BTreeMap<String, RuleTerm> {
        &self.values
    }
}

impl Default for VariableBindings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::RuleId;

    fn pattern() -> RulePattern {
        RulePattern::new(
            RuleTerm::variable("x").expect("valid variable"),
            RuleTerm::text("parent-of").expect("valid predicate"),
            RuleTerm::variable("y").expect("valid variable"),
        )
    }

    fn metadata() -> RuleMetadata {
        RuleMetadata::new("parent inverse", "test rule").expect("metadata should be valid")
    }

    #[test]
    fn rule_version_compatibility_is_major_version_based() {
        assert!(RuleVersion::new(1, 0, 0).compatible_with(RuleVersion::new(1, 9, 9)));
        assert!(!RuleVersion::new(1, 0, 0).compatible_with(RuleVersion::new(2, 0, 0)));
    }

    #[test]
    fn rule_rejects_ungrounded_conclusion_variables() {
        let conclusion = RulePattern::new(
            RuleTerm::variable("x").expect("valid variable"),
            RuleTerm::text("child-of").expect("valid predicate"),
            RuleTerm::variable("z").expect("valid variable"),
        );

        let error = Rule::new(
            RuleId::new("rule-1").expect("valid rule id"),
            RuleVersion::default(),
            RuleKind::Inverse,
            vec![pattern()],
            Vec::new(),
            Vec::new(),
            conclusion,
            RuleScope::unrestricted(),
            metadata(),
        )
        .expect_err("unbound conclusion variable must be rejected");

        assert!(matches!(
            error,
            RuleValidationError::UnboundConclusionVariable(_)
        ));
    }

    #[test]
    fn inverse_rule_accepts_one_premise() {
        let rule = Rule::new(
            RuleId::new("rule-1").expect("valid rule id"),
            RuleVersion::default(),
            RuleKind::Inverse,
            vec![pattern()],
            Vec::new(),
            Vec::new(),
            RulePattern::new(
                RuleTerm::variable("y").expect("valid variable"),
                RuleTerm::text("child-of").expect("valid predicate"),
                RuleTerm::variable("x").expect("valid variable"),
            ),
            RuleScope::unrestricted(),
            metadata(),
        )
        .expect("valid inverse rule");

        assert_eq!(rule.kind(), RuleKind::Inverse);
        assert_eq!(rule.version(), RuleVersion::default());
    }

    #[test]
    fn variable_bindings_reject_conflicting_values() {
        let mut bindings = VariableBindings::new();
        assert!(bindings.bind("x", RuleTerm::text("one").expect("valid text")));
        assert!(!bindings.bind("x", RuleTerm::text("two").expect("valid text")));
        assert_eq!(
            bindings.get("x"),
            Some(&RuleTerm::text("one").expect("valid text"))
        );
    }
}
