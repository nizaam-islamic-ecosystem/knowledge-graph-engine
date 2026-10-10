//! Temporal relationships kept separate from valid-time qualification.
//!
//! A temporal relationship expresses how two semantic objects relate in time
//! (for example, event A occurs before event B). It is not an interval and does
//! not state the interval during which either object is valid. This module
//! stores declared relations only; it does not infer them from timestamps.

/// A declared temporal relation between two semantic targets.
///
/// The vocabulary is based on interval-relation semantics. It is structural
/// metadata only: selecting or validating a relation against concrete interval
/// values is deliberately outside this module's current responsibility.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TemporalRelation {
    /// The subject ends before the object begins.
    Before,
    /// The subject begins after the object ends.
    After,
    /// The subject ends exactly where the object begins.
    Meets,
    /// The subject begins exactly where the object ends.
    MetBy,
    /// The subject starts earlier and overlaps the beginning of the object.
    Overlaps,
    /// The subject starts later and overlaps the end of the object.
    OverlappedBy,
    /// The subject and object begin together, and the subject ends first.
    Starts,
    /// The subject and object begin together, and the object ends first.
    StartedBy,
    /// The subject is temporally inside the object.
    During,
    /// The subject temporally contains the object.
    Contains,
    /// The subject and object end together, and the subject begins later.
    Finishes,
    /// The subject and object end together, and the object begins later.
    FinishedBy,
    /// The subject and object have equal temporal extent.
    Equal,
}

impl TemporalRelation {
    /// Returns the equivalent relation when subject and object are swapped.
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::Before => Self::After,
            Self::After => Self::Before,
            Self::Meets => Self::MetBy,
            Self::MetBy => Self::Meets,
            Self::Overlaps => Self::OverlappedBy,
            Self::OverlappedBy => Self::Overlaps,
            Self::Starts => Self::StartedBy,
            Self::StartedBy => Self::Starts,
            Self::During => Self::Contains,
            Self::Contains => Self::During,
            Self::Finishes => Self::FinishedBy,
            Self::FinishedBy => Self::Finishes,
            Self::Equal => Self::Equal,
        }
    }
}

/// A declared temporal relation between typed semantic targets.
///
/// Subject and object types may differ, while each side retains its own strong
/// identity type. The value does not assign temporal validity to either target.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TemporalRelationship<S, O> {
    subject: S,
    relation: TemporalRelation,
    object: O,
}

impl<S, O> TemporalRelationship<S, O> {
    /// Creates a declared relationship without performing temporal inference.
    #[must_use]
    pub fn new(subject: S, relation: TemporalRelation, object: O) -> Self {
        Self {
            subject,
            relation,
            object,
        }
    }

    /// Returns the relationship subject.
    #[must_use]
    pub fn subject(&self) -> &S {
        &self.subject
    }

    /// Returns the declared temporal relation.
    #[must_use]
    pub fn relation(&self) -> TemporalRelation {
        self.relation
    }

    /// Returns the relationship object.
    #[must_use]
    pub fn object(&self) -> &O {
        &self.object
    }

    /// Consumes the value and returns its typed components.
    #[must_use]
    pub fn into_parts(self) -> (S, TemporalRelation, O) {
        (self.subject, self.relation, self.object)
    }
}

impl<S: Clone, O: Clone> TemporalRelationship<S, O> {
    /// Produces the inverse declaration, swapping the subject and object.
    #[must_use]
    pub fn inverse(&self) -> TemporalRelationship<O, S> {
        TemporalRelationship::new(
            self.object.clone(),
            self.relation.inverse(),
            self.subject.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{TemporalRelation, TemporalRelationship};

    #[test]
    fn temporal_relationship_preserves_subject_relation_and_object() {
        let relation = TemporalRelationship::new("event-a", TemporalRelation::Before, "event-b");

        assert_eq!(relation.subject(), &"event-a");
        assert_eq!(relation.relation(), TemporalRelation::Before);
        assert_eq!(relation.object(), &"event-b");
    }

    #[test]
    fn inverse_relationship_swaps_targets_and_inverts_the_relation() {
        let relation = TemporalRelationship::new(1_u32, TemporalRelation::Before, 2_u32);
        let inverse = relation.inverse();

        assert_eq!(inverse.subject(), &2);
        assert_eq!(inverse.relation(), TemporalRelation::After);
        assert_eq!(inverse.object(), &1);
    }

    #[test]
    fn symmetric_equal_relation_is_its_own_inverse() {
        assert_eq!(TemporalRelation::Equal.inverse(), TemporalRelation::Equal);
    }
}
