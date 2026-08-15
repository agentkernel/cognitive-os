//! Candidate-only boundary for invoking an external Pi process.
//!
//! Pi is an external coding-agent process. This policy deliberately does not
//! turn its output into authority, an Effect, or a completed Task. In
//! particular it disables Pi tools, project-local extensions, skills, context
//! files and session persistence. That reduction is useful for supervised
//! model evaluation, but is not an OS sandbox and must not be called C0/C1.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// Maximum size of one daemon-to-adapter candidate frame.
pub const DAEMON_CANDIDATE_FRAME_LIMIT: usize = 256 * 1024;

/// Bounded request accepted by the daemon-private adapter protocol.
///
/// This request contains only candidate-generation data. It deliberately does
/// not contain a session bearer, bootstrap material, Provider credential,
/// worker authorization, Effect, or any other daemon authority fact.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DaemonCandidateRequest {
    pub protocol: String,
    pub task_ref: String,
    pub contract_epoch: i64,
    pub rendered_context: String,
}

/// The only response shape that the daemon-private adapter may emit.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DaemonCandidateResponse {
    pub tool_ref: String,
    pub action: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    pub parameters_digest: String,
    pub expected_state_version: i64,
    pub operation_descriptor_id: String,
}

/// Parse exactly one bounded JSON frame from the adapter boundary.
pub fn parse_daemon_candidate_request(frame: &[u8]) -> Result<DaemonCandidateRequest, String> {
    if frame.is_empty() {
        return Err("daemon candidate request is empty".to_owned());
    }
    if frame.len() > DAEMON_CANDIDATE_FRAME_LIMIT {
        return Err("daemon candidate request exceeds transport limit".to_owned());
    }
    let request: DaemonCandidateRequest = serde_json::from_slice(frame)
        .map_err(|error| format!("daemon candidate request is invalid: {error}"))?;
    if request.protocol != "cognitiveos.private-candidate/1" {
        return Err("daemon candidate request declares an unsupported protocol".to_owned());
    }
    if request.task_ref.trim().is_empty() {
        return Err("daemon candidate request task_ref is empty".to_owned());
    }
    if request.contract_epoch < 1 {
        return Err("daemon candidate request contract_epoch is invalid".to_owned());
    }
    if request.rendered_context.is_empty() {
        return Err("daemon candidate request rendered_context is empty".to_owned());
    }
    Ok(request)
}

/// Parse exactly one bounded JSON response from the adapter boundary.
pub fn parse_daemon_candidate_response(frame: &[u8]) -> Result<DaemonCandidateResponse, String> {
    if frame.is_empty() {
        return Err("daemon candidate response is empty".to_owned());
    }
    if frame.len() > DAEMON_CANDIDATE_FRAME_LIMIT {
        return Err("daemon candidate response exceeds transport limit".to_owned());
    }
    let response: DaemonCandidateResponse = serde_json::from_slice(frame)
        .map_err(|error| format!("daemon candidate response is invalid: {error}"))?;
    if response.tool_ref.trim().is_empty()
        || response.action.trim().is_empty()
        || response.target.trim().is_empty()
        || response.parameters_digest.trim().is_empty()
        || response.operation_descriptor_id.trim().is_empty()
    {
        return Err("daemon candidate response contains an empty field".to_owned());
    }
    if response.expected_state_version < 1 {
        return Err("daemon candidate response expected_state_version is invalid".to_owned());
    }
    Ok(response)
}

