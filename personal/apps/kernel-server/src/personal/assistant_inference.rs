//! P13-T03 hidden Pi Assistant real inference (daemon side).
//!
//! `assistant.turn` explain / navigate / research / propose no longer echo the
//! client payload as a "candidate". The daemon assembles a bounded Context in
//! T10 inject order, invokes the exact pinned Pi once through
//! `pi-agent-adapter assistant-turn`, forwards Pi's single completion to the
//! bound Provider through the one-shot private completion socket (P8-T13
//! `agent://personal/pi` binding, key never leaves the daemon), parses Pi's
//! final text into a closed candidate object chain with typed provenance, and
//! only then registers the candidate through the v26 store path.
//!
//! When no Provider is bound the route answers with a Settings pointer and
//! registers nothing; the create-page chat renders that pointer instead of a
//! chat box. Research fetches go only through the registered
//! `HttpFetchReadOnly` pre-validator against the daemon's pinned origins.
//! Nothing here writes authority, SecretStore, archive, or Memory.

use std::path::{Path, PathBuf};

use cognitive_provider_transport::{
    ReadOnlyFetchMethod, ReadOnlyFetchRequest, ReadOnlyFetchTransport, RustlsReadOnlyFetchTransport,
};
use cognitive_runtime::{
    ASSISTANT_CONTEXT_BUDGET_BYTES, ASSISTANT_RESEARCH_EXCERPT_BYTES,
    ASSISTANT_RESEARCH_MAX_TARGETS, ASSISTANT_RESEARCH_RESPONSE_LIMIT, ASSISTANT_RESEARCH_TASK_REF,
    ASSISTANT_RESEARCH_TIMEOUT_MS, AssistantContextLayer, AssistantInferenceRequest,
    ProviderBindingState, assemble_bounded_context, parse_assistant_object_chain,
    validate_assistant_inference_request, validate_research_target,
};
use cognitive_secret::{ProviderConfigRepository, SelectedModelRepository};
use cognitive_store::{
    ASSISTANT_ENGINE_ID, ASSISTANT_INFERENCE_PROTOCOL, ASSISTANT_PI_PIN,
    ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL, ASSISTANT_RESEARCH_FETCH_FAMILY,
    ASSISTANT_SETTINGS_ROUTE, AssistantInferenceRecord, AssistantPlane, AssistantTurnSpec,
    ProjectAggregateStore, SqliteAuthorityStore, provider_unbound_guidance,
    reject_closed_candidate_schema,
};
use serde_json::{Value, json};

use super::project_aggregate::{error, now_ms, ok, parse_json, store_error};
use super::provider_control_plane::{PI_AGENT, selected_binding_model};
use super::resource_api::ResourceApiResponse;

/// Bounded number of conversation archive refs rendered into the `summary`
/// Context layer.
const SUMMARY_REF_LIMIT: usize = 16;

/// What the daemon observed while exact Pi produced one turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservedInference {
    /// Pi's final assistant text, untrusted until parsed on the daemon side.
    pub assistant_text: String,
    /// Model the adapter saw in Pi's events, informational only.
    pub response_model: Option<String>,
    /// Completions the daemon forwarded through its private proxy.
    pub provider_round_trips: u32,
}

/// Why the daemon could not obtain an inference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InferenceFailure {
    /// Pi runtime is not configured or unusable on this daemon host.
    PiUnavailable(String),
    /// The invocation ran and failed (adapter, Pi, or Provider proxy). Only the
    /// Unix production path can reach this; Windows stays `PiUnavailable`.
    #[cfg_attr(not(unix), allow(dead_code))]
    Failed(String),
}

/// Daemon-owned facts and effects the assistant route needs. Production uses
/// the exact Pi runtime; tests script it so the HTTP boundary can be exercised
/// on every CI host without a Pi process.
pub(crate) trait AssistantRuntime {
    /// Provider binding for `agent://personal/pi`, derived from daemon facts.
    fn binding_state(&self) -> ProviderBindingState;
    /// Whether the exact Pi runtime and private adapter are configured.
    fn pi_available(&self) -> Result<(), String>;
    /// Pinned HTTPS origins the assistant research fetch may target.
    fn pinned_research_origins(&self) -> Vec<String>;
    /// Read-only GET of an already validated research target.
    fn fetch_research(&self, uri: &str) -> Result<Vec<u8>, String>;
    /// Run exact Pi once for the request and report what the daemon observed.
    fn infer(
        &self,
        request: &AssistantInferenceRequest,
    ) -> Result<ObservedInference, InferenceFailure>;
}

/// Production runtime: exact pinned Pi through `pi-agent-adapter
/// assistant-turn`, Provider reached only through the daemon's one-shot
/// private completion socket.
pub(crate) struct DaemonAssistantRuntime {
    config_dir: PathBuf,
    data_dir: PathBuf,
    authority_store: SqliteAuthorityStore,
    fetch_transport: RustlsReadOnlyFetchTransport,
}

impl DaemonAssistantRuntime {
    pub(crate) fn new(config_dir: &Path, data_dir: &Path, store: SqliteAuthorityStore) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
            data_dir: data_dir.to_path_buf(),
            authority_store: store,
            fetch_transport: RustlsReadOnlyFetchTransport::default(),
        }
    }
}

/// Provider binding for the hidden assistant: the P8-T13 `agent://personal/pi`
/// binding first, then the legacy `provider.json` + selected model carrier the
/// private candidate path also honours. Never derived from the client.
pub(crate) fn provider_binding_state(
    store: &SqliteAuthorityStore,
    config_dir: &Path,
) -> ProviderBindingState {
    if let Some(model_id) = selected_binding_model(store, PI_AGENT) {
        return ProviderBindingState::Bound {
            model_id,
            source: "agent-binding",
        };
    }
    let has_provider_config = ProviderConfigRepository::under_config_dir(config_dir)
        .load()
        .is_ok();
    let selected = SelectedModelRepository::under_config_dir(config_dir)
        .load()
        .ok()
        .flatten();
    match (has_provider_config, selected) {
        (true, Some(selected)) => ProviderBindingState::Bound {
            model_id: selected.model_id().to_owned(),
            source: "selected-model",
        },
        _ => ProviderBindingState::Unbound,
    }
}

