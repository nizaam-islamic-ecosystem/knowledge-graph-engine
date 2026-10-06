//! Runtime boundary for the Nizaam Knowledge Graph engine.
//!
//! Phase 0 intentionally does not implement a second runtime system.
//! `KgRuntime` is a thin KG-facing composition layer over the Core-backed
//! runtime adapter in `integration::core`.
//!
//! Nizaam Core remains authoritative for:
//! - lifecycle state and transition validity
//! - request admission
//! - cancellation
//! - deadlines
//! - execution context
//! - capability dispatch
//! - shutdown coordination
//! - background task management
//!
//! Engine registration remains separate and is owned by `registration.rs`.
//! Phase 0 scaffolding intentionally retains the complete Core integration surface
//! before the final crate-level public wiring is added.
//! The implementation is kept intact; this local allowance prevents intermediate
//! dead-code diagnostics from masking real errors.
#![allow(dead_code)]

use nizaam_core::capability::{CapabilityDispatchResult, CapabilityInvocation};
use nizaam_core::error::InvalidTransition;
use nizaam_core::identity::{EngineId, EngineInstanceId};
use nizaam_core::operation::OperationContext;
use nizaam_core::runtime::{EngineContext, LifecycleState, RequestAdmissionError};

use crate::integration::core::{CoreCapabilities, CoreRuntime};

use super::lifecycle;

/// Result returned by the KG runtime after Core request admission.
///
/// A request rejected at the lifecycle admission boundary returns Core's
/// `RequestAdmissionError`. Once admitted, capability dispatch remains
/// represented by Core's `CapabilityDispatchResult`.
pub(crate) type RuntimeDispatchResult = Result<CapabilityDispatchResult, RequestAdmissionError>;

/// Knowledge Graph runtime boundary over the Core runtime.
///
/// This type contains no KG-specific lifecycle state. The underlying
/// `CoreRuntime` remains the single source of truth for lifecycle, admission,
/// cancellation, deadlines, and shutdown.
#[derive(Debug)]
pub(crate) struct KgRuntime {
    core: CoreRuntime,
}

impl KgRuntime {
    /// Creates a new KG runtime in Core's `Created` state.
    #[must_use]
    pub(crate) fn new(engine_id: EngineId, engine_instance_id: EngineInstanceId) -> Self {
        Self {
            core: CoreRuntime::new(engine_id, engine_instance_id),
        }
    }

    /// Returns the logical engine identity owned by Core.
    #[must_use]
    pub(crate) fn engine_id(&self) -> &EngineId {
        self.core.engine_id()
    }

    /// Returns the concrete engine-instance identity owned by Core.
    #[must_use]
    pub(crate) fn engine_instance_id(&self) -> &EngineInstanceId {
        self.core.engine_instance_id()
    }

    /// Returns the current Core lifecycle state.
    #[must_use]
    pub(crate) fn state(&self) -> LifecycleState {
        self.core.state()
    }

    /// Advances the runtime through the non-registration startup states.
    ///
    /// Registration remains deliberately separate and is handled by the
    /// engine composition layer after the runtime reaches `Capabilities`.
    pub(crate) fn start(&self) -> Result<(), InvalidTransition> {
        lifecycle::start(self.core.runtime())
    }

    /// Enters Core's `Registering` state.
    ///
    /// Actual engine registration remains the responsibility of
    /// `registration.rs`.
    pub(crate) fn begin_registration(&self) -> Result<(), InvalidTransition> {
        lifecycle::begin_registration(self.core.runtime())
    }

    /// Marks the runtime ready after required registration work has
    /// successfully completed.
    pub(crate) fn mark_ready(&self) -> Result<(), InvalidTransition> {
        lifecycle::mark_ready(self.core.runtime())
    }

    /// Enters Core's `Serving` state.
    pub(crate) fn serve(&self) -> Result<(), InvalidTransition> {
        lifecycle::serve(self.core.runtime())
    }

    /// Begins graceful draining through Core.
    pub(crate) fn drain(&self) -> Result<(), InvalidTransition> {
        lifecycle::drain(self.core.runtime())
    }

    /// Performs Core request admission for one normal request.
    ///
    /// Core decides whether the current lifecycle state permits new work.
    pub(crate) fn admit_request(&self) -> Result<(), RequestAdmissionError> {
        self.core.admit_request()
    }

    /// Creates a Core execution context from an existing operation context.
    ///
    /// KG does not create another operation, cancellation, deadline,
    /// security, or provenance system.
    #[must_use]
    pub(crate) fn context(&self, operation: OperationContext) -> EngineContext {
        self.core.context(operation)
    }

    /// Admits and dispatches one capability invocation.
    ///
    /// The runtime performs only the local admission boundary. Capability
    /// resolution and execution remain Core capability-system concerns.
    pub(crate) fn dispatch(
        &self,
        capabilities: &CoreCapabilities,
        context: &EngineContext,
        invocation: &CapabilityInvocation,
    ) -> RuntimeDispatchResult {
        self.admit_request()?;
        Ok(capabilities.dispatch(context, invocation))
    }

    /// Gracefully shuts down through Core.
    ///
    /// Core owns draining, runtime cleanup, cancellation, and the terminal
    /// `Stopped` transition.
    pub(crate) fn shutdown(&self) -> Result<bool, InvalidTransition> {
        lifecycle::shutdown(self.core.runtime())
    }

    /// Returns Core's shutdown cancellation token.
    #[must_use]
    pub(crate) fn shutdown_token(&self) -> &nizaam_core::runtime::CancellationToken {
        self.core.shutdown_token()
    }

