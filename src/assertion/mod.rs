//! Knowledge-assertion identity boundary for Phase 1.
//!
//! Phase 1 establishes only [`KnowledgeAssertionId`]. The semantic
//! `KnowledgeAssertion` model and its subject, predicate, object, qualifier,
//! status, and context components are intentionally deferred to a later phase.
//!
//! The child scaffolds in this directory are therefore not wired into the
//! Phase 1 public module. Keeping them undeclared prevents deferred
//! functionality from becoming part of the compiled/public Phase 1 boundary.

pub use crate::identity::KnowledgeAssertionId;
