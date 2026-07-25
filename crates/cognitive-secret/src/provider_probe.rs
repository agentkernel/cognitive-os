//! OpenAI-compatible Provider discovery and active capability probes (P1-T03).
//!
//! Resolves opaque secret material only long enough to attach a Bearer header
//! for egress. Never writes secret bytes to config, SQLite, env, argv, logs, or
//! snapshot digests. Tool-call probe success means the model returned a
//! candidate-shaped tool request; it is not Effect dispatch or Task completion.

use crate::material::SecretMaterial;
use crate::provider_config::ProviderConfig;
use crate::provider_service::{ProviderKeyService, ProviderKeyServiceError};
use crate::provider_snapshot::{
    DiscoveredModel, ProbeErrorClass, ProbeOutcome, ProviderCapabilitySnapshot,
    ProviderSnapshotError,
};
use crate::provider_transport::{
    ProviderHttpMethod, ProviderHttpRequest, ProviderHttpResponse, ProviderTransport,
    ProviderTransportError, bearer_authorization_header_value,
};
use crate::store::SecretStore;
use std::fmt;
use std::time::Instant;

/// Default soft budget for a full discovery + probe campaign (milliseconds).
pub const DEFAULT_PROVIDER_PROBE_BUDGET_MS: u32 = 60_000;

/// Soft budget for a single HTTP exchange (milliseconds).
pub const DEFAULT_PROVIDER_EXCHANGE_TIMEOUT_MS: u32 = 15_000;

/// Failures while discovering models or probing capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderProbeError {
    /// Provider is not configured or secret material cannot be resolved.
    Key(ProviderKeyServiceError),
    /// Snapshot construction failed.
    Snapshot(ProviderSnapshotError),
    /// Transport-level failure before HTTP status interpretation.
    Transport(ProviderTransportError),
    /// Classified Provider HTTP / capability failure.
    Classified {
        class: ProbeErrorClass,
        detail: &'static str,
    },
    /// Local policy rejected the operation.
    Policy { detail: &'static str },
}

impl fmt::Display for ProviderProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(error) => write!(formatter, "provider probe key failure: {error}"),
            Self::Snapshot(error) => write!(formatter, "provider probe snapshot failure: {error}"),
            Self::Transport(error) => {
                write!(formatter, "provider probe transport failure: {error}")
            }
            Self::Classified { class, detail } => {
                write!(
                    formatter,
                    "provider probe classified failure ({}): {detail}",
                    class.as_str()
                )
            }
            Self::Policy { detail } => write!(formatter, "provider probe policy: {detail}"),
        }
    }
}

impl std::error::Error for ProviderProbeError {}

impl From<ProviderKeyServiceError> for ProviderProbeError {
    fn from(error: ProviderKeyServiceError) -> Self {
        Self::Key(error)
    }
}

impl From<ProviderSnapshotError> for ProviderProbeError {
    fn from(error: ProviderSnapshotError) -> Self {
        Self::Snapshot(error)
    }
}

impl From<ProviderTransportError> for ProviderProbeError {
    fn from(error: ProviderTransportError) -> Self {
        Self::Transport(error)
    }
}

/// Selection of which model to probe after discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSelection {
    /// Use the first discovered model id.
    FirstDiscovered,
    /// Require the model id to appear in the discovered catalog.
    ExactCatalog {
        /// Requested model id.
        model_id: String,
    },
    /// Allow probing a model id even when the catalog is empty or missing it.
    ManualFallback {
        /// Operator-supplied model id.
        model_id: String,
    },
}

/// Options for a discovery + probe campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderProbeOptions {
    /// Total soft budget for the campaign.
    pub budget_ms: u32,
    /// Per-exchange soft timeout.
    pub exchange_timeout_ms: u32,
    /// Model selection policy.
    pub selection: ModelSelection,
}

impl Default for ProviderProbeOptions {
    fn default() -> Self {
        Self {
            budget_ms: DEFAULT_PROVIDER_PROBE_BUDGET_MS,
            exchange_timeout_ms: DEFAULT_PROVIDER_EXCHANGE_TIMEOUT_MS,
            selection: ModelSelection::FirstDiscovered,
        }
    }
}