impl AssistantRuntime for DaemonAssistantRuntime {
    fn binding_state(&self) -> ProviderBindingState {
        provider_binding_state(&self.authority_store, &self.config_dir)
    }

    fn pi_available(&self) -> Result<(), String> {
        let config = super::pi_runtime::load_pi_config(&self.config_dir)
            .map_err(|_| "Pi runtime is not configured on this daemon".to_owned())?;
        if !config
            .candidate_adapter_path()
            .is_some_and(|path| path.is_file())
        {
            return Err("private Pi candidate adapter is not configured".to_owned());
        }
        if !config
            .candidate_extension_entry_path()
            .is_some_and(|path| path.is_file())
        {
            return Err("private Pi candidate extension is not configured".to_owned());
        }
        if !config.executable_path().is_file() {
            return Err("configured Pi executable is missing".to_owned());
        }
        if !cfg!(unix) {
            return Err(
                "assistant inference requires a Unix-domain socket host; Windows Pi route is not-run until P13-T13"
                    .to_owned(),
            );
        }
        Ok(())
    }

    fn pinned_research_origins(&self) -> Vec<String> {
        super::pinned_https::allowed_origins(&self.data_dir, ASSISTANT_RESEARCH_TASK_REF)
    }

    fn fetch_research(&self, uri: &str) -> Result<Vec<u8>, String> {
        let response = self
            .fetch_transport
            .fetch(&ReadOnlyFetchRequest {
                method: ReadOnlyFetchMethod::Get,
                url: uri.to_owned(),
                timeout_ms: ASSISTANT_RESEARCH_TIMEOUT_MS,
                maximum_response_bytes: ASSISTANT_RESEARCH_RESPONSE_LIMIT,
            })
            .map_err(|error| format!("read-only fetch failed: {error:?}"))?;
        if response.status != 200 {
            return Err(format!("read-only fetch answered HTTP {}", response.status));
        }
        Ok(response.body)
    }

    #[cfg(unix)]
    fn infer(
        &self,
        request: &AssistantInferenceRequest,
    ) -> Result<ObservedInference, InferenceFailure> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::{Duration, Instant};

        use cognitive_runtime::{
            ASSISTANT_INFERENCE_FRAME_LIMIT, parse_assistant_inference_response,
        };

        use super::pi_runtime::{
            CANDIDATE_ENVIRONMENT_ALLOWLIST, PRIVATE_ADAPTER_TIMEOUT, PrivateCompletionSocket,
            adapter_rejection_message, load_pi_config,
        };

        self.pi_available()
            .map_err(InferenceFailure::PiUnavailable)?;
        let config = load_pi_config(&self.config_dir).map_err(|_| {
            InferenceFailure::PiUnavailable("Pi runtime is not configured".to_owned())
        })?;
        let adapter_path = config
            .candidate_adapter_path()
            .ok_or_else(|| InferenceFailure::PiUnavailable("adapter missing".to_owned()))?
            .to_path_buf();
        let extension_path = config
            .candidate_extension_entry_path()
            .and_then(|path| path.to_str())
            .ok_or_else(|| InferenceFailure::PiUnavailable("extension path invalid".to_owned()))?
            .to_owned();
        let executable_path = config
            .executable_path()
            .to_str()
            .ok_or_else(|| InferenceFailure::PiUnavailable("Pi path invalid".to_owned()))?
            .to_owned();
        let ProviderBindingState::Bound { model_id, .. } = self.binding_state() else {
            return Err(InferenceFailure::Failed("Provider is not bound".to_owned()));
        };
        let request_json = serde_json::to_vec(request)
            .map_err(|_| InferenceFailure::Failed("inference request is not JSON".to_owned()))?;
        if request_json.len() > ASSISTANT_INFERENCE_FRAME_LIMIT {
            return Err(InferenceFailure::Failed(
                "inference request exceeds transport limit".to_owned(),
            ));
        }
        let socket = PrivateCompletionSocket::create_with_store(
            &self.config_dir,
            Some(self.authority_store.clone()),
        )
        .map_err(InferenceFailure::Failed)?;
        let socket_path = socket
            .path()
            .to_str()
            .ok_or_else(|| InferenceFailure::Failed("socket path invalid".to_owned()))?
            .to_owned();
        let mut command = Command::new(&adapter_path);
        command.env_clear();
        for key in CANDIDATE_ENVIRONMENT_ALLOWLIST {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        let mut child = command
            .arg("assistant-turn")
            .arg("--pi")
            .arg(&executable_path)
            .arg("--model")
            .arg(&model_id)
            .arg("--work-dir")
            .arg(socket.runtime_dir())
            .arg("--config-dir")
            .arg(socket.runtime_dir())
            .arg("--extension")
            .arg(&extension_path)
            .env("COGNITIVEOS_PRIVATE_COMPLETION_SOCKET", &socket_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| {
                InferenceFailure::Failed("assistant adapter invocation failed".to_owned())
            })?;
        child
            .stdin
            .as_mut()
            .ok_or_else(|| InferenceFailure::Failed("adapter stdin was not captured".to_owned()))?
            .write_all(&request_json)
            .map_err(|_| {
                InferenceFailure::Failed("inference request could not be written".to_owned())
            })?;
        drop(child.stdin.take());
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| InferenceFailure::Failed("adapter stdout was not captured".to_owned()))?
            .take((ASSISTANT_INFERENCE_FRAME_LIMIT + 1) as u64);
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| InferenceFailure::Failed("adapter stderr was not captured".to_owned()))?
            .take(4096);
        let stdout_reader = thread::spawn(move || {
            let mut output = Vec::new();
            let mut stdout = stdout;
            stdout.read_to_end(&mut output).map_err(|_| ())?;
            Ok::<_, ()>(output)
        });
        let stderr_reader = thread::spawn(move || {
            let mut output = Vec::new();
            let mut stderr = stderr;
            let _ = stderr.read_to_end(&mut output);
            output
        });
        let started = Instant::now();
        let mut termination_error = None;
        let exit_status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Err(_) => {
                    termination_error = Some("assistant adapter wait failed".to_owned());
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                Ok(None) if started.elapsed() >= PRIVATE_ADAPTER_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    termination_error = Some("assistant adapter timed out".to_owned());
                    break None;
                }
                Ok(None) => thread::sleep(Duration::from_millis(25)),
            }
        };
        let output = stdout_reader
            .join()
            .map_err(|_| InferenceFailure::Failed("adapter stdout reader panicked".to_owned()))?
            .map_err(|_| InferenceFailure::Failed("adapter stdout could not be read".to_owned()))?;
        let stderr_output = stderr_reader.join().unwrap_or_default();
        let accepted = socket.accepted();
        if let Some(error) = termination_error {
            drop(socket);
            return Err(InferenceFailure::Failed(error));
        }
        let exit_status = exit_status.ok_or_else(|| {
            InferenceFailure::Failed("assistant adapter exited without a final status".to_owned())
        })?;
        if !exit_status.success() {
            drop(socket);
            return Err(InferenceFailure::Failed(adapter_rejection_message(
                exit_status.code(),
                &stderr_output,
            )));
        }
        if accepted {
            socket.finish().map_err(InferenceFailure::Failed)?;
        } else {
            drop(socket);
        }
        if output.len() > ASSISTANT_INFERENCE_FRAME_LIMIT {
            return Err(InferenceFailure::Failed(
                "assistant adapter response exceeds transport limit".to_owned(),
            ));
        }
        let response =
            parse_assistant_inference_response(&output).map_err(InferenceFailure::Failed)?;
        Ok(ObservedInference {
            assistant_text: response.assistant_text,
            response_model: response.response_model,
            provider_round_trips: u32::from(accepted),
        })
    }

    #[cfg(not(unix))]
    fn infer(
        &self,
        request: &AssistantInferenceRequest,
    ) -> Result<ObservedInference, InferenceFailure> {
        let _ = request;
        Err(InferenceFailure::PiUnavailable(
            "assistant inference requires a Unix-domain socket host; Windows Pi route is not-run until P13-T13"
                .to_owned(),
        ))
    }
}

