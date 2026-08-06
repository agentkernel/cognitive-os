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
    DAEMON_CANDIDATE_FRAME_LIMIT, DaemonCandidateRequest, PiCompatibilityPin, PiLaunchPolicy,
    extract_daemon_candidate_response_from_pi_events, observed_response_models,
    parse_daemon_candidate_request, redact_secret,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, Instant};

const LOCAL_NATIVE_PROVIDER_SECRET_DEVELOPMENT_FLAG: &str =
    "allow-local-native-provider-secret-development";
const EXTENSION_LOAD_TIMEOUT: Duration = Duration::from_secs(30);
const P0_T06_EXTENSION_FIXTURE: &str = "apps/pi-agent-adapter/fixtures/p0_t06_extension.ts";
const P0_T06_EXTENSION_STATUS_COMMAND: &str = "/cognitiveos-p0-t06-status";
const DAEMON_CANDIDATE_PROVIDER_ID: &str = "cognitiveos-private-candidate";
const DAEMON_CANDIDATE_TIMEOUT: Duration = Duration::from_secs(60);
const USAGE: &str = "pi-agent-adapter <run|evaluate|extension-load|daemon-candidate> --pi <path> --model <model> --work-dir <dir> --config-dir <pi-dir> [--extension <candidate-extension-path> --runs <1..=20> --expected-text <text>]";

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
        "extension-load" => extension_load_record(&flags),
        "daemon-candidate" => daemon_candidate(&flags),
        _ => Err(format!("unsupported verb `{verb}`")),
    }
}

/// Invoke the daemon-private Pi candidate protocol once. The request travels
/// over stdin and the Pi prompt travels over Pi RPC stdin, never argv. This
/// path deliberately does not resolve a Provider key or read provider config;
/// the registered candidate extension must use the daemon-created private
/// completion socket supplied by its environment.
fn daemon_candidate(flags: &ParsedFlags) -> Result<Value, String> {
    let pi = required(flags, "pi")?;
    let model = required(flags, "model")?;
    let work_dir = required(flags, "work-dir")?;
    let config_dir = required(flags, "config-dir")?;
    let extension = required(flags, "extension")?;
    let request = read_daemon_candidate_request()?;

    verify_pinned_pi_version(pi)?;
    if !std::path::Path::new(extension).is_file() {
        return Err("daemon candidate extension file is missing".to_owned());
    }
    let mut child = candidate_only_command(pi)
        .arg("--provider")
        .arg(DAEMON_CANDIDATE_PROVIDER_ID)
        .arg("--model")
        .arg(model)
        // The pinned Pi CLI uses `-e` for an explicitly selected extension;
        // discovery remains disabled separately by `--no-extensions`.
        .arg("-e")
        .arg(extension)
        .arg("--no-tools")
        .arg("--no-extensions")
        .arg("--no-skills")
        .arg("--no-context-files")
        .arg("--no-session")
        .arg("--no-approve")
        .arg("--mode")
        .arg("rpc")
        .current_dir(work_dir)
        // A disposable daemon-created directory prevents Pi from discovering
        // a user's home configuration while its environment is allowlisted.
        .env("HOME", config_dir)
        .env("PI_CODING_AGENT_DIR", config_dir)
        .env("COGNITIVEOS_PRIVATE_COMPLETION_MODEL", model)
        .env("PI_TELEMETRY", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("daemon candidate Pi launch failed: {error}"))?;
    let stdout_reader = collect_child_stream(
        child
            .stdout
            .take()
            .ok_or_else(|| "daemon candidate Pi stdout was not captured".to_owned())?
            .take((DAEMON_CANDIDATE_FRAME_LIMIT + 1) as u64),
    );
    let stderr_reader = collect_child_stream(
        child
            .stderr
            .take()
            .ok_or_else(|| "daemon candidate Pi stderr was not captured".to_owned())?
            .take((DAEMON_CANDIDATE_FRAME_LIMIT + 1) as u64),
    );
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "daemon candidate Pi stdin was not captured".to_owned())?;
    write_rpc_prompt(&mut stdin, "private-candidate", &candidate_prompt(&request))?;
    drop(stdin);

    let started = Instant::now();
    let exit_status = wait_for_candidate_exit(&mut child, started)?;
    let stdout = join_child_stream(stdout_reader, "stdout")?;
    let _stderr = join_child_stream(stderr_reader, "stderr")?;
    if stdout.len() > DAEMON_CANDIDATE_FRAME_LIMIT {
        return Err("daemon candidate Pi stdout exceeds transport limit".to_owned());
    }
    if !exit_status.success() {
        return Err("daemon candidate Pi exited unsuccessfully".to_owned());
    }
    let response = extract_daemon_candidate_response_from_pi_events(
        std::str::from_utf8(&stdout)
            .map_err(|_| "daemon candidate Pi emitted non-UTF-8 output".to_owned())?,
    )?;
    serde_json::to_value(response)
        .map_err(|error| format!("daemon candidate response serialization failed: {error}"))
}

