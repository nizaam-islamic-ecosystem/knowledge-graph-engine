//! Strongly typed identity for a verification record.

use nizaam_core::identity;

identity!(
    /// Identifies a verification record for evidence or knowledge.
    VerificationId
);

#[cfg(test)]
mod tests {
    use super::VerificationId;

    #[test]
    fn verification_id_can_be_generated() {
        let id = VerificationId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn verification_id_generation_produces_distinct_values() {
        assert_ne!(VerificationId::generate(), VerificationId::generate());
    }

    #[test]
    fn verification_id_accepts_valid_values_and_rejects_empty_values() {
        let id = VerificationId::new("verification-1").expect("valid verification identity");
        assert_eq!(id.as_str(), "verification-1");
        assert!(VerificationId::new("").is_err());
        assert!(VerificationId::new("   ").is_err());
    }
}
