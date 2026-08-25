//! Typed domain-layer errors.
//!
//! These are pure validation errors: they carry no registered machine error
//! code by themselves. The deterministic kernel maps domain rejections to
//! codes registered in `specs/registry/errors.yaml` at its gate
//! (`cognitive_kernel::error`); this layer never invents codes
//! (`docs/standards/error-contract.md` section 2).

use crate::transitions::TableAssetError;

/// Pure validation failure raised by domain newtypes and table lookups.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    /// Value is not a lowercase canonical UUIDv7 (REQ-GOBJ-ID-001, ADR-0005).
    #[error("invalid-uuid-v7: {0}")]
    InvalidUuidV7(String),
    /// Value violates the state-name grammar `^[A-Z][A-Z0-9_]*$`
    /// (`state-transition-request.schema.json`).
    #[error("invalid-state-name: {0}")]
    InvalidStateName(String),
    /// Value violates the reason-code grammar `^[A-Z][A-Z0-9_]*$`.
    #[error("invalid-reason-code: {0}")]
    InvalidReasonCode(String),
    /// Value is not a usable non-empty URI reference.
    #[error("invalid-uri-ref: {0}")]
    InvalidUriRef(String),
    /// Value is not a canonical RFC 3339 UTC timestamp
    /// (`docs/standards/canonical-encoding-and-digest.md` section 6).
    #[error("invalid-wall-timestamp: {0}")]
    InvalidWallTimestamp(String),
    /// Version is outside the valid logical-version range.
    #[error("invalid-version: {0}")]
    InvalidVersion(i64),
    /// Version increment would leave the I-JSON safe-integer range.
    #[error("version-overflow: {0}")]
    VersionOverflow(i64),
    /// Not one of the five registered execution lifecycle domains.
    #[error("unknown-lifecycle-domain: {0}")]
    UnknownLifecycleDomain(String),
    /// An embedded transition-table asset failed to parse or validate.
    #[error(transparent)]
    TableAsset(#[from] TableAssetError),
}
