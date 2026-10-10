//! Inspectable links among source origins, activities, and resulting KG objects.
//!
//! Lineage records relationships such as extraction, transformation, review,
//! derivation, correction, and modification. This module records those links;
//! it does not execute derivation or reasoning, traverse a graph, or infer new
//! relationships automatically.

use core::fmt;

use crate::identity::ActivityId;
use crate::temporal::Instant;

use super::model::{ProvenanceError, ProvenanceTarget};

/// Meaning of a directed historical lineage link.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LineageKind {
    /// The target was derived from the source.
    DerivedFrom,
    /// The target was extracted from the source.
    ExtractedFrom,
    /// The target was transformed from the source.
    TransformedFrom,
    /// The target was reviewed in relation to the source.
    ReviewedFrom,
    /// The target was modified from the source.
    ModifiedFrom,
    /// The target corrects or supersedes the represented source information.
    CorrectedFrom,
    /// The target was imported from the source.
    ImportedFrom,
    /// A domain-defined lineage relationship.
    Custom(String),
}

impl LineageKind {
    /// Creates a domain-defined lineage relationship.
    pub fn custom(value: impl Into<String>) -> Result<Self, LineageError> {
        let value = value.into();
        validate_label(&value, "lineage kind")?;
        Ok(Self::Custom(value))
    }

    /// Returns a stable textual label for the relationship.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::DerivedFrom => "derived-from",
            Self::ExtractedFrom => "extracted-from",
            Self::TransformedFrom => "transformed-from",
            Self::ReviewedFrom => "reviewed-from",
            Self::ModifiedFrom => "modified-from",
            Self::CorrectedFrom => "corrected-from",
            Self::ImportedFrom => "imported-from",
            Self::Custom(value) => value,
        }
    }

    fn validate(&self) -> Result<(), LineageError> {
        if let Self::Custom(value) = self {
            validate_label(value, "lineage kind")?;
        }
        Ok(())
    }
}

/// One immutable directed relationship between two KG provenance targets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageLink {
    from: ProvenanceTarget,
    to: ProvenanceTarget,
    kind: LineageKind,
    activity_id: Option<ActivityId>,
    recorded_at: Option<Instant>,
    note: Option<String>,
}

impl LineageLink {
    /// Creates a directed lineage link after validating the endpoints and kind.
    pub fn new(
        from: ProvenanceTarget,
        to: ProvenanceTarget,
        kind: LineageKind,
    ) -> Result<Self, LineageError> {
        from.validate().map_err(LineageError::InvalidTarget)?;
        to.validate().map_err(LineageError::InvalidTarget)?;
        if from == to {
            return Err(LineageError::SelfLink);
        }
        kind.validate()?;
        Ok(Self {
            from,
            to,
            kind,
            activity_id: None,
            recorded_at: None,
            note: None,
        })
    }

    /// Associates the link with the activity that produced or recorded it.
    #[must_use]
    pub fn with_activity(mut self, activity_id: ActivityId) -> Self {
        self.activity_id = Some(activity_id);
        self
    }

    /// Records when the lineage link was added to the historical model.
    #[must_use]
    pub fn with_recorded_at(mut self, instant: Instant) -> Self {
        self.recorded_at = Some(instant);
        self
    }

    /// Adds a validated explanatory note.
    pub fn with_note(mut self, note: impl Into<String>) -> Result<Self, LineageError> {
        let note = note.into();
        validate_label(&note, "lineage note")?;
        self.note = Some(note);
        Ok(self)
    }

    /// Returns the source/parent endpoint of the link.
    #[must_use]
    pub fn from(&self) -> &ProvenanceTarget {
        &self.from
    }

    /// Returns the target/derived endpoint of the link.
    #[must_use]
    pub fn to(&self) -> &ProvenanceTarget {
        &self.to
    }

    /// Returns the lineage relationship kind.
    #[must_use]
    pub fn kind(&self) -> &LineageKind {
        &self.kind
    }

