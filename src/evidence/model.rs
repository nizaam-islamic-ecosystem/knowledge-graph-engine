//! First-class evidence objects and structured source locations for Phase 4.
//!
//! Evidence describes why a knowledge assertion may be supported, refuted, or
//! qualified. It is deliberately distinct from provenance (how knowledge was
//! produced), authority, verification, and confidence evaluation.
//!
//! A support role is attached to an [`EvidenceSupport`] association rather than
//! permanently to the evidence object: the same evidence can support one
//! assertion while refuting another.

use core::fmt;
use std::collections::BTreeSet;

use crate::identity::{EvidenceId, ReferenceId, SourceId};

/// The semantic role evidence plays in relation to a particular assertion.
///
/// The built-in vocabulary is intentionally small. `Custom` permits a domain
/// package to add a role without redesigning the core evidence model. Use
/// [`EvidenceRole::custom`] when possible; consumers must still validate roles
/// because the enum's public custom variant can be constructed directly.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceRole {
    /// Evidence supports an assertion.
    Supports,
    /// Evidence refutes an assertion.
    Refutes,
    /// Evidence qualifies the scope or interpretation of an assertion.
    Qualifies,
    /// Evidence independently corroborates an assertion.
    Corroborates,
    /// Evidence supplies context for an assertion.
    Contextualizes,
    /// Evidence illustrates an assertion without necessarily proving it.
    Illustrates,
    /// Domain-defined role.
    Custom(String),
}

impl EvidenceRole {
    /// Creates a valid domain-defined role.
    pub fn custom(name: impl Into<String>) -> Result<Self, EvidenceRoleError> {
        let name = name.into();
        validate_nonempty_label(&name, "evidence role")?;
        Ok(Self::Custom(name))
    }

    /// Returns the stable textual label for this role.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Supports => "supports",
            Self::Refutes => "refutes",
            Self::Qualifies => "qualifies",
            Self::Corroborates => "corroborates",
            Self::Contextualizes => "contextualizes",
            Self::Illustrates => "illustrates",
            Self::Custom(name) => name,
        }
    }

    /// Validates values including a directly constructed `Custom` variant.
    pub fn validate(&self) -> Result<(), EvidenceRoleError> {
        if let Self::Custom(name) = self {
            validate_nonempty_label(name, "evidence role")?;
        }
        Ok(())
    }
}

/// Errors produced while constructing an extensible evidence role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceRoleError {
    /// A custom role name was empty or whitespace-only.
    EmptyName,
    /// A custom role name contained a control character.
    ControlCharacter { index: usize },
}

impl fmt::Display for EvidenceRoleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("evidence role name must not be empty"),
            Self::ControlCharacter { index } => write!(
                formatter,
                "evidence role name contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for EvidenceRoleError {}

/// How an evidence item is connected to source material or other evidence.
///
/// `DerivedSupport` declares the upstream evidence records on which this item
/// depends. Phase 4 records this chain but does not execute a reasoning engine.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceMechanism {
    /// Evidence comes directly from one or more identified source locations.
    DirectSource,
    /// Evidence was derived from one or more other evidence records.
    DerivedSupport { inputs: BTreeSet<EvidenceId> },
}

impl EvidenceMechanism {
    /// Creates a direct-source mechanism.
    #[must_use]
    pub const fn direct_source() -> Self {
        Self::DirectSource
    }

    /// Creates a derived mechanism from a deterministic, deduplicated input set.
    pub fn derived_support<I>(inputs: I) -> Result<Self, EvidenceValidationError>
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        let inputs = inputs.into_iter().collect::<BTreeSet<_>>();
        if inputs.is_empty() {
            return Err(EvidenceValidationError::DerivedSupportRequiresInput);
        }
        Ok(Self::DerivedSupport { inputs })
    }

    /// Returns the upstream evidence identities for derived support.
    #[must_use]
    pub fn inputs(&self) -> Option<&BTreeSet<EvidenceId>> {
        match self {
            Self::DirectSource => None,
            Self::DerivedSupport { inputs } => Some(inputs),
        }
    }
}

/// Unit used by a textual span's offsets.
///
/// A span is only required for source formats that expose textual offsets. A
/// source/page/verse locator can remain structured without inventing offsets.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextOffsetUnit {
    /// Offsets count UTF-8 bytes.
    Utf8Byte,
    /// Offsets count Unicode scalar values.
    UnicodeScalar,
    /// Offsets count UTF-16 code units.
    Utf16CodeUnit,
    /// Offsets count tokens according to a caller-supplied tokenizer.
    Token,
}

