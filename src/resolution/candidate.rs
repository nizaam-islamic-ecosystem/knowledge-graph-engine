//! Ephemeral entity-resolution candidates and deterministic candidate signals.
//!
//! Candidates are working representations only. They deliberately have no
//! `CandidateId` and cannot create or mutate canonical entities.

use std::collections::BTreeSet;
use std::fmt;

use crate::identity::{EntityId, MentionId, ReferenceId, SourceId};

use super::crosswalk::ExternalIdentifier;
use super::matching::{exact_match, normalized_match, transliteration_match};

/// The source-level object being resolved.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResolutionReference {
    /// A mention occurrence.
    Mention(MentionId),

    /// A generic reference object.
    Reference(ReferenceId),
}

/// Input presented to the Phase 3 deterministic resolution pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionInput {
    reference: ResolutionReference,
    text: String,
    identifier: Option<String>,
    language: Option<String>,
    source_id: Option<SourceId>,
    external_identifier: Option<ExternalIdentifier>,
    transliterations: BTreeSet<String>,
    lexical_keys: BTreeSet<String>,
    semantic_relationship_keys: BTreeSet<String>,
    context_keys: BTreeSet<String>,
    graph_neighborhood_keys: BTreeSet<String>,
}

impl ResolutionInput {
    /// Creates a resolution input from a mention/reference and its source-level
    /// representation.
    #[must_use]
    pub fn new(reference: ResolutionReference, text: impl Into<String>) -> Self {
        Self {
            reference,
            text: text.into(),
            identifier: None,
            language: None,
            source_id: None,
            external_identifier: None,
            transliterations: BTreeSet::new(),
            lexical_keys: BTreeSet::new(),
            semantic_relationship_keys: BTreeSet::new(),
            context_keys: BTreeSet::new(),
            graph_neighborhood_keys: BTreeSet::new(),
        }
    }

    /// Adds a source-provided identifier used by exact identifier matching.
    #[must_use]
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Adds a language marker without performing language-specific analysis.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Associates the input with a source identity.
    #[must_use]
    pub fn with_source(mut self, source_id: SourceId) -> Self {
        self.source_id = Some(source_id);
        self
    }

    /// Adds a source-owned external identifier.
    #[must_use]
    pub fn with_external_identifier(mut self, identifier: ExternalIdentifier) -> Self {
        self.external_identifier = Some(identifier);
        self
    }

    /// Adds a linguistically prepared transliteration supplied by another
    /// component such as the Arabic Engine.
    #[must_use]
    pub fn with_transliteration(mut self, transliteration: impl Into<String>) -> Self {
        self.transliterations.insert(transliteration.into());
        self
    }

    /// Adds an opaque lexical-mapping signal key.
    #[must_use]
    pub fn with_lexical_key(mut self, key: impl Into<String>) -> Self {
        self.lexical_keys.insert(key.into());
        self
    }

    /// Adds an opaque semantic-relationship signal key.
    #[must_use]
    pub fn with_semantic_relationship_key(mut self, key: impl Into<String>) -> Self {
        self.semantic_relationship_keys.insert(key.into());
        self
    }

    /// Adds an opaque context signal key.
    #[must_use]
    pub fn with_context_key(mut self, key: impl Into<String>) -> Self {
        self.context_keys.insert(key.into());
        self
    }

    /// Adds an opaque graph-neighborhood signal key produced by an external
    /// graph-assistance layer. Resolution itself does not traverse the graph.
    #[must_use]
    pub fn with_graph_neighborhood_key(mut self, key: impl Into<String>) -> Self {
        self.graph_neighborhood_keys.insert(key.into());
        self
    }

    /// Returns the source-level reference.
    #[must_use]
    pub fn reference(&self) -> &ResolutionReference {
        &self.reference
    }

    /// Returns the source representation exactly as supplied.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the optional source-level identifier.
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Returns the optional language marker.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Returns the optional source identity.
    #[must_use]
    pub fn source_id(&self) -> Option<&SourceId> {
        self.source_id.as_ref()
    }

    /// Returns the optional external identifier.
    #[must_use]
    pub fn external_identifier(&self) -> Option<&ExternalIdentifier> {
        self.external_identifier.as_ref()
    }

    /// Returns supplied transliteration forms.
    #[must_use]
    pub fn transliterations(&self) -> &BTreeSet<String> {
        &self.transliterations
    }

    /// Returns lexical mapping keys.
    #[must_use]
    pub fn lexical_keys(&self) -> &BTreeSet<String> {
        &self.lexical_keys
    }

    /// Returns semantic relationship keys.
    #[must_use]
    pub fn semantic_relationship_keys(&self) -> &BTreeSet<String> {
        &self.semantic_relationship_keys
    }

