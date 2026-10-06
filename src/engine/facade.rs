//! Top-level Knowledge Graph engine facade.
//!
//! The facade composes the three Phase 0 engine boundaries:
//!
//! - [`super::registration`] for Core Control Plane registration.
//! - [`super::runtime`] for Core runtime and lifecycle integration.
//! - [`crate::contract::Capability`] for the minimal capability contract.
//!
//! The facade does not implement a second runtime, registry, lifecycle
//! machine, or universal request/response system. Those responsibilities
//! remain owned by `nizaam-core`.
//! Phase 0 scaffolding intentionally retains the complete Core integration surface
//! before the final crate-level public wiring is added.
//! The implementation is kept intact; this local allowance prevents intermediate
//! dead-code diagnostics from masking real errors.
#![allow(dead_code)]

use std::sync::Arc;

use nizaam_core::capability::{CapabilityDefinition, CapabilityHandler, CapabilityInvocation};
use nizaam_core::contracts::UniversalRequest;
use nizaam_core::control_plane::registry::EngineRegistry;
use nizaam_core::error::InvalidTransition;
use nizaam_core::identity::{CapabilityId, EngineId, EngineInstanceId};
use nizaam_core::runtime::{EngineContext, LifecycleState, RequestAdmissionError};

use super::registration::KgRegistration;
use super::runtime::{KgRuntime, RuntimeDispatchResult};
use crate::contract::Capability;
use crate::integration::core::{CoreCapabilities, capability_invocation};

/// Thin facade-level setup error.
///
/// Core lifecycle, Control Plane, and capability-registry errors remain
/// represented by their existing Core error types. The only local invariant
/// represented here is capability ownership by this engine.
#[derive(Debug)]
pub enum EngineSetupError {
    Lifecycle(InvalidTransition),
    Registry(nizaam_core::control_plane::registry::RegistryError),
    CapabilityRegistry(nizaam_core::capability::RegistryError),
    CapabilityOwnerMismatch {
        capability_id: CapabilityId,
        expected_engine: EngineId,
        actual_engine: EngineId,
    },
    Phase0CapabilityNotRegistered,
    EngineNotRegistered,
}

impl std::fmt::Display for EngineSetupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lifecycle(error) => error.fmt(formatter),

            Self::Registry(error) => error.fmt(formatter),

            Self::CapabilityRegistry(error) => error.fmt(formatter),

            Self::CapabilityOwnerMismatch {
                capability_id,
                expected_engine,
                actual_engine,
            } => write!(
                formatter,
                "capability `{capability_id}` belongs to engine `{actual_engine}`, \
                 expected `{expected_engine}`",
            ),

            Self::Phase0CapabilityNotRegistered => {
                write!(formatter, "the Phase 0 capability has not been registered")
            }

            Self::EngineNotRegistered => {
                write!(
                    formatter,
                    "the engine has not been registered with the Core Control Plane"
                )
            }
        }
    }
}

impl std::error::Error for EngineSetupError {}

impl From<InvalidTransition> for EngineSetupError {
    fn from(error: InvalidTransition) -> Self {
        Self::Lifecycle(error)
    }
}

impl From<nizaam_core::control_plane::registry::RegistryError> for EngineSetupError {
    fn from(error: nizaam_core::control_plane::registry::RegistryError) -> Self {
        Self::Registry(error)
    }
}

impl From<nizaam_core::capability::RegistryError> for EngineSetupError {
    fn from(error: nizaam_core::capability::RegistryError) -> Self {
        Self::CapabilityRegistry(error)
    }
}

/// Error produced while handling a universal request at the KG boundary.
///
/// Core remains authoritative for lifecycle admission. The target mismatch
/// variants enforce the local-engine routing invariant before capability
/// dispatch is reached.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestHandlingError {
    Admission(RequestAdmissionError),
    TargetEngineMismatch {
        expected: EngineId,
        actual: EngineId,
    },
    TargetInstanceMismatch {
        expected: EngineInstanceId,
        actual: EngineInstanceId,
    },
}

