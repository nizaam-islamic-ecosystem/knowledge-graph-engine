//! Raw material and source-record contracts for Phase 5 ingestion.
//!
//! Raw bytes are stored/retrieved through the Core artifact mechanism. This
//! module carries a typed [`ArtifactReference`] and the metadata required to
//! interpret the source snapshot; it does not implement another artifact store.

use core::fmt;
use std::collections::BTreeSet;

use nizaam_core::artifact::ArtifactReference;
use nizaam_core::identity::OperationId;

use crate::entity::ExternalIdentifier;
use crate::identity::SourceId;
use crate::temporal::Instant;

/// Domain-neutral classification used by source-specific ingestion policies.
///
/// Built-in labels are conveniences, not an exhaustive list of source domains.
/// New source classes can be supplied without changing the ingestion model.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceClass(String);

impl SourceClass {
    /// Creates a validated source-class label.
    pub fn new(value: impl Into<String>) -> Result<Self, SourceMetadataError> {
        let value = value.into();
        validate_label(&value, "source class")?;
        Ok(Self(value))
    }

    /// Generic structured-data source class.
    pub fn structured_data() -> Self {
        Self("structured-data".to_owned())
    }

    /// Generic semi-structured-data source class.
    pub fn semi_structured_data() -> Self {
        Self("semi-structured-data".to_owned())
    }

    /// Generic unstructured-document source class.
    pub fn unstructured_document() -> Self {
        Self("unstructured-document".to_owned())
    }

    /// Source class for material produced by a model or automated extraction.
    pub fn machine_generated() -> Self {
        Self("machine-generated".to_owned())
    }

    /// Source class for material explicitly curated by a human.
    pub fn human_curated() -> Self {
        Self("human-curated".to_owned())
    }

    /// Returns the stable source-class label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A raw source input whose content is represented by a Core artifact reference.
///
/// `captured_at` is the time the KG ingestion boundary recorded this material;
/// it is not the semantic valid time of any resulting assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawMaterial {
    operation_id: OperationId,
    source_id: SourceId,
    source_class: SourceClass,
    artifact: ArtifactReference,
    content_type: String,
    source_version: Option<String>,
    captured_at: Instant,
}

impl RawMaterial {
    /// Creates raw-material metadata around an artifact already addressed by Core.
    pub fn new(
        operation_id: OperationId,
        source_id: SourceId,
        source_class: SourceClass,
        artifact: ArtifactReference,
        content_type: impl Into<String>,
        captured_at: Instant,
    ) -> Result<Self, RawMaterialError> {
        let content_type = content_type.into();
        validate_label(&content_type, "content type").map_err(RawMaterialError::InvalidMetadata)?;
        if !artifact.is_valid() || !artifact.is_exact() {
            return Err(RawMaterialError::InvalidArtifactReference);
        }

        Ok(Self {
            operation_id,
            source_id,
            source_class,
            artifact,
            content_type,
            source_version: None,
            captured_at,
        })
    }

    /// Adds the source-owned version or snapshot label without interpreting it
    /// as a Knowledge Graph storage version.
    pub fn with_source_version(
        mut self,
        version: impl Into<String>,
    ) -> Result<Self, RawMaterialError> {
        let version = version.into();
        validate_label(&version, "source version").map_err(RawMaterialError::InvalidMetadata)?;
        self.source_version = Some(version);
        Ok(self)
    }

    /// Revalidates raw material before an adapter uses it.
    pub fn validate(&self) -> Result<(), RawMaterialError> {
        validate_label(&self.content_type, "content type")
            .map_err(RawMaterialError::InvalidMetadata)?;
        if let Some(version) = &self.source_version {
            validate_label(version, "source version").map_err(RawMaterialError::InvalidMetadata)?;
        }
        if !self.artifact.is_valid() || !self.artifact.is_exact() {
            return Err(RawMaterialError::InvalidArtifactReference);
        }
        Ok(())
    }

    /// Returns the Core operation responsible for this ingestion attempt.
    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the source class used by governance policy.
    #[must_use]
    pub fn source_class(&self) -> &SourceClass {
        &self.source_class
    }

