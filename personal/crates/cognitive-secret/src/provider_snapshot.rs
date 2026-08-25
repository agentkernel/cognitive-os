//! Non-secret Provider capability snapshot (P1-T03).
//!
//! Snapshots record discovered models and active probe outcomes. They never
//! contain API keys, Authorization headers, or raw Provider response bodies.
//! The identity digest is product-local (not a registry digest or Profile claim).

use std::fmt;

/// Schema version embedded in capability snapshots.
pub const PROVIDER_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Probe implementation version. Bump when probe request shapes change.
pub const PROVIDER_PROBE_VERSION: u32 = 1;

/// One discovered model id from GET `/models`.
#[derive(Clone, PartialEq, Eq)]
pub struct DiscoveredModel {
    model_id: String,
}

impl DiscoveredModel {
    /// Validate and construct a discovered model id.
    pub fn new(model_id: impl Into<String>) -> Result<Self, ProviderSnapshotError> {
        let model_id = model_id.into();
        validate_model_id(&model_id)?;
        Ok(Self { model_id })
    }

    /// Borrow the model identifier.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }
}

impl fmt::Debug for DiscoveredModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DiscoveredModel")
            .field("model_id", &self.model_id)
            .finish()
    }
}

/// Outcome of a single active capability probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// Probe succeeded.
    Passed {
        /// Observed latency budget consumption in milliseconds (best-effort).
        latency_ms: u64,
    },
    /// Probe failed with a classified error.
    Failed {
        /// Stable error class for readiness projection.
        class: ProbeErrorClass,
        /// Static detail without response body content.
        detail: &'static str,
    },
    /// Probe intentionally not executed.
    Skipped {
        /// Static reason.
        reason: &'static str,
    },
}

impl ProbeOutcome {
    /// True when this outcome is [`ProbeOutcome::Passed`].
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed { .. })
    }
}

/// Stable classification of Provider probe failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeErrorClass {
    /// HTTP 401.
    Unauthorized,
    /// HTTP 403.
    Forbidden,
    /// HTTP 404.
    NotFound,
    /// HTTP 429.
    RateLimited,
    /// HTTP 5xx.
    ServerError,
    /// Soft timeout or cancel abort.
    Timeout,
    /// Response shape invalid without embedding body text.
    InvalidResponse,
    /// HTTP 200 but required capability (for example tool_calls) missing.
    CapabilityMissing,
    /// Network/TLS failure.
    Network,
    /// Local policy (HTTPS, model id, budget).
    Policy,
    /// Selected model is not present in the discovered catalog.
    AliasDrift,
}

impl ProbeErrorClass {
    /// Stable snake_case token for diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::RateLimited => "rate_limited",
            Self::ServerError => "server_error",
            Self::Timeout => "timeout",
            Self::InvalidResponse => "invalid_response",
            Self::CapabilityMissing => "capability_missing",
            Self::Network => "network",
            Self::Policy => "policy",
            Self::AliasDrift => "alias_drift",
        }
    }
}

/// Boolean capability flags derived from active probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProviderCapabilityFlags {
    /// Minimal non-streaming chat completion succeeded.
    pub chat: bool,
    /// Streaming chat response shape accepted.
    pub stream: bool,
    /// Tool-call candidate shape accepted (not executed as Effect).
    pub tool_call: bool,
    /// In-flight cancel / abort path observed.
    pub cancel: bool,
}

/// Durable, non-secret capability snapshot for one selected model.
#[derive(Clone, PartialEq, Eq)]
pub struct ProviderCapabilitySnapshot {
    provider_id: String,
    base_url: String,
    selected_model: String,
    observed_models: Vec<String>,
    manual_model_fallback: bool,
    probe_version: u32,
    capabilities: ProviderCapabilityFlags,
    chat: ProbeOutcome,
    stream: ProbeOutcome,
    tool_call: ProbeOutcome,
    cancel: ProbeOutcome,
}