fn read_daemon_candidate_request() -> Result<DaemonCandidateRequest, String> {
    let mut frame = Vec::new();
    std::io::stdin()
        .take((DAEMON_CANDIDATE_FRAME_LIMIT + 1) as u64)
        .read_to_end(&mut frame)
        .map_err(|error| format!("daemon candidate request read failed: {error}"))?;
    parse_daemon_candidate_request(&frame)
}

fn candidate_prompt(request: &DaemonCandidateRequest) -> String {
    format!(
        "Return exactly one JSON object and no Markdown or prose. Its only fields must be tool_ref (string), action (string), target (string), parameters_digest (string), expected_state_version (integer >= 0), and operation_descriptor_id (string). Do not invoke tools. Context follows:\n{}",
        request.rendered_context
    )
}

fn wait_for_candidate_exit(
    child: &mut std::process::Child,
    started: Instant,
) -> Result<std::process::ExitStatus, String> {
    loop {
        match child
            .try_wait()
            .map_err(|error| format!("daemon candidate Pi wait failed: {error}"))?
        {
            Some(status) => return Ok(status),
            None if started.elapsed() >= DAEMON_CANDIDATE_TIMEOUT => {
                child
                    .kill()
                    .map_err(|error| format!("daemon candidate Pi timeout kill failed: {error}"))?;
                return child
                    .wait()
                    .map_err(|error| format!("daemon candidate Pi timeout wait failed: {error}"));
            }
            None => sleep(Duration::from_millis(25)),
        }
    }
}

