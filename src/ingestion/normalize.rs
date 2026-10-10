//! Deterministic, versioned normalization contracts for Phase 5.
//!
//! This module provides generic stage metadata, record envelopes, and a small
//! whitespace-only text normalizer. Source-specific field mapping and language
//! analysis remain adapter responsibilities; no Arabic morphology is performed.

use core::fmt;

use crate::temporal::Instant;

use super::raw::{SourceRecord, SourceRecordMetadata, SourceRecordMetadataError};

/// Version information required to reproduce one normalization result.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NormalizationMetadata {
    stage_version: String,
    configuration_version: String,
    normalized_at: Instant,
}

impl NormalizationMetadata {
    /// Creates versioned metadata for a normalization stage execution.
    pub fn new(
        stage_version: impl Into<String>,
        configuration_version: impl Into<String>,
        normalized_at: Instant,
    ) -> Result<Self, NormalizationError> {
        let stage_version = stage_version.into();
        let configuration_version = configuration_version.into();
        validate_label(&stage_version, "normalization stage version")?;
        validate_label(
            &configuration_version,
            "normalization configuration version",
        )?;
        Ok(Self {
            stage_version,
            configuration_version,
            normalized_at,
        })
    }

    /// Returns the implementation/stage version.
    #[must_use]
    pub fn stage_version(&self) -> &str {
        &self.stage_version
    }

    /// Returns the normalization configuration version.
    #[must_use]
    pub fn configuration_version(&self) -> &str {
        &self.configuration_version
    }

    /// Returns when this normalization result was produced.
    #[must_use]
    pub const fn normalized_at(&self) -> Instant {
        self.normalized_at
    }
}

/// Normalized payload with the original source identity and version preserved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedRecord<T> {
    source: SourceRecordMetadata,
    payload: T,
    normalization: NormalizationMetadata,
}

impl<T> NormalizedRecord<T> {
    /// Creates a normalized record and validates the retained source metadata.
    pub fn new(
        source: SourceRecordMetadata,
        payload: T,
        normalization: NormalizationMetadata,
    ) -> Result<Self, NormalizationError> {
        source
            .validate()
            .map_err(NormalizationError::InvalidSourceMetadata)?;
        Ok(Self {
            source,
            payload,
            normalization,
        })
    }

    /// Returns original source-record metadata.
    #[must_use]
    pub fn source(&self) -> &SourceRecordMetadata {
        &self.source
    }

    /// Returns the normalized stage payload.
    #[must_use]
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Returns normalization stage version, configuration version, and time.
    #[must_use]
    pub fn normalization(&self) -> &NormalizationMetadata {
        &self.normalization
    }

    /// Consumes the wrapper while preserving source metadata and stage evidence.
    #[must_use]
    pub fn into_parts(self) -> (SourceRecordMetadata, T, NormalizationMetadata) {
        (self.source, self.payload, self.normalization)
    }
}

/// Pluggable typed normalization-stage contract.
pub trait NormalizationStage<Input, Output> {
    /// Returns the deterministic implementation version.
    fn stage_version(&self) -> &str;

    /// Returns the configuration version used by this stage.
    fn configuration_version(&self) -> &str;

    /// Normalizes one source record into zero or more output records.
    fn normalize(
        &self,
        input: SourceRecord<Input>,
        normalized_at: Instant,
    ) -> Result<Vec<NormalizedRecord<Output>>, NormalizationError>;
}

/// Generic whitespace handling that does not perform linguistic analysis.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextNormalizationMode {
    /// Preserve all source text exactly.
    Preserve,
    /// Remove leading/trailing Unicode whitespace only.
    Trim,
    /// Collapse each run of Unicode whitespace into one ASCII space and trim.
    CollapseWhitespace,
}

/// Minimal deterministic text normalizer for source formats whose contract
/// explicitly permits the selected whitespace transformation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericTextNormalizer {
    stage_version: String,
    configuration_version: String,
    mode: TextNormalizationMode,
}

