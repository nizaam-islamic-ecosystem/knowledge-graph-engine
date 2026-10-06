//! Capability contract for the Nizaam Knowledge Graph Engine.
//!
//! This module defines only the minimal capability contract required to prove
//! the KG engine's Core integration path. Core remains authoritative for
//! capability registration, lookup, lifecycle admission, cancellation,
//! deadline handling, dispatch, and execution coordination.
//!
//! The Phase 0 capability performs no KG work. Its handler simply returns the
//! opaque invocation payload unchanged so later engine/runtime layers can prove
//! that a Core capability can be defined, registered, resolved, invoked, and
//! converted into a Core-owned capability result without introducing a second
//! request/response or dispatch system.
//! Phase 0 scaffolding intentionally retains the complete Core integration surface
//! before the final crate-level public wiring is added.
//! The implementation is kept intact; this local allowance prevents intermediate
//! dead-code diagnostics from masking real errors.
#![allow(dead_code)]

use std::sync::Arc;

use nizaam_core::capability::{
    CapabilityDefinition, CapabilityHandler, CapabilityInvocation, CapabilityOutcome, arc_handler,
};
use nizaam_core::identity::{CapabilityId, EngineId};
use nizaam_core::runtime::EngineContext;

/// Stable identifier for the temporary Phase 0 capability probe.
///
/// This is an integration-test capability, not a final KG domain capability.
pub(crate) const PHASE0_CAPABILITY_ID: &str = "nizaam.kg.phase0.probe";

/// Human-readable name for the Phase 0 capability probe.
pub(crate) const PHASE0_CAPABILITY_NAME: &str = "Phase 0 capability probe";

/// Minimal KG capability contract.
///
/// The contract owns the capability's identity, Core definition construction,
/// and deliberately trivial handler. It does not own a registry or dispatch
/// algorithm; those remain part of the Core capability infrastructure and the
/// engine/runtime integration boundary.
pub(crate) struct Capability;

impl Capability {
    /// Returns the stable Core capability identity used by the current probe.
    #[must_use]
    pub(crate) fn id() -> CapabilityId {
        CapabilityId::new(PHASE0_CAPABILITY_ID)
            .expect("Phase 0 capability identifier must be valid")
    }

    /// Constructs the Core capability definition owned by the KG engine.
    ///
    /// The supplied [`EngineId`] is the logical KG engine identity. The
    /// capability definition therefore records ownership without introducing a
    /// second engine or capability identity system.
    #[must_use]
    pub(crate) fn definition(engine_id: &EngineId) -> CapabilityDefinition {
        CapabilityDefinition::new(Self::id(), engine_id.clone(), PHASE0_CAPABILITY_NAME)
            .expect("Phase 0 capability definition must be valid")
    }

    /// Creates the current capability handler.
    ///
    /// The handler deliberately treats the invocation payload as opaque data
    /// and returns it unchanged. No KG semantic interpretation, storage,
    /// indexing, search, traversal, ingestion, or reasoning is performed.
    #[must_use]
    pub(crate) fn handler() -> Arc<dyn CapabilityHandler> {
        arc_handler(
            |_context: &EngineContext, invocation: &CapabilityInvocation| {
                Ok(CapabilityOutcome::new(invocation.payload_bytes().to_vec()))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use nizaam_core::identity::{ContractId, CorrelationId, OperationId};
    use nizaam_core::operation::{Operation, OperationContext};

    fn engine_id(value: &str) -> EngineId {
        EngineId::new(value).expect("test engine id must be valid")
    }

    fn context() -> EngineContext {
        let operation = Operation::new(
            OperationId::new("nizaam.kg.phase0.test.operation")
                .expect("test operation id must be valid"),
            CorrelationId::new("nizaam.kg.phase0.test.correlation")
                .expect("test correlation id must be valid"),
        );

        EngineContext::new(OperationContext::new(operation))
    }

    fn invocation(payload: &[u8]) -> CapabilityInvocation {
        CapabilityInvocation::new(
            Capability::id(),
            ContractId::new("nizaam.kg.phase0.test.contract")
                .expect("test contract id must be valid"),
            payload.to_vec(),
        )
    }

    fn outcome_bytes(outcome: CapabilityOutcome) -> Vec<u8> {
        outcome.into_bytes()
    }

    #[test]
    fn capability_id_is_stable_and_well_formed() {
        let first = Capability::id();
        let second = Capability::id();

        assert_eq!(first, second);
        assert_eq!(first.as_str(), PHASE0_CAPABILITY_ID);
    }

    #[test]
    fn definition_preserves_logical_engine_ownership() {
        let engine = engine_id("nizaam.knowledge-graph");
        let definition = Capability::definition(&engine);

        assert_eq!(definition.capability_id(), &Capability::id());
        assert_eq!(definition.owning_engine(), &engine);
        assert_eq!(definition.name(), PHASE0_CAPABILITY_NAME);
    }

    #[test]
    fn definition_is_independent_for_distinct_engine_owners() {
        let first_engine = engine_id("nizaam.knowledge-graph.a");
        let second_engine = engine_id("nizaam.knowledge-graph.b");

        let first = Capability::definition(&first_engine);
        let second = Capability::definition(&second_engine);

        assert_eq!(first.capability_id(), second.capability_id());
        assert_ne!(first.owning_engine(), second.owning_engine());
    }

    #[test]
    fn handler_returns_the_opaque_payload_unchanged() {
        let handler = Capability::handler();
        let payload = b"phase0-opaque-payload";
        let invocation = invocation(payload);
        let context = context();

        let outcome = handler
            .invoke(&context, &invocation)
            .expect("Capability handler should succeed");

        assert_eq!(outcome_bytes(outcome), payload);
    }

    #[test]
    fn handler_accepts_an_empty_payload() {
        let handler = Capability::handler();
        let invocation = invocation(&[]);
        let context = context();

        let outcome = handler
            .invoke(&context, &invocation)
            .expect("Capability handler should accept empty opaque payloads");

        assert!(outcome_bytes(outcome).is_empty());
    }
}
