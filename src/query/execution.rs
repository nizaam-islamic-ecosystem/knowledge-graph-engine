//! Logical query execution against KG access abstractions.
//!
//! Execution consumes a validated plan and delegates semantic data access to a
//! caller-supplied `QueryAccess` implementation. This keeps the query layer
//! storage-independent while allowing graph, repository, and Indexing adapters
//! to provide the actual data.

use core::fmt;

use nizaam_core::operation::OperationContext;

use super::plan::{IndexAccessRequirement, QueryOperator, QueryPlan};
use super::ranking::{DeterministicRankingProvider, RankedCandidate, RankingProvider};
use super::request::{QueryOrdering, ReasoningProfile};
use super::result::{
    QueryExplanation, QueryMatchCandidate, QueryResult, QueryResultItem, traversal_path_key,
};

/// Semantic data-access boundary consumed by the query executor.
///
/// Implementations may combine the in-memory graph, canonical repositories,
/// semantic registries, and the KG index-access layer. The query module never
/// requires a physical database implementation.
pub trait QueryAccess {
    /// Adapter-specific access failure.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Performs direct semantic lookup.
    fn lookup(
        &self,
        target: &super::request::LookupTarget,
        context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error>;

    /// Performs bounded graph traversal for the supplied logical traversal plan.
    ///
    /// Implementations must enforce every field of the plan's [`super::request::TraversalBudget`],
    /// including node, edge, result, and expansion limits, rather than treating
    /// the graph-layer bounds as the complete query budget.
    fn traverse(
        &self,
        plan: &super::plan::TraversalPlan,
        context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error>;

    /// Performs exact/lexical/conceptual/relational/semantic retrieval.
    fn retrieve(
        &self,
        plan: &super::plan::RetrievalPlan,
        context: &OperationContext,
    ) -> Result<Vec<QueryMatchCandidate>, Self::Error>;
}

/// Executes a plan using the default deterministic KG ranking policy.
pub fn execute<A>(
    access: &A,
    plan: &QueryPlan,
    context: &OperationContext,
) -> Result<QueryResult, QueryExecutionError<A::Error>>
where
    A: QueryAccess,
{
    let ranking = DeterministicRankingProvider;
    execute_with_ranking(access, plan, context, &ranking)
}

/// Executes a plan with a caller-supplied KG-owned ranking provider.
pub fn execute_with_ranking<A, P>(
    access: &A,
    plan: &QueryPlan,
    context: &OperationContext,
    ranking_provider: &P,
) -> Result<QueryResult, QueryExecutionError<A::Error>>
where
    A: QueryAccess,
    P: RankingProvider,
{
    plan.validate().map_err(QueryExecutionError::InvalidPlan)?;

    if plan.reasoning() != ReasoningProfile::None {
        return Err(QueryExecutionError::ReasoningUnavailable {
            profile: plan.reasoning(),
        });
    }

    let candidates = collect_candidates(access, plan.operator(), context)?;
    let filtered = candidates
        .into_iter()
        .filter(|candidate| visible(candidate, plan.visibility()))
        .filter(|candidate| plan.filter().matches(candidate))
        .collect::<Vec<_>>();

    let mut ranked = filtered
        .into_iter()
        .map(|candidate| RankedCandidate {
            ranking: ranking_provider.rank(&plan.ranking(), &candidate),
            candidate,
        })
        .collect::<Vec<_>>();

    sort_ranked(&mut ranked, plan.ordering());

    let total_matches = ranked.len();
    let mut truncated_by_budget = false;

    if let QueryOperator::Traverse(traversal) = plan.operator()
        && ranked.len() > traversal.budget().max_results()
    {
        ranked.truncate(traversal.budget().max_results());
        truncated_by_budget = true;
    }

    let start = if let Some(cursor) = plan.pagination().cursor() {
        let position = ranked
            .iter()
            .enumerate()
            .position(|(position, candidate)| {
                QueryResultItem::from_candidate(
                    candidate.candidate.clone(),
                    candidate.ranking.clone(),
                )
                .cursor_token_with_position(position)
                    == cursor
            })
            .ok_or_else(|| QueryExecutionError::InvalidCursor(cursor.to_owned()))?;
        position.saturating_add(1)
    } else {
        plan.pagination().offset().unwrap_or(0)
    };

    if start > ranked.len() {
        return Err(QueryExecutionError::InvalidPaginationPosition {
            start,
            len: ranked.len(),
        });
    }

    let end = start
        .saturating_add(plan.pagination().limit())
        .min(ranked.len());
    let page = ranked[start..end].to_vec();
    let has_more = end < ranked.len();

    let next_cursor = has_more.then(|| {
        let last = page
            .last()
            .expect("a non-empty page is required when another page exists");
        QueryResultItem::from_candidate(last.candidate.clone(), last.ranking.clone())
            .cursor_token_with_position(start + page.len() - 1)
    });

    let items = page
        .into_iter()
        .map(|ranked| QueryResultItem::from_candidate(ranked.candidate, ranked.ranking))
        .collect::<Vec<_>>();

    let explanation = QueryExplanation::new(
        format!("executed {} query", plan.kind()),
        matches!(plan.operator(), QueryOperator::Retrieve(retrieval) if retrieval.expansion() == super::request::SemanticExpansion::Controlled),
        false,
        plan.index_requirement() != IndexAccessRequirement::Optional,
        plan.ranking().name(),
    )
    .map_err(QueryExecutionError::Explanation)?;

    let pagination = super::result::PaginationMetadata::new(
        total_matches,
        items.len(),
        plan.pagination().limit(),
        plan.pagination().offset(),
        plan.pagination().cursor().is_some(),
        next_cursor,
        truncated_by_budget,
    );

    Ok(QueryResult::new(items, pagination, explanation))
}

fn collect_candidates<A>(
    access: &A,
    operator: &QueryOperator,
    context: &OperationContext,
) -> Result<Vec<QueryMatchCandidate>, QueryExecutionError<A::Error>>
where
    A: QueryAccess,
{
    match operator {
        QueryOperator::Lookup(plan) => access
            .lookup(plan.target(), context)
            .map_err(QueryExecutionError::Access),
        QueryOperator::Traverse(plan) => access
            .traverse(plan, context)
            .map_err(QueryExecutionError::Access),
        QueryOperator::Retrieve(plan) => access
            .retrieve(plan, context)
            .map_err(QueryExecutionError::Access),
        QueryOperator::Composite(children) => {
            let mut candidates = Vec::new();
            for child in children {
                candidates.extend(collect_candidates(access, child, context)?);
            }
            Ok(candidates)
        }
        QueryOperator::Filter { .. }
        | QueryOperator::Rank { .. }
        | QueryOperator::Paginate { .. } => Err(QueryExecutionError::UnsupportedOperator),
    }
}

fn visible(candidate: &QueryMatchCandidate, profile: &super::request::VisibilityProfile) -> bool {
    if let Some(required) = profile.required_publication()
        && candidate.publication() != Some(required)
    {
        return false;
    }

    if !profile.include_inferred()
        && candidate.inference_status() == super::result::InferenceStatus::Inferred
    {
        return false;
    }

    if !profile.include_machine_generated()
        && candidate.inference_status() == super::result::InferenceStatus::MachineGeneratedCandidate
    {
        return false;
    }

    if !profile.include_disputed()
        && candidate.status() == Some(crate::assertion::AssertionStatus::Disputed)
    {
        return false;
    }

    if !profile.allowed_statuses().is_empty()
        && candidate
            .status()
            .is_none_or(|status| !profile.allowed_statuses().contains(&status))
    {
        return false;
    }

    true
}

fn sort_ranked(ranked: &mut [RankedCandidate], ordering: QueryOrdering) {
    ranked.sort_by(|left, right| match ordering {
        QueryOrdering::RelevanceDescending => right
            .ranking
            .score()
            .cmp(&left.ranking.score())
            .then_with(|| {
                left.candidate
                    .reference()
                    .stable_key()
                    .cmp(&right.candidate.reference().stable_key())
            })
            .then_with(|| {
                traversal_path_key(left.candidate.path())
                    .cmp(&traversal_path_key(right.candidate.path()))
            }),
        QueryOrdering::PathLengthAscending => left
            .candidate
            .path()
            .map_or(0, |path| path.len())
            .cmp(&right.candidate.path().map_or(0, |path| path.len()))
            .then_with(|| right.ranking.score().cmp(&left.ranking.score()))
            .then_with(|| {
                left.candidate
                    .reference()
                    .stable_key()
                    .cmp(&right.candidate.reference().stable_key())
            })
            .then_with(|| {
                traversal_path_key(left.candidate.path())
                    .cmp(&traversal_path_key(right.candidate.path()))
            }),
        QueryOrdering::ReferenceAscending => left
            .candidate
            .reference()
            .stable_key()
            .cmp(&right.candidate.reference().stable_key())
            .then_with(|| right.ranking.score().cmp(&left.ranking.score()))
            .then_with(|| {
                traversal_path_key(left.candidate.path())
                    .cmp(&traversal_path_key(right.candidate.path()))
            }),
    });
}

/// Typed query-execution failures.
#[derive(Debug)]
pub enum QueryExecutionError<E> {
    /// The logical plan was structurally invalid.
    InvalidPlan(super::plan::PlanValidationError),
    /// The access adapter failed.
    Access(E),
    /// A requested Phase 8 reasoning profile cannot execute in Phase 6.
    ReasoningUnavailable { profile: ReasoningProfile },
    /// The supplied opaque cursor did not identify an item in this result set.
    InvalidCursor(String),
    /// The requested offset/cursor position is outside the candidate page.
    InvalidPaginationPosition { start: usize, len: usize },
    /// The general AST contains an operator whose execution semantics are not
    /// implemented by this initial Phase 6 executor.
    UnsupportedOperator,
    /// Result explanation metadata could not be constructed.
    Explanation(super::result::ResultExplanationError),
}

impl<E> fmt::Display for QueryExecutionError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlan(error) => write!(formatter, "invalid query plan: {error}"),
            Self::Access(error) => write!(formatter, "query access failed: {error}"),
            Self::ReasoningUnavailable { profile } => {
                write!(
                    formatter,
                    "reasoning profile {profile:?} requires Phase 8 execution"
                )
            }
            Self::InvalidCursor(cursor) => write!(formatter, "invalid query cursor: {cursor}"),
            Self::InvalidPaginationPosition { start, len } => write!(
                formatter,
                "query pagination position {start} is outside candidate range of length {len}"
            ),
            Self::UnsupportedOperator => formatter
                .write_str("query operator is not executable in the initial Phase 6 executor"),
            Self::Explanation(error) => write!(formatter, "query explanation failed: {error}"),
        }
    }
}

