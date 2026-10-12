//! Historical semantic change tracking for the Knowledge Graph.
//!
//! A Change Set represents the canonical semantic change that became part of
//! the KG state. It is intentionally distinct from a physical journal,
//! WAL record, audit record, or temporary transaction mutation.

use nizaam_core::identity;

use crate::identity::{
    ActivityId, AgentId, ConceptId, ContradictionId, EntityId, EvidenceId, KnowledgeAssertionId,
    LexicalFormId, MentionId, ReferenceId, SourceId, VerificationId,
};

use super::KgVersionId;

identity!(
    /// Identifies a semantic Change Set in KG history.
    ChangeSetId
);

/// Semantic kind of change applied to a KG object.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeKind {
    /// A new semantic object became part of the KG.
    Created,
    /// An existing semantic object was changed.
    Updated,
    /// An existing semantic object was retracted from the canonical state.
    Retracted,
    /// An existing semantic object was superseded by another semantic state.
    Superseded,
    /// A semantic relationship/link was established.
    Linked,
    /// A semantic relationship/link was removed.
    Unlinked,
    /// Non-core semantic metadata was changed.
    MetadataChanged,
}

impl ChangeKind {
    /// Returns the stable persisted representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Retracted => "retracted",
            Self::Superseded => "superseded",
            Self::Linked => "linked",
            Self::Unlinked => "unlinked",
            Self::MetadataChanged => "metadata_changed",
        }
    }

    /// Parses a persisted change kind.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "created" => Self::Created,
            "updated" => Self::Updated,
            "retracted" => Self::Retracted,
            "superseded" => Self::Superseded,
            "linked" => Self::Linked,
            "unlinked" => Self::Unlinked,
            "metadata_changed" => Self::MetadataChanged,
            _ => return None,
        })
    }
}

impl std::str::FromStr for ChangeKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ChangeKind::from_str(value).ok_or("invalid persisted change kind")
    }
}

/// The semantic object affected by a Change Set entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeTarget {
    Entity(EntityId),
    Concept(ConceptId),
    KnowledgeAssertion(KnowledgeAssertionId),
    Source(SourceId),
    Reference(ReferenceId),
    LexicalForm(LexicalFormId),
    Mention(MentionId),
    Evidence(EvidenceId),
    Activity(ActivityId),
    Agent(AgentId),
    Verification(VerificationId),
    Contradiction(ContradictionId),
}

impl ChangeTarget {
    /// Returns a stable semantic kind name for the target.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Entity(_) => "entity",
            Self::Concept(_) => "concept",
            Self::KnowledgeAssertion(_) => "knowledge_assertion",
            Self::Source(_) => "source",
            Self::Reference(_) => "reference",
            Self::LexicalForm(_) => "lexical_form",
            Self::Mention(_) => "mention",
            Self::Evidence(_) => "evidence",
            Self::Activity(_) => "activity",
            Self::Agent(_) => "agent",
            Self::Verification(_) => "verification",
            Self::Contradiction(_) => "contradiction",
        }
    }

    /// Returns the underlying semantic identity value.
    pub fn id(&self) -> &str {
        match self {
            Self::Entity(value) => value.as_str(),
            Self::Concept(value) => value.as_str(),
            Self::KnowledgeAssertion(value) => value.as_str(),
            Self::Source(value) => value.as_str(),
            Self::Reference(value) => value.as_str(),
            Self::LexicalForm(value) => value.as_str(),
            Self::Mention(value) => value.as_str(),
            Self::Evidence(value) => value.as_str(),
            Self::Activity(value) => value.as_str(),
            Self::Agent(value) => value.as_str(),
            Self::Verification(value) => value.as_str(),
            Self::Contradiction(value) => value.as_str(),
        }
    }

    /// Reconstructs a semantic target from its persisted kind and identity.
    pub(crate) fn from_kind_and_id(kind: &str, id: &str) -> Result<Self, String> {
        macro_rules! construct {
            ($type:ty, $variant:path) => {
                <$type>::new(id)
                    .map($variant)
                    .map_err(|_| format!("invalid {kind} identity"))
            };
        }

        match kind {
            "entity" => construct!(EntityId, Self::Entity),
            "concept" => construct!(ConceptId, Self::Concept),
            "knowledge_assertion" => construct!(KnowledgeAssertionId, Self::KnowledgeAssertion),
            "source" => construct!(SourceId, Self::Source),
            "reference" => construct!(ReferenceId, Self::Reference),
            "lexical_form" => construct!(LexicalFormId, Self::LexicalForm),
            "mention" => construct!(MentionId, Self::Mention),
            "evidence" => construct!(EvidenceId, Self::Evidence),
            "activity" => construct!(ActivityId, Self::Activity),
            "agent" => construct!(AgentId, Self::Agent),
            "verification" => construct!(VerificationId, Self::Verification),
            "contradiction" => construct!(ContradictionId, Self::Contradiction),
            _ => Err(format!("unknown change target kind '{kind}'")),
        }
    }
}