/// Attempts one real pinned Pi Extension session without changing the
/// candidate-only launcher policy. This is local PoC evidence only: the child
/// receives the approved development credential exception and remains
/// uncontained. No authority or Effect surface is available in this mode.
fn extension_load_record(flags: &ParsedFlags) -> Result<Value, String> {
    let pi = required(flags, "pi")?;
    let model = required(flags, "model")?;
    let prompt = required(flags, "prompt")?;
    let work_dir = required(flags, "work-dir")?;
    let config_dir = required(flags, "config-dir")?;
    let provider_config_dir = required(flags, "provider-config-dir")?;
    require_local_native_provider_secret_development_flag(flags)?;
    verify_pinned_pi_version(pi)?;
    let fixture = required_fixture_path(flags)?;
    if prompt != P0_T06_EXTENSION_STATUS_COMMAND {
        return Err(format!(
            "P0-T06 evidence mode requires --prompt {P0_T06_EXTENSION_STATUS_COMMAND}"
        ));
    }
    let provider_material = resolve_local_development_provider_material(provider_config_dir)?;
    let mut child = sanitized_command(pi, &provider_material)?
        .arg("-e")
        .arg(fixture)
        .arg("--provider")
        .arg("deepseek")
        .arg("--model")
        .arg(model)
        .arg("--no-tools")
        // Explicit -e remains enabled while discovery of user or project
        // extensions is disabled by the pinned Pi CLI.
        .arg("--no-extensions")
        .arg("--no-skills")
        .arg("--no-context-files")
        .arg("--no-session")
        .arg("--no-approve")
        .arg("--mode")
        .arg("rpc")
        .current_dir(work_dir)
        .env("PI_CODING_AGENT_DIR", config_dir)
        .env("PI_TELEMETRY", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Pi Extension session failed to launch: {error}"))?;

    let stdout_reader = collect_child_stream(
        child
            .stdout
            .take()
            .ok_or_else(|| "Pi Extension session stdout was not captured".to_owned())?,
    );
    let stderr_reader = collect_child_stream(
        child
            .stderr
            .take()
            .ok_or_else(|| "Pi Extension session stderr was not captured".to_owned())?,
    );
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Pi Extension session stdin was not captured".to_owned())?;
    write_rpc_probe_command(&mut stdin, "p0-t06-commands", "get_commands")?;
    write_rpc_probe_command(&mut stdin, "p0-t06-state", "get_state")?;
    write_rpc_prompt(&mut stdin, "p0-t06-status", prompt)?;
    drop(stdin);

    let started = Instant::now();
    let (exit_status, timed_out) = loop {
        match child
            .try_wait()
            .map_err(|error| format!("Pi Extension session wait failed: {error}"))?
        {
            Some(status) => break (status, false),
            None if started.elapsed() >= EXTENSION_LOAD_TIMEOUT => {
                child.kill().map_err(|error| {
                    format!("Pi Extension session timeout kill failed: {error}")
                })?;
                let status = child.wait().map_err(|error| {
                    format!("Pi Extension session timeout wait failed: {error}")
                })?;
                break (status, true);
            }
            None => sleep(Duration::from_millis(25)),
        }
    };
    let stdout_bytes = join_child_stream(stdout_reader, "stdout")?;
    let stderr_bytes = join_child_stream(stderr_reader, "stderr")?;
    let raw_stdout = String::from_utf8_lossy(&stdout_bytes);
    let rpc_records = pi_agent_adapter::parse_rpc_jsonl_records(&raw_stdout)
        .map_err(|error| format!("Pi Extension session emitted invalid RPC JSONL: {error}"))?;
    let provider_key = std::str::from_utf8(provider_material.expose_bytes()).map_err(|_| {
        "native Provider secret must be valid UTF-8 for the Pi development exception".to_owned()
    })?;
    let stdout = redact_secret(&raw_stdout, provider_key);
    let stderr = redact_secret(&String::from_utf8_lossy(&stderr_bytes), provider_key);
    let extension_command_registered = rpc_records.iter().any(|record| {
        record["id"] == "p0-t06-commands"
            && record["type"] == "response"
            && record["command"] == "get_commands"
            && record["success"] == true
            && record["data"]["commands"]
                .as_array()
                .is_some_and(|commands| {
                    commands.iter().any(|command| {
                        command["name"] == "cognitiveos-p0-t06-status"
                            && command["source"] == "extension"
                    })
                })
    });
    let session_start_hook_observed = rpc_records.iter().any(|record| {
        record["type"] == "extension_ui_request"
            && record["method"] == "setStatus"
            && record["statusKey"] == "cognitiveos-p0-t06"
    });
    let status_command_observed = rpc_records.iter().any(|record| {
        record["id"] == "p0-t06-status"
            && record["type"] == "response"
            && record["command"] == "prompt"
            && record["success"] == true
    });
    Ok(json!({
        "evidence_kind": "p0_t06_extension_session_load",
        "status": if timed_out { "timeout" } else if exit_status.success() { "executed" } else { "failed" },
        "classification": "uncontained_candidate_only",
        "platform": platform().as_str(),
        "fixture": P0_T06_EXTENSION_FIXTURE,
        "extension_load_attempted": true,
        "extension_command_registered": extension_command_registered,
        "session_process_exited": !timed_out,
        "session_start_hook_observed": session_start_hook_observed,
        "status_command_observed": status_command_observed,
        "project_trust_hook_observed": false,
        "mutating_tool_hooks_observed": false,
        "raw_output_included": false,
        "output_redacted": true,
        "pi_exit_code": exit_status.code(),
        "stdout_length": stdout.len(),
        "stderr_length": stderr.len(),
        "authority_committed": false,
        "effects_created": false,
        "task_transitions": 0,
        "capabilities_granted": 0,
        "non_claims": ["project trust and mutating tool runtime observations not established", "no containment", "no Profile or release claim"]
    }))
}

fn collect_child_stream<R>(mut stream: R) -> JoinHandle<Result<Vec<u8>, std::io::Error>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = Vec::new();
        stream.read_to_end(&mut output)?;
        Ok(output)
    })
}

fn join_child_stream(
    reader: JoinHandle<Result<Vec<u8>, std::io::Error>>,
    stream_name: &str,
) -> Result<Vec<u8>, String> {
    reader
        .join()
        .map_err(|_| format!("Pi Extension session {stream_name} reader panicked"))?
        .map_err(|error| format!("Pi Extension session {stream_name} read failed: {error}"))
}

fn write_rpc_probe_command(
    stdin: &mut std::process::ChildStdin,
    request_id: &str,
    command_type: &str,
) -> Result<(), String> {
    let record = json!({ "id": request_id, "type": command_type });
    write_rpc_record(stdin, &record)
}

