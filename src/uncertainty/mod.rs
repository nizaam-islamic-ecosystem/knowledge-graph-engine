//! Phase 4 epistemic status, confidence, and contradiction foundations.
//!
//! Status extends the Phase 2 assertion-status type, confidence is recorded
//! with its support basis and context, and contradictions remain explicit
//! inspectable records. This module does not perform full query execution,
//! confidence calculation, or advanced contradiction resolution.

mod confidence;
mod contradiction;
mod status;

pub use confidence::{
    ConfidenceAssessment, ConfidenceBasis, ConfidenceContext, ConfidenceError, ConfidenceLabel,
    ConfidenceScore, ConfidenceTarget, ConfidenceValue,
};
pub use contradiction::{
    Contradiction, ContradictionError, ContradictionFinding, ContradictionKind, ContradictionSet,
    ContradictionStatus, CurrentViewPolicy, detect_contradiction,
};
pub use status::{EPISTEMIC_STATES, EpistemicStatus, is_phase4_epistemic_status};

#[cfg(test)]
mod tests {
    use super::{
        ConfidenceAssessment, ConfidenceBasis, ConfidenceContext, ConfidenceTarget,
        ConfidenceValue, EPISTEMIC_STATES, EpistemicStatus, is_phase4_epistemic_status,
    };
    use crate::assertion::AssertionStatus;
    use crate::evidence::EvidenceRole;
    use crate::identity::{EvidenceId, KnowledgeAssertionId};

    #[test]
    fn public_uncertainty_boundary_exposes_all_phase4_epistemic_states() {
        assert_eq!(EPISTEMIC_STATES.len(), 6);
        assert!(EPISTEMIC_STATES.into_iter().all(is_phase4_epistemic_status));
        assert_eq!(EpistemicStatus::Disputed, AssertionStatus::Disputed);
    }

    #[test]
    fn confidence_and_status_are_separate_values() {
        let target = ConfidenceTarget::new(
            KnowledgeAssertionId::new("assertion-public").unwrap(),
            EvidenceId::new("evidence-public").unwrap(),
            EvidenceRole::Supports,
        )
        .unwrap();
        let basis = ConfidenceBasis::new(
            [EvidenceId::new("evidence-public").unwrap()],
            std::iter::empty(),
            Some("source passage was reviewed".to_owned()),
        )
        .unwrap();
        let assessment = ConfidenceAssessment::new(
            target,
            ConfidenceValue::qualitative("moderate").unwrap(),
            basis,
            ConfidenceContext::new(),
        );

        assert_eq!(assessment.value().as_qualitative(), Some("moderate"));
        assert_eq!(AssertionStatus::Uncertain.as_str(), "uncertain");
    }
}
