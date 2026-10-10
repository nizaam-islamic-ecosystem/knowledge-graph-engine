//! Source-class approval policy and the minimal Phase 5 governance state machine.
//!
//! Human review is the initial approval mechanism. Source-class configuration
//! controls curation requirements and whether unverified/disputed/non-authentic
//! material may be represented; those allowances are explicit and never imply
//! endorsement or alter the source's Phase 4 authority metadata.

use core::fmt;
use std::collections::BTreeMap;

use crate::identity::AgentId;
use crate::temporal::Instant;

use super::mapping::{CandidateKey, MappedCandidate};
use super::raw::SourceClass;
use super::validation::{ValidationResult, ValidationStatus};

/// Whether a source class requires additional curation beyond human approval.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurationRequirement {
    /// Curatorial review must be recorded before publication.
    Required,
    /// Curation may be performed, but is not a mandatory publication gate.
    Optional,
}

/// Result of curation for one candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurationOutcome {
    /// The configured policy does not require separate curation.
    NotRequired,
    /// A curation action was completed and recorded.
    Performed,
    /// Curation was deferred; publication is allowed only for optional curation.
    Deferred,
}

/// Epistemic assessment of source authenticity supplied to governance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceAuthenticity {
    /// The source material is treated as authentic under the supplied assessment.
    Authentic,
    /// Authenticity has not yet been established.
    Unverified,
    /// The source or material is explicitly disputed.
    Disputed,
    /// The material is known to be non-authentic but may be preserved under policy.
    NonAuthentic,
}

/// Source-class governance settings.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceApprovalRule {
    curation: CurationRequirement,
    allow_unverified: bool,
    allow_disputed: bool,
    allow_non_authentic: bool,
}

impl SourceApprovalRule {
    /// Creates a source-class rule. Human approval remains mandatory regardless
    /// of these preservation/curation settings.
    #[must_use]
    pub const fn new(
        curation: CurationRequirement,
        allow_unverified: bool,
        allow_disputed: bool,
        allow_non_authentic: bool,
    ) -> Self {
        Self {
            curation,
            allow_unverified,
            allow_disputed,
            allow_non_authentic,
        }
    }

    /// Returns the curation requirement.
    #[must_use]
    pub const fn curation(&self) -> CurationRequirement {
        self.curation
    }

    /// Returns whether unverified material may be considered for publication.
    #[must_use]
    pub const fn allows_unverified(&self) -> bool {
        self.allow_unverified
    }

    /// Returns whether disputed material may be considered for publication.
    #[must_use]
    pub const fn allows_disputed(&self) -> bool {
        self.allow_disputed
    }

    /// Returns whether non-authentic material may be represented with explicit labels.
    #[must_use]
    pub const fn allows_non_authentic(&self) -> bool {
        self.allow_non_authentic
    }
}

impl Default for SourceApprovalRule {
    fn default() -> Self {
        Self::new(CurationRequirement::Optional, false, false, false)
    }
}

/// Config-driven, source-class-specific approval policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalPolicy {
    default_rule: SourceApprovalRule,
    source_rules: BTreeMap<SourceClass, SourceApprovalRule>,
    configuration_version: String,
}

impl ApprovalPolicy {
    /// Creates an approval policy with a source-class fallback rule.
    pub fn new(
        configuration_version: impl Into<String>,
        default_rule: SourceApprovalRule,
    ) -> Result<Self, ApprovalError> {
        let configuration_version = configuration_version.into();
        validate_label(&configuration_version, "approval policy version")?;
        Ok(Self {
            default_rule,
            source_rules: BTreeMap::new(),
            configuration_version,
        })
    }

    /// Creates the conservative Phase 5 default: human approval is always needed,
    /// curation is optional, and unverified/disputed/non-authentic material is
    /// not eligible for publication unless a source-specific rule allows it.
    #[must_use]
    pub fn phase5_default() -> Self {
        Self {
            default_rule: SourceApprovalRule::default(),
            source_rules: BTreeMap::new(),
            configuration_version: "phase5-approval-v1".to_owned(),
        }
    }