fn write_rpc_prompt(
    stdin: &mut std::process::ChildStdin,
    request_id: &str,
    message: &str,
) -> Result<(), String> {
    let record = json!({ "id": request_id, "type": "prompt", "message": message });
    write_rpc_record(stdin, &record)
}

fn write_rpc_record(stdin: &mut std::process::ChildStdin, record: &Value) -> Result<(), String> {
    let mut line = serde_json::to_vec(record)
        .map_err(|error| format!("cannot serialize Pi RPC probe command: {error}"))?;
    line.push(b'\n');
    stdin
        .write_all(&line)
        .and_then(|()| stdin.flush())
        .map_err(|error| format!("cannot write Pi RPC probe command: {error}"))
}

fn required_fixture_path(flags: &ParsedFlags) -> Result<String, String> {
    let fixture = flags
        .values
        .get("fixture")
        .map(String::as_str)
        .unwrap_or(P0_T06_EXTENSION_FIXTURE);
    if fixture != P0_T06_EXTENSION_FIXTURE {
        return Err(format!(
            "P0-T06 evidence mode only permits the pinned fixture `{P0_T06_EXTENSION_FIXTURE}`"
        ));
    }
    let fixture_path = std::env::current_dir()
        .map_err(|error| format!("cannot resolve repository root for Pi fixture: {error}"))?
        .join(fixture);
    if !fixture_path.is_file() {
        return Err(format!(
            "pinned Pi Extension fixture is missing: {}",
            fixture_path.display()
        ));
    }
    fixture_path
        .canonicalize()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| format!("cannot canonicalize Pi Extension fixture: {error}"))
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
    require_local_native_provider_secret_host()?;
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

fn require_local_native_provider_secret_host() -> Result<(), String> {
    validate_local_native_provider_secret_host(
        platform(),
        environment_value_enables_ci(env::var_os("CI").as_deref()),
    )
}

fn validate_local_native_provider_secret_host(
    host_platform: SandboxPlatform,
    ci_enabled: bool,
) -> Result<(), String> {
    if ci_enabled {
        return Err("local development Provider secret delivery is forbidden in CI".to_owned());
    }

    match host_platform {
        SandboxPlatform::LinuxNative => Ok(()),
        SandboxPlatform::WindowsWsl2LinuxGuest => Err(
            "local development Provider secret delivery requires Linux native and rejects WSL2"
                .to_owned(),
        ),
        SandboxPlatform::WindowsNative => Err(
            "local development Provider secret delivery requires Linux native and rejects Windows"
                .to_owned(),
        ),
    }
}

fn environment_value_enables_ci(value: Option<&OsStr>) -> bool {
    value.is_some_and(|value| {
        let normalized_value = value.to_string_lossy().trim().to_ascii_lowercase();
        !normalized_value.is_empty() && normalized_value != "0" && normalized_value != "false"
    })
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
    let proc_version = std::fs::read_to_string("/proc/version").unwrap_or_default();
    let kernel_release = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    let has_wsl_environment =
        env::var_os("WSL_DISTRO_NAME").is_some() || env::var_os("WSL_INTEROP").is_some();
    classify_platform(
        cfg!(target_os = "windows"),
        &proc_version,
        &kernel_release,
        has_wsl_environment,
    )
}

