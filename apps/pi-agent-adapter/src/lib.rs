//! Candidate-only boundary for invoking an external Pi process.
//!
//! Pi is an external coding-agent process. This policy deliberately does not
//! turn its output into authority, an Effect, or a completed Task. In
//! particular it disables Pi tools, project-local extensions, skills, context
//! files and session persistence. That reduction is useful for supervised
//! model evaluation, but is not an OS sandbox and must not be called C0/C1.

use serde_json::Value;
use std::collections::BTreeSet;

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
