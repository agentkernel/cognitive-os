//! Hidden hosted DSH stdio broker (P13-T02).
//!
//! The daemon spawns the exact-artifact DSH child itself, hands it one bounded
//! Context request frame on stdin, and reads newline-delimited JSON frames
//! (`observation` / `candidate` / `heartbeat` / `response`) from stdout under
//! a wall-clock timeout and byte caps. Everything that crosses back is an
//! observation: no frame advances Task, Effect or Verification state, a
//! `response` with `status: done` is not completion, and process exit of any
//! kind is `terminal_kind != success`. Secrets never enter child env or argv;
//! the child reaches the Provider only through the daemon proxy
//! (`POST /provider/v1/dsh/chat/completions`), so any direct Provider shape in
//! the launch plan or in a frame is refused and recorded.
//!
//! On Unix the child runs in its own process group and a timeout kills the
//! whole group so a dsh grandchild cannot be orphaned; Windows relies on
//! `Child::kill` only. Isolated spawn stays fenced on `DEV-WIN-GNU-01`;
//! Windows sandbox / ACL / supply-chain qualification is `P13-T13` and is
//! not claimed here.

use cognitive_store::{
    HOSTED_ATTEMPT_CONTEXT_MAX_BYTES, HOSTED_DSH_ARTIFACT_DIGEST, HOSTED_DSH_PROVIDER_PROXY,
    HostedArtifactObservation, HostedAttemptFrameSpec,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Frame protocol spoken over the child's stdin/stdout. Not a public contract.
pub const HOSTED_FRAME_PROTOCOL: &str = "cognitiveos.personal.hosted-dsh-stdio/0.1";
/// Bounded Context payload ceiling handed to one child (same number the
/// durable ledger CHECK pins).
pub const HOSTED_CONTEXT_MAX_BYTES: usize = HOSTED_ATTEMPT_CONTEXT_MAX_BYTES;
/// Bounded stdout accepted from one child before the rest is dropped.
pub const HOSTED_STDOUT_MAX_BYTES: usize = 256 * 1024;
/// Redacted stderr tail retained per child.
pub const HOSTED_STDERR_TAIL_BYTES: usize = 2048;
/// Maximum frames retained per child run.
pub const HOSTED_MAX_FRAMES: usize = 512;
/// Redacted text retained per frame.
pub const HOSTED_FRAME_TEXT_MAX_CHARS: usize = 1024;
/// Default wall-clock ceiling for one child Attempt.
pub const HOSTED_DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
/// Hard ceiling a caller may request.
pub const HOSTED_MAX_TIMEOUT: Duration = Duration::from_secs(30 * 60);
/// Direct Provider hosts / shapes a hosted child must never be pointed at.
pub const HOSTED_DIRECT_PROVIDER_MARKERS: &[&str] = &[
    "api.deepseek.com",
    "api.openai.com",
    "api.anthropic.com",
    "generativelanguage.googleapis.com",
    "openrouter.ai",
    "--api-key-file",
    "--direct-base-url",
    "deepseek_base_url",
    "openai_base_url",
    "anthropic_base_url",
];
/// Config file the daemon reads to locate the pinned artifact (same schema as
/// `cognitive dsh configure`).
pub const HOSTED_DSH_CONFIG_FILE_NAME: &str = "dsh.json";
/// Pin file inside the configured dsh root.
pub const HOSTED_DSH_REVISION_FILE_NAME: &str = ".cognitiveos-dsh-revision";
/// Relative path of the product hosted-attempt child inside the adapter root.
pub const HOSTED_ATTEMPT_CHILD_SCRIPT: &str = "scripts/hosted-attempt-child.mjs";

/// Fail-closed broker errors. None of them is a Task outcome.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HostedBrokerError {
    #[error(
        "isolated DSH spawn is fenced on DEV-WIN-GNU-01; DEV-WINDOWS-NATIVE-OPC-01 remains not-run"
    )]
    Fenced,
    #[error("secret-shaped material must not enter hosted DSH child {surface}")]
    SecretMaterial { surface: &'static str },
    #[error("native MCP/base tool/HMR/home patch is not hosted DSH")]
    NativeHarnessEscape,
    #[error("hosted DSH child must not reach a Provider directly: {detail}")]
    DirectProvider { detail: String },
    #[error("hosted DSH artifact digest mismatch")]
    ArtifactDigestMismatch,
    #[error("bounded context is empty")]
    ContextEmpty,
    #[error("bounded context exceeds {max} bytes ({bytes})")]
    ContextTooLarge { bytes: usize, max: usize },
    #[error("hosted DSH launch plan invalid: {detail}")]
    InvalidPlan { detail: String },
    #[error("hosted DSH artifact unavailable: {detail}")]
    ArtifactUnavailable { detail: String },
    #[error("hosted DSH broker io: {detail}")]
    Io { detail: String },
}

/// Daemon-owned launch plan. The caller never receives raw secrets to place here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedChildLaunchPlan {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cwd: Option<PathBuf>,
    pub artifact_digest: String,
    pub timeout: Duration,
    pub max_stdout_bytes: usize,
    pub max_frames: usize,
}

