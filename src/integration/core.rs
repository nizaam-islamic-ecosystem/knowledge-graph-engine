//! Nizaam Core integration for the Knowledge Graph engine.
//!
//! This module is the concrete KG-to-Core adaptation boundary.
//!
//! Core remains authoritative for:
//! - runtime and lifecycle
//! - request admission
//! - execution context
//! - cancellation and deadlines
//! - capability registration and dispatch
//! - engine registration
//! - Control Plane coordination
//! - universal request/response contracts
//!
//! KG-specific modules consume these adapters instead of recreating any of
//! those Core mechanisms. Phase-specific integration adapters, such as the
//! Indexing boundary, remain separate modules and reuse these Core contracts.
//! The dead-code allowance is retained while later-phase consumers are wired
//! incrementally; it does not represent a second lifecycle or runtime.
#![allow(dead_code)]

use std::sync::Arc;

use nizaam_core::capability::registry::CapabilityRegistry;
use nizaam_core::capability::{
    CapabilityDefinition, CapabilityDispatchResult, CapabilityHandler, CapabilityInvocation,
    CapabilityOutcome, arc_handler,
};
use nizaam_core::contracts::{UniversalRequest, UniversalResponse};
use nizaam_core::control_plane::registration::{EngineRegistration, RegistrationResult};
use nizaam_core::control_plane::registry::{
    EngineRegistry, RegistryError as ControlPlaneRegistryError,
};
use nizaam_core::error::InvalidTransition;
use nizaam_core::identity::{CapabilityId, EngineId, EngineInstanceId};
use nizaam_core::operation::OperationContext;
use nizaam_core::runtime::{EngineContext, EngineRuntime, LifecycleState, RequestAdmissionError};

/// Result of Core-backed capability dispatch.
pub(crate) type CoreDispatchResult = Result<CapabilityDispatchResult, RequestAdmissionError>;

/// Core-backed capability registry adapter.
///
/// The registry itself is entirely owned by `nizaam-core`; this wrapper only
/// keeps the KG integration boundary explicit and exposes Core registration
/// and dispatch without introducing a second capability system.
#[derive(Debug)]
pub(crate) struct CoreCapabilities {
    inner: CapabilityRegistry,
}

impl CoreCapabilities {
    /// Creates an empty Core capability registry.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            inner: CapabilityRegistry::new(),
        }
    }

    /// Registers one capability through Core.
    pub(crate) fn register(
        &self,
        definition: CapabilityDefinition,
        handler: Arc<dyn CapabilityHandler>,
    ) -> Result<(), nizaam_core::capability::RegistryError> {
        self.inner.register(definition, handler)
    }

    /// Returns whether a capability is registered in Core.
    #[must_use]
    pub(crate) fn contains(&self, capability_id: &CapabilityId) -> bool {
        self.inner.contains(capability_id)
    }

    /// Returns the number of capabilities registered in Core.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }

    /// Dispatches through Core's canonical capability dispatcher.
    #[must_use]
    pub(crate) fn dispatch(
        &self,
        context: &EngineContext,
        invocation: &CapabilityInvocation,
    ) -> CapabilityDispatchResult {
        nizaam_core::capability::dispatch(&self.inner, context, invocation)
    }
}

/// Core-backed runtime adapter used by the Knowledge Graph engine.
///
/// This type owns no KG-specific lifecycle state. The contained
/// `EngineRuntime` remains the single source of truth for lifecycle,
/// admission, cancellation, deadlines, and shutdown.
#[derive(Debug)]
pub(crate) struct CoreRuntime {
    inner: EngineRuntime,
}

impl CoreRuntime {
    /// Creates a Core runtime for one KG engine instance.
    #[must_use]
    pub(crate) fn new(engine_id: EngineId, engine_instance_id: EngineInstanceId) -> Self {
        Self {
            inner: EngineRuntime::new(engine_id, engine_instance_id),
        }
    }

    /// Returns the logical engine identity owned by Core.
    #[must_use]
    pub(crate) fn engine_id(&self) -> &EngineId {
        self.inner.engine_id()
    }

    /// Returns the concrete engine-instance identity owned by Core.
    #[must_use]
    pub(crate) fn engine_instance_id(&self) -> &EngineInstanceId {
        self.inner.instance_id()
    }

    /// Returns the current Core lifecycle state.
    #[must_use]
    pub(crate) fn state(&self) -> LifecycleState {
        self.inner.state()
    }

    /// Returns the underlying Core runtime.
    ///
    /// KG lifecycle sequencing delegates through the dedicated lifecycle adapter
    /// while Core remains the owner of the actual runtime state machine.
    #[must_use]
    pub(crate) fn runtime(&self) -> &EngineRuntime {
        &self.inner
    }

