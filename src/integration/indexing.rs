//! Knowledge Graph integration contracts for the Nizaam Indexing Engine.
//!
//! Core owns UniversalRequest/UniversalResponse and the engine communication
//! boundary. Indexing owns the typed indexing event and the assigned identifier
//! for a source object. The internal `IndexId` is an Indexing assignment-operation
//! identifier and is intentionally not exposed through the KG publication API.
//! This module carries readiness and post-publication synchronization records
//! for the publication boundary. Phase 6 query/search access contracts live in
//! `crate::index` and delegate to Indexing without being duplicated here. This
//! module does not implement transport, index activation, provider selection,
//! query planning, or an Indexing lifecycle.

/// Core request envelope used to carry an Indexing request.
///
/// This is an alias, not a new KG request protocol.
pub type CoreIndexingRequest = nizaam_core::contracts::UniversalRequest;

/// Core response envelope used to carry an Indexing response.
///
/// This is an alias, not a new KG response protocol.
pub type CoreIndexingResponse = nizaam_core::contracts::UniversalResponse;

/// Indexing-owned typed request content carried through Core's request boundary.
pub type TypedIndexingEvent = nizaam_indexing::event::index_event::IndexEvent;

/// Indexing-owned typed result content carried through Core's response boundary.
pub type TypedIndexingEventResponse =
    nizaam_indexing::event::index_event_response::IndexEventResponse;

/// Why an Indexing acknowledgement may not yet satisfy KG publication readiness.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexingReadinessBlocker {
    /// The Indexing Engine reported that the relevant indexing resource is unavailable.
    Unavailable,
    /// The relevant index is stale for the publication requirement.
    Stale,
    /// The relevant index is rebuilding and cannot satisfy the requirement yet.
    Rebuilding,
    /// An integrity/readiness check did not pass.
    IntegrityFailure,
    /// Indexing could not admit the work because of capacity pressure.
    CapacityPressure,
    /// A required Indexing dependency is unavailable.
    DependencyUnavailable,
    /// A source-specific or future Indexing reason not represented above.
    Other(String),
}

/// KG publication gate derived from the Indexing Engine's acknowledgement.
///
/// `Ready` means the relevant Indexing readiness precondition is acknowledged.
/// It does not activate an index or publish KG state. `Unknown` and blocked
/// states fail closed.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexingPublicationReadiness {
    /// Indexing explicitly acknowledged readiness for the publication requirement.
    Ready,
    /// Indexing explicitly reported that the requirement is not yet ready.
    Blocked(IndexingReadinessBlocker),
    /// No reliable readiness acknowledgement is available.
    Unknown,
}

impl IndexingPublicationReadiness {
    /// Returns whether this state satisfies the KG's publication precondition.
    #[must_use]
    pub const fn permits_publication(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Typed acknowledgement linking Indexing readiness to a source-owned object.
///
/// Uses `IndexAssignedId`, which identifies the target/source object exposed by
/// Indexing's typed response. The internal Indexing `IndexId` identifies an
/// assignment operation and does not cross this public KG boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexingReadinessReceipt {
    assigned_id: nizaam_indexing::IndexAssignedId,
    object_reference: nizaam_indexing::index::reference::ObjectReference,
    readiness: IndexingPublicationReadiness,
}

impl IndexingReadinessReceipt {
    /// Records a readiness acknowledgement for an assigned source object.
    #[must_use]
    pub fn new(
        assigned_id: nizaam_indexing::IndexAssignedId,
        object_reference: nizaam_indexing::index::reference::ObjectReference,
        readiness: IndexingPublicationReadiness,
    ) -> Self {
        Self {
            assigned_id,
            object_reference,
            readiness,
        }
    }

    /// Returns Indexing's assigned identity for the source object.
    #[must_use]
    pub fn assigned_id(&self) -> &nizaam_indexing::IndexAssignedId {
        &self.assigned_id
    }

    /// Returns the opaque source-owned object reference.
    #[must_use]
    pub fn object_reference(&self) -> &nizaam_indexing::index::reference::ObjectReference {
        &self.object_reference
    }

    /// Returns the readiness state reported to the KG publication boundary.
    #[must_use]
    pub fn readiness(&self) -> &IndexingPublicationReadiness {
        &self.readiness
    }

    /// Returns the assigned object identity only when readiness is satisfied.
    pub fn require_ready(
        &self,
    ) -> Result<&nizaam_indexing::IndexAssignedId, IndexingIntegrationError> {
        match &self.readiness {
            IndexingPublicationReadiness::Ready => Ok(&self.assigned_id),
            IndexingPublicationReadiness::Blocked(_) => {
                Err(IndexingIntegrationError::PublicationNotReady)
            }
            IndexingPublicationReadiness::Unknown => {
                Err(IndexingIntegrationError::ReadinessUnknown)
            }
        }
    }
}

/// Last observed post-publication synchronization state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexingSynchronizationStatus {
    /// Synchronization has been requested but no completion has been acknowledged.
    Pending,
    /// Indexing acknowledged synchronization for the published state.
    Synchronized,
    /// Synchronization failed in a way that permits a targeted retry.
    RetryableFailure,
    /// Synchronization failed and requires an explicit decision before retrying.
    TerminalFailure,
}

