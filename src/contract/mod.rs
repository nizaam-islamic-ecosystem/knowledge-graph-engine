//! Knowledge Graph contract boundary.
//!
//! The contract module owns the minimal capability definition used by the
//! Phase 0 bootstrap path. Runtime and dispatch mechanisms remain Core-owned.

pub mod capability;

pub(crate) use capability::Capability;