    /// Returns the Core artifact reference; this module does not load its bytes.
    #[must_use]
    pub fn artifact(&self) -> &ArtifactReference {
        &self.artifact
    }

    /// Returns the declared content type.
    #[must_use]
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// Returns a source-owned version/snapshot label when supplied.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }

    /// Returns when this raw material was captured by the ingestion boundary.
    #[must_use]
    pub const fn captured_at(&self) -> Instant {
        self.captured_at
    }
}

/// Stable metadata identifying one record extracted from a source snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceRecordMetadata {
    source_id: SourceId,
    source_class: SourceClass,
    source_record_key: String,
    source_version: Option<String>,
    source_locator: Option<String>,
    external_identifiers: BTreeSet<ExternalIdentifier>,
}

impl SourceRecordMetadata {
    /// Creates metadata for one source-owned record.
    pub fn new(
        source_id: SourceId,
        source_class: SourceClass,
        source_record_key: impl Into<String>,
        source_version: Option<String>,
        source_locator: Option<String>,
        external_identifiers: impl IntoIterator<Item = ExternalIdentifier>,
    ) -> Result<Self, SourceRecordMetadataError> {
        let value = Self {
            source_id,
            source_class,
            source_record_key: source_record_key.into(),
            source_version,
            source_locator,
            external_identifiers: external_identifiers.into_iter().collect(),
        };
        value.validate()?;
        Ok(value)
    }

    /// Revalidates the public record metadata at stage boundaries.
    pub fn validate(&self) -> Result<(), SourceRecordMetadataError> {
        validate_label(&self.source_record_key, "source record key")
            .map_err(SourceRecordMetadataError::InvalidLabel)?;
        if let Some(version) = &self.source_version {
            validate_label(version, "source version")
                .map_err(SourceRecordMetadataError::InvalidLabel)?;
        }
        if let Some(locator) = &self.source_locator {
            validate_label(locator, "source locator")
                .map_err(SourceRecordMetadataError::InvalidLabel)?;
        }
        for identifier in &self.external_identifiers {
            if identifier.source_id() != &self.source_id {
                return Err(
                    SourceRecordMetadataError::ExternalIdentifierSourceMismatch {
                        expected: self.source_id.clone(),
                        actual: identifier.source_id().clone(),
                    },
                );
            }
        }
        Ok(())
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the policy classification for this source record.
    #[must_use]
    pub fn source_class(&self) -> &SourceClass {
        &self.source_class
    }

    /// Returns the opaque stable key supplied by the source adapter.
    #[must_use]
    pub fn source_record_key(&self) -> &str {
        &self.source_record_key
    }

    /// Returns the source-owned version/snapshot label when supplied.
    #[must_use]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }

    /// Returns the locator within the source artifact when supplied.
    #[must_use]
    pub fn source_locator(&self) -> Option<&str> {
        self.source_locator.as_deref()
    }

    /// Returns all external identifiers, preserving their source namespace.
    #[must_use]
    pub fn external_identifiers(&self) -> &BTreeSet<ExternalIdentifier> {
        &self.external_identifiers
    }
}

/// Source-specific extracted record wrapped in stable ingestion metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceRecord<T> {
    metadata: SourceRecordMetadata,
    payload: T,
}

impl<T> SourceRecord<T> {
    /// Creates a record after validating its source metadata.
    pub fn new(metadata: SourceRecordMetadata, payload: T) -> Result<Self, SourceRecordError> {
        metadata
            .validate()
            .map_err(SourceRecordError::InvalidMetadata)?;
        Ok(Self { metadata, payload })
    }

    /// Returns stable source metadata.
    #[must_use]
    pub fn metadata(&self) -> &SourceRecordMetadata {
        &self.metadata
    }

    /// Returns the source-specific payload without interpreting its semantics.
    #[must_use]
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Consumes the record into metadata and payload.
    #[must_use]
    pub fn into_parts(self) -> (SourceRecordMetadata, T) {
        (self.metadata, self.payload)
    }
}

/// Contract implemented by source-specific adapters.
///
/// The caller loads artifact bytes through Core's artifact mechanism and passes
/// them to [`SourceAdapter::decode`]. Adapters produce source records, not final
/// canonical KG entities or assertions.
pub trait SourceAdapter {
    /// Source-specific record payload emitted by this adapter.
    type Record;

