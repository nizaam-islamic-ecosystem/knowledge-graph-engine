//! Core Control Plane registration boundary for the Nizaam Knowledge Graph
//! engine.
//!
//! Phase 0 does not implement a second engine-registration system.
//!
//! `nizaam_core::control_plane::registration::EngineRegistration` remains the
//! authoritative declarative registration value, while
//! `nizaam_core::control_plane::registry::EngineRegistry` remains the
//! authoritative registry for concrete engine instances.
//!
//! This module only adapts those Core facilities for the Knowledge Graph
//! engine.
//! Phase 0 scaffolding intentionally retains the complete Core integration surface
//! before the final crate-level public wiring is added.
//! The implementation is kept intact; this local allowance prevents intermediate
//! dead-code diagnostics from masking real errors.
#![allow(dead_code)]

use crate::integration::core::CoreRegistration;
use nizaam_core::capability::CapabilityDefinition;

use nizaam_core::control_plane::registration::RegistrationResult;
use nizaam_core::control_plane::registry::{EngineRegistry, RegistryError};
use nizaam_core::identity::{EngineId, EngineInstanceId};

/// Knowledge Graph engine registration boundary over Core registration.
///
/// The underlying `EngineRegistration` remains owned by Core. This wrapper
/// exists only so the KG engine has an explicit integration boundary for its
/// registration composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KgRegistration {
    core: CoreRegistration,
}

impl KgRegistration {
    /// Creates a registration for one logical KG engine and one concrete
    /// runtime instance.
    #[must_use]
    pub(crate) fn new(engine_id: EngineId, engine_instance_id: EngineInstanceId) -> Self {
        Self {
            core: CoreRegistration::new(engine_id, engine_instance_id),
        }
    }

    /// Returns the logical engine identity owned by this registration.
    #[must_use]
    pub(crate) fn engine_id(&self) -> &EngineId {
        self.core.engine_id()
    }

    /// Returns the concrete engine-instance identity owned by this
    /// registration.
    #[must_use]
    pub(crate) fn engine_instance_id(&self) -> &EngineInstanceId {
        self.core.engine_instance_id()
    }

    /// Adds a capability advertisement through Core's registration model.
    ///
    /// Core validates that the advertised capability belongs to the same
    /// logical engine as this registration.
    pub(crate) fn with_capability(
        mut self,
        capability: CapabilityDefinition,
    ) -> RegistrationResult<Self> {
        self.core = self.core.with_capability(capability)?;
        Ok(self)
    }

    /// Validates the complete declarative registration through Core.
    ///
    /// Validation remains purely structural. Registry membership, health,
    /// routing, transport connectivity, authorization, and capability
    /// execution remain outside this operation.
    pub(crate) fn validate(&self) -> RegistrationResult<()> {
        self.core.validate()
    }

    /// Registers this concrete KG engine instance through the Core Control
    /// Plane registry.
    ///
    /// The registry remains responsible for membership of concrete engine
    /// instances. This adapter does not maintain a second local registry.
    pub(crate) fn register(&self, registry: &EngineRegistry) -> Result<(), RegistryError> {
        self.validate().map_err(RegistryError::from)?;

        registry.register(self.core.as_core().clone())
    }

    /// Returns the underlying Core registration.
    ///
    /// This is intentionally crate-visible so the engine composition layer can
    /// pass the Core-owned registration to other Core integration boundaries
    /// without introducing a KG-specific registration representation.
    #[must_use]
    pub(crate) fn as_core(&self) -> &nizaam_core::control_plane::registration::EngineRegistration {
        self.core.as_core()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nizaam_core::identity::CapabilityId;

    fn engine_id(value: &str) -> EngineId {
        EngineId::new(value).expect("test engine id must be valid")
    }

    fn instance_id(value: &str) -> EngineInstanceId {
        EngineInstanceId::new(value).expect("test engine instance id must be valid")
    }

    #[test]
    fn construction_preserves_core_engine_identity() {
        let engine = engine_id("nizaam.kg.registration.test");
        let instance = instance_id("nizaam.kg.registration.test.instance");
        let registration = KgRegistration::new(engine.clone(), instance.clone());

        assert_eq!(registration.engine_id(), &engine);
        assert_eq!(registration.engine_instance_id(), &instance);
    }

    #[test]
    fn empty_registration_is_structurally_valid() {
        let registration = KgRegistration::new(
            engine_id("nizaam.kg.registration.valid"),
            instance_id("nizaam.kg.registration.valid.instance"),
        );

        registration
            .validate()
            .expect("Core should accept the minimal registration");
    }

    #[test]
    fn capability_is_added_to_core_registration() {
        let engine = engine_id("nizaam.kg.registration.capability");
        let capability = CapabilityDefinition::new(
            CapabilityId::new("nizaam.kg.registration.capability.probe")
                .expect("capability id must be valid"),
            engine.clone(),
            "Registration test capability",
        )
        .expect("capability definition must be valid");

        let registration = KgRegistration::new(
            engine.clone(),
            instance_id("nizaam.kg.registration.capability.instance"),
        )
        .with_capability(capability.clone())
        .expect("Core should accept an owned capability");

        assert_eq!(registration.as_core().capabilities(), &[capability]);
    }

    #[test]
    fn foreign_capability_owner_is_rejected_by_core_registration() {
        let engine = engine_id("nizaam.kg.registration.owner");
        let foreign_engine = engine_id("nizaam.kg.registration.foreign");
        let capability = CapabilityDefinition::new(
            CapabilityId::new("nizaam.kg.registration.foreign.capability")
                .expect("capability id must be valid"),
            foreign_engine,
            "Foreign capability",
        )
        .expect("capability definition must be valid");

        let result =
            KgRegistration::new(engine, instance_id("nizaam.kg.registration.owner.instance"))
                .with_capability(capability);

        assert!(result.is_err());
    }

    #[test]
    fn register_adds_the_instance_to_the_core_registry() {
        let engine = engine_id("nizaam.kg.registration.registry");
        let instance = instance_id("nizaam.kg.registration.registry.instance");
        let registration = KgRegistration::new(engine, instance.clone());
        let registry = EngineRegistry::new();

        registration
            .register(&registry)
            .expect("Core registry registration should succeed");

        assert!(registry.contains(&instance));
        assert_eq!(registry.len(), 1);
    }
}