impl std::fmt::Display for RequestHandlingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admission(error) => error.fmt(formatter),
            Self::TargetEngineMismatch { expected, actual } => write!(
                formatter,
                "request target engine {} does not match local engine {}",
                actual.as_str(),
                expected.as_str(),
            ),
            Self::TargetInstanceMismatch { expected, actual } => write!(
                formatter,
                "request target instance {} does not match local instance {}",
                actual.as_str(),
                expected.as_str(),
            ),
        }
    }
}

impl std::error::Error for RequestHandlingError {}

impl From<RequestAdmissionError> for RequestHandlingError {
    fn from(error: RequestAdmissionError) -> Self {
        Self::Admission(error)
    }
}

/// Result returned by the public KG request boundary.
pub type UniversalRequestResult =
    Result<nizaam_core::capability::CapabilityDispatchResult, RequestHandlingError>;

/// Top-level Phase 0 Knowledge Graph engine facade.
///
/// This type composes Core-backed registration, runtime, and capability
/// boundaries without taking ownership of the mechanisms themselves.
#[derive(Debug)]
pub struct KgEngine {
    registration: KgRegistration,
    runtime: KgRuntime,
    capabilities: CoreCapabilities,
    engine_registered: bool,
    phase0_capability_registered: bool,
}

impl KgEngine {
    /// Creates a new KG engine facade in Core's `Created` state.
    #[must_use]
    pub fn new(engine_id: EngineId, engine_instance_id: EngineInstanceId) -> Self {
        Self {
            registration: KgRegistration::new(engine_id.clone(), engine_instance_id.clone()),
            runtime: KgRuntime::new(engine_id, engine_instance_id),
            capabilities: CoreCapabilities::new(),
            engine_registered: false,
            phase0_capability_registered: false,
        }
    }

    /// Returns the logical engine identity.
    #[must_use]
    pub fn engine_id(&self) -> &EngineId {
        self.registration.engine_id()
    }

    /// Returns the concrete engine-instance identity.
    #[must_use]
    pub fn engine_instance_id(&self) -> &EngineInstanceId {
        self.registration.engine_instance_id()
    }

    /// Returns the declarative Core-backed registration boundary.
    #[must_use]
    pub(crate) fn registration(&self) -> &KgRegistration {
        &self.registration
    }

    /// Returns the Core-backed runtime boundary.
    #[must_use]
    pub(crate) fn runtime(&self) -> &KgRuntime {
        &self.runtime
    }

    /// Returns the Core-backed capability integration boundary.
    #[must_use]
    pub(crate) fn capabilities(&self) -> &CoreCapabilities {
        &self.capabilities
    }

    /// Returns the current Core lifecycle state.
    #[must_use]
    pub fn state(&self) -> LifecycleState {
        self.runtime.state()
    }

    /// Advances the engine through the non-registration startup states.
    pub fn start(&self) -> Result<(), InvalidTransition> {
        self.runtime.start()
    }

    /// Enters the Core `Registering` lifecycle state.
    ///
    /// Actual registration remains a separate operation.
    pub fn begin_registration(&self) -> Result<(), InvalidTransition> {
        self.runtime.begin_registration()
    }

    /// Registers this concrete engine instance through the Core Control
    /// Plane registry.
    ///
    /// Registration is valid only while the Core runtime is in `Registering`.
    pub fn register_engine(&mut self, registry: &EngineRegistry) -> Result<(), EngineSetupError> {
        self.require_registering()?;

        let capability = Capability::definition(self.engine_id());
        self.registration = self
            .registration
            .clone()
            .with_capability(capability)
            .map_err(|error| EngineSetupError::Registry(error.into()))?;

        self.registration.register(registry)?;

        self.engine_registered = true;

        Ok(())
    }

