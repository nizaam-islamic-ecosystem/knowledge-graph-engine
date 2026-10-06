//! Foundational knowledge-graph concept boundary for Phase 1.
//!
//! Phase 1 exposes only the minimal [`Concept`] model and its Core-backed
//! [`crate::identity::ConceptId`]. Relationship and type scaffolds remain
//! intentionally deferred because they do not contain Phase 1 functionality.

mod model;

pub use crate::identity::ConceptId;
pub use model::Concept;
