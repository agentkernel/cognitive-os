//! Supervised candidate-only Pi process launcher.
//!
//! This binary deliberately has no install/commit verb. It invokes Pi with all
//! built-in tools and project-local extension surfaces disabled, then emits an
//! untrusted candidate record. A real OS sandbox and durable installation
//! authority are prerequisites for a governed AgentInstallation claim.

use cognitive_runtime::SandboxPlatform;
use cognitive_secret::{
    ProductionSecretBackend, ProviderConfigRepository, ProviderKeyService, SecretMaterial,
    select_production_secret_store,
};
use pi_agent_adapter::{
    PiCompatibilityPin, PiLaunchPolicy, observed_response_models, redact_secret,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::process::Command;
use std::time::Instant;

const LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG: &str =
    "allow-local-native-provider-secret-development";
const USAGE: &str = "pi-agent-adapter <run|evaluate> --pi <path> --model <deepseek-model> --prompt <text> --work-dir <dir> --config-dir <pi-dir> --provider-config-dir <personal-provider-config-dir> --allow-local-native-provider-secret-development [--runs <1..=20> --expected-text <text>]";

#[derive(Debug, Default, PartialEq, Eq)]
struct ParsedFlags {
    values: BTreeMap<String, String>,
    enabled_flags: BTreeSet<String>,
}

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

fn candidate_record(flags: &ParsedFlags) -> Result<Value, String> {
    let pi = required(flags, "pi")?;
    let model = required(flags, "model")?;
    let prompt = required(flags, "prompt")?;
    let work_dir = required(flags, "work-dir")?;
    let config_dir = required(flags, "config-dir")?;
    let provider_config_dir = required(flags, "provider-config-dir")?;
    require_local_native_provider_secret_development_flag(flags)?;

    // Verify all non-secret admission before resolving native secret material.
    verify_pinned_pi_version(pi)?;
    let policy = PiLaunchPolicy::deepseek_candidate(model)?;
    let args = policy.command_args(prompt)?;
    let provider_material = resolve_local_development_provider_material(provider_config_dir)?;

    let output = sanitized_command(pi, &provider_material)?
        .args(args)
        .current_dir(work_dir)
        .env("PI_CODING_AGENT_DIR", config_dir)
        .env("PI_TELEMETRY", "0")
        .output()
        .map_err(|error| format!("Pi launch failed: {error}"))?;

    let provider_key = std::str::from_utf8(provider_material.expose_bytes()).map_err(|_| {
        "native Provider secret must be valid UTF-8 for the Pi development exception".to_owned()
    })?;
    let stdout = redact_secret(&String::from_utf8_lossy(&output.stdout), provider_key);
    let stderr = redact_secret(&String::from_utf8_lossy(&output.stderr), provider_key);
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

fn require_local_native_provider_secret_development_flag(
    flags: &ParsedFlags,
) -> Result<(), String> {
    if flags
        .enabled_flags
        .contains(LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG)
    {
        Ok(())
    } else {
        Err(format!(
            "Pi Provider key delivery is disabled by default; pass --{LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG} only for the owner-approved local development exception"
        ))
    }
}

fn resolve_local_development_provider_material(
    provider_config_dir: &str,
) -> Result<SecretMaterial, String> {
    let repository = ProviderConfigRepository::under_config_dir(provider_config_dir);
    match select_production_secret_store() {
        ProductionSecretBackend::LinuxSecretTool(secret_store) => {
            let provider_key_service = ProviderKeyService::new(secret_store, repository);
            let provider_config = provider_key_service
                .load_config()
                .map_err(|error| {
                    format!("local development Provider configuration is unavailable: {error}")
                })?
                .ok_or_else(|| {
                    "local development Provider configuration is not present".to_owned()
                })?;
            if provider_config.provider_id() != "deepseek" {
                return Err(
                    "local development exception accepts only the configured deepseek Provider"
                        .to_owned(),
                );
            }
            provider_key_service
                .resolve_provider_material()
                .map_err(|error| {
                    format!("local development Provider secret is unavailable: {error}")
                })
        }
        ProductionSecretBackend::Unavailable(_) => Err(
            "local development exception requires an available Linux native Secret Service backend"
                .to_owned(),
        ),
    }
}

fn verify_pinned_pi_version(pi: &str) -> Result<(), String> {
    let output = Command::new(pi)
        .arg("--version")
        .output()
        .map_err(|error| format!("Pi version check failed to launch: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Pi version check failed with exit status {:?}",
            output.status.code()
        ));
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    PiCompatibilityPin::expected().validate_reported_version(&version_output)
}

/// Runs a bounded number of identical, no-tools candidate calls. This records
/// external-process latency only; it is explicitly not a REQ-PERF-004 campaign.
fn evaluate_candidates(flags: &ParsedFlags) -> Result<Value, String> {
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

fn parse_flags(args: &[String]) -> Result<ParsedFlags, String> {
    let mut flags = ParsedFlags::default();
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let Some(name) = flag.strip_prefix("--") else {
            return Err(format!("unexpected argument `{flag}`"));
        };
        if name == LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG {
            if !flags.enabled_flags.insert(name.to_owned()) {
                return Err(format!("flag --{name} given twice"));
            }
            continue;
        }
        let Some(value) = iter.next() else {
            return Err(format!("flag --{name} requires a value"));
        };
        if flags
            .values
            .insert(name.to_owned(), value.to_owned())
            .is_some()
        {
            return Err(format!("flag --{name} given twice"));
        }
    }
    Ok(flags)
}

fn required<'a>(flags: &'a ParsedFlags, name: &str) -> Result<&'a str, String> {
    flags
        .values
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
/// development credential; no ambient user API tokens are inherited.
fn sanitized_command(program: &str, provider_material: &SecretMaterial) -> Result<Command, String> {
    let provider_key = std::str::from_utf8(provider_material.expose_bytes()).map_err(|_| {
        "native Provider secret must be valid UTF-8 for the Pi development exception".to_owned()
    })?;
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
    // This exception scopes the key to the initial Pi process. Pi remains
    // uncontained and may pass its environment to descendants, so it is not
    // containment evidence and expires at the P2 boundary.
    command.env("DEEPSEEK_API_KEY", OsString::from(provider_key));
    Ok(command)
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
    fn development_secret_exception_requires_exact_explicit_flag() -> Result<(), String> {
        let without_exception = parse_flags(&[])?;
        assert!(require_local_native_provider_secret_development_flag(&without_exception).is_err());

        let with_exception =
            parse_flags(&[format!("--{LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG}")])?;
        assert!(require_local_native_provider_secret_development_flag(&with_exception).is_ok());

        let duplicate_exception = vec![
            format!("--{LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG}"),
            format!("--{LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG}"),
        ];
        assert!(parse_flags(&duplicate_exception).is_err());
        Ok(())
    }

    #[test]
    fn child_environment_does_not_inherit_other_api_key_names() -> Result<(), String> {
        let provider_material =
            SecretMaterial::from_bytes("test-deepseek-key").map_err(|error| error.to_string())?;
        let command = sanitized_command("pi", &provider_material)?;
        let names: Vec<String> = command
            .get_envs()
            .filter_map(|(name, value)| value.map(|_| name.to_string_lossy().into_owned()))
            .collect();
        assert!(names.iter().any(|name| name == "DEEPSEEK_API_KEY"));
        assert!(!names.iter().any(|name| name == "OPENAI_API_KEY"));
        Ok(())
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
