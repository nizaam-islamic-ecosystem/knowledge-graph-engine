//! Rich, explainable query results.
//!
//! Query results are views over already-existing KG knowledge. They preserve
//! references and metadata needed for explanation without hydrating or creating
//! new semantic objects.

use std::collections::BTreeSet;
use std::fmt;

use crate::assertion::{AssertionContext, AssertionObject, AssertionStatus};
use crate::authority::AuthorityDimension;
use crate::identity::{ActivityId, EvidenceId, KnowledgeAssertionId, SourceId};
use crate::ingestion::{PublicationOutcome, SourceAuthenticity};
use crate::semantics::SemanticType;
use crate::temporal::TemporalValidity;

use super::ranking::RankingMetadata;

/// Reference-oriented identity returned by a query.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryReference {
    /// A semantic object reference.
    Object(AssertionObject),
    /// A canonical knowledge assertion reference.
    Assertion(KnowledgeAssertionId),
}

impl QueryReference {
    /// Returns a deterministic, type-qualified key suitable for tie-breaking.
    #[must_use]
    pub fn stable_key(&self) -> String {
        match self {
            Self::Object(AssertionObject::Entity(id)) => format!("entity:{}", id.as_str()),
            Self::Object(AssertionObject::Concept(id)) => format!("concept:{}", id.as_str()),
            Self::Object(AssertionObject::Source(id)) => format!("source:{}", id.as_str()),
            Self::Object(AssertionObject::Reference(id)) => {
                format!("reference:{}", id.as_str())
            }
            Self::Object(AssertionObject::LexicalForm(id)) => {
                format!("lexical-form:{}", id.as_str())
            }
            Self::Object(AssertionObject::Mention(id)) => format!("mention:{}", id.as_str()),
            Self::Assertion(id) => format!("assertion:{}", id.as_str()),
        }
    }
}

/// Kind of match that produced a query candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryMatchType {
    /// Direct identity lookup.
    Lookup,
    /// Exact retrieval match.
    Exact,
    /// Lexical/text retrieval match.
    Lexical,
    /// Conceptual retrieval match.
    Conceptual,
    /// Relationship-oriented retrieval match.
    Relational,
    /// Controlled semantic retrieval match.
    Semantic,
    /// Known graph traversal match.
    Traversal,
}

impl fmt::Display for QueryMatchType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lookup => "lookup",
            Self::Exact => "exact",
            Self::Lexical => "lexical",
            Self::Conceptual => "conceptual",
            Self::Relational => "relational",
            Self::Semantic => "semantic",
            Self::Traversal => "traversal",
        })
    }
}

/// Distinguishes knowledge origin in a result without conflating it with
/// assertion status, confidence, authority, or ranking.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InferenceStatus {
    /// Directly observed/sourced knowledge.
    Observed,
    /// Human/curator-approved canonical knowledge.
    Curated,
    /// Knowledge derived by a controlled inference engine.
    Inferred,
    /// Machine-generated candidate knowledge that remains distinguishable from
    /// canonical truth.
    MachineGeneratedCandidate,
}

/// Explanation metadata attached to one query match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchExplanation {
    summary: String,
    matched_terms: Vec<String>,
}

impl MatchExplanation {
    /// Creates a concise explanation summary.
    pub fn new(summary: impl Into<String>) -> Result<Self, ResultExplanationError> {
        let summary = summary.into();
        validate_text(&summary, "match explanation summary")?;
        Ok(Self {
            summary,
            matched_terms: Vec::new(),
        })
    }

    /// Adds one matched term or semantic label.
    pub fn with_matched_term(
        mut self,
        term: impl Into<String>,
    ) -> Result<Self, ResultExplanationError> {
        let term = term.into();
        validate_text(&term, "matched term")?;
        self.matched_terms.push(term);
        Ok(self)
    }

    /// Returns the explanation summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the matched terms in stable insertion order.
    #[must_use]
    pub fn matched_terms(&self) -> &[String] {
        &self.matched_terms
    }
}

