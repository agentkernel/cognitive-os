//! Fail-closed secret-store errors. Messages never embed secret material.

use std::fmt;

/// Deterministic secret-store failures. Display text is intentionally free of
/// secret bytes and of caller-supplied material that might itself be secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretError {
    /// No usable native secret backend is available.
    Unavailable { reason: &'static str },
    /// The collection exists but is locked and no interactive unlock is allowed.
    Locked,
    /// An interactive prompt would be required and daemon mode forbids it.
    PromptUnavailable,
    /// The opaque reference does not resolve to an item in this backend.
    NotFound,
    /// Attributes or label failed structural validation.
    InvalidAttributes { detail: &'static str },
    /// Backend rejected a replace/delete race or concurrent mutation.
    Conflict,
    /// Backend returned an unexpected failure without secret content.
    Backend { detail: &'static str },
}

impl fmt::Display for SecretError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { reason } => {
                write!(formatter, "secret store unavailable: {reason}")
            }
            Self::Locked => write!(
                formatter,
                "secret store locked: interactive unlock is not permitted for daemon use"
            ),
            Self::PromptUnavailable => write!(
                formatter,
                "secret store prompt unavailable: non-interactive daemon cannot complete unlock"
            ),
            Self::NotFound => write!(formatter, "secret ref not found"),
            Self::InvalidAttributes { detail } => {
                write!(formatter, "invalid secret attributes: {detail}")
            }
            Self::Conflict => write!(formatter, "secret store conflict"),
            Self::Backend { detail } => {
                write!(formatter, "secret store backend failure: {detail}")
            }
        }
    }
}

impl std::error::Error for SecretError {}
