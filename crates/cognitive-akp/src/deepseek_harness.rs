//! Candidate-only DeepSeek Harness (`dsh`) bridge.
//!
//! The bridge owns no dsh internals and is not an authority writer. A dsh
//! plugin shim emits this stable wire shape; the daemon-facing side translates
//! it into AKP and, for Workspace* candidates, into bounded public-candidate
//! fields. dsh responses never complete a Task.

use super::{VERSION, digest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

/// Default AKP adapter identity for a dsh sidecar registration.
pub const ADAPTER_ID: &str = "deepseek.dsh.akp";
/// The bridge protocol is versioned independently from dsh's internal plugin APIs.
pub const BRIDGE_PROTOCOL: &str = "cognitiveos.dsh-akp/0.1";
/// Exact DeepSeek Harness source pin used by live registration (git object).
pub const PINNED_DSH_REVISION: &str = "528c682e061696f5a160f363f236ecbf53cbd006";
/// JSONL / HTTP frame ceiling shared with the TypeScript shim.
pub const MAX_FRAME_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginEventKind {
    Candidate,
    Observation,
    Lifecycle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DshPluginEvent {
    pub kind: PluginEventKind,
    pub operation: String,
    pub payload: Value,
    #[serde(default)]
    pub authority_claim: bool,
    #[serde(default)]
    pub secret_shaped: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DshAdapterRequest {
    pub bridge_protocol: String,
    pub dsh_version: String,
    pub schema_digest: String,
    pub session_id: String,
    pub fencing_epoch: u64,
    pub sequence: u64,
    pub plugin_id: String,
    pub correlation_id: String,
    pub deadline: String,
    /// Daemon-bound Task URI supplied by the CognitiveOS adapter, never by a
    /// dsh plugin payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_ref: Option<String>,
    pub event: DshPluginEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshAdapterConfig {
    pub expected_dsh_version: String,
    pub expected_schema_digest: String,
    pub expected_fencing_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Registered,
    Active,
    Stopped,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DshAdapterError {
    #[error("dsh AKP adapter is not active")]
    Inactive,
    #[error("dsh bridge protocol is unsupported")]
    BridgeProtocolMismatch,
    #[error("dsh version is not the registered exact pin")]
    DshVersionMismatch,
    #[error("AKP schema digest is not the registered exact pin")]
    SchemaDigestMismatch,
    #[error("dsh adapter identity or session material is missing")]
    MissingIdentity,
    #[error("dsh plugin event session does not match the fenced session")]
    WrongSession,
    #[error("dsh fencing epoch is stale")]
    StaleFencingEpoch,
    #[error("dsh plugin event claims authority")]
    AuthorityClaimForbidden,
    #[error("dsh plugin event is secret-shaped")]
    SecretShapedPayload,
    #[error("dsh plugin event sequence is stale or duplicated")]
    SequenceNotMonotonic,
    #[error("dsh operation or payload is missing")]
    InvalidEvent,
    #[error("dsh payload contains a forbidden secret-shaped or authority-shaped field")]
    ForbiddenPayloadField,
    #[error("dsh JSONL frame exceeds the configured byte limit")]
    FrameTooLarge,
    #[error("dsh JSONL frame is not JSON")]
    MalformedJson,
    #[error("AKP envelope construction failed: {0}")]
    Akp(String),
}

impl DshAdapterError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Inactive => "INACTIVE",
            Self::BridgeProtocolMismatch => "BRIDGE_PROTOCOL_MISMATCH",
            Self::DshVersionMismatch => "DSH_VERSION_MISMATCH",
            Self::SchemaDigestMismatch => "SCHEMA_DIGEST_MISMATCH",
            Self::MissingIdentity => "MISSING_IDENTITY",
            Self::WrongSession => "WRONG_SESSION",
            Self::StaleFencingEpoch => "STALE_FENCING_EPOCH",
            Self::AuthorityClaimForbidden => "AUTHORITY_CLAIM_FORBIDDEN",
            Self::SecretShapedPayload => "SECRET_SHAPED_PAYLOAD",
            Self::SequenceNotMonotonic => "SEQUENCE_NOT_MONOTONIC",
            Self::InvalidEvent => "INVALID_EVENT",
            Self::ForbiddenPayloadField => "FORBIDDEN_PAYLOAD_FIELD",
            Self::FrameTooLarge => "FRAME_TOO_LARGE",
            Self::MalformedJson => "MALFORMED_JSON",
            Self::Akp(_) => "AKP",
        }
    }
}

/// Daemon-side state for one fenced dsh session.
#[derive(Debug, Clone)]
pub struct DeepSeekHarnessAdapter {
    config: DshAdapterConfig,
    state: AdapterState,
    session_id: Option<String>,
    last_sequence: Option<u64>,
    bound_task_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DshWireResponse {
    pub accepted: bool,
    pub sequence: u64,
    pub candidate_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl DshWireResponse {
    pub fn rejected(sequence: u64, error: &DshAdapterError) -> Self {
        Self {
            accepted: false,
            sequence,
            candidate_only: true,
            result: None,
            error: Some(error.code().to_owned()),
        }
    }

    pub fn accepted(sequence: u64, result: Value) -> Self {
        Self {
            accepted: true,
            sequence,
            candidate_only: true,
            result: Some(result),
            error: None,
        }
    }
}

/// Bounded Workspace* fields extracted from a candidate event. These remain
/// untrusted candidate data until the daemon admits them.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceCandidateFields {
    pub tool_ref: String,
    pub action: String,
    pub target: String,
    pub parameters: Option<Value>,
    pub parameters_digest: String,
    pub expected_state_version: i64,
    pub operation_descriptor_id: String,
}

impl DeepSeekHarnessAdapter {
    pub fn new(config: DshAdapterConfig) -> Result<Self, DshAdapterError> {
        if config.expected_dsh_version.trim().is_empty()
            || config.expected_schema_digest.trim().is_empty()
            || config.expected_fencing_epoch == 0
        {
            return Err(DshAdapterError::MissingIdentity);
        }
        Ok(Self {
            config,
            state: AdapterState::Registered,
            session_id: None,
            last_sequence: None,
            bound_task_ref: None,
        })
    }

    pub fn state(&self) -> AdapterState {
        self.state
    }

    pub fn bound_task_ref(&self) -> Option<&str> {
        self.bound_task_ref.as_deref()
    }

    /// Fenced session id, if the adapter is active.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Fencing epoch this adapter was constructed to enforce.
    pub fn fencing_epoch(&self) -> u64 {
        self.config.expected_fencing_epoch
    }

    /// Last accepted plugin event sequence, if any.
    pub fn last_sequence(&self) -> Option<u64> {
        self.last_sequence
    }

    pub fn activate(&mut self, session_id: &str) -> Result<(), DshAdapterError> {
        self.activate_fenced(session_id, self.config.expected_fencing_epoch, None)
    }

    pub fn activate_fenced(
        &mut self,
        session_id: &str,
        fencing_epoch: u64,
        task_ref: Option<String>,
    ) -> Result<(), DshAdapterError> {
        if session_id.trim().is_empty() || fencing_epoch == 0 {
            return Err(DshAdapterError::MissingIdentity);
        }
        if fencing_epoch != self.config.expected_fencing_epoch {
            return Err(DshAdapterError::StaleFencingEpoch);
        }
        if self.state == AdapterState::Active {
            return Err(DshAdapterError::InvalidEvent);
        }
        self.state = AdapterState::Active;
        self.session_id = Some(session_id.to_owned());
        self.last_sequence = None;
        self.bound_task_ref = task_ref.filter(|value| !value.trim().is_empty());
        Ok(())
    }

    pub fn stop(&mut self) {
        self.state = AdapterState::Stopped;
        self.session_id = None;
        self.last_sequence = None;
        self.bound_task_ref = None;
    }

    /// Translate one dsh shim event into a fully pinned AKP request.
    pub fn translate(&mut self, request: &DshAdapterRequest) -> Result<Value, DshAdapterError> {
        if self.state != AdapterState::Active {
            return Err(DshAdapterError::Inactive);
        }
        if request.bridge_protocol != BRIDGE_PROTOCOL {
            return Err(DshAdapterError::BridgeProtocolMismatch);
        }
        if request.dsh_version != self.config.expected_dsh_version {
            return Err(DshAdapterError::DshVersionMismatch);
        }
        if request.schema_digest != self.config.expected_schema_digest {
            return Err(DshAdapterError::SchemaDigestMismatch);
        }
        if request.fencing_epoch != self.config.expected_fencing_epoch {
            return Err(DshAdapterError::StaleFencingEpoch);
        }
        if request.session_id.trim().is_empty()
            || request.plugin_id.trim().is_empty()
            || request.correlation_id.trim().is_empty()
            || request.deadline.trim().is_empty()
            || request.event.operation.trim().is_empty()
        {
            return Err(DshAdapterError::MissingIdentity);
        }
        if self.session_id.as_deref() != Some(request.session_id.as_str()) {
            return Err(DshAdapterError::WrongSession);
        }
        if self
            .last_sequence
            .is_some_and(|last| request.sequence <= last)
        {
            return Err(DshAdapterError::SequenceNotMonotonic);
        }
        if request.event.authority_claim {
            return Err(DshAdapterError::AuthorityClaimForbidden);
        }
        if request.event.secret_shaped {
            return Err(DshAdapterError::SecretShapedPayload);
        }
        if let Some(rejection) = payload_rejection(&request.event.payload) {
            return Err(rejection);
        }
        if request.event.payload.is_null() {
            return Err(DshAdapterError::InvalidEvent);
        }
        if let Some(declared_task) = request.task_ref.as_deref()
            && self
                .bound_task_ref
                .as_deref()
                .is_some_and(|bound| bound != declared_task)
        {
            return Err(DshAdapterError::WrongSession);
        }

        let operation = match request.event.kind {
            PluginEventKind::Candidate => "agent.candidate.observe",
            PluginEventKind::Observation => "agent.observation.append",
            PluginEventKind::Lifecycle => "agent.lifecycle.observe",
        };
        let payload = json!({
            "adapter_id": ADAPTER_ID,
            "bridge_protocol": BRIDGE_PROTOCOL,
            "dsh_version": request.dsh_version,
            "session_id": request.session_id,
            "sequence": request.sequence,
            "plugin_id": request.plugin_id,
            "event_kind": request.event.kind,
            "operation": request.event.operation,
            "authority": "candidate_only",
            "payload": request.event.payload,
        });
        let payload_digest =
            digest(&payload).map_err(|error| DshAdapterError::Akp(error.to_string()))?;
        let envelope = json!({
            "protocol_version": VERSION,
            "schema_digest": self.config.expected_schema_digest.clone(),
            "message_id": format!("dsh:{}:{}", request.session_id, request.sequence),
            "correlation_id": request.correlation_id,
            "sender": format!("urn:cognitiveos:adapter:{}", ADAPTER_ID),
            "audience": "urn:cognitiveos:daemon",
            "operation": operation,
            "deadline": request.deadline,
            "idempotency_key": format!("dsh:{}:{}", request.session_id, request.sequence),
            "payload": payload,
            "payload_digest": payload_digest,
            "extensions": [{"id": "dsh.version", "critical": false}],
        });
        self.last_sequence = Some(request.sequence);
        Ok(envelope)
    }

    pub fn workspace_candidate(
        &self,
        request: &DshAdapterRequest,
    ) -> Result<Option<WorkspaceCandidateFields>, DshAdapterError> {
        if request.event.kind != PluginEventKind::Candidate {
            return Ok(None);
        }
        let Some((tool_ref, action, operation_descriptor_id)) =
            catalog_workspace_op(request.event.operation.as_str())
        else {
            return Ok(None);
        };
        let payload = request
            .event
            .payload
            .as_object()
            .ok_or(DshAdapterError::InvalidEvent)?;
        if let Some(declared) = payload.get("tool_ref").and_then(Value::as_str)
            && declared != tool_ref
        {
            return Err(DshAdapterError::InvalidEvent);
        }
        if let Some(declared) = payload.get("action").and_then(Value::as_str)
            && declared != action
            && declared != request.event.operation
        {
            return Err(DshAdapterError::InvalidEvent);
        }
        if let Some(declared) = payload
            .get("operation_descriptor_id")
            .and_then(Value::as_str)
            && declared != operation_descriptor_id
        {
            return Err(DshAdapterError::InvalidEvent);
        }
        let target = workspace_target(payload, request.event.operation.as_str())?;
        let expected_state_version = payload
            .get("expected_state_version")
            .and_then(Value::as_i64)
            .unwrap_or(1);
        if expected_state_version < 1 {
            return Err(DshAdapterError::InvalidEvent);
        }
        let (parameters, digest_source) =
            workspace_parameters(payload, request.event.operation.as_str())?;
        let parameters_digest = match payload.get("parameters_digest").and_then(Value::as_str) {
            Some(digest) if !digest.trim().is_empty() => digest.to_owned(),
            _ => digest_candidate_parameters(&digest_source)?,
        };
        Ok(Some(WorkspaceCandidateFields {
            tool_ref: tool_ref.to_owned(),
            action: action.to_owned(),
            target,
            parameters,
            parameters_digest,
            expected_state_version,
            operation_descriptor_id: operation_descriptor_id.to_owned(),
        }))
    }
}

fn workspace_target(
    payload: &serde_json::Map<String, Value>,
    operation: &str,
) -> Result<String, DshAdapterError> {
    let raw = match payload.get("target").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => value.to_owned(),
        _ if operation == "WorkspaceSearch" => "workspace://".to_owned(),
        _ => return Err(DshAdapterError::InvalidEvent),
    };
    let target = if raw.starts_with("workspace://") {
        raw
    } else {
        format!("workspace://{raw}")
    };
    if target == "workspace://" && operation != "WorkspaceSearch" {
        return Err(DshAdapterError::InvalidEvent);
    }
    Ok(target)
}

fn workspace_parameters(
    payload: &serde_json::Map<String, Value>,
    operation: &str,
) -> Result<(Option<Value>, Value), DshAdapterError> {
    let nested = payload.get("parameters").filter(|value| !value.is_null());
    match operation {
        "WorkspaceRead" => {
            if nested.is_some() {
                return Err(DshAdapterError::InvalidEvent);
            }
            let digest_source = json!({ "family": "WorkspaceRead" });
            Ok((None, digest_source))
        }
        "WorkspaceSearch" => {
            let query = payload_string(payload, nested, "query")?;
            let parameters = json!({
                "family": "WorkspaceSearch",
                "query": query,
            });
            Ok((Some(parameters.clone()), parameters))
        }
        "WorkspaceWrite" | "WorkspacePatch" => {
            let input_b64 = payload_string(payload, nested, "input_b64")?;
            let preimage = payload_string(payload, nested, "preimage")?;
            let parameters = json!({
                "family": operation,
                "input_b64": input_b64,
                "preimage": preimage,
            });
            Ok((Some(parameters.clone()), parameters))
        }
        _ => Err(DshAdapterError::InvalidEvent),
    }
}

fn payload_string(
    payload: &serde_json::Map<String, Value>,
    nested: Option<&Value>,
    key: &str,
) -> Result<String, DshAdapterError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .or_else(|| {
            nested
                .and_then(Value::as_object)
                .and_then(|object| object.get(key))
                .and_then(Value::as_str)
        })
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(DshAdapterError::InvalidEvent)
}

fn catalog_workspace_op(operation: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match operation {
        "WorkspaceRead" => Some((
            "native.workspace.read",
            "read",
            "00000000-0000-7000-8000-000000002001",
        )),
        "WorkspaceSearch" => Some((
            "native.workspace.search",
            "search",
            "00000000-0000-7000-8000-000000002002",
        )),
        "WorkspaceWrite" => Some((
            "native.workspace.write",
            "write",
            "00000000-0000-7000-8000-000000002003",
        )),
        "WorkspacePatch" => Some((
            "native.workspace.patch",
            "patch",
            "00000000-0000-7000-8000-000000002004",
        )),
        _ => None,
    }
}

fn digest_candidate_parameters(parameters: &Value) -> Result<String, DshAdapterError> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(parameters)
        .map_err(|error| DshAdapterError::Akp(error.to_string()))?;
    cognitive_contracts::canonical::digest(&bytes, "cognitiveos.personal.candidate-parameters/0.1")
        .map_err(|error| DshAdapterError::Akp(error.to_string()))
}

