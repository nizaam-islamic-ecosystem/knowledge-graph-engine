//! Layered validation findings for Phase 5 ingestion candidates.
//!
//! Fatal findings block a candidate from publication. Recoverable errors place
//! only that candidate into quarantine. A contradiction can trigger governance
//! review, but it is not inherently a validation failure.

use core::fmt;
use std::collections::BTreeMap;

use super::mapping::{CandidateKey, MappedCandidate};
use super::raw::SourceRecordMetadataError;

/// Severity assigned to a validation finding by a policy.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValidationSeverity {
    /// Informational observation that does not affect progression.
    Info,
    /// Warning that should remain visible but does not block progression.
    Warning,
    /// Recoverable validation issue; quarantine/reprocessing may resolve it.
    Error,
    /// Non-recoverable finding; this candidate cannot be published.
    Fatal,
}

/// Logical stage at which a validation finding was produced.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValidationStage {
    /// Input envelope and source-record metadata checks before resolution.
    Structural,
    /// Semantic checks after entity resolution.
    Semantic,
    /// Ontology domain/range and constraint checks.
    Ontology,
    /// Source, document, or reference integrity checks.
    Reference,
    /// Temporal-value and temporal-validity checks.
    Temporal,
    /// Required source/evidence/provenance checks.
    EvidenceAndProvenance,
    /// Source-specific or future validation stage.
    Custom(String),
}

/// One deterministic, typed validation finding.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValidationFinding {
    code: String,
    severity: ValidationSeverity,
    stage: ValidationStage,
    message: String,
}

impl ValidationFinding {
    /// Creates a validated finding with a stable code and human-readable message.
    pub fn new(
        code: impl Into<String>,
        severity: ValidationSeverity,
        stage: ValidationStage,
        message: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let code = code.into();
        let message = message.into();
        validate_label(&code, "validation finding code")?;
        validate_label(&message, "validation finding message")?;
        if let ValidationStage::Custom(name) = &stage {
            validate_label(name, "custom validation stage")?;
        }
        Ok(Self {
            code,
            severity,
            stage,
            message,
        })
    }

    /// Returns the stable rule/finding code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the assigned severity.
    #[must_use]
    pub const fn severity(&self) -> ValidationSeverity {
        self.severity
    }

    /// Returns the stage that produced the finding.
    #[must_use]
    pub fn stage(&self) -> &ValidationStage {
        &self.stage
    }

    /// Returns the explanatory message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Candidate disposition after classifying its accumulated findings.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValidationStatus {
    /// No findings were recorded.
    Valid,
    /// Only informational findings/warnings were recorded.
    ValidWithWarnings,
    /// A recoverable error exists; quarantine and targeted reprocessing are allowed.
    Quarantined,
    /// A fatal finding exists; publication is forbidden.
    Blocked,
}

/// Aggregate validation result for exactly one candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    candidate: CandidateKey,
    findings: Vec<ValidationFinding>,
    status: ValidationStatus,
}

impl ValidationResult {
    /// Creates a deterministic result, sorting and deduplicating identical findings.
    #[must_use]
    pub fn new<I>(candidate: CandidateKey, findings: I) -> Self
    where
        I: IntoIterator<Item = ValidationFinding>,
    {
        let unique = findings
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let findings = unique.into_iter().collect::<Vec<_>>();
        let status = classify(&findings);
        Self {
            candidate,
            findings,
            status,
        }
    }

    /// Creates a successful result with no findings.
    #[must_use]
    pub fn valid(candidate: CandidateKey) -> Self {
        Self::new(candidate, std::iter::empty())
    }

    /// Returns the candidate to which the result applies.
    #[must_use]
    pub fn candidate(&self) -> &CandidateKey {
        &self.candidate
    }

    /// Returns the deterministic ordered findings.
    #[must_use]
    pub fn findings(&self) -> &[ValidationFinding] {
        &self.findings
    }

    /// Returns the aggregate candidate disposition.
    #[must_use]
    pub const fn status(&self) -> ValidationStatus {
        self.status
    }

    /// Returns whether the candidate may proceed to approval/publication checks.
    #[must_use]
    pub const fn is_publishable(&self) -> bool {
        matches!(
            self.status,
            ValidationStatus::Valid | ValidationStatus::ValidWithWarnings
        )
    }

    /// Returns whether at least one finding has fatal severity.
    #[must_use]
    pub fn has_fatal_finding(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.severity() == ValidationSeverity::Fatal)
    }

    /// Returns whether the result should be quarantined for recovery/reprocessing.
    #[must_use]
    pub const fn is_quarantined(&self) -> bool {
        matches!(self.status, ValidationStatus::Quarantined)
    }
}

/// Config-driven severity rules, with a supplied default for unknown rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationPolicy {
    overrides: BTreeMap<String, ValidationSeverity>,
    default_severity: ValidationSeverity,
}

impl ValidationPolicy {
    /// Creates a policy. Defaults to recoverable `Error` for unclassified rules.
    #[must_use]
    pub fn new() -> Self {
        Self {
            overrides: BTreeMap::new(),
            default_severity: ValidationSeverity::Error,
        }
    }

    /// Sets the default severity used when a rule has no override.
    #[must_use]
    pub fn with_default_severity(mut self, severity: ValidationSeverity) -> Self {
        self.default_severity = severity;
        self
    }

