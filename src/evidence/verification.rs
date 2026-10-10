//! Verification records for evidence and other knowledge targets.
//!
//! Verification is represented as an independently addressable record rather
//! than a mutable `verified: bool`. Multiple records for the same target may
//! coexist, retaining different times, actors, outcomes, and evaluated inputs.

use core::fmt;
use std::collections::BTreeSet;

use crate::identity::{
    AgentId, EvidenceId, KnowledgeAssertionId, ReferenceId, SourceId, VerificationId,
};
use crate::temporal::Instant;

use super::model::{EvidenceSourceReference, EvidenceSourceReferenceError};

/// A target that a verification record evaluates.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VerificationTarget {
    /// An evidence item.
    Evidence(EvidenceId),
    /// A canonical knowledge assertion.
    Assertion(KnowledgeAssertionId),
    /// A source.
    Source(SourceId),
    /// A generic source/reference object.
    Reference(ReferenceId),
}

/// Actor that performed a verification.
///
/// `Agent` reuses the Phase 4 provenance identity boundary. `AutomatedSystem`
/// is available for a named tool/process where an agent record is not yet
/// available; it does not create a separate identity system.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VerificationPerformer {
    /// A person, system, or other agent represented by the provenance model.
    Agent(AgentId),
    /// An opaque name identifying an automated verifier.
    AutomatedSystem(String),
}

impl VerificationPerformer {
    /// Creates a named automated verifier after validating its name.
    pub fn automated_system(name: impl Into<String>) -> Result<Self, VerificationError> {
        let name = name.into();
        validate_name(&name)?;
        Ok(Self::AutomatedSystem(name))
    }

    fn validate(&self) -> Result<(), VerificationError> {
        if let Self::AutomatedSystem(name) = self {
            validate_name(name)?;
        }
        Ok(())
    }
}

/// Outcome of a verification activity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VerificationOutcome {
    /// The evaluated target was confirmed by this verification activity.
    Confirmed,
    /// The evaluated target was refuted by this verification activity.
    Refuted,
    /// The available information did not produce a conclusive result.
    Inconclusive,
    /// The target could not be evaluated by the performed procedure.
    UnableToVerify,
    /// The verification procedure determined that the check did not apply.
    NotApplicable,
}

/// An immutable-by-interface record of one verification activity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationRecord {
    id: VerificationId,
    target: VerificationTarget,
    performer: VerificationPerformer,
    outcome: VerificationOutcome,
    performed_at: Option<Instant>,
    evidence_considered: BTreeSet<EvidenceId>,
    source_references: BTreeSet<EvidenceSourceReference>,
    notes: Option<String>,
}

impl VerificationRecord {
    /// Creates one verification record.
    ///
    /// The record starts with no timestamp, notes, or considered evidence;
    /// consuming builders can add those fields without mutating prior records.
    pub fn new(
        id: VerificationId,
        target: VerificationTarget,
        performer: VerificationPerformer,
        outcome: VerificationOutcome,
    ) -> Result<Self, VerificationError> {
        performer.validate()?;
        Ok(Self {
            id,
            target,
            performer,
            outcome,
            performed_at: None,
            evidence_considered: BTreeSet::new(),
            source_references: BTreeSet::new(),
            notes: None,
        })
    }

    /// Adds the time at which this verification was performed.
    #[must_use]
    pub fn with_performed_at(mut self, instant: Instant) -> Self {
        self.performed_at = Some(instant);
        self
    }

    /// Records evidence considered by this verification activity.
    #[must_use]
    pub fn with_evidence_considered<I>(mut self, evidence: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.evidence_considered.extend(evidence);
        self
    }

    /// Records structured source locations considered during verification.
    pub fn with_source_references<I>(mut self, references: I) -> Self
    where
        I: IntoIterator<Item = EvidenceSourceReference>,
    {
        self.source_references.extend(references);
        self
    }