impl ProviderCapabilitySnapshot {
    /// Construct a snapshot after probes complete.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: impl Into<String>,
        base_url: impl Into<String>,
        selected_model: impl Into<String>,
        observed_models: Vec<String>,
        manual_model_fallback: bool,
        chat: ProbeOutcome,
        stream: ProbeOutcome,
        tool_call: ProbeOutcome,
        cancel: ProbeOutcome,
    ) -> Result<Self, ProviderSnapshotError> {
        let provider_id = provider_id.into();
        validate_token(&provider_id, "provider_id")?;
        let base_url = base_url.into();
        if !base_url.to_ascii_lowercase().starts_with("https://") {
            return Err(ProviderSnapshotError::Invalid {
                detail: "snapshot base_url must use https://",
            });
        }
        let selected_model = selected_model.into();
        validate_model_id(&selected_model)?;
        for model_id in &observed_models {
            validate_model_id(model_id)?;
        }
        let capabilities = ProviderCapabilityFlags {
            chat: chat.is_passed(),
            stream: stream.is_passed(),
            tool_call: tool_call.is_passed(),
            cancel: cancel.is_passed(),
        };
        Ok(Self {
            provider_id,
            base_url,
            selected_model,
            observed_models,
            manual_model_fallback,
            probe_version: PROVIDER_PROBE_VERSION,
            capabilities,
            chat,
            stream,
            tool_call,
            cancel,
        })
    }

    /// Provider id from config.
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// HTTPS base URL used for probes.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Selected model id.
    pub fn selected_model(&self) -> &str {
        &self.selected_model
    }

    /// Observed catalog model ids (may be empty under manual fallback).
    pub fn observed_models(&self) -> &[String] {
        &self.observed_models
    }

    /// True when the selected model was supplied without catalog membership.
    pub fn manual_model_fallback(&self) -> bool {
        self.manual_model_fallback
    }

    /// Probe implementation version.
    pub fn probe_version(&self) -> u32 {
        self.probe_version
    }

    /// Derived capability flags.
    pub fn capabilities(&self) -> ProviderCapabilityFlags {
        self.capabilities
    }

    /// Chat probe outcome.
    pub fn chat(&self) -> &ProbeOutcome {
        &self.chat
    }

    /// Stream probe outcome.
    pub fn stream(&self) -> &ProbeOutcome {
        &self.stream
    }

    /// Tool-call probe outcome.
    pub fn tool_call(&self) -> &ProbeOutcome {
        &self.tool_call
    }

    /// Cancel probe outcome.
    pub fn cancel(&self) -> &ProbeOutcome {
        &self.cancel
    }

    /// True when chat probe passed. Other capabilities may still be false.
    pub fn is_minimally_ready(&self) -> bool {
        self.capabilities.chat
    }

    /// Canonical non-secret document used for the identity digest.
    pub fn to_canonical_document(&self) -> String {
        let observed = self
            .observed_models
            .iter()
            .map(|model_id| format!("\"{}\"", escape_json_string(model_id)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\
\"schema_version\":{schema},\
\"provider_id\":\"{provider_id}\",\
\"base_url\":\"{base_url}\",\
\"selected_model\":\"{selected_model}\",\
\"observed_models\":[{observed}],\
\"manual_model_fallback\":{manual},\
\"probe_version\":{probe_version},\
\"capabilities\":{{\"chat\":{chat},\"stream\":{stream},\"tool_call\":{tool_call},\"cancel\":{cancel}}},\
\"chat\":\"{chat_outcome}\",\
\"stream\":\"{stream_outcome}\",\
\"tool_call\":\"{tool_outcome}\",\
\"cancel\":\"{cancel_outcome}\"\
}}",
            schema = PROVIDER_SNAPSHOT_SCHEMA_VERSION,
            provider_id = escape_json_string(&self.provider_id),
            base_url = escape_json_string(&self.base_url),
            selected_model = escape_json_string(&self.selected_model),
            observed = observed,
            manual = if self.manual_model_fallback {
                "true"
            } else {
                "false"
            },
            probe_version = self.probe_version,
            chat = self.capabilities.chat,
            stream = self.capabilities.stream,
            tool_call = self.capabilities.tool_call,
            cancel = self.capabilities.cancel,
            chat_outcome = outcome_token(&self.chat),
            stream_outcome = outcome_token(&self.stream),
            tool_outcome = outcome_token(&self.tool_call),
            cancel_outcome = outcome_token(&self.cancel),
        )
    }

    /// Product-local identity digest of the canonical snapshot document.
    ///
    /// This is not a CognitiveOS registry digest, not RFC 8785, and not a
    /// Profile claim. It only identifies the selected snapshot in config.
    pub fn identity_digest(&self) -> String {
        fnv1a64_identity_digest(&self.to_canonical_document())
    }
}

impl fmt::Debug for ProviderCapabilitySnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderCapabilitySnapshot")
            .field("provider_id", &self.provider_id)
            .field("base_url", &self.base_url)
            .field("selected_model", &self.selected_model)
            .field("observed_models", &self.observed_models)
            .field("manual_model_fallback", &self.manual_model_fallback)
            .field("probe_version", &self.probe_version)
            .field("capabilities", &self.capabilities)
            .field("chat", &self.chat)
            .field("stream", &self.stream)
            .field("tool_call", &self.tool_call)
            .field("cancel", &self.cancel)
            .field("identity_digest", &self.identity_digest())
            .finish()
    }
}

/// Snapshot construction failures. Messages never embed secret material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderSnapshotError {
    /// Structural validation failed.
    Invalid { detail: &'static str },
}

impl fmt::Display for ProviderSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { detail } => write!(formatter, "invalid provider snapshot: {detail}"),
        }
    }
}

impl std::error::Error for ProviderSnapshotError {}

fn validate_model_id(model_id: &str) -> Result<(), ProviderSnapshotError> {
    if model_id.is_empty() || model_id.len() > 128 {
        return Err(ProviderSnapshotError::Invalid {
            detail: "model_id length out of range",
        });
    }
    if !model_id.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
    }) {
        return Err(ProviderSnapshotError::Invalid {
            detail: "model_id has unsupported characters",
        });
    }
    Ok(())
}

fn validate_token(token: &str, field: &'static str) -> Result<(), ProviderSnapshotError> {
    let _ = field;
    if token.is_empty() || token.len() > 64 {
        return Err(ProviderSnapshotError::Invalid {
            detail: "token length out of range",
        });
    }
    if !token
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ProviderSnapshotError::Invalid {
            detail: "token has unsupported characters",
        });
    }
    Ok(())
}

fn outcome_token(outcome: &ProbeOutcome) -> String {
    match outcome {
        ProbeOutcome::Passed { latency_ms } => format!("passed:{latency_ms}"),
        ProbeOutcome::Failed { class, detail } => {
            format!("failed:{}:{detail}", class.as_str())
        }
        ProbeOutcome::Skipped { reason } => format!("skipped:{reason}"),
    }
}

fn escape_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(escaped, "\\u{:04x}", u32::from(ch));
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

/// Domain-separated FNV-1a 64-bit identity digest for snapshot documents.
fn fnv1a64_identity_digest(document: &str) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in b"CognitiveOS-Personal-ProviderSnapshot-V1\0"
        .iter()
        .chain(document.as_bytes())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("fnv1a64:{hash:016x}")
}