impl HostedChildLaunchPlan {
    /// Plan with the product ceilings and the exact artifact pin.
    pub fn new(program: impl Into<PathBuf>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            env: BTreeMap::new(),
            cwd: None,
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST.to_owned(),
            timeout: HOSTED_DEFAULT_TIMEOUT,
            max_stdout_bytes: HOSTED_STDOUT_MAX_BYTES,
            max_frames: HOSTED_MAX_FRAMES,
        }
    }

    /// Redacted argv for durable records (values after flags are elided).
    pub fn argv_redacted(&self) -> Vec<String> {
        self.args
            .iter()
            .map(|arg| {
                if arg.starts_with('-') {
                    arg.chars().take(32).collect()
                } else {
                    "<arg>".to_owned()
                }
            })
            .collect()
    }

    /// Environment keys only (never values).
    pub fn env_keys(&self) -> Vec<String> {
        self.env.keys().cloned().collect()
    }
}

/// Bounded Context handed to one child Attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedContextPayload {
    pub attempt_id: String,
    pub task_ref: String,
    pub employee_id: String,
    pub project_id: String,
    pub bounded_context: String,
    /// Loopback daemon origin the child may use for the Provider proxy / AKP.
    pub daemon_origin: Option<String>,
    /// Path (not content) of the daemon bootstrap file the child reads itself.
    pub bootstrap_file: Option<PathBuf>,
}

impl HostedContextPayload {
    /// SHA-256 hex of the bounded Context bytes.
    pub fn context_digest(&self) -> String {
        format!("{:x}", Sha256::digest(self.bounded_context.as_bytes()))
    }

    /// Fail-closed size / emptiness check.
    pub fn validate(&self) -> Result<(), HostedBrokerError> {
        if self.bounded_context.trim().is_empty() {
            return Err(HostedBrokerError::ContextEmpty);
        }
        let bytes = self.bounded_context.len();
        if bytes > HOSTED_CONTEXT_MAX_BYTES {
            return Err(HostedBrokerError::ContextTooLarge {
                bytes,
                max: HOSTED_CONTEXT_MAX_BYTES,
            });
        }
        if self.attempt_id.trim().is_empty() || self.task_ref.trim().is_empty() {
            return Err(HostedBrokerError::InvalidPlan {
                detail: "attempt_id and task_ref required".to_owned(),
            });
        }
        if let Some(origin) = &self.daemon_origin
            && !origin_is_loopback(origin)
        {
            return Err(HostedBrokerError::DirectProvider {
                detail: "daemon_origin must be loopback".to_owned(),
            });
        }
        Ok(())
    }

    /// The single request frame written to the child's stdin.
    pub fn request_frame(&self) -> Value {
        json!({
            "frame": "request",
            "protocol": HOSTED_FRAME_PROTOCOL,
            "attempt_id": self.attempt_id,
            "task_ref": self.task_ref,
            "employee_id": self.employee_id,
            "project_id": self.project_id,
            "context": self.bounded_context,
            "context_digest": self.context_digest(),
            "context_bytes": self.bounded_context.len(),
            "provider_proxy": HOSTED_DSH_PROVIDER_PROXY,
            "daemon_origin": self.daemon_origin,
            "bootstrap_file": self.bootstrap_file.as_ref().map(|path| path.display().to_string()),
            "completion_authority": "daemon",
        })
    }
}

/// Frame kinds a child may emit. All are observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedFrameKind {
    Observation,
    Candidate,
    Heartbeat,
    Response,
}

impl HostedFrameKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Candidate => "candidate",
            Self::Heartbeat => "heartbeat",
            Self::Response => "response",
        }
    }
}

/// Accepted, redacted frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedFrame {
    pub seq: u64,
    pub kind: HostedFrameKind,
    pub operation: Option<String>,
    pub payload_digest: Option<String>,
    pub text_redacted: String,
    /// Canonical JSON of a `candidate` payload, retained in memory only so the
    /// daemon can put the observed bytes into its CAS after the terminal
    /// observation (P13-T04). `None` for every other frame kind and for
    /// payloads above `HOSTED_CANDIDATE_PAYLOAD_MAX_BYTES`. Never written to
    /// the observation ledger and never redacted — the ingest path refuses
    /// secret-shaped deliverables instead of storing a redacted copy.
    pub payload_canonical: Option<String>,
}

/// Canonical candidate payload ceiling retained for CAS ingest (bytes).
pub const HOSTED_CANDIDATE_PAYLOAD_MAX_BYTES: usize = 256 * 1024;

/// Frame refused by the broker (recorded, never acted upon).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedRejectedFrame {
    pub seq: u64,
    pub reason: &'static str,
    pub text_redacted: String,
}

/// How the child ended. There is deliberately no `Success` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedTerminalKind {
    Exited { code: i32 },
    Signaled,
    TimedOut,
    SpawnFailed,
}