    /// Adds source references produced by fallible source-location constructors.
    ///
    /// If any reference is invalid, the validation error is returned rather than
    /// silently omitting that reference. Because this builder consumes `self`,
    /// a failed call does not expose a partially updated record.
    pub fn try_with_source_references<I>(
        mut self,
        references: I,
    ) -> Result<Self, EvidenceSourceReferenceError>
    where
        I: IntoIterator<Item = Result<EvidenceSourceReference, EvidenceSourceReferenceError>>,
    {
        for reference in references {
            self.source_references.insert(reference?);
        }

        Ok(self)
    }

    /// Adds optional explanatory notes. The value is retained verbatim.
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Returns the verification identity.
    #[must_use]
    pub fn id(&self) -> &VerificationId {
        &self.id
    }

    /// Returns the target evaluated by this verification.
    #[must_use]
    pub fn target(&self) -> &VerificationTarget {
        &self.target
    }

    /// Returns who or what performed the verification.
    #[must_use]
    pub fn performer(&self) -> &VerificationPerformer {
        &self.performer
    }

    /// Returns the preserved verification outcome.
    #[must_use]
    pub const fn outcome(&self) -> VerificationOutcome {
        self.outcome
    }

    /// Returns the recorded verification instant, if known.
    #[must_use]
    pub const fn performed_at(&self) -> Option<Instant> {
        self.performed_at
    }

    /// Returns the distinct evidence records considered.
    #[must_use]
    pub fn evidence_considered(&self) -> &BTreeSet<EvidenceId> {
        &self.evidence_considered
    }

    /// Returns structured source references considered during verification.
    #[must_use]
    pub fn source_references(&self) -> &BTreeSet<EvidenceSourceReference> {
        &self.source_references
    }

