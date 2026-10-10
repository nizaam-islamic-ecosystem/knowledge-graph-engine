//! Immutable-value models for knowledge provenance in Phase 4.
//!
//! This module represents KG-specific epistemic lineage using an initial
//! Activity + Agent model. It layers on Core's operation/provenance facilities;
//! it does not recreate runtime provenance, logging, persistence, or storage
//! versioning.

use core::fmt;
use std::collections::BTreeSet;

use nizaam_core::identity::OperationId;

use crate::identity::{
    ActivityId, AgentId, ConceptId, ContradictionId, EntityId, EvidenceId, KnowledgeAssertionId,
    LexicalFormId, MentionId, ReferenceId, SourceId,
};
use crate::temporal::Instant;

use super::origin::{KnowledgeOrigin, KnowledgeOriginError};

/// General category of an agent participating in a provenance activity.
///
/// The generic categories are deliberately broad. Domain packages can use
/// `Custom` without forcing every future actor kind into the core vocabulary.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AgentType {
    /// A human actor.
    Human,
    /// An organization or institution.
    Organization,
    /// A software application or service.
    Software,
    /// An automated processing pipeline.
    Pipeline,
    /// A model or model-backed process.
    Model,
    /// A system whose more specific nature is not recorded.
    System,
    /// Another known agent category.
    Other,
    /// The agent category is unknown.
    Unknown,
    /// A domain-defined agent category.
    Custom(String),
}

impl AgentType {
    /// Creates a domain-defined agent category.
    pub fn custom(value: impl Into<String>) -> Result<Self, ProvenanceError> {
        let value = value.into();
        validate_label(&value, "agent type")?;
        Ok(Self::Custom(value))
    }

    /// Returns the stable textual label for the category.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Human => "human",
            Self::Organization => "organization",
            Self::Software => "software",
            Self::Pipeline => "pipeline",
            Self::Model => "model",
            Self::System => "system",
            Self::Other => "other",
            Self::Unknown => "unknown",
            Self::Custom(value) => value,
        }
    }

    fn validate(&self) -> Result<(), ProvenanceError> {
        if let Self::Custom(value) = self {
            validate_label(value, "agent type")?;
        }
        Ok(())
    }
}

/// An identifiable human, organization, software, pipeline, model, or other
/// participant in a knowledge-provenance activity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Agent {
    id: AgentId,
    agent_type: AgentType,
    name: String,
}

impl Agent {
    /// Creates an agent with a validated display name and category.
    pub fn new(
        id: AgentId,
        agent_type: AgentType,
        name: impl Into<String>,
    ) -> Result<Self, ProvenanceError> {
        let name = name.into();
        validate_label(&name, "agent name")?;
        agent_type.validate()?;
        Ok(Self {
            id,
            agent_type,
            name,
        })
    }

    /// Returns the Core-backed agent identity.
    #[must_use]
    pub fn id(&self) -> &AgentId {
        &self.id
    }

    /// Returns the general agent category.
    #[must_use]
    pub fn agent_type(&self) -> &AgentType {
        &self.agent_type
    }

    /// Returns the agent's display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Kind of knowledge activity recorded in provenance.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityKind {
    /// Capture or identification of source material.
    SourceCapture,
    /// Extraction of knowledge from source material.
    Extraction,
    /// Transformation or normalization of existing material.
    Transformation,
    /// Human or automated review.
    Review,
    /// Derivation of an object from earlier knowledge.
    Derivation,
    /// Modification of an existing semantic representation.
    Modification,
    /// Import from another system or dataset.
    Import,
    /// Structural or semantic validation.
    Validation,
    /// A domain-defined activity kind.
    Custom(String),
}

impl ActivityKind {
    /// Creates a domain-defined activity kind.
    pub fn custom(value: impl Into<String>) -> Result<Self, ProvenanceError> {
        let value = value.into();
        validate_label(&value, "activity kind")?;
        Ok(Self::Custom(value))
    }

    /// Returns the stable textual label for the activity kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::SourceCapture => "source-capture",
            Self::Extraction => "extraction",
            Self::Transformation => "transformation",
            Self::Review => "review",
            Self::Derivation => "derivation",
            Self::Modification => "modification",
            Self::Import => "import",
            Self::Validation => "validation",
            Self::Custom(value) => value,
        }
    }

    fn validate(&self) -> Result<(), ProvenanceError> {
        if let Self::Custom(value) = self {
            validate_label(value, "activity kind")?;
        }
        Ok(())
    }
}

/// One activity in a knowledge object's historical lineage.
///
/// Builders consume `self`; no API is provided to edit an existing activity in
/// place. Core `OperationId` is an optional link to the operational context
/// rather than a KG-owned replacement identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activity {
    id: ActivityId,
    kind: ActivityKind,
    agents: BTreeSet<AgentId>,
    started_at: Option<Instant>,
    ended_at: Option<Instant>,
    core_operation_id: Option<OperationId>,
    description: Option<String>,
}