/// A half-open textual span `[start, end)` with an explicit offset unit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextSpan {
    start: usize,
    end: usize,
    unit: TextOffsetUnit,
}

impl TextSpan {
    /// Creates a non-empty half-open span.
    pub fn new(start: usize, end: usize, unit: TextOffsetUnit) -> Result<Self, TextSpanError> {
        if start >= end {
            return Err(TextSpanError::InvalidBounds { start, end });
        }
        Ok(Self { start, end, unit })
    }

    /// Returns the starting offset (inclusive).
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the ending offset (exclusive).
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the unit used by the offsets.
    #[must_use]
    pub const fn unit(self) -> TextOffsetUnit {
        self.unit
    }
}

/// Invalid textual-span boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextSpanError {
    /// A half-open span must satisfy `start < end`.
    InvalidBounds { start: usize, end: usize },
}

impl fmt::Display for TextSpanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBounds { start, end } => write!(
                formatter,
                "text span must be non-empty and satisfy start < end: start={start}, end={end}"
            ),
        }
    }
}

impl std::error::Error for TextSpanError {}

/// Supported structured source-location forms for evidence.
///
/// `ReferenceId` is used for generic document/passage references because Phase
/// 1 intentionally does not require concrete document and passage objects.
/// `SourceVersion` is an opaque source-version label; it does not implement
/// Phase 7 storage/versioning.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceSourceKind {
    /// Refers to a source without requiring a document or text locator.
    Source(SourceId),
    /// Refers to a specific, opaque version label of a source.
    SourceVersion {
        source_id: SourceId,
        version: String,
    },
    /// Refers to a document through the generic reference boundary.
    Document(ReferenceId),
    /// Refers to a passage through the generic reference boundary.
    Passage(ReferenceId),
    /// Refers to a named section within a generic reference.
    Section {
        reference: ReferenceId,
        section: String,
    },
    /// Refers to a page within a generic reference. Page labels remain opaque.
    Page {
        reference: ReferenceId,
        page: String,
    },
    /// Refers to a paragraph within a generic reference. Labels remain opaque.
    Paragraph {
        reference: ReferenceId,
        paragraph: String,
    },
    /// Refers to a verse/ayah locator within a generic reference.
    Verse {
        reference: ReferenceId,
        locator: String,
    },
    /// Refers to a Hadith locator within a generic reference.
    Hadith {
        reference: ReferenceId,
        locator: String,
    },
    /// Refers to a text span in a generic reference.
    TextSpan {
        reference: ReferenceId,
        span: TextSpan,
    },
}

/// Validated structured pointer to source material.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvidenceSourceReference {
    kind: EvidenceSourceKind,
}

impl EvidenceSourceReference {
    /// Validates and wraps a structured source-location kind.
    pub fn new(kind: EvidenceSourceKind) -> Result<Self, EvidenceSourceReferenceError> {
        match &kind {
            EvidenceSourceKind::Source(_)
            | EvidenceSourceKind::Document(_)
            | EvidenceSourceKind::Passage(_) => {}
            EvidenceSourceKind::SourceVersion { version, .. } => {
                validate_locator(version, "source version")?;
            }
            EvidenceSourceKind::Section { section, .. } => {
                validate_locator(section, "section")?;
            }
            EvidenceSourceKind::Page { page, .. } => {
                validate_locator(page, "page")?;
            }
            EvidenceSourceKind::Paragraph { paragraph, .. } => {
                validate_locator(paragraph, "paragraph")?;
            }
            EvidenceSourceKind::Verse { locator, .. } => {
                validate_locator(locator, "verse locator")?;
            }
            EvidenceSourceKind::Hadith { locator, .. } => {
                validate_locator(locator, "Hadith locator")?;
            }
            EvidenceSourceKind::TextSpan { .. } => {}
        }
        Ok(Self { kind })
    }

    /// Creates a source-level reference.
    #[must_use]
    pub fn source(source_id: SourceId) -> Self {
        Self {
            kind: EvidenceSourceKind::Source(source_id),
        }
    }

    /// Creates a source-version reference after validating its opaque version label.
    pub fn source_version(
        source_id: SourceId,
        version: impl Into<String>,
    ) -> Result<Self, EvidenceSourceReferenceError> {
        Self::new(EvidenceSourceKind::SourceVersion {
            source_id,
            version: version.into(),
        })
    }

    /// Creates a document reference.
    #[must_use]
    pub fn document(reference: ReferenceId) -> Self {
        Self {
            kind: EvidenceSourceKind::Document(reference),
        }
    }

