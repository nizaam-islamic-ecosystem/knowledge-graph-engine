//! Level 3 architectural conformance tests for KG Phase 0 through Phase 2.
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

// -----------------------------------------------------------------------------
// Phase 2 semantic, relationship, and graph boundaries
// -----------------------------------------------------------------------------

#[test]
fn phase2_knowledge_assertion_is_a_first_class_public_semantic_object() {
    use nizaam_knowledge_graph::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifier, Qualifiers,
    };
    use nizaam_knowledge_graph::identity::{ConceptId, EntityId};

    let mut context = AssertionContext::new();
    context
        .insert("source", "quran")
        .expect("valid assertion context");
    let qualifier = Qualifier::new("scope", "primary").expect("valid assertion qualifier");

    let assertion = KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
        AssertionPredicate::new("has-name").expect("valid relationship predicate"),
        AssertionObject::Concept(ConceptId::new("concept-1").expect("valid concept identity")),
        context,
        Qualifiers::from_iter([qualifier]),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    assert!(assertion.validate().is_ok());
    assert!(!assertion.id().as_str().is_empty());
}

#[test]
fn phase2_knowledge_assertion_identity_is_deterministic_and_status_independent() {
    use nizaam_knowledge_graph::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use nizaam_knowledge_graph::identity::EntityId;

    fn make(status: AssertionStatus) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            status,
            AssertionPolarity::Positive,
        )
    }

    let accepted = make(AssertionStatus::Accepted);
    let provisional = make(AssertionStatus::Provisional);

    assert_eq!(accepted.id(), provisional.id());
    assert_eq!(accepted, provisional);
}

#[test]
fn phase2_assertion_polarity_is_semantically_identity_relevant() {
    use nizaam_knowledge_graph::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use nizaam_knowledge_graph::identity::EntityId;

    fn make(polarity: AssertionPolarity) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new("has-name").expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            polarity,
        )
    }

    assert_ne!(
        make(AssertionPolarity::Positive).id(),
        make(AssertionPolarity::Negative).id()
    );
}

#[test]
fn phase2_typed_assertion_objects_remain_distinct_semantic_types() {
    use nizaam_knowledge_graph::assertion::AssertionObject;
    use nizaam_knowledge_graph::identity::{ConceptId, EntityId};

    let entity =
        AssertionObject::Entity(EntityId::new("shared-value").expect("valid entity identity"));
    let concept =
        AssertionObject::Concept(ConceptId::new("shared-value").expect("valid concept identity"));

    assert_ne!(entity, concept);
}

#[test]
fn phase2_relationship_definition_exposes_structural_semantics() {
    use nizaam_knowledge_graph::relationship::{
        Relationship, RelationshipCharacteristics, RelationshipDirection, RelationshipFamily,
        RelationshipPredicate,
    };

    let relationship = Relationship::new(
        RelationshipPredicate::new("has-name").expect("valid relationship predicate"),
        RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid relationship family"),
        RelationshipDirection::SubjectToObject,
        RelationshipCharacteristics::new(),
    );

    assert_eq!(
        relationship.direction(),
        RelationshipDirection::SubjectToObject
    );
    assert_eq!(relationship.family().as_str(), RelationshipFamily::SEMANTIC);
}

#[test]
fn phase2_inverse_view_reuses_one_canonical_edge_and_assertion() {
    use nizaam_knowledge_graph::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use nizaam_knowledge_graph::graph::{Graph, TraversalDirection, traverse};
    use nizaam_knowledge_graph::identity::EntityId;
    use nizaam_knowledge_graph::relationship::{
        Relationship, RelationshipCharacteristics, RelationshipDirection, RelationshipFamily,
        RelationshipPredicate,
    };

    let assertion = KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
        AssertionPredicate::new("has-name").expect("valid relationship predicate"),
        AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    let mut graph = Graph::new();
    let edge_id = graph
        .add_assertion(&assertion)
        .expect("assertion should be added");
    let edge = graph.edge(&edge_id).expect("edge should exist");

    let relationship = Relationship::new(
        RelationshipPredicate::new("has-name").expect("valid relationship predicate"),
        RelationshipFamily::new(RelationshipFamily::SEMANTIC).expect("valid relationship family"),
        RelationshipDirection::SubjectToObject,
        RelationshipCharacteristics::new(),
    )
    .with_inverse_predicate(RelationshipPredicate::new("name-of").expect("valid inverse predicate"))
    .expect("valid inverse relationship");

    let forward = traverse(edge, &assertion, &relationship, TraversalDirection::Forward)
        .expect("forward traversal should succeed");
    let inverse = traverse(edge, &assertion, &relationship, TraversalDirection::Inverse)
        .expect("inverse traversal should succeed");

    assert_eq!(forward.assertion_id(), inverse.assertion_id());
    assert_eq!(forward.edge_id(), inverse.edge_id());
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(inverse.predicate().as_str(), "kg.relationship.name-of");
}

