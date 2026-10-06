//! Knowledge Graph engine module boundary.
//!
//! This module composes the Phase 0 engine implementation while keeping the
//! individual responsibilities separated:
//!
//! - `contract::Capability` owns the minimal Phase 0 capability contract.
//! - `lifecycle` owns lifecycle sequencing over Core's runtime.
//! - `registration` owns the Core Control Plane registration adapter.
//! - `runtime` owns the KG-facing runtime boundary.
//! - `facade` composes those boundaries into the top-level `KgEngine`.
//!
//! The module itself does not introduce another runtime, registry, lifecycle
//! state machine, or request/response system.
//! Phase 0 scaffolding intentionally retains the complete Core integration surface
//! before the final crate-level public wiring is added.
//! The implementation is kept intact; this local allowance prevents intermediate
//! dead-code diagnostics from masking real errors.
#![allow(dead_code)]

mod facade;
mod lifecycle;
mod registration;
mod runtime;

pub use facade::KgEngine;

#[cfg(test)]
mod tests {
    use super::facade::KgEngine;
    use crate::contract::Capability;

    use nizaam_core::capability::{CapabilityDispatchResult, CapabilityInvocation};
    use nizaam_core::control_plane::registry::EngineRegistry;
    use nizaam_core::identity::{
        ContractId, CorrelationId, EngineId, EngineInstanceId, OperationId,
    };
    use nizaam_core::operation::{Operation, OperationContext};
    use nizaam_core::runtime::LifecycleState;

    fn engine_id() -> EngineId {
        EngineId::new("nizaam.kg.engine.integration").expect("test engine id must be valid")
    }

    fn instance_id() -> EngineInstanceId {
        EngineInstanceId::new("nizaam.kg.engine.integration.instance")
            .expect("test engine instance id must be valid")
    }

    fn operation_context(name: &str) -> OperationContext {
        OperationContext::new(Operation::new(
            OperationId::new(format!("nizaam.kg.engine.{name}.operation"))
                .expect("test operation id must be valid"),
            CorrelationId::new(format!("nizaam.kg.engine.{name}.correlation"))
                .expect("test correlation id must be valid"),
        ))
    }

    #[test]
    fn level2_engine_composes_runtime_registration_and_phase0_capability() {
        let mut engine = KgEngine::new(engine_id(), instance_id());
        let registry = EngineRegistry::new();

        assert_eq!(engine.state(), LifecycleState::Created);

        engine.start().expect("Core startup should succeed");
        engine
            .begin_registration()
            .expect("engine should enter registering state");

        engine
            .register_engine(&registry)
            .expect("engine should register through Core");

        let capability_id = engine
            .register_phase0_capability()
            .expect("Phase 0 capability should register");

        engine
            .mark_ready()
            .expect("engine should become ready after required registration");
        engine.serve().expect("engine should enter serving");

        assert_eq!(engine.state(), LifecycleState::Serving);
        assert!(engine.is_registered());
        assert_eq!(engine.engine_id(), &engine_id());
        assert_eq!(engine.engine_instance_id(), &instance_id());
        assert!(registry.contains(engine.engine_instance_id()));
        assert_eq!(registry.len(), 1);
        assert_eq!(capability_id, Capability::id());
    }

    #[test]
    fn level2_registered_phase0_capability_executes_through_core_backed_dispatch() {
        let mut engine = KgEngine::new(engine_id(), instance_id());
        let registry = EngineRegistry::new();

        engine.start().expect("Core startup should succeed");
        engine
            .begin_registration()
            .expect("engine should enter registering state");
        engine
            .register_engine(&registry)
            .expect("engine should register through Core");
        engine
            .register_phase0_capability()
            .expect("Phase 0 capability should register");
        engine.mark_ready().expect("engine should become ready");
        engine.serve().expect("engine should enter serving");

        let context = engine.runtime().context(operation_context("phase0"));
        let invocation = CapabilityInvocation::new(
            Capability::id(),
            ContractId::new("nizaam.kg.engine.integration.phase0.contract")
                .expect("contract id must be valid"),
            b"phase0-integration".to_vec(),
        );

        let result = engine
            .runtime()
            .dispatch(engine.capabilities(), &context, &invocation)
            .expect("serving runtime should admit capability dispatch");

        assert!(matches!(
            result,
            CapabilityDispatchResult::Outcome(outcome)
                if outcome.as_bytes() == b"phase0-integration"
        ));
    }

    #[test]
    fn level2_facade_preserves_core_registration_identity() {
        let mut engine = KgEngine::new(engine_id(), instance_id());
        let registry = EngineRegistry::new();

        engine.start().expect("Core startup should succeed");
        engine
            .begin_registration()
            .expect("engine should enter registering state");
        engine
            .register_engine(&registry)
            .expect("engine should register through Core");

        let record = registry
            .get(engine.engine_instance_id())
            .expect("registered instance should be present");

        assert_eq!(record.engine_id(), engine.engine_id());
        assert_eq!(record.engine_instance_id(), engine.engine_instance_id());
    }

    #[test]
    fn level2_runtime_and_registration_share_the_same_identity() {
        let engine = KgEngine::new(engine_id(), instance_id());

        assert_eq!(
            engine.registration().engine_id(),
            engine.runtime().engine_id()
        );
        assert_eq!(
            engine.registration().engine_instance_id(),
            engine.runtime().engine_instance_id()
        );
    }
}