impl HostedTerminalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exited { .. } => "exited",
            Self::Signaled => "signaled",
            Self::TimedOut => "timed-out",
            Self::SpawnFailed => "spawn-failed",
        }
    }

    /// Process death is never completion (A4).
    pub fn implies_completion(self) -> bool {
        false
    }

    pub fn exit_code(self) -> Option<i32> {
        match self {
            Self::Exited { code } => Some(code),
            _ => None,
        }
    }
}

/// Redacted, bounded outcome of one real child run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedChildRun {
    pub pid: Option<u32>,
    pub artifact_digest: String,
    pub context_digest: String,
    pub terminal: HostedTerminalKind,
    pub frames: Vec<HostedFrame>,
    pub rejected_frames: Vec<HostedRejectedFrame>,
    pub unknown_lines: usize,
    pub stdout_bytes: usize,
    pub stdout_truncated: bool,
    pub stderr_tail_redacted: String,
    pub response_status: Option<String>,
    pub elapsed_ms: u64,
}

impl HostedChildRun {
    /// Never true: completion belongs to the daemon verifier, not to the child.
    pub fn completion_claimed(&self) -> bool {
        false
    }

    pub fn candidate_count(&self) -> usize {
        self.frames
            .iter()
            .filter(|frame| frame.kind == HostedFrameKind::Candidate)
            .count()
    }

    pub fn observation_count(&self) -> usize {
        self.frames
            .iter()
            .filter(|frame| {
                matches!(
                    frame.kind,
                    HostedFrameKind::Observation | HostedFrameKind::Heartbeat
                )
            })
            .count()
    }

    /// Accepted and refused frames merged in child sequence order, shaped for
    /// the durable observation ledger (every row is `authority_written = 0`).
    pub fn ledger_frames(&self) -> Vec<HostedAttemptFrameSpec> {
        let mut specs: Vec<HostedAttemptFrameSpec> = self
            .frames
            .iter()
            .map(|frame| HostedAttemptFrameSpec {
                seq: frame.seq,
                kind: frame.kind.as_str().to_owned(),
                operation: frame.operation.clone(),
                payload_digest: frame.payload_digest.clone(),
                reject_reason: None,
                text_redacted: frame.text_redacted.clone(),
            })
            .chain(
                self.rejected_frames
                    .iter()
                    .map(|frame| HostedAttemptFrameSpec {
                        seq: frame.seq,
                        kind: "rejected".to_owned(),
                        operation: None,
                        payload_digest: None,
                        reject_reason: Some(frame.reason.to_owned()),
                        text_redacted: frame.text_redacted.clone(),
                    }),
            )
            .collect();
        specs.sort_by_key(|spec| spec.seq);
        specs
    }
}

/// Windows GNU cannot host an isolated DSH child on this toolchain.
pub fn isolated_spawn_is_fenced() -> bool {
    cfg!(all(windows, target_env = "gnu"))
}

/// Fail-closed pre-spawn validation. Nothing is spawned when this errs.
pub fn validate_launch_plan(plan: &HostedChildLaunchPlan) -> Result<(), HostedBrokerError> {
    if isolated_spawn_is_fenced() {
        return Err(HostedBrokerError::Fenced);
    }
    if plan.artifact_digest != HOSTED_DSH_ARTIFACT_DIGEST {
        return Err(HostedBrokerError::ArtifactDigestMismatch);
    }
    if plan.program.as_os_str().is_empty() {
        return Err(HostedBrokerError::InvalidPlan {
            detail: "program required".to_owned(),
        });
    }
    if plan.timeout.is_zero() || plan.timeout > HOSTED_MAX_TIMEOUT {
        return Err(HostedBrokerError::InvalidPlan {
            detail: "timeout must be within (0, 30m]".to_owned(),
        });
    }
    for (key, value) in &plan.env {
        if secret_shaped_key(key) || secret_shaped_value(value) {
            return Err(HostedBrokerError::SecretMaterial { surface: "env" });
        }
    }
    for arg in &plan.args {
        if secret_shaped_key(arg) || secret_shaped_value(arg) {
            return Err(HostedBrokerError::SecretMaterial { surface: "argv" });
        }
    }
    let argv_and_keys: Vec<String> = plan
        .args
        .iter()
        .cloned()
        .chain(plan.env.keys().cloned())
        .map(|item| item.to_ascii_lowercase())
        .collect();
    for item in &argv_and_keys {
        if item.contains("--mcp")
            || item.contains("native-mcp")
            || item.contains("base-tool")
            || item.contains("hmr")
            || item.contains("home-patch")
        {
            return Err(HostedBrokerError::NativeHarnessEscape);
        }
    }
    for item in argv_and_keys
        .iter()
        .chain(plan.env.values())
        .map(|item| item.to_ascii_lowercase())
    {
        for marker in HOSTED_DIRECT_PROVIDER_MARKERS {
            if item.contains(marker) {
                return Err(HostedBrokerError::DirectProvider {
                    detail: format!("launch plan carries `{marker}`"),
                });
            }
        }
    }
    if let Some(index) = plan.args.iter().position(|arg| arg == "--provider-path")
        && plan.args.get(index + 1).map(String::as_str) != Some("b")
    {
        return Err(HostedBrokerError::DirectProvider {
            detail: "only Path B (daemon proxy) is allowed".to_owned(),
        });
    }
    Ok(())
}

