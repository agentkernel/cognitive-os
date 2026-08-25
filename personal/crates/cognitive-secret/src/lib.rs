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
//! - Provider configuration (P1-T02) may store only opaque `SecretRef` handles
//! - Provider discovery/probes (P1-T03) attach secret material only to egress headers
//! - the Windows Credential Manager backend (P7-T07) keeps the same fail-closed
//!   boundary; secret material transits only helper stdin/stdout, never argv

mod backend_select;
mod backends;
mod endpoint_trust;
mod error;
mod linux_secret_tool;
mod material;
mod provider_config;
mod provider_probe;
mod provider_service;
mod provider_snapshot;
mod provider_transport;
mod secret_input;
mod selected_model;
mod store;
mod windows_credential_manager;

pub use backend_select::{
    ProductionSecretBackend, select_production_secret_store, select_production_secret_store_with,
};
pub use backends::{
    EphemeralSecretStore, LinuxSecretServiceProbe, SecretServiceSimulation,
    SimulatedSecretServiceStore, UnavailableSecretStore,
};
pub use endpoint_trust::{
    ANTHROPIC_OFFICIAL_ENDPOINT, EndpointTrustError, EndpointTrustGrant, NetworkScope,
    OPENAI_OFFICIAL_ENDPOINT, ProviderKind, TrustedEndpoint, evaluate_resolved_targets,
    reject_caller_headers,
};
pub use error::SecretError;
pub use linux_secret_tool::LinuxSecretToolStore;
pub use material::{SecretAttributes, SecretLabel, SecretMaterial, SecretRef};
pub use provider_config::{
    PROVIDER_CONFIG_FILE_NAME, ProviderConfig, ProviderConfigError, ProviderConfigRepository,
};
pub use provider_probe::{
    DEFAULT_PROVIDER_EXCHANGE_TIMEOUT_MS, DEFAULT_PROVIDER_PROBE_BUDGET_MS, ModelDiscoveryResult,
    ModelSelection, ProviderDiscoveryService, ProviderProbeError, ProviderProbeOptions,
    ProviderReadinessSnapshot,
};
pub use provider_service::{
    ProviderKeyService, ProviderKeyServiceError, provider_account_secret_attributes,
    provider_secret_attributes, provider_secret_label,
};
pub use provider_snapshot::{
    DiscoveredModel, PROVIDER_PROBE_VERSION, PROVIDER_SNAPSHOT_SCHEMA_VERSION, ProbeErrorClass,
    ProbeOutcome, ProviderCapabilityFlags, ProviderCapabilitySnapshot, ProviderSnapshotError,
};
pub use provider_transport::{
    ANTHROPIC_API_KEY_HEADER_NAME, ANTHROPIC_API_VERSION, ANTHROPIC_VERSION_HEADER_NAME,
    AUTHORIZATION_HEADER_NAME, ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse,
    ProviderTransport, ProviderTransportError, StreamedProviderExchange,
    anthropic_api_key_header_value, bearer_authorization_header_value,
    is_authorization_header_name, redacted_headers,
};
pub use secret_input::read_secret_material_from_reader;
pub use selected_model::{
    SELECTED_MODEL_FILE_NAME, SelectedModel, SelectedModelError, SelectedModelRepository,
};
pub use store::{SecretStore, SecretStoreAvailability, SecretStoreClass};
pub use windows_credential_manager::WindowsCredentialManagerStore;