    /// Adds/replaces the rule for one source class.
    pub fn set_rule(&mut self, source_class: SourceClass, rule: SourceApprovalRule) {
        self.source_rules.insert(source_class, rule);
    }

    /// Returns the configured rule, falling back to the default for unknown classes.
    #[must_use]
    pub fn rule_for(&self, source_class: &SourceClass) -> &SourceApprovalRule {
        self.source_rules
            .get(source_class)
            .unwrap_or(&self.default_rule)
    }

    /// Returns the policy configuration version used for reproducibility.
    #[must_use]
    pub fn configuration_version(&self) -> &str {
        &self.configuration_version
    }

    /// Validates the candidate's validation and human decision against this policy.
    pub fn validate_for_publication<T>(
        &self,
        candidate: &MappedCandidate<T>,
        validation: &ValidationResult,
        decision: &ApprovalDecision,
        authenticity: SourceAuthenticity,
    ) -> Result<(), ApprovalError> {
        if validation.candidate() != candidate.key() {
            return Err(ApprovalError::CandidateMismatch);
        }
        if !validation.is_publishable() {
            return Err(match validation.status() {
                ValidationStatus::Blocked => ApprovalError::ValidationBlocked,
                ValidationStatus::Quarantined => ApprovalError::ValidationQuarantined,
                ValidationStatus::Valid | ValidationStatus::ValidWithWarnings => {
                    ApprovalError::ValidationNotPublishable
                }
            });
        }
        if decision.candidate() != candidate.key()
            || decision.source_class() != candidate.source().source_class()
        {
            return Err(ApprovalError::ApprovalTargetMismatch);
        }
        if decision.outcome() != ApprovalOutcome::Approved {
            return Err(ApprovalError::NotApproved);
        }

        let rule = self.rule_for(candidate.source().source_class());
        match authenticity {
            SourceAuthenticity::Authentic => {}
            SourceAuthenticity::Unverified if rule.allows_unverified() => {}
            SourceAuthenticity::Disputed if rule.allows_disputed() => {}
            SourceAuthenticity::NonAuthentic if rule.allows_non_authentic() => {}
            SourceAuthenticity::Unverified
            | SourceAuthenticity::Disputed
            | SourceAuthenticity::NonAuthentic => {
                return Err(ApprovalError::AuthenticityNotPermitted { authenticity });
            }
        }
        if rule.curation() == CurationRequirement::Required
            && decision.curation() != CurationOutcome::Performed
        {
            return Err(ApprovalError::CurationRequired);
        }
        Ok(())
    }
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self::phase5_default()
    }
}

/// Human approval decision recorded for one candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalDecision {
    candidate: CandidateKey,
    source_class: SourceClass,
    outcome: ApprovalOutcome,
    reviewer: AgentId,
    decided_at: Instant,
    curation: CurationOutcome,
    rationale: Option<String>,
}

impl ApprovalDecision {
    /// Creates a validated human decision. The caller must identify the reviewer
    /// using the Phase 4/Core-backed agent identity.
    pub fn new(
        candidate: CandidateKey,
        source_class: SourceClass,
        outcome: ApprovalOutcome,
        reviewer: AgentId,
        decided_at: Instant,
        curation: CurationOutcome,
        rationale: Option<String>,
    ) -> Result<Self, ApprovalError> {
        if let Some(rationale) = &rationale {
            validate_label(rationale, "approval rationale")?;
        }
        Ok(Self {
            candidate,
            source_class,
            outcome,
            reviewer,
            decided_at,
            curation,
            rationale,
        })
    }

    /// Returns the target candidate.
    #[must_use]
    pub fn candidate(&self) -> &CandidateKey {
        &self.candidate
    }

    /// Returns the source class used when applying policy.
    #[must_use]
    pub fn source_class(&self) -> &SourceClass {
        &self.source_class
    }