    /// Returns optional explanatory notes.
    #[must_use]
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

/// Verification-record construction errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationError {
    /// Automated verifier names must not be empty or whitespace-only.
    EmptyPerformerName,
    /// Automated verifier names must not contain control characters.
    PerformerControlCharacter { index: usize },
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPerformerName => {
                formatter.write_str("verification performer name must not be empty")
            }
            Self::PerformerControlCharacter { index } => write!(
                formatter,
                "verification performer name contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for VerificationError {}

fn validate_name(name: &str) -> Result<(), VerificationError> {
    if name.trim().is_empty() {
        return Err(VerificationError::EmptyPerformerName);
    }
    if let Some(index) = name.chars().position(char::is_control) {
        return Err(VerificationError::PerformerControlCharacter { index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        VerificationError, VerificationOutcome, VerificationPerformer, VerificationRecord,
        VerificationTarget,
    };
    use crate::evidence::{
        EvidenceSourceReference, EvidenceSourceReferenceError, TextOffsetUnit, TextSpan,
    };
    use crate::identity::{
        AgentId, EvidenceId, KnowledgeAssertionId, ReferenceId, SourceId, VerificationId,
    };
    use crate::temporal::Instant;

    #[test]
    fn verification_record_preserves_target_performer_and_outcome() {
        let agent = AgentId::new("agent-reviewer").expect("valid agent identity");
        let record = VerificationRecord::new(
            VerificationId::new("verification-1").expect("valid verification identity"),
            VerificationTarget::Evidence(
                EvidenceId::new("evidence-1").expect("valid evidence identity"),
            ),
            VerificationPerformer::Agent(agent.clone()),
            VerificationOutcome::Confirmed,
        )
        .expect("valid verification record");

        assert_eq!(
            record.target(),
            &VerificationTarget::Evidence(EvidenceId::new("evidence-1").unwrap())
        );
        assert_eq!(record.performer(), &VerificationPerformer::Agent(agent));
        assert_eq!(record.outcome(), VerificationOutcome::Confirmed);
    }

    #[test]
    fn verification_can_target_assertions_sources_and_references() {
        let performer = VerificationPerformer::automated_system("validator-v1")
            .expect("valid automated verifier");
        let targets = [
            VerificationTarget::Assertion(KnowledgeAssertionId::new("assertion-1").unwrap()),
            VerificationTarget::Source(SourceId::new("source-1").unwrap()),
            VerificationTarget::Reference(ReferenceId::new("reference-1").unwrap()),
        ];

        for (index, target) in targets.into_iter().enumerate() {
            let record = VerificationRecord::new(
                VerificationId::new(format!("verification-{index}")).unwrap(),
                target.clone(),
                performer.clone(),
                VerificationOutcome::Inconclusive,
            )
            .expect("valid verification record");
            assert_eq!(record.target(), &target);
        }
    }

    #[test]
    fn automated_verifier_names_are_validated() {
        assert_eq!(
            VerificationPerformer::automated_system(" "),
            Err(VerificationError::EmptyPerformerName)
        );
        assert!(VerificationPerformer::automated_system("bad\nverifier").is_err());
    }

    #[test]
    fn verification_preserves_time_evidence_source_location_and_notes() {
        let evidence_id = EvidenceId::new("evidence-1").expect("valid evidence identity");
        let source_ref = EvidenceSourceReference::text_span(
            ReferenceId::new("text-reference-1").expect("valid reference"),
            TextSpan::new(2, 8, TextOffsetUnit::UnicodeScalar).expect("valid text span"),
        );
        let instant = Instant::from_unix_seconds(1_750_000_000);
        let record = VerificationRecord::new(
            VerificationId::new("verification-detailed").expect("valid verification identity"),
            VerificationTarget::Evidence(evidence_id.clone()),
            VerificationPerformer::automated_system("text-checker")
                .expect("valid automated verifier"),
            VerificationOutcome::Refuted,
        )
        .expect("valid verification record")
        .with_performed_at(instant)
        .with_evidence_considered([evidence_id.clone(), evidence_id.clone()])
        .with_source_references([source_ref.clone()])
        .with_notes("The cited range does not support the assertion.");

        assert_eq!(record.performed_at(), Some(instant));
        assert_eq!(record.evidence_considered().len(), 1);
        assert!(record.evidence_considered().contains(&evidence_id));
        assert!(record.source_references().contains(&source_ref));
        assert_eq!(
            record.notes(),
            Some("The cited range does not support the assertion.")
        );
    }

    #[test]
    fn fallible_source_reference_builder_propagates_validation_errors() {
        let record = VerificationRecord::new(
            VerificationId::new("verification-invalid-source-ref").unwrap(),
            VerificationTarget::Source(SourceId::new("source-invalid-ref").unwrap()),
            VerificationPerformer::automated_system("source-checker").unwrap(),
            VerificationOutcome::Inconclusive,
        )
        .expect("valid verification record");

        let invalid_reference = EvidenceSourceReference::section(
            ReferenceId::new("reference-invalid-section").unwrap(),
            "   ",
        );

        let result = record.try_with_source_references([invalid_reference]);

        assert_eq!(
            result,
            Err(EvidenceSourceReferenceError::EmptyLocator { kind: "section" })
        );
    }

    #[test]
    fn multiple_verification_records_coexist_without_overwriting_history() {
        let target = VerificationTarget::Source(SourceId::new("source-1").unwrap());
        let first = VerificationRecord::new(
            VerificationId::new("verification-first").unwrap(),
            target.clone(),
            VerificationPerformer::automated_system("checker-v1").unwrap(),
            VerificationOutcome::Inconclusive,
        )
        .expect("first verification");
        let second = VerificationRecord::new(
            VerificationId::new("verification-second").unwrap(),
            target,
            VerificationPerformer::automated_system("checker-v2").unwrap(),
            VerificationOutcome::Confirmed,
        )
        .expect("second verification");

        assert_ne!(first.id(), second.id());
        assert_eq!(first.outcome(), VerificationOutcome::Inconclusive);
        assert_eq!(second.outcome(), VerificationOutcome::Confirmed);
    }

    #[test]
    fn all_verification_outcomes_are_distinct_values() {
        let outcomes = [
            VerificationOutcome::Confirmed,
            VerificationOutcome::Refuted,
            VerificationOutcome::Inconclusive,
            VerificationOutcome::UnableToVerify,
            VerificationOutcome::NotApplicable,
        ];
        let unique = outcomes
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 5);
    }
}