    /// Creates a passage reference.
    #[must_use]
    pub fn passage(reference: ReferenceId) -> Self {
        Self {
            kind: EvidenceSourceKind::Passage(reference),
        }
    }

    /// Creates a named section reference.
    pub fn section(
        reference: ReferenceId,
        section: impl Into<String>,
    ) -> Result<Self, EvidenceSourceReferenceError> {
        Self::new(EvidenceSourceKind::Section {
            reference,
            section: section.into(),
        })
    }

    /// Creates a page reference.
    pub fn page(
        reference: ReferenceId,
        page: impl Into<String>,
    ) -> Result<Self, EvidenceSourceReferenceError> {
        Self::new(EvidenceSourceKind::Page {
            reference,
            page: page.into(),
        })
    }

    /// Creates a paragraph reference.
    pub fn paragraph(
        reference: ReferenceId,
        paragraph: impl Into<String>,
    ) -> Result<Self, EvidenceSourceReferenceError> {
        Self::new(EvidenceSourceKind::Paragraph {
            reference,
            paragraph: paragraph.into(),
        })
    }

    /// Creates a verse/ayah reference.
    pub fn verse(
        reference: ReferenceId,
        locator: impl Into<String>,
    ) -> Result<Self, EvidenceSourceReferenceError> {
        Self::new(EvidenceSourceKind::Verse {
            reference,
            locator: locator.into(),
        })
    }

    /// Creates a Hadith reference.
    pub fn hadith(
        reference: ReferenceId,
        locator: impl Into<String>,
    ) -> Result<Self, EvidenceSourceReferenceError> {
        Self::new(EvidenceSourceKind::Hadith {
            reference,
            locator: locator.into(),
        })
    }

    /// Creates a textual-span reference.
    #[must_use]
    pub fn text_span(reference: ReferenceId, span: TextSpan) -> Self {
        Self {
            kind: EvidenceSourceKind::TextSpan { reference, span },
        }
    }

    /// Returns the validated source-location kind.
    #[must_use]
    pub fn kind(&self) -> &EvidenceSourceKind {
        &self.kind
    }
}

/// Invalid source-location metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceSourceReferenceError {
    /// A source locator was empty or whitespace-only.
    EmptyLocator { kind: &'static str },
    /// A source locator contained a Unicode control character.
    ControlCharacter { kind: &'static str, index: usize },
}

impl fmt::Display for EvidenceSourceReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLocator { kind } => write!(formatter, "{kind} locator must not be empty"),
            Self::ControlCharacter { kind, index } => write!(
                formatter,
                "{kind} locator contains a control character at index {index}"
            ),
        }
    }
}

impl std::error::Error for EvidenceSourceReferenceError {}

/// A first-class item of evidence, independently addressable from assertions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    id: EvidenceId,
    mechanism: EvidenceMechanism,
    source_references: BTreeSet<EvidenceSourceReference>,
}

impl Evidence {
    /// Creates and validates an evidence item.
    pub fn new<I>(
        id: EvidenceId,
        mechanism: EvidenceMechanism,
        source_references: I,
    ) -> Result<Self, EvidenceValidationError>
    where
        I: IntoIterator<Item = EvidenceSourceReference>,
    {
        let source_references = source_references.into_iter().collect::<BTreeSet<_>>();
        let value = Self {
            id,
            mechanism,
            source_references,
        };
        value.validate()?;
        Ok(value)
    }

    /// Creates evidence that directly references one or more source locations.
    pub fn direct_source<I>(
        id: EvidenceId,
        source_references: I,
    ) -> Result<Self, EvidenceValidationError>
    where
        I: IntoIterator<Item = EvidenceSourceReference>,
    {
        Self::new(id, EvidenceMechanism::DirectSource, source_references)
    }

    /// Creates derived evidence and records its upstream evidence dependencies.
    pub fn derived_support<I, S>(
        id: EvidenceId,
        inputs: I,
        source_references: S,
    ) -> Result<Self, EvidenceValidationError>
    where
        I: IntoIterator<Item = EvidenceId>,
        S: IntoIterator<Item = EvidenceSourceReference>,
    {
        Self::new(
            id,
            EvidenceMechanism::derived_support(inputs)?,
            source_references,
        )
    }

    /// Returns the evidence identity.
    #[must_use]
    pub fn id(&self) -> &EvidenceId {
        &self.id
    }

    /// Returns the direct/derived evidence mechanism.
    #[must_use]
    pub fn mechanism(&self) -> &EvidenceMechanism {
        &self.mechanism
    }

