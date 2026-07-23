//! Supervised candidate-only Pi process launcher.
//!
//! This binary deliberately has no install/commit verb. It invokes Pi with all
//! built-in tools and project-local extension surfaces disabled, then emits an
//! untrusted candidate record. A real OS sandbox and durable installation
//! authority are prerequisites for a governed AgentInstallation claim.

use cognitive_runtime::SandboxPlatform;
use pi_agent_adapter::{PiLaunchPolicy, observed_response_models, redact_secret};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::process::Command;
use std::time::Instant;

const USAGE: &str = "pi-agent-adapter <run|evaluate> --pi <path> --model <deepseek-model> --prompt <text> --work-dir <dir> --config-dir <dir> [--runs <1..=20> --expected-text <text>]";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(record) => match serde_json::to_string(&record) {
            Ok(line) => println!("{line}"),
            Err(error) => {
                eprintln!("candidate record serialization failed: {error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("{error}\n{USAGE}");
            std::process::exit(2);
        }
    }
}

fn run(args: &[String]) -> Result<Value, String> {
    let Some((verb, rest)) = args.split_first() else {
        return Err("missing verb".to_owned());
    };
    let flags = parse_flags(rest)?;
    match verb.as_str() {
        "run" => candidate_record(&flags),
        "evaluate" => evaluate_candidates(&flags),
        _ => Err(format!("unsupported verb `{verb}`")),
    }
}

fn candidate_record(flags: &BTreeMap<String, String>) -> Result<Value, String> {
    let pi = required(flags, "pi")?;
    let model = required(flags, "model")?;
    let prompt = required(flags, "prompt")?;
    let work_dir = required(flags, "work-dir")?;
    let config_dir = required(flags, "config-dir")?;
    let key = env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "DEEPSEEK_API_KEY is required for a DeepSeek candidate run".to_owned())?;
    let policy = PiLaunchPolicy::deepseek_candidate(model)?;
    let args = policy.command_args(prompt)?;

    let output = sanitized_command(pi, &key)
        .args(args)
        .current_dir(work_dir)
        .env("PI_CODING_AGENT_DIR", config_dir)
        .env("PI_TELEMETRY", "0")
        .output()
        .map_err(|error| format!("Pi launch failed: {error}"))?;

    let stdout = redact_secret(&String::from_utf8_lossy(&output.stdout), &key);
    let stderr = redact_secret(&String::from_utf8_lossy(&output.stderr), &key);
    let observed_models = observed_response_models(&stdout);
    Ok(json!({
        "classification": policy.classification(),
        "platform": platform().as_str(),
        "authority_committed": policy.authority_committed(),
        "effects_created": policy.effects_created(),
        "provider": "deepseek",
        "requested_model": model,
        "observed_response_models": observed_models,
        "pi_exit_code": output.status.code(),
        "stdout": stdout,
        "stderr": stderr,
    }))
}

