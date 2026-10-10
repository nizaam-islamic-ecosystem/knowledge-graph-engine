//! Operational audit records for changes affecting KG semantic objects.
//!
//! Audit and epistemic provenance are intentionally separate. Provenance
//! describes how knowledge came to exist; audit records what changed, when,
//! and through which Core operation. This module stores audit values but does
//! not replace Core logging, tracing, runtime provenance, or persistence.

use core::fmt;
use std::collections::BTreeSet;

use nizaam_core::identity::OperationId;

use crate::identity::AgentId;
use crate::temporal::Instant;

use super::model::{ProvenanceError, ProvenanceTarget};

/// High-level kind of operational change recorded by the KG audit model.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuditAction {
    /// A semantic object was created or first registered.
    Created,
    /// Current semantic metadata or a controlled representation was changed.
    Updated,
    /// A status value was changed.
    StatusChanged,
    /// A relationship or metadata association was attached.
    Attached,
    /// A relationship or metadata association was detached.
    Detached,
    /// The object was superseded by a successor representation.
    Superseded,
    /// The object was retracted or withdrawn from current use.
    Retracted,
    /// A domain-defined change action.
    Custom(String),
}

impl AuditAction {
    /// Creates a domain-defined audit action.
    pub fn custom(value: impl Into<String>) -> Result<Self, AuditError> {
        let value = value.into();
        validate_label(&value, "audit action")?;
        Ok(Self::Custom(value))
    }

    /// Returns the stable textual label for the action.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::StatusChanged => "status-changed",
            Self::Attached => "attached",
            Self::Detached => "detached",
            Self::Superseded => "superseded",
            Self::Retracted => "retracted",
            Self::Custom(value) => value,
        }
    }

    fn validate(&self) -> Result<(), AuditError> {
        if let Self::Custom(value) = self {
            validate_label(value, "audit action")?;
        }
        Ok(())
    }
}

/// Immutable record of one operational change to a KG target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    operation_id: OperationId,
    target: ProvenanceTarget,
    action: AuditAction,
    occurred_at: Instant,
    actor: Option<AgentId>,
    changed_fields: BTreeSet<String>,
    summary: String,
}

impl AuditRecord {
    /// Creates an audit record linked to a Core operation identity.
    pub fn new(
        operation_id: OperationId,
        target: ProvenanceTarget,
        action: AuditAction,
        occurred_at: Instant,
        summary: impl Into<String>,
    ) -> Result<Self, AuditError> {
        target.validate().map_err(AuditError::InvalidTarget)?;
        action.validate()?;
        let summary = summary.into();
        validate_label(&summary, "audit summary")?;
        Ok(Self {
            operation_id,
            target,
            action,
            occurred_at,
            actor: None,
            changed_fields: BTreeSet::new(),
            summary,
        })
    }

    /// Associates a typed KG agent with the operation, when known.
    #[must_use]
    pub fn with_actor(mut self, actor: AgentId) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Records the semantic field names affected by the operation.
    pub fn with_changed_fields<I>(mut self, fields: I) -> Result<Self, AuditError>
    where
        I: IntoIterator<Item = String>,
    {
        for field in fields {
            validate_label(&field, "changed field")?;
            self.changed_fields.insert(field);
        }
        Ok(self)
    }

    /// Returns the Core-owned operation identity associated with the change.
    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    /// Returns the target affected by the operation.
    #[must_use]
    pub fn target(&self) -> &ProvenanceTarget {
        &self.target
    }

    /// Returns the change action.
    #[must_use]
    pub fn action(&self) -> &AuditAction {
        &self.action
    }

    /// Returns the instant at which the audit event was recorded.
    #[must_use]
    pub const fn occurred_at(&self) -> Instant {
        self.occurred_at
    }

    /// Returns the KG agent responsible for the change, when identified.
    #[must_use]
    pub fn actor(&self) -> Option<&AgentId> {
        self.actor.as_ref()
    }