/// Spawn the exact child, broker its stdio, and observe its terminal state.
///
/// `on_spawn` runs right after the OS process exists (with its pid) so the
/// caller can persist the dispatch marker of an already-persisted Intent.
pub fn run_hosted_child(
    plan: &HostedChildLaunchPlan,
    payload: &HostedContextPayload,
    on_spawn: impl FnOnce(u32),
) -> Result<HostedChildRun, HostedBrokerError> {
    validate_launch_plan(plan)?;
    payload.validate()?;
    let started = Instant::now();
    let context_digest = payload.context_digest();

    let mut command = Command::new(&plan.program);
    command
        .args(&plan.args)
        .env_clear()
        .envs(&plan.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = &plan.cwd {
        command.current_dir(cwd);
    }
    // Own process group: a timeout kills the child and any dsh grandchild
    // together instead of orphaning the grandchild (Unix only).
    #[cfg(unix)]
    command.process_group(0);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return Ok(HostedChildRun {
                pid: None,
                artifact_digest: plan.artifact_digest.clone(),
                context_digest,
                terminal: HostedTerminalKind::SpawnFailed,
                frames: Vec::new(),
                rejected_frames: Vec::new(),
                unknown_lines: 0,
                stdout_bytes: 0,
                stdout_truncated: false,
                stderr_tail_redacted: redact(&format!("spawn failed: {}", error.kind())),
                response_status: None,
                elapsed_ms: elapsed_ms(started),
            });
        }
    };
    let pid = child.id();
    on_spawn(pid);

    let mut request_frame = payload.request_frame();
    request_frame["timeout_ms"] =
        json!(u64::try_from(plan.timeout.as_millis()).unwrap_or(u64::MAX));
    let request_line = format!("{request_frame}\n");
    if let Some(mut stdin) = child.stdin.take() {
        // A child that closed stdin early is observed through its exit, not here.
        let _ = stdin.write_all(request_line.as_bytes());
        let _ = stdin.flush();
    }

    let stdout_reader = child.stdout.take().map(|stdout| {
        let max_bytes = plan.max_stdout_bytes;
        let max_frames = plan.max_frames;
        thread::spawn(move || read_frames(stdout, max_bytes, max_frames))
    });
    let stderr_reader = child
        .stderr
        .take()
        .map(|stderr| thread::spawn(move || read_stderr_tail(stderr)));

    let terminal = wait_with_timeout(&mut child, plan.timeout);

    let frames = stdout_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let stderr_tail_redacted = stderr_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    Ok(HostedChildRun {
        pid: Some(pid),
        artifact_digest: plan.artifact_digest.clone(),
        context_digest,
        terminal,
        response_status: frames.response_status.clone(),
        frames: frames.accepted,
        rejected_frames: frames.rejected,
        unknown_lines: frames.unknown_lines,
        stdout_bytes: frames.bytes,
        stdout_truncated: frames.truncated,
        stderr_tail_redacted,
        elapsed_ms: elapsed_ms(started),
    })
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> HostedTerminalKind {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return match status.code() {
                    Some(code) => HostedTerminalKind::Exited { code },
                    None => HostedTerminalKind::Signaled,
                };
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_child_group(child);
                    return HostedTerminalKind::TimedOut;
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => {
                kill_child_group(child);
                return HostedTerminalKind::Signaled;
            }
        }
    }
}