/// Runtime used when the HTTP plane is exercised without a configured daemon
/// (store-only unit tests). Unbound, no Pi: the route answers with the Settings
/// pointer and registers nothing.
#[cfg(test)]
pub(crate) struct UnconfiguredAssistantRuntime;

#[cfg(test)]
impl AssistantRuntime for UnconfiguredAssistantRuntime {
    fn binding_state(&self) -> ProviderBindingState {
        ProviderBindingState::Unbound
    }

    fn pi_available(&self) -> Result<(), String> {
        Err("Pi runtime is not configured on this daemon".to_owned())
    }

    fn pinned_research_origins(&self) -> Vec<String> {
        Vec::new()
    }

    fn fetch_research(&self, _uri: &str) -> Result<Vec<u8>, String> {
        Err("no research fetch without a configured runtime".to_owned())
    }

    fn infer(
        &self,
        _request: &AssistantInferenceRequest,
    ) -> Result<ObservedInference, InferenceFailure> {
        Err(InferenceFailure::PiUnavailable(
            "Pi runtime is not configured on this daemon".to_owned(),
        ))
    }
}

/// `GET assistant.status`: what the create-page chat may render. `ready`
/// means bound Provider + configured exact Pi; anything else disables the chat
/// input and points at Settings (or states the Pi gap honestly).
pub(crate) fn handle_status(runtime: &dyn AssistantRuntime) -> ResourceApiResponse {
    let mut body = json!({
        "engine": ASSISTANT_ENGINE_ID,
        "pi_pin": ASSISTANT_PI_PIN,
        "protocol": ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL,
        "inference_protocol": ASSISTANT_INFERENCE_PROTOCOL,
        "installed_agent": false,
        "settings_route": ASSISTANT_SETTINGS_ROUTE,
        "research_fetch_family": ASSISTANT_RESEARCH_FETCH_FAMILY,
        "research_origins_pinned": runtime.pinned_research_origins().len(),
        "observation_only": true,
    });
    match runtime.binding_state() {
        ProviderBindingState::Unbound => {
            merge(&mut body, provider_unbound_guidance());
        }
        ProviderBindingState::Bound { model_id, source } => {
            body["model_id"] = json!(model_id);
            body["binding_source"] = json!(source);
            match runtime.pi_available() {
                Ok(()) => {
                    body["status"] = json!("ready");
                    body["chat_input"] = json!(true);
                    body["guidance"] = json!(
                        "Assistant is bound and exact Pi is configured. Turns produce candidates only; nothing is written until you confirm on the canvas."
                    );
                }
                Err(detail) => {
                    body["status"] = json!("pi_unavailable");
                    body["chat_input"] = json!(false);
                    body["pi_detail"] = json!(detail);
                    body["guidance"] = json!(
                        "A Provider is bound but the exact Pi runtime is not available on this daemon, so the assistant cannot infer here."
                    );
                }
            }
        }
    }
    ok(body)
}

fn merge(target: &mut Value, extra: Value) {
    if let (Some(target), Some(extra)) = (target.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            target.insert(key.clone(), value.clone());
        }
    }
}

