//! Explainable confidence representations for evidence/support evaluation.
//!
//! Phase 4 stores confidence together with its basis and evaluation context. It
//! deliberately defines no universal formula, ranking policy, or evaluator.
//! Confidence is attached to a particular evidence/assertion support role so
//! two pieces of evidence may receive different assessments for the same claim.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::evidence::EvidenceRole;
use crate::identity::{EvidenceId, KnowledgeAssertionId, VerificationId};
use crate::temporal::Instant;

/// A validated numeric confidence score in the closed interval `[0, 1]`.
///
/// This is a representation only. It does not say how the score was computed.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ConfidenceScore(f64);

impl ConfidenceScore {
    /// Creates a score after validating that it is finite and in `[0, 1]`.
    pub fn new(value: f64) -> Result<Self, ConfidenceError> {
        if !value.is_finite() {
            return Err(ConfidenceError::NonFiniteNumericValue);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ConfidenceError::NumericValueOutOfRange);
        }
        Ok(Self(value))
    }

    /// Returns the represented numeric value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// A validated caller/domain-defined qualitative confidence label.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConfidenceLabel(String);

impl ConfidenceLabel {
    /// Creates a non-empty qualitative label.
    pub fn new(label: impl Into<String>) -> Result<Self, ConfidenceError> {
        let label = label.into();
        validate_label(&label, "confidence label")?;
        Ok(Self(label))
    }

    /// Returns the label verbatim.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A hybrid confidence representation: numeric, qualitative, or explicitly unknown.
#[derive(Clone, Debug, PartialEq)]
pub enum ConfidenceValue {
    /// A numeric score in `[0, 1]`, without an implied calculation formula.
    Numeric(ConfidenceScore),
    /// A domain-specific qualitative value.
    Qualitative(ConfidenceLabel),
    /// Confidence has not been established or is not available.
    Unknown,
}

impl ConfidenceValue {
    /// Creates a numeric confidence value.
    pub fn numeric(value: f64) -> Result<Self, ConfidenceError> {
        Ok(Self::Numeric(ConfidenceScore::new(value)?))
    }

    /// Creates a qualitative confidence value.
    pub fn qualitative(label: impl Into<String>) -> Result<Self, ConfidenceError> {
        Ok(Self::Qualitative(ConfidenceLabel::new(label)?))
    }

    /// Represents confidence as explicitly unknown.
    #[must_use]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    /// Returns a numeric score when this is the numeric variant.
    #[must_use]
    pub const fn as_numeric(&self) -> Option<ConfidenceScore> {
        match self {
            Self::Numeric(value) => Some(*value),
            Self::Qualitative(_) | Self::Unknown => None,
        }
    }

    /// Returns a qualitative label when this is the qualitative variant.
    #[must_use]
    pub fn as_qualitative(&self) -> Option<&str> {
        match self {
            Self::Qualitative(label) => Some(label.as_str()),
            Self::Numeric(_) | Self::Unknown => None,
        }
    }

    /// Returns whether confidence is explicitly unknown.
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// The assertion/evidence association whose support is being evaluated.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConfidenceTarget {
    assertion_id: KnowledgeAssertionId,
    evidence_id: EvidenceId,
    role: EvidenceRole,
}

impl ConfidenceTarget {
    /// Creates a target for one evidence role against one canonical assertion.
    pub fn new(
        assertion_id: KnowledgeAssertionId,
        evidence_id: EvidenceId,
        role: EvidenceRole,
    ) -> Result<Self, ConfidenceError> {
        role.validate()
            .map_err(ConfidenceError::InvalidEvidenceRole)?;
        Ok(Self {
            assertion_id,
            evidence_id,
            role,
        })
    }

    /// Returns the canonical assertion identity.
    #[must_use]
    pub fn assertion_id(&self) -> &KnowledgeAssertionId {
        &self.assertion_id
    }

    /// Returns the evidence identity.
    #[must_use]
    pub fn evidence_id(&self) -> &EvidenceId {
        &self.evidence_id
    }

    /// Returns the role being evaluated.
    #[must_use]
    pub fn role(&self) -> &EvidenceRole {
        &self.role
    }
}

/// Evidence and verification references, with an optional explanation, that form
/// the basis for a confidence assessment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfidenceBasis {
    evidence_ids: BTreeSet<EvidenceId>,
    verification_ids: BTreeSet<VerificationId>,
    explanation: Option<String>,
}