/// Kill the child and, on Unix, its whole process group (dsh grandchildren).
fn kill_child_group(child: &mut Child) {
    #[cfg(unix)]
    {
        if let Ok(group) = i32::try_from(child.id())
            && group > 0
        {
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(-group),
                nix::sys::signal::Signal::SIGKILL,
            );
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[derive(Default)]
struct FrameStream {
    accepted: Vec<HostedFrame>,
    rejected: Vec<HostedRejectedFrame>,
    unknown_lines: usize,
    bytes: usize,
    truncated: bool,
    response_status: Option<String>,
}

fn read_frames(stdout: impl Read, max_bytes: usize, max_frames: usize) -> FrameStream {
    let mut stream = FrameStream::default();
    let mut seq: u64 = 0;
    let reader = BufReader::new(stdout);
    for line in reader.split(b'\n') {
        let Ok(line) = line else {
            break;
        };
        stream.bytes = stream.bytes.saturating_add(line.len() + 1);
        if stream.bytes > max_bytes {
            // Keep draining so the child never blocks on a full pipe, but
            // retain nothing beyond the cap.
            stream.truncated = true;
            continue;
        }
        let text = String::from_utf8_lossy(&line);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        seq += 1;
        let parsed: Option<Value> = serde_json::from_str(trimmed).ok();
        let Some(parsed) = parsed else {
            stream.unknown_lines += 1;
            continue;
        };
        if stream.accepted.len() + stream.rejected.len() >= max_frames {
            stream.truncated = true;
            continue;
        }
        classify_frame(&mut stream, seq, &parsed, trimmed);
    }
    stream
}

fn classify_frame(stream: &mut FrameStream, seq: u64, parsed: &Value, raw: &str) {
    let Some(kind) = parsed.get("frame").and_then(Value::as_str) else {
        // JSON without a frame discriminator is unknown output, never success.
        stream.unknown_lines += 1;
        return;
    };
    let redacted_raw = bounded_redacted(raw);
    if let Some(reason) = direct_provider_reason(parsed) {
        stream.rejected.push(HostedRejectedFrame {
            seq,
            reason,
            text_redacted: redacted_raw,
        });
        return;
    }
    let frame_kind = match kind {
        "observation" => HostedFrameKind::Observation,
        "candidate" => HostedFrameKind::Candidate,
        "heartbeat" => HostedFrameKind::Heartbeat,
        "response" => HostedFrameKind::Response,
        "request" | "provider_request" | "authority" | "task_complete" | "effect" => {
            stream.rejected.push(HostedRejectedFrame {
                seq,
                reason: "child-cannot-emit-authority-frame",
                text_redacted: redacted_raw,
            });
            return;
        }
        _ => {
            stream.unknown_lines += 1;
            return;
        }
    };
    let operation = parsed
        .get("operation")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if frame_kind == HostedFrameKind::Candidate && operation.is_none() {
        stream.rejected.push(HostedRejectedFrame {
            seq,
            reason: "candidate-without-operation",
            text_redacted: redacted_raw,
        });
        return;
    }
    let payload_canonical_json = parsed.get("payload").map(|payload| {
        serde_json_canonicalizer::to_string(payload).unwrap_or_else(|_| payload.to_string())
    });
    let payload_digest = payload_canonical_json
        .as_deref()
        .map(|canonical| format!("{:x}", Sha256::digest(canonical.as_bytes())));
    let payload_canonical = payload_canonical_json.filter(|canonical| {
        frame_kind == HostedFrameKind::Candidate
            && canonical.len() <= HOSTED_CANDIDATE_PAYLOAD_MAX_BYTES
    });
    if frame_kind == HostedFrameKind::Response {
        let status = parsed
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let normalized = match status {
            "done" | "failed" | "blocked" => status,
            _ => "unknown",
        };
        stream.response_status = Some(normalized.to_owned());
    }
    let text = parsed
        .get("text")
        .and_then(Value::as_str)
        .map(bounded_redacted)
        .unwrap_or(redacted_raw);
    stream.accepted.push(HostedFrame {
        seq,
        kind: frame_kind,
        operation,
        payload_digest,
        text_redacted: text,
        payload_canonical,
    });
}

fn direct_provider_reason(parsed: &Value) -> Option<&'static str> {
    if parsed.get("frame").and_then(Value::as_str) == Some("provider_request") {
        return Some("child-direct-provider");
    }
    if parsed.get("provider_direct").and_then(Value::as_bool) == Some(true) {
        return Some("child-direct-provider");
    }
    let mut candidates = Vec::new();
    for key in ["url", "base_url", "endpoint", "host"] {
        if let Some(value) = parsed.get(key).and_then(Value::as_str) {
            candidates.push(value);
        }
        if let Some(value) = parsed
            .get("payload")
            .and_then(|payload| payload.get(key))
            .and_then(Value::as_str)
        {
            candidates.push(value);
        }
    }
    for candidate in candidates {
        let lowered = candidate.to_ascii_lowercase();
        if HOSTED_DIRECT_PROVIDER_MARKERS
            .iter()
            .any(|marker| lowered.contains(marker))
        {
            return Some("child-direct-provider");
        }
    }
    None
}

fn read_stderr_tail(stderr: impl Read) -> String {
    let mut reader = BufReader::new(stderr);
    let mut buffer = Vec::new();
    let _ = reader.read_to_end(&mut buffer);
    let text = String::from_utf8_lossy(&buffer);
    let redacted = redact(&text);
    let tail_start = redacted
        .char_indices()
        .rev()
        .nth(HOSTED_STDERR_TAIL_BYTES.saturating_sub(1))
        .map(|(index, _)| index)
        .unwrap_or(0);
    redacted[tail_start..].to_owned()
}

fn bounded_redacted(text: &str) -> String {
    redact(text)
        .chars()
        .take(HOSTED_FRAME_TEXT_MAX_CHARS)
        .collect()
}

/// Redact secret-shaped tokens before anything durable or loggable.
pub fn redact(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        let lowered = rest.to_ascii_lowercase();
        let next = ["sk-", "bearer ", "ssv1:"]
            .iter()
            .filter_map(|marker| lowered.find(marker).map(|index| (index, marker.len())))
            .min_by_key(|(index, _)| *index);
        let Some((index, marker_len)) = next else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..index]);
        output.push_str(&rest[index..index + marker_len]);
        output.push_str("[redacted]");
        let after = &rest[index + marker_len..];
        let token_end = after
            .find(|character: char| {
                character.is_whitespace() || character == '"' || character == '\''
            })
            .unwrap_or(after.len());
        rest = &after[token_end..];
    }
    output
}