impl IndexingSynchronizationStatus {
    /// Returns whether synchronization has been acknowledged as complete.
    #[must_use]
    pub const fn is_synchronized(self) -> bool {
        matches!(self, Self::Synchronized)
    }

    /// Returns whether this status calls for a targeted synchronization retry.
    #[must_use]
    pub const fn requires_retry(self) -> bool {
        matches!(self, Self::Pending | Self::RetryableFailure)
    }
}

/// Immutable value describing the latest observed synchronization outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexingSynchronizationRecord {
    assigned_id: nizaam_indexing::IndexAssignedId,
    object_reference: nizaam_indexing::index::reference::ObjectReference,
    status: IndexingSynchronizationStatus,
}

impl IndexingSynchronizationRecord {
    /// Creates a synchronization status value for a published source object.
    #[must_use]
    pub fn new(
        assigned_id: nizaam_indexing::IndexAssignedId,
        object_reference: nizaam_indexing::index::reference::ObjectReference,
        status: IndexingSynchronizationStatus,
    ) -> Self {
        Self {
            assigned_id,
            object_reference,
            status,
        }
    }

    /// Returns Indexing's assigned identity for the source object.
    #[must_use]
    pub fn assigned_id(&self) -> &nizaam_indexing::IndexAssignedId {
        &self.assigned_id
    }

    /// Returns the source-owned object reference associated with synchronization.
    #[must_use]
    pub fn object_reference(&self) -> &nizaam_indexing::index::reference::ObjectReference {
        &self.object_reference
    }

    /// Returns the last observed synchronization status.
    #[must_use]
    pub const fn status(&self) -> IndexingSynchronizationStatus {
        self.status
    }
}

/// Errors raised when KG ingestion attempts to pass the Indexing publication gate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IndexingIntegrationError {
    /// Indexing explicitly reported a blocked readiness condition.
    PublicationNotReady,
    /// The caller has no reliable readiness acknowledgement.
    ReadinessUnknown,
}

impl core::fmt::Display for IndexingIntegrationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PublicationNotReady => {
                formatter.write_str("Indexing readiness precondition is not satisfied")
            }
            Self::ReadinessUnknown => {
                formatter.write_str("Indexing readiness has not been acknowledged")
            }
        }
    }
}
impl std::error::Error for IndexingIntegrationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use nizaam_indexing::index::reference::ObjectReference;
    use nizaam_indexing::{IndexAssignedId, TargetReferenceType};

    fn assigned_id(reference: &ObjectReference) -> IndexAssignedId {
        let target_type = TargetReferenceType::new("entity").expect("valid target reference type");
        IndexAssignedId::generate(&target_type, reference)
    }

    #[test]
    fn only_explicit_ready_state_satisfies_publication_gate() {
        let reference = ObjectReference::new("kg", "entity-1").expect("valid object reference");
        let ready = IndexingReadinessReceipt::new(
            assigned_id(&reference),
            reference.clone(),
            IndexingPublicationReadiness::Ready,
        );
        assert!(ready.require_ready().is_ok());

        let blocked = IndexingReadinessReceipt::new(
            assigned_id(&reference),
            reference.clone(),
            IndexingPublicationReadiness::Blocked(IndexingReadinessBlocker::IntegrityFailure),
        );
        assert!(blocked.require_ready().is_err());

        let unknown = IndexingReadinessReceipt::new(
            assigned_id(&reference),
            reference,
            IndexingPublicationReadiness::Unknown,
        );
        assert!(unknown.require_ready().is_err());
    }

    #[test]
    fn synchronization_record_preserves_assigned_object_identity() {
        let reference = ObjectReference::new("kg", "entity-1").expect("valid object reference");
        let assigned = assigned_id(&reference);
        let record = IndexingSynchronizationRecord::new(
            assigned.clone(),
            reference.clone(),
            IndexingSynchronizationStatus::Pending,
        );
        assert_eq!(record.assigned_id(), &assigned);
        assert_eq!(record.object_reference(), &reference);
        assert_eq!(record.status(), IndexingSynchronizationStatus::Pending);
    }

    #[test]
    fn synchronization_status_does_not_conflate_pending_and_completed() {
        assert!(!IndexingSynchronizationStatus::Pending.is_synchronized());
        assert!(IndexingSynchronizationStatus::Synchronized.is_synchronized());
        assert!(IndexingSynchronizationStatus::Pending.requires_retry());
        assert!(IndexingSynchronizationStatus::RetryableFailure.requires_retry());
        assert!(!IndexingSynchronizationStatus::TerminalFailure.requires_retry());
    }
}
