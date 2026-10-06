//! Nizaam Knowledge Graph
//!
//! Top-level public library boundary for the Knowledge Graph engine.
//!
//! The crate root declares the complete current module tree up front. This keeps
//! the top-level module boundary stable as later phases populate the existing
//! scaffolded directories.
//!
//! The module declarations below intentionally include both implemented and
//! future-phase modules. Declaring a module does not mean its future behavior is
//! implemented or part of the current Phase 1 semantic contract.
//!
//! The public `use`/re-export surface below exposes the currently established
//! Phase 0 and Phase 1 contracts without recreating Core or Indexing types.
//!
//! Core remains authoritative for universal runtime, lifecycle, request
//! admission, execution context, cancellation, deadlines, capability dispatch,
//! registration, Control Plane coordination, and universal request/response
//! infrastructure. The Knowledge Graph adapts to those mechanisms rather than
//! recreating them.

pub mod assertion;
pub mod authority;
pub mod concept;
pub mod contract;
pub mod engine;
pub mod entity;
pub mod evidence;
pub mod graph;
pub mod identity;
pub mod index;
pub mod ingestion;
pub mod integration;
pub mod lexical;
pub mod ml;
pub mod ontology;
pub mod provenance;
pub mod query;
pub mod reasoning;
pub mod relationship;
pub mod resolution;
pub mod semantics;
pub mod source;
pub mod storage;
pub mod temporal;
pub mod uncertainty;
pub mod versioning;

// -----------------------------------------------------------------------------
// Phase 0 public surface
// -----------------------------------------------------------------------------

// The engine module remains the primary namespace for the Phase 0 engine
// facade. Its existing public items are consumed through `engine::...`.
// No Core runtime/registry/capability implementation is duplicated here.

// -----------------------------------------------------------------------------
// Phase 1 identity public surface
// -----------------------------------------------------------------------------

pub use identity::{
    ConceptId, EntityId, KnowledgeAssertionId, LexicalFormId, MentionId, ReferenceId, SourceId,
};

// -----------------------------------------------------------------------------
// Phase 1 entity public surface
// -----------------------------------------------------------------------------

pub use entity::{Alias, Entity, Mention, Name};

// -----------------------------------------------------------------------------
// Phase 1 concept public surface
// -----------------------------------------------------------------------------

pub use concept::Concept;

// -----------------------------------------------------------------------------
// Phase 1 source public surface
// -----------------------------------------------------------------------------

pub use source::{Reference, Source};

// -----------------------------------------------------------------------------
// Phase 1 lexical public surface
// -----------------------------------------------------------------------------

// `LexicalFormId` is re-exported above from the canonical identity module.
// There is intentionally no concrete `LexicalForm` root export in Phase 1.

// -----------------------------------------------------------------------------
// Phase 1 assertion public surface
// -----------------------------------------------------------------------------

// `KnowledgeAssertionId` is re-exported above from the canonical identity
// module. The semantic `KnowledgeAssertion` model remains deferred.