/// Result of listing models without probing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDiscoveryResult {
    /// Discovered models in catalog order.
    pub models: Vec<DiscoveredModel>,
}

/// Full readiness snapshot after discovery and active probes.
#[derive(Clone, PartialEq, Eq)]
pub struct ProviderReadinessSnapshot {
    /// Capability snapshot (non-secret).
    pub snapshot: ProviderCapabilitySnapshot,
    /// Product-local identity digest persisted into Provider config.
    pub snapshot_digest: String,
}

impl fmt::Debug for ProviderReadinessSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderReadinessSnapshot")
            .field("snapshot", &self.snapshot)
            .field("snapshot_digest", &self.snapshot_digest)
            .finish()
    }
}

/// Discovers models and runs active capability probes via an injected transport.
pub struct ProviderDiscoveryService<'a, S, T> {
    key_service: &'a ProviderKeyService<S>,
    transport: T,
}

impl<'a, S, T> fmt::Debug for ProviderDiscoveryService<'a, S, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderDiscoveryService")
            .field("key_service", self.key_service)
            .field("transport", &"<injected-transport>")
            .finish()
    }
}

impl<'a, S: SecretStore, T: ProviderTransport> ProviderDiscoveryService<'a, S, T> {
    /// Construct a discovery service over config/secret binding and transport.
    pub fn new(key_service: &'a ProviderKeyService<S>, transport: T) -> Self {
        Self {
            key_service,
            transport,
        }
    }

    /// GET `/models` and parse the OpenAI-compatible catalog.
    pub fn list_models(&self) -> Result<ModelDiscoveryResult, ProviderProbeError> {
        let config = require_config(self.key_service)?;
        let material = self.key_service.resolve_provider_material()?;
        let response = self.authorized_exchange(
            &config,
            &material,
            ProviderHttpMethod::Get,
            models_url(config.base_url()),
            None,
            DEFAULT_PROVIDER_EXCHANGE_TIMEOUT_MS,
            false,
        )?;
        let models = parse_models_catalog(&response)?;
        Ok(ModelDiscoveryResult { models })
    }

    /// Discover models, select one, run chat/stream/tool/cancel probes, and
    /// persist the non-secret snapshot digest into Provider config.
    pub fn discover_probe_and_persist(
        &self,
        options: &ProviderProbeOptions,
    ) -> Result<ProviderReadinessSnapshot, ProviderProbeError> {
        let campaign_started = Instant::now();
        let config = require_config(self.key_service)?;
        let material = self.key_service.resolve_provider_material()?;

        let discovery_response = self.authorized_exchange(
            &config,
            &material,
            ProviderHttpMethod::Get,
            models_url(config.base_url()),
            None,
            options.exchange_timeout_ms,
            false,
        );

        let (observed_models, manual_fallback, selected_model) =
            resolve_selection(options, discovery_response)?;

        ensure_budget(campaign_started, options.budget_ms)?;
        let chat = self.probe_chat(
            &config,
            &material,
            &selected_model,
            options.exchange_timeout_ms,
        );
        ensure_budget(campaign_started, options.budget_ms)?;
        let stream = self.probe_stream(
            &config,
            &material,
            &selected_model,
            options.exchange_timeout_ms,
        );
        ensure_budget(campaign_started, options.budget_ms)?;
        let tool_call = self.probe_tool_call(
            &config,
            &material,
            &selected_model,
            options.exchange_timeout_ms,
        );
        ensure_budget(campaign_started, options.budget_ms)?;
        let cancel = self.probe_cancel(
            &config,
            &material,
            &selected_model,
            options.exchange_timeout_ms,
        );

        let snapshot = ProviderCapabilitySnapshot::new(
            config.provider_id(),
            config.base_url(),
            selected_model,
            observed_models,
            manual_fallback,
            chat,
            stream,
            tool_call,
            cancel,
        )?;
        let snapshot_digest = snapshot.identity_digest();
        self.key_service
            .persist_selected_snapshot_digest(Some(snapshot_digest.clone()))?;
        Ok(ProviderReadinessSnapshot {
            snapshot,
            snapshot_digest,
        })
    }

