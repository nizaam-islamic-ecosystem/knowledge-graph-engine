//! Typed reliability assessments for sources and knowledge-producing processes.
//!
//! Source reliability and process/extraction reliability are separate types.
//! The model deliberately stores named, optionally vocabulary-qualified values
//! instead of defining a universal numeric reliability scale.

use core::fmt;
use std::collections::BTreeSet;

use crate::identity::EvidenceId;

/// Errors raised while constructing a reliability assessment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReliabilityError {
    /// A required text field is empty or whitespace-only.
    EmptyField {
        /// Name of the field that failed validation.
        field: &'static str,
    },
    /// A text field contains a Unicode control character.
    ControlCharacter {
        /// Name of the field that failed validation.
        field: &'static str,
        /// Character index of the invalid value.
        index: usize,
    },
}

impl fmt::Display for ReliabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => {
                write!(formatter, "reliability {field} must not be empty")
            }
            Self::ControlCharacter { field, index } => write!(
                formatter,
                "reliability {field} contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for ReliabilityError {}

fn validate_text(value: &str, field: &'static str) -> Result<(), ReliabilityError> {
    if value.trim().is_empty() {
        return Err(ReliabilityError::EmptyField { field });
    }

    if let Some(index) = value.chars().position(char::is_control) {
        return Err(ReliabilityError::ControlCharacter { field, index });
    }

    Ok(())
}

/// A non-numeric reliability statement with optional basis and supporting evidence.
///
/// `label` is intentionally vocabulary-neutral. Values such as `reliable`,
/// `mixed`, or a domain-defined term are descriptions, not ordinal scores.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReliabilityAssessment {
    vocabulary: Option<String>,
    label: String,
    basis: Option<String>,
    evidence_ids: BTreeSet<EvidenceId>,
}

impl ReliabilityAssessment {
    /// Creates an unqualified reliability label.
    pub fn new(label: impl Into<String>) -> Result<Self, ReliabilityError> {
        let label = label.into();
        validate_text(&label, "label")?;

        Ok(Self {
            vocabulary: None,
            label,
            basis: None,
            evidence_ids: BTreeSet::new(),
        })
    }

    /// Creates a label belonging to an explicit vocabulary or evaluation scheme.
    pub fn in_vocabulary(
        vocabulary: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self, ReliabilityError> {
        let vocabulary = vocabulary.into();
        let label = label.into();
        validate_text(&vocabulary, "vocabulary")?;
        validate_text(&label, "label")?;

        Ok(Self {
            vocabulary: Some(vocabulary),
            label,
            basis: None,
            evidence_ids: BTreeSet::new(),
        })
    }

    /// Adds a concise explanation of the assessment basis.
    pub fn with_basis(mut self, basis: impl Into<String>) -> Result<Self, ReliabilityError> {
        let basis = basis.into();
        validate_text(&basis, "basis")?;
        self.basis = Some(basis);
        Ok(self)
    }

    /// Associates evidence identities considered by this assessment.
    #[must_use]
    pub fn with_evidence<I>(mut self, evidence_ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.evidence_ids.extend(evidence_ids);
        self
    }

    /// Returns the vocabulary, if one was supplied.
    #[must_use]
    pub fn vocabulary(&self) -> Option<&str> {
        self.vocabulary.as_deref()
    }

    /// Returns the descriptive reliability label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the optional rationale for the assessment.
    #[must_use]
    pub fn basis(&self) -> Option<&str> {
        self.basis.as_deref()
    }

    /// Returns the evidence identities considered by the assessment.
    #[must_use]
    pub fn evidence_ids(&self) -> &BTreeSet<EvidenceId> {
        &self.evidence_ids
    }
}

/// Reliability attributed to a source itself.
///
/// This is not interchangeable with process reliability: a trustworthy source
/// can still be extracted poorly, and a sound extraction process can be used
/// on a weak source.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceReliability {
    assessment: ReliabilityAssessment,
}

impl SourceReliability {
    /// Creates source reliability from a descriptive label.
    pub fn new(label: impl Into<String>) -> Result<Self, ReliabilityError> {
        Ok(Self {
            assessment: ReliabilityAssessment::new(label)?,
        })
    }

