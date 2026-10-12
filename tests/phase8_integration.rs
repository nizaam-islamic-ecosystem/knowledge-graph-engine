//! Level 3 Phase 8 cross-phase integration tests.
//!
//! These tests verify the Phase 6 profile/delegation boundary and Phase 7
//! snapshot/version context consumed by the Phase 8 reasoner. They deliberately
//! keep canonical persistence outside the reasoning engine.

use std::cell::Cell;

use nizaam_core::identity::{CorrelationId, OperationId};
use nizaam_core::operation::{Operation, OperationContext};
use nizaam_knowledge_graph::assertion::{
    AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
    KnowledgeAssertion, Qualifiers,
};
use nizaam_knowledge_graph::identity::{EntityId, RuleId};
use nizaam_knowledge_graph::ingestion::PublicationOutcome;
use nizaam_knowledge_graph::query::{
    InferenceStatus, LookupRequest, QueryAccess, QueryExecutionError, QueryMatchCandidate,
    QueryMatchType, QueryOptions, QueryReference, QueryRequest, ReasoningProfile, execute, plan,
};
use nizaam_knowledge_graph::reasoning::{
    Reasoner, ReasoningExecutionStatus, ReasoningInput, Rule, RuleKind, RuleMetadata, RulePattern,
    RuleRegistry, RuleScope, RuleTerm, RuleVersion,
};
use nizaam_knowledge_graph::storage::{Document, RepositoryRecord, Snapshot};
use nizaam_knowledge_graph::versioning::{ChangeTarget, KgVersion};

#[derive(Debug)]
struct AccessError;

impl std::fmt::Display for AccessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("phase8 integration access error")
    }
}

impl std::error::Error for AccessError {}

fn context() -> OperationContext {
    OperationContext::new(Operation::new(
        OperationId::new("nizaam.kg.phase8.integration.operation").expect("valid operation"),
        CorrelationId::new("nizaam.kg.phase8.integration.correlation").expect("valid correlation"),
    ))
}

fn entity(value: &str) -> AssertionObject {
    AssertionObject::Entity(EntityId::new(value).expect("valid entity"))
}

fn assertion(subject: &str, predicate: &str, object: &str) -> KnowledgeAssertion {
    KnowledgeAssertion::new(
        entity(subject),
        AssertionPredicate::new(predicate).expect("valid predicate"),
        entity(object),
        AssertionContext::new(),
        Qualifiers::new(),
        AssertionStatus::Accepted,
        AssertionPolarity::Positive,
    )
}

fn snapshot() -> Snapshot {
    Snapshot::new(
        KgVersion::initial(),
        vec![RepositoryRecord::new(
            ChangeTarget::Entity(EntityId::new("integration-root").expect("valid id")),
            Document::from_slice(b"integration-root"),
        )],
    )
    .expect("snapshot should be valid")
}

fn inverse_rule() -> Rule {
    Rule::new(
        RuleId::new("integration-inverse").expect("valid rule id"),
        RuleVersion::default(),
        RuleKind::Inverse,
        vec![RulePattern::new(
            RuleTerm::variable("x").expect("valid variable"),
            RuleTerm::text("parent-of").expect("valid predicate"),
            RuleTerm::variable("y").expect("valid variable"),
        )],
        Vec::new(),
        Vec::new(),
        RulePattern::new(
            RuleTerm::variable("y").expect("valid variable"),
            RuleTerm::text("child-of").expect("valid predicate"),
            RuleTerm::variable("x").expect("valid variable"),
        ),
        RuleScope::unrestricted(),
        RuleMetadata::new("integration inverse", "phase 6/7/8 integration rule").expect("metadata"),
    )
    .expect("rule should validate")
}

fn reasoning_result_candidate() -> QueryMatchCandidate {
    QueryMatchCandidate::new(
        QueryReference::Object(entity("derived-entity")),
        QueryMatchType::Semantic,
    )
    .with_publication(PublicationOutcome::Published)
    .with_inference_status(InferenceStatus::Inferred)
}

struct DelegatingAccess {
    candidates: Vec<QueryMatchCandidate>,
    reason_calls: Cell<usize>,
}

impl QueryAccess for DelegatingAccess {
    type Error = AccessError;

    fn lookup(
        &self,
        _target: &nizaam_knowledge_graph::query::LookupTarget,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        Ok(self.candidates.clone())
    }