impl Activity {
    /// Creates an activity with no optional metadata yet supplied.
    pub fn new(id: ActivityId, kind: ActivityKind) -> Result<Self, ProvenanceError> {
        kind.validate()?;
        Ok(Self {
            id,
            kind,
            agents: BTreeSet::new(),
            started_at: None,
            ended_at: None,
            core_operation_id: None,
            description: None,
        })
    }

    /// Adds an agent participating in the activity.
    #[must_use]
    pub fn with_agent(mut self, agent_id: AgentId) -> Self {
        self.agents.insert(agent_id);
        self
    }

    /// Adds all supplied participants, deduplicating them deterministically.
    pub fn with_agents<I>(mut self, agents: I) -> Self
    where
        I: IntoIterator<Item = AgentId>,
    {
        self.agents.extend(agents);
        self
    }

    /// Adds the activity start instant, rejecting a range that would be reversed.
    pub fn with_started_at(mut self, instant: Instant) -> Result<Self, ProvenanceError> {
        if let Some(ended_at) = self.ended_at
            && instant > ended_at
        {
            return Err(ProvenanceError::InvalidActivityTimeRange {
                started_at: instant,
                ended_at,
            });
        }
        self.started_at = Some(instant);
        Ok(self)
    }

    /// Adds the activity end instant, rejecting a range that would be reversed.
    pub fn with_ended_at(mut self, instant: Instant) -> Result<Self, ProvenanceError> {
        if let Some(started_at) = self.started_at
            && started_at > instant
        {
            return Err(ProvenanceError::InvalidActivityTimeRange {
                started_at,
                ended_at: instant,
            });
        }
        self.ended_at = Some(instant);
        Ok(self)
    }

    /// Links this KG activity to the existing Core operation identity.
    #[must_use]
    pub fn with_core_operation_id(mut self, operation_id: OperationId) -> Self {
        self.core_operation_id = Some(operation_id);
        self
    }

    /// Adds a validated optional description.
    pub fn with_description(mut self, value: impl Into<String>) -> Result<Self, ProvenanceError> {
        let value = value.into();
        validate_label(&value, "activity description")?;
        self.description = Some(value);
        Ok(self)
    }

    /// Returns the activity identity.
    #[must_use]
    pub fn id(&self) -> &ActivityId {
        &self.id
    }

    /// Returns the activity kind.
    #[must_use]
    pub fn kind(&self) -> &ActivityKind {
        &self.kind
    }

    /// Returns participating agent identities.
    #[must_use]
    pub fn agents(&self) -> &BTreeSet<AgentId> {
        &self.agents
    }

    /// Returns the known start instant.
    #[must_use]
    pub const fn started_at(&self) -> Option<Instant> {
        self.started_at
    }

    /// Returns the known end instant.
    #[must_use]
    pub const fn ended_at(&self) -> Option<Instant> {
        self.ended_at
    }

    /// Returns the Core operation identity, when linked.
    #[must_use]
    pub fn core_operation_id(&self) -> Option<&OperationId> {
        self.core_operation_id.as_ref()
    }

    /// Returns the optional activity description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Rechecks invariants at the provenance-record boundary.
    pub fn validate(&self) -> Result<(), ProvenanceError> {
        self.kind.validate()?;
        if let (Some(started_at), Some(ended_at)) = (self.started_at, self.ended_at)
            && started_at > ended_at
        {
            return Err(ProvenanceError::InvalidActivityTimeRange {
                started_at,
                ended_at,
            });
        }
        if let Some(description) = &self.description {
            validate_label(description, "activity description")?;
        }
        Ok(())
    }
}

/// A canonical semantic object that may have a provenance history.
///
/// `Other` supports future KG object types without requiring a new enum variant
/// for every later-phase module. Its type name and identifier are validated by
/// [`ProvenanceTarget::opaque`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProvenanceTarget {
    /// A canonical entity.
    Entity(EntityId),
    /// A canonical concept.
    Concept(ConceptId),
    /// A lexical-form object.
    LexicalForm(LexicalFormId),
    /// A source mention.
    Mention(MentionId),
    /// A knowledge assertion.
    Assertion(KnowledgeAssertionId),
    /// A source identity.
    Source(SourceId),
    /// A generic reference identity.
    Reference(ReferenceId),
    /// A first-class evidence object.
    Evidence(EvidenceId),
    /// A first-class contradiction record.
    Contradiction(ContradictionId),
    /// An extensible reference to a later-phase KG object type.
    Other {
        type_name: String,
        identifier: String,
    },
}

