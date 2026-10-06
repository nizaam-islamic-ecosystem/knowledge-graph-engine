//! Epistemic status for knowledge assertions.
//!
//! Phase 2 intentionally provides only a small epistemic-status vocabulary.
//! This is not a truth engine, authority system, evidence system, provenance
//! system, or reasoning engine.

use core::fmt;

/// Minimal Phase 2 epistemic status of a knowledge assertion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub enum AssertionStatus {
    /// The assertion has been introduced but has not reached an accepted
    /// epistemic state.
    #[default]
    Provisional,

    /// The assertion is currently accepted by the semantic model.
    Accepted,

    /// The assertion is currently contested.
    Disputed,

    /// The assertion has been rejected.
    Rejected,
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
    fn status_values_have_stable_representations() {
        assert_eq!(AssertionStatus::Provisional.as_str(), "provisional");
        assert_eq!(AssertionStatus::Accepted.as_str(), "accepted");
        assert_eq!(AssertionStatus::Disputed.as_str(), "disputed");
        assert_eq!(AssertionStatus::Rejected.as_str(), "rejected");
    }

    #[test]
    fn provisional_is_the_default_status() {
        assert_eq!(AssertionStatus::default(), AssertionStatus::Provisional);
    }

    #[test]
    fn statuses_remain_distinct() {
        assert_ne!(AssertionStatus::Accepted, AssertionStatus::Disputed);

        assert_ne!(AssertionStatus::Disputed, AssertionStatus::Rejected);
    }

    #[test]
    fn display_matches_the_stable_representation() {
        assert_eq!(AssertionStatus::Accepted.to_string(), "accepted");
    }
}