/// Extract one strict candidate response from Pi's documented JSON print-mode
/// event stream. Pi may emit lifecycle and streaming events, but only one
/// finalized assistant `message_end` payload is eligible to carry the opaque
/// candidate JSON. Any tool event, non-text block, duplicate final message, or
/// surrounding prose fails closed.
pub fn extract_daemon_candidate_response_from_pi_events(
    event_stream: &str,
) -> Result<DaemonCandidateResponse, String> {
    let events = parse_rpc_jsonl_records(event_stream)
        .map_err(|error| format!("Pi candidate event stream is invalid: {error}"))?;
    let mut candidate_payload: Option<String> = None;

    for event in events {
        let event_type = event.get("type").and_then(Value::as_str);
        if matches!(
            event_type,
            Some("tool_execution_start" | "tool_execution_update" | "tool_execution_end")
        ) {
            return Err("Pi candidate event stream attempted a tool operation".to_owned());
        }
        if event_type != Some("message_end") {
            continue;
        }

        let message = event
            .get("message")
            .and_then(Value::as_object)
            .ok_or_else(|| "Pi candidate final message is malformed".to_owned())?;
        if message.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let content = message
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| "Pi candidate final message content is malformed".to_owned())?;
        if content.len() != 1 {
            return Err("Pi candidate final message must contain one text block".to_owned());
        }
        let payload = content[0]
            .as_object()
            .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
            .and_then(|block| block.get("text").and_then(Value::as_str))
            .ok_or_else(|| "Pi candidate final message must contain one text block".to_owned())?;
        if candidate_payload.replace(payload.to_owned()).is_some() {
            return Err("Pi candidate event stream has multiple final responses".to_owned());
        }
    }

    let payload = candidate_payload
        .ok_or_else(|| "Pi candidate event stream has no final assistant response".to_owned())?;
    parse_daemon_candidate_response(payload.as_bytes())
        .map_err(|error| format!("Pi candidate final response is invalid: {error}"))
}

/// Immutable metadata for the Pi release reviewed by P0-T06.
///
/// The package integrity and source commit are recorded compatibility pins.
/// They are not a substitute for the trusted provenance verifier required
/// before a governed `AgentInstallation` claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiCompatibilityPin {
    package_version: &'static str,
    npm_integrity: &'static str,
    source_commit: &'static str,
    repository_url: &'static str,
    repository_directory: &'static str,
    node_engine: &'static str,
}

impl PiCompatibilityPin {
    /// Returns the only Pi release accepted by this candidate-only adapter.
    pub const fn expected() -> Self {
        Self {
            package_version: "0.81.1",
            npm_integrity: "sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==",
            source_commit: "20be4b18d4c57487f8993d2762bace129f0cf7c6",
            repository_url: "https://github.com/earendil-works/pi.git",
            repository_directory: "packages/coding-agent",
            node_engine: ">=22.19.0",
        }
    }

    /// Pinned npm package version.
    pub const fn package_version(&self) -> &'static str {
        self.package_version
    }

    /// Pinned npm Subresource Integrity value.
    pub const fn npm_integrity(&self) -> &'static str {
        self.npm_integrity
    }

    /// Source commit reported by the reviewed npm package metadata.
    pub const fn source_commit(&self) -> &'static str {
        self.source_commit
    }

    /// Canonical source repository for the reviewed package.
    pub const fn repository_url(&self) -> &'static str {
        self.repository_url
    }

    /// Source subdirectory published as the reviewed npm package.
    pub const fn repository_directory(&self) -> &'static str {
        self.repository_directory
    }

    /// Minimum Node.js engine declared by the reviewed package.
    pub const fn node_engine(&self) -> &'static str {
        self.node_engine
    }

    /// Rejects an external Pi binary whose reported version differs from the
    /// reviewed candidate-only compatibility pin.
    pub fn validate_reported_version(&self, version_output: &str) -> Result<(), String> {
        let reported_version = version_output
            .split_ascii_whitespace()
            .find(|token| is_semver_like(token))
            .ok_or_else(|| "Pi version command returned no semantic version".to_owned())?;

        if reported_version == self.package_version {
            Ok(())
        } else {
            Err(format!(
                "Pi version mismatch: expected {}, reported {reported_version}",
                self.package_version
            ))
        }
    }
}

fn is_semver_like(value: &str) -> bool {
    let mut segments = value.split('.');
    matches!(
        (segments.next(), segments.next(), segments.next(), segments.next()),
        (Some(major), Some(minor), Some(patch), None)
            if !major.is_empty()
                && !minor.is_empty()
                && !patch.is_empty()
                && major.bytes().all(|byte| byte.is_ascii_digit())
                && minor.bytes().all(|byte| byte.is_ascii_digit())
                && patch.bytes().all(|byte| byte.is_ascii_digit())
    )
}