impl ProvenanceTarget {
    /// Creates a target for a later-phase or domain-defined object type.
    pub fn opaque(
        type_name: impl Into<String>,
        identifier: impl Into<String>,
    ) -> Result<Self, ProvenanceError> {
        let type_name = type_name.into();
        let identifier = identifier.into();
        validate_label(&type_name, "provenance target type")?;
        validate_label(&identifier, "provenance target identifier")?;
        Ok(Self::Other {
            type_name,
            identifier,
        })
    }

    /// Validates extensible variants, including values built directly.
    pub fn validate(&self) -> Result<(), ProvenanceError> {
        if let Self::Other {
            type_name,
            identifier,
        } = self
        {
            validate_label(type_name, "provenance target type")?;
            validate_label(identifier, "provenance target identifier")?;
        }
        Ok(())
    }
}

/// Immutable historical association between one target, an activity, and its
/// optional origin metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    target: ProvenanceTarget,
    activity: Activity,
    recorded_at: Instant,
    origin: Option<KnowledgeOrigin>,
}

impl ProvenanceRecord {
    /// Creates a validated historical provenance record.
    pub fn new(
        target: ProvenanceTarget,
        activity: Activity,
        recorded_at: Instant,
        origin: Option<KnowledgeOrigin>,
    ) -> Result<Self, ProvenanceError> {
        target.validate()?;
        activity.validate()?;
        if let Some(origin) = &origin {
            origin.validate().map_err(ProvenanceError::InvalidOrigin)?;
        }
        Ok(Self {
            target,
            activity,
            recorded_at,
            origin,
        })
    }

    /// Returns the semantic target described by this record.
    #[must_use]
    pub fn target(&self) -> &ProvenanceTarget {
        &self.target
    }

    /// Returns the immutable activity snapshot for this record.
    #[must_use]
    pub fn activity(&self) -> &Activity {
        &self.activity
    }

    /// Returns when this provenance record was recorded by the KG.
    #[must_use]
    pub const fn recorded_at(&self) -> Instant {
        self.recorded_at
    }

    /// Returns origin metadata, when known.
    #[must_use]
    pub fn origin(&self) -> Option<&KnowledgeOrigin> {
        self.origin.as_ref()
    }
}

/// Append-only in-memory history for one semantic target.
///
/// Records are exposed only through an immutable slice. The API allows appending
/// a new historical value but intentionally has no edit or removal operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceHistory {
    target: ProvenanceTarget,
    records: Vec<ProvenanceRecord>,
}

impl ProvenanceHistory {
    /// Creates an empty history for one target.
    pub fn new(target: ProvenanceTarget) -> Result<Self, ProvenanceError> {
        target.validate()?;
        Ok(Self {
            target,
            records: Vec::new(),
        })
    }

    /// Appends a new record without overwriting previous history.
    ///
    /// Reusing an activity identity for the same target is rejected to avoid
    /// silently replacing or duplicating a historical event.
    pub fn append(&mut self, record: ProvenanceRecord) -> Result<(), ProvenanceError> {
        if record.target != self.target {
            return Err(ProvenanceError::RecordTargetMismatch {
                expected: self.target.clone(),
                actual: record.target,
            });
        }
        if self
            .records
            .iter()
            .any(|previous| previous.activity.id() == record.activity.id())
        {
            return Err(ProvenanceError::DuplicateActivityRecord {
                activity_id: record.activity.id().clone(),
            });
        }
        self.records.push(record);
        Ok(())
    }

    /// Returns the history's target.
    #[must_use]
    pub fn target(&self) -> &ProvenanceTarget {
        &self.target
    }

    /// Returns all records in append order.
    #[must_use]
    pub fn records(&self) -> &[ProvenanceRecord] {
        &self.records
    }

    /// Returns the number of historical records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether no records have been appended.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Structural failures while creating or appending provenance values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceError {
    /// A required label was empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// A label contained a control character.
    ControlCharacter { field: &'static str, index: usize },
    /// An activity's start is later than its end.
    InvalidActivityTimeRange {
        started_at: Instant,
        ended_at: Instant,
    },
    /// A record's target differs from its owning history's target.
    RecordTargetMismatch {
        expected: ProvenanceTarget,
        actual: ProvenanceTarget,
    },
    /// The same activity was already recorded for this target.
    DuplicateActivityRecord { activity_id: ActivityId },
    /// Origin metadata failed its own structural validation.
    InvalidOrigin(KnowledgeOriginError),
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel { field } => write!(formatter, "{field} must not be empty"),
            Self::ControlCharacter { field, index } => {
                write!(
                    formatter,
                    "{field} contains a control character at index {index}"
                )
            }
            Self::InvalidActivityTimeRange {
                started_at,
                ended_at,
            } => write!(
                formatter,
                "activity start {started_at} must not be later than activity end {ended_at}"
            ),
            Self::RecordTargetMismatch { expected, actual } => write!(
                formatter,
                "provenance record target {actual:?} does not match history target {expected:?}"
            ),
            Self::DuplicateActivityRecord { activity_id } => write!(
                formatter,
                "activity {activity_id} already has a provenance record for this target"
            ),
            Self::InvalidOrigin(error) => write!(formatter, "invalid provenance origin: {error}"),
        }
    }
}

