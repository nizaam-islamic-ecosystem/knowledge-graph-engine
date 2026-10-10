//! Multi-dimensional authority metadata for sources and assertions.
//!
//! Authority is descriptive semantic metadata, not runtime authorization and
//! not a universal trust score. Multiple distinct values can coexist, and
//! domain-specific evaluation profiles are recorded without running a scoring
//! algorithm in Phase 4.

use core::fmt;
use std::collections::BTreeSet;

use crate::identity::{EvidenceId, KnowledgeAssertionId, SourceId};

use super::reliability::{ProcessReliability, SourceReliability};
use super::scholarly::ScholarlyStatus;

/// The semantic object to which authority metadata applies.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuthorityTarget {
    /// Authority information about a source as a whole.
    Source(SourceId),
    /// Authority information about one canonical knowledge assertion.
    Assertion(KnowledgeAssertionId),
}

/// A generic, optionally vocabulary-qualified authority value.
///
/// It is intended for dimensions such as authentication, verification, human
/// review, machine extraction, curation status, and source authority. Numeric
/// weights and global scores are intentionally not part of this type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AuthorityValue {
    vocabulary: Option<String>,
    value: String,
    rationale: Option<String>,
    evidence_ids: BTreeSet<EvidenceId>,
}

impl AuthorityValue {
    /// Creates an unqualified generic authority value.
    pub fn new(value: impl Into<String>) -> Result<Self, AuthorityError> {
        let value = value.into();
        validate_text(&value, "authority value")?;

        Ok(Self {
            vocabulary: None,
            value,
            rationale: None,
            evidence_ids: BTreeSet::new(),
        })
    }

    /// Creates a value qualified by a domain vocabulary or evaluation scheme.
    pub fn in_vocabulary(
        vocabulary: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, AuthorityError> {
        let vocabulary = vocabulary.into();
        let value = value.into();
        validate_text(&vocabulary, "authority vocabulary")?;
        validate_text(&value, "authority value")?;

        Ok(Self {
            vocabulary: Some(vocabulary),
            value,
            rationale: None,
            evidence_ids: BTreeSet::new(),
        })
    }

    /// Adds the rationale or descriptive basis for the value.
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Result<Self, AuthorityError> {
        let rationale = rationale.into();
        validate_text(&rationale, "authority rationale")?;
        self.rationale = Some(rationale);
        Ok(self)
    }

    /// Associates evidence identities supporting this metadata value.
    #[must_use]
    pub fn with_evidence<I>(mut self, evidence_ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.evidence_ids.extend(evidence_ids);
        self
    }

    /// Returns the vocabulary, if supplied.
    #[must_use]
    pub fn vocabulary(&self) -> Option<&str> {
        self.vocabulary.as_deref()
    }

    /// Returns the stored value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the descriptive rationale, if supplied.
    #[must_use]
    pub fn rationale(&self) -> Option<&str> {
        self.rationale.as_deref()
    }

    /// Returns the associated evidence identities.
    #[must_use]
    pub fn evidence_ids(&self) -> &BTreeSet<EvidenceId> {
        &self.evidence_ids
    }
}

/// A typed authority dimension and its preserved value.
///
/// Reliability and scholarly status use their dedicated types so source
/// reliability, process reliability, and scholarly classifications cannot be
/// accidentally treated as interchangeable text labels.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuthorityDimension {
    /// Source-level authority designation.
    SourceAuthority(AuthorityValue),
    /// Reliability attributed to the source itself.
    SourceReliability(SourceReliability),
    /// Authentication designation.
    Authentication(AuthorityValue),
    /// Scholarly status from a generic or domain-specific vocabulary.
    ScholarlyStatus(ScholarlyStatus),
    /// Verification designation; verification records remain in the evidence module.
    Verification(AuthorityValue),
    /// Human-review designation.
    HumanReview(AuthorityValue),
    /// Machine-extraction designation.
    MachineExtraction(AuthorityValue),
    /// Curation status designation.
    CurationStatus(AuthorityValue),
    /// Reliability attributed to a process or extraction step.
    ProcessReliability(ProcessReliability),
    /// A future/domain-specific authority dimension.
    Extension {
        /// Domain-defined dimension name.
        name: String,
        /// Value for that dimension.
        value: AuthorityValue,
    },
}

impl AuthorityDimension {
    /// Creates a validated domain-specific authority dimension.
    pub fn extension(
        name: impl Into<String>,
        value: AuthorityValue,
    ) -> Result<Self, AuthorityError> {
        let name = name.into();
        validate_text(&name, "authority extension name")?;
        Ok(Self::Extension { name, value })
    }

    /// Returns the stable core dimension name or the extension's supplied name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::SourceAuthority(_) => "source-authority",
            Self::SourceReliability(_) => "source-reliability",
            Self::Authentication(_) => "authentication",
            Self::ScholarlyStatus(_) => "scholarly-status",
            Self::Verification(_) => "verification",
            Self::HumanReview(_) => "human-review",
            Self::MachineExtraction(_) => "machine-extraction",
            Self::CurationStatus(_) => "curation-status",
            Self::ProcessReliability(_) => "process-reliability",
            Self::Extension { name, .. } => name,
        }
    }

    fn validate(&self) -> Result<(), AuthorityError> {
        if let Self::Extension { name, .. } = self {
            validate_text(name, "authority extension name")?;
        }
        Ok(())
    }
}

