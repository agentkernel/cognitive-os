//! SecretStore backends for the P0-T05 PoC.
//!
//! Production Personal must use a native Secret Service backend. The ephemeral
//! store exists only as an automated test double and is never a plaintext
//! product fallback.

use crate::error::SecretError;
use crate::material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};
use crate::store::{SecretStore, SecretStoreAvailability, SecretStoreClass};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// Always-unavailable backend. Used when init cannot select a native store.
#[derive(Debug, Default)]
pub struct UnavailableSecretStore;

impl SecretStore for UnavailableSecretStore {
    fn class(&self) -> SecretStoreClass {
        SecretStoreClass::Unavailable
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        Ok(SecretStoreAvailability::Unavailable)
    }

    fn put(
        &self,
        _label: &SecretLabel,
        _attributes: &SecretAttributes,
        _material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        Err(SecretError::Unavailable {
            reason: "no native secret backend selected",
        })
    }

    fn get(&self, _secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        Err(SecretError::Unavailable {
            reason: "no native secret backend selected",
        })
    }

    fn delete(&self, _secret_ref: &SecretRef) -> Result<(), SecretError> {
        Err(SecretError::Unavailable {
            reason: "no native secret backend selected",
        })
    }
}

/// Simulated Secret Service readiness modes for fail-closed daemon tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretServiceSimulation {
    /// Full put/get/delete is permitted.
    Available,
    /// Service is absent from the session bus.
    ServiceAbsent,
    /// Collection is locked; interactive unlock is forbidden for daemons.
    CollectionLocked,
    /// Unlock would require a prompt that the daemon cannot complete.
    PromptUnavailable,
}

struct SimulatedItem {
    attributes: Vec<(String, String)>,
    material: Vec<u8>,
}

impl Drop for SimulatedItem {
    fn drop(&mut self) {
        for byte in &mut self.material {
            *byte = 0;
        }
        self.material.clear();
    }
}

/// Process-local Secret Service simulation.
///
/// This is an automated PoC/test double only (`SecretStoreClass::EphemeralTestDouble`).
/// It must not be selected as a production Personal backend and does not write
/// SQLite, config files, environment variables, or evidence.
pub struct SimulatedSecretServiceStore {
    mode: Mutex<SecretServiceSimulation>,
    items: Mutex<HashMap<String, SimulatedItem>>,
    next_id: AtomicU64,
}

