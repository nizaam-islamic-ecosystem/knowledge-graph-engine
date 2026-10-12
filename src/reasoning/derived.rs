//! Rebuildable derived knowledge and mandatory derivation provenance.
//!
//! Derived state is intentionally separate from canonical KG state. This first
//! implementation is in-memory and rebuildable; it does not write inferred
//! assertions back into Phase 7 canonical storage.

use std::collections::BTreeMap;

use crate::identity::{KnowledgeAssertionId, RuleId};
use crate::query::ReasoningProfile;
use crate::storage::Snapshot;
use crate::versioning::KgVersion;

use super::inference::RuleApplication;
use super::rule::RuleVersion;

/// One compact provenance trace for a derived conclusion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivationTrace {
    rule_id: RuleId,
    rule_version: RuleVersion,
    premises: Vec<KnowledgeAssertionId>,
    profile: ReasoningProfile,
    input_snapshot: String,
    input_version: crate::versioning::KgVersionId,
    depth: usize,
}

impl DerivationTrace {
    /// Creates a trace pinned to a stable Phase 7 snapshot.
    #[must_use]
    pub fn new(
        rule_id: RuleId,
        rule_version: RuleVersion,
        premises: Vec<KnowledgeAssertionId>,
        profile: ReasoningProfile,
        snapshot: &Snapshot,
        depth: usize,
    ) -> Self {
        Self {
            rule_id,
            rule_version,
            premises,
            profile,
            input_snapshot: snapshot.id().as_str().to_owned(),
            input_version: snapshot.version().id().clone(),
            depth,
        }
    }

    /// Returns the rule identity.
    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    /// Returns the rule version.
    #[must_use]
    pub const fn rule_version(&self) -> RuleVersion {
        self.rule_version
    }

    /// Returns the premise assertion identities.
    #[must_use]
    pub fn premises(&self) -> &[KnowledgeAssertionId] {
        &self.premises
    }

    /// Returns the reasoning profile.
    #[must_use]
    pub const fn profile(&self) -> ReasoningProfile {
        self.profile
    }

    /// Returns the snapshot identity as a stable string.
    #[must_use]
    pub fn input_snapshot(&self) -> &str {
        &self.input_snapshot
    }

    /// Returns the KG version pinned by the trace.
    #[must_use]
    pub const fn input_version(&self) -> &crate::versioning::KgVersionId {
        &self.input_version
    }

    /// Returns derivation depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}

/// One derivation step in an expanded explanation graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivationStep {
    conclusion: KnowledgeAssertionId,
    trace: DerivationTrace,
}

impl DerivationStep {
    /// Creates one derivation step.
    #[must_use]
    pub fn new(conclusion: KnowledgeAssertionId, trace: DerivationTrace) -> Self {
        Self { conclusion, trace }
    }

    /// Returns the conclusion identity.
    #[must_use]
    pub const fn conclusion(&self) -> &KnowledgeAssertionId {
        &self.conclusion
    }

    /// Returns the step trace.
    #[must_use]
    pub const fn trace(&self) -> &DerivationTrace {
        &self.trace
    }
}

/// Ordered explanation path for one derived conclusion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivationPath {
    steps: Vec<DerivationStep>,
}

impl DerivationPath {
    /// Creates an empty derivation path.
    #[must_use]
    pub const fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Appends a derivation step.
    pub fn push(&mut self, step: DerivationStep) {
        self.steps.push(step);
    }

    /// Returns all steps.
    #[must_use]
    pub fn steps(&self) -> &[DerivationStep] {
        &self.steps
    }
}

impl Default for DerivationPath {
    fn default() -> Self {
        Self::new()
    }
}

/// One semantic conclusion with one or more derivation traces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedKnowledge {
    assertion: crate::assertion::KnowledgeAssertion,
    traces: Vec<DerivationTrace>,
}

impl DerivedKnowledge {
    /// Creates one derived conclusion from an application.
    #[must_use]
    pub fn from_application(
        application: &RuleApplication,
        snapshot: &Snapshot,
        profile: ReasoningProfile,
    ) -> Self {
        let trace = DerivationTrace::new(
            application.rule_id().clone(),
            application.rule_version(),
            application
                .premises()
                .iter()
                .map(|premise| premise.id().clone())
                .collect(),
            profile,
            snapshot,
            application.depth(),
        );
        Self {
            assertion: application.conclusion().clone(),
            traces: vec![trace],
        }
    }

    /// Returns the derived assertion.
    #[must_use]
    pub const fn assertion(&self) -> &crate::assertion::KnowledgeAssertion {
        &self.assertion
    }

    /// Returns all derivation traces for this semantic conclusion.
    #[must_use]
    pub fn traces(&self) -> &[DerivationTrace] {
        &self.traces
    }

