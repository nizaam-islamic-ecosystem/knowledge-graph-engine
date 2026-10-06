//! Core lifecycle integration for the Nizaam Knowledge Graph engine.
//!
//! Phase 0 does not implement a second lifecycle system.
//!
//! `nizaam_core::runtime::EngineRuntime` remains the authoritative owner of
//! engine lifecycle state, transition validity, request admission, draining,
//! and shutdown coordination.
//!
//! This module only provides the KG engine's lifecycle sequencing boundary.
//!
//! The Phase 0 engine lifecycle is:
//!
//! ```text
//! Created
//!     -> Starting
//!     -> Configuring
//!     -> Dependencies
//!     -> Capabilities
//!     -> Registering
//!     -> Ready
//!     -> Serving
//!     -> Draining
//!     -> Stopped
//! ```
//!
//! Engine registration itself remains separate from lifecycle transition and
//! belongs to `registration.rs`.
//! Phase 0 scaffolding intentionally retains the complete Core integration surface
//! before the final crate-level public wiring is added.
//! The implementation is kept intact; this local allowance prevents intermediate
//! dead-code diagnostics from masking real errors.
#![allow(dead_code)]

use nizaam_core::error::InvalidTransition;
use nizaam_core::runtime::{EngineRuntime, LifecycleState};

/// Advances the Core runtime through the non-registration startup sequence.
///
/// The sequence is deliberately explicit so the KG engine does not introduce
/// its own lifecycle state machine:
///
/// ```text
/// Created
///     -> Starting
///     -> Configuring
///     -> Dependencies
///     -> Capabilities
/// ```
///
/// Core validates every transition and remains authoritative over the legal
/// lifecycle graph.
pub(crate) fn start(runtime: &EngineRuntime) -> Result<(), InvalidTransition> {
    for next in [
        LifecycleState::Starting,
        LifecycleState::Configuring,
        LifecycleState::Dependencies,
        LifecycleState::Capabilities,
    ] {
        runtime.transition(next)?;
    }

    Ok(())
}

/// Enters the Core `Registering` lifecycle state.
///
/// Actual engine registration remains the responsibility of
/// `registration.rs`. Keeping these operations separate preserves the
/// distinction between lifecycle progression and Control Plane registration.
pub(crate) fn begin_registration(runtime: &EngineRuntime) -> Result<(), InvalidTransition> {
    runtime.transition(LifecycleState::Registering)
}

/// Marks the Core runtime `Ready`.
///
/// The caller is responsible for ensuring that the required registration work
/// has completed successfully before invoking this transition.
pub(crate) fn mark_ready(runtime: &EngineRuntime) -> Result<(), InvalidTransition> {
    runtime.transition(LifecycleState::Ready)
}

/// Enters the Core `Serving` state.
///
/// `Ready` and `Serving` intentionally remain separate. Normal request
/// admission is governed by Core and is enabled only when the runtime is
/// serving.
pub(crate) fn serve(runtime: &EngineRuntime) -> Result<(), InvalidTransition> {
    runtime.transition(LifecycleState::Serving)
}

/// Begins graceful draining through the Core lifecycle.
///
/// Core remains responsible for rejecting newly admitted normal work after
/// entering `Draining` while coordinating already-admitted execution.
pub(crate) fn drain(runtime: &EngineRuntime) -> Result<(), InvalidTransition> {
    runtime.transition(LifecycleState::Draining)
}

/// Gracefully shuts down through the Core runtime.
///
/// Core owns the complete shutdown sequence, including draining, runtime-owned
/// cleanup, cancellation, and the terminal `Stopped` state.
pub(crate) fn shutdown(runtime: &EngineRuntime) -> Result<bool, InvalidTransition> {
    runtime.shutdown()
}

#[cfg(test)]
mod tests {
    use super::*;

    use nizaam_core::identity::{EngineId, EngineInstanceId};

    fn runtime() -> EngineRuntime {
        EngineRuntime::new(
            EngineId::new("nizaam.kg.lifecycle.test").expect("test engine id must be valid"),
            EngineInstanceId::new("nizaam.kg.lifecycle.test.instance")
                .expect("test engine instance id must be valid"),
        )
    }

    #[test]
    fn start_advances_through_the_core_startup_sequence() {
        let runtime = runtime();

        start(&runtime).expect("Core startup sequence should succeed");

        assert_eq!(runtime.state(), LifecycleState::Capabilities);
    }

    #[test]
    fn registration_transition_is_separate_from_startup() {
        let runtime = runtime();

        start(&runtime).expect("startup should succeed");
        begin_registration(&runtime).expect("registering transition should succeed");

        assert_eq!(runtime.state(), LifecycleState::Registering);
    }

    #[test]
    fn ready_and_serving_remain_distinct_states() {
        let runtime = runtime();

        start(&runtime).expect("startup should succeed");
        begin_registration(&runtime).expect("registering transition should succeed");
        mark_ready(&runtime).expect("ready transition should succeed");

        assert_eq!(runtime.state(), LifecycleState::Ready);

        serve(&runtime).expect("serving transition should succeed");
        assert_eq!(runtime.state(), LifecycleState::Serving);
    }

    #[test]
    fn drain_and_shutdown_delegate_to_core() {
        let runtime = runtime();

        start(&runtime).expect("startup should succeed");
        begin_registration(&runtime).expect("registering transition should succeed");
        mark_ready(&runtime).expect("ready transition should succeed");
        serve(&runtime).expect("serving transition should succeed");

        drain(&runtime).expect("draining transition should succeed");
        assert_eq!(runtime.state(), LifecycleState::Draining);

        shutdown(&runtime).expect("Core shutdown should succeed");
        assert_eq!(runtime.state(), LifecycleState::Stopped);
    }
}