    /// Returns all structured source references in deterministic order.
    #[must_use]
    pub fn source_references(&self) -> &BTreeSet<EvidenceSourceReference> {
        &self.source_references
    }

    /// Validates mechanism-specific invariants.
    pub fn validate(&self) -> Result<(), EvidenceValidationError> {
        match &self.mechanism {
            EvidenceMechanism::DirectSource if self.source_references.is_empty() => {
                Err(EvidenceValidationError::DirectSourceRequiresSourceReference)
            }
            EvidenceMechanism::DerivedSupport { inputs } if inputs.is_empty() => {
                Err(EvidenceValidationError::DerivedSupportRequiresInput)
            }
            EvidenceMechanism::DerivedSupport { inputs } if inputs.contains(&self.id) => {
                Err(EvidenceValidationError::CannotDependOnItself {
                    evidence_id: self.id.clone(),
                })
            }
            EvidenceMechanism::DirectSource | EvidenceMechanism::DerivedSupport { .. } => Ok(()),
        }
    }
}

/// Structural evidence-model failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceValidationError {
    /// Direct-source evidence must identify at least one source location.
    DirectSourceRequiresSourceReference,
    /// Derived support must name at least one upstream evidence record.
    DerivedSupportRequiresInput,
    /// Evidence cannot directly depend on itself.
    CannotDependOnItself { evidence_id: EvidenceId },
}

impl fmt::Display for EvidenceValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectSourceRequiresSourceReference => formatter
                .write_str("direct-source evidence must reference at least one source location"),
            Self::DerivedSupportRequiresInput => formatter
                .write_str("derived evidence must reference at least one upstream evidence record"),
            Self::CannotDependOnItself { evidence_id } => {
                write!(formatter, "evidence {evidence_id} cannot depend on itself")
            }
        }
    }
}

impl std::error::Error for EvidenceValidationError {}

fn validate_locator(value: &str, kind: &'static str) -> Result<(), EvidenceSourceReferenceError> {
    validate_nonempty_label(value, kind).map_err(|error| match error {
        EvidenceRoleError::EmptyName => EvidenceSourceReferenceError::EmptyLocator { kind },
        EvidenceRoleError::ControlCharacter { index } => {
            EvidenceSourceReferenceError::ControlCharacter { kind, index }
        }
    })
}