impl SimulatedSecretServiceStore {
    /// Creates a store in the given readiness mode.
    pub fn new(mode: SecretServiceSimulation) -> Self {
        Self {
            mode: Mutex::new(mode),
            items: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// Test helper to switch readiness without rebuilding the store.
    pub fn set_mode(&self, mode: SecretServiceSimulation) -> Result<(), SecretError> {
        let mut guard = self.mode.lock().map_err(|_| SecretError::Backend {
            detail: "simulation mutex poisoned",
        })?;
        *guard = mode;
        Ok(())
    }

    fn require_available(&self) -> Result<(), SecretError> {
        let mode = *self.mode.lock().map_err(|_| SecretError::Backend {
            detail: "simulation mutex poisoned",
        })?;
        match mode {
            SecretServiceSimulation::Available => Ok(()),
            SecretServiceSimulation::ServiceAbsent => Err(SecretError::Unavailable {
                reason: "org.freedesktop.secrets is absent from the session bus",
            }),
            SecretServiceSimulation::CollectionLocked => Err(SecretError::Locked),
            SecretServiceSimulation::PromptUnavailable => Err(SecretError::PromptUnavailable),
        }
    }
}

impl Default for SimulatedSecretServiceStore {
    fn default() -> Self {
        Self::new(SecretServiceSimulation::Available)
    }
}

impl SecretStore for SimulatedSecretServiceStore {
    fn class(&self) -> SecretStoreClass {
        SecretStoreClass::EphemeralTestDouble
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        let mode = *self.mode.lock().map_err(|_| SecretError::Backend {
            detail: "simulation mutex poisoned",
        })?;
        Ok(match mode {
            SecretServiceSimulation::Available => SecretStoreAvailability::Available,
            SecretServiceSimulation::ServiceAbsent => SecretStoreAvailability::Unavailable,
            SecretServiceSimulation::CollectionLocked => SecretStoreAvailability::Locked,
            SecretServiceSimulation::PromptUnavailable => {
                SecretStoreAvailability::PromptUnavailable
            }
        })
    }

    fn put(
        &self,
        _label: &SecretLabel,
        attributes: &SecretAttributes,
        material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        self.require_available()?;
        let mut items = self.items.lock().map_err(|_| SecretError::Backend {
            detail: "simulation mutex poisoned",
        })?;
        let attr_pairs = attributes.pairs().to_vec();
        if let Some((existing_ref, existing)) = items
            .iter_mut()
            .find(|(_, item)| item.attributes == attr_pairs)
        {
            existing.material = material.expose_bytes().to_vec();
            return SecretRef::from_opaque(existing_ref.clone());
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let opaque = format!("ssv1:ephemeral/{id}");
        items.insert(
            opaque.clone(),
            SimulatedItem {
                attributes: attr_pairs,
                material: material.expose_bytes().to_vec(),
            },
        );
        SecretRef::from_opaque(opaque)
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        self.require_available()?;
        let items = self.items.lock().map_err(|_| SecretError::Backend {
            detail: "simulation mutex poisoned",
        })?;
        let item = items
            .get(secret_ref.as_str())
            .ok_or(SecretError::NotFound)?;
        SecretMaterial::from_bytes(item.material.clone())
    }

    fn delete(&self, secret_ref: &SecretRef) -> Result<(), SecretError> {
        self.require_available()?;
        let mut items = self.items.lock().map_err(|_| SecretError::Backend {
            detail: "simulation mutex poisoned",
        })?;
        if items.remove(secret_ref.as_str()).is_none() {
            return Err(SecretError::NotFound);
        }
        Ok(())
    }
}

/// Alias retained for task-card wording ("ephemeral" test double).
pub type EphemeralSecretStore = SimulatedSecretServiceStore;

/// Linux Secret Service environment probe without storing secrets.
///
/// This backend never implements a plaintext fallback. On non-Linux hosts it is
/// permanently unavailable. On Linux it only inspects session-bus environment
/// signals; live D-Bus put/get remains P1-T02 with a real native adapter.
#[derive(Debug, Default)]
pub struct LinuxSecretServiceProbe;

impl LinuxSecretServiceProbe {
    /// Probe-only constructor.
    pub fn new() -> Self {
        Self
    }

    fn linux_session_bus_present() -> bool {
        if let Ok(address) = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            && !address.trim().is_empty()
        {
            return true;
        }
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let bus_path = std::path::Path::new(&runtime_dir).join("bus");
            if bus_path.exists() {
                return true;
            }
        }
        false
    }
}

impl SecretStore for LinuxSecretServiceProbe {
    fn class(&self) -> SecretStoreClass {
        SecretStoreClass::Native
    }

    fn probe(&self) -> Result<SecretStoreAvailability, SecretError> {
        if !cfg!(target_os = "linux") {
            return Ok(SecretStoreAvailability::Unavailable);
        }
        if Self::linux_session_bus_present() {
            // Presence of a session bus is necessary but not sufficient for a
            // unlocked Secret Service collection. P0-T05 freezes fail-closed
            // probe semantics; mutating native I/O is owned by P1-T02.
            Ok(SecretStoreAvailability::Available)
        } else {
            Ok(SecretStoreAvailability::Unavailable)
        }
    }

    fn put(
        &self,
        _label: &SecretLabel,
        _attributes: &SecretAttributes,
        _material: SecretMaterial,
    ) -> Result<SecretRef, SecretError> {
        Err(SecretError::Unavailable {
            reason: "native Secret Service mutating adapter is deferred to P1-T02",
        })
    }

    fn get(&self, _secret_ref: &SecretRef) -> Result<SecretMaterial, SecretError> {
        Err(SecretError::Unavailable {
            reason: "native Secret Service mutating adapter is deferred to P1-T02",
        })
    }

    fn delete(&self, _secret_ref: &SecretRef) -> Result<(), SecretError> {
        Err(SecretError::Unavailable {
            reason: "native Secret Service mutating adapter is deferred to P1-T02",
        })
    }
}