impl std::error::Error for ProvenanceError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), ProvenanceError> {
    if value.trim().is_empty() {
        return Err(ProvenanceError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(ProvenanceError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Activity, ActivityKind, Agent, AgentType, ProvenanceError, ProvenanceHistory,
        ProvenanceRecord, ProvenanceTarget,
    };
    use crate::identity::{ActivityId, AgentId, EntityId, SourceId};
    use crate::provenance::KnowledgeOrigin;
    use crate::temporal::Instant;
    use nizaam_core::identity::OperationId;

    fn activity(id: &str) -> Activity {
        Activity::new(
            ActivityId::new(id).expect("valid activity identity"),
            ActivityKind::Extraction,
        )
        .expect("valid activity")
    }

    #[test]
    fn agent_and_activity_retain_typed_identity_and_metadata() {
        let agent = Agent::new(
            AgentId::new("agent-extractor").expect("valid agent identity"),
            AgentType::Pipeline,
            "quran-extraction-pipeline",
        )
        .expect("valid agent");
        let operation = OperationId::new("nizaam.kg.provenance.extract")
            .expect("valid Core operation identity");
        let activity = activity("activity-extract")
            .with_agent(agent.id().clone())
            .with_core_operation_id(operation.clone())
            .with_description("Extract assertion candidate")
            .expect("valid description");

        assert_eq!(agent.agent_type(), &AgentType::Pipeline);
        assert_eq!(activity.agents().len(), 1);
        assert_eq!(activity.core_operation_id(), Some(&operation));
    }

    #[test]
    fn activity_rejects_reversed_time_ranges() {
        let later = Instant::from_unix_seconds(20);
        let earlier = Instant::from_unix_seconds(10);
        let activity = activity("activity-time").with_started_at(later).unwrap();

        assert!(matches!(
            activity.with_ended_at(earlier),
            Err(ProvenanceError::InvalidActivityTimeRange { .. })
        ));
    }

    #[test]
    fn provenance_history_appends_without_replacing_prior_records() {
        let target = ProvenanceTarget::Entity(
            EntityId::new("entity-provenance").expect("valid entity identity"),
        );
        let origin = KnowledgeOrigin::from_source(
            SourceId::new("source-quran").expect("valid source identity"),
        );
        let mut history = ProvenanceHistory::new(target.clone()).expect("valid history target");
        let first = ProvenanceRecord::new(
            target.clone(),
            activity("activity-import"),
            Instant::from_unix_seconds(1_000),
            Some(origin.clone()),
        )
        .expect("valid first record");
        let second = ProvenanceRecord::new(
            target.clone(),
            activity("activity-review"),
            Instant::from_unix_seconds(2_000),
            Some(origin),
        )
        .expect("valid second record");

        history.append(first.clone()).expect("append first record");
        history
            .append(second.clone())
            .expect("append second record");

        assert_eq!(history.len(), 2);
        assert_eq!(history.records()[0], first);
        assert_eq!(history.records()[1], second);
    }

    #[test]
    fn provenance_history_rejects_duplicate_activity_and_wrong_target() {
        let target =
            ProvenanceTarget::Entity(EntityId::new("entity-a").expect("valid entity identity"));
        let other_target =
            ProvenanceTarget::Entity(EntityId::new("entity-b").expect("valid entity identity"));
        let activity = activity("activity-duplicate");
        let record = ProvenanceRecord::new(
            target.clone(),
            activity.clone(),
            Instant::from_unix_seconds(1),
            None,
        )
        .expect("valid record");
        let wrong_target_record =
            ProvenanceRecord::new(other_target, activity, Instant::from_unix_seconds(2), None)
                .expect("valid record for other target");
        let mut history = ProvenanceHistory::new(target).expect("valid history");
        history.append(record.clone()).expect("first append");

        assert!(matches!(
            history.append(record),
            Err(ProvenanceError::DuplicateActivityRecord { .. })
        ));
        assert!(matches!(
            history.append(wrong_target_record),
            Err(ProvenanceError::RecordTargetMismatch { .. })
        ));
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn opaque_provenance_targets_are_validated() {
        assert!(ProvenanceTarget::opaque("  ", "id").is_err());
        assert!(ProvenanceTarget::opaque("future-object", "  ").is_err());
        assert!(ProvenanceTarget::opaque("future-object", "id-1").is_ok());
    }
}