fn validate_nonempty_label(value: &str, kind: &'static str) -> Result<(), EvidenceRoleError> {
    if value.trim().is_empty() {
        return Err(EvidenceRoleError::EmptyName);
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(EvidenceRoleError::ControlCharacter { index });
    }
    let _ = kind;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Evidence, EvidenceMechanism, EvidenceRole, EvidenceSourceKind, EvidenceSourceReference,
        EvidenceSourceReferenceError, EvidenceValidationError, TextOffsetUnit, TextSpan,
        TextSpanError,
    };
    use crate::identity::{EvidenceId, ReferenceId, SourceId};
    use std::collections::BTreeSet;

    fn evidence_id(value: &str) -> EvidenceId {
        EvidenceId::new(value).expect("valid evidence identity")
    }

    fn source_id(value: &str) -> SourceId {
        SourceId::new(value).expect("valid source identity")
    }

    fn reference_id(value: &str) -> ReferenceId {
        ReferenceId::new(value).expect("valid reference identity")
    }

    #[test]
    fn evidence_role_preserves_builtin_and_custom_roles() {
        assert_eq!(EvidenceRole::Supports.as_str(), "supports");
        let custom = EvidenceRole::custom("scholarly-corroboration").expect("valid custom role");
        assert_eq!(custom.as_str(), "scholarly-corroboration");
    }

    #[test]
    fn custom_evidence_role_rejects_empty_and_control_values() {
        assert!(EvidenceRole::custom("  ").is_err());
        assert!(EvidenceRole::custom("bad\nrole").is_err());
    }

    #[test]
    fn text_span_preserves_offsets_and_unit() {
        let span = TextSpan::new(3, 9, TextOffsetUnit::Utf8Byte).expect("valid span");
        assert_eq!(span.start(), 3);
        assert_eq!(span.end(), 9);
        assert_eq!(span.unit(), TextOffsetUnit::Utf8Byte);
    }

    #[test]
    fn empty_and_reversed_text_spans_are_rejected() {
        assert_eq!(
            TextSpan::new(4, 4, TextOffsetUnit::UnicodeScalar),
            Err(TextSpanError::InvalidBounds { start: 4, end: 4 })
        );
        assert!(TextSpan::new(7, 2, TextOffsetUnit::Utf16CodeUnit).is_err());
    }

    #[test]
    fn structured_source_references_cover_source_document_passage_and_text_span() {
        let source = EvidenceSourceReference::source(source_id("source-quran"));
        let document = EvidenceSourceReference::document(reference_id("document-quran"));
        let passage = EvidenceSourceReference::passage(reference_id("passage-1"));
        let span = EvidenceSourceReference::text_span(
            reference_id("text-reference"),
            TextSpan::new(0, 12, TextOffsetUnit::UnicodeScalar).expect("valid span"),
        );

        assert!(matches!(source.kind(), EvidenceSourceKind::Source(_)));
        assert!(matches!(document.kind(), EvidenceSourceKind::Document(_)));
        assert!(matches!(passage.kind(), EvidenceSourceKind::Passage(_)));
        assert!(matches!(span.kind(), EvidenceSourceKind::TextSpan { .. }));
    }

    #[test]
    fn structured_source_references_support_versions_sections_pages_paragraphs_and_citations() {
        let reference = reference_id("quran-reference");
        let values = [
            EvidenceSourceReference::source_version(source_id("quran"), "edition-1")
                .expect("valid version"),
            EvidenceSourceReference::section(reference.clone(), "Chapter 2")
                .expect("valid section"),
            EvidenceSourceReference::page(reference.clone(), "17").expect("valid page"),
            EvidenceSourceReference::paragraph(reference.clone(), "paragraph-3")
                .expect("valid paragraph"),
            EvidenceSourceReference::verse(reference.clone(), "2:255").expect("valid verse"),
            EvidenceSourceReference::hadith(reference, "book-1/hadith-12")
                .expect("valid Hadith locator"),
        ];

        assert_eq!(values.len(), 6);
    }

    #[test]
    fn malformed_source_location_labels_are_rejected() {
        assert!(matches!(
            EvidenceSourceReference::source_version(source_id("quran"), " "),
            Err(EvidenceSourceReferenceError::EmptyLocator {
                kind: "source version"
            })
        ));
        assert!(EvidenceSourceReference::section(reference_id("ref-1"), "section\n1").is_err());
        assert!(EvidenceSourceReference::verse(reference_id("ref-1"), "\t").is_err());
    }

    #[test]
    fn direct_evidence_requires_a_source_reference() {
        assert_eq!(
            Evidence::direct_source(evidence_id("evidence-1"), []),
            Err(EvidenceValidationError::DirectSourceRequiresSourceReference)
        );
    }

    #[test]
    fn direct_evidence_preserves_its_identity_mechanism_and_sources() {
        let source = EvidenceSourceReference::source(source_id("source-1"));
        let evidence = Evidence::direct_source(evidence_id("evidence-1"), [source.clone()])
            .expect("direct source evidence should be valid");

        assert_eq!(evidence.id().as_str(), "evidence-1");
        assert_eq!(evidence.mechanism(), &EvidenceMechanism::DirectSource);
        assert!(evidence.source_references().contains(&source));
    }

    #[test]
    fn derived_evidence_requires_upstream_evidence() {
        assert_eq!(
            EvidenceMechanism::derived_support([]),
            Err(EvidenceValidationError::DerivedSupportRequiresInput)
        );
    }

    #[test]
    fn derived_evidence_preserves_deduplicated_input_links() {
        let parent = evidence_id("parent-evidence");
        let mechanism = EvidenceMechanism::derived_support([parent.clone(), parent.clone()])
            .expect("valid derived mechanism");
        let evidence = Evidence::new(evidence_id("derived-evidence"), mechanism, [])
            .expect("derived evidence may refer to other evidence without a direct source");

        assert_eq!(evidence.mechanism().inputs().unwrap().len(), 1);
        assert!(evidence.mechanism().inputs().unwrap().contains(&parent));
    }

    #[test]
    fn evidence_cannot_depend_on_its_own_identity() {
        let id = evidence_id("same-evidence");
        let mechanism = EvidenceMechanism::DerivedSupport {
            inputs: BTreeSet::from([id.clone()]),
        };

        assert_eq!(
            Evidence::new(id.clone(), mechanism, []),
            Err(EvidenceValidationError::CannotDependOnItself { evidence_id: id })
        );
    }

    #[test]
    fn multiple_evidence_records_remain_distinct() {
        let source = EvidenceSourceReference::source(source_id("source-1"));
        let first = Evidence::direct_source(evidence_id("evidence-1"), [source.clone()])
            .expect("first evidence");
        let second =
            Evidence::direct_source(evidence_id("evidence-2"), [source]).expect("second evidence");

        assert_ne!(first.id(), second.id());
    }
}