/// A factual candidate returned by a KG access implementation before ranking,
/// pagination, and final result shaping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryMatchCandidate {
    reference: QueryReference,
    match_type: QueryMatchType,
    path: Option<crate::graph::TraversalPath>,
    subject: Option<AssertionObject>,
    object: Option<AssertionObject>,
    relationship_predicate: Option<crate::relationship::RelationshipPredicate>,
    source_ids: BTreeSet<SourceId>,
    evidence_ids: BTreeSet<EvidenceId>,
    provenance_activity_ids: BTreeSet<ActivityId>,
    authority_dimensions: BTreeSet<AuthorityDimension>,
    semantic_types: BTreeSet<SemanticType>,
    status: Option<AssertionStatus>,
    publication: Option<PublicationOutcome>,
    source_authenticity: Option<SourceAuthenticity>,
    validity: Option<TemporalValidity>,
    context: Option<AssertionContext>,
    inference_status: InferenceStatus,
    explanation: Option<MatchExplanation>,
}

impl QueryMatchCandidate {
    /// Creates a reference-oriented query candidate.
    #[must_use]
    pub fn new(reference: QueryReference, match_type: QueryMatchType) -> Self {
        Self {
            reference,
            match_type,
            path: None,
            subject: None,
            object: None,
            relationship_predicate: None,
            source_ids: BTreeSet::new(),
            evidence_ids: BTreeSet::new(),
            provenance_activity_ids: BTreeSet::new(),
            authority_dimensions: BTreeSet::new(),
            semantic_types: BTreeSet::new(),
            status: None,
            publication: None,
            source_authenticity: None,
            validity: None,
            context: None,
            inference_status: InferenceStatus::Observed,
            explanation: None,
        }
    }

    /// Adds a matched traversal path.
    #[must_use]
    pub fn with_path(mut self, path: crate::graph::TraversalPath) -> Self {
        self.path = Some(path);
        self
    }

    /// Adds assertion subject/object references for typed entity filters.
    #[must_use]
    pub fn with_assertion_ends(
        mut self,
        subject: AssertionObject,
        object: AssertionObject,
    ) -> Self {
        self.subject = Some(subject);
        self.object = Some(object);
        self
    }

    /// Adds the matched relationship predicate.
    #[must_use]
    pub fn with_relationship_predicate(
        mut self,
        predicate: crate::relationship::RelationshipPredicate,
    ) -> Self {
        self.relationship_predicate = Some(predicate);
        self
    }

    /// Adds source references.
    #[must_use]
    pub fn with_sources<I>(mut self, source_ids: I) -> Self
    where
        I: IntoIterator<Item = SourceId>,
    {
        self.source_ids.extend(source_ids);
        self
    }

    /// Adds evidence references.
    #[must_use]
    pub fn with_evidence<I>(mut self, evidence_ids: I) -> Self
    where
        I: IntoIterator<Item = EvidenceId>,
    {
        self.evidence_ids.extend(evidence_ids);
        self
    }

    /// Adds provenance activity references.
    #[must_use]
    pub fn with_provenance<I>(mut self, activity_ids: I) -> Self
    where
        I: IntoIterator<Item = ActivityId>,
    {
        self.provenance_activity_ids.extend(activity_ids);
        self
    }

    /// Adds authority dimensions.
    #[must_use]
    pub fn with_authority<I>(mut self, dimensions: I) -> Self
    where
        I: IntoIterator<Item = AuthorityDimension>,
    {
        self.authority_dimensions.extend(dimensions);
        self
    }

    /// Adds semantic types.
    #[must_use]
    pub fn with_semantic_types<I>(mut self, semantic_types: I) -> Self
    where
        I: IntoIterator<Item = SemanticType>,
    {
        self.semantic_types.extend(semantic_types);
        self
    }