fn classify_platform(
    is_windows_native: bool,
    proc_version: &str,
    kernel_release: &str,
    has_wsl_environment: bool,
) -> SandboxPlatform {
    if is_windows_native {
        return SandboxPlatform::WindowsNative;
    }

    let linux_platform_description =
        format!("{proc_version}\n{kernel_release}").to_ascii_lowercase();
    let kernel_reports_wsl = linux_platform_description.contains("microsoft")
        || linux_platform_description.contains("wsl");
    if has_wsl_environment || kernel_reports_wsl {
        SandboxPlatform::WindowsWsl2LinuxGuest
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

/// Build the daemon candidate child environment without forwarding ambient
/// credentials. The private completion socket path is deliberately not set
/// here: only the daemon composition may provide it after it has created a
/// one-shot completion listener.
fn candidate_only_command(program: &str) -> Command {
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
    // This is a daemon-created, one-shot Unix-domain completion endpoint, not
    // a session bearer or Provider credential. Its lifetime and single-use
    // enforcement remain daemon-owned.
    if let Some(socket_path) = env::var_os("COGNITIVEOS_PRIVATE_COMPLETION_SOCKET") {
        command.env("COGNITIVEOS_PRIVATE_COMPLETION_SOCKET", socket_path);
    }
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
    fn development_secret_host_classification_rejects_wsl_and_windows() {
        assert_eq!(
            classify_platform(false, "Linux version 6.8.0", "6.8.0-generic", false),
            SandboxPlatform::LinuxNative
        );
        assert_eq!(
            classify_platform(
                false,
                "Linux version 6.6.87.2-microsoft-standard-WSL2",
                "6.6.87.2-microsoft-standard-WSL2",
                false,
            ),
            SandboxPlatform::WindowsWsl2LinuxGuest
        );
        assert_eq!(
            classify_platform(false, "Linux version 6.8.0", "6.8.0-generic", true),
            SandboxPlatform::WindowsWsl2LinuxGuest
        );
        assert_eq!(
            classify_platform(true, "", "", false),
            SandboxPlatform::WindowsNative
        );
    }

    #[test]
    fn development_secret_host_classification_rejects_ci_values() {
        assert!(!environment_value_enables_ci(None));
        assert!(!environment_value_enables_ci(Some(OsStr::new(""))));
        assert!(!environment_value_enables_ci(Some(OsStr::new("false"))));
        assert!(!environment_value_enables_ci(Some(OsStr::new("0"))));
        assert!(environment_value_enables_ci(Some(OsStr::new("true"))));
        assert!(environment_value_enables_ci(Some(OsStr::new("1"))));

        assert!(
            validate_local_native_provider_secret_host(SandboxPlatform::LinuxNative, false).is_ok()
        );
        assert!(
            validate_local_native_provider_secret_host(SandboxPlatform::LinuxNative, true).is_err()
        );
        assert!(
            validate_local_native_provider_secret_host(
                SandboxPlatform::WindowsWsl2LinuxGuest,
                false,
            )
            .is_err()
        );
        assert!(
            validate_local_native_provider_secret_host(SandboxPlatform::WindowsNative, false)
                .is_err()
        );
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
    fn daemon_candidate_child_does_not_receive_provider_or_authority_environment() {
        let command = candidate_only_command("pi");
        let names: Vec<String> = command
            .get_envs()
            .filter_map(|(name, value)| value.map(|_| name.to_string_lossy().into_owned()))
            .collect();
        for forbidden in [
            "DEEPSEEK_API_KEY",
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "COGNITIVEOS_BOOTSTRAP",
            "COGNITIVEOS_MANAGEMENT_BEARER",
        ] {
            assert!(
                !names.iter().any(|name| name == forbidden),
                "{forbidden} must not reach the daemon candidate child"
            );
        }
    }

    #[test]
    fn private_candidate_prompt_describes_the_complete_untrusted_schema() {
        let request = DaemonCandidateRequest {
            protocol: "cognitiveos.private-candidate/1".to_owned(),
            task_ref: "task://personal/test".to_owned(),
            contract_epoch: 1,
            rendered_context: "untrusted Context text".to_owned(),
        };
        let prompt = candidate_prompt(&request);
        for field in [
            "tool_ref",
            "action",
            "target",
            "parameters_digest",
            "expected_state_version",
            "operation_descriptor_id",
        ] {
            assert!(prompt.contains(field), "prompt must name {field}");
        }
        assert!(prompt.contains("untrusted Context text"));
        assert!(!prompt.contains("wia"));
        assert!(!prompt.contains("effect"));
    }

    #[test]
    fn percentile_uses_nearest_rank_and_preserves_tail_samples() {
        let samples = [10_u128, 20, 30, 40, 50];
        assert_eq!(percentile_ms(&samples, 50), Some(30));
        assert_eq!(percentile_ms(&samples, 95), Some(50));
        assert_eq!(percentile_ms(&samples, 99), Some(50));
        assert_eq!(percentile_ms(&[], 50), None);
    }

    #[test]
    fn extension_evidence_mode_rejects_unpinned_fixture_paths() -> Result<(), String> {
        let flags = parse_flags(&[
            "--fixture".to_owned(),
            "fixtures/unreviewed-extension.ts".to_owned(),
        ])?;
        let error = match required_fixture_path(&flags) {
            Ok(_) => return Err("unreviewed fixture must fail".to_owned()),
            Err(error) => error,
        };
        assert!(error.contains("only permits the pinned fixture"));
        Ok(())
    }

    #[test]
    fn extension_evidence_mode_uses_only_the_registered_status_command() {
        assert_eq!(
            P0_T06_EXTENSION_STATUS_COMMAND,
            "/cognitiveos-p0-t06-status"
        );
    }
}