/// One semantic change inside a Change Set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    kind: ChangeKind,
    target: ChangeTarget,
}

impl Change {
    /// Creates one semantic change.
    pub const fn new(kind: ChangeKind, target: ChangeTarget) -> Self {
        Self { kind, target }
    }

    /// Returns the semantic change kind.
    pub const fn kind(&self) -> ChangeKind {
        self.kind
    }

    /// Returns the semantic object affected by the change.
    pub const fn target(&self) -> &ChangeTarget {
        &self.target
    }
}

/// A canonical semantic Change Set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    id: ChangeSetId,
    base_version: Option<KgVersionId>,
    changes: Vec<Change>,
}

impl ChangeSet {
    /// Creates a Change Set against the supplied base version.
    pub fn new(base_version: Option<KgVersionId>) -> Self {
        Self {
            id: ChangeSetId::generate(),
            base_version,
            changes: Vec::new(),
        }
    }

    /// Creates an empty Change Set for the initial KG state.
    pub fn initial() -> Self {
        Self::new(None)
    }

    /// Reconstructs a persisted Change Set inside the KG crate.
    pub(crate) fn from_parts(
        id: ChangeSetId,
        base_version: Option<KgVersionId>,
        changes: Vec<Change>,
    ) -> Self {
        Self {
            id,
            base_version,
            changes,
        }
    }

    /// Returns the Change Set identity.
    pub fn id(&self) -> &ChangeSetId {
        &self.id
    }

    /// Returns the version from which this Change Set was constructed.
    pub fn base_version(&self) -> Option<&KgVersionId> {
        self.base_version.as_ref()
    }

    /// Adds one semantic change to the Change Set.
    pub fn push(&mut self, change: Change) {
        self.changes.push(change);
    }

    /// Returns all semantic changes in insertion order.
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    /// Returns the number of semantic changes in this Change Set.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns whether the Change Set contains no semantic changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_kind_persistence_names_round_trip() {
        for kind in [
            ChangeKind::Created,
            ChangeKind::Updated,
            ChangeKind::Retracted,
            ChangeKind::Superseded,
            ChangeKind::Linked,
            ChangeKind::Unlinked,
            ChangeKind::MetadataChanged,
        ] {
            assert_eq!(ChangeKind::from_str(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn change_target_exposes_semantic_identity_without_physical_storage() {
        let entity = EntityId::new("entity-1").expect("valid entity identity");
        let target = ChangeTarget::Entity(entity);

        assert_eq!(target.kind(), "entity");
        assert_eq!(target.id(), "entity-1");

        let restored = ChangeTarget::from_kind_and_id(target.kind(), target.id())
            .expect("target should restore");
        assert_eq!(restored, target);
    }

    #[test]
    fn change_set_can_be_reconstructed() {
        let version = KgVersionId::generate();
        let change_set_id = ChangeSetId::generate();
        let change = Change::new(
            ChangeKind::Created,
            ChangeTarget::Entity(EntityId::new("entity-1").expect("valid entity")),
        );

        let restored = ChangeSet::from_parts(
            change_set_id.clone(),
            Some(version.clone()),
            vec![change.clone()],
        );

        assert_eq!(restored.id(), &change_set_id);
        assert_eq!(restored.base_version(), Some(&version));
        assert_eq!(restored.changes(), &[change]);
    }
}
