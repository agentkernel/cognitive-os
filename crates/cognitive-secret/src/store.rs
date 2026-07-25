//! Frozen SecretStore surface for Personal daemon use (P0-T05).

use crate::error::SecretError;
use crate::material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};

/// Coarse availability reported by [`SecretStore::probe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStoreAvailability {
    /// Backend can serve put/get/delete without interactive prompts.
    Available,
    /// Backend is present but locked; daemon must fail closed.
    Locked,
    /// Backend would require an interactive prompt; daemon must fail closed.
    PromptUnavailable,
    /// Backend is absent or not usable on this host/session.
    Unavailable,
}

/// Deployment class of a backend. Production Personal must use Native only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStoreClass {
    /// Linux Secret Service / other OS-native secret store.
    Native,
    /// Process-local double used only for automated PoC tests.
    EphemeralTestDouble,
    /// Explicit fail-closed placeholder when no backend can be selected.
    Unavailable,
}

/// Daemon-facing secret port frozen by P0-T05.
///
/// `put` with matching attributes replaces existing material (rotate). There is
/// no plaintext fallback method on this trait.
pub trait SecretStore {
    /// Class of this backend implementation.
    fn class(&self) -> SecretStoreClass;

    /// Non-mutating readiness probe. Must not create items or log secrets.
    fn probe(&self) -> Result<SecretStoreAvailability, SecretError>;

    /// Insert or replace secret material identified by attributes.
    ///
    /// When an item with the same attributes already exists, the material is
    /// replaced and the same or a refreshed [`SecretRef`] is returned. This is
    /// the rotate path for Personal Provider keys.
    fn put(
        &self,
        label: &SecretLabel,
        attributes: &SecretAttributes,
        material: SecretMaterial,
    ) -> Result<SecretRef, SecretError>;

    /// Read secret material for an opaque ref. Missing refs fail closed.
    fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError>;

    /// Delete secret material for an opaque ref. Missing refs fail closed.
    fn delete(&self, secret_ref: &SecretRef) -> Result<(), SecretError>;
}

impl<T: SecretStore + ?Sized> SecretStore for &T {
    fn class(&self) -> SecretStoreClass {
        (**self).class()
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        (**self).probe()
    }

    fn put(
        &self,
        label: &SecretLabel,
        attributes: &SecretAttributes,
        material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        (**self).put(label, attributes, material)
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        (**self).get(secret_ref)
    }

    fn delete(&self, secret_ref: &SecretRef) -> Result<(), SecretError> {
        (**self).delete(secret_ref)
    }
}

impl<T: SecretStore + ?Sized> SecretStore for std::sync::Arc<T> {
    fn class(&self) -> SecretStoreClass {
        (**self).class()
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        (**self).probe()
    }

    fn put(
        &self,
        label: &SecretLabel,
        attributes: &SecretAttributes,
        material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        (**self).put(label, attributes, material)
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        (**self).get(secret_ref)
    }

    fn delete(&self, secret_ref: &SecretRef) -> Result<(), SecretError> {
        (**self).delete(secret_ref)
    }
}