fn payload_rejection(value: &Value) -> Option<DshAdapterError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "api_key" | "apikey" | "authorization" | "password" | "secret" | "token"
                ) {
                    return Some(DshAdapterError::SecretShapedPayload);
                }
                if matches!(
                    normalized.as_str(),
                    "task_ref"
                        | "authorization_id"
                        | "effect"
                        | "acceptance"
                        | "budget"
                        | "lease"
                        | "wia"
                        | "worker_authorization"
                        | "complete"
                        | "completed"
                        | "capability"
                ) {
                    return Some(DshAdapterError::ForbiddenPayloadField);
                }
                if let Some(rejection) = payload_rejection(child) {
                    return Some(rejection);
                }
            }
            None
        }
        Value::Array(values) => values.iter().find_map(payload_rejection),
        Value::String(value) if value.starts_with("sk-") || value.contains("Bearer ") => {
            Some(DshAdapterError::SecretShapedPayload)
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
    }
}

/// Build a configuration using the generated AKP request schema pin.
pub fn default_config(expected_dsh_version: impl Into<String>) -> DshAdapterConfig {
    DshAdapterConfig {
        expected_dsh_version: expected_dsh_version.into(),
        expected_schema_digest: cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST
            .to_owned(),
        expected_fencing_epoch: 1,
    }
}