    /// Returns the source namespace owned by this adapter.
    fn source_id(&self) -> &SourceId;

    /// Returns the source class used by approval policy.
    fn source_class(&self) -> &SourceClass;

    /// Decodes source bytes into source-specific records.
    fn decode_payload(
        &self,
        raw: &RawMaterial,
        artifact_bytes: &[u8],
    ) -> Result<Vec<SourceRecord<Self::Record>>, SourceAdapterError>;

    /// Validates the raw source envelope and adapter output around decoding.
    fn decode(
        &self,
        raw: &RawMaterial,
        artifact_bytes: &[u8],
    ) -> Result<Vec<SourceRecord<Self::Record>>, SourceAdapterError> {
        raw.validate()
            .map_err(SourceAdapterError::InvalidRawMaterial)?;
        if raw.source_id() != self.source_id() {
            return Err(SourceAdapterError::SourceMismatch);
        }
        if raw.source_class() != self.source_class() {
            return Err(SourceAdapterError::SourceClassMismatch);
        }

        let records = self.decode_payload(raw, artifact_bytes)?;
        for record in &records {
            record
                .metadata()
                .validate()
                .map_err(SourceAdapterError::InvalidRecordMetadata)?;
            if record.metadata().source_id() != raw.source_id() {
                return Err(SourceAdapterError::SourceMismatch);
            }
            if record.metadata().source_class() != raw.source_class() {
                return Err(SourceAdapterError::SourceClassMismatch);
            }
            if record.metadata().source_version() != raw.source_version() {
                return Err(SourceAdapterError::SourceVersionMismatch);
            }
        }
        Ok(records)
    }
}

/// Generic structured-JSON adapter used as the initial source adapter.
///
/// A top-level JSON object is treated as one record; a top-level array is
/// treated as an ordered collection of records. Every record must be an object.
/// When configured, a record-key field must contain a string or number. Without
/// one, a deterministic ordinal key is generated within the exact source snapshot.
/// This adapter performs no semantic mapping or language-specific extraction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredJsonAdapter {
    source_id: SourceId,
    source_class: SourceClass,
    record_key_field: Option<String>,
    external_identifier_field: Option<String>,
}

impl StructuredJsonAdapter {
    /// Creates an adapter for a source using the generic structured-data class.
    #[must_use]
    pub fn new(source_id: SourceId) -> Self {
        Self {
            source_id,
            source_class: SourceClass::structured_data(),
            record_key_field: None,
            external_identifier_field: None,
        }
    }

    /// Uses a source-specific class for approval policy selection.
    #[must_use]
    pub fn with_source_class(mut self, source_class: SourceClass) -> Self {
        self.source_class = source_class;
        self
    }

    /// Configures a JSON object field used as the stable source-record key.
    pub fn with_record_key_field(
        mut self,
        field: impl Into<String>,
    ) -> Result<Self, SourceMetadataError> {
        let field = field.into();
        validate_label(&field, "JSON record-key field")?;
        self.record_key_field = Some(field);
        Ok(self)
    }

    /// Configures an optional JSON field containing a source-owned external ID.
    pub fn with_external_identifier_field(
        mut self,
        field: impl Into<String>,
    ) -> Result<Self, SourceMetadataError> {
        let field = field.into();
        validate_label(&field, "JSON external-identifier field")?;
        self.external_identifier_field = Some(field);
        Ok(self)
    }

    fn scalar_field<'a>(
        object: &'a serde_json::Map<String, serde_json::Value>,
        field: &str,
    ) -> Option<&'a serde_json::Value> {
        object.get(field)
    }

    fn scalar_text(value: &serde_json::Value) -> Option<String> {
        value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_number().map(ToString::to_string))
    }
}

impl SourceAdapter for StructuredJsonAdapter {
    type Record = serde_json::Value;

    fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    fn source_class(&self) -> &SourceClass {
        &self.source_class
    }

