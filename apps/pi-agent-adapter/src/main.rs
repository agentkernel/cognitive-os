//! Supervised candidate-only Pi process launcher.
//!
//! This binary deliberately has no install/commit verb. It invokes Pi with all
//! built-in tools and project-local extension surfaces disabled, then emits an
//! untrusted candidate record. A real OS sandbox and durable installation
//! authority are prerequisites for a governed AgentInstallation claim.

use cognitive_runtime::SandboxPlatform;
use pi_agent_adapter::{
    DAEMON_CANDIDATE_FRAME_LIMIT, DaemonCandidateRequest, PiCompatibilityPin,
    extract_daemon_candidate_response_from_pi_events, parse_daemon_candidate_request,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, Instant};

/// Former ADR-0018 owner-approved local-native Provider secret development
/// exception. P2-T08 removes it: residual use must fail closed.
const EXPIRED_LOCAL_NATIVE_PROVIDER_SECRET_FLAG: &str =
    "allow-local-native-provider-secret-development";
const EXTENSION_LOAD_TIMEOUT: Duration = Duration::from_secs(30);
const P0_T06_EXTENSION_FIXTURE: &str = "apps/pi-agent-adapter/fixtures/p0_t06_extension.ts";
const P0_T06_EXTENSION_STATUS_COMMAND: &str = "/cognitiveos-p0-t06-status";
const DAEMON_CANDIDATE_PROVIDER_ID: &str = "cognitiveos-private-candidate";
const DAEMON_CANDIDATE_TIMEOUT: Duration = Duration::from_secs(60);
const USAGE: &str = "pi-agent-adapter <daemon-candidate|run|evaluate|extension-load> --pi <path> --model <model> --work-dir <dir> --config-dir <pi-dir> [--extension <candidate-extension-path> --runs <1..=20> --expected-text <text>]
Note: run/evaluate/extension-load no longer inject Provider secrets. Use daemon-candidate with the daemon Provider proxy.";

fn expired_local_native_provider_exception_message() -> String {
    format!(
        "ADR-0018 local-native Provider secret development exception expired at P2 end; refuse --{EXPIRED_LOCAL_NATIVE_PROVIDER_SECRET_FLAG} and direct Pi Provider-key injection. Use daemon-candidate with the daemon-owned Provider proxy"
    )
}

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
    let (stdout_reader, agent_end_receiver) = collect_candidate_event_stream(
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
    let (close_stdin_sender, close_stdin_receiver) = mpsc::channel();
    let stdin_holder = thread::spawn(move || {
        let _stdin = stdin;
        let _ = close_stdin_receiver.recv();
    });

    let started = Instant::now();
    let exit_status = wait_for_candidate_exit(
        &mut child,
        started,
        &agent_end_receiver,
        &close_stdin_sender,
    )?;
    let _ = close_stdin_sender.send(());
    let _ = stdin_holder.join();
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
        "Return exactly one JSON object and no Markdown or prose. Its only fields must be tool_ref (string), action (string), target (string), parameters_digest (string), expected_state_version (integer >= 1), and operation_descriptor_id (string). Do not invoke tools. Context follows:\n{}",
        request.rendered_context
    )
}

fn wait_for_candidate_exit(
    child: &mut std::process::Child,
    started: Instant,
    agent_end_receiver: &Receiver<()>,
    close_stdin_sender: &Sender<()>,
) -> Result<std::process::ExitStatus, String> {
    let mut agent_end_observed = false;
    loop {
        match child
            .try_wait()
            .map_err(|error| format!("daemon candidate Pi wait failed: {error}"))?
        {
            Some(status) => return Ok(status),
            None if !agent_end_observed && agent_end_receiver.try_recv().is_ok() => {
                agent_end_observed = true;
                let _ = close_stdin_sender.send(());
            }
            None if started.elapsed() >= DAEMON_CANDIDATE_TIMEOUT => {
                let _ = close_stdin_sender.send(());
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

fn collect_candidate_event_stream<R>(
    stream: R,
) -> (JoinHandle<Result<Vec<u8>, std::io::Error>>, Receiver<()>)
where
    R: Read + Send + 'static,
{
    let (agent_end_sender, agent_end_receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut output = Vec::new();
        let mut buffered_stream = BufReader::new(stream);
        loop {
            let mut line = Vec::new();
            let bytes_read = buffered_stream.read_until(b'\n', &mut line)?;
            if bytes_read == 0 {
                return Ok(output);
            }
            if output.len().saturating_add(line.len()) > DAEMON_CANDIDATE_FRAME_LIMIT {
                output.extend_from_slice(&line);
                return Ok(output);
            }
            if serde_json::from_slice::<Value>(&line)
                .ok()
                .and_then(|record| {
                    record
                        .get("type")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .as_deref()
                == Some("agent_end")
            {
                let _ = agent_end_sender.send(());
            }
            output.extend_from_slice(&line);
        }
    });
    (reader, agent_end_receiver)
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

/// Former local PoC verbs that injected Provider secrets into Pi.
/// P2-T08/D02 expires the ADR-0018 exception; callers must use daemon-candidate.
fn extension_load_record(_flags: &ParsedFlags) -> Result<Value, String> {
    let _ = (
        EXTENSION_LOAD_TIMEOUT,
        P0_T06_EXTENSION_FIXTURE,
        P0_T06_EXTENSION_STATUS_COMMAND,
    );
    Err(expired_local_native_provider_exception_message())
}

fn candidate_record(_flags: &ParsedFlags) -> Result<Value, String> {
    Err(expired_local_native_provider_exception_message())
}

fn evaluate_candidates(_flags: &ParsedFlags) -> Result<Value, String> {
    Err(expired_local_native_provider_exception_message())
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
        if name == EXPIRED_LOCAL_NATIVE_PROVIDER_SECRET_FLAG {
            return Err(expired_local_native_provider_exception_message());
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
    fn expired_local_native_provider_exception_is_rejected() {
        let error = parse_flags(&[format!("--{EXPIRED_LOCAL_NATIVE_PROVIDER_SECRET_FLAG}")])
            .expect_err("expired exception flag must fail closed");
        assert!(error.contains("expired at P2 end"));
        assert!(candidate_record(&ParsedFlags::default())
            .expect_err("run path must fail closed")
            .contains("expired at P2 end"));
        assert!(extension_load_record(&ParsedFlags::default())
            .expect_err("extension-load path must fail closed")
            .contains("expired at P2 end"));
        assert!(evaluate_candidates(&ParsedFlags::default())
            .expect_err("evaluate path must fail closed")
            .contains("expired at P2 end"));
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
    fn expired_exception_keeps_pinned_extension_fixture_constant() {
        assert_eq!(
            P0_T06_EXTENSION_FIXTURE,
            "apps/pi-agent-adapter/fixtures/p0_t06_extension.ts"
        );
        assert_eq!(
            P0_T06_EXTENSION_STATUS_COMMAND,
            "/cognitiveos-p0-t06-status"
        );
    }
}
