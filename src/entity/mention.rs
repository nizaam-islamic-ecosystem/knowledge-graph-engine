//! Minimal mention representation for Phase 1.
//!
//! A mention is an occurrence of a representation in a source/context.
//! Phase 1 keeps the model deliberately small and does not implement entity
//! resolution, candidate generation, matching, disambiguation, or ranking.

use crate::identity::MentionId;

/// A minimal occurrence/mention representation.
///
/// The representation text is preserved as supplied. Source linkage and
/// richer contextual semantics can be added by later phases when required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mention {
    id: MentionId,
    representation: String,
}

impl Mention {
    /// Constructs a mention from its identity and represented text.
    #[must_use]
    pub fn new(id: MentionId, representation: impl Into<String>) -> Self {
        Self {
            id,
            representation: representation.into(),
        }
    }

    /// Returns the mention identity.
    #[must_use]
    pub fn id(&self) -> &MentionId {
        &self.id
    }

    /// Returns the represented text exactly as supplied.
    #[must_use]
    pub fn representation(&self) -> &str {
        &self.representation
    }
}

#[cfg(test)]
mod tests {
    use super::Mention;
    use crate::identity::MentionId;

    #[test]
    fn mention_preserves_identity_and_representation() {
        let id = MentionId::generate();
        let mention = Mention::new(id.clone(), "محمد");

        assert_eq!(mention.id(), &id);
        assert_eq!(mention.representation(), "محمد");
    }

    #[test]
    fn mention_can_preserve_multilingual_representation_text() {
        let arabic = Mention::new(MentionId::generate(), "محمد");
        let english = Mention::new(MentionId::generate(), "Muhammad");
        let urdu = Mention::new(MentionId::generate(), "محمد");

        assert_eq!(arabic.representation(), "محمد");
        assert_eq!(english.representation(), "Muhammad");
        assert_eq!(urdu.representation(), "محمد");
    }
}