/// Metadata describing how a domain-specific profile interprets authority values.
///
/// This records a profile identity and optional domain/version labels. It does
/// not evaluate the profile or calculate a score.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AuthorityEvaluationProfile {
    vocabulary: String,
    name: String,
    domain: Option<String>,
    version: Option<String>,
}

impl AuthorityEvaluationProfile {
    /// Creates a named profile within an explicit vocabulary.
    pub fn new(
        vocabulary: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, AuthorityError> {
        let vocabulary = vocabulary.into();
        let name = name.into();
        validate_text(&vocabulary, "evaluation-profile vocabulary")?;
        validate_text(&name, "evaluation-profile name")?;

        Ok(Self {
            vocabulary,
            name,
            domain: None,
            version: None,
        })
    }

    /// Adds the domain for which this profile is intended.
    pub fn with_domain(mut self, domain: impl Into<String>) -> Result<Self, AuthorityError> {
        let domain = domain.into();
        validate_text(&domain, "evaluation-profile domain")?;
        self.domain = Some(domain);
        Ok(self)
    }

    /// Adds the profile version as an opaque label.
    pub fn with_version(mut self, version: impl Into<String>) -> Result<Self, AuthorityError> {
        let version = version.into();
        validate_text(&version, "evaluation-profile version")?;
        self.version = Some(version);
        Ok(self)
    }

    /// Returns the profile vocabulary.
    #[must_use]
    pub fn vocabulary(&self) -> &str {
        &self.vocabulary
    }

    /// Returns the profile name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the profile's domain, if supplied.
    #[must_use]
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    /// Returns the profile's version label, if supplied.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

/// Multi-dimensional authority metadata attached to one source or assertion.
///
/// Distinct values are stored in a deterministic set. Repeating the exact same
/// entry is idempotent; different assessments of one dimension can coexist.
/// This type exposes no global trust score and performs no implicit evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authority {
    target: AuthorityTarget,
    dimensions: BTreeSet<AuthorityDimension>,
    evaluation_profiles: BTreeSet<AuthorityEvaluationProfile>,
}

impl Authority {
    /// Creates authority metadata for one semantic target.
    #[must_use]
    pub fn new(target: AuthorityTarget) -> Self {
        Self {
            target,
            dimensions: BTreeSet::new(),
            evaluation_profiles: BTreeSet::new(),
        }
    }

    /// Returns the source/assertion receiving this authority metadata.
    #[must_use]
    pub fn target(&self) -> &AuthorityTarget {
        &self.target
    }

    /// Adds a typed dimension without replacing distinct existing assessments.
    pub fn add_dimension(&mut self, dimension: AuthorityDimension) -> Result<bool, AuthorityError> {
        dimension.validate()?;
        Ok(self.dimensions.insert(dimension))
    }

    /// Consuming-builder form of [`Self::add_dimension`].
    pub fn with_dimension(mut self, dimension: AuthorityDimension) -> Result<Self, AuthorityError> {
        self.add_dimension(dimension)?;
        Ok(self)
    }

    /// Adds a domain-specific evaluation profile to the record.
    pub fn add_evaluation_profile(&mut self, profile: AuthorityEvaluationProfile) -> bool {
        self.evaluation_profiles.insert(profile)
    }

    /// Consuming-builder form for adding an evaluation profile.
    #[must_use]
    pub fn with_evaluation_profile(mut self, profile: AuthorityEvaluationProfile) -> Self {
        self.add_evaluation_profile(profile);
        self
    }

    /// Iterates over authority dimensions in deterministic order.
    pub fn dimensions(&self) -> impl Iterator<Item = &AuthorityDimension> {
        self.dimensions.iter()
    }

    /// Returns the number of distinct dimension/value entries.
    #[must_use]
    pub fn dimension_count(&self) -> usize {
        self.dimensions.len()
    }

    /// Iterates over the attached domain-specific evaluation profiles.
    pub fn evaluation_profiles(&self) -> impl Iterator<Item = &AuthorityEvaluationProfile> {
        self.evaluation_profiles.iter()
    }

    /// Returns whether no authority dimensions or evaluation profiles are recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dimensions.is_empty() && self.evaluation_profiles.is_empty()
    }
}

/// Structural errors in authority metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorityError {
    /// A required field is empty or whitespace-only.
    EmptyField {
        /// Field name used in the error message.
        field: &'static str,
    },
    /// A field contains a Unicode control character.
    ControlCharacter {
        /// Field name used in the error message.
        field: &'static str,
        /// Character index of the invalid value.
        index: usize,
    },
}