    /// Returns context signal keys.
    #[must_use]
    pub fn context_keys(&self) -> &BTreeSet<String> {
        &self.context_keys
    }

    /// Returns graph neighborhood signal keys.
    #[must_use]
    pub fn graph_neighborhood_keys(&self) -> &BTreeSet<String> {
        &self.graph_neighborhood_keys
    }
}

/// Ephemeral information about one known canonical entity used during
/// candidate generation. It is not a stored entity representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityCandidateProfile {
    entity_id: EntityId,
    names: Vec<String>,
    aliases: Vec<String>,
    identifiers: BTreeSet<String>,
    source_identifiers: BTreeSet<ExternalIdentifier>,
    transliterations: BTreeSet<String>,
    languages: BTreeSet<String>,
    lexical_keys: BTreeSet<String>,
    semantic_relationship_keys: BTreeSet<String>,
    context_keys: BTreeSet<String>,
    graph_neighborhood_keys: BTreeSet<String>,
}

impl EntityCandidateProfile {
    /// Creates a minimal candidate profile for one canonical entity.
    #[must_use]
    pub fn new(entity_id: EntityId) -> Self {
        let mut identifiers = BTreeSet::new();
        identifiers.insert(entity_id.as_str().to_owned());

        Self {
            entity_id,
            names: Vec::new(),
            aliases: Vec::new(),
            identifiers,
            source_identifiers: BTreeSet::new(),
            transliterations: BTreeSet::new(),
            languages: BTreeSet::new(),
            lexical_keys: BTreeSet::new(),
            semantic_relationship_keys: BTreeSet::new(),
            context_keys: BTreeSet::new(),
            graph_neighborhood_keys: BTreeSet::new(),
        }
    }

    /// Adds a canonical name/representation.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.names.push(name.into());
        self
    }

    /// Adds an exact alias representation.
    #[must_use]
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Adds an identifier value used for exact identifier matching.
    #[must_use]
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifiers.insert(identifier.into());
        self
    }

    /// Adds a source-owned external identifier.
    #[must_use]
    pub fn with_source_identifier(mut self, identifier: ExternalIdentifier) -> Self {
        self.source_identifiers.insert(identifier);
        self
    }

    /// Adds a supplied transliteration form.
    #[must_use]
    pub fn with_transliteration(mut self, transliteration: impl Into<String>) -> Self {
        self.transliterations.insert(transliteration.into());
        self
    }

    /// Adds a language marker.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.languages.insert(language.into());
        self
    }

    /// Adds an opaque lexical mapping key.
    #[must_use]
    pub fn with_lexical_key(mut self, key: impl Into<String>) -> Self {
        self.lexical_keys.insert(key.into());
        self
    }

    /// Adds an opaque semantic relationship key.
    #[must_use]
    pub fn with_semantic_relationship_key(mut self, key: impl Into<String>) -> Self {
        self.semantic_relationship_keys.insert(key.into());
        self
    }

    /// Adds an opaque context key.
    #[must_use]
    pub fn with_context_key(mut self, key: impl Into<String>) -> Self {
        self.context_keys.insert(key.into());
        self
    }

    /// Adds an opaque graph neighborhood key.
    #[must_use]
    pub fn with_graph_neighborhood_key(mut self, key: impl Into<String>) -> Self {
        self.graph_neighborhood_keys.insert(key.into());
        self
    }

    /// Returns the canonical entity identity.
    #[must_use]
    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    /// Returns candidate names.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Returns candidate aliases.
    #[must_use]
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// Returns candidate identifiers.
    #[must_use]
    pub fn identifiers(&self) -> &BTreeSet<String> {
        &self.identifiers
    }

    /// Returns source-owned external identifiers.
    #[must_use]
    pub fn source_identifiers(&self) -> &BTreeSet<ExternalIdentifier> {
        &self.source_identifiers
    }

    /// Returns candidate transliteration forms.
    #[must_use]
    pub fn transliterations(&self) -> &BTreeSet<String> {
        &self.transliterations
    }

    /// Returns candidate language markers.
    #[must_use]
    pub fn languages(&self) -> &BTreeSet<String> {
        &self.languages
    }

    /// Returns lexical mapping keys.
    #[must_use]
    pub fn lexical_keys(&self) -> &BTreeSet<String> {
        &self.lexical_keys
    }

    /// Returns semantic relationship keys.
    #[must_use]
    pub fn semantic_relationship_keys(&self) -> &BTreeSet<String> {
        &self.semantic_relationship_keys
    }

    /// Returns context keys.
    #[must_use]
    pub fn context_keys(&self) -> &BTreeSet<String> {
        &self.context_keys
    }

    /// Returns graph neighborhood keys.
    #[must_use]
    pub fn graph_neighborhood_keys(&self) -> &BTreeSet<String> {
        &self.graph_neighborhood_keys
    }
}