impl ConfidenceBasis {
    /// Creates a basis and rejects an entirely empty explanation/reference set.
    pub fn new<E, V>(
        evidence_ids: E,
        verification_ids: V,
        explanation: Option<String>,
    ) -> Result<Self, ConfidenceError>
    where
        E: IntoIterator<Item = EvidenceId>,
        V: IntoIterator<Item = VerificationId>,
    {
        let evidence_ids = evidence_ids.into_iter().collect::<BTreeSet<_>>();
        let verification_ids = verification_ids.into_iter().collect::<BTreeSet<_>>();
        if let Some(value) = explanation.as_deref() {
            validate_label(value, "confidence basis explanation")?;
        }
        if evidence_ids.is_empty() && verification_ids.is_empty() && explanation.is_none() {
            return Err(ConfidenceError::EmptyBasis);
        }

        Ok(Self {
            evidence_ids,
            verification_ids,
            explanation,
        })
    }

    /// Returns distinct evidence identities used as a basis.
    #[must_use]
    pub fn evidence_ids(&self) -> &BTreeSet<EvidenceId> {
        &self.evidence_ids
    }

    /// Returns distinct verification-record identities used as a basis.
    #[must_use]
    pub fn verification_ids(&self) -> &BTreeSet<VerificationId> {
        &self.verification_ids
    }

    /// Returns the optional explanation.
    #[must_use]
    pub fn explanation(&self) -> Option<&str> {
        self.explanation.as_deref()
    }
}

/// Context in which a confidence assessment was made.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfidenceContext {
    domain: Option<String>,
    profile: Option<String>,
    evaluator: Option<String>,
    method: Option<String>,
    evaluated_at: Option<Instant>,
    attributes: BTreeMap<String, String>,
}

impl ConfidenceContext {
    /// Creates an empty, but explicit, context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the domain whose interpretation applies to this assessment.
    pub fn with_domain(mut self, domain: impl Into<String>) -> Result<Self, ConfidenceError> {
        let value = domain.into();
        validate_label(&value, "confidence domain")?;
        self.domain = Some(value);
        Ok(self)
    }

    /// Adds the named evaluation profile, without executing it.
    pub fn with_profile(mut self, profile: impl Into<String>) -> Result<Self, ConfidenceError> {
        let value = profile.into();
        validate_label(&value, "confidence profile")?;
        self.profile = Some(value);
        Ok(self)
    }

    /// Records the actor/tool label that performed the assessment.
    pub fn with_evaluator(mut self, evaluator: impl Into<String>) -> Result<Self, ConfidenceError> {
        let value = evaluator.into();
        validate_label(&value, "confidence evaluator")?;
        self.evaluator = Some(value);
        Ok(self)
    }

    /// Records a caller-defined method label, not a calculation implementation.
    pub fn with_method(mut self, method: impl Into<String>) -> Result<Self, ConfidenceError> {
        let value = method.into();
        validate_label(&value, "confidence method")?;
        self.method = Some(value);
        Ok(self)
    }

    /// Adds the instant when the assessment was evaluated, if known.
    #[must_use]
    pub fn with_evaluated_at(mut self, instant: Instant) -> Self {
        self.evaluated_at = Some(instant);
        self
    }

    /// Adds a caller-defined context attribute.
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ConfidenceError> {
        let key = key.into();
        let value = value.into();
        validate_label(&key, "confidence context key")?;
        if let Some(index) = value.chars().position(char::is_control) {
            return Err(ConfidenceError::ControlCharacter {
                field: "confidence context value",
                index,
            });
        }
        self.attributes.insert(key, value);
        Ok(self)
    }

    /// Returns the domain label, if any.
    #[must_use]
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    /// Returns the profile label, if any.
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns the evaluator label, if any.
    #[must_use]
    pub fn evaluator(&self) -> Option<&str> {
        self.evaluator.as_deref()
    }

    /// Returns the method label, if any.
    #[must_use]
    pub fn method(&self) -> Option<&str> {
        self.method.as_deref()
    }

    /// Returns the evaluation instant, if known.
    #[must_use]
    pub const fn evaluated_at(&self) -> Option<Instant> {
        self.evaluated_at
    }

    /// Iterates over caller-defined context attributes in deterministic key order.
    pub fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.attributes
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

/// One confidence assessment attached to a particular evidence/assertion role.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfidenceAssessment {
    target: ConfidenceTarget,
    value: ConfidenceValue,
    basis: ConfidenceBasis,
    context: ConfidenceContext,
}

impl ConfidenceAssessment {
    /// Creates a confidence value with explicit basis and evaluation context.
    pub fn new(
        target: ConfidenceTarget,
        value: ConfidenceValue,
        basis: ConfidenceBasis,
        context: ConfidenceContext,
    ) -> Self {
        Self {
            target,
            value,
            basis,
            context,
        }
    }

    /// Returns the support/evidence target.
    #[must_use]
    pub fn target(&self) -> &ConfidenceTarget {
        &self.target
    }

    /// Returns the represented confidence value.
    #[must_use]
    pub fn value(&self) -> &ConfidenceValue {
        &self.value
    }