    /// Registers one capability through the Core-backed capability boundary.
    ///
    /// Capability ownership is checked at the facade boundary because it is a
    /// KG engine composition invariant. Actual capability registration and
    /// dispatch remain Core-owned.
    pub fn register_capability(
        &mut self,
        definition: CapabilityDefinition,
        handler: Arc<dyn CapabilityHandler>,
    ) -> Result<(), EngineSetupError> {
        self.require_registering()?;

        if definition.owning_engine() != self.engine_id() {
            return Err(EngineSetupError::CapabilityOwnerMismatch {
                capability_id: definition.capability_id().clone(),
                expected_engine: self.engine_id().clone(),
                actual_engine: definition.owning_engine().clone(),
            });
        }

        self.capabilities.register(definition, handler)?;

        Ok(())
    }

    /// Registers the Phase 0 capability required by the bootstrap engine.
    ///
    /// The capability implementation itself remains defined by the contract
    /// layer. This facade only composes it into the Core capability boundary.
    pub fn register_phase0_capability(&mut self) -> Result<CapabilityId, EngineSetupError> {
        self.require_registering()?;

        let definition = Capability::definition(self.engine_id());
        let capability_id = definition.capability_id().clone();
        let handler = Capability::handler();

        self.register_capability(definition, handler)?;
        self.phase0_capability_registered = true;

        Ok(capability_id)
    }

    /// Marks the engine `Ready`.
    ///
    /// Phase 0 requires both engine registration and Phase 0 capability
    /// registration before the facade can enter Core's `Ready` state.
    pub fn mark_ready(&self) -> Result<(), EngineSetupError> {
        if !self.engine_registered {
            return Err(EngineSetupError::EngineNotRegistered);
        }

        if !self.phase0_capability_registered {
            return Err(EngineSetupError::Phase0CapabilityNotRegistered);
        }

        self.runtime.mark_ready()?;

        Ok(())
    }

    /// Enters Core's `Serving` state.
    pub fn serve(&self) -> Result<(), InvalidTransition> {
        self.runtime.serve()
    }

    /// Handles one universal request through the Core-backed runtime and
    /// capability boundaries.
    ///
    /// The facade does not interpret the universal request as a KG-specific
    /// query. Core remains responsible for request admission, capability
    /// resolution, cancellation, deadlines, and handler execution.
    pub fn handle_request(&self, request: &UniversalRequest) -> UniversalRequestResult {
        self.runtime.admit_request()?;

        let envelope = &request.universal_event().envelope;
        let participants = &envelope.metadata.participants;

        if participants.target != self.engine_id().clone() {
            return Err(RequestHandlingError::TargetEngineMismatch {
                expected: self.engine_id().clone(),
                actual: participants.target.clone(),
            });
        }

        if let Some(target_instance) = participants.target_instance.as_ref()
            && target_instance != self.engine_instance_id()
        {
            return Err(RequestHandlingError::TargetInstanceMismatch {
                expected: self.engine_instance_id().clone(),
                actual: target_instance.clone(),
            });
        }

        let context: EngineContext = self.runtime.context(envelope.operation_context.clone());
        let invocation = capability_invocation(request);

        self.dispatch_request(&context, &invocation)
            .map_err(RequestHandlingError::Admission)
    }

    /// Begins graceful draining through Core.
    pub fn drain(&self) -> Result<(), InvalidTransition> {
        self.runtime.drain()
    }

    /// Gracefully shuts down through Core.
    pub fn shutdown(&self) -> Result<bool, InvalidTransition> {
        self.runtime.shutdown()
    }

    fn dispatch_request(
        &self,
        context: &EngineContext,
        invocation: &CapabilityInvocation,
    ) -> RuntimeDispatchResult {
        self.runtime
            .dispatch(&self.capabilities, context, invocation)
    }

    /// Requires the Core runtime to be in `Registering`.
    fn require_registering(&self) -> Result<(), EngineSetupError> {
        if self.state() == LifecycleState::Registering {
            Ok(())
        } else {
            Err(EngineSetupError::Lifecycle(InvalidTransition::new(
                format!("{:?}", self.state()),
                format!("{:?}", LifecycleState::Registering),
            )))
        }
    }

