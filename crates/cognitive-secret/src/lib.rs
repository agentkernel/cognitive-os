//! Personal SecretStore port and fail-closed backends (P0-T05).
//!
//! This crate freezes the daemon-facing secret surface for CognitiveOS Personal:
//! [`SecretStore::{probe, put, get, delete}`] plus opaque [`SecretRef`]. It is a
//! product PoC, not a registry REQ, schema, transition, vector, or Profile claim.
//!
//! Hard boundaries:
//! - secrets are never stringified into logs, errors, Display, or Debug
//! - no plaintext fallback path exists when a backend is unavailable
//! - this crate is not an authority writer and never touches SQLite
//! - production Provider configuration remains P1-T02

mod backends;
mod error;
mod material;
mod store;

pub use backends::{
    EphemeralSecretStore, LinuxSecretServiceProbe, SecretServiceSimulation,
    SimulatedSecretServiceStore, UnavailableSecretStore,
};
pub use error::SecretError;
pub use material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};
pub use store::{SecretStore, SecretStoreAvailability, SecretStoreClass};