/// Parses Pi RPC records without using generic Unicode-aware line splitting.
///
/// Pi RPC is JSONL framed by LF. The pinned protocol accepts CRLF input by
/// removing only the CR immediately before an LF, while preserving U+2028 and
/// U+2029 when they occur inside valid JSON strings.
pub fn parse_rpc_jsonl_records(input: &str) -> Result<Vec<Value>, String> {
    if input.is_empty() {
        return Err("Pi RPC JSONL input must contain at least one record".to_owned());
    }

    let mut records = Vec::new();
    for raw_record in input.split_terminator('\n') {
        let record = raw_record.strip_suffix('\r').unwrap_or(raw_record);
        if record.contains('\r') {
            return Err("Pi RPC JSONL records must use LF delimiters".to_owned());
        }
        if record.is_empty() {
            return Err("Pi RPC JSONL does not permit empty records".to_owned());
        }

        let parsed_record: Value = serde_json::from_str(record)
            .map_err(|error| format!("Pi RPC JSONL record is invalid JSON: {error}"))?;
        if !parsed_record.is_object() {
            return Err("Pi RPC JSONL records must be JSON objects".to_owned());
        }
        records.push(parsed_record);
    }

    if records.is_empty() {
        return Err("Pi RPC JSONL input must contain at least one record".to_owned());
    }
    Ok(records)
}

/// Fixed launch policy for a DeepSeek-backed Pi candidate invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiLaunchPolicy {
    args: Vec<String>,
}

impl PiLaunchPolicy {
    /// Builds the only launch form supported by the reference adapter.
    ///
    /// Model identifiers remain explicit so the evidence caller can record
    /// exactly what was evaluated. Provider prefixes from another provider
    /// are rejected before a child process is created.
    pub fn deepseek_candidate(model: &str) -> Result<Self, String> {
        if model.is_empty() {
            return Err("DeepSeek model identifier must not be empty".to_owned());
        }
        if !model.starts_with("deepseek-") {
            return Err(
                "candidate-only adapter accepts DeepSeek model identifiers only".to_owned(),
            );
        }
        Ok(Self {
            args: vec![
                "--provider".to_owned(),
                "deepseek".to_owned(),
                "--model".to_owned(),
                model.to_owned(),
                "--no-tools".to_owned(),
                "--no-extensions".to_owned(),
                "--no-skills".to_owned(),
                "--no-context-files".to_owned(),
                "--no-session".to_owned(),
                "--no-approve".to_owned(),
                "--mode".to_owned(),
                "json".to_owned(),
                "--print".to_owned(),
            ],
        })
    }

    /// Arguments to place before the caller-provided prompt.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns the complete Pi argument list, with the prompt as one final
    /// argument rather than shell-interpolated command text.
    pub fn command_args(&self, prompt: &str) -> Result<Vec<String>, String> {
        if prompt.is_empty() {
            return Err("candidate prompt must not be empty".to_owned());
        }
        let mut args = self.args.clone();
        args.push(prompt.to_owned());
        Ok(args)
    }

    /// External model output is only a candidate; it has no authority state.
    pub const fn authority_committed(&self) -> bool {
        false
    }

    /// This boundary never creates an Effect.
    pub const fn effects_created(&self) -> bool {
        false
    }

    /// Honest compatibility label: no OS containment evidence is implied.
    pub const fn classification(&self) -> &'static str {
        "uncontained_candidate_only"
    }
}

/// Removes a process-scoped credential from captured child output before it
/// reaches a caller, test artifact, or diagnostic.
pub fn redact_secret(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        return text.to_owned();
    }
    text.replace(secret, "[REDACTED]")
}

/// Reads Pi's JSON event stream to record the model actually named by the
/// provider response. Request aliases are not treated as measurement facts.
pub fn observed_response_models(output: &str) -> Vec<String> {
    let mut models = BTreeSet::new();
    for line in output.lines() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            collect_response_models(&value, &mut models);
        }
    }
    models.into_iter().collect()
}

fn collect_response_models(value: &Value, models: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(model)) = map.get("responseModel") {
                models.insert(model.clone());
            }
            for nested in map.values() {
                collect_response_models(nested, models);
            }
        }
        Value::Array(values) => {
            for nested in values {
                collect_response_models(nested, models);
            }
        }
        _ => {}
    }
}