    /// Returns the associated activity, if known.
    #[must_use]
    pub fn activity_id(&self) -> Option<&ActivityId> {
        self.activity_id.as_ref()
    }

    /// Returns the recording instant, if known.
    #[must_use]
    pub const fn recorded_at(&self) -> Option<Instant> {
        self.recorded_at
    }

    /// Returns the explanatory note, if supplied.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// Append-only collection of inspectable provenance links.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lineage {
    links: Vec<LineageLink>,
}

impl Lineage {
    /// Creates an empty lineage collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a link. Returns `false` for an identical link already present.
    ///
    /// Existing links cannot be changed or removed through this public API.
    pub fn append(&mut self, link: LineageLink) -> Result<bool, LineageError> {
        link.from.validate().map_err(LineageError::InvalidTarget)?;
        link.to.validate().map_err(LineageError::InvalidTarget)?;
        if link.from == link.to {
            return Err(LineageError::SelfLink);
        }
        link.kind.validate()?;
        if self.links.contains(&link) {
            return Ok(false);
        }
        self.links.push(link);
        Ok(true)
    }

    /// Returns all lineage links in append order.
    #[must_use]
    pub fn links(&self) -> &[LineageLink] {
        &self.links
    }

    /// Returns the number of recorded links.
    #[must_use]
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns whether no links have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

/// Errors produced when defining or appending lineage links.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineageError {
    /// A target failed structural validation.
    InvalidTarget(ProvenanceError),
    /// A target cannot be linked as derived from itself.
    SelfLink,
    /// A label was empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// A label contained a control character.
    ControlCharacter { field: &'static str, index: usize },
}

impl fmt::Display for LineageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(error) => write!(formatter, "invalid lineage target: {error}"),
            Self::SelfLink => formatter.write_str("a provenance target cannot link to itself"),
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

impl std::error::Error for LineageError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), LineageError> {
    if value.trim().is_empty() {
        return Err(LineageError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(LineageError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Lineage, LineageError, LineageKind, LineageLink};
    use crate::identity::{ActivityId, EntityId, SourceId};
    use crate::provenance::ProvenanceTarget;
    use crate::temporal::Instant;

    #[test]
    fn lineage_link_preserves_origin_activity_and_result_target() {
        let source = ProvenanceTarget::Source(SourceId::new("source-1").unwrap());
        let entity = ProvenanceTarget::Entity(EntityId::new("entity-1").unwrap());
        let activity = ActivityId::new("activity-extract").unwrap();
        let link = LineageLink::new(source.clone(), entity.clone(), LineageKind::ExtractedFrom)
            .expect("valid lineage link")
            .with_activity(activity.clone())
            .with_recorded_at(Instant::from_unix_seconds(5))
            .with_note("Extracted from source passage")
            .expect("valid note");

        assert_eq!(link.from(), &source);
        assert_eq!(link.to(), &entity);
        assert_eq!(link.activity_id(), Some(&activity));
        assert_eq!(link.recorded_at(), Some(Instant::from_unix_seconds(5)));
    }

    #[test]
    fn lineage_rejects_self_links_and_deduplicates_identical_links() {
        let target = ProvenanceTarget::Entity(EntityId::new("entity-1").unwrap());
        assert!(matches!(
            LineageLink::new(target.clone(), target.clone(), LineageKind::DerivedFrom),
            Err(LineageError::SelfLink)
        ));

        let link = LineageLink::new(
            ProvenanceTarget::Source(SourceId::new("source-1").unwrap()),
            target,
            LineageKind::DerivedFrom,
        )
        .unwrap();
        let mut lineage = Lineage::new();
        assert!(lineage.append(link.clone()).unwrap());
        assert!(!lineage.append(link).unwrap());
        assert_eq!(lineage.len(), 1);
    }

    #[test]
    fn custom_lineage_kinds_are_validated() {
        assert!(LineageKind::custom("  ").is_err());
        assert!(LineageKind::custom("domain.derives-from").is_ok());
    }
}