    fn traverse(
        &self,
        _plan: &nizaam_knowledge_graph::query::TraversalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        Ok(self.candidates.clone())
    }

    fn retrieve(
        &self,
        _plan: &nizaam_knowledge_graph::query::RetrievalPlan,
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
        Ok(self.candidates.clone())
    }

    fn reason(
        &self,
        plan: &nizaam_knowledge_graph::query::QueryPlan,
        _base_candidates: &[QueryMatchCandidate],
        _context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, QueryExecutionError<Self::Error>> {
        self.reason_calls.set(self.reason_calls.get() + 1);
        assert_eq!(plan.reasoning(), ReasoningProfile::Basic);
        Ok(vec![reasoning_result_candidate()])
    }
}

#[test]
fn phase7_snapshot_is_pinned_as_the_reasoning_input_context() {
    let mut registry = RuleRegistry::new();
    let rule = inverse_rule();
    let id = rule.id().clone();
    registry.register_built_in(rule).expect("register");
    registry.approve(&id).expect("approve");
    registry
        .version(&id, RuleVersion::default())
        .expect("version");
    registry.activate(&id).expect("activate");

    let snapshot = snapshot();
    let source = assertion("a", "parent-of", "b");
    let assertions = vec![source];
    let input = ReasoningInput::new(&snapshot, &assertions, None);
    let result = Reasoner::new(registry)
        .reason(&input, ReasoningProfile::Basic)
        .expect("reasoning should execute");

    assert_eq!(result.profile(), ReasoningProfile::Basic);
    assert!(!result.derived().is_empty());
    assert!(matches!(
        result.status(),
        ReasoningExecutionStatus::Complete | ReasoningExecutionStatus::Incomplete
    ));
    let trace = result
        .derived()
        .values()
        .next()
        .and_then(|value| value.traces().first())
        .expect("derived result should contain a trace");
    assert_eq!(trace.input_version(), snapshot.version().id());
    assert_eq!(trace.input_snapshot(), snapshot.id().as_str());
    assert_eq!(trace.rule_id(), &id);
    assert_eq!(trace.rule_version(), RuleVersion::default());
}

#[test]
fn phase6_reasoning_profile_is_carried_to_the_access_delegation_boundary() {
    let request = QueryRequest::Lookup(
        LookupRequest::object(entity("query-goal"))
            .with_options(QueryOptions::new().with_reasoning(ReasoningProfile::Basic)),
    );
    let planned = plan(&request).expect("request should plan");
    assert_eq!(planned.reasoning(), ReasoningProfile::Basic);

    let access = DelegatingAccess {
        candidates: vec![
            QueryMatchCandidate::new(
                QueryReference::Object(entity("direct")),
                QueryMatchType::Lookup,
            )
            .with_publication(PublicationOutcome::Published),
        ],
        reason_calls: Cell::new(0),
    };

    let derived = access
        .reason(&planned, &[], &context())
        .expect("adapter should accept the delegated profile");
    assert_eq!(access.reason_calls.get(), 1);
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].inference_status(), InferenceStatus::Inferred);
}

#[test]
fn explicit_and_derived_results_remain_distinguishable_at_query_boundary() {
    let explicit = QueryMatchCandidate::new(
        QueryReference::Object(entity("explicit")),
        QueryMatchType::Exact,
    )
    .with_inference_status(InferenceStatus::Observed);
    let derived = reasoning_result_candidate();

    assert_ne!(explicit.inference_status(), derived.inference_status());
}

#[test]
fn reasoning_disabled_keeps_the_existing_non_reasoning_query_path_available() {
    let request = QueryRequest::Lookup(LookupRequest::object(entity("plain")));
    let planned = plan(&request).expect("plain query should plan");
    let access = DelegatingAccess {
        candidates: vec![
            QueryMatchCandidate::new(
                QueryReference::Object(entity("plain")),
                QueryMatchType::Lookup,
            )
            .with_publication(PublicationOutcome::Published),
        ],
        reason_calls: Cell::new(0),
    };

    let result = execute(&access, &planned, &context()).expect("plain query should execute");
    assert_eq!(result.items().len(), 1);
    assert_eq!(access.reason_calls.get(), 0);
    assert!(!result.explanation().reasoning_requested());
}