    fn decode_payload(
        &self,
        raw: &RawMaterial,
        artifact_bytes: &[u8],
    ) -> Result<Vec<SourceRecord<Self::Record>>, SourceAdapterError> {
        let media_type = raw
            .content_type()
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if media_type != "application/json" && !media_type.ends_with("+json") {
            return Err(SourceAdapterError::UnsupportedContentType {
                content_type: raw.content_type().to_owned(),
            });
        }

        let document: serde_json::Value = serde_json::from_slice(artifact_bytes)
            .map_err(|error| SourceAdapterError::DecodeFailed(error.to_string()))?;
        let (values, is_array) = match document {
            serde_json::Value::Array(values) => (values, true),
            object @ serde_json::Value::Object(_) => (vec![object], false),
            _ => {
                return Err(SourceAdapterError::DecodeFailed(
                    "JSON source root must be an object or an array of objects".to_owned(),
                ));
            }
        };

        let mut records = Vec::with_capacity(values.len());
        for (ordinal, value) in values.into_iter().enumerate() {
            let serde_json::Value::Object(object) = &value else {
                return Err(SourceAdapterError::DecodeFailed(format!(
                    "JSON record at ordinal {ordinal} is not an object"
                )));
            };
            let record_key = match &self.record_key_field {
                Some(field) => Self::scalar_field(object, field)
                    .and_then(Self::scalar_text)
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        SourceAdapterError::DecodeFailed(format!(
                            "JSON record at ordinal {ordinal} is missing a usable key field"
                        ))
                    })?,
                None => format!("record-{ordinal}"),
            };

            let mut external_identifiers = Vec::new();
            if let Some(field) = &self.external_identifier_field
                && let Some(value) = Self::scalar_field(object, field)
            {
                let value = Self::scalar_text(value).ok_or_else(|| SourceAdapterError::DecodeFailed(format!(
                        "JSON record at ordinal {ordinal} has a non-string/non-number external identifier"
                    )))?;
                if !value.trim().is_empty() {
                    external_identifiers.push(
                        ExternalIdentifier::new(self.source_id.clone(), value)
                            .map_err(|error| SourceAdapterError::DecodeFailed(error.to_string()))?,
                    );
                }
            }

            let locator = is_array.then(|| format!("/{ordinal}"));
            let metadata = SourceRecordMetadata::new(
                self.source_id.clone(),
                self.source_class.clone(),
                record_key,
                raw.source_version().map(str::to_owned),
                locator,
                external_identifiers,
            )
            .map_err(SourceAdapterError::InvalidRecordMetadata)?;
            records.push(
                SourceRecord::new(metadata, value)
                    .map_err(|error| SourceAdapterError::DecodeFailed(error.to_string()))?,
            );
        }
        Ok(records)
    }
}

/// Errors from validating a source-class or metadata label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceMetadataError {
    /// A required label is empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// A label contains a Unicode control character.
    ControlCharacter { field: &'static str, index: usize },
}

impl fmt::Display for SourceMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel { field } => write!(formatter, "{field} must not be empty"),
            Self::ControlCharacter { field, index } => {
                write!(
                    formatter,
                    "{field} contains a control character at index {index}"
                )
            }
        }
    }
}
impl std::error::Error for SourceMetadataError {}

/// Raw-material envelope validation errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RawMaterialError {
    /// A required metadata label is invalid.
    InvalidMetadata(SourceMetadataError),
    /// The Core artifact reference is invalid or is not pinned to an exact version.
    InvalidArtifactReference,
}

impl fmt::Display for RawMaterialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMetadata(error) => {
                write!(formatter, "invalid raw-material metadata: {error}")
            }
            Self::InvalidArtifactReference => {
                formatter.write_str("raw material has an invalid Core artifact reference")
            }
        }
    }
}
impl std::error::Error for RawMaterialError {}

/// Source-record metadata validation errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceRecordMetadataError {
    /// A record metadata label is invalid.
    InvalidLabel(SourceMetadataError),
    /// An external identifier belongs to a different source namespace.
    ExternalIdentifierSourceMismatch {
        expected: SourceId,
        actual: SourceId,
    },
}

impl fmt::Display for SourceRecordMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel(error) => {
                write!(formatter, "invalid source-record metadata: {error}")
            }
            Self::ExternalIdentifierSourceMismatch { expected, actual } => write!(
                formatter,
                "external identifier source mismatch: expected {expected}, got {actual}"
            ),
        }
    }
}
impl std::error::Error for SourceRecordMetadataError {}

