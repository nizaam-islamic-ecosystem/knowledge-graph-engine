//! Phase 4 knowledge provenance and operational audit foundations.
//!
//! KG provenance represents the epistemic lineage of knowledge through
//! origins, activities, agents, resulting objects, and inspectable lineage
//! links. Audit records separately represent operational changes. Core remains
//! authoritative for operation identity and runtime provenance. This module
//! does not implement ingestion governance, persistence, KG versioning, or a
//! standards-complete PROV framework.

mod audit;
mod lineage;
mod model;
mod origin;

pub use audit::{AuditAction, AuditError, AuditRecord, AuditTrail};
pub use lineage::{Lineage, LineageError, LineageKind, LineageLink};
pub use model::{
    Activity, ActivityKind, Agent, AgentType, ProvenanceError, ProvenanceHistory, ProvenanceRecord,
    ProvenanceTarget,
};
pub use origin::{KnowledgeOrigin, KnowledgeOriginError};

// These are the canonical Core-backed types. Re-exporting OperationId does not
// create a provenance-specific operation identity or duplicate Core ownership.
pub use crate::identity::{ActivityId, AgentId};
pub use nizaam_core::identity::OperationId;

#[cfg(test)]
mod tests {
    use super::{
        Activity, ActivityKind, Agent, AgentType, AuditAction, AuditRecord, AuditTrail,
        KnowledgeOrigin, Lineage, LineageKind, LineageLink, ProvenanceHistory, ProvenanceRecord,
        ProvenanceTarget,
    };
    use crate::identity::{ActivityId, AgentId, EntityId, SourceId};
    use crate::temporal::Instant;

    #[test]
    fn public_provenance_boundary_connects_source_activity_agent_and_target() {
        let source_id = SourceId::new("source-public-boundary").expect("valid source");
        let target = ProvenanceTarget::Entity(EntityId::new("entity-public-boundary").unwrap());
        let agent = Agent::new(
            AgentId::new("agent-public-boundary").unwrap(),
            AgentType::Software,
            "kg-importer",
        )
        .expect("valid agent");
        let activity = Activity::new(
            ActivityId::new("activity-public-boundary").unwrap(),
            ActivityKind::Import,
        )
        .expect("valid activity")
        .with_agent(agent.id().clone());
        let record = ProvenanceRecord::new(
            target.clone(),
            activity,
            Instant::from_unix_seconds(100),
            Some(KnowledgeOrigin::from_source(source_id)),
        )
        .expect("valid provenance record");
        let mut history = ProvenanceHistory::new(target.clone()).expect("valid history");
        history.append(record).expect("append history record");

        assert_eq!(history.target(), &target);
        assert_eq!(history.len(), 1);
        assert_eq!(history.records()[0].activity().agents().len(), 1);
    }

    #[test]
    fn lineage_and_audit_are_distinct_public_concepts() {
        let source = ProvenanceTarget::Source(SourceId::new("source-lineage").unwrap());
        let entity = ProvenanceTarget::Entity(EntityId::new("entity-lineage").unwrap());
        let link = LineageLink::new(source, entity.clone(), LineageKind::DerivedFrom)
            .expect("valid lineage link");
        let mut lineage = Lineage::new();
        lineage.append(link).expect("append lineage");

        let operation_id = super::OperationId::new("nizaam.kg.public.audit")
            .expect("valid Core operation identity");
        let audit = AuditRecord::new(
            operation_id,
            entity,
            AuditAction::Updated,
            Instant::from_unix_seconds(200),
            "Update current semantic metadata",
        )
        .expect("valid audit record");
        let mut trail = AuditTrail::new();
        assert!(trail.append(audit));

        assert_eq!(lineage.len(), 1);
        assert_eq!(trail.len(), 1);
        assert_ne!(
            std::any::TypeId::of::<Lineage>(),
            std::any::TypeId::of::<AuditTrail>()
        );
    }
}