fn origin_is_loopback(origin: &str) -> bool {
    let lowered = origin.to_ascii_lowercase();
    let without_scheme = lowered
        .strip_prefix("http://")
        .or_else(|| lowered.strip_prefix("https://"))
        .unwrap_or(&lowered);
    let host_port = without_scheme.split('/').next().unwrap_or("");
    let host = host_port
        .rsplit_once(':')
        .map(|(host, _)| host)
        .unwrap_or(host_port);
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host == "127.0.0.1" || host == "localhost" || host == "::1"
}

fn secret_shaped_key(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("password")
        || lowered.contains("authorization")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("api-key")
}

fn secret_shaped_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
}

/// Pinned artifact facts resolved from the daemon-owned `dsh.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedDshArtifact {
    pub dsh_root: PathBuf,
    pub adapter_root: PathBuf,
    pub revision: String,
    pub child_script: PathBuf,
    /// SHA-256 hex of the child script bytes at observation time.
    pub child_script_digest: String,
    /// `pinned` when config, pin file and child script agree with the pin.
    pub health: String,
}

impl HostedDshArtifact {
    /// Observe the configured artifact for the durable fact ledger. Never
    /// errors and never spawns: every failure class becomes a health value.
    pub fn observe(config_dir: &Path) -> HostedArtifactObservation {
        let config_path = config_dir.join(HOSTED_DSH_CONFIG_FILE_NAME);
        let Ok(document) = std::fs::read_to_string(&config_path) else {
            return HostedArtifactObservation {
                configured_revision: None,
                pin_file_revision: None,
                health: "absent".to_owned(),
                child_script_digest: None,
                detail: "dsh configuration is absent; run `cognitive dsh configure`".to_owned(),
            };
        };
        let Some(object) = serde_json::from_str::<Value>(&document)
            .ok()
            .and_then(|value| value.as_object().cloned())
        else {
            return HostedArtifactObservation {
                configured_revision: None,
                pin_file_revision: None,
                health: "corrupt".to_owned(),
                child_script_digest: None,
                detail: "dsh configuration document is corrupt".to_owned(),
            };
        };
        let text = |name: &str| -> Option<String> {
            object
                .get(name)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        let configured_revision = text("revision");
        let dsh_root = text("dsh_root").map(PathBuf::from);
        let adapter_root = text("adapter_root").map(PathBuf::from);
        let candidate_only = object.get("candidate_only").and_then(Value::as_bool) == Some(true);
        let pin_file_revision = dsh_root.as_ref().and_then(|root| {
            std::fs::read_to_string(root.join(HOSTED_DSH_REVISION_FILE_NAME))
                .ok()
                .map(|pinned| pinned.trim().to_owned())
                .filter(|pinned| !pinned.is_empty())
        });
        let child_script_digest = adapter_root.as_ref().and_then(|root| {
            std::fs::read(root.join(HOSTED_ATTEMPT_CHILD_SCRIPT))
                .ok()
                .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
        });
        let (health, detail) = if !candidate_only || dsh_root.is_none() || adapter_root.is_none() {
            (
                "corrupt",
                "dsh configuration must name dsh_root, adapter_root and stay candidate-only",
            )
        } else if configured_revision.as_deref() != Some(HOSTED_DSH_ARTIFACT_DIGEST) {
            ("mismatch", "configured revision is not the product pin")
        } else if pin_file_revision.is_none() {
            ("absent", "dsh revision pin file is absent")
        } else if pin_file_revision.as_deref() != Some(HOSTED_DSH_ARTIFACT_DIGEST) {
            ("mismatch", "dsh revision pin file is not the product pin")
        } else if child_script_digest.is_none() {
            ("script-missing", "hosted attempt child script is missing")
        } else {
            (
                "pinned",
                "config, pin file and child script agree with the product pin",
            )
        };
        HostedArtifactObservation {
            configured_revision,
            pin_file_revision,
            health: health.to_owned(),
            child_script_digest,
            detail: detail.to_owned(),
        }
    }

    /// Resolve and health-check the configured artifact. Never spawns.
    pub fn resolve(config_dir: &Path) -> Result<Self, HostedBrokerError> {
        let config_path = config_dir.join(HOSTED_DSH_CONFIG_FILE_NAME);
        let document = std::fs::read_to_string(&config_path).map_err(|_| {
            HostedBrokerError::ArtifactUnavailable {
                detail: "dsh configuration is absent; run `cognitive dsh configure`".to_owned(),
            }
        })?;
        let value: Value = serde_json::from_str(&document).map_err(|_| {
            HostedBrokerError::ArtifactUnavailable {
                detail: "dsh configuration document is corrupt".to_owned(),
            }
        })?;
        let object = value
            .as_object()
            .ok_or_else(|| HostedBrokerError::ArtifactUnavailable {
                detail: "dsh configuration must be an object".to_owned(),
            })?;
        if object.get("candidate_only").and_then(Value::as_bool) != Some(true) {
            return Err(HostedBrokerError::ArtifactUnavailable {
                detail: "dsh configuration must remain candidate-only".to_owned(),
            });
        }
        let field = |name: &str| -> Result<String, HostedBrokerError> {
            object
                .get(name)
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| HostedBrokerError::ArtifactUnavailable {
                    detail: format!("dsh configuration has no {name}"),
                })
        };
        let dsh_root = PathBuf::from(field("dsh_root")?);
        let adapter_root = PathBuf::from(field("adapter_root")?);
        let revision = field("revision")?;
        if revision != HOSTED_DSH_ARTIFACT_DIGEST {
            return Err(HostedBrokerError::ArtifactDigestMismatch);
        }
        let pin_path = dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME);
        let pinned = std::fs::read_to_string(&pin_path).map_err(|_| {
            HostedBrokerError::ArtifactUnavailable {
                detail: "dsh revision pin file is absent".to_owned(),
            }
        })?;
        if pinned.trim() != revision {
            return Err(HostedBrokerError::ArtifactDigestMismatch);
        }
        let child_script = adapter_root.join(HOSTED_ATTEMPT_CHILD_SCRIPT);
        let script_bytes =
            std::fs::read(&child_script).map_err(|_| HostedBrokerError::ArtifactUnavailable {
                detail: "hosted attempt child script is missing".to_owned(),
            })?;
        Ok(Self {
            dsh_root,
            adapter_root,
            revision,
            child_script,
            child_script_digest: format!("{:x}", Sha256::digest(script_bytes)),
            health: "pinned".to_owned(),
        })
    }

    /// Product launch plan: `node <adapter_root>/scripts/hosted-attempt-child.mjs`.
    pub fn launch_plan(&self, timeout: Duration) -> HostedChildLaunchPlan {
        let mut plan = HostedChildLaunchPlan::new(
            "node",
            vec![
                self.child_script.display().to_string(),
                "--dsh-root".to_owned(),
                self.dsh_root.display().to_string(),
                "--adapter-root".to_owned(),
                self.adapter_root.display().to_string(),
                "--revision".to_owned(),
                self.revision.clone(),
                "--provider-path".to_owned(),
                "b".to_owned(),
            ],
        );
        plan.cwd = Some(self.dsh_root.clone());
        plan.timeout = timeout;
        plan.env = inherited_child_environment();
        plan
    }
}