/// `POST assistant.turn`: admit → binding → bounded Context → exact Pi →
/// parse chain → register candidate. Every refusal happens before any write.
pub(crate) fn handle_turn(
    body: &[u8],
    store: &SqliteAuthorityStore,
    runtime: &dyn AssistantRuntime,
) -> ResourceApiResponse {
    if reject_closed_candidate_schema(body).is_err() {
        return error(
            422,
            "ASSISTANT_SCHEMA_CLOSED",
            "closed candidate schema: grant/secret/trigger-arm fields rejected",
        );
    }
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(kind) = document.get("kind").and_then(Value::as_str) else {
        return error(400, "ASSISTANT_KIND_REQUIRED", "kind required");
    };
    let Some(draft_id) = document.get("draft_id").and_then(Value::as_str) else {
        return error(400, "DRAFT_ID_REQUIRED", "draft_id required");
    };
    let Some(object_kind) = document.get("object_kind").and_then(Value::as_str) else {
        return error(400, "OBJECT_KIND_REQUIRED", "object_kind required");
    };
    let payload = document
        .get("payload")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let provenance_json = match document.get("provenance") {
        Some(Value::Object(_)) | Some(Value::Array(_)) => document
            .get("provenance")
            .cloned()
            .and_then(|value| serde_json::to_string(&value).ok()),
        Some(Value::String(raw)) => Some(raw.clone()),
        Some(_) | None => None,
    };
    let Some(provenance_json) = provenance_json else {
        return error(
            422,
            "ASSISTANT_PROVENANCE_REQUIRED",
            "typed provenance required (sources | owner-stated | assistant-assumption)",
        );
    };
    let project_id = document
        .get("project_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let tool_owned: Vec<String> = document
        .get("tools")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let tools: Vec<&str> = tool_owned.iter().map(String::as_str).collect();
    if let Err(refusal) =
        AssistantPlane::admit_turn_request(kind, object_kind, &provenance_json, &tools)
    {
        return store_error(refusal);
    }
    let projects = ProjectAggregateStore::from_authority_store(store);
    let draft_seq = match projects.get_draft_seq(draft_id) {
        Ok(seq) => seq,
        Err(refusal) => return store_error(refusal),
    };
    let ProviderBindingState::Bound { model_id, source } = runtime.binding_state() else {
        return provider_unbound_response();
    };

    let plane = AssistantPlane::from_authority_store(store);
    let context_refs = match plane.context_refs(project_id) {
        Ok(refs) => refs,
        Err(refusal) => return store_error(refusal),
    };
    let owner_provenance: Value = match serde_json::from_str(&provenance_json) {
        Ok(value) => value,
        Err(_) => {
            return error(
                422,
                "ASSISTANT_PROVENANCE_REQUIRED",
                "typed provenance required (sources | owner-stated | assistant-assumption)",
            );
        }
    };
    let mut allowed_source_uris = owner_supplied_source_uris(&owner_provenance);

    let mut layers = vec![AssistantContextLayer {
        layer: "task-contract".to_owned(),
        body: format!(
            "draft {draft_id} (base_seq {draft_seq}); {kind} turn for object_kind {object_kind}; candidate-only, nothing is written until the owner confirms on the canvas"
        ),
        source_ref: Some(draft_id.to_owned()),
    }];
    if let Some(project_id) = project_id {
        match projects.get_project(project_id) {
            Ok(Some(project)) => layers.push(AssistantContextLayer {
                layer: "fixed-decision".to_owned(),
                body: format!(
                    "project {} state {}; charter revision {}; plan revision {}",
                    project.project_id,
                    project.state,
                    project.current_charter_revision_id,
                    project
                        .current_plan_revision_id
                        .as_deref()
                        .unwrap_or("(none)")
                ),
                source_ref: Some(project_id.to_owned()),
            }),
            Ok(None) => return error(404, "PROJECT_NOT_FOUND", "project not found"),
            Err(refusal) => return store_error(refusal),
        }
    }
    if !context_refs.is_empty() {
        let summary = context_refs
            .iter()
            .take(SUMMARY_REF_LIMIT)
            .map(|record_id| {
                format!("conversation record {record_id} (index only, body not injected)")
            })
            .collect::<Vec<_>>()
            .join("\n");
        layers.push(AssistantContextLayer {
            layer: "summary".to_owned(),
            body: summary,
            source_ref: project_id.map(ToOwned::to_owned),
        });
    }

    let mut research = ResearchOutcome::default();
    if kind == "research" {
        let targets: Vec<String> = document
            .get("research_targets")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|uri| !uri.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        if targets.len() > ASSISTANT_RESEARCH_MAX_TARGETS {
            return error(
                422,
                "ASSISTANT_RESEARCH_BOUNDED",
                "research_targets exceeds the bounded target count",
            );
        }
        let pinned = runtime.pinned_research_origins();
        for uri in targets {
            if let Err(reason) = validate_research_target(&uri, &pinned) {
                research
                    .refused
                    .push(json!({ "uri": uri, "reason": reason }));
                continue;
            }
            match runtime.fetch_research(&uri) {
                Ok(bytes) => {
                    layers.push(AssistantContextLayer {
                        layer: "sourced-excerpt".to_owned(),
                        body: research_excerpt(&bytes),
                        source_ref: Some(uri.clone()),
                    });
                    allowed_source_uris.push(uri.clone());
                    research.fetched.push(uri);
                }
                Err(reason) => {
                    research
                        .refused
                        .push(json!({ "uri": uri, "reason": reason }));
                }
            }
        }
    }
    allowed_source_uris.sort();
    allowed_source_uris.dedup();

    let bounded = match assemble_bounded_context(layers, ASSISTANT_CONTEXT_BUDGET_BYTES) {
        Ok(bounded) => bounded,
        Err(detail) => return error(422, "ASSISTANT_CONTEXT_INVALID", &detail),
    };
    let request = AssistantInferenceRequest {
        protocol: ASSISTANT_INFERENCE_PROTOCOL.to_owned(),
        turn: kind.to_owned(),
        object_kind: object_kind.to_owned(),
        draft_id: draft_id.to_owned(),
        project_id: project_id.map(ToOwned::to_owned),
        owner_payload: payload.clone(),
        owner_provenance: owner_provenance.clone(),
        context: bounded.layers.clone(),
        allowed_source_uris: allowed_source_uris.clone(),
    };
    if let Err(detail) = validate_assistant_inference_request(&request) {
        return error(422, "ASSISTANT_CONTEXT_INVALID", &detail);
    }

    let observed = match runtime.infer(&request) {
        Ok(observed) => observed,
        Err(InferenceFailure::PiUnavailable(detail)) => {
            return error(503, "ASSISTANT_PI_UNAVAILABLE", &detail);
        }
        Err(InferenceFailure::Failed(detail)) => {
            return error(502, "ASSISTANT_INFERENCE_FAILED", &detail);
        }
    };
    let chain = match parse_assistant_object_chain(&observed.assistant_text, &allowed_source_uris) {
        Ok(chain) => chain,
        Err(detail) => {
            // Refused text is observation-only: a bounded, secret-scrubbed
            // excerpt lets the owner see why nothing was registered.
            return ResourceApiResponse {
                status: 422,
                body: json!({
                    "status": "error",
                    "code": "ASSISTANT_CANDIDATE_REFUSED",
                    "message": detail,
                    "candidate_registered": false,
                    "provider_round_trips": observed.provider_round_trips,
                    "assistant_text_excerpt": refused_text_excerpt(&observed.assistant_text),
                })
                .to_string(),
                content_type: "application/json",
            };
        }
    };
    let record = AssistantInferenceRecord {
        protocol: ASSISTANT_INFERENCE_PROTOCOL,
        model_id: &model_id,
        provider_round_trips: observed.provider_round_trips,
        objects: &chain.objects,
        reply: &chain.reply,
        allowed_source_uris: &allowed_source_uris,
    };
    match plane.run_turn(&AssistantTurnSpec {
        kind,
        draft_id,
        object_kind,
        payload: &payload,
        provenance_json: &provenance_json,
        project_id,
        tools: &tools,
        inference: &record,
        now_ms: now_ms(),
    }) {
        Ok(outcome) => ok(json!({
            "status": "ok",
            "engine": ASSISTANT_ENGINE_ID,
            "pi_pin": ASSISTANT_PI_PIN,
            "protocol": ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL,
            "inference_protocol": ASSISTANT_INFERENCE_PROTOCOL,
            "installed_agent": false,
            "model_id": outcome.model_id,
            "binding_source": source,
            "response_model": observed.response_model,
            "provider_round_trips": outcome.provider_round_trips,
            "candidate_id": outcome.candidate_id,
            "candidate_digest": outcome.candidate_digest,
            "preview_id": outcome.preview_id,
            "object_kind": outcome.object_kind,
            "chain_object_kinds": outcome.chain_object_kinds,
            "chain": chain.objects,
            "reply": outcome.reply,
            "context_refs": outcome.context_refs,
            "context": {
                "inject_order_ref": "CONTEXT_INJECT_ORDER",
                "layers": bounded.layers.iter().map(|layer| json!({
                    "layer": layer.layer,
                    "source_ref": layer.source_ref,
                    "bytes": layer.body.len(),
                })).collect::<Vec<_>>(),
                "dropped_layers": bounded.dropped_layers,
                "bytes": bounded.bytes,
            },
            "research": {
                "fetch_family": ASSISTANT_RESEARCH_FETCH_FAMILY,
                "fetched": research.fetched,
                "refused": research.refused,
            },
            "observation_only": true,
        })),
        Err(refusal) => store_error(refusal),
    }
}

/// Fixed Settings pointer. HTTP 409: the request is well-formed but the
/// daemon has no bound Provider to infer with. No candidate, no chat box.
fn provider_unbound_response() -> ResourceApiResponse {
    let mut body = json!({
        "status": "error",
        "code": "ASSISTANT_PROVIDER_UNBOUND",
        "message": "no Provider is bound to the assistant; open Settings to connect one",
        "engine": ASSISTANT_ENGINE_ID,
        "pi_pin": ASSISTANT_PI_PIN,
        "inference_protocol": ASSISTANT_INFERENCE_PROTOCOL,
    });
    merge(&mut body, provider_unbound_guidance());
    body["status"] = json!("provider_unbound");
    ResourceApiResponse {
        status: 409,
        body: body.to_string(),
        content_type: "application/json",
    }
}

#[derive(Default)]
struct ResearchOutcome {
    fetched: Vec<String>,
    refused: Vec<Value>,
}

/// Owner-supplied `sources[]` uris are citable without a fetch.
fn owner_supplied_source_uris(provenance: &Value) -> Vec<String> {
    let items: Vec<&Value> = match provenance {
        Value::Array(items) => items.iter().collect(),
        Value::Object(object) if object.get("kind").and_then(Value::as_str) == Some("sources") => {
            object
                .get("sources")
                .and_then(Value::as_array)
                .map(|items| items.iter().collect())
                .unwrap_or_default()
        }
        _ => Vec::new(),
    };
    items
        .into_iter()
        .filter_map(|source| source.get("uri").and_then(Value::as_str))
        .map(str::trim)
        .filter(|uri| !uri.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Bounded excerpt of a refused assistant text. Key-shaped tokens are
/// scrubbed even though the child never receives Provider material.
fn refused_text_excerpt(text: &str) -> String {
    const EXCERPT_CHARS: usize = 1000;
    let scrubbed = text
        .split_whitespace()
        .map(|token| {
            let lowered = token.to_ascii_lowercase();
            if lowered.starts_with("sk-")
                || lowered.contains("api_key")
                || lowered.contains("bearer")
            {
                "<redacted>".to_owned()
            } else {
                token.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut excerpt: String = scrubbed.chars().take(EXCERPT_CHARS).collect();
    if scrubbed.chars().count() > EXCERPT_CHARS {
        excerpt.push('…');
    }
    excerpt
}

/// Bounded, control-character-free excerpt of a fetched research body.
fn research_excerpt(bytes: &[u8]) -> String {
    let end = bytes.len().min(ASSISTANT_RESEARCH_EXCERPT_BYTES);
    String::from_utf8_lossy(&bytes[..end])
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\n')
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
pub(crate) mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use cognitive_store::{
        ASSISTANT_RESEARCH_FETCH_FAMILY, PersonalDataLayout, prepare_personal_databases,
    };
    use tempfile::TempDir;

    use super::super::project_aggregate::handle_with_assistant;
    use super::*;

    /// Scripted runtime: the HTTP boundary is exercised without a Pi process.
    pub(crate) struct ScriptedAssistantRuntime {
        pub binding: ProviderBindingState,
        pub pi: Result<(), String>,
        pub pinned: Vec<String>,
        pub pages: BTreeMap<String, Vec<u8>>,
        pub script: Result<ObservedInference, InferenceFailure>,
        pub requests: RefCell<Vec<AssistantInferenceRequest>>,
        pub fetches: RefCell<Vec<String>>,
    }

    impl ScriptedAssistantRuntime {
        pub(crate) fn bound(text: &str, round_trips: u32) -> Self {
            Self {
                binding: ProviderBindingState::Bound {
                    model_id: "deepseek-chat".to_owned(),
                    source: "agent-binding",
                },
                pi: Ok(()),
                pinned: Vec::new(),
                pages: BTreeMap::new(),
                script: Ok(ObservedInference {
                    assistant_text: text.to_owned(),
                    response_model: Some("deepseek-chat".to_owned()),
                    provider_round_trips: round_trips,
                }),
                requests: RefCell::new(Vec::new()),
                fetches: RefCell::new(Vec::new()),
            }
        }

        pub(crate) fn unbound() -> Self {
            let mut runtime = Self::bound("", 0);
            runtime.binding = ProviderBindingState::Unbound;
            runtime
        }
    }

    impl AssistantRuntime for ScriptedAssistantRuntime {
        fn binding_state(&self) -> ProviderBindingState {
            self.binding.clone()
        }

        fn pi_available(&self) -> Result<(), String> {
            self.pi.clone()
        }

        fn pinned_research_origins(&self) -> Vec<String> {
            self.pinned.clone()
        }

        fn fetch_research(&self, uri: &str) -> Result<Vec<u8>, String> {
            self.fetches.borrow_mut().push(uri.to_owned());
            self.pages
                .get(uri)
                .cloned()
                .ok_or_else(|| "scripted page missing".to_owned())
        }

        fn infer(
            &self,
            request: &AssistantInferenceRequest,
        ) -> Result<ObservedInference, InferenceFailure> {
            self.requests.borrow_mut().push(request.clone());
            self.script.clone()
        }
    }

    const CHARTER_CHAIN: &str = r#"{"reply":"Here is a candidate charter; nothing is written until you confirm.","objects":[{"object_kind":"business-brief","fields":{"goal":{"value":"weekly client report","provenance":{"kind":"owner-stated"}}}},{"object_kind":"charter","fields":{"title":{"value":"Weekly client report","provenance":{"kind":"owner-stated"}},"cadence":{"value":"weekly","provenance":{"kind":"assistant-assumption"}}}}]}"#;

    fn authority() -> (TempDir, SqliteAuthorityStore, String) {
        let temporary = TempDir::new().expect("temp");
        let root = temporary.path();
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        prepare_personal_databases(&layout).expect("prepare");
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).expect("open");
        let (draft_id, _) = ProjectAggregateStore::from_authority_store(&store)
            .create_draft(b"payload", 1)
            .unwrap();
        (temporary, store, draft_id)
    }

    fn turn_body(kind: &str, draft_id: &str, object_kind: &str) -> Vec<u8> {
        json!({
            "kind": kind,
            "draft_id": draft_id,
            "object_kind": object_kind,
            "payload": {"text": "weekly report for my clients"},
            "provenance": {"kind": "owner-stated"}
        })
        .to_string()
        .into_bytes()
    }

    fn post(
        body: &[u8],
        store: &SqliteAuthorityStore,
        runtime: &dyn AssistantRuntime,
    ) -> ResourceApiResponse {
        handle_with_assistant(
            "POST /management/project/v1/assistant.turn",
            body,
            store,
            runtime,
        )
    }

    fn candidate_count(store: &SqliteAuthorityStore, draft_id: &str) -> i64 {
        AssistantPlane::from_authority_store(store)
            .candidate_count(draft_id)
            .unwrap()
    }

    #[test]
    fn provider_unbound_turn_points_at_settings_and_registers_nothing() {
        let (_tmp, store, draft_id) = authority();
        let runtime = ScriptedAssistantRuntime::unbound();
        let response = post(
            &turn_body("propose", &draft_id, "charter"),
            &store,
            &runtime,
        );
        assert_eq!(response.status, 409, "{}", response.body);
        let body: Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body["code"], "ASSISTANT_PROVIDER_UNBOUND");
        assert_eq!(body["status"], "provider_unbound");
        assert_eq!(body["settings_route"], ASSISTANT_SETTINGS_ROUTE);
        assert_eq!(body["chat_input"], false);
        assert_eq!(body["silent_bind"], false);
        assert_eq!(body["candidate_registered"], false);
        assert!(body.get("candidate_digest").is_none());
        assert!(
            runtime.requests.borrow().is_empty(),
            "Pi is never invoked unbound"
        );
        assert_eq!(candidate_count(&store, &draft_id), 0);

        let status = handle_with_assistant(
            "GET /management/project/v1/assistant.status",
            b"",
            &store,
            &runtime,
        );
        assert_eq!(status.status, 200, "{}", status.body);
        let status_body: Value = serde_json::from_str(&status.body).unwrap();
        assert_eq!(status_body["status"], "provider_unbound");
        assert_eq!(status_body["chat_input"], false);
        assert_eq!(status_body["settings_route"], ASSISTANT_SETTINGS_ROUTE);
        assert_eq!(status_body["installed_agent"], false);
        assert!(!status.body.contains("Approve"));
    }

    #[test]
    fn ambient_tool_and_missing_draft_are_refused_before_pi_spawns() {
        let (_tmp, store, draft_id) = authority();
        let runtime = ScriptedAssistantRuntime::bound(CHARTER_CHAIN, 1);
        let ambient = json!({
            "kind": "propose",
            "draft_id": draft_id,
            "object_kind": "charter",
            "payload": {"text": "x"},
            "provenance": {"kind": "owner-stated"},
            "tools": ["bash"]
        })
        .to_string();
        let response = post(ambient.as_bytes(), &store, &runtime);
        assert_eq!(response.status, 403, "{}", response.body);
        let missing = post(
            &turn_body("propose", "draft-missing", "charter"),
            &store,
            &runtime,
        );
        assert_eq!(missing.status, 404, "{}", missing.body);
        let closed = json!({
            "kind": "propose",
            "draft_id": draft_id,
            "object_kind": "recipe",
            "payload": {"text": "x"},
            "provenance": {"kind": "owner-stated"},
            "grant": "workspace-write"
        })
        .to_string();
        assert_eq!(post(closed.as_bytes(), &store, &runtime).status, 422);
        let unlabeled = json!({
            "kind": "propose",
            "draft_id": draft_id,
            "object_kind": "charter",
            "payload": {"text": "x"},
            "provenance": "notes"
        })
        .to_string();
        assert_eq!(post(unlabeled.as_bytes(), &store, &runtime).status, 422);
        assert!(
            runtime.requests.borrow().is_empty(),
            "no inference before admission"
        );
        assert_eq!(candidate_count(&store, &draft_id), 0);
    }

    #[test]
    fn prose_echo_fabricated_source_and_zero_round_trip_register_nothing() {
        let (_tmp, store, draft_id) = authority();
        let prose = ScriptedAssistantRuntime::bound(
            "I would suggest a weekly report. Use sk-abcdefghijklmnopqrstuvwxyz if needed.",
            1,
        );
        let response = post(&turn_body("propose", &draft_id, "charter"), &store, &prose);
        assert_eq!(response.status, 422, "{}", response.body);
        let refused: Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(refused["code"], "ASSISTANT_CANDIDATE_REFUSED");
        assert_eq!(refused["candidate_registered"], false);
        assert_eq!(refused["provider_round_trips"], 1);
        let excerpt = refused["assistant_text_excerpt"].as_str().unwrap();
        assert!(excerpt.starts_with("I would suggest a weekly report."));
        assert!(
            !excerpt.contains("sk-abc"),
            "key-shaped token is scrubbed: {excerpt}"
        );
        assert!(excerpt.contains("<redacted>"));
        assert_eq!(candidate_count(&store, &draft_id), 0);

        let fabricated = ScriptedAssistantRuntime::bound(
            r#"{"reply":"x","objects":[{"object_kind":"research-run","fields":{"f":{"value":1,"provenance":{"kind":"sources","sources":[{"uri":"https://example.invalid/never-fetched"}]}}}}]}"#,
            1,
        );
        let response = post(
            &turn_body("research", &draft_id, "research-run"),
            &store,
            &fabricated,
        );
        assert_eq!(response.status, 422, "{}", response.body);
        assert!(response.body.contains("fabricated"));
        assert_eq!(candidate_count(&store, &draft_id), 0);

        let echo = ScriptedAssistantRuntime::bound(CHARTER_CHAIN, 0);
        let response = post(&turn_body("propose", &draft_id, "charter"), &store, &echo);
        assert_eq!(response.status, 422, "{}", response.body);
        assert!(response.body.contains("inference required"));
        assert_eq!(candidate_count(&store, &draft_id), 0);

        let failed = ScriptedAssistantRuntime {
            script: Err(InferenceFailure::Failed("adapter exit code 3".to_owned())),
            ..ScriptedAssistantRuntime::bound(CHARTER_CHAIN, 1)
        };
        let response = post(&turn_body("propose", &draft_id, "charter"), &store, &failed);
        assert_eq!(response.status, 502, "{}", response.body);
        let pi_missing = ScriptedAssistantRuntime {
            script: Err(InferenceFailure::PiUnavailable(
                "Pi runtime is not configured".to_owned(),
            )),
            pi: Err("Pi runtime is not configured".to_owned()),
            ..ScriptedAssistantRuntime::bound(CHARTER_CHAIN, 1)
        };
        let response = post(
            &turn_body("propose", &draft_id, "charter"),
            &store,
            &pi_missing,
        );
        assert_eq!(response.status, 503, "{}", response.body);
        let status = handle_with_assistant(
            "GET /management/project/v1/assistant.status",
            b"",
            &store,
            &pi_missing,
        );
        let status_body: Value = serde_json::from_str(&status.body).unwrap();
        assert_eq!(status_body["status"], "pi_unavailable");
        assert_eq!(status_body["chat_input"], false);
        assert_eq!(candidate_count(&store, &draft_id), 0);
    }

    #[test]
    fn inferred_turn_registers_chain_with_bounded_context_and_no_approve() {
        let (_tmp, store, draft_id) = authority();
        let runtime = ScriptedAssistantRuntime::bound(CHARTER_CHAIN, 1);
        let response = post(
            &turn_body("propose", &draft_id, "charter"),
            &store,
            &runtime,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let body: Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body["model_id"], "deepseek-chat");
        assert_eq!(body["provider_round_trips"], 1);
        assert_eq!(body["inference_protocol"], ASSISTANT_INFERENCE_PROTOCOL);
        assert_eq!(
            body["chain_object_kinds"],
            json!(["business-brief", "charter"])
        );
        assert_eq!(body["chain"][1]["object_kind"], "charter");
        assert_eq!(body["installed_agent"], false);
        assert!(body["candidate_digest"].as_str().unwrap().len() == 64);
        assert!(
            body["preview_id"].is_string(),
            "propose announces a preview"
        );
        assert!(body["reply"].as_str().unwrap().contains("candidate"));
        assert_eq!(body["context"]["inject_order_ref"], "CONTEXT_INJECT_ORDER");
        assert_eq!(body["context"]["layers"][0]["layer"], "task-contract");
        assert!(!response.body.contains("Approve"));
        assert!(!response.body.contains("preview_digest"));
        assert_eq!(candidate_count(&store, &draft_id), 1);

        let requests = runtime.requests.borrow();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.protocol, ASSISTANT_INFERENCE_PROTOCOL);
        assert_eq!(request.turn, "propose");
        assert_eq!(
            request.owner_payload["text"],
            "weekly report for my clients"
        );
        assert_eq!(request.context[0].layer, "task-contract");
        let frame = serde_json::to_string(request).unwrap();
        for forbidden in [
            "bearer",
            "bootstrap",
            "secret_ref",
            "api_key",
            "wia",
            "effect",
        ] {
            assert!(
                !frame.to_ascii_lowercase().contains(forbidden),
                "{forbidden} in frame"
            );
        }
        drop(requests);

        let explain = post(
            &turn_body("explain", &draft_id, "business-brief"),
            &store,
            &runtime,
        );
        assert_eq!(explain.status, 200, "{}", explain.body);
        let explain_body: Value = serde_json::from_str(&explain.body).unwrap();
        assert!(
            explain_body["preview_id"].is_null(),
            "explain announces no preview"
        );
        assert_eq!(candidate_count(&store, &draft_id), 2);
        assert_eq!(
            ProjectAggregateStore::from_authority_store(&store)
                .get_draft_seq(&draft_id)
                .unwrap(),
            0,
            "candidate registration does not apply"
        );
    }

    #[test]
    fn research_targets_outside_pinned_origins_are_never_fetched() {
        let (_tmp, store, draft_id) = authority();
        let chain = r#"{"reply":"Research candidate.","objects":[{"object_kind":"research-run","fields":{"format":{"value":"one page","provenance":{"kind":"sources","sources":[{"uri":"https://example.invalid/report-format"}]}}}}]}"#;
        let mut runtime = ScriptedAssistantRuntime::bound(chain, 1);
        runtime.pinned = vec!["https://example.invalid".to_owned()];
        runtime.pages.insert(
            "https://example.invalid/report-format".to_owned(),
            b"One page, three sections.\x00\x01".to_vec(),
        );
        let body = json!({
            "kind": "research",
            "draft_id": draft_id,
            "object_kind": "research-run",
            "payload": {"text": "how are weekly client reports formatted?"},
            "provenance": {"kind": "owner-stated"},
            "tools": [ASSISTANT_RESEARCH_FETCH_FAMILY],
            "research_targets": [
                "https://example.invalid/report-format",
                "http://example.invalid/plaintext",
                "https://other.invalid/not-pinned",
                "https://user@example.invalid/userinfo"
            ]
        })
        .to_string();
        let response = post(body.as_bytes(), &store, &runtime);
        assert_eq!(response.status, 200, "{}", response.body);
        let parsed: Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(
            parsed["research"]["fetch_family"],
            ASSISTANT_RESEARCH_FETCH_FAMILY
        );
        assert_eq!(
            parsed["research"]["fetched"],
            json!(["https://example.invalid/report-format"])
        );
        assert_eq!(parsed["research"]["refused"].as_array().unwrap().len(), 3);
        assert_eq!(
            runtime.fetches.borrow().as_slice(),
            ["https://example.invalid/report-format".to_owned()],
            "only the pinned HTTPS target was fetched"
        );
        let requests = runtime.requests.borrow();
        let excerpt = requests[0]
            .context
            .iter()
            .find(|layer| layer.layer == "sourced-excerpt")
            .expect("fetched excerpt is a sourced-excerpt layer");
        assert_eq!(excerpt.body, "One page, three sections.");
        assert_eq!(
            requests[0].allowed_source_uris,
            ["https://example.invalid/report-format".to_owned()]
        );
        drop(requests);

        let mut unpinned = ScriptedAssistantRuntime::bound(chain, 1);
        unpinned.pages = runtime.pages.clone();
        let response = post(body.as_bytes(), &store, &unpinned);
        assert_eq!(response.status, 422, "{}", response.body);
        assert!(
            response.body.contains("fabricated"),
            "unfetched uri is not citable"
        );
        assert!(
            unpinned.fetches.borrow().is_empty(),
            "default-empty registry fetches nothing"
        );

        let too_many = json!({
            "kind": "research",
            "draft_id": draft_id,
            "object_kind": "research-run",
            "payload": {},
            "provenance": {"kind": "owner-stated"},
            "research_targets": ["https://a.invalid", "https://b.invalid", "https://c.invalid", "https://d.invalid", "https://e.invalid"]
        })
        .to_string();
        assert_eq!(post(too_many.as_bytes(), &store, &runtime).status, 422);
    }

    #[test]
    fn unconfigured_runtime_is_unbound_and_task_channel_is_forbidden() {
        let (_tmp, store, draft_id) = authority();
        let response = post(
            &turn_body("propose", &draft_id, "charter"),
            &store,
            &UnconfiguredAssistantRuntime,
        );
        assert_eq!(response.status, 409, "{}", response.body);
        let task = handle_with_assistant(
            "GET /task/project/v1/assistant.status",
            b"",
            &store,
            &UnconfiguredAssistantRuntime,
        );
        assert_eq!(task.status, 403);
        let (tmp, _, _) = authority();
        let state = provider_binding_state(&store, &tmp.path().join("config"));
        assert_eq!(state, ProviderBindingState::Unbound);
    }
}
