//! Logical search/retrieval adapter over Nizaam Indexing.
//!
//! Nizaam Indexing already owns query planning, provider capability negotiation,
//! physical retrieval, and the canonical reference-oriented `QueryResult`. The
//! KG layer therefore delegates retrieval instead of implementing a second
//! search engine.

use nizaam_core::operation::OperationContext;
use nizaam_indexing::query::planner::RetrievalPlan;
use nizaam_indexing::query::result::QueryResult;
use nizaam_indexing::query::retrieval::{self, ProviderRetriever, RetrievalError};

/// Thin logical search adapter over an Indexing provider retriever.
pub trait IndexSearchAccess {
    /// Provider/retrieval failure type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Executes an already-planned logical retrieval using the supplied Core context.
    fn search(
        &self,
        plan: &RetrievalPlan,
        context: &OperationContext,
    ) -> Result<QueryResult, Self::Error>;
}

/// Blanket adapter for every Nizaam Indexing provider retriever.
///
/// This implementation deliberately calls Indexing's canonical retrieval
/// function and passes the caller's Core `OperationContext` unchanged.
impl<P> IndexSearchAccess for P
where
    P: ProviderRetriever,
{
    type Error = RetrievalError<P::Error>;

    fn search(
        &self,
        plan: &RetrievalPlan,
        context: &OperationContext,
    ) -> Result<QueryResult, Self::Error> {
        retrieval::execute(plan, self, context)
    }
}

/// Executes a planned logical search through the supplied adapter.
pub fn search<A>(
    adapter: &A,
    plan: &RetrievalPlan,
    context: &OperationContext,
) -> Result<QueryResult, A::Error>
where
    A: IndexSearchAccess,
{
    adapter.search(plan, context)
}