    /// Adds or replaces the severity for one stable rule code.
    pub fn set_rule_severity(
        &mut self,
        code: impl Into<String>,
        severity: ValidationSeverity,
    ) -> Result<(), ValidationError> {
        let code = code.into();
        validate_label(&code, "validation rule code")?;
        self.overrides.insert(code, severity);
        Ok(())
    }

    /// Returns the configured severity for a rule.
    #[must_use]
    pub fn severity_for(&self, code: &str) -> ValidationSeverity {
        self.overrides
            .get(code)
            .copied()
            .unwrap_or(self.default_severity)
    }

    /// Builds a finding using the policy's severity classification.
    pub fn finding(
        &self,
        code: impl Into<String>,
        default_severity: ValidationSeverity,
        stage: ValidationStage,
        message: impl Into<String>,
    ) -> Result<ValidationFinding, ValidationError> {
        let code = code.into();
        let severity = self
            .overrides
            .get(&code)
            .copied()
            .unwrap_or(default_severity);
        ValidationFinding::new(code, severity, stage, message)
    }
}

impl Default for ValidationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs the generic structural validation that is safe before entity resolution.
///
/// Domain-specific semantic/ontology checks are intentionally supplied by later
/// validation stages after the source records have passed resolution.
#[must_use]
pub fn validate_candidate_structure<T>(
    candidate: &MappedCandidate<T>,
    policy: &ValidationPolicy,
) -> ValidationResult {
    let findings = candidate.source().validate().err().map(|error| {
        policy
            .finding(
                "source-record-metadata-invalid",
                ValidationSeverity::Fatal,
                ValidationStage::Structural,
                error.to_string(),
            )
            .expect("static finding code and generated error message are valid labels")
    });
    ValidationResult::new(candidate.key().clone(), findings)
}

/// Errors while constructing or policy-classifying validation results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// A required rule code, stage label, or message is empty or contains controls.
    InvalidLabel { field: &'static str },
    /// Reserved for adapters that must report source metadata errors directly.
    InvalidSourceMetadata(SourceRecordMetadataError),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel { field } => write!(formatter, "{field} is invalid"),
            Self::InvalidSourceMetadata(error) => {
                write!(formatter, "invalid source metadata: {error}")
            }
        }
    }
}
impl std::error::Error for ValidationError {}

fn classify(findings: &[ValidationFinding]) -> ValidationStatus {
    if findings
        .iter()
        .any(|finding| finding.severity() == ValidationSeverity::Fatal)
    {
        ValidationStatus::Blocked
    } else if findings
        .iter()
        .any(|finding| finding.severity() == ValidationSeverity::Error)
    {
        ValidationStatus::Quarantined
    } else if findings.is_empty() {
        ValidationStatus::Valid
    } else {
        ValidationStatus::ValidWithWarnings
    }
}

fn validate_label(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(ValidationError::InvalidLabel { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ValidationFinding, ValidationPolicy, ValidationResult, ValidationSeverity, ValidationStage,
        ValidationStatus,
    };
    use crate::identity::SourceId;
    use crate::ingestion::mapping::CandidateKey;

    fn key() -> CandidateKey {
        CandidateKey::new(
            SourceId::new("source-validation").unwrap(),
            "record-1",
            None,
            0,
        )
        .unwrap()
    }

    fn finding(code: &str, severity: ValidationSeverity) -> ValidationFinding {
        ValidationFinding::new(code, severity, ValidationStage::Semantic, "test finding").unwrap()
    }

    #[test]
    fn fatal_findings_block_while_recoverable_errors_quarantine_only_the_candidate() {
        let error = ValidationResult::new(
            key(),
            [finding("missing-reference", ValidationSeverity::Error)],
        );
        assert_eq!(error.status(), ValidationStatus::Quarantined);
        assert!(!error.is_publishable());
        assert!(error.is_quarantined());

        let fatal = ValidationResult::new(
            key(),
            [finding("invalid-structure", ValidationSeverity::Fatal)],
        );
        assert_eq!(fatal.status(), ValidationStatus::Blocked);
        assert!(fatal.has_fatal_finding());
        assert!(!fatal.is_publishable());
    }

    #[test]
    fn warnings_remain_visible_without_blocking_publication() {
        let result = ValidationResult::new(
            key(),
            [finding(
                "optional-metadata-missing",
                ValidationSeverity::Warning,
            )],
        );
        assert_eq!(result.status(), ValidationStatus::ValidWithWarnings);
        assert!(result.is_publishable());
        assert_eq!(result.findings().len(), 1);
    }

    #[test]
    fn severity_is_configured_by_stable_rule_code() {
        let mut policy = ValidationPolicy::new();
        policy
            .set_rule_severity("optional-metadata-missing", ValidationSeverity::Info)
            .unwrap();
        let finding = policy
            .finding(
                "optional-metadata-missing",
                ValidationSeverity::Fatal,
                ValidationStage::Structural,
                "metadata is optional",
            )
            .unwrap();
        assert_eq!(finding.severity(), ValidationSeverity::Info);
    }

    #[test]
    fn identical_findings_are_deduplicated_deterministically() {
        let finding = finding("same-code", ValidationSeverity::Warning);
        let result = ValidationResult::new(key(), [finding.clone(), finding]);
        assert_eq!(result.findings().len(), 1);
    }
}