    /// Adds a distinct derivation trace without duplicating the conclusion.
    pub fn add_trace(&mut self, trace: DerivationTrace) {
        if !self.traces.contains(&trace) {
            self.traces.push(trace);
        }
    }
}

/// Rebuildable collection of derived conclusions keyed by semantic identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DerivedKnowledgeSet {
    values: BTreeMap<KnowledgeAssertionId, DerivedKnowledge>,
}

impl DerivedKnowledgeSet {
    /// Creates an empty derived set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts an application while preserving multiple derivations.
    pub fn insert_application(
        &mut self,
        application: &RuleApplication,
        snapshot: &Snapshot,
        profile: ReasoningProfile,
    ) {
        let key = application.conclusion().id().clone();
        let trace = DerivationTrace::new(
            application.rule_id().clone(),
            application.rule_version(),
            application
                .premises()
                .iter()
                .map(|premise| premise.id().clone())
                .collect(),
            profile,
            snapshot,
            application.depth(),
        );

        match self.values.get_mut(&key) {
            Some(existing) => existing.add_trace(trace),
            None => {
                self.values.insert(
                    key,
                    DerivedKnowledge {
                        assertion: application.conclusion().clone(),
                        traces: vec![trace],
                    },
                );
            }
        }
    }

    /// Returns one derived conclusion by identity.
    #[must_use]
    pub fn get(&self, id: &KnowledgeAssertionId) -> Option<&DerivedKnowledge> {
        self.values.get(id)
    }

    /// Returns derived conclusions in deterministic identity order.
    pub fn values(&self) -> impl Iterator<Item = &DerivedKnowledge> {
        self.values.values()
    }

    /// Returns the number of distinct semantic conclusions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no derived conclusion exists.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the pinned input KG version of a snapshot used by derived state.
    #[must_use]
    pub fn input_version(snapshot: &Snapshot) -> &KgVersion {
        snapshot.version()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertion::{
        AssertionContext, AssertionObject, AssertionPolarity, AssertionPredicate, AssertionStatus,
        KnowledgeAssertion, Qualifiers,
    };
    use crate::identity::{EntityId, RuleId};
    use crate::reasoning::inference::RuleApplication;
    use crate::reasoning::rule::RuleVersion;
    use crate::storage::Document;

    fn assertion(id: &str) -> KnowledgeAssertion {
        KnowledgeAssertion::new(
            AssertionObject::Entity(EntityId::new(id).expect("valid id")),
            AssertionPredicate::new("knows").expect("valid predicate"),
            AssertionObject::Entity(EntityId::new("object").expect("valid id")),
            AssertionContext::new(),
            Qualifiers::new(),
            AssertionStatus::Accepted,
            AssertionPolarity::Positive,
        )
    }

    fn snapshot() -> Snapshot {
        Snapshot::new(
            KgVersion::initial(),
            vec![crate::storage::RepositoryRecord::new(
                crate::versioning::ChangeTarget::Entity(
                    EntityId::new("entity-1").expect("valid id"),
                ),
                Document::from_slice(b"entity-1"),
            )],
        )
        .expect("snapshot should succeed")
    }

    #[test]
    fn derived_set_preserves_multiple_traces_for_one_conclusion() {
        let conclusion = assertion("derived");
        let premise = assertion("premise");
        let application = RuleApplication::new(
            RuleId::new("rule-1").expect("valid rule id"),
            RuleVersion::default(),
            vec![premise],
            conclusion.clone(),
            Default::default(),
            1,
        );
        let snapshot = snapshot();
        let mut set = DerivedKnowledgeSet::new();
        set.insert_application(&application, &snapshot, ReasoningProfile::Basic);
        set.insert_application(&application, &snapshot, ReasoningProfile::Basic);
        assert_eq!(set.len(), 1);
        assert_eq!(
            set.get(conclusion.id())
                .expect("derived value")
                .traces()
                .len(),
            1
        );
    }

    #[test]
    fn trace_contains_required_provenance_fields() {
        let application = RuleApplication::new(
            RuleId::new("rule-1").expect("valid rule id"),
            RuleVersion::default(),
            vec![assertion("premise")],
            assertion("derived"),
            Default::default(),
            2,
        );
        let snapshot = snapshot();
        let derived =
            DerivedKnowledge::from_application(&application, &snapshot, ReasoningProfile::Deep);
        let trace = &derived.traces()[0];
        assert_eq!(trace.rule_id().as_str(), "rule-1");
        assert_eq!(trace.profile(), ReasoningProfile::Deep);
        assert_eq!(trace.depth(), 2);
        assert!(!trace.input_snapshot().is_empty());
        assert_eq!(trace.input_version(), snapshot.version().id());
    }
}