/// Parse one JSONL frame, enforce the byte ceiling, and translate it.
pub fn handle_jsonl_line(
    adapter: &mut DeepSeekHarnessAdapter,
    line: &str,
    max_frame_bytes: usize,
) -> DshWireResponse {
    let frame_bytes = line.len();
    if frame_bytes > max_frame_bytes {
        return DshWireResponse::rejected(0, &DshAdapterError::FrameTooLarge);
    }
    let request: DshAdapterRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(_) => return DshWireResponse::rejected(0, &DshAdapterError::MalformedJson),
    };
    match adapter.translate(&request) {
        Ok(envelope) => DshWireResponse::accepted(request.sequence, envelope),
        Err(error) => DshWireResponse::rejected(request.sequence, &error),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn schema_digest() -> String {
        cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST.to_owned()
    }

    fn request(sequence: u64) -> DshAdapterRequest {
        DshAdapterRequest {
            bridge_protocol: BRIDGE_PROTOCOL.to_owned(),
            dsh_version: "0.1.1-rc.1".to_owned(),
            schema_digest: schema_digest(),
            session_id: "dsh-session-1".to_owned(),
            fencing_epoch: 1,
            sequence,
            plugin_id: "plugin.core".to_owned(),
            correlation_id: "corr-1".to_owned(),
            deadline: "2030-01-01T00:00:00Z".to_owned(),
            task_ref: None,
            event: DshPluginEvent {
                kind: PluginEventKind::Candidate,
                operation: "context.propose".to_owned(),
                payload: json!({"text":"candidate"}),
                authority_claim: false,
                secret_shaped: false,
            },
        }
    }

    #[test]
    fn translates_candidate_to_pinned_candidate_only_akp() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        adapter.activate("dsh-session-1").expect("activate");
        let envelope = adapter.translate(&request(1)).expect("translate");
        assert_eq!(envelope["protocol_version"], VERSION);
        assert_eq!(envelope["operation"], "agent.candidate.observe");
        assert_eq!(envelope["payload"]["authority"], "candidate_only");
        assert_eq!(envelope["idempotency_key"], "dsh:dsh-session-1:1");
        let encoded = serde_json::to_vec(&envelope).expect("encode");
        let parsed = super::super::parse_request(
            &encoded,
            cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST,
        )
        .expect("AKP parse");
        assert_eq!(parsed.operation, "agent.candidate.observe");
    }

    #[test]
    fn rejects_stale_version_sequence_and_authority_claims() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        adapter.activate("dsh-session-1").expect("activate");

        let mut wrong_version = request(1);
        wrong_version.dsh_version = "0.1.0".to_owned();
        assert_eq!(
            adapter.translate(&wrong_version).unwrap_err(),
            DshAdapterError::DshVersionMismatch
        );

        adapter.translate(&request(2)).expect("first accepted");
        assert_eq!(
            adapter.translate(&request(2)).unwrap_err(),
            DshAdapterError::SequenceNotMonotonic
        );

        let mut authority = request(3);
        authority.event.authority_claim = true;
        assert_eq!(
            adapter.translate(&authority).unwrap_err(),
            DshAdapterError::AuthorityClaimForbidden
        );
    }

    #[test]
    fn rejects_secret_shaped_payloads_and_inactive_sessions() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        assert_eq!(
            adapter.translate(&request(1)).unwrap_err(),
            DshAdapterError::Inactive
        );
        adapter.activate("dsh-session-1").expect("activate");
        let mut secret = request(1);
        secret.event.payload = json!({"api_key":"sk-test"});
        assert_eq!(
            adapter.translate(&secret).unwrap_err(),
            DshAdapterError::SecretShapedPayload
        );
    }

    #[test]
    fn rejects_wrong_protocol_digest_session_epoch_and_authority_fields() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        adapter.activate("dsh-session-1").expect("activate");

        let mut protocol = request(1);
        protocol.bridge_protocol = "other".to_owned();
        assert_eq!(
            adapter.translate(&protocol).unwrap_err(),
            DshAdapterError::BridgeProtocolMismatch
        );

        let mut digest = request(1);
        digest.schema_digest = "sha256:deadbeef".to_owned();
        assert_eq!(
            adapter.translate(&digest).unwrap_err(),
            DshAdapterError::SchemaDigestMismatch
        );

        let mut session = request(1);
        session.session_id = "other-session".to_owned();
        assert_eq!(
            adapter.translate(&session).unwrap_err(),
            DshAdapterError::WrongSession
        );

        let mut epoch = request(1);
        epoch.fencing_epoch = 9;
        assert_eq!(
            adapter.translate(&epoch).unwrap_err(),
            DshAdapterError::StaleFencingEpoch
        );

        let mut authority_field = request(1);
        authority_field.event.payload = json!({"text":"x","task_ref":"task://forged"});
        assert_eq!(
            adapter.translate(&authority_field).unwrap_err(),
            DshAdapterError::ForbiddenPayloadField
        );
    }

    #[test]
    fn jsonl_rejects_malformed_and_oversized_frames_without_authority() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        adapter.activate("dsh-session-1").expect("activate");
        let malformed = handle_jsonl_line(&mut adapter, "{not-json", MAX_FRAME_BYTES);
        assert!(!malformed.accepted);
        assert!(malformed.candidate_only);
        assert_eq!(malformed.error.as_deref(), Some("MALFORMED_JSON"));

        let oversized = "x".repeat(MAX_FRAME_BYTES + 1);
        let large = handle_jsonl_line(&mut adapter, &oversized, MAX_FRAME_BYTES);
        assert_eq!(large.error.as_deref(), Some("FRAME_TOO_LARGE"));
        assert!(large.candidate_only);
    }

    #[test]
    fn maps_workspace_read_candidate_without_completing_a_task() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        adapter
            .activate_fenced(
                "dsh-session-1",
                1,
                Some("task://personal/dsh-read".to_owned()),
            )
            .expect("activate");
        let mut read = request(1);
        read.event.operation = "WorkspaceRead".to_owned();
        read.event.payload = json!({
            "target": "README.md"
        });
        adapter.translate(&read).expect("translate");
        let fields = adapter
            .workspace_candidate(&read)
            .expect("map")
            .expect("workspace");
        assert_eq!(fields.tool_ref, "native.workspace.read");
        assert_eq!(fields.action, "read");
        assert_eq!(fields.target, "workspace://README.md");
        assert_eq!(fields.parameters, None);
        assert!(fields.parameters_digest.starts_with("sha256:"));
        assert_eq!(
            fields.operation_descriptor_id,
            "00000000-0000-7000-8000-000000002001"
        );
        assert_eq!(adapter.bound_task_ref(), Some("task://personal/dsh-read"));
    }

    #[test]
    fn maps_workspace_search_and_rejects_read_parameter_object() {
        let mut adapter =
            DeepSeekHarnessAdapter::new(default_config("0.1.1-rc.1")).expect("config");
        adapter
            .activate_fenced(
                "dsh-session-1",
                1,
                Some("task://personal/dsh-search".to_owned()),
            )
            .expect("activate");
        let mut search = request(1);
        search.event.operation = "WorkspaceSearch".to_owned();
        search.event.payload = json!({
            "query": "needle"
        });
        let fields = adapter
            .workspace_candidate(&search)
            .expect("map")
            .expect("workspace");
        assert_eq!(fields.tool_ref, "native.workspace.search");
        assert_eq!(fields.target, "workspace://");
        assert_eq!(
            fields.parameters,
            Some(json!({"family":"WorkspaceSearch","query":"needle"}))
        );

        let mut read_with_parameters = request(2);
        read_with_parameters.event.operation = "WorkspaceRead".to_owned();
        read_with_parameters.event.payload = json!({
            "target": "README.md",
            "parameters": { "family": "WorkspaceRead" }
        });
        assert_eq!(
            adapter
                .workspace_candidate(&read_with_parameters)
                .unwrap_err(),
            DshAdapterError::InvalidEvent
        );
    }
}
