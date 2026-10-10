//! Epistemic status for knowledge assertions.
//!
//! The original Phase 2 variants remain available for source compatibility.
//! Phase 4 extends this same type with an explicit epistemic vocabulary rather
//! than introducing a competing assertion-status enum. Resolution state is a
//! separate concept owned by the Phase 3 entity-resolution module.

use core::fmt;

/// Status of a canonical knowledge assertion.
///
/// `Provisional`, `Accepted`, and `Rejected` retain the initial Phase 2 status
/// vocabulary. Phase 4 also uses this type for the explicit epistemic states
/// `Known`, `Unknown`, `Uncertain`, `Ambiguous`, `Disputed`, and `Conflicting`.
/// Existing variants are retained and their textual representations are stable.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub enum AssertionStatus {
    /// The assertion has been introduced but has not reached an accepted state.
    #[default]
    Provisional,
    /// The assertion is accepted by the current semantic model.
    Accepted,
    /// The assertion's epistemic status is disputed.
    Disputed,
    /// The assertion has been rejected by the current semantic model.
    Rejected,
    /// The assertion is treated as known within its declared context.
    Known,
    /// The relevant knowledge is explicitly unknown.
    Unknown,
    /// The available support does not justify a determinate conclusion.
    Uncertain,
    /// More than one plausible interpretation or value remains.
    Ambiguous,
    /// The assertion participates in an explicit unresolved conflict.
    Conflicting,
}

impl AssertionStatus {
    /// Returns the stable textual representation of the status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Provisional => "provisional",
            Self::Accepted => "accepted",
            Self::Disputed => "disputed",
            Self::Rejected => "rejected",
            Self::Known => "known",
            Self::Unknown => "unknown",
            Self::Uncertain => "uncertain",
            Self::Ambiguous => "ambiguous",
            Self::Conflicting => "conflicting",
        }
    }
}

impl fmt::Display for AssertionStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::AssertionStatus;

    #[test]
    fn phase2_status_representations_remain_stable() {
        assert_eq!(AssertionStatus::Provisional.as_str(), "provisional");
        assert_eq!(AssertionStatus::Accepted.as_str(), "accepted");
        assert_eq!(AssertionStatus::Disputed.as_str(), "disputed");
        assert_eq!(AssertionStatus::Rejected.as_str(), "rejected");
    }

    #[test]
    fn phase4_epistemic_states_have_stable_representations() {
        assert_eq!(AssertionStatus::Known.as_str(), "known");
        assert_eq!(AssertionStatus::Unknown.as_str(), "unknown");
        assert_eq!(AssertionStatus::Uncertain.as_str(), "uncertain");
        assert_eq!(AssertionStatus::Ambiguous.as_str(), "ambiguous");
        assert_eq!(AssertionStatus::Disputed.as_str(), "disputed");
        assert_eq!(AssertionStatus::Conflicting.as_str(), "conflicting");
    }

    #[test]
    fn provisional_remains_the_default_for_phase2_compatibility() {
        assert_eq!(AssertionStatus::default(), AssertionStatus::Provisional);
    }

    #[test]
    fn status_values_remain_distinct() {
        let statuses = [
            AssertionStatus::Provisional,
            AssertionStatus::Accepted,
            AssertionStatus::Disputed,
            AssertionStatus::Rejected,
            AssertionStatus::Known,
            AssertionStatus::Unknown,
            AssertionStatus::Uncertain,
            AssertionStatus::Ambiguous,
            AssertionStatus::Conflicting,
        ];
        let unique = statuses
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 9);
    }

    #[test]
    fn display_matches_the_stable_representation() {
        assert_eq!(AssertionStatus::Accepted.to_string(), "accepted");
        assert_eq!(AssertionStatus::Conflicting.to_string(), "conflicting");
    }
}