    /// Adds current assertion status.
    #[must_use]
    pub const fn with_status(mut self, status: AssertionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Adds governed publication outcome.
    #[must_use]
    pub const fn with_publication(mut self, publication: PublicationOutcome) -> Self {
        self.publication = Some(publication);
        self
    }

    /// Adds source authenticity metadata.
    #[must_use]
    pub const fn with_source_authenticity(mut self, value: SourceAuthenticity) -> Self {
        self.source_authenticity = Some(value);
        self
    }

    /// Adds valid-time metadata.
    #[must_use]
    pub const fn with_validity(mut self, validity: TemporalValidity) -> Self {
        self.validity = Some(validity);
        self
    }

    /// Adds assertion context.
    #[must_use]
    pub fn with_context(mut self, context: AssertionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Sets the knowledge-origin classification.
    #[must_use]
    pub const fn with_inference_status(mut self, status: InferenceStatus) -> Self {
        self.inference_status = status;
        self
    }

    /// Adds explainability metadata.
    #[must_use]
    pub fn with_explanation(mut self, explanation: MatchExplanation) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Returns the result reference.
    #[must_use]
    pub fn reference(&self) -> &QueryReference {
        &self.reference
    }

    /// Returns the match kind.
    #[must_use]
    pub const fn match_type(&self) -> QueryMatchType {
        self.match_type
    }

    /// Returns the matched path, if any.
    #[must_use]
    pub fn path(&self) -> Option<&crate::graph::TraversalPath> {
        self.path.as_ref()
    }

    /// Returns the assertion subject, if available.
    #[must_use]
    pub fn subject(&self) -> Option<&AssertionObject> {
        self.subject.as_ref()
    }

    /// Returns the assertion object, if available.
    #[must_use]
    pub fn object(&self) -> Option<&AssertionObject> {
        self.object.as_ref()
    }

    /// Returns the relationship predicate, if available.
    #[must_use]
    pub fn relationship_predicate(&self) -> Option<&crate::relationship::RelationshipPredicate> {
        self.relationship_predicate.as_ref()
    }

    /// Returns source references.
    #[must_use]
    pub fn source_ids(&self) -> &BTreeSet<SourceId> {
        &self.source_ids
    }

    /// Returns evidence references.
    #[must_use]
    pub fn evidence_ids(&self) -> &BTreeSet<EvidenceId> {
        &self.evidence_ids
    }

    /// Returns provenance activity references.
    #[must_use]
    pub fn provenance_activity_ids(&self) -> &BTreeSet<ActivityId> {
        &self.provenance_activity_ids
    }

    /// Returns authority dimensions.
    #[must_use]
    pub fn authority_dimensions(&self) -> &BTreeSet<AuthorityDimension> {
        &self.authority_dimensions
    }

    /// Returns semantic types.
    #[must_use]
    pub fn semantic_types(&self) -> &BTreeSet<SemanticType> {
        &self.semantic_types
    }

    /// Returns current assertion status.
    #[must_use]
    pub const fn status(&self) -> Option<AssertionStatus> {
        self.status
    }

    /// Returns publication outcome.
    #[must_use]
    pub const fn publication(&self) -> Option<PublicationOutcome> {
        self.publication
    }

    /// Returns source authenticity metadata.
    #[must_use]
    pub const fn source_authenticity(&self) -> Option<SourceAuthenticity> {
        self.source_authenticity
    }

    /// Returns valid-time metadata.
    #[must_use]
    pub const fn validity(&self) -> Option<TemporalValidity> {
        self.validity
    }

    /// Returns assertion context metadata.
    #[must_use]
    pub fn context(&self) -> Option<&AssertionContext> {
        self.context.as_ref()
    }

    /// Returns the knowledge-origin classification.
    #[must_use]
    pub const fn inference_status(&self) -> InferenceStatus {
        self.inference_status
    }

    /// Returns explanation metadata.
    #[must_use]
    pub fn explanation(&self) -> Option<&MatchExplanation> {
        self.explanation.as_ref()
    }
}

/// One final explainable query result item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryResultItem {
    reference: QueryReference,
    match_type: QueryMatchType,
    path: Option<crate::graph::TraversalPath>,
    ranking: RankingMetadata,
    source_ids: BTreeSet<SourceId>,
    evidence_ids: BTreeSet<EvidenceId>,
    provenance_activity_ids: BTreeSet<ActivityId>,
    authority_dimensions: BTreeSet<AuthorityDimension>,
    status: Option<AssertionStatus>,
    publication: Option<PublicationOutcome>,
    source_authenticity: Option<SourceAuthenticity>,
    inference_status: InferenceStatus,
    explanation: Option<MatchExplanation>,
}

impl QueryResultItem {
    /// Creates a result item from a ranked candidate.
    #[must_use]
    pub fn from_candidate(candidate: QueryMatchCandidate, ranking: RankingMetadata) -> Self {
        Self {
            reference: candidate.reference,
            match_type: candidate.match_type,
            path: candidate.path,
            ranking,
            source_ids: candidate.source_ids,
            evidence_ids: candidate.evidence_ids,
            provenance_activity_ids: candidate.provenance_activity_ids,
            authority_dimensions: candidate.authority_dimensions,
            status: candidate.status,
            publication: candidate.publication,
            source_authenticity: candidate.source_authenticity,
            inference_status: candidate.inference_status,
            explanation: candidate.explanation,
        }
    }