#[test]
fn phase2_graph_supports_multiple_assertions_between_the_same_nodes() {
    use nizaam_knowledge_graph::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use nizaam_knowledge_graph::graph::Graph;
    use nizaam_knowledge_graph::identity::EntityId;

    fn make(predicate: &str) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
            AssertionPredicate::new(predicate).expect("valid relationship predicate"),
            AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    let first = make("has-name");
    let second = make("aliases");
    let mut graph = Graph::new();

    graph
        .add_assertion(&first)
        .expect("first assertion should be added");
    graph
        .add_assertion(&second)
        .expect("second assertion should be added");

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn phase2_graph_edge_is_structural_and_references_the_canonical_assertion() {
    use nizaam_knowledge_graph::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use nizaam_knowledge_graph::graph::Graph;
    use nizaam_knowledge_graph::identity::EntityId;

    let assertion = KnowledgeAssertion::new(
        AssertionObject::Entity(EntityId::new("entity-1").expect("valid entity identity")),
        AssertionPredicate::new("has-name").expect("valid relationship predicate"),
        AssertionObject::Entity(EntityId::new("entity-2").expect("valid entity identity")),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    );

    let mut graph = Graph::new();
    let edge_id = graph
        .add_assertion(&assertion)
        .expect("assertion should be added");
    let edge = graph.edge(&edge_id).expect("edge should exist");

    assert_eq!(edge.assertion_id(), assertion.id());
    assert_ne!(edge.id().as_str(), assertion.id().as_str());
}

#[test]
fn phase5_ingestion_and_indexing_contracts_are_public_without_duplicate_identity_types() {
    use std::any::TypeId;

    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::IngestionPipeline>(),
        TypeId::of::<nizaam_knowledge_graph::ingestion::IngestionPipeline>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::ExternalIdentifier>(),
        TypeId::of::<nizaam_knowledge_graph::entity::ExternalIdentifier>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::ExternalIdentifier>(),
        TypeId::of::<nizaam_knowledge_graph::resolution::ExternalIdentifier>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::IndexingReadinessReceipt>(),
        TypeId::of::<nizaam_knowledge_graph::integration::indexing::IndexingReadinessReceipt>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::IndexAssignedId>(),
        TypeId::of::<nizaam_indexing::IndexAssignedId>(),
    );
}

#[test]
fn phase6_query_graph_and_index_contracts_are_public_at_the_crate_boundary() {
    use std::any::TypeId;

    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::TraversalPath>(),
        TypeId::of::<nizaam_knowledge_graph::graph::TraversalPath>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::TraversalBounds>(),
        TypeId::of::<nizaam_knowledge_graph::graph::TraversalBounds>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::IndexStateObservation>(),
        TypeId::of::<nizaam_knowledge_graph::index::IndexStateObservation>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::QueryRequest>(),
        TypeId::of::<nizaam_knowledge_graph::query::QueryRequest>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::QueryPlan>(),
        TypeId::of::<nizaam_knowledge_graph::query::QueryPlan>(),
    );
    assert_eq!(
        TypeId::of::<nizaam_knowledge_graph::QueryResult>(),
        TypeId::of::<nizaam_knowledge_graph::query::QueryResult>(),
    );

    let request = nizaam_knowledge_graph::QueryRequest::Lookup(
        nizaam_knowledge_graph::LookupRequest::object(
            nizaam_knowledge_graph::assertion::AssertionObject::Entity(
                nizaam_knowledge_graph::identity::EntityId::new("phase6-boundary")
                    .expect("valid entity id"),
            ),
        ),
    );
    let planned = nizaam_knowledge_graph::plan(&request).expect("public planner should be usable");
    assert_eq!(planned.kind(), nizaam_knowledge_graph::QueryKind::Lookup);
}