    /// Returns whether the reviewer approved or rejected the candidate.
    #[must_use]
    pub const fn outcome(&self) -> ApprovalOutcome {
        self.outcome
    }

    /// Returns the reviewer identity.
    #[must_use]
    pub fn reviewer(&self) -> &AgentId {
        &self.reviewer
    }

    /// Returns when the decision was recorded.
    #[must_use]
    pub const fn decided_at(&self) -> Instant {
        self.decided_at
    }

    /// Returns the curation outcome accompanying the human decision.
    #[must_use]
    pub const fn curation(&self) -> CurationOutcome {
        self.curation
    }

    /// Returns the recorded rationale, if supplied.
    #[must_use]
    pub fn rationale(&self) -> Option<&str> {
        self.rationale.as_deref()
    }
}

/// Outcome of a human approval decision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApprovalOutcome {
    /// The reviewer permits the candidate to continue to publication gating.
    Approved,
    /// The reviewer rejects this candidate; the run may continue for other candidates.
    Rejected,
}

/// Minimal governance state associated with one candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GovernanceState {
    /// Structural and semantic validation has not completed.
    AwaitingValidation,
    /// A recoverable issue is awaiting targeted correction/reprocessing.
    Quarantined,
    /// A fatal finding blocks publication.
    Blocked,
    /// Validation passed; explicit human review is pending.
    AwaitingApproval,
    /// A human reviewer approved the candidate.
    Approved,
    /// A human reviewer rejected the candidate.
    Rejected,
    /// An earlier published record was withdrawn through a governed operation.
    Withdrawn,
}

impl GovernanceState {
    /// Returns whether no ordinary transition can leave this state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Blocked | Self::Rejected | Self::Withdrawn)
    }
}

/// Actor responsible for a governance transition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GovernanceActor {
    /// Human reviewer represented by an existing agent identity.
    Human(AgentId),
    /// Ingestion system action, such as quarantine or validation outcome.
    System(String),
}

/// Immutable history item for a single governance state transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceTransition {
    from: GovernanceState,
    to: GovernanceState,
    at: Instant,
    actor: GovernanceActor,
    reason: String,
}

impl GovernanceTransition {
    /// Returns the previous state.
    #[must_use]
    pub const fn from(&self) -> GovernanceState {
        self.from
    }
    /// Returns the next state.
    #[must_use]
    pub const fn to(&self) -> GovernanceState {
        self.to
    }
    /// Returns the transition time.
    #[must_use]
    pub const fn at(&self) -> Instant {
        self.at
    }
    /// Returns the actor that caused the transition.
    #[must_use]
    pub fn actor(&self) -> &GovernanceActor {
        &self.actor
    }
    /// Returns the transition reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Checked governance state machine for a candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceRecord {
    candidate: CandidateKey,
    state: GovernanceState,
    history: Vec<GovernanceTransition>,
}

impl GovernanceRecord {
    /// Creates a governance record before validation begins.
    #[must_use]
    pub fn new(candidate: CandidateKey) -> Self {
        Self {
            candidate,
            state: GovernanceState::AwaitingValidation,
            history: Vec::new(),
        }
    }

    /// Returns the candidate identity.
    #[must_use]
    pub fn candidate(&self) -> &CandidateKey {
        &self.candidate
    }
    /// Returns the current governance state.
    #[must_use]
    pub const fn state(&self) -> GovernanceState {
        self.state
    }
    /// Returns immutable transition history.
    #[must_use]
    pub fn history(&self) -> &[GovernanceTransition] {
        &self.history
    }