impl GenericTextNormalizer {
    /// Creates a versioned whitespace normalizer.
    pub fn new(
        stage_version: impl Into<String>,
        configuration_version: impl Into<String>,
        mode: TextNormalizationMode,
    ) -> Result<Self, NormalizationError> {
        let stage_version = stage_version.into();
        let configuration_version = configuration_version.into();
        validate_label(&stage_version, "normalization stage version")?;
        validate_label(
            &configuration_version,
            "normalization configuration version",
        )?;
        Ok(Self {
            stage_version,
            configuration_version,
            mode,
        })
    }

    /// Returns the configured whitespace mode.
    #[must_use]
    pub const fn mode(&self) -> TextNormalizationMode {
        self.mode
    }
}

impl NormalizationStage<String, String> for GenericTextNormalizer {
    fn stage_version(&self) -> &str {
        &self.stage_version
    }

    fn configuration_version(&self) -> &str {
        &self.configuration_version
    }

    fn normalize(
        &self,
        input: SourceRecord<String>,
        normalized_at: Instant,
    ) -> Result<Vec<NormalizedRecord<String>>, NormalizationError> {
        let (source, payload) = input.into_parts();
        let normalized = match self.mode {
            TextNormalizationMode::Preserve => payload,
            TextNormalizationMode::Trim => payload.trim().to_owned(),
            TextNormalizationMode::CollapseWhitespace => {
                payload.split_whitespace().collect::<Vec<_>>().join(" ")
            }
        };
        let metadata = NormalizationMetadata::new(
            self.stage_version.clone(),
            self.configuration_version.clone(),
            normalized_at,
        )?;
        Ok(vec![NormalizedRecord::new(source, normalized, metadata)?])
    }
}

/// Errors produced at normalization boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizationError {
    /// A stage or configuration version is empty or contains control characters.
    InvalidLabel { field: &'static str },
    /// Source metadata could not be preserved/validated.
    InvalidSourceMetadata(SourceRecordMetadataError),
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel { field } => write!(formatter, "{field} is invalid"),
            Self::InvalidSourceMetadata(error) => {
                write!(
                    formatter,
                    "normalization received invalid source metadata: {error}"
                )
            }
        }
    }
}
impl std::error::Error for NormalizationError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), NormalizationError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(NormalizationError::InvalidLabel { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        GenericTextNormalizer, NormalizationMetadata, NormalizationStage, TextNormalizationMode,
    };
    use crate::identity::SourceId;
    use crate::ingestion::raw::{SourceClass, SourceRecord, SourceRecordMetadata};
    use crate::temporal::Instant;

    fn record(payload: &str) -> SourceRecord<String> {
        SourceRecord::new(
            SourceRecordMetadata::new(
                SourceId::new("source-normalize").unwrap(),
                SourceClass::structured_data(),
                "record-1",
                Some("snapshot-1".to_owned()),
                None,
                [],
            )
            .unwrap(),
            payload.to_owned(),
        )
        .unwrap()
    }

    #[test]
    fn normalization_preserves_source_identity_and_versions() {
        let normalizer = GenericTextNormalizer::new(
            "text-normalizer-v1",
            "whitespace-v1",
            TextNormalizationMode::CollapseWhitespace,
        )
        .unwrap();
        let output = normalizer
            .normalize(
                record("  one\t two\nthree  "),
                Instant::from_unix_seconds(50),
            )
            .unwrap();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0].payload(), "one two three");
        assert_eq!(output[0].source().source_record_key(), "record-1");
        assert_eq!(output[0].source().source_version(), Some("snapshot-1"));
        assert_eq!(
            output[0].normalization().stage_version(),
            "text-normalizer-v1"
        );
    }

    #[test]
    fn preserve_mode_does_not_modify_source_text() {
        let normalizer = GenericTextNormalizer::new(
            "text-normalizer-v1",
            "preserve-v1",
            TextNormalizationMode::Preserve,
        )
        .unwrap();
        let output = normalizer
            .normalize(record("  محمد\n "), Instant::from_unix_seconds(51))
            .unwrap();
        assert_eq!(output[0].payload(), "  محمد\n ");
    }

    #[test]
    fn stage_and_configuration_versions_are_required() {
        assert!(GenericTextNormalizer::new(" ", "config-v1", TextNormalizationMode::Trim).is_err());
        assert!(
            NormalizationMetadata::new("stage-v1", "\n", Instant::from_unix_seconds(0)).is_err()
        );
    }
}
