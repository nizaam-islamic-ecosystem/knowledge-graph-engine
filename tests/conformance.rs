//! Level 3 architectural conformance tests for KG Phase 0.
//!
//! These tests verify observable architectural properties rather than private
//! implementation details. They protect the boundary described by the Phase 0
//! scope: KG adapts to Core and does not recreate Core.

mod common;

use common::helpers::{
    engine_id, instance_id, registering_engine, serving_engine, universal_request,
};

use nizaam_core::capability::{CapabilityDefinition, CapabilityDispatchResult};
use nizaam_core::identity::{CapabilityId, EngineId};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_core::runtime::{EngineContext, LifecycleState};

#[test]
fn kg_engine_starts_with_core_created_lifecycle_state() {
    let engine = common::helpers::new_engine("core-lifecycle");

    assert_eq!(engine.state(), LifecycleState::Created);
}

#[test]
fn kg_engine_uses_core_engine_identity_types_without_replacing_them() {
    let engine = common::helpers::new_engine("core-identities");

    let _: &nizaam_core::identity::EngineId = engine.engine_id();
    let _: &nizaam_core::identity::EngineInstanceId = engine.engine_instance_id();

    assert_ne!(
        engine.engine_id().as_str(),
        engine.engine_instance_id().as_str()
    );
}

#[test]
fn kg_registration_is_stored_in_the_core_control_plane_registry() {
    let (mut engine, registry) = registering_engine("core-control-plane");

    engine
        .register_engine(&registry)
        .expect("Core registration should succeed");

    assert_eq!(registry.len(), 1);
    assert!(registry.contains(engine.engine_instance_id()));
}

#[test]
fn kg_phase0_capability_registration_uses_a_core_capability_definition() {
    let (mut engine, _) = registering_engine("core-capability-definition");

    let capability_id = engine
        .register_phase0_capability()
        .expect("Phase 0 capability registration should succeed");

    let _: CapabilityId = capability_id;
}

#[test]
fn capability_owner_identity_is_the_same_logical_engine_identity_used_by_registration() {
    let (mut engine, _) = registering_engine("capability-owner");

    let capability = CapabilityDefinition::new(
        CapabilityId::new("nizaam.kg.tests.owner-check").expect("capability id must be valid"),
        engine.engine_id().clone(),
        "owner-check capability",
    )
    .expect("capability definition should be valid");

    let owner = capability.owning_engine().clone();

    engine
        .register_capability(
            capability,
            nizaam_core::capability::arc_handler(|_context, invocation| {
                Ok(nizaam_core::capability::CapabilityOutcome::new(
                    invocation.payload_bytes().to_vec(),
                ))
            }),
        )
        .expect("owned capability should register");

    assert_eq!(owner, *engine.engine_id());
}

#[test]
fn foreign_capability_ownership_is_rejected_without_creating_a_second_engine_identity() {
    let (mut engine, registry) = registering_engine("foreign-owner");

    let foreign_engine =
        EngineId::new("nizaam.kg.tests.foreign").expect("foreign engine id must be valid");

    let capability = CapabilityDefinition::new(
        CapabilityId::new("nizaam.kg.tests.foreign.capability")
            .expect("capability id must be valid"),
        foreign_engine.clone(),
        "foreign capability",
    )
    .expect("capability definition should be valid");

    assert!(
        engine
            .register_capability(
                capability,
                nizaam_core::capability::arc_handler(|_context, invocation| {
                    Ok(nizaam_core::capability::CapabilityOutcome::new(
                        invocation.payload_bytes().to_vec(),
                    ))
                })
            )
            .is_err()
    );

    assert_eq!(registry.len(), 0);
    assert_ne!(engine.engine_id(), &foreign_engine);
}

#[test]
fn core_universal_request_is_the_request_boundary() {
    let (engine, _, capability_id) = serving_engine("universal-request-boundary");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"opaque",
        "universal-request-boundary",
    );

    let _: &nizaam_core::contracts::UniversalRequest = &request;
    assert!(request.has_request_interaction());
}

#[test]
fn core_operation_context_is_the_request_execution_identity() {
    let (engine, _, capability_id) = serving_engine("operation-context");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"context",
        "operation-context",
    );

    let _: &OperationContext = &request.universal_event().envelope.operation_context;
}

#[test]
fn core_engine_context_is_constructible_from_the_core_operation_context() {
    let operation = Operation::new(
        nizaam_core::identity::OperationId::new("nizaam.kg.tests.context.operation")
            .expect("operation id must be valid"),
        nizaam_core::identity::CorrelationId::new("nizaam.kg.tests.context.correlation")
            .expect("correlation id must be valid"),
    );

    let context = EngineContext::new(OperationContext::new(operation));

    assert_eq!(
        context.operation().operation.id.as_str(),
        "nizaam.kg.tests.context.operation"
    );
}

#[test]
fn phase0_dispatch_returns_core_capability_dispatch_results() {
    let (engine, _, capability_id) = serving_engine("dispatch-result");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"core-result",
        "dispatch-result",
    );

    let result = engine
        .handle_request(&request)
        .expect("serving request should be admitted");

    let _: CapabilityDispatchResult = result;
}

#[test]
fn draining_stops_new_normal_request_admission() {
    let (engine, _, capability_id) = serving_engine("draining-admission");

    engine.drain().expect("engine should enter Draining");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"rejected",
        "draining-admission",
    );

    assert!(engine.handle_request(&request).is_err());
}

#[test]
fn shutdown_reaches_core_stopped_state() {
    let (engine, _, _) = serving_engine("stopped-state");

    engine.drain().expect("engine should enter Draining");
    engine.shutdown().expect("Core shutdown should succeed");

    assert_eq!(engine.state(), LifecycleState::Stopped);
}

#[test]
fn phase0_contains_no_graph_or_domain_execution_requirement() {
    // The executable assertion is intentionally architectural: the only
    // Phase 0 capability exercised by this suite is the opaque probe. No graph,
    // search, traversal, storage, ingestion, ontology, or reasoning API is
    // required to construct or serve the engine.
    let (engine, _, capability_id) = serving_engine("no-domain-functionality");

    let request = universal_request(
        EngineId::new("nizaam.kg.tests.caller").expect("caller id must be valid"),
        engine.engine_id().clone(),
        engine.engine_instance_id().clone(),
        capability_id,
        b"opaque-only",
        "no-domain-functionality",
    );

    match engine
        .handle_request(&request)
        .expect("Phase 0 serving request should be admitted")
    {
        CapabilityDispatchResult::Outcome(outcome) => {
            assert_eq!(outcome.as_bytes(), b"opaque-only");
        }
        CapabilityDispatchResult::Error(error) => {
            panic!("unexpected Phase 0 capability error: {error:?}");
        }
    }
}

#[test]
fn control_plane_registry_is_not_replaced_by_a_kg_registry() {
    let (mut engine, registry) = registering_engine("single-registry");

    engine
        .register_engine(&registry)
        .expect("Core registry registration should succeed");

    assert_eq!(registry.len(), 1);
    assert!(registry.contains(engine.engine_instance_id()));
}

#[test]
fn phase0_engine_and_instance_identities_are_stable_across_lifecycle() {
    let (engine, _, _) = serving_engine("stable-identities");

    assert_eq!(engine.engine_id(), &engine_id("stable-identities"));
    assert_eq!(
        engine.engine_instance_id(),
        &instance_id("stable-identities")
    );
}