    /// Applies an allowed transition and appends an immutable history entry.
    pub fn transition(
        &mut self,
        next: GovernanceState,
        at: Instant,
        actor: GovernanceActor,
        reason: impl Into<String>,
    ) -> Result<(), ApprovalError> {
        let reason = reason.into();
        validate_label(&reason, "governance transition reason")?;
        if !allowed_transition(self.state, next) {
            return Err(ApprovalError::InvalidGovernanceTransition {
                from: self.state,
                to: next,
            });
        }
        let transition = GovernanceTransition {
            from: self.state,
            to: next,
            at,
            actor,
            reason,
        };
        self.history.push(transition);
        self.state = next;
        Ok(())
    }
}

fn allowed_transition(from: GovernanceState, to: GovernanceState) -> bool {
    use GovernanceState as S;
    matches!(
        (from, to),
        (
            S::AwaitingValidation,
            S::Quarantined | S::Blocked | S::AwaitingApproval | S::Rejected
        ) | (
            S::Quarantined,
            S::AwaitingValidation | S::AwaitingApproval | S::Rejected | S::Blocked
        ) | (
            S::AwaitingApproval,
            S::Approved | S::Rejected | S::Quarantined | S::Blocked
        ) | (S::Approved, S::Withdrawn)
    )
}

/// Approval and governance errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalError {
    /// A required policy/rationale label is invalid.
    InvalidLabel { field: &'static str },
    /// A validation result belongs to a different candidate.
    CandidateMismatch,
    /// A fatal validation finding blocks publication.
    ValidationBlocked,
    /// Recoverable findings require quarantine/reprocessing first.
    ValidationQuarantined,
    /// A candidate is not in a publishable validation state.
    ValidationNotPublishable,
    /// The approval record belongs to another candidate or source class.
    ApprovalTargetMismatch,
    /// The candidate does not have an approved decision.
    NotApproved,
    /// A required curation step was not completed.
    CurationRequired,
    /// The configured source-class policy does not allow this authenticity state.
    AuthenticityNotPermitted { authenticity: SourceAuthenticity },
    /// The proposed governance-state transition is not allowed.
    InvalidGovernanceTransition {
        from: GovernanceState,
        to: GovernanceState,
    },
}