    /// Returns the result reference.
    #[must_use]
    pub fn reference(&self) -> &QueryReference {
        &self.reference
    }

    /// Returns the match kind.
    #[must_use]
    pub const fn match_type(&self) -> QueryMatchType {
        self.match_type
    }

    /// Returns the matched path, if any.
    #[must_use]
    pub fn path(&self) -> Option<&crate::graph::TraversalPath> {
        self.path.as_ref()
    }

    /// Returns ranking metadata.
    #[must_use]
    pub const fn ranking(&self) -> &RankingMetadata {
        &self.ranking
    }

    /// Returns source references.
    #[must_use]
    pub fn source_ids(&self) -> &BTreeSet<SourceId> {
        &self.source_ids
    }

    /// Returns evidence references.
    #[must_use]
    pub fn evidence_ids(&self) -> &BTreeSet<EvidenceId> {
        &self.evidence_ids
    }

    /// Returns provenance activity references.
    #[must_use]
    pub fn provenance_activity_ids(&self) -> &BTreeSet<ActivityId> {
        &self.provenance_activity_ids
    }

    /// Returns authority dimensions.
    #[must_use]
    pub fn authority_dimensions(&self) -> &BTreeSet<AuthorityDimension> {
        &self.authority_dimensions
    }

    /// Returns current assertion status.
    #[must_use]
    pub const fn status(&self) -> Option<AssertionStatus> {
        self.status
    }

    /// Returns publication outcome.
    #[must_use]
    pub const fn publication(&self) -> Option<PublicationOutcome> {
        self.publication
    }

    /// Returns source authenticity metadata.
    #[must_use]
    pub const fn source_authenticity(&self) -> Option<SourceAuthenticity> {
        self.source_authenticity
    }

    /// Returns the knowledge-origin classification.
    #[must_use]
    pub const fn inference_status(&self) -> InferenceStatus {
        self.inference_status
    }

    /// Returns the result explanation, if any.
    #[must_use]
    pub fn explanation(&self) -> Option<&MatchExplanation> {
        self.explanation.as_ref()
    }

    /// Creates the opaque continuation token for this deterministic ordering.
    #[must_use]
    pub fn cursor_token(&self) -> String {
        format!(
            "score={};reference={}",
            self.ranking.score(),
            self.reference.stable_key()
        )
    }
}

/// Pagination metadata returned with every query result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaginationMetadata {
    total_matches: usize,
    returned: usize,
    limit: usize,
    offset: Option<usize>,
    cursor_used: bool,
    next_cursor: Option<String>,
    truncated_by_budget: bool,
}

impl PaginationMetadata {
    /// Creates pagination metadata for one result page.
    #[must_use]
    pub const fn new(
        total_matches: usize,
        returned: usize,
        limit: usize,
        offset: Option<usize>,
        cursor_used: bool,
        next_cursor: Option<String>,
        truncated_by_budget: bool,
    ) -> Self {
        Self {
            total_matches,
            returned,
            limit,
            offset,
            cursor_used,
            next_cursor,
            truncated_by_budget,
        }
    }

    /// Returns the number of matches after filtering and before pagination.
    #[must_use]
    pub const fn total_matches(&self) -> usize {
        self.total_matches
    }

    /// Returns the number of returned items.
    #[must_use]
    pub const fn returned(&self) -> usize {
        self.returned
    }

    /// Returns the requested page limit.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Returns the offset, if offset pagination was used.
    #[must_use]
    pub const fn offset(&self) -> Option<usize> {
        self.offset
    }

    /// Returns whether a continuation cursor was consumed.
    #[must_use]
    pub const fn cursor_used(&self) -> bool {
        self.cursor_used
    }

