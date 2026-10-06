//! Reusable fixtures for the Knowledge Graph Phase 0 Level 3 test suite.
//!
//! Helpers deliberately build real Core contracts and real KG engine instances.
//! They do not introduce mock lifecycle, registry, request, or capability systems.

use nizaam_core::contracts::metadata::{ContractMetadata, Participants};
use nizaam_core::contracts::{
    ContractDescriptor, EncodedPayload, Interaction, MessageEnvelope, PayloadDescriptor,
    UniversalRequest, Version,
};
use nizaam_core::control_plane::registry::EngineRegistry;
use nizaam_core::identity::{
    CapabilityId, ContractId, CorrelationId, EngineId, EngineInstanceId, MessageId, OperationId,
};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_core::runtime::LifecycleState;

use nizaam_knowledge_graph::engine::KgEngine;

/// Construct a stable test logical engine identity.
pub fn engine_id(name: &str) -> EngineId {
    EngineId::new(format!("nizaam.kg.tests.{name}")).expect("test engine id must be valid")
}

/// Construct a stable test concrete engine-instance identity.
pub fn instance_id(name: &str) -> EngineInstanceId {
    EngineInstanceId::new(format!("nizaam.kg.tests.{name}.instance"))
        .expect("test engine instance id must be valid")
}

/// Construct a fresh KG engine with the requested Core identities.
pub fn new_engine(name: &str) -> KgEngine {
    KgEngine::new(engine_id(name), instance_id(name))
}

/// Construct a deterministic operation context for one scenario.
pub fn operation_context(name: &str) -> OperationContext {
    OperationContext::new(Operation::new(
        OperationId::new(format!("nizaam.kg.tests.{name}.operation"))
            .expect("test operation id must be valid"),
        CorrelationId::new(format!("nizaam.kg.tests.{name}.correlation"))
            .expect("test correlation id must be valid"),
    ))
}

/// Move a fresh engine through Core startup into `Registering`.
pub fn registering_engine(name: &str) -> (KgEngine, EngineRegistry) {
    let engine = new_engine(name);
    let registry = EngineRegistry::new();

    engine.start().expect("Core startup should succeed");
    engine
        .begin_registration()
        .expect("Core should enter Registering");

    (engine, registry)
}

/// Register the engine and Phase 0 capability, then enter `Serving`.
pub fn serving_engine(name: &str) -> (KgEngine, EngineRegistry, CapabilityId) {
    let (mut engine, registry) = registering_engine(name);

    engine
        .register_engine(&registry)
        .expect("engine registration should succeed");

    let capability_id = engine
        .register_phase0_capability()
        .expect("Phase 0 capability registration should succeed");

    engine
        .mark_ready()
        .expect("engine should become Ready after required registration");

    engine.serve().expect("engine should become Serving");

    assert_eq!(engine.state(), LifecycleState::Serving);

    (engine, registry, capability_id)
}

/// Build a genuine Core UniversalRequest targeting the supplied KG instance.
pub fn universal_request(
    sender: EngineId,
    target: EngineId,
    target_instance: EngineInstanceId,
    capability_id: CapabilityId,
    payload: &[u8],
    operation_name: &str,
) -> UniversalRequest {
    let payload_descriptor =
        PayloadDescriptor::new("application/octet-stream", Version::new(1, 0, 0))
            .expect("payload descriptor must be valid");

    let descriptor = ContractDescriptor::new(
        ContractId::new(format!("nizaam.kg.tests.{operation_name}.contract"))
            .expect("contract id must be valid"),
        capability_id,
        Version::new(1, 0, 0),
        Interaction::Request,
        payload_descriptor.clone(),
    );

    let metadata = ContractMetadata::new(
        descriptor,
        Participants::new(sender, target).with_target_instance(target_instance),
    );

    let envelope = MessageEnvelope::new(
        MessageId::new(format!("nizaam.kg.tests.{operation_name}.message"))
            .expect("message id must be valid"),
        operation_context(operation_name),
        metadata,
        EncodedPayload::new(payload_descriptor, payload.to_vec()),
    );

    UniversalRequest::new(envelope)
}