impl fmt::Display for AuthorityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "authority {field} must not be empty"),
            Self::ControlCharacter { field, index } => write!(
                formatter,
                "authority {field} contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for AuthorityError {}

fn validate_text(value: &str, field: &'static str) -> Result<(), AuthorityError> {
    if value.trim().is_empty() {
        return Err(AuthorityError::EmptyField { field });
    }

    if let Some(index) = value.chars().position(char::is_control) {
        return Err(AuthorityError::ControlCharacter { field, index });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Authority, AuthorityDimension, AuthorityError, AuthorityEvaluationProfile, AuthorityTarget,
        AuthorityValue,
    };
    use crate::authority::{ProcessReliability, ScholarlyStatus, SourceReliability};
    use crate::identity::{EvidenceId, KnowledgeAssertionId, SourceId};

    #[test]
    fn authority_can_attach_to_sources_and_assertions_without_conflating_targets() {
        let source = Authority::new(AuthorityTarget::Source(
            SourceId::new("source-quran").expect("valid source identity"),
        ));
        let assertion = Authority::new(AuthorityTarget::Assertion(
            KnowledgeAssertionId::new("assertion-1").expect("valid assertion identity"),
        ));

        assert_ne!(source.target(), assertion.target());
        assert!(matches!(source.target(), AuthorityTarget::Source(_)));
        assert!(matches!(assertion.target(), AuthorityTarget::Assertion(_)));
    }

    #[test]
    fn multiple_authority_dimensions_coexist_without_a_global_score() {
        let evidence_id = EvidenceId::new("evidence-review").expect("valid evidence identity");
        let mut authority = Authority::new(AuthorityTarget::Source(
            SourceId::new("source-1").expect("valid source identity"),
        ));

        assert!(
            authority
                .add_dimension(AuthorityDimension::SourceAuthority(
                    AuthorityValue::new("recognized").expect("valid authority value"),
                ))
                .expect("valid dimension")
        );
        assert!(
            authority
                .add_dimension(AuthorityDimension::SourceReliability(
                    SourceReliability::new("highly-reliable")
                        .expect("valid source reliability")
                        .with_evidence([evidence_id.clone()]),
                ))
                .expect("valid dimension")
        );
        assert!(
            authority
                .add_dimension(AuthorityDimension::ProcessReliability(
                    ProcessReliability::new("inconsistent-extraction")
                        .expect("valid process reliability")
                        .with_evidence([evidence_id.clone()]),
                ))
                .expect("valid dimension")
        );
        assert!(
            authority
                .add_dimension(AuthorityDimension::ScholarlyStatus(
                    ScholarlyStatus::in_vocabulary("domain-review", "disputed")
                        .expect("valid scholarly status"),
                ))
                .expect("valid dimension")
        );
        assert_eq!(authority.dimension_count(), 4);
        assert!(
            authority
                .dimensions()
                .any(|item| item.name() == "source-reliability")
        );
        assert!(
            authority
                .dimensions()
                .any(|item| item.name() == "process-reliability")
        );
    }

    #[test]
    fn exact_dimension_duplicates_are_idempotent_but_distinct_values_coexist() {
        let mut authority = Authority::new(AuthorityTarget::Assertion(
            KnowledgeAssertionId::new("assertion-2").expect("valid assertion identity"),
        ));
        let first = AuthorityDimension::Authentication(
            AuthorityValue::new("not-yet-reviewed").expect("valid value"),
        );
        let second = AuthorityDimension::Authentication(
            AuthorityValue::new("authenticated-by-review").expect("valid value"),
        );

        assert!(authority.add_dimension(first.clone()).unwrap());
        assert!(!authority.add_dimension(first).unwrap());
        assert!(authority.add_dimension(second).unwrap());
        assert_eq!(authority.dimension_count(), 2);
    }

    #[test]
    fn domain_specific_evaluation_profiles_are_preserved_without_evaluation() {
        let profile = AuthorityEvaluationProfile::new("kg-eval", "scholarly-source-profile")
            .expect("valid profile")
            .with_domain("Islamic scholarship")
            .expect("valid domain")
            .with_version("v1")
            .expect("valid version");
        let authority = Authority::new(AuthorityTarget::Source(
            SourceId::new("source-2").expect("valid source identity"),
        ))
        .with_evaluation_profile(profile.clone());

        let stored = authority
            .evaluation_profiles()
            .next()
            .expect("profile stored");
        assert_eq!(stored, &profile);
        assert_eq!(stored.domain(), Some("Islamic scholarship"));
        assert_eq!(stored.version(), Some("v1"));
        assert_eq!(stored.name(), "scholarly-source-profile");
    }

    #[test]
    fn invalid_extensions_and_value_text_are_rejected() {
        let value = AuthorityValue::new("domain-value").unwrap();
        assert!(matches!(
            AuthorityDimension::extension("  ", value.clone()),
            Err(AuthorityError::EmptyField {
                field: "authority extension name"
            })
        ));
        assert!(AuthorityValue::new("invalid\nvalue").is_err());
        assert!(
            AuthorityValue::in_vocabulary("profile", "value")
                .unwrap()
                .with_rationale("  ")
                .is_err()
        );
        assert!(AuthorityEvaluationProfile::new("", "profile").is_err());
    }
}
