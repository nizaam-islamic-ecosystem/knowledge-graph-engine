//! Integration boundaries for the Knowledge Graph Engine.
//!
//! Core remains the owner of shared runtime, lifecycle, context, security,
//! transport, and universal communication mechanisms. Engine-specific
//! contracts are exposed through separate adapters.

pub mod core;
pub mod indexing;