    fn probe_chat(
        &self,
        config: &ProviderConfig,
        material: &SecretMaterial,
        model_id: &str,
        timeout_ms: u32,
    ) -> ProbeOutcome {
        let body = chat_completion_body(model_id, false, false);
        match self.authorized_exchange(
            config,
            material,
            ProviderHttpMethod::Post,
            chat_completions_url(config.base_url()),
            Some(body.into_bytes()),
            timeout_ms,
            false,
        ) {
            Ok(response) => interpret_chat_response(&response, false),
            Err(error) => outcome_from_probe_error(error),
        }
    }

    fn probe_stream(
        &self,
        config: &ProviderConfig,
        material: &SecretMaterial,
        model_id: &str,
        timeout_ms: u32,
    ) -> ProbeOutcome {
        let body = chat_completion_body(model_id, true, false);
        match self.authorized_exchange(
            config,
            material,
            ProviderHttpMethod::Post,
            chat_completions_url(config.base_url()),
            Some(body.into_bytes()),
            timeout_ms,
            false,
        ) {
            Ok(response) => interpret_chat_response(&response, true),
            Err(error) => outcome_from_probe_error(error),
        }
    }

    fn probe_tool_call(
        &self,
        config: &ProviderConfig,
        material: &SecretMaterial,
        model_id: &str,
        timeout_ms: u32,
    ) -> ProbeOutcome {
        let body = chat_completion_body(model_id, false, true);
        match self.authorized_exchange(
            config,
            material,
            ProviderHttpMethod::Post,
            chat_completions_url(config.base_url()),
            Some(body.into_bytes()),
            timeout_ms,
            false,
        ) {
            Ok(response) => interpret_tool_call_response(&response),
            Err(error) => outcome_from_probe_error(error),
        }
    }

    fn probe_cancel(
        &self,
        config: &ProviderConfig,
        material: &SecretMaterial,
        model_id: &str,
        timeout_ms: u32,
    ) -> ProbeOutcome {
        let body = chat_completion_body(model_id, false, false);
        match self.authorized_exchange(
            config,
            material,
            ProviderHttpMethod::Post,
            chat_completions_url(config.base_url()),
            Some(body.into_bytes()),
            timeout_ms,
            true,
        ) {
            Ok(_response) => ProbeOutcome::Failed {
                class: ProbeErrorClass::CapabilityMissing,
                detail: "cancel probe completed without abort",
            },
            Err(ProviderProbeError::Transport(ProviderTransportError::Timeout)) => {
                ProbeOutcome::Passed { latency_ms: 0 }
            }
            Err(error) => outcome_from_probe_error(error),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn authorized_exchange(
        &self,
        config: &ProviderConfig,
        material: &SecretMaterial,
        method: ProviderHttpMethod,
        url: String,
        body: Option<Vec<u8>>,
        timeout_ms: u32,
        cancel_requested: bool,
    ) -> Result<ProviderHttpResponse, ProviderProbeError> {
        if !config
            .base_url()
            .to_ascii_lowercase()
            .starts_with("https://")
        {
            return Err(ProviderProbeError::Policy {
                detail: "provider base_url must use https://",
            });
        }
        let authorization = bearer_authorization_header_value(material.expose_bytes())?;
        let request = ProviderHttpRequest {
            method,
            url,
            headers: vec![
                ("Authorization".to_owned(), authorization),
                ("Content-Type".to_owned(), "application/json".to_owned()),
                ("Accept".to_owned(), "application/json".to_owned()),
            ],
            body,
            timeout_ms,
            cancel_requested,
        };
        let started = Instant::now();
        let response = self.transport.exchange(&request)?;
        let _elapsed = started.elapsed();
        Ok(response)
    }
}

fn require_config<S: SecretStore>(
    key_service: &ProviderKeyService<S>,
) -> Result<ProviderConfig, ProviderProbeError> {
    key_service.load_config()?.ok_or(ProviderProbeError::Key(
        ProviderKeyServiceError::NotConfigured,
    ))
}

fn resolve_selection(
    options: &ProviderProbeOptions,
    discovery_response: Result<ProviderHttpResponse, ProviderProbeError>,
) -> Result<(Vec<String>, bool, String), ProviderProbeError> {
    match discovery_response {
        Ok(response) => {
            let models = parse_models_catalog(&response)?;
            let observed: Vec<String> = models
                .iter()
                .map(|model| model.model_id().to_owned())
                .collect();
            match &options.selection {
                ModelSelection::FirstDiscovered => {
                    let selected =
                        observed
                            .first()
                            .cloned()
                            .ok_or(ProviderProbeError::Classified {
                                class: ProbeErrorClass::NotFound,
                                detail: "provider model catalog is empty",
                            })?;
                    Ok((observed, false, selected))
                }
                ModelSelection::ExactCatalog { model_id } => {
                    if !observed.iter().any(|item| item == model_id) {
                        return Err(ProviderProbeError::Classified {
                            class: ProbeErrorClass::AliasDrift,
                            detail: "requested model is absent from discovered catalog",
                        });
                    }
                    Ok((observed, false, model_id.clone()))
                }
                ModelSelection::ManualFallback { model_id } => {
                    let manual = !observed.iter().any(|item| item == model_id);
                    Ok((observed, manual, model_id.clone()))
                }
            }
        }
        Err(error) => match &options.selection {
            ModelSelection::ManualFallback { model_id } => {
                // Catalog failures may still allow an explicit operator fallback.
                let _ = error;
                Ok((Vec::new(), true, model_id.clone()))
            }
            _ => Err(error),
        },
    }
}

fn ensure_budget(started: Instant, budget_ms: u32) -> Result<(), ProviderProbeError> {
    let elapsed_ms = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
    if elapsed_ms > budget_ms {
        return Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::Timeout,
            detail: "provider probe campaign exceeded soft budget",
        });
    }
    Ok(())
}