impl<E> std::error::Error for QueryExecutionError<E> where E: std::error::Error + 'static {}

#[cfg(test)]
mod tests {
    use super::{QueryAccess, QueryExecutionError, execute};
    use crate::identity::KnowledgeAssertionId;
    use crate::ingestion::PublicationOutcome;
    use crate::query::{
        InferenceStatus, QueryMatchCandidate, QueryMatchType, QueryReference, QueryRequest,
        RetrievalRequest, plan,
    };
    use nizaam_core::{
        identity::{CorrelationId, OperationId},
        operation::{Operation, OperationContext},
    };

    #[derive(Debug)]
    struct AccessError;

    impl std::fmt::Display for AccessError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("test access failure")
        }
    }

    impl std::error::Error for AccessError {}

    struct FakeAccess {
        candidates: Vec<QueryMatchCandidate>,
    }

    impl Default for FakeAccess {
        fn default() -> Self {
            Self {
                candidates: vec![
                    QueryMatchCandidate::new(
                        QueryReference::Assertion(
                            KnowledgeAssertionId::new("assertion-execution-a").unwrap(),
                        ),
                        QueryMatchType::Lexical,
                    )
                    .with_publication(PublicationOutcome::Published)
                    .with_inference_status(InferenceStatus::Observed),
                    QueryMatchCandidate::new(
                        QueryReference::Assertion(
                            KnowledgeAssertionId::new("assertion-execution-b").unwrap(),
                        ),
                        QueryMatchType::Lexical,
                    )
                    .with_publication(PublicationOutcome::Published)
                    .with_inference_status(InferenceStatus::Observed),
                ],
            }
        }
    }

    impl QueryAccess for FakeAccess {
        type Error = AccessError;

        fn lookup(
            &self,
            _target: &crate::query::LookupTarget,
            _context: &OperationContext,
        ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
            Ok(Vec::new())
        }

        fn traverse(
            &self,
            _plan: &crate::query::TraversalPlan,
            _context: &OperationContext,
        ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
            Ok(Vec::new())
        }

        fn retrieve(
            &self,
            _plan: &crate::query::RetrievalPlan,
            _context: &OperationContext,
        ) -> Result<Vec<QueryMatchCandidate>, Self::Error> {
            Ok(self.candidates.clone())
        }
    }

    fn context() -> OperationContext {
        OperationContext::new(Operation::new(
            OperationId::new("nizaam.kg.query.execution.test.operation").unwrap(),
            CorrelationId::new("nizaam.kg.query.execution.test.correlation").unwrap(),
        ))
    }

    #[test]
    fn executor_returns_reference_oriented_explainable_results() {
        let request = QueryRequest::Retrieval(RetrievalRequest::lexical("sabr"));
        let plan = plan(&request).expect("retrieval should plan");
        let result =
            execute(&FakeAccess::default(), &plan, &context()).expect("execution should succeed");

        assert_eq!(result.items().len(), 2);
        assert_eq!(
            result.items()[0].reference().stable_key(),
            "assertion:assertion-execution-a"
        );
        assert!(result.items()[0].ranking().score() > 0);
        assert_eq!(result.pagination().returned(), 2);
    }

    #[test]
    fn reasoning_is_explicitly_deferred_to_phase8() {
        let options =
            crate::query::QueryOptions::new().with_reasoning(crate::query::ReasoningProfile::Basic);
        let request =
            QueryRequest::Retrieval(RetrievalRequest::lexical("sabr").with_options(options));
        let plan = plan(&request).expect("planning should preserve reasoning intent");
        let error = execute(&FakeAccess::default(), &plan, &context())
            .expect_err("phase 6 must not silently execute phase 8 reasoning");

        assert!(matches!(
            error,
            QueryExecutionError::ReasoningUnavailable {
                profile: crate::query::ReasoningProfile::Basic
            }
        ));
    }

    #[test]
    fn cursor_pagination_distinguishes_duplicate_references_with_different_paths() {
        use crate::assertion::{
            AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate,
            AssertionStatus, KnowledgeAssertion, Qualifiers,
        };
        use crate::graph::{GraphEdge, GraphNode, TraversalDirection, TraversalPath, traverse};
        use crate::relationship::{
            Relationship, RelationshipCharacteristics, RelationshipDirection, RelationshipFamily,
            RelationshipPredicate,
        };

        let make_candidate = |source: &str| {
            let assertion = KnowledgeAssertion::new(
                AssertionObject::Entity(crate::identity::EntityId::new(source).unwrap()),
                AssertionPredicate::new("knows").unwrap(),
                AssertionObject::Entity(crate::identity::EntityId::new("shared-result").unwrap()),
                AssertionContext::new(),
                Qualifiers::new(),
                AssertionStatus::Accepted,
                AssertionPolarity::Positive,
            );
            let source_node = GraphNode::new(assertion.subject().clone());
            let target_node = GraphNode::new(assertion.object().clone());
            let edge = GraphEdge::new(&assertion, &source_node, &target_node).unwrap();
            let relationship = Relationship::new(
                RelationshipPredicate::new("knows").unwrap(),
                RelationshipFamily::new(RelationshipFamily::SEMANTIC).unwrap(),
                RelationshipDirection::SubjectToObject,
                RelationshipCharacteristics::new(),
            );
            let step = traverse(
                &edge,
                &assertion,
                &relationship,
                TraversalDirection::Forward,
            )
            .unwrap();
            QueryMatchCandidate::new(
                QueryReference::Object(assertion.object().clone()),
                QueryMatchType::Traversal,
            )
            .with_publication(PublicationOutcome::Published)
            .with_path(TraversalPath::from_steps(step.source().clone(), [step]).unwrap())
        };

        let candidates = vec![make_candidate("path-a"), make_candidate("path-b")];
        let access = FakeAccess { candidates };
        let options =
            crate::query::QueryOptions::new().with_pagination(crate::query::Pagination::new(1));
        let request = QueryRequest::Retrieval(
            RetrievalRequest::semantic(crate::query::RetrievalTarget::Text("paths".to_owned()))
                .with_options(options),
        );
        let first_plan = plan(&request).expect("first page should plan");
        let first = execute(&access, &first_plan, &context()).expect("first page should execute");
        let cursor = first
            .next_cursor()
            .expect("first page should expose a cursor")
            .to_owned();

        let second_options = crate::query::QueryOptions::new()
            .with_pagination(crate::query::Pagination::new(1).with_cursor(cursor));
        let second_request = QueryRequest::Retrieval(
            RetrievalRequest::semantic(crate::query::RetrievalTarget::Text("paths".to_owned()))
                .with_options(second_options),
        );
        let second_plan = plan(&second_request).expect("second page should plan");
        let second =
            execute(&access, &second_plan, &context()).expect("second page should execute");

        assert_eq!(first.items()[0].reference(), second.items()[0].reference());
        assert_ne!(
            first.items()[0].path().unwrap(),
            second.items()[0].path().unwrap(),
        );
        assert!(second.next_cursor().is_none());
    }

    #[test]
    fn cursor_pagination_is_deterministic() {
        let first_options =
            crate::query::QueryOptions::new().with_pagination(crate::query::Pagination::new(1));
        let first_request =
            QueryRequest::Retrieval(RetrievalRequest::lexical("sabr").with_options(first_options));
        let first_plan = plan(&first_request).expect("first page should plan");
        let first = execute(&FakeAccess::default(), &first_plan, &context())
            .expect("first page should execute");
        let cursor = first
            .next_cursor()
            .expect("first page should expose a cursor")
            .to_owned();

        let second_options = crate::query::QueryOptions::new()
            .with_pagination(crate::query::Pagination::new(1).with_cursor(cursor));
        let second_request =
            QueryRequest::Retrieval(RetrievalRequest::lexical("sabr").with_options(second_options));
        let second_plan = plan(&second_request).expect("second page should plan");
        let second = execute(&FakeAccess::default(), &second_plan, &context())
            .expect("second page should execute");

        assert_eq!(
            first.items()[0].reference().stable_key(),
            "assertion:assertion-execution-a"
        );
        assert_eq!(
            second.items()[0].reference().stable_key(),
            "assertion:assertion-execution-b"
        );
        assert!(second.next_cursor().is_none());
    }
}
