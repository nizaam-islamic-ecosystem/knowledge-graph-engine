//! Level 3 architectural conformance tests for KG Phase 0 and Phase 1.
//!
//! These tests verify observable architectural properties rather than private
//! implementation details. They protect the boundary described by the Phase 0
//! scope: KG adapts to Core and does not recreate Core, while Phase 1 keeps
//! the KG semantic foundation within its defined identity/object boundaries.

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

#[test]
fn phase1_kg_identity_layer_exposes_all_required_strong_types() {
    use nizaam_knowledge_graph::identity::{
        ConceptId, EntityId, KnowledgeAssertionId, LexicalFormId, MentionId, ReferenceId, SourceId,
    };
    use std::any::TypeId;

    let _: EntityId = EntityId::generate();
    let _: ConceptId = ConceptId::generate();
    let _: SourceId = SourceId::generate();
    let _: ReferenceId = ReferenceId::generate();
    let _: LexicalFormId = LexicalFormId::generate();
    let _: MentionId = MentionId::generate();
    let _: KnowledgeAssertionId = KnowledgeAssertionId::generate();

    let ids = [
        TypeId::of::<EntityId>(),
        TypeId::of::<ConceptId>(),
        TypeId::of::<SourceId>(),
        TypeId::of::<ReferenceId>(),
        TypeId::of::<LexicalFormId>(),
        TypeId::of::<MentionId>(),
        TypeId::of::<KnowledgeAssertionId>(),
    ];

    for (index, left) in ids.iter().enumerate() {
        for right in ids.iter().skip(index + 1) {
            assert_ne!(left, right);
        }
    }
}

#[test]
fn phase1_entity_concept_source_reference_and_mention_are_distinct_objects() {
    use nizaam_knowledge_graph::{Concept, Entity, Mention, Reference, Source};
    use std::any::TypeId;

    let object_types = [
        TypeId::of::<Entity>(),
        TypeId::of::<Concept>(),
        TypeId::of::<Source>(),
        TypeId::of::<Reference>(),
        TypeId::of::<Mention>(),
    ];

    for (index, left) in object_types.iter().enumerate() {
        for right in object_types.iter().skip(index + 1) {
            assert_ne!(left, right);
        }
    }
}

#[test]
fn phase1_names_and_aliases_remain_entity_representation_data() {
    use nizaam_knowledge_graph::{
        entity::{Alias, Entity, Name},
        identity::EntityId,
    };

    let entity = Entity::new(
        EntityId::generate(),
        Name::new("الله", "ar"),
        vec![Alias::new("Allah", "en"), Alias::new("اللہ", "ur")],
    );

    assert_eq!(entity.name().value(), "الله");
    assert_eq!(entity.name().language(), "ar");
    assert_eq!(entity.aliases()[0].value(), "Allah");
    assert_eq!(entity.aliases()[0].language(), "en");
    assert_eq!(entity.aliases()[1].value(), "اللہ");
    assert_eq!(entity.aliases()[1].language(), "ur");
}

#[test]
fn phase1_concept_and_lexical_identity_remain_separate_boundaries() {
    use nizaam_knowledge_graph::identity::{ConceptId, LexicalFormId};
    use std::any::TypeId;

    let concept_id = ConceptId::generate();
    let lexical_id = LexicalFormId::generate();

    assert!(!concept_id.as_str().is_empty());
    assert!(!lexical_id.as_str().is_empty());
    assert_ne!(TypeId::of::<ConceptId>(), TypeId::of::<LexicalFormId>());
}

#[test]
fn phase1_source_and_reference_remain_separate_generic_boundaries() {
    use nizaam_knowledge_graph::{
        identity::{ReferenceId, SourceId},
        source::{Reference, Source},
    };

    let source = Source::new(SourceId::generate(), "Example source");
    let reference = Reference::new(ReferenceId::generate(), "opaque-reference-value");

    assert_eq!(source.label(), "Example source");
    assert_eq!(reference.value(), "opaque-reference-value");
}

#[test]
fn phase1_index_assigned_id_remains_an_external_indexing_identity() {
    use nizaam_indexing::identity::IndexAssignedId;
    use nizaam_knowledge_graph::identity::{ConceptId, EntityId, MentionId, ReferenceId, SourceId};
    use std::any::TypeId;

    // The KG consumes Indexing's assigned-object identity concept; it does
    // not replace that identity with one of its semantic identity types.
    assert_ne!(TypeId::of::<IndexAssignedId>(), TypeId::of::<EntityId>());
    assert_ne!(TypeId::of::<IndexAssignedId>(), TypeId::of::<ConceptId>());
    assert_ne!(TypeId::of::<IndexAssignedId>(), TypeId::of::<SourceId>());
    assert_ne!(TypeId::of::<IndexAssignedId>(), TypeId::of::<ReferenceId>());
    assert_ne!(TypeId::of::<IndexAssignedId>(), TypeId::of::<MentionId>());
}