fn models_url(base_url: &str) -> String {
    join_url(base_url, "/models")
}

fn chat_completions_url(base_url: &str) -> String {
    join_url(base_url, "/chat/completions")
}

fn join_url(base_url: &str, path: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    format!("{trimmed}{path}")
}

fn chat_completion_body(model_id: &str, stream: bool, with_tools: bool) -> String {
    let tools = if with_tools {
        ",\"tools\":[{\"type\":\"function\",\"function\":{\"name\":\"cognitiveos_probe_noop\",\"description\":\"capability probe only\",\"parameters\":{\"type\":\"object\",\"properties\":{}}}}],\"tool_choice\":\"auto\""
    } else {
        ""
    };
    format!(
        "{{\"model\":\"{model}\",\"messages\":[{{\"role\":\"user\",\"content\":\"cognitiveos-provider-probe\"}}],\"max_tokens\":1,\"stream\":{stream}{tools}}}",
        model = escape_json(model_id),
        stream = if stream { "true" } else { "false" },
        tools = tools,
    )
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn parse_models_catalog(
    response: &ProviderHttpResponse,
) -> Result<Vec<DiscoveredModel>, ProviderProbeError> {
    classify_http_status(response.status)?;
    let body = std::str::from_utf8(&response.body).map_err(|_| ProviderProbeError::Classified {
        class: ProbeErrorClass::InvalidResponse,
        detail: "models response body is not utf-8",
    })?;
    // Minimal OpenAI-compatible extraction: collect every "id":"..." under data.
    let mut models = Vec::new();
    let mut search_from = 0usize;
    while let Some(relative) = body[search_from..].find("\"id\"") {
        let absolute = search_from + relative;
        let after_id = &body[absolute + 4..];
        let Some(after_colon) = after_id.split_once(':').map(|(_, rest)| rest.trim_start()) else {
            break;
        };
        if !after_colon.starts_with('"') {
            search_from = absolute + 4;
            continue;
        }
        let mut decoded = String::new();
        let mut chars = after_colon[1..].chars();
        let mut closed = false;
        while let Some(character) = chars.next() {
            match character {
                '"' => {
                    closed = true;
                    break;
                }
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        decoded.push(escaped);
                    }
                }
                other => decoded.push(other),
            }
        }
        if closed
            && !decoded.is_empty()
            && let Ok(model) = DiscoveredModel::new(decoded)
        {
            let already_present = models
                .iter()
                .any(|existing: &DiscoveredModel| existing.model_id() == model.model_id());
            if !already_present {
                models.push(model);
            }
        }
        search_from = absolute + 4;
    }
    Ok(models)
}

