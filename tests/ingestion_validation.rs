//! Level 3 tests for candidate-scoped layered validation and severity policy.

use nizaam_knowledge_graph::identity::SourceId;
use nizaam_knowledge_graph::ingestion::{
    CandidateKey, ValidationFinding, ValidationPolicy, ValidationResult, ValidationSeverity,
    ValidationStage, ValidationStatus,
};

fn candidate_key(record: &str) -> CandidateKey {
    CandidateKey::new(
        SourceId::new("source-validation-level3").unwrap(),
        record,
        Some("snapshot-1".to_owned()),
        0,
    )
    .unwrap()
}

fn finding(code: &str, severity: ValidationSeverity, stage: ValidationStage) -> ValidationFinding {
    ValidationFinding::new(code, severity, stage, format!("finding: {code}")).unwrap()
}

#[test]
fn fatal_validation_finding_blocks_only_the_candidate_result() {
    let result = ValidationResult::new(
        candidate_key("fatal-record"),
        [finding(
            "malformed-required-field",
            ValidationSeverity::Fatal,
            ValidationStage::Structural,
        )],
    );
    assert_eq!(result.status(), ValidationStatus::Blocked);
    assert!(result.has_fatal_finding());
    assert!(!result.is_publishable());
    assert!(!result.is_quarantined());
}

#[test]
fn recoverable_errors_quarantine_while_warnings_remain_publishable() {
    let quarantined = ValidationResult::new(
        candidate_key("recoverable-record"),
        [finding(
            "missing-optional-reference",
            ValidationSeverity::Error,
            ValidationStage::Semantic,
        )],
    );
    assert_eq!(quarantined.status(), ValidationStatus::Quarantined);
    assert!(quarantined.is_quarantined());
    assert!(!quarantined.is_publishable());

    let warning = ValidationResult::new(
        candidate_key("warning-record"),
        [finding(
            "noncanonical-label",
            ValidationSeverity::Warning,
            ValidationStage::Semantic,
        )],
    );
    assert_eq!(warning.status(), ValidationStatus::ValidWithWarnings);
    assert!(warning.is_publishable());
    assert_eq!(warning.findings().len(), 1);
}

#[test]
fn configured_rule_severity_overrides_policy_default_deterministically() {
    let mut policy = ValidationPolicy::new();
    policy
        .set_rule_severity("known-optional-field", ValidationSeverity::Info)
        .unwrap();
    let first = policy
        .finding(
            "known-optional-field",
            ValidationStage::Structural,
            "field is not mandatory",
        )
        .unwrap();
    let second = policy
        .finding(
            "known-optional-field",
            ValidationStage::Structural,
            "field is not mandatory",
        )
        .unwrap();
    assert_eq!(first.severity(), ValidationSeverity::Info);
    assert_eq!(second.severity(), ValidationSeverity::Info);
}

#[test]
fn unconfigured_rules_use_the_validation_policys_default_severity() {
    let policy = ValidationPolicy::new().with_default_severity(ValidationSeverity::Warning);
    let finding = policy
        .finding(
            "unknown-rule",
            ValidationStage::Structural,
            "no override configured",
        )
        .unwrap();
    assert_eq!(finding.severity(), ValidationSeverity::Warning);

    let default_policy = ValidationPolicy::new();
    let finding = default_policy
        .finding(
            "another-unknown-rule",
            ValidationStage::Structural,
            "fallback to policy default",
        )
        .unwrap();
    assert_eq!(finding.severity(), ValidationSeverity::Error);
}

#[test]
fn duplicate_findings_do_not_duplicate_the_validation_report() {
    let finding = finding(
        "duplicate-finding",
        ValidationSeverity::Warning,
        ValidationStage::Semantic,
    );
    let result = ValidationResult::new(candidate_key("record-dedup"), [finding.clone(), finding]);
    assert_eq!(result.findings().len(), 1);
}
