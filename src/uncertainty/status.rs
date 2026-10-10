//! Phase 4 epistemic-status vocabulary.
//!
//! This module intentionally reuses the Phase 2 `AssertionStatus` type. It does
//! not define a second enum competing with the status stored by
//! `KnowledgeAssertion`. Phase 3 `ResolutionState` remains a separate type.

use crate::assertion::AssertionStatus;

/// Compatibility-facing name for the shared assertion epistemic status type.
///
/// This is a re-export, not a second enum. The Phase 2 variants remain
/// available for compatibility; use [`is_phase4_epistemic_status`] to identify
/// one of the six explicit Phase 4 epistemic states.
pub use crate::assertion::AssertionStatus as EpistemicStatus;

/// The six explicit epistemic states defined by the Phase 4 scope.
pub const EPISTEMIC_STATES: [EpistemicStatus; 6] = [
    AssertionStatus::Known,
    AssertionStatus::Unknown,
    AssertionStatus::Uncertain,
    AssertionStatus::Ambiguous,
    AssertionStatus::Disputed,
    AssertionStatus::Conflicting,
];

/// Returns whether a status belongs to the explicit Phase 4 epistemic vocabulary.
#[must_use]
pub const fn is_phase4_epistemic_status(status: AssertionStatus) -> bool {
    matches!(
        status,
        AssertionStatus::Known
            | AssertionStatus::Unknown
            | AssertionStatus::Uncertain
            | AssertionStatus::Ambiguous
            | AssertionStatus::Disputed
            | AssertionStatus::Conflicting
    )
}

#[cfg(test)]
mod tests {
    use super::{EPISTEMIC_STATES, EpistemicStatus, is_phase4_epistemic_status};
    use crate::assertion::AssertionStatus;
    use crate::resolution::ResolutionState;
    use std::any::TypeId;

    #[test]
    fn public_epistemic_vocabulary_contains_exactly_the_six_phase4_states() {
        assert_eq!(EPISTEMIC_STATES.len(), 6);
        for status in EPISTEMIC_STATES {
            assert!(is_phase4_epistemic_status(status));
        }
        assert!(!is_phase4_epistemic_status(AssertionStatus::Provisional));
        assert!(!is_phase4_epistemic_status(AssertionStatus::Accepted));
        assert!(!is_phase4_epistemic_status(AssertionStatus::Rejected));
    }

    #[test]
    fn epistemic_status_is_an_alias_not_a_competing_enum() {
        assert_eq!(
            TypeId::of::<EpistemicStatus>(),
            TypeId::of::<AssertionStatus>()
        );
    }

    #[test]
    fn resolution_state_remains_a_distinct_type() {
        assert_ne!(
            TypeId::of::<ResolutionState>(),
            TypeId::of::<EpistemicStatus>()
        );
        assert_eq!(ResolutionState::Ambiguous.as_str(), "ambiguous");
        assert_eq!(EpistemicStatus::Ambiguous.as_str(), "ambiguous");
    }
}