/// Source-record construction errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceRecordError {
    /// Source record metadata failed validation.
    InvalidMetadata(SourceRecordMetadataError),
}

impl fmt::Display for SourceRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMetadata(error) => write!(formatter, "invalid source record: {error}"),
        }
    }
}
impl std::error::Error for SourceRecordError {}

/// Errors produced by an adapter or by validating its decoded records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceAdapterError {
    /// The input's source identity differs from the adapter's source identity.
    SourceMismatch,
    /// The input's source class differs from the adapter's configured class.
    SourceClassMismatch,
    /// A decoded record changed or omitted the source snapshot version.
    SourceVersionMismatch,
    /// Raw-material metadata or its Core artifact reference is invalid.
    InvalidRawMaterial(RawMaterialError),
    /// A decoded record has invalid source metadata.
    InvalidRecordMetadata(SourceRecordMetadataError),
    /// The adapter could not interpret the supplied source representation.
    DecodeFailed(String),
    /// The configured adapter does not support the raw material content type.
    UnsupportedContentType { content_type: String },
}

impl fmt::Display for SourceAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceMismatch => {
                formatter.write_str("source adapter identity does not match raw material")
            }
            Self::SourceClassMismatch => {
                formatter.write_str("source adapter class does not match raw material")
            }
            Self::SourceVersionMismatch => {
                formatter.write_str("source adapter output does not preserve the source version")
            }
            Self::InvalidRawMaterial(error) => write!(formatter, "invalid raw material: {error}"),
            Self::InvalidRecordMetadata(error) => {
                write!(formatter, "invalid decoded source record: {error}")
            }
            Self::DecodeFailed(message) => write!(
                formatter,
                "source adapter failed to decode input: {message}"
            ),
            Self::UnsupportedContentType { content_type } => write!(
                formatter,
                "structured JSON adapter does not support content type {content_type}"
            ),
        }
    }
}
impl std::error::Error for SourceAdapterError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), SourceMetadataError> {
    if value.trim().is_empty() {
        return Err(SourceMetadataError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(SourceMetadataError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RawMaterial, SourceAdapter, SourceAdapterError, SourceClass, SourceRecord,
        SourceRecordMetadata, StructuredJsonAdapter,
    };
    use crate::entity::ExternalIdentifier;
    use crate::identity::SourceId;
    use crate::temporal::Instant;
    use nizaam_core::artifact::ArtifactReference;
    use nizaam_core::identity::{ArtifactId, OperationId};

    fn raw(source: SourceId, class: SourceClass) -> RawMaterial {
        RawMaterial::new(
            OperationId::new("nizaam.kg.ingestion.raw-test.operation").unwrap(),
            source,
            class,
            ArtifactReference::new(ArtifactId::new("artifact-raw-test").unwrap(), "v1"),
            "application/json",
            Instant::from_unix_seconds(1_750_000_000),
        )
        .unwrap()
    }

    struct TestAdapter {
        source_id: SourceId,
        source_class: SourceClass,
    }

    impl SourceAdapter for TestAdapter {
        type Record = String;

        fn source_id(&self) -> &SourceId {
            &self.source_id
        }

        fn source_class(&self) -> &SourceClass {
            &self.source_class
        }

        fn decode_payload(
            &self,
            raw: &RawMaterial,
            artifact_bytes: &[u8],
        ) -> Result<Vec<SourceRecord<Self::Record>>, SourceAdapterError> {
            let text = std::str::from_utf8(artifact_bytes)
                .map_err(|error| SourceAdapterError::DecodeFailed(error.to_string()))?;
            let metadata = SourceRecordMetadata::new(
                raw.source_id().clone(),
                raw.source_class().clone(),
                "record-1",
                raw.source_version().map(str::to_owned),
                Some("items/0".to_owned()),
                [],
            )
            .map_err(SourceAdapterError::InvalidRecordMetadata)?;
            Ok(vec![SourceRecord::new(metadata, text.to_owned()).map_err(
                |error| SourceAdapterError::DecodeFailed(error.to_string()),
            )?])
        }
    }

    #[test]
    fn source_class_is_extensible_and_validated() {
        assert_eq!(
            SourceClass::new("hadith-source").unwrap().as_str(),
            "hadith-source"
        );
        assert!(SourceClass::new("  ").is_err());
        assert_eq!(SourceClass::structured_data().as_str(), "structured-data");
    }

    #[test]
    fn source_record_metadata_rejects_cross_source_external_identifiers() {
        let source = SourceId::new("source-a").unwrap();
        let external = ExternalIdentifier::new(SourceId::new("source-b").unwrap(), "42").unwrap();
        let result = SourceRecordMetadata::new(
            source,
            SourceClass::structured_data(),
            "record-1",
            None,
            None,
            [external],
        );
        assert!(result.is_err());
    }

    #[test]
    fn adapter_outputs_records_with_source_and_snapshot_metadata_preserved() {
        let source = SourceId::new("source-a").unwrap();
        let class = SourceClass::structured_data();
        let adapter = TestAdapter {
            source_id: source.clone(),
            source_class: class.clone(),
        };
        let raw = raw(source, class);
        let records = adapter.decode(&raw, br#"{"name":"example"}"#).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].metadata().source_record_key(), "record-1");
        assert_eq!(records[0].payload(), r#"{"name":"example"}"#);
    }

    #[test]
    fn structured_json_adapter_decodes_records_with_stable_keys_and_external_ids() {
        let source = SourceId::new("source-json").unwrap();
        let class = SourceClass::structured_data();
        let adapter = StructuredJsonAdapter::new(source.clone())
            .with_record_key_field("id")
            .unwrap()
            .with_external_identifier_field("external_id")
            .unwrap();
        let raw = RawMaterial::new(
            OperationId::new("nizaam.kg.ingestion.json-test.operation").unwrap(),
            source,
            class,
            ArtifactReference::new(ArtifactId::new("artifact-json-test").unwrap(), "v1"),
            "application/json; charset=utf-8",
            Instant::from_unix_seconds(10),
        )
        .unwrap()
        .with_source_version("snapshot-2")
        .unwrap();

        let records = adapter
            .decode(&raw, br#"[{"id":"a-1","external_id":"P-1"},{"id":2}]"#)
            .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].metadata().source_record_key(), "a-1");
        assert_eq!(records[0].metadata().source_locator(), Some("/0"));
        assert_eq!(
            records[0]
                .metadata()
                .external_identifiers()
                .first()
                .unwrap()
                .value(),
            "P-1"
        );
        assert_eq!(records[1].metadata().source_record_key(), "2");
        assert_eq!(records[1].metadata().source_version(), Some("snapshot-2"));
    }

    #[test]
    fn structured_json_adapter_rejects_invalid_roots_and_non_json_content_types() {
        let source = SourceId::new("source-json-invalid").unwrap();
        let class = SourceClass::structured_data();
        let adapter = StructuredJsonAdapter::new(source.clone());
        let raw = raw(source, class);
        assert!(adapter.decode(&raw, b"null").is_err());

        let non_json = RawMaterial::new(
            OperationId::new("nizaam.kg.ingestion.json-content-type.operation").unwrap(),
            SourceId::new("source-json-invalid").unwrap(),
            SourceClass::structured_data(),
            ArtifactReference::new(ArtifactId::new("artifact-json-content-type").unwrap(), "v1"),
            "text/plain",
            Instant::from_unix_seconds(10),
        )
        .unwrap();
        assert!(matches!(
            adapter.decode(&non_json, b"{}"),
            Err(SourceAdapterError::UnsupportedContentType { .. })
        ));
    }

    #[test]
    fn adapter_rejects_a_raw_material_from_another_source() {
        let adapter = TestAdapter {
            source_id: SourceId::new("source-a").unwrap(),
            source_class: SourceClass::structured_data(),
        };
        let raw = raw(
            SourceId::new("source-b").unwrap(),
            SourceClass::structured_data(),
        );
        assert_eq!(
            adapter.decode(&raw, b"{}"),
            Err(SourceAdapterError::SourceMismatch)
        );
    }
}