fn interpret_chat_response(response: &ProviderHttpResponse, stream: bool) -> ProbeOutcome {
    if let Err(error) = classify_http_status(response.status) {
        return outcome_from_probe_error(error);
    }
    let Ok(body) = std::str::from_utf8(&response.body) else {
        return ProbeOutcome::Failed {
            class: ProbeErrorClass::InvalidResponse,
            detail: "chat response body is not utf-8",
        };
    };
    if stream {
        if body.contains("data:") || body.contains("\"choices\"") {
            ProbeOutcome::Passed { latency_ms: 0 }
        } else {
            ProbeOutcome::Failed {
                class: ProbeErrorClass::InvalidResponse,
                detail: "stream response missing data frames",
            }
        }
    } else if body.contains("\"choices\"") {
        ProbeOutcome::Passed { latency_ms: 0 }
    } else {
        ProbeOutcome::Failed {
            class: ProbeErrorClass::InvalidResponse,
            detail: "chat response missing choices",
        }
    }
}

fn interpret_tool_call_response(response: &ProviderHttpResponse) -> ProbeOutcome {
    if let Err(error) = classify_http_status(response.status) {
        return outcome_from_probe_error(error);
    }
    let Ok(body) = std::str::from_utf8(&response.body) else {
        return ProbeOutcome::Failed {
            class: ProbeErrorClass::InvalidResponse,
            detail: "tool-call response body is not utf-8",
        };
    };
    if body.contains("\"tool_calls\"") || body.contains("\"function_call\"") {
        // Candidate-only: success means the model requested a tool shape.
        // It does not dispatch an Effect or complete a Task.
        ProbeOutcome::Passed { latency_ms: 0 }
    } else if body.contains("\"choices\"") {
        ProbeOutcome::Failed {
            class: ProbeErrorClass::CapabilityMissing,
            detail: "http 200 chat response lacked tool_calls",
        }
    } else {
        ProbeOutcome::Failed {
            class: ProbeErrorClass::InvalidResponse,
            detail: "tool-call response missing choices",
        }
    }
}

fn classify_http_status(status: u16) -> Result<(), ProviderProbeError> {
    match status {
        200..=299 => Ok(()),
        401 => Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::Unauthorized,
            detail: "provider returned http 401",
        }),
        403 => Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::Forbidden,
            detail: "provider returned http 403",
        }),
        404 => Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::NotFound,
            detail: "provider returned http 404",
        }),
        429 => Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::RateLimited,
            detail: "provider returned http 429",
        }),
        500..=599 => Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::ServerError,
            detail: "provider returned http 5xx",
        }),
        _ => Err(ProviderProbeError::Classified {
            class: ProbeErrorClass::InvalidResponse,
            detail: "provider returned unexpected http status",
        }),
    }
}

fn outcome_from_probe_error(error: ProviderProbeError) -> ProbeOutcome {
    match error {
        ProviderProbeError::Transport(ProviderTransportError::Timeout) => ProbeOutcome::Failed {
            class: ProbeErrorClass::Timeout,
            detail: "provider transport timeout",
        },
        ProviderProbeError::Transport(ProviderTransportError::Network { .. }) => {
            ProbeOutcome::Failed {
                class: ProbeErrorClass::Network,
                detail: "provider transport network failure",
            }
        }
        ProviderProbeError::Transport(ProviderTransportError::Policy { detail }) => {
            ProbeOutcome::Failed {
                class: ProbeErrorClass::Policy,
                detail,
            }
        }
        ProviderProbeError::Transport(ProviderTransportError::Backend { detail }) => {
            ProbeOutcome::Failed {
                class: ProbeErrorClass::Network,
                detail,
            }
        }
        ProviderProbeError::Classified { class, detail } => ProbeOutcome::Failed { class, detail },
        ProviderProbeError::Policy { detail } => ProbeOutcome::Failed {
            class: ProbeErrorClass::Policy,
            detail,
        },
        ProviderProbeError::Key(_) => ProbeOutcome::Failed {
            class: ProbeErrorClass::Policy,
            detail: "provider key resolution failed",
        },
        ProviderProbeError::Snapshot(_) => ProbeOutcome::Failed {
            class: ProbeErrorClass::Policy,
            detail: "provider snapshot construction failed",
        },
    }
}
