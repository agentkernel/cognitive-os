//! Management-plane error surface.
//!
//! Denial codes are consumed from the GENERATED error-registry binding
//! (`cognitive_contracts::generated::error_registry`, digest-pinned to
//! `specs/registry/errors.yaml`) — this crate never hand-writes a code
//! table, so an unregistered code cannot be surfaced.

use cognitive_contracts::generated::common_defs::ErrorCategory;
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_kernel::effects::EffectError;
use cognitive_kernel::error::TransitionRejection;
use cognitive_kernel::ports::{PortFailure, StorePortError};
use cognitive_kernel::recovery::RecoveryError;

/// A deterministic management-gate denial carrying a registered code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{detail} (code {})", self.code.as_str())]
pub struct ManagementDenial {
    /// Registered machine code (category/retryable pinned by the registry
    /// binding).
    pub code: RegisteredErrorCode,
    /// Deterministic denial detail.
    pub detail: String,
}

impl ManagementDenial {
    /// Build a denial for a registered code.
    pub fn new(code: RegisteredErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    /// The registered code text.
    pub fn code_str(&self) -> &'static str {
        self.code.as_str()
    }

    /// The registered category.
    pub fn category(&self) -> ErrorCategory {
        self.code.entry().category
    }

    /// The registered retryable flag.
    pub fn retryable(&self) -> bool {
        self.code.entry().retryable
    }
}

/// Wire category text of a registered category (the serde rename values of
/// the generated enum; exhaustive so a new category cannot be forgotten).
pub fn category_str(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::State => "state",
        ErrorCategory::Context => "context",
        ErrorCategory::Auth => "auth",
        ErrorCategory::Effect => "effect",
        ErrorCategory::Protocol => "protocol",
        ErrorCategory::Resource => "resource",
        ErrorCategory::Profile => "profile",
        ErrorCategory::Agent => "agent",
        ErrorCategory::Memory => "memory",
        ErrorCategory::Catalog => "catalog",
        ErrorCategory::Semantic => "semantic",
        ErrorCategory::Discovery => "discovery",
        ErrorCategory::Knowledge => "knowledge",
        ErrorCategory::Performance => "performance",
        ErrorCategory::Shell => "shell",
        ErrorCategory::Intent => "intent",
        ErrorCategory::Lifecycle => "lifecycle",
        ErrorCategory::Watch => "watch",
    }
}

/// The registered parts every management failure maps onto for the wire
/// (`code`, `category`, `retryable`, `detail`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredParts {
    /// Registered code text.
    pub code: String,
    /// Registered category text.
    pub category: String,
    /// Registered retryable flag.
    pub retryable: bool,
    /// Deterministic detail.
    pub detail: String,
}

/// Failure surface of the deterministic management plane. Every variant
/// maps onto a registered code via [`ManagementError::registered_parts`];
/// nothing is silently converted to success (REQ-ERR-001).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ManagementError {
    /// Denied at the management gate before any read or write.
    #[error(transparent)]
    Denied(#[from] ManagementDenial),
    /// Rejected by the central transition gate.
    #[error(transparent)]
    Rejected(#[from] TransitionRejection),
    /// Failed inside the effect protocol.
    #[error(transparent)]
    Effect(#[from] EffectError),
    /// Failed inside the recovery sequence.
    #[error(transparent)]
    Recovery(#[from] RecoveryError),
    /// The authority store failed (fail closed).
    #[error(transparent)]
    Store(#[from] StorePortError),
    /// An infrastructure port (clock, id generation) failed (fail closed).
    #[error(transparent)]
    Port(#[from] PortFailure),
    /// The governance ledger could not be read or persisted (fail closed).
    #[error("governance-ledger: {0}")]
    Ledger(String),
}

impl ManagementError {
    /// Map this failure onto registered wire parts. Conflict-class store
    /// failures surface `STATE_CONFLICT`; unavailability (store, ports,
    /// ledger, replay barriers, order violations) surfaces
    /// `STATE_STORE_UNAVAILABLE` — both registered, both fail closed.
    pub fn registered_parts(&self) -> RegisteredParts {
        match self {
            Self::Denied(denial) => RegisteredParts {
                code: denial.code_str().to_owned(),
                category: category_str(denial.category()).to_owned(),
                retryable: denial.retryable(),
                detail: denial.detail.clone(),
            },
            Self::Rejected(rejection) => rejection_parts(rejection),
            Self::Effect(EffectError::Rejected(rejection)) => rejection_parts(rejection),
            Self::Effect(EffectError::Denied(denial)) => RegisteredParts {
                code: denial.registered.code.to_owned(),
                category: denial.registered.category.to_owned(),
                retryable: denial.registered.retryable,
                detail: denial.detail.clone(),
            },
            Self::Recovery(RecoveryError::Rejected(rejection)) => rejection_parts(rejection),
            Self::Recovery(RecoveryError::Protocol(EffectError::Rejected(rejection))) => {
                rejection_parts(rejection)
            }
            Self::Recovery(RecoveryError::Protocol(EffectError::Denied(denial))) => {
                RegisteredParts {
                    code: denial.registered.code.to_owned(),
                    category: denial.registered.category.to_owned(),
                    retryable: denial.registered.retryable,
                    detail: denial.detail.clone(),
                }
            }
            Self::Recovery(other) => registry_parts(
                RegisteredErrorCode::StateStoreUnavailable,
                format!("recovery failed closed: {other}"),
            ),
            Self::Store(StorePortError::Conflict { detail }) => registry_parts(
                RegisteredErrorCode::StateConflict,
                format!("store conflict: {detail}"),
            ),
            Self::Store(StorePortError::Unavailable { detail }) => registry_parts(
                RegisteredErrorCode::StateStoreUnavailable,
                format!("store unavailable: {detail}"),
            ),
            Self::Port(failure) => registry_parts(
                RegisteredErrorCode::StateStoreUnavailable,
                format!("port unavailable: {}", failure.detail),
            ),
            Self::Ledger(detail) => registry_parts(
                RegisteredErrorCode::StateStoreUnavailable,
                format!("governance ledger unavailable: {detail}"),
            ),
        }
    }
}

fn registry_parts(code: RegisteredErrorCode, detail: String) -> RegisteredParts {
    RegisteredParts {
        code: code.as_str().to_owned(),
        category: category_str(code.entry().category).to_owned(),
        retryable: code.entry().retryable,
        detail,
    }
}

fn rejection_parts(rejection: &TransitionRejection) -> RegisteredParts {
    let registered = rejection.registered();
    RegisteredParts {
        code: registered.code.to_owned(),
        category: registered.category.to_owned(),
        retryable: registered.retryable,
        detail: rejection.detail.clone(),
    }
}