    /// Returns evidence/verification/context that explain the value.
    #[must_use]
    pub fn basis(&self) -> &ConfidenceBasis {
        &self.basis
    }

    /// Returns the evaluation context.
    #[must_use]
    pub fn context(&self) -> &ConfidenceContext {
        &self.context
    }
}

/// Structural failures while constructing confidence values or context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfidenceError {
    /// Numeric confidence must be finite.
    NonFiniteNumericValue,
    /// Numeric confidence must be within `[0, 1]`.
    NumericValueOutOfRange,
    /// A required source of support/explanation was not supplied.
    EmptyBasis,
    /// A text label was empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// Text contained a control character.
    ControlCharacter { field: &'static str, index: usize },
    /// An evidence role failed its own structural validation.
    InvalidEvidenceRole(crate::evidence::EvidenceRoleError),
}

impl fmt::Display for ConfidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteNumericValue => formatter.write_str("numeric confidence must be finite"),
            Self::NumericValueOutOfRange => {
                formatter.write_str("numeric confidence must be in [0, 1]")
            }
            Self::EmptyBasis => formatter.write_str(
                "confidence assessment requires evidence, verification, or an explanation",
            ),
            Self::EmptyLabel { field } => write!(formatter, "{field} must not be empty"),
            Self::ControlCharacter { field, index } => {
                write!(
                    formatter,
                    "{field} contains a control character at index {index}"
                )
            }
            Self::InvalidEvidenceRole(error) => {
                write!(formatter, "invalid confidence target role: {error}")
            }
        }
    }
}

impl std::error::Error for ConfidenceError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), ConfidenceError> {
    if value.trim().is_empty() {
        return Err(ConfidenceError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(ConfidenceError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ConfidenceAssessment, ConfidenceBasis, ConfidenceContext, ConfidenceError,
        ConfidenceTarget, ConfidenceValue,
    };
    use crate::evidence::EvidenceRole;
    use crate::identity::{EvidenceId, KnowledgeAssertionId, VerificationId};
    use crate::temporal::Instant;

    #[test]
    fn numeric_confidence_is_validated_without_a_calculation_formula() {
        assert!(ConfidenceValue::numeric(0.0).is_ok());
        assert!(ConfidenceValue::numeric(1.0).is_ok());
        assert_eq!(
            ConfidenceValue::numeric(1.01),
            Err(ConfidenceError::NumericValueOutOfRange)
        );
        assert_eq!(
            ConfidenceValue::numeric(f64::NAN),
            Err(ConfidenceError::NonFiniteNumericValue)
        );
    }

    #[test]
    fn qualitative_and_unknown_confidence_remain_available() {
        assert_eq!(
            ConfidenceValue::qualitative("high")
                .expect("valid qualitative confidence")
                .as_qualitative(),
            Some("high")
        );
        assert!(ConfidenceValue::unknown().is_unknown());
        assert!(ConfidenceValue::qualitative(" ").is_err());
    }

    #[test]
    fn confidence_preserves_target_basis_and_evaluation_context() {
        let assertion_id = KnowledgeAssertionId::new("assertion-1").unwrap();
        let evidence_id = EvidenceId::new("evidence-1").unwrap();
        let target = ConfidenceTarget::new(
            assertion_id.clone(),
            evidence_id.clone(),
            EvidenceRole::Supports,
        )
        .expect("valid confidence target");
        let verification_id = VerificationId::new("verification-1").unwrap();
        let basis = ConfidenceBasis::new(
            [evidence_id.clone()],
            [verification_id.clone()],
            Some("human-reviewed source span".to_owned()),
        )
        .expect("valid confidence basis");
        let context = ConfidenceContext::new()
            .with_domain("hadith")
            .expect("valid domain")
            .with_profile("initial-review")
            .expect("valid profile")
            .with_evaluator("reviewer-1")
            .expect("valid evaluator")
            .with_evaluated_at(Instant::from_unix_seconds(100));
        let assessment = ConfidenceAssessment::new(
            target,
            ConfidenceValue::numeric(0.75).unwrap(),
            basis,
            context,
        );

        assert_eq!(assessment.target().assertion_id(), &assertion_id);
        assert_eq!(assessment.target().evidence_id(), &evidence_id);
        assert_eq!(assessment.basis().verification_ids().len(), 1);
        assert_eq!(assessment.basis().evidence_ids().len(), 1);
        assert_eq!(assessment.context().domain(), Some("hadith"));
        assert_eq!(
            assessment.context().evaluated_at(),
            Some(Instant::from_unix_seconds(100))
        );
    }

    #[test]
    fn empty_confidence_basis_is_rejected() {
        assert_eq!(
            ConfidenceBasis::new(std::iter::empty(), std::iter::empty(), None,),
            Err(ConfidenceError::EmptyBasis)
        );
    }
}