/// Runs a bounded number of identical, no-tools candidate calls. This records
/// external-process latency only; it is explicitly not a REQ-PERF-004 campaign.
fn evaluate_candidates(flags: &BTreeMap<String, String>) -> Result<Value, String> {
    let runs = required(flags, "runs")?
        .parse::<usize>()
        .map_err(|_| "--runs must be an integer".to_owned())?;
    if !(1..=20).contains(&runs) {
        return Err("--runs must be between 1 and 20".to_owned());
    }
    let expected = required(flags, "expected-text")?;
    if expected.is_empty() {
        return Err("--expected-text must not be empty".to_owned());
    }

    let mut samples = Vec::with_capacity(runs);
    let mut results = Vec::with_capacity(runs);
    let mut observed_models = BTreeSet::new();
    let mut pass_count = 0_usize;
    for index in 0..runs {
        let started = Instant::now();
        let record = candidate_record(flags)?;
        let elapsed_ms = started.elapsed().as_millis();
        let stdout = record
            .get("stdout")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let pi_exit_code = record.get("pi_exit_code").and_then(Value::as_i64);
        let expected_output = stdout.contains(expected);
        let tool_results_empty = stdout.contains("\"toolResults\":[]");
        let run_models: Vec<String> = record
            .get("observed_response_models")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        observed_models.extend(run_models.iter().cloned());
        let passed = pi_exit_code == Some(0) && expected_output && tool_results_empty;
        if passed {
            pass_count = pass_count.saturating_add(1);
        }
        samples.push(elapsed_ms);
        results.push(json!({
            "index": index + 1,
            "elapsed_ms": elapsed_ms,
            "pi_exit_code": pi_exit_code,
            "expected_output": expected_output,
            "tool_results_empty": tool_results_empty,
            "observed_response_models": run_models,
            "passed": passed,
        }));
    }

    let requested_model = required(flags, "model")?;
    Ok(json!({
        "evaluation_kind": "candidate_only_smoke_nonclaim",
        "performance_claim": "not_a_REQ-PERF-004_campaign",
        "classification": "uncontained_candidate_only",
        "platform": platform().as_str(),
        "authority_committed": false,
        "effects_created": false,
        "provider": "deepseek",
        "requested_model": requested_model,
        "observed_response_models": observed_models,
        "runs": results,
        "summary": {
            "total": runs,
            "passed": pass_count,
            "failed": runs.saturating_sub(pass_count),
            "latency_ms": {
                "p50": percentile_ms(&samples, 50),
                "p95": percentile_ms(&samples, 95),
                "p99": percentile_ms(&samples, 99),
            }
        }
    }))
}

fn percentile_ms(samples: &[u128], percentile: u8) -> Option<u128> {
    if samples.is_empty() || percentile == 0 || percentile > 100 {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted
        .len()
        .saturating_mul(usize::from(percentile))
        .saturating_add(99)
        / 100;
    sorted.get(rank.saturating_sub(1)).copied()
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut flags = BTreeMap::new();
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let Some(name) = flag.strip_prefix("--") else {
            return Err(format!("unexpected argument `{flag}`"));
        };
        let Some(value) = iter.next() else {
            return Err(format!("flag --{name} requires a value"));
        };
        if flags.insert(name.to_owned(), value.to_owned()).is_some() {
            return Err(format!("flag --{name} given twice"));
        }
    }
    Ok(flags)
}

fn required<'a>(flags: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str, String> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required flag --{name}"))
}

fn platform() -> SandboxPlatform {
    if cfg!(target_os = "windows") {
        SandboxPlatform::WindowsNative
    } else {
        SandboxPlatform::LinuxNative
    }
}

/// Child processes receive only operating-system essentials and the scoped
/// DeepSeek credential; no ambient user API tokens are inherited.
fn sanitized_command(program: &str, deepseek_key: &str) -> Command {
    let mut command = Command::new(program);
    command.env_clear();
    for key in [
        "ComSpec",
        "PATHEXT",
        "PATH",
        "SystemRoot",
        "TEMP",
        "TMP",
        "USERPROFILE",
        "WINDIR",
    ] {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
    command.env("DEEPSEEK_API_KEY", OsString::from(deepseek_key));
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_duplicate_or_positional_flags() {
        let duplicate = vec![
            "--model".to_owned(),
            "deepseek-chat".to_owned(),
            "--model".to_owned(),
            "deepseek-v4-flash".to_owned(),
        ];
        assert!(parse_flags(&duplicate).is_err());
        assert!(parse_flags(&["unexpected".to_owned()]).is_err());
    }

    #[test]
    fn child_environment_does_not_inherit_other_api_key_names() {
        let command = sanitized_command("pi", "test-deepseek-key");
        let names: Vec<String> = command
            .get_envs()
            .filter_map(|(name, value)| value.map(|_| name.to_string_lossy().into_owned()))
            .collect();
        assert!(names.iter().any(|name| name == "DEEPSEEK_API_KEY"));
        assert!(!names.iter().any(|name| name == "OPENAI_API_KEY"));
    }

    #[test]
    fn percentile_uses_nearest_rank_and_preserves_tail_samples() {
        let samples = [10_u128, 20, 30, 40, 50];
        assert_eq!(percentile_ms(&samples, 50), Some(30));
        assert_eq!(percentile_ms(&samples, 95), Some(50));
        assert_eq!(percentile_ms(&samples, 99), Some(50));
        assert_eq!(percentile_ms(&[], 50), None);
    }
}