    /// Returns Core's background-task manager.
    ///
    /// Phase 0 does not define a KG-specific task manager.
    #[must_use]
    pub(crate) fn background_tasks(&self) -> &nizaam_core::runtime::BackgroundTasks {
        self.core.background_tasks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use nizaam_core::capability::{CapabilityDefinition, CapabilityOutcome, arc_handler};
    use nizaam_core::contracts::Version;
    use nizaam_core::identity::{CapabilityId, ContractId, CorrelationId, OperationId};
    use nizaam_core::operation::{Operation, OperationContext};

    fn engine_id() -> EngineId {
        EngineId::new("nizaam.kg.runtime.test").expect("test engine id must be valid")
    }

    fn instance_id() -> EngineInstanceId {
        EngineInstanceId::new("nizaam.kg.runtime.test.instance")
            .expect("test engine instance id must be valid")
    }

    fn runtime() -> KgRuntime {
        KgRuntime::new(engine_id(), instance_id())
    }

    fn operation_context(name: &str) -> OperationContext {
        OperationContext::new(Operation::new(
            OperationId::new(format!("nizaam.kg.runtime.{name}.operation"))
                .expect("test operation id must be valid"),
            CorrelationId::new(format!("nizaam.kg.runtime.{name}.correlation"))
                .expect("test correlation id must be valid"),
        ))
    }

    fn serving_runtime() -> KgRuntime {
        let runtime = runtime();
        runtime.start().expect("startup should succeed");
        runtime
            .begin_registration()
            .expect("registering transition should succeed");
        runtime
            .mark_ready()
            .expect("ready transition should succeed");
        runtime.serve().expect("serving transition should succeed");
        runtime
    }

    #[test]
    fn construction_delegates_identity_and_state_to_core() {
        let runtime = runtime();

        assert_eq!(runtime.engine_id(), &engine_id());
        assert_eq!(runtime.engine_instance_id(), &instance_id());
        assert_eq!(runtime.state(), LifecycleState::Created);
    }

    #[test]
    fn start_uses_core_lifecycle_sequence() {
        let runtime = runtime();

        runtime.start().expect("startup should succeed");

        assert_eq!(runtime.state(), LifecycleState::Capabilities);
    }

    #[test]
    fn request_admission_is_rejected_before_serving() {
        let runtime = runtime();

        assert!(runtime.admit_request().is_err());
    }

    #[test]
    fn context_preserves_the_supplied_operation_context() {
        let runtime = runtime();
        let operation = operation_context("context");
        let context = runtime.context(operation.clone());

        assert_eq!(context.operation(), &operation);
        assert!(context.deadline().is_none());
    }

    #[test]
    fn dispatch_admits_and_invokes_a_registered_capability() {
        let runtime = serving_runtime();
        let capabilities = CoreCapabilities::new();
        let capability_id =
            CapabilityId::new("nizaam.kg.runtime.dispatch").expect("capability id must be valid");
        let contract_id = ContractId::new("nizaam.kg.runtime.dispatch.contract")
            .expect("contract id must be valid");
        let definition = CapabilityDefinition::new(
            capability_id.clone(),
            runtime.engine_id().clone(),
            "Runtime dispatch test capability",
        )
        .expect("definition must be valid")
        .with_version(Version::new(1, 0, 0));

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = Arc::clone(&calls);
        capabilities
            .register(
                definition,
                arc_handler(move |_context, invocation| {
                    calls_for_handler.fetch_add(1, Ordering::SeqCst);
                    Ok(CapabilityOutcome::new(invocation.payload_bytes().to_vec()))
                }),
            )
            .expect("capability registration should succeed");

        let invocation =
            CapabilityInvocation::new(capability_id, contract_id, b"runtime-payload".to_vec());
        let context = runtime.context(operation_context("dispatch"));

        let result = runtime
            .dispatch(&capabilities, &context, &invocation)
            .expect("serving runtime should admit dispatch");

        assert!(matches!(
            result,
            CapabilityDispatchResult::Outcome(outcome)
                if outcome.as_bytes() == b"runtime-payload"
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn draining_rejects_new_dispatch_before_capability_execution() {
        let runtime = serving_runtime();
        let capabilities = CoreCapabilities::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = Arc::clone(&calls);
        let capability_id =
            CapabilityId::new("nizaam.kg.runtime.draining").expect("capability id must be valid");
        let contract_id = ContractId::new("nizaam.kg.runtime.draining.contract")
            .expect("contract id must be valid");
        let definition = CapabilityDefinition::new(
            capability_id.clone(),
            runtime.engine_id().clone(),
            "Draining test capability",
        )
        .expect("definition must be valid");

        capabilities
            .register(
                definition,
                arc_handler(move |_context, invocation| {
                    calls_for_handler.fetch_add(1, Ordering::SeqCst);
                    Ok(CapabilityOutcome::new(invocation.payload_bytes().to_vec()))
                }),
            )
            .expect("capability registration should succeed");

        let invocation =
            CapabilityInvocation::new(capability_id, contract_id, b"draining".to_vec());
        let context = runtime.context(operation_context("draining"));

        runtime.drain().expect("draining should succeed");

        assert!(
            runtime
                .dispatch(&capabilities, &context, &invocation)
                .is_err()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn shutdown_resources_are_exposed_from_core_without_reimplementation() {
        let runtime = runtime();

        let _shutdown_token = runtime.shutdown_token();
        let _background_tasks = runtime.background_tasks();
    }

    #[test]
    fn shutdown_delegates_terminal_state_to_core() {
        let runtime = serving_runtime();

        runtime.shutdown().expect("shutdown should succeed");

        assert_eq!(runtime.state(), LifecycleState::Stopped);
    }
}