impl fmt::Display for ApprovalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel { field } => write!(formatter, "{field} is invalid"),
            Self::CandidateMismatch => {
                formatter.write_str("validation result targets a different candidate")
            }
            Self::ValidationBlocked => {
                formatter.write_str("fatal validation finding blocks publication")
            }
            Self::ValidationQuarantined => {
                formatter.write_str("candidate is quarantined and requires reprocessing")
            }
            Self::ValidationNotPublishable => {
                formatter.write_str("candidate has not passed validation")
            }
            Self::ApprovalTargetMismatch => formatter
                .write_str("approval decision target or source class does not match candidate"),
            Self::NotApproved => {
                formatter.write_str("candidate has not been approved by a human reviewer")
            }
            Self::CurationRequired => {
                formatter.write_str("source-class policy requires curation before publication")
            }
            Self::AuthenticityNotPermitted { authenticity } => write!(
                formatter,
                "source-class policy does not allow authenticity state {authenticity:?}"
            ),
            Self::InvalidGovernanceTransition { from, to } => write!(
                formatter,
                "invalid governance transition: {from:?} -> {to:?}"
            ),
        }
    }
}
impl std::error::Error for ApprovalError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), ApprovalError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(ApprovalError::InvalidLabel { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ApprovalDecision, ApprovalOutcome, ApprovalPolicy, CurationOutcome, CurationRequirement,
        GovernanceActor, GovernanceRecord, GovernanceState, SourceApprovalRule, SourceAuthenticity,
    };
    use crate::identity::{AgentId, SourceId};
    use crate::ingestion::mapping::CandidateKey;
    use crate::ingestion::raw::SourceClass;
    use crate::ingestion::validation::ValidationResult;
    use crate::temporal::Instant;

    fn candidate_key() -> CandidateKey {
        CandidateKey::new(
            SourceId::new("source-approval").unwrap(),
            "record-1",
            None,
            0,
        )
        .unwrap()
    }

    fn decision(outcome: ApprovalOutcome, curation: CurationOutcome) -> ApprovalDecision {
        ApprovalDecision::new(
            candidate_key(),
            SourceClass::structured_data(),
            outcome,
            AgentId::new("reviewer-1").unwrap(),
            Instant::from_unix_seconds(5),
            curation,
            Some("reviewed source context".to_owned()),
        )
        .unwrap()
    }

    #[test]
    fn conservative_policy_requires_an_explicit_approved_decision() {
        let policy = ApprovalPolicy::phase5_default();
        let validation = ValidationResult::valid(candidate_key());
        let mut rules_policy = policy.clone();
        rules_policy.set_rule(
            SourceClass::structured_data(),
            SourceApprovalRule::new(CurationRequirement::Optional, false, false, false),
        );
        assert!(
            rules_policy
                .validate_for_publication(
                    &test_candidate::candidate(),
                    &validation,
                    &decision(ApprovalOutcome::Approved, CurationOutcome::Deferred),
                    SourceAuthenticity::Authentic,
                )
                .is_ok()
        );
        assert!(
            policy
                .validate_for_publication(
                    &test_candidate::candidate(),
                    &validation,
                    &decision(ApprovalOutcome::Rejected, CurationOutcome::NotRequired),
                    SourceAuthenticity::Authentic,
                )
                .is_err()
        );
    }

    #[test]
    fn non_authentic_content_requires_explicit_source_class_allowance() {
        let mut policy = ApprovalPolicy::phase5_default();
        policy.set_rule(
            SourceClass::structured_data(),
            SourceApprovalRule::new(CurationRequirement::Optional, false, false, true),
        );
        assert!(
            policy
                .rule_for(&SourceClass::structured_data())
                .allows_non_authentic()
        );
        assert!(
            !policy
                .rule_for(&SourceClass::structured_data())
                .allows_disputed()
        );
    }

    #[test]
    fn governance_transitions_are_checked_and_history_is_preserved() {
        let mut record = GovernanceRecord::new(candidate_key());
        record
            .transition(
                GovernanceState::AwaitingApproval,
                Instant::from_unix_seconds(1),
                GovernanceActor::System("validation-stage".to_owned()),
                "validation passed",
            )
            .unwrap();
        record
            .transition(
                GovernanceState::Approved,
                Instant::from_unix_seconds(2),
                GovernanceActor::Human(AgentId::new("reviewer-1").unwrap()),
                "approved",
            )
            .unwrap();
        assert!(
            record
                .transition(
                    GovernanceState::Quarantined,
                    Instant::from_unix_seconds(3),
                    GovernanceActor::System("later-stage".to_owned()),
                    "invalid backwards transition",
                )
                .is_err()
        );
        assert_eq!(record.state(), GovernanceState::Approved);
        assert_eq!(record.history().len(), 2);
    }

    // A tiny candidate payload keeps this test focused on governance contracts.
    mod test_candidate {
        use crate::identity::SourceId;
        use crate::ingestion::mapping::{MappedCandidate, MappingMetadata};
        use crate::ingestion::normalize::{NormalizationMetadata, NormalizedRecord};
        use crate::ingestion::raw::{SourceClass, SourceRecordMetadata};
        use crate::temporal::Instant;

        pub(super) fn candidate() -> MappedCandidate<()> {
            let record = NormalizedRecord::new(
                SourceRecordMetadata::new(
                    SourceId::new("source-approval").unwrap(),
                    SourceClass::structured_data(),
                    "record-1",
                    None,
                    None,
                    [],
                )
                .unwrap(),
                (),
                NormalizationMetadata::new(
                    "normalize-v1",
                    "config-v1",
                    Instant::from_unix_seconds(1),
                )
                .unwrap(),
            )
            .unwrap();
            MappedCandidate::from_normalized(
                record,
                0,
                (),
                MappingMetadata::new("mapping-v1", "config-v1").unwrap(),
            )
            .unwrap()
        }
    }
}