    /// Returns the affected field names in deterministic order.
    #[must_use]
    pub fn changed_fields(&self) -> &BTreeSet<String> {
        &self.changed_fields
    }

    /// Returns the non-empty human-readable change summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

/// Append-only in-memory operational audit trail.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuditTrail {
    records: Vec<AuditRecord>,
}

impl AuditTrail {
    /// Creates an empty audit trail.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an audit record. Identical records are not appended twice.
    /// Existing records cannot be edited or removed through this API.
    pub fn append(&mut self, record: AuditRecord) -> bool {
        if self.records.contains(&record) {
            return false;
        }
        self.records.push(record);
        true
    }

    /// Returns records in append order.
    #[must_use]
    pub fn records(&self) -> &[AuditRecord] {
        &self.records
    }

    /// Returns the number of audit records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether the trail is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Failures while creating audit records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditError {
    /// Target did not satisfy the provenance target's structural constraints.
    InvalidTarget(ProvenanceError),
    /// A required label was empty or whitespace-only.
    EmptyLabel { field: &'static str },
    /// A label contained a control character.
    ControlCharacter { field: &'static str, index: usize },
}

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(error) => write!(formatter, "invalid audit target: {error}"),
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

impl std::error::Error for AuditError {}

fn validate_label(value: &str, field: &'static str) -> Result<(), AuditError> {
    if value.trim().is_empty() {
        return Err(AuditError::EmptyLabel { field });
    }
    if let Some(index) = value.chars().position(char::is_control) {
        return Err(AuditError::ControlCharacter { field, index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AuditAction, AuditError, AuditRecord, AuditTrail};
    use crate::identity::{AgentId, EntityId};
    use crate::provenance::ProvenanceTarget;
    use crate::temporal::Instant;
    use nizaam_core::identity::OperationId;

    fn record(action: AuditAction) -> AuditRecord {
        AuditRecord::new(
            OperationId::new("nizaam.kg.audit.update").expect("valid Core operation identity"),
            ProvenanceTarget::Entity(EntityId::new("entity-audit").expect("valid entity")),
            action,
            Instant::from_unix_seconds(123),
            "Update entity display metadata",
        )
        .expect("valid audit record")
    }

    #[test]
    fn audit_record_captures_operation_target_time_actor_and_change_details() {
        let actor = AgentId::new("agent-editor").expect("valid agent");
        let record = record(AuditAction::Updated)
            .with_actor(actor.clone())
            .with_changed_fields(["label".to_owned(), "aliases".to_owned()])
            .expect("valid field list");

        assert_eq!(record.actor(), Some(&actor));
        assert_eq!(record.changed_fields().len(), 2);
        assert_eq!(record.action(), &AuditAction::Updated);
        assert_eq!(record.occurred_at(), Instant::from_unix_seconds(123));
    }

    #[test]
    fn audit_trail_appends_records_without_overwriting_prior_events() {
        let first = record(AuditAction::Updated);
        let second = record(AuditAction::StatusChanged);
        let mut trail = AuditTrail::new();

        assert!(trail.append(first.clone()));
        assert!(trail.append(second.clone()));
        assert!(!trail.append(second));
        assert_eq!(trail.len(), 2);
        assert_eq!(trail.records()[0], first);
        assert_eq!(trail.records()[1].action(), &AuditAction::StatusChanged);
    }

    #[test]
    fn invalid_audit_summaries_and_custom_actions_are_rejected() {
        assert!(matches!(
            AuditRecord::new(
                OperationId::new("nizaam.kg.audit.invalid").unwrap(),
                ProvenanceTarget::Entity(EntityId::new("entity-audit").unwrap()),
                AuditAction::Updated,
                Instant::from_unix_seconds(1),
                "  ",
            ),
            Err(AuditError::EmptyLabel { .. })
        ));
        assert!(AuditAction::custom("\n").is_err());
        assert!(AuditAction::custom("kg.policy.changed").is_ok());
    }
}
