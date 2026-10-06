//! Level 3 repository tests for the Phase 0 KG engine shell.
//!
//! These tests exercise the coherent engine facade and Core-owned lifecycle/
//! registration boundaries through the library API. They intentionally do not
//! reach into KG implementation modules.

pub mod common;

use common::helpers::{engine_id, instance_id, new_engine, registering_engine, serving_engine};

use nizaam_core::control_plane::registry::EngineRegistry;
use nizaam_core::identity::{CapabilityId, EngineId};
use nizaam_core::runtime::LifecycleState;

#[test]
fn engine_construction_preserves_logical_and_instance_identity() {
    let engine = new_engine("construction");

    assert_eq!(engine.engine_id(), &engine_id("construction"));
    assert_eq!(engine.engine_instance_id(), &instance_id("construction"));
    assert_eq!(engine.state(), LifecycleState::Created);
    assert!(!engine.is_registered());
}

#[test]
fn engine_start_and_registration_transition_are_core_owned() {
    let engine = new_engine("startup");

    engine.start().expect("startup should succeed");
    assert_ne!(engine.state(), LifecycleState::Created);

    engine
        .begin_registration()
        .expect("engine should enter Core Registering state");

    assert_eq!(engine.state(), LifecycleState::Registering);
}

#[test]
fn engine_registration_is_rejected_before_registering() {
    let mut engine = new_engine("registration-before-state");
    let registry = EngineRegistry::new();

    assert!(engine.register_engine(&registry).is_err());
}

#[test]
fn phase0_capability_registration_is_rejected_before_registering() {
    let mut engine = new_engine("capability-before-state");

    assert!(engine.register_phase0_capability().is_err());
}

#[test]
fn ready_requires_engine_registration() {
    let engine = new_engine("ready-without-engine-registration");

    assert!(engine.mark_ready().is_err());
}

#[test]
fn ready_requires_phase0_capability_registration() {
    let (mut engine, registry) = registering_engine("ready-without-capability");

    engine
        .register_engine(&registry)
        .expect("engine registration should succeed");

    assert!(engine.mark_ready().is_err());
}

#[test]
fn engine_registration_is_visible_in_the_core_control_plane() {
    let (mut engine, registry) = registering_engine("control-plane-registration");

    engine
        .register_engine(&registry)
        .expect("engine registration should succeed");

    assert!(registry.contains(engine.engine_instance_id()));
    assert_eq!(registry.len(), 1);

    let record = registry
        .get(engine.engine_instance_id())
        .expect("registered instance should be present");

    assert_eq!(record.engine_id(), engine.engine_id());
    assert_eq!(record.engine_instance_id(), engine.engine_instance_id());
}

#[test]
fn phase0_capability_registration_is_separate_from_engine_registration() {
    let (mut engine, registry) = registering_engine("separate-registration");

    engine
        .register_engine(&registry)
        .expect("engine registration should succeed");

    assert!(!engine.is_registered());

    let capability_id = engine
        .register_phase0_capability()
        .expect("Phase 0 capability should register");

    assert!(!capability_id.as_str().is_empty());
    assert!(engine.is_registered());
}

#[test]
fn complete_registration_allows_ready_and_serving() {
    let (engine, registry, capability_id) = serving_engine("serving");

    assert!(registry.contains(engine.engine_instance_id()));
    assert!(engine.is_registered());
    assert!(!capability_id.as_str().is_empty());
    assert_eq!(engine.state(), LifecycleState::Serving);
}

#[test]
fn capability_registration_rejects_a_foreign_owner() {
    let (mut engine, _) = registering_engine("foreign-capability");

    let foreign_engine =
        EngineId::new("nizaam.kg.tests.foreign-owner").expect("foreign engine id must be valid");
    let capability_id = CapabilityId::new("nizaam.kg.tests.foreign-capability")
        .expect("capability id must be valid");

    let definition = nizaam_core::capability::CapabilityDefinition::new(
        capability_id,
        foreign_engine,
        "foreign-owner-test capability",
    )
    .expect("capability definition should be valid");

    let result = engine.register_capability(
        definition,
        nizaam_core::capability::arc_handler(|_context, invocation| {
            Ok(nizaam_core::capability::CapabilityOutcome::new(
                invocation.payload_bytes().to_vec(),
            ))
        }),
    );

    assert!(result.is_err());
}

#[test]
fn serving_engine_can_drain_and_stop_through_core() {
    let (engine, _, _) = serving_engine("shutdown");

    engine.drain().expect("draining should succeed");
    assert_eq!(engine.state(), LifecycleState::Draining);

    engine.shutdown().expect("shutdown should succeed");
    assert_eq!(engine.state(), LifecycleState::Stopped);
}
