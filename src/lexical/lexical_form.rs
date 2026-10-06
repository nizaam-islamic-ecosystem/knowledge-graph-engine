//! Lexical-form semantic boundary reserved for a later phase.
//!
//! Phase 1 establishes `LexicalFormId` in `crate::identity` but does not need
//! a concrete `LexicalForm` object to satisfy the current semantic-foundation
//! contract. Keeping this file declaration-only avoids inventing lexical
//! semantics before the later lexical model is defined.
//!
//! In particular, Phase 1 does not implement:
//! - lemma semantics;
//! - root extraction;
//! - sense modeling;
//! - lexical mappings;
//! - morphology;
//! - stemming;
//! - parsing;
//! - Arabic linguistic analysis.
