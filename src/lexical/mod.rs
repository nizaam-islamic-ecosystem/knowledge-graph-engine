//! Minimal lexical boundary for Phase 1.
//!
//! Phase 1 already establishes the strongly typed [`LexicalFormId`] identity.
//! A concrete `LexicalForm` object is intentionally deferred because the
//! current phase does not require additional lexical behavior.
//!
//! The lexical module therefore exports the identity only and does not expose
//! the deferred lemma, root, sense, or mapping scaffolds.

pub use crate::identity::LexicalFormId;