    /// Creates source reliability from a vocabulary-qualified label.
    pub fn in_vocabulary(
        vocabulary: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self, ReliabilityError> {
        Ok(Self {
            assessment: ReliabilityAssessment::in_vocabulary(vocabulary, label)?,
        })
    }

    /// Wraps an already validated assessment.
    #[must_use]
    pub fn from_assessment(assessment: ReliabilityAssessment) -> Self {
        Self { assessment }
    }

    /// Adds the assessment basis.
    pub fn with_basis(mut self, basis: impl Into<String>) -> Result<Self, ReliabilityError> {
        self.assessment = self.assessment.with_basis(basis)?;
        Ok(self)
    }

    /// Adds evidence considered by the source-reliability assessment.
    #[must_use]
    pub fn with_evidence<I>(mut self, evidence_ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.assessment = self.assessment.with_evidence(evidence_ids);
        self
    }

    /// Returns the underlying descriptive assessment.
    #[must_use]
    pub fn assessment(&self) -> &ReliabilityAssessment {
        &self.assessment
    }
}

/// Reliability attributed to a process, such as extraction or transformation.
///
/// The label is descriptive and is not combined automatically with
/// [`SourceReliability`] or any other authority dimension.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProcessReliability {
    assessment: ReliabilityAssessment,
}

impl ProcessReliability {
    /// Creates process reliability from a descriptive label.
    pub fn new(label: impl Into<String>) -> Result<Self, ReliabilityError> {
        Ok(Self {
            assessment: ReliabilityAssessment::new(label)?,
        })
    }

    /// Creates process reliability from a vocabulary-qualified label.
    pub fn in_vocabulary(
        vocabulary: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self, ReliabilityError> {
        Ok(Self {
            assessment: ReliabilityAssessment::in_vocabulary(vocabulary, label)?,
        })
    }

    /// Wraps an already validated assessment.
    #[must_use]
    pub fn from_assessment(assessment: ReliabilityAssessment) -> Self {
        Self { assessment }
    }

    /// Adds the assessment basis.
    pub fn with_basis(mut self, basis: impl Into<String>) -> Result<Self, ReliabilityError> {
        self.assessment = self.assessment.with_basis(basis)?;
        Ok(self)
    }

    /// Adds evidence considered by the process-reliability assessment.
    #[must_use]
    pub fn with_evidence<I>(mut self, evidence_ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.assessment = self.assessment.with_evidence(evidence_ids);
        self
    }

    /// Returns the underlying descriptive assessment.
    #[must_use]
    pub fn assessment(&self) -> &ReliabilityAssessment {
        &self.assessment
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessReliability, ReliabilityAssessment, ReliabilityError, SourceReliability};
    use crate::identity::EvidenceId;
    use std::any::TypeId;

    #[test]
    fn source_and_process_reliability_are_distinct_types_and_values() {
        let source = SourceReliability::new("reliable").expect("valid source reliability");
        let process = ProcessReliability::new("poor").expect("valid process reliability");

        assert_eq!(source.assessment().label(), "reliable");
        assert_eq!(process.assessment().label(), "poor");
        assert_ne!(
            TypeId::of::<SourceReliability>(),
            TypeId::of::<ProcessReliability>()
        );
    }

    #[test]
    fn reliability_labels_support_domain_vocabularies_and_basis() {
        let source = SourceReliability::in_vocabulary("source-review-v1", "mixed")
            .expect("valid vocabulary and label")
            .with_basis("Independent source review")
            .expect("valid basis");

        assert_eq!(source.assessment().vocabulary(), Some("source-review-v1"));
        assert_eq!(source.assessment().label(), "mixed");
        assert_eq!(
            source.assessment().basis(),
            Some("Independent source review")
        );
    }

    #[test]
    fn reliability_can_retain_evidence_basis_without_scoring_it() {
        let evidence = EvidenceId::new("evidence-source-review").expect("valid evidence ID");
        let assessment = ReliabilityAssessment::new("supported")
            .expect("valid assessment")
            .with_evidence([evidence.clone(), evidence.clone()]);

        assert_eq!(assessment.evidence_ids().len(), 1);
        assert!(assessment.evidence_ids().contains(&evidence));
    }

    #[test]
    fn empty_or_control_containing_reliability_text_is_rejected() {
        assert_eq!(
            ReliabilityAssessment::new("  "),
            Err(ReliabilityError::EmptyField { field: "label" })
        );
        assert!(matches!(
            ReliabilityAssessment::in_vocabulary("review\nsource", "reliable"),
            Err(ReliabilityError::ControlCharacter {
                field: "vocabulary",
                ..
            })
        ));
        assert!(SourceReliability::new("\t").is_err());
        assert!(ProcessReliability::new("extraction\u{0000}pipeline").is_err());
    }
}