/// Allowlisted, secret-free environment for the child (no CognitiveOS secrets).
pub fn inherited_child_environment() -> BTreeMap<String, String> {
    let mut environment = BTreeMap::new();
    for key in [
        "PATH",
        "HOME",
        "USER",
        "LOGNAME",
        "SHELL",
        "LANG",
        "LC_ALL",
        "TZ",
        "TMPDIR",
        "TMP",
        "TEMP",
        "PNPM_HOME",
        "COREPACK_HOME",
        "SystemRoot",
        "WINDIR",
        "ComSpec",
        "PATHEXT",
        "USERPROFILE",
        "APPDATA",
        "LOCALAPPDATA",
    ] {
        if let Ok(value) = std::env::var(key)
            && !secret_shaped_value(&value)
        {
            environment.insert(key.to_owned(), value);
        }
    }
    environment
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn plan() -> HostedChildLaunchPlan {
        HostedChildLaunchPlan::new("node", vec!["--isolated".to_owned()])
    }

    #[test]
    fn redact_hides_secret_shaped_tokens() {
        let text = "Authorization: Bearer abc.def key sk-live-123 ref ssv1:zzz tail";
        let redacted = redact(text);
        assert!(!redacted.contains("abc.def"));
        assert!(!redacted.contains("live-123"));
        assert!(!redacted.contains("zzz"));
        assert!(redacted.ends_with("tail"));
    }

    #[test]
    fn launch_plan_refuses_secret_env_and_argv() {
        if isolated_spawn_is_fenced() {
            assert_eq!(
                validate_launch_plan(&plan()),
                Err(HostedBrokerError::Fenced)
            );
            return;
        }
        let mut with_env = plan();
        with_env
            .env
            .insert("OPENAI_API_KEY".to_owned(), "sk-not-real".to_owned());
        assert_eq!(
            validate_launch_plan(&with_env),
            Err(HostedBrokerError::SecretMaterial { surface: "env" })
        );
        let mut with_argv = plan();
        with_argv.args.push("--token".to_owned());
        assert_eq!(
            validate_launch_plan(&with_argv),
            Err(HostedBrokerError::SecretMaterial { surface: "argv" })
        );
    }

    #[test]
    fn launch_plan_refuses_direct_provider_and_native_escape() {
        if isolated_spawn_is_fenced() {
            return;
        }
        let mut direct = plan();
        direct.args.push("--direct-base-url".to_owned());
        assert!(matches!(
            validate_launch_plan(&direct),
            Err(HostedBrokerError::DirectProvider { .. })
        ));
        let mut path_a = plan();
        path_a.args.push("--provider-path".to_owned());
        path_a.args.push("a".to_owned());
        assert!(matches!(
            validate_launch_plan(&path_a),
            Err(HostedBrokerError::DirectProvider { .. })
        ));
        let mut host = plan();
        host.env.insert(
            "DSH_PROVIDER".to_owned(),
            "https://api.deepseek.com".to_owned(),
        );
        assert!(matches!(
            validate_launch_plan(&host),
            Err(HostedBrokerError::DirectProvider { .. })
        ));
        let mut escape = plan();
        escape.args.push("--mcp".to_owned());
        assert_eq!(
            validate_launch_plan(&escape),
            Err(HostedBrokerError::NativeHarnessEscape)
        );
        let mut digest = plan();
        digest.artifact_digest = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_owned();
        assert_eq!(
            validate_launch_plan(&digest),
            Err(HostedBrokerError::ArtifactDigestMismatch)
        );
    }

    #[test]
    fn frame_stream_never_promotes_unknown_output() {
        let stdout = concat!(
            "ok\n",
            "success\n",
            "{\"status\":\"success\"}\n",
            "{\"frame\":\"observation\",\"text\":\"Bearer abc123 working\"}\n",
            "{\"frame\":\"candidate\",\"operation\":\"WorkspaceWrite\",\"payload\":{\"target\":\"a.txt\"}}\n",
            "{\"frame\":\"candidate\",\"payload\":{}}\n",
            "{\"frame\":\"provider_request\",\"url\":\"https://api.deepseek.com/v1\"}\n",
            "{\"frame\":\"candidate\",\"operation\":\"HttpFetch\",\"payload\":{\"url\":\"https://api.openai.com/v1/chat\"}}\n",
            "{\"frame\":\"task_complete\"}\n",
            "{\"frame\":\"response\",\"status\":\"done\"}\n",
        );
        let stream = read_frames(
            stdout.as_bytes(),
            HOSTED_STDOUT_MAX_BYTES,
            HOSTED_MAX_FRAMES,
        );
        assert_eq!(stream.unknown_lines, 3);
        assert_eq!(stream.accepted.len(), 3);
        assert_eq!(stream.rejected.len(), 4);
        assert!(
            stream
                .rejected
                .iter()
                .filter(|frame| frame.reason == "child-direct-provider")
                .count()
                == 2
        );
        assert!(
            stream.accepted[0]
                .text_redacted
                .contains("Bearer [redacted]")
        );
        assert!(!stream.accepted[0].text_redacted.contains("abc123"));
        assert!(stream.accepted[1].payload_digest.is_some());
        // P13-T04: the candidate's canonical payload is retained in memory for
        // CAS ingest and hashes to the recorded digest; other frames retain none.
        let canonical = stream.accepted[1]
            .payload_canonical
            .as_deref()
            .expect("candidate payload retained");
        assert_eq!(canonical, "{\"target\":\"a.txt\"}");
        assert_eq!(
            stream.accepted[1].payload_digest.as_deref(),
            Some(format!("{:x}", Sha256::digest(canonical.as_bytes())).as_str())
        );
        assert!(stream.accepted[0].payload_canonical.is_none());
        assert!(stream.accepted[2].payload_canonical.is_none());
        assert_eq!(stream.response_status.as_deref(), Some("done"));
    }

    #[test]
    fn terminal_kind_never_implies_completion() {
        for kind in [
            HostedTerminalKind::Exited { code: 0 },
            HostedTerminalKind::Exited { code: 1 },
            HostedTerminalKind::Signaled,
            HostedTerminalKind::TimedOut,
            HostedTerminalKind::SpawnFailed,
        ] {
            assert!(!kind.implies_completion());
            assert_ne!(kind.as_str(), "success");
        }
    }

    #[test]
    fn context_payload_is_bounded() {
        let mut payload = HostedContextPayload {
            attempt_id: "att".to_owned(),
            task_ref: "task://personal/x".to_owned(),
            employee_id: "emp".to_owned(),
            project_id: "proj".to_owned(),
            bounded_context: "x".repeat(HOSTED_CONTEXT_MAX_BYTES + 1),
            daemon_origin: None,
            bootstrap_file: None,
        };
        assert!(matches!(
            payload.validate(),
            Err(HostedBrokerError::ContextTooLarge { .. })
        ));
        payload.bounded_context = "   ".to_owned();
        assert_eq!(payload.validate(), Err(HostedBrokerError::ContextEmpty));
        payload.bounded_context = "do the task".to_owned();
        payload.daemon_origin = Some("https://api.deepseek.com".to_owned());
        assert!(matches!(
            payload.validate(),
            Err(HostedBrokerError::DirectProvider { .. })
        ));
        payload.daemon_origin = Some("http://127.0.0.1:48181".to_owned());
        assert!(payload.validate().is_ok());
        let frame = payload.request_frame();
        assert_eq!(frame["protocol"], HOSTED_FRAME_PROTOCOL);
        assert_eq!(frame["context_digest"], payload.context_digest());
        assert_eq!(frame["completion_authority"], "daemon");
    }
}
