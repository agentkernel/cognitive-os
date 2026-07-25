//! Production SecretStore selection for Personal (P1-T02).
//!
//! Production paths never select the ephemeral test double. When no native
//! backend is usable, selection fails closed via [`UnavailableSecretStore`].

use crate::backends::UnavailableSecretStore;
use crate::linux_secret_tool::LinuxSecretToolStore;
use crate::store::{SecretStore, SecretStoreAvailability, SecretStoreClass};

/// Result of production backend selection. Never includes an ephemeral double.
#[derive(Debug)]
pub enum ProductionSecretBackend {
    /// Linux `secret-tool` / FreeDesktop Secret Service path.
    LinuxSecretTool(LinuxSecretToolStore),
    /// Explicit fail-closed placeholder. No plaintext fallback exists.
    Unavailable(UnavailableSecretStore),
}

impl ProductionSecretBackend {
    /// Borrow the selected store as a trait object surface.
    pub fn as_secret_store(&self) -> &dyn SecretStore {
        match self {
            Self::LinuxSecretTool(store) => store,
            Self::Unavailable(store) => store,
        }
    }

    /// Class of the selected backend.
    pub fn class(&self) -> SecretStoreClass {
        self.as_secret_store().class()
    }

    /// Probe the selected backend without mutating secrets.
    pub fn probe(&self) -> Result<SecretStoreAvailability, crate::error::SecretError> {
        self.as_secret_store().probe()
    }
}

/// Select a production SecretStore for the current host.
///
/// Rules:
/// 1. Never returns [`SecretStoreClass::EphemeralTestDouble`].
/// 2. On Linux, prefer [`LinuxSecretToolStore`] when probe reports Available.
/// 3. Otherwise return [`UnavailableSecretStore`] (fail closed).
pub fn select_production_secret_store() -> ProductionSecretBackend {
    select_production_secret_store_with(cfg!(target_os = "linux"))
}

/// Testable selection entry that overrides the Linux host signal.
pub fn select_production_secret_store_with(prefer_linux_native: bool) -> ProductionSecretBackend {
    if prefer_linux_native {
        let candidate = LinuxSecretToolStore::new();
        match candidate.probe() {
            Ok(SecretStoreAvailability::Available) => {
                return ProductionSecretBackend::LinuxSecretTool(candidate);
            }
            Ok(_) | Err(_) => {
                return ProductionSecretBackend::Unavailable(UnavailableSecretStore);
            }
        }
    }
    ProductionSecretBackend::Unavailable(UnavailableSecretStore)
}
