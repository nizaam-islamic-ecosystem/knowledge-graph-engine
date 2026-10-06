//! Semantic direction for Knowledge Graph relationships.
//!
//! This module represents the semantic orientation of a relationship
//! definition. It must not be confused with graph traversal direction.
//!
//! For example, a relationship may have semantic direction:
//!
//! ```text
//! Person ── acted-in ──> Movie
//! ```
//!
//! A graph traversal may later inspect that relationship from the opposite
//! endpoint without changing the semantic direction stored by the
//! relationship definition.

use core::fmt;

/// Semantic orientation of a relationship definition.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelationshipDirection {
    /// The semantic relationship is oriented from the assertion subject to
    /// the assertion object.
    #[default]
    SubjectToObject,

    /// The semantic relationship is oriented from the assertion object to
    /// the assertion subject.
    ObjectToSubject,
}

impl RelationshipDirection {
    /// Returns the stable textual representation of this direction.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubjectToObject => "subject-to-object",
            Self::ObjectToSubject => "object-to-subject",
        }
    }
}

impl fmt::Display for RelationshipDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::RelationshipDirection;

    #[test]
    fn subject_to_object_is_the_default_direction() {
        assert_eq!(
            RelationshipDirection::default(),
            RelationshipDirection::SubjectToObject
        );
    }

    #[test]
    fn directions_have_stable_representations() {
        assert_eq!(
            RelationshipDirection::SubjectToObject.as_str(),
            "subject-to-object"
        );

        assert_eq!(
            RelationshipDirection::ObjectToSubject.as_str(),
            "object-to-subject"
        );
    }

    #[test]
    fn semantic_directions_are_distinct() {
        assert_ne!(
            RelationshipDirection::SubjectToObject,
            RelationshipDirection::ObjectToSubject
        );
    }

    #[test]
    fn display_matches_the_stable_representation() {
        assert_eq!(
            RelationshipDirection::SubjectToObject.to_string(),
            "subject-to-object"
        );
    }
}
