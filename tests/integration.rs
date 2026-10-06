//! Level 3 end-to-end Phase 0 integration tests.
//!
//! The central scenario intentionally follows the frozen Phase 0 path:
//! construction -> Core startup -> engine registration -> capability
//! registration -> Ready -> Serving -> UniversalRequest -> Core admission ->
//! capability dispatch -> handler result -> Draining -> Stopped.

mod common;

use common::helpers::{
    instance_id, operation_context, registering_engine, serving_engine, universal_request,
};

use nizaam_core::capability::CapabilityDispatchResult;
use nizaam_core::identity::{CapabilityId, EngineId};
use nizaam_core::runtime::LifecycleState;

#[test]
fn phase0_normal_flow_uses_the_complete_core_backed_engine_path() {
    let (mut engine, registry) = registering_engine("normal-flow");

    engine
        .register_engine(&registry)
        .expect("engine registration should succeed");

    let capability_id = engine
        .register_phase0_capability()
        .expect("Phase 0 capability registration should succeed");

    engine.mark_ready().expect("engine should become Ready");
    engine.serve().expect("engine should become Serving");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"phase0-integration",
        "normal-flow",
    );

    let result = engine
        .handle_request(&request)
        .expect("Serving engine should admit the request");

    match result {
        CapabilityDispatchResult::Outcome(outcome) => {
            assert_eq!(outcome.as_bytes(), b"phase0-integration");
        }
        CapabilityDispatchResult::Error(error) => {
            panic!("unexpected capability error: {error:?}");
        }
    }

    engine.drain().expect("engine should enter Draining");
    assert_eq!(engine.state(), LifecycleState::Draining);

    engine.shutdown().expect("engine should stop cleanly");
    assert_eq!(engine.state(), LifecycleState::Stopped);
}

#[test]
fn request_to_a_non_serving_engine_is_rejected_before_capability_execution() {
    let (engine, registry, capability_id) = serving_engine("admission");

    engine.drain().expect("engine should enter Draining");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"must-not-run",
        "admission",
    );

    assert!(engine.handle_request(&request).is_err());

    engine.shutdown().expect("engine should stop cleanly");
    assert!(registry.contains(engine.engine_instance_id()));
}

#[test]
fn local_dispatch_rejects_a_mismatched_logical_target() {
    let (engine, _, capability_id) = serving_engine("target-engine");

    let wrong_target =
        EngineId::new("nizaam.kg.tests.other-engine").expect("target engine id must be valid");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        wrong_target,
        engine.engine_instance_id().clone(),
        capability_id,
        b"wrong-target",
        "target-engine-mismatch",
    );

    let result = engine.handle_request(&request);

    assert!(result.is_err());
}

#[test]
fn local_dispatch_rejects_a_mismatched_instance_target() {
    let (engine, _, capability_id) = serving_engine("target-instance");

    let wrong_instance = instance_id("another-instance");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        wrong_instance,
        capability_id,
        b"wrong-instance",
        "target-instance-mismatch",
    );

    let result = engine.handle_request(&request);

    assert!(result.is_err());
}

#[test]
fn unknown_capability_is_resolved_by_core_and_reported_as_a_dispatch_error() {
    let (engine, _, _) = serving_engine("unknown-capability");

    let unknown_capability = CapabilityId::new("nizaam.kg.tests.unknown-capability")
        .expect("capability id must be valid");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        unknown_capability,
        b"unknown",
        "unknown-capability",
    );

    let result = engine
        .handle_request(&request)
        .expect("lifecycle admission should succeed while serving");

    assert!(matches!(result, CapabilityDispatchResult::Error(_)));
}

#[test]
fn universal_request_operation_context_is_preserved_by_the_request_contract() {
    let (engine, _, capability_id) = serving_engine("context-preservation");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"context",
        "context-preservation",
    );

    let operation = &request.universal_event().envelope.operation_context;

    assert_eq!(
        operation.operation.id.as_str(),
        "nizaam.kg.tests.context-preservation.operation"
    );
    assert_eq!(
        operation.operation.correlation_id.as_str(),
        "nizaam.kg.tests.context-preservation.correlation"
    );
}

#[test]
fn core_engine_identity_and_concrete_instance_identity_remain_distinct() {
    let (engine, _, _) = serving_engine("identity-distinction");

    assert_ne!(
        engine.engine_id().as_str(),
        engine.engine_instance_id().as_str()
    );
}

#[test]
fn repeated_phase0_registration_does_not_create_a_second_engine_membership() {
    let (mut engine, registry) = registering_engine("single-membership");

    engine
        .register_engine(&registry)
        .expect("engine registration should succeed");

    assert_eq!(registry.len(), 1);

    assert!(engine.register_engine(&registry).is_err());
    assert_eq!(registry.len(), 1);
}

#[test]
fn shutdown_is_core_lifecycle_shutdown_not_a_second_kg_shutdown_system() {
    let (engine, _, _) = serving_engine("core-shutdown");

    engine.drain().expect("engine should drain");
    engine.shutdown().expect("engine should stop");

    assert_eq!(engine.state(), LifecycleState::Stopped);

    let operation = operation_context("post-shutdown");
    assert_eq!(
        operation.operation.id.as_str(),
        "nizaam.kg.tests.post-shutdown.operation"
    );
}