/// Deterministic candidate-generation signal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CandidateSignal {
    /// Exact canonical/source identifier match.
    ExactIdentifier,
    /// Exact alias match.
    ExactAlias,
    /// Source-scoped external identifier match.
    SourceIdentifier,
    /// Generic normalized textual match.
    Normalized,
    /// Match against a caller-supplied transliteration.
    Transliteration,
    /// Existing lexical mapping supports the candidate.
    LexicalMapping,
    /// Language metadata supports the candidate.
    Language,
    /// Existing semantic relationship metadata supports the candidate.
    SemanticRelationship,
    /// Context metadata supports the candidate.
    Context,
    /// Graph neighborhood metadata supports the candidate.
    GraphNeighborhood,
}

impl CandidateSignal {
    /// Returns the fixed deterministic priority order used by the initial
    /// Phase 3 ranking policy. Larger values are stronger. These are ordinal
    /// ranks, not weighted contributions.
    #[must_use]
    pub const fn priority(self) -> u8 {
        match self {
            Self::ExactIdentifier => 10,
            Self::ExactAlias => 9,
            Self::SourceIdentifier => 8,
            Self::Normalized => 7,
            Self::Transliteration => 6,
            Self::LexicalMapping => 5,
            Self::Language => 4,
            Self::SemanticRelationship => 3,
            Self::Context => 2,
            Self::GraphNeighborhood => 1,
        }
    }
}

impl fmt::Display for CandidateSignal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::ExactIdentifier => "exact-identifier",
            Self::ExactAlias => "exact-alias",
            Self::SourceIdentifier => "source-identifier",
            Self::Normalized => "normalized",
            Self::Transliteration => "transliteration",
            Self::LexicalMapping => "lexical-mapping",
            Self::Language => "language",
            Self::SemanticRelationship => "semantic-relationship",
            Self::Context => "context",
            Self::GraphNeighborhood => "graph-neighborhood",
        };
        formatter.write_str(value)
    }
}

/// An ephemeral candidate produced by deterministic candidate generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    entity_id: EntityId,
    signals: BTreeSet<CandidateSignal>,
}

impl Candidate {
    /// Creates a candidate from one canonical entity identity and its signals.
    #[must_use]
    pub fn new(entity_id: EntityId, signals: BTreeSet<CandidateSignal>) -> Self {
        Self { entity_id, signals }
    }

    /// Returns the candidate's canonical entity identity.
    #[must_use]
    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    /// Returns the deterministic signals supporting this candidate.
    #[must_use]
    pub fn signals(&self) -> &BTreeSet<CandidateSignal> {
        &self.signals
    }

    /// Returns the strongest signal priority for this candidate.
    #[must_use]
    pub fn primary_priority(&self) -> u8 {
        self.signals
            .iter()
            .map(|signal| signal.priority())
            .max()
            .unwrap_or(0)
    }

    /// Returns whether the candidate has at least one deterministic signal.
    #[must_use]
    pub fn is_supported(&self) -> bool {
        !self.signals.is_empty()
    }
}

/// Generates candidates from one input and an ephemeral pool of known entity
/// profiles.
///
/// This function performs high-recall deterministic candidate generation only.
/// It does not rank, canonicalize, merge, persist, or mutate entities.
pub fn generate_candidates(
    input: &ResolutionInput,
    profiles: &[EntityCandidateProfile],
) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for profile in profiles {
        let mut signals = BTreeSet::new();

        if let Some(identifier) = input.identifier()
            && profile
                .identifiers()
                .iter()
                .any(|value| exact_match(identifier, value))
        {
            signals.insert(CandidateSignal::ExactIdentifier);
        }

        if profile
            .aliases()
            .iter()
            .any(|alias| exact_match(input.text(), alias))
        {
            signals.insert(CandidateSignal::ExactAlias);
        }

        if let Some(external) = input.external_identifier()
            && profile.source_identifiers().contains(external)
        {
            signals.insert(CandidateSignal::SourceIdentifier);
        }

        if profile
            .names()
            .iter()
            .chain(profile.aliases())
            .any(|value| normalized_match(input.text(), value).unwrap_or(false))
        {
            signals.insert(CandidateSignal::Normalized);
        }

        if input.transliterations().iter().any(|reference_form| {
            profile.transliterations().iter().any(|candidate_form| {
                transliteration_match(reference_form, candidate_form).unwrap_or(false)
            })
        }) {
            signals.insert(CandidateSignal::Transliteration);
        }

        if !input.lexical_keys().is_disjoint(profile.lexical_keys()) {
            signals.insert(CandidateSignal::LexicalMapping);
        }

        if let Some(language) = input.language()
            && profile.languages().contains(language)
        {
            signals.insert(CandidateSignal::Language);
        }

        if !input
            .semantic_relationship_keys()
            .is_disjoint(profile.semantic_relationship_keys())
        {
            signals.insert(CandidateSignal::SemanticRelationship);
        }

        if !input.context_keys().is_disjoint(profile.context_keys()) {
            signals.insert(CandidateSignal::Context);
        }

        if !input
            .graph_neighborhood_keys()
            .is_disjoint(profile.graph_neighborhood_keys())
        {
            signals.insert(CandidateSignal::GraphNeighborhood);
        }

        if !signals.is_empty() {
            candidates.push(Candidate::new(profile.entity_id().clone(), signals));
        }
    }

    candidates.sort_by(|left, right| {
        right
            .primary_priority()
            .cmp(&left.primary_priority())
            .then_with(|| left.entity_id().cmp(right.entity_id()))
    });

    candidates
}