    /// Returns the next continuation cursor, if another page exists.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    /// Returns whether the execution budget truncated candidate work.
    #[must_use]
    pub const fn truncated_by_budget(&self) -> bool {
        self.truncated_by_budget
    }
}

/// Top-level explainability metadata for a query execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryExplanation {
    summary: String,
    semantic_expansion: bool,
    reasoning_requested: bool,
    index_acceleration_expected: bool,
    ranking_profile: String,
}

impl QueryExplanation {
    /// Creates top-level explanation metadata.
    pub fn new(
        summary: impl Into<String>,
        semantic_expansion: bool,
        reasoning_requested: bool,
        index_acceleration_expected: bool,
        ranking_profile: impl Into<String>,
    ) -> Result<Self, ResultExplanationError> {
        let summary = summary.into();
        let ranking_profile = ranking_profile.into();
        validate_text(&summary, "query explanation summary")?;
        validate_text(&ranking_profile, "query ranking profile")?;
        Ok(Self {
            summary,
            semantic_expansion,
            reasoning_requested,
            index_acceleration_expected,
            ranking_profile,
        })
    }

    /// Returns the execution summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns whether controlled semantic expansion participated.
    #[must_use]
    pub const fn semantic_expansion(&self) -> bool {
        self.semantic_expansion
    }

    /// Returns whether reasoning was requested.
    #[must_use]
    pub const fn reasoning_requested(&self) -> bool {
        self.reasoning_requested
    }

    /// Returns whether the plan expects index acceleration.
    #[must_use]
    pub const fn index_acceleration_expected(&self) -> bool {
        self.index_acceleration_expected
    }

    /// Returns the ranking profile label.
    #[must_use]
    pub fn ranking_profile(&self) -> &str {
        &self.ranking_profile
    }
}

/// Final reference-oriented Phase 6 query result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryResult {
    items: Vec<QueryResultItem>,
    pagination: PaginationMetadata,
    explanation: QueryExplanation,
}

impl QueryResult {
    /// Creates a final query result.
    #[must_use]
    pub fn new(
        items: Vec<QueryResultItem>,
        pagination: PaginationMetadata,
        explanation: QueryExplanation,
    ) -> Self {
        Self {
            items,
            pagination,
            explanation,
        }
    }

    /// Returns result items in deterministic execution order.
    #[must_use]
    pub fn items(&self) -> &[QueryResultItem] {
        &self.items
    }

    /// Returns pagination metadata.
    #[must_use]
    pub const fn pagination(&self) -> &PaginationMetadata {
        &self.pagination
    }

    /// Returns top-level explanation metadata.
    #[must_use]
    pub const fn explanation(&self) -> &QueryExplanation {
        &self.explanation
    }

    /// Returns the next page cursor, if available.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.pagination.next_cursor()
    }
}

/// Structural errors for explainability metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultExplanationError {
    /// A textual explanation field was empty or whitespace-only.
    EmptyText,
}

impl fmt::Display for ResultExplanationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText => formatter.write_str("query explanation text must not be empty"),
        }
    }
}

impl std::error::Error for ResultExplanationError {}

fn validate_text(value: &str, _field: &'static str) -> Result<(), ResultExplanationError> {
    if value.trim().is_empty() {
        return Err(ResultExplanationError::EmptyText);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{InferenceStatus, QueryMatchCandidate, QueryMatchType, QueryReference};
    use crate::identity::KnowledgeAssertionId;

    #[test]
    fn query_reference_stable_keys_are_type_qualified() {
        let reference = QueryReference::Assertion(
            KnowledgeAssertionId::new("assertion-result").expect("valid assertion"),
        );
        assert_eq!(reference.stable_key(), "assertion:assertion-result");
    }

    #[test]
    fn result_candidate_preserves_inference_status_as_distinct_metadata() {
        let candidate = QueryMatchCandidate::new(
            QueryReference::Assertion(
                KnowledgeAssertionId::new("assertion-result-status").unwrap(),
            ),
            QueryMatchType::Semantic,
        )
        .with_inference_status(InferenceStatus::Inferred);

        assert_eq!(candidate.inference_status(), InferenceStatus::Inferred);
        assert_ne!(
            candidate.status(),
            Some(crate::assertion::AssertionStatus::Known)
        );
    }
}