    /// Applies a lifecycle transition through Core.
    pub(crate) fn transition(&self, next: LifecycleState) -> Result<(), InvalidTransition> {
        self.inner.transition(next)
    }

    /// Performs Core request admission.
    pub(crate) fn admit_request(&self) -> Result<(), RequestAdmissionError> {
        self.inner.admit_request()
    }

    /// Builds the Core execution context from an existing operation context.
    ///
    /// KG does not create a second cancellation, deadline, security, or
    /// provenance system.
    #[must_use]
    pub(crate) fn context(&self, operation: OperationContext) -> EngineContext {
        EngineContext::new(operation)
    }

    /// Gracefully shuts down through Core.
    pub(crate) fn shutdown(&self) -> Result<bool, InvalidTransition> {
        self.inner.shutdown()
    }

    /// Returns the Core-owned shutdown cancellation token.
    #[must_use]
    pub(crate) fn shutdown_token(&self) -> &nizaam_core::runtime::CancellationToken {
        self.inner.shutdown_token()
    }

    /// Returns the Core-owned background task manager.
    #[must_use]
    pub(crate) fn background_tasks(&self) -> &nizaam_core::runtime::BackgroundTasks {
        self.inner.background_tasks()
    }
}

/// Core-backed engine registration adapter.
///
/// The registration value is owned by Core. KG only supplies its engine
/// identity and advertised capabilities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoreRegistration {
    inner: EngineRegistration,
}

impl CoreRegistration {
    /// Creates a Core engine registration for one KG runtime instance.
    #[must_use]
    pub(crate) fn new(engine_id: EngineId, engine_instance_id: EngineInstanceId) -> Self {
        Self {
            inner: EngineRegistration::new(engine_id, engine_instance_id),
        }
    }

    /// Returns the logical engine identity.
    #[must_use]
    pub(crate) fn engine_id(&self) -> &EngineId {
        self.inner.engine_id()
    }

    /// Returns the concrete engine-instance identity.
    #[must_use]
    pub(crate) fn engine_instance_id(&self) -> &EngineInstanceId {
        self.inner.engine_instance_id()
    }

    /// Advertises one Core capability through Core's registration model.
    pub(crate) fn with_capability(
        mut self,
        capability: CapabilityDefinition,
    ) -> RegistrationResult<Self> {
        self.inner = self.inner.with_capability(capability)?;
        Ok(self)
    }

    /// Validates the Core registration.
    pub(crate) fn validate(&self) -> RegistrationResult<()> {
        self.inner.validate()
    }

    /// Returns the advertised Core capability definitions.
    #[must_use]
    pub(crate) fn capabilities(&self) -> &[CapabilityDefinition] {
        self.inner.capabilities()
    }

    /// Returns the underlying Core registration value.
    #[must_use]
    pub(crate) fn as_core(&self) -> &EngineRegistration {
        &self.inner
    }

    /// Registers this KG engine instance through the Core Control Plane
    /// registry.
    pub(crate) fn register(
        &self,
        registry: &EngineRegistry,
    ) -> Result<(), ControlPlaneRegistryError> {
        registry.register(self.inner.clone())
    }
}

/// Builds the Core capability handler used by Phase 0.
///
/// The handler intentionally treats the invocation payload as opaque and
/// returns it unchanged. Capability semantics remain outside this adapter.
#[must_use]
pub(crate) fn opaque_capability_handler() -> Arc<dyn CapabilityHandler> {
    arc_handler(
        |_context: &EngineContext, invocation: &CapabilityInvocation| {
            Ok(CapabilityOutcome::new(invocation.payload_bytes().to_vec()))
        },
    )
}

/// Creates a Core capability invocation from the universal request envelope.
///
/// Core remains the owner of the universal request contract. KG only adapts
/// the envelope into the Core capability invocation expected by dispatch.
#[must_use]
pub(crate) fn capability_invocation(request: &UniversalRequest) -> CapabilityInvocation {
    let envelope = &request.universal_event().envelope;

    CapabilityInvocation::new(
        envelope.metadata.descriptor.capability_id.clone(),
        envelope.metadata.descriptor.contract_id.clone(),
        envelope.payload.bytes().to_vec(),
    )
}

/// Converts a Core capability outcome into the response payload.
///
/// The response envelope itself remains a Core universal-contract concern and
/// is constructed by the engine boundary using the Core request/response API.
#[must_use]
pub(crate) fn outcome_payload(outcome: &CapabilityOutcome) -> Vec<u8> {
    outcome.as_bytes().to_vec()
}

/// Keeps the Core universal response type at the integration boundary without
/// introducing a KG-specific response abstraction.
pub(crate) type CoreResponse = UniversalResponse;

/// Keeps Core identity types at the integration boundary.
pub(crate) type CoreCapabilityId = CapabilityId;