#[cfg(test)]
mod tests {
    use super::super::crosswalk::ExternalIdentifier;
    use super::{
        CandidateSignal, EntityCandidateProfile, ResolutionInput, ResolutionReference,
        generate_candidates,
    };
    use crate::identity::{EntityId, MentionId, SourceId};

    fn entity(value: &str) -> EntityId {
        EntityId::new(value).expect("valid entity")
    }

    #[test]
    fn candidate_has_no_persistent_identity() {
        let candidate = super::Candidate::new(
            entity("entity-1"),
            [CandidateSignal::ExactIdentifier].into_iter().collect(),
        );

        assert_eq!(candidate.entity_id().as_str(), "entity-1");
        assert!(candidate.is_supported());
    }

    #[test]
    fn exact_identifier_generates_the_strongest_signal() {
        let profile = EntityCandidateProfile::new(entity("entity-1")).with_identifier("ref-1");
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            "Muhammad",
        )
        .with_identifier("ref-1");

        let candidates = generate_candidates(&input, &[profile]);

        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0]
                .signals()
                .contains(&CandidateSignal::ExactIdentifier)
        );
        assert_eq!(candidates[0].primary_priority(), 10);
    }

    #[test]
    fn exact_alias_and_normalized_matching_are_supported() {
        let profile = EntityCandidateProfile::new(entity("entity-1"))
            .with_alias("Muhammad")
            .with_name("Muhammad");
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            " Muhammad ",
        );

        let candidates = generate_candidates(&input, &[profile]);

        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0]
                .signals()
                .contains(&CandidateSignal::Normalized)
        );
    }

    #[test]
    fn source_identifier_matching_is_source_scoped() {
        let source = SourceId::new("source-a").expect("valid source");
        let external = ExternalIdentifier::new(source.clone(), "42").expect("valid external");
        let profile = EntityCandidateProfile::new(entity("entity-1"))
            .with_source_identifier(external.clone());
        let input = ResolutionInput::new(
            ResolutionReference::Reference(
                crate::identity::ReferenceId::new("ref-1").expect("valid reference"),
            ),
            "unknown",
        )
        .with_source(source)
        .with_external_identifier(external);

        let candidates = generate_candidates(&input, &[profile]);

        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0]
                .signals()
                .contains(&CandidateSignal::SourceIdentifier)
        );
    }

    #[test]
    fn transliteration_is_consumed_without_generating_language_specific_forms() {
        let profile =
            EntityCandidateProfile::new(entity("entity-1")).with_transliteration("Muhammad");
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            "محمد",
        )
        .with_transliteration("Muhammad");

        let candidates = generate_candidates(&input, &[profile]);

        assert_eq!(candidates.len(), 1);
        assert!(
            candidates[0]
                .signals()
                .contains(&CandidateSignal::Transliteration)
        );
    }

    #[test]
    fn context_and_graph_signals_can_assist_candidate_generation() {
        let profile = EntityCandidateProfile::new(entity("entity-1"))
            .with_context_key("quran")
            .with_graph_neighborhood_key("prophet");
        let input = ResolutionInput::new(
            ResolutionReference::Mention(MentionId::new("mention-1").expect("valid mention")),
            "unknown",
        )
        .with_context_key("quran")
        .with_graph_neighborhood_key("prophet");

        let candidates = generate_candidates(&input, &[profile]);

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].signals().contains(&CandidateSignal::Context));
        assert!(
            candidates[0]
                .signals()
                .contains(&CandidateSignal::GraphNeighborhood)
        );
    }
}
