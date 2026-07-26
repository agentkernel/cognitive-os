//! Candidate-only boundary for invoking an external Pi process.
//!
//! Pi is an external coding-agent process. This policy deliberately does not
//! turn its output into authority, an Effect, or a completed Task. In
//! particular it disables Pi tools, project-local extensions, skills, context
//! files and session persistence. That reduction is useful for supervised
//! model evaluation, but is not an OS sandbox and must not be called C0/C1.

use serde_json::Value;
use std::collections::BTreeSet;

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
