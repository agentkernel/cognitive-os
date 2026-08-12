//! Production SecretStore selection for Personal (P1-T02; Windows in P7-T07).
//!
//! Production paths never select the ephemeral test double. When no native
//! backend is usable, selection fails closed via [`UnavailableSecretStore`].

use crate::backends::UnavailableSecretStore;
use crate::linux_secret_tool::LinuxSecretToolStore;
use crate::store::{SecretStore, SecretStoreAvailability, SecretStoreClass};
use crate::windows_credential_manager::WindowsCredentialManagerStore;

/// Result of production backend selection. Never includes an ephemeral double.
#[derive(Debug)]
pub enum ProductionSecretBackend {
    /// Linux `secret-tool` / FreeDesktop Secret Service path.
    LinuxSecretTool(LinuxSecretToolStore),
    /// Windows Credential Manager path (P7-T07).
    WindowsCredentialManager(WindowsCredentialManagerStore),
    /// Explicit fail-closed placeholder. No plaintext fallback exists.
    Unavailable(UnavailableSecretStore),
}

impl ProductionSecretBackend {
    /// Borrow the selected store as a trait object surface.
    pub fn as_secret_store(&self) -> &dyn SecretStore {
        match self {
            Self::LinuxSecretTool(store) => store,
            Self::WindowsCredentialManager(store) => store,
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
/// 3. On Windows, prefer [`WindowsCredentialManagerStore`] when probe reports
///    Available.
/// 4. Otherwise return [`UnavailableSecretStore`] (fail closed).
pub fn select_production_secret_store() -> ProductionSecretBackend {
    if cfg!(target_os = "linux") {
        return select_production_secret_store_with(true);
    }
    if cfg!(target_os = "windows") {
        let candidate = WindowsCredentialManagerStore::new();
        return match candidate.probe() {
            Ok(SecretStoreAvailability::Available) => {
                ProductionSecretBackend::WindowsCredentialManager(candidate)
            }
            Ok(_) | Err(_) => ProductionSecretBackend::Unavailable(UnavailableSecretStore),
        };
    }
    ProductionSecretBackend::Unavailable(UnavailableSecretStore)
}

/// Testable selection entry that overrides the Linux host signal.
///
/// `false` deliberately stays fail-closed `Unavailable` instead of falling
/// through to the Windows backend, preserving the frozen P1-T02 selection
/// contract for existing callers and tests.
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