    /// Returns whether the engine has completed the required Phase 0
    /// registration work.
    #[must_use]
    pub fn is_registered(&self) -> bool {
        self.engine_registered && self.phase0_capability_registered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_id() -> EngineId {
        EngineId::new("nizaam.kg.facade.test").expect("test engine id must be valid")
    }

    fn instance_id() -> EngineInstanceId {
        EngineInstanceId::new("nizaam.kg.facade.test.instance")
            .expect("test engine instance id must be valid")
    }

    #[test]
    fn construction_preserves_shared_engine_identity_and_created_state() {
        let engine = KgEngine::new(engine_id(), instance_id());

        assert_eq!(engine.engine_id(), &engine_id());
        assert_eq!(engine.engine_instance_id(), &instance_id());
        assert_eq!(engine.runtime().engine_id(), &engine_id());
        assert_eq!(engine.runtime().engine_instance_id(), &instance_id());
        assert_eq!(engine.state(), LifecycleState::Created);
        assert!(!engine.is_registered());
    }

    #[test]
    fn engine_registration_advertises_phase0_capability() {
        let mut engine = KgEngine::new(engine_id(), instance_id());
        engine.start().expect("startup should succeed");
        engine
            .begin_registration()
            .expect("registration state should be entered");

        let registry = EngineRegistry::new();
        engine
            .register_engine(&registry)
            .expect("engine registration should succeed");

        let record = registry
            .get(engine.engine_instance_id())
            .expect("registered engine should be present");

        let expected = Capability::definition(engine.engine_id());
        assert_eq!(record.registration().capabilities(), &[expected]);
    }

    #[test]
    fn registration_operations_are_rejected_before_registering_state() {
        let mut engine = KgEngine::new(engine_id(), instance_id());

        let registry = EngineRegistry::new();
        let result = engine.register_engine(&registry);

        assert!(matches!(result, Err(EngineSetupError::Lifecycle(_))));
    }

    #[test]
    fn phase0_capability_registration_requires_registering_state() {
        let mut engine = KgEngine::new(engine_id(), instance_id());

        let result = engine.register_phase0_capability();

        assert!(matches!(result, Err(EngineSetupError::Lifecycle(_))));
    }

    #[test]
    fn ready_requires_engine_registration() {
        let engine = KgEngine::new(engine_id(), instance_id());

        let result = engine.mark_ready();

        assert!(matches!(result, Err(EngineSetupError::EngineNotRegistered)));
    }

    #[test]
    fn ready_requires_phase0_capability_registration() {
        let mut engine = KgEngine::new(engine_id(), instance_id());
        engine.start().expect("startup should succeed");
        engine
            .begin_registration()
            .expect("registration state should be entered");

        let registry = EngineRegistry::new();
        engine
            .register_engine(&registry)
            .expect("engine registration should succeed");

        let result = engine.mark_ready();

        assert!(matches!(
            result,
            Err(EngineSetupError::Phase0CapabilityNotRegistered)
        ));
    }

    #[test]
    fn capability_owner_mismatch_is_rejected_at_the_facade_boundary() {
        let mut engine = KgEngine::new(engine_id(), instance_id());
        engine.start().expect("startup should succeed");
        engine
            .begin_registration()
            .expect("registration state should be entered");

        let foreign_engine =
            EngineId::new("nizaam.kg.foreign.test").expect("foreign engine id must be valid");
        let capability = CapabilityDefinition::new(
            CapabilityId::new("nizaam.kg.facade.foreign").expect("capability id must be valid"),
            foreign_engine.clone(),
            "Foreign capability",
        )
        .expect("capability definition must be valid");

        let result = engine.register_capability(capability, Capability::handler());

        assert!(matches!(
            result,
            Err(EngineSetupError::CapabilityOwnerMismatch {
                expected_engine,
                actual_engine,
                ..
            }) if expected_engine == engine_id() && actual_engine == foreign_engine
        ));
    }
}
