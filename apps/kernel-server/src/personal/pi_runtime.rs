//! Personal Pi runtime probe (P1-T07).
//!
//! The readiness `pi` component used to be hard-coded `not_configured`, which
//! made `first_conversation_ready` structurally false. This module replaces the
//! placeholder with a real, non-secret observation while keeping ADR-0023's
//! aggregation rules untouched: `pi` stays an optional component, and only its
//! status becomes fact-derived.
//!
//! An observation is only `Ready` when all three of these hold:
//!   1. a Personal Pi configuration exists and is readable;
//!   2. the configured Pi executable and the configured CognitiveOS Extension
//!      entry file are both present on disk;
//!   3. the executable reports exactly the pinned Pi version.
//!
//! Absence is reported as absence. A missing configuration is `NotConfigured`,
//! not an error, so a host that has never configured Pi reads exactly as it did
//! before this module existed.
//!
//! The probe hands the child process no credential: the environment is cleared
//! and rebuilt from an OS-essentials allowlist, so an ambient Provider key in
//! the daemon's environment cannot reach Pi through a readiness check.

#[cfg(test)]
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
#[cfg(unix)]
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use cognitive_secret::{
    ProviderConfigRepository, SelectedModelRepository, select_production_secret_store,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(unix)]
use super::provider_proxy::{ProviderProxyService, RustlsProviderTransport};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

/// Personal Pi configuration file name, alongside `provider.json`.
pub const PI_CONFIG_FILE_NAME: &str = "pi.json";

/// Product-local schema version for `pi.json` (not a registry schema).
pub const PI_CONFIG_SCHEMA_VERSION: u64 = 1;

/// Surface marker so an unrelated JSON file cannot be mistaken for this one.
pub const PI_CONFIG_SURFACE: &str = "personal-pi-config";

/// Pi version this product is pinned to.
///
/// Mirrors `PiCompatibilityPin::expected().package_version` in
/// `apps/pi-agent-adapter/src/lib.rs`, which stays the single source of truth.
/// `pinned_pi_version_matches_the_adapter_pin` fails if the two ever drift.
pub const PINNED_PI_VERSION: &str = "0.81.1";

/// Deadline for `pi --version`. A hung executable must not hang a readiness
/// request, which has no response-side timeout of its own.
pub const PI_VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum private candidate request/response size. This is a transport
/// bound, not a Context budget; the resolver remains responsible for Context
/// byte and token budgets.
pub const PRIVATE_PI_CANDIDATE_FRAME_LIMIT: usize = 256 * 1024;

#[cfg(unix)]
const PRIVATE_COMPLETION_TIMEOUT: Duration = Duration::from_secs(65);
#[cfg(unix)]
const PRIVATE_ADAPTER_TIMEOUT: Duration = Duration::from_secs(70);
#[cfg(unix)]
static PRIVATE_SOCKET_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Non-secret Personal Pi configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiConfig {
    executable_path: PathBuf,
    extension_entry_path: PathBuf,
    candidate_adapter_path: Option<PathBuf>,
    candidate_extension_entry_path: Option<PathBuf>,
}

/// One bounded request sent over the daemon-supervised private Pi transport.
/// The rendered Context is data-plane input only; it contains no bearer,
/// bootstrap secret, capability, WIA, Effect, progress, or Task authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PrivatePiCandidateRequest {
    pub protocol: &'static str,
    pub task_ref: String,
    pub contract_epoch: i64,
    pub rendered_context: String,
}

/// The only response shape accepted from the private Pi transport. The
/// scheduler converts this into its own untrusted candidate type and performs
/// all descriptor, contract, authorization, sealing, and admission checks.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrivatePiCandidateResponse {
    pub tool_ref: String,
    pub action: String,
    pub target: String,
    #[serde(default)]
    pub parameters: Option<Value>,
    pub parameters_digest: String,
    pub expected_state_version: i64,
    pub operation_descriptor_id: String,
}

/// Daemon-supervised one-shot Pi candidate transport.
pub(crate) struct PrivatePiCandidateProcess {
    #[cfg(unix)]
    configured_executable_path: PathBuf,
    #[cfg(unix)]
    configured_candidate_adapter_path: Option<PathBuf>,
    #[cfg(unix)]
    configured_candidate_extension_entry_path: Option<PathBuf>,
    #[cfg(unix)]
    provider_config_dir: PathBuf,
}

impl PrivatePiCandidateProcess {
    // The scheduler caller is added separately; retain this narrow
    // construction boundary instead of exposing Pi paths outside this module.
    pub(crate) fn from_config(config: &PiConfig, provider_config_dir: &Path) -> Self {
        #[cfg(not(unix))]
        let _ = (config, provider_config_dir);
        Self {
            #[cfg(unix)]
            configured_executable_path: config.executable_path.clone(),
            #[cfg(unix)]
            configured_candidate_adapter_path: config.candidate_adapter_path.clone(),
            #[cfg(unix)]
            configured_candidate_extension_entry_path: config
                .candidate_extension_entry_path
                .clone(),
            #[cfg(unix)]
            provider_config_dir: provider_config_dir.to_path_buf(),
        }
    }

    pub(crate) fn propose(
        &self,
        request: &PrivatePiCandidateRequest,
    ) -> Result<PrivatePiCandidateResponse, String> {
        let request_json = serde_json::to_vec(request).map_err(|error| error.to_string())?;
        if request_json.len() > PRIVATE_PI_CANDIDATE_FRAME_LIMIT {
            return Err("private Pi candidate request exceeds transport limit".to_owned());
        }
        #[cfg(unix)]
        {
            self.propose_over_private_completion_socket(&request_json)
        }
        #[cfg(not(unix))]
        {
            let _ = request_json;
            Err("private Pi candidate transport requires a Unix-domain socket host".to_owned())
        }
    }

    #[cfg(unix)]
    fn propose_over_private_completion_socket(
        &self,
        request_json: &[u8],
    ) -> Result<PrivatePiCandidateResponse, String> {
        let adapter_path = self
            .configured_candidate_adapter_path
            .as_deref()
            .filter(|path| path.is_file())
            .ok_or_else(|| "private Pi candidate adapter is not configured".to_owned())?;
        let extension_path = self
            .configured_candidate_extension_entry_path
            .as_deref()
            .filter(|path| path.is_file())
            .ok_or_else(|| "private Pi candidate extension is not configured".to_owned())?;
        let selected_model = SelectedModelRepository::under_config_dir(&self.provider_config_dir)
            .load()
            .map_err(|_| "private Pi selected model is unavailable".to_owned())?
            .ok_or_else(|| "private Pi selected model is unavailable".to_owned())?;
        let socket = PrivateCompletionSocket::create(&self.provider_config_dir)?;
        let socket_path = socket
            .path()
            .to_str()
            .ok_or_else(|| "private Pi completion socket path is not valid UTF-8".to_owned())?;
        let executable_path = self
            .configured_executable_path
            .to_str()
            .ok_or_else(|| "configured Pi executable path is not valid UTF-8".to_owned())?;
        let extension_path = extension_path
            .to_str()
            .ok_or_else(|| "private Pi candidate extension path is not valid UTF-8".to_owned())?;
        let mut command = Command::new(adapter_path);
        command.env_clear();
        for key in PROBE_ENVIRONMENT_ALLOWLIST {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        let mut child = command
            .arg("daemon-candidate")
            .arg("--pi")
            .arg(executable_path)
            .arg("--model")
            .arg(selected_model.model_id())
            .arg("--work-dir")
            .arg(socket.runtime_dir())
            .arg("--config-dir")
            .arg(socket.runtime_dir())
            .arg("--extension")
            .arg(extension_path)
            .env("COGNITIVEOS_PRIVATE_COMPLETION_SOCKET", socket_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "private Pi candidate adapter invocation failed".to_owned())?;
        child
            .stdin
            .as_mut()
            .ok_or_else(|| "private Pi adapter stdin was not captured".to_owned())?
            .write_all(request_json)
            .map_err(|_| "private Pi candidate request could not be written".to_owned())?;
        drop(child.stdin.take());
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "private Pi adapter stdout was not captured".to_owned())?
            .take((PRIVATE_PI_CANDIDATE_FRAME_LIMIT + 1) as u64);
        let stdout_reader = thread::spawn(move || {
            let mut output = Vec::new();
            let mut stdout = stdout;
            stdout.read_to_end(&mut output).map_err(|_| ())?;
            Ok::<_, ()>(output)
        });
        // The adapter owns a shorter Pi deadline, while this outer deadline
        // ensures a broken adapter cannot strand the scheduler indefinitely.
        let started = Instant::now();
        let mut termination_error = None;
        let exit_status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Err(_) => {
                    termination_error = Some("private Pi adapter wait failed".to_owned());
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                Ok(None) if started.elapsed() >= PRIVATE_ADAPTER_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    termination_error = Some("private Pi candidate adapter timed out".to_owned());
                    break None;
                }
                Ok(None) => thread::sleep(Duration::from_millis(25)),
            }
        };
        let output = stdout_reader
            .join()
            .map_err(|_| "private Pi adapter stdout reader panicked".to_owned())?
            .map_err(|_| "private Pi adapter stdout could not be read".to_owned())?;
        if let Some(error) = termination_error {
            let _ = socket.finish();
            return Err(error);
        }
        let exit_status = exit_status.ok_or_else(|| {
            "private Pi candidate adapter exited without a final status".to_owned()
        })?;
        if !exit_status.success() {
            let _ = socket.finish();
            return Err("private Pi candidate adapter rejected the request".to_owned());
        }
        if output.len() <= PRIVATE_PI_CANDIDATE_FRAME_LIMIT {
            if let Ok(parsed) = serde_json::from_slice::<PrivatePiCandidateResponse>(&output) {
                // A stub adapter may emit the untrusted candidate on stdout
                // without connecting to the Provider completion socket. The
                // daemon still validates descriptor, digest, and authorization.
                drop(socket);
                return Ok(parsed);
            }
        }
        socket.finish()?;
        if output.len() > PRIVATE_PI_CANDIDATE_FRAME_LIMIT {
            return Err("private Pi candidate response exceeds transport limit".to_owned());
        }
        serde_json::from_slice(&output)
            .map_err(|_| "private Pi candidate response is malformed".to_owned())
    }
}

#[cfg(unix)]
struct PrivateCompletionSocket {
    runtime_directory: PathBuf,
    socket_path: PathBuf,
    server: Option<thread::JoinHandle<Result<(), String>>>,
}

#[cfg(unix)]
impl PrivateCompletionSocket {
    fn create(config_dir: &Path) -> Result<Self, String> {
        let socket_directory = config_dir.join("private-completions");
        fs::create_dir_all(&socket_directory)
            .map_err(|_| "private completion socket directory is unavailable".to_owned())?;
        fs::set_permissions(&socket_directory, fs::Permissions::from_mode(0o700)).map_err(
            |_| "private completion socket directory permissions are unavailable".to_owned(),
        )?;
        let runtime_directory = socket_directory.join(format!(
            "candidate-{}-{}",
            std::process::id(),
            PRIVATE_SOCKET_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&runtime_directory)
            .map_err(|_| "private completion runtime directory could not be created".to_owned())?;
        fs::set_permissions(&runtime_directory, fs::Permissions::from_mode(0o700)).map_err(
            |_| "private completion runtime directory permissions are unavailable".to_owned(),
        )?;
        let socket_path = runtime_directory.join("completion.sock");
        let listener = UnixListener::bind(&socket_path)
            .map_err(|_| "private completion socket could not be created".to_owned())?;
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))
            .map_err(|_| "private completion socket permissions are unavailable".to_owned())?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "private completion socket could not be configured".to_owned())?;
        let provider_config_dir = config_dir.to_path_buf();
        let server =
            thread::spawn(move || serve_one_private_completion(listener, provider_config_dir));
        Ok(Self {
            runtime_directory,
            socket_path,
            server: Some(server),
        })
    }

    fn path(&self) -> &Path {
        &self.socket_path
    }

    fn runtime_dir(&self) -> &Path {
        &self.runtime_directory
    }

    fn finish(mut self) -> Result<(), String> {
        let result = self
            .server
            .take()
            .ok_or_else(|| "private completion server was unavailable".to_owned())?
            .join()
            .map_err(|_| "private completion server panicked".to_owned())?;
        let _ = fs::remove_file(&self.socket_path);
        let _ = fs::remove_dir_all(&self.runtime_directory);
        result
    }
}

#[cfg(unix)]
impl Drop for PrivateCompletionSocket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket_path);
        let _ = fs::remove_dir_all(&self.runtime_directory);
    }
}

#[cfg(unix)]
fn serve_one_private_completion(listener: UnixListener, config_dir: PathBuf) -> Result<(), String> {
    let started = Instant::now();
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && started.elapsed() < PRIVATE_COMPLETION_TIMEOUT =>
            {
                thread::sleep(Duration::from_millis(20))
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Err("private completion socket timed out".to_owned());
            }
            Err(_) => return Err("private completion socket refused a connection".to_owned()),
        }
    };
    forward_one_private_completion(stream, &config_dir)
}

#[cfg(unix)]
fn forward_one_private_completion(mut stream: UnixStream, config_dir: &Path) -> Result<(), String> {
    stream
        .set_read_timeout(Some(PRIVATE_COMPLETION_TIMEOUT))
        .map_err(|_| "private completion socket read timeout is unavailable".to_owned())?;
    let request = read_one_private_completion_request(&mut stream)?;
    let body = parse_private_completion_request(&request)?;
    let secret_backend = select_production_secret_store();
    let transport = RustlsProviderTransport::default();
    let service = ProviderProxyService::new(
        secret_backend.as_secret_store(),
        ProviderConfigRepository::under_config_dir(config_dir),
        &transport,
    );
    let response = service
        .forward_private_candidate_completion(body)
        .map_err(|_| "private completion provider request was refused".to_owned())?;
    if response.status != 200 || response.body.len() > PRIVATE_PI_CANDIDATE_FRAME_LIMIT {
        return Err("private completion provider response exceeds transport limit".to_owned());
    }
    let header = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        response.body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(&response.body))
        .map_err(|_| "private completion response could not be written".to_owned())
}

#[cfg(unix)]
fn read_one_private_completion_request(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut request = Vec::new();
    let mut expected_request_length = None;
    loop {
        if request.len() > PRIVATE_PI_CANDIDATE_FRAME_LIMIT {
            return Err("private completion request exceeds transport limit".to_owned());
        }
        if let Some(total_length) = expected_request_length {
            if request.len() == total_length {
                return Ok(request);
            }
            if request.len() > total_length {
                return Err("private completion request has trailing data".to_owned());
            }
        } else if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
        {
            let header_text = std::str::from_utf8(&request[..header_end + 4])
                .map_err(|_| "private completion request headers are malformed".to_owned())?;
            let content_length = header_text
                .split("\r\n")
                .filter_map(|line| line.split_once(':'))
                .find_map(|(name, value)| {
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .ok_or_else(|| "private completion body length is invalid".to_owned())?;
            let total_request_length = header_end + 4 + content_length;
            expected_request_length = Some(total_request_length);
            // A single Unix-domain socket read can contain both the complete
            // header and body. Do not require a second read merely to notice
            // that the exact framed request is already complete.
            if request.len() == total_request_length {
                return Ok(request);
            }
        }
        let mut buffer = [0_u8; 4096];
        let bytes_read = stream
            .read(&mut buffer)
            .map_err(|_| "private completion request could not be read".to_owned())?;
        if bytes_read == 0 {
            return Err("private completion request ended early".to_owned());
        }
        request.extend_from_slice(&buffer[..bytes_read]);
    }
}

#[cfg(unix)]
fn parse_private_completion_request(request: &[u8]) -> Result<&[u8], String> {
    if request.is_empty() || request.len() > PRIVATE_PI_CANDIDATE_FRAME_LIMIT {
        return Err("private completion request exceeds transport limit".to_owned());
    }
    let separator = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "private completion request is malformed".to_owned())?;
    let (headers, body) = request.split_at(separator + 4);
    let headers = std::str::from_utf8(headers)
        .map_err(|_| "private completion request headers are malformed".to_owned())?;
    let mut lines = headers.split("\r\n");
    if lines.next() != Some("POST /chat/completions HTTP/1.1") {
        return Err("private completion route is refused".to_owned());
    }
    let mut content_length = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("authorization") {
            return Err("private completion authorization is refused".to_owned());
        }
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err("private completion body length is ambiguous".to_owned());
            }
            content_length = value.trim().parse::<usize>().ok();
        }
    }
    if content_length != Some(body.len()) {
        return Err("private completion body length is invalid".to_owned());
    }
    let body_json: Value = serde_json::from_slice(body)
        .map_err(|_| "private completion body is invalid JSON".to_owned())?;
    let body_object = body_json
        .as_object()
        .ok_or_else(|| "private completion body is not an object".to_owned())?;
    if !body_object
        .keys()
        .all(|field| ["model", "stream", "messages"].contains(&field.as_str()))
        || body_object.get("stream") != Some(&Value::Bool(false))
        || !body_object.get("model").is_some_and(Value::is_string)
        || !body_object.get("messages").is_some_and(Value::is_array)
        || !body_object["messages"].as_array().is_some_and(|messages| {
            messages.iter().all(|message| {
                let Some(message) = message.as_object() else {
                    return false;
                };
                matches!(
                    message.get("role").and_then(Value::as_str),
                    Some("system" | "user" | "assistant")
                ) && message.get("content").is_some_and(Value::is_string)
                    && message.len() == 2
            })
        })
    {
        return Err("private completion body is outside candidate protocol".to_owned());
    }
    Ok(body)
}

/// Why a `pi.json` could not be turned into a `PiConfig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiConfigError {
    /// No configuration file. Pi is simply not configured on this host.
    NotFound,
    /// Present but unusable. `detail` is a fixed string, never caller data.
    Corrupt { detail: &'static str },
    /// Present but unreadable (permissions, IO).
    Unreadable,
}

/// What the daemon observed about the Pi runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiRuntimeObservation {
    /// No `pi.json`; Pi has not been configured for this installation.
    NotConfigured,
    /// `pi.json` exists but cannot be used.
    ConfigUnusable { detail: &'static str },
    /// Configured executable is not a file.
    ExecutableMissing,
    /// Configured CognitiveOS Extension entry file is not a file.
    ExtensionMissing,
    /// The executable could not be started or its output could not be read.
    ProbeFailed,
    /// The executable did not answer `--version` within the deadline.
    ProbeTimedOut,
    /// The executable answered, but not with the pinned version.
    VersionMismatch { observed: String },
    /// Configuration, files and pinned version all check out.
    Ready,
}

impl PiRuntimeObservation {
    /// Stable error class for the readiness projection, or `None` when ready.
    pub fn error_class(&self) -> Option<&'static str> {
        match self {
            Self::NotConfigured => Some("pi_not_configured"),
            Self::ConfigUnusable { .. } => Some("pi_config_unusable"),
            Self::ExecutableMissing => Some("pi_executable_missing"),
            Self::ExtensionMissing => Some("pi_extension_missing"),
            Self::ProbeFailed => Some("pi_probe_failed"),
            Self::ProbeTimedOut => Some("pi_probe_timeout"),
            Self::VersionMismatch { .. } => Some("pi_version_mismatch"),
            Self::Ready => None,
        }
    }

    /// Short non-secret summary used as the `package_status` fact.
    pub fn package_status(&self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::ConfigUnusable { .. } => "config_unusable",
            Self::ExecutableMissing => "executable_missing",
            Self::ExtensionMissing => "extension_missing",
            Self::ProbeFailed => "probe_failed",
            Self::ProbeTimedOut => "probe_timeout",
            Self::VersionMismatch { .. } => "version_mismatch",
            Self::Ready => "ready",
        }
    }

    /// The version the executable reported, when one was observed.
    pub fn observed_version(&self) -> Option<&str> {
        match self {
            Self::VersionMismatch { observed } => Some(observed.as_str()),
            Self::Ready => Some(PINNED_PI_VERSION),
            _ => None,
        }
    }
}

/// Read and validate `<config_dir>/pi.json`.
pub fn load_pi_config(config_dir: &Path) -> Result<PiConfig, PiConfigError> {
    let path = config_dir.join(PI_CONFIG_FILE_NAME);
    if !path.exists() {
        return Err(PiConfigError::NotFound);
    }
    let document = match fs::read_to_string(&path) {
        Ok(document) => document,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(PiConfigError::NotFound);
        }
        Err(_) => return Err(PiConfigError::Unreadable),
    };
    parse_pi_config(&document)
}

/// Parse a `pi.json` document. Split out so the validation is unit testable
/// without touching the filesystem.
pub fn parse_pi_config(document: &str) -> Result<PiConfig, PiConfigError> {
    let value: Value = match serde_json::from_str(document) {
        Ok(value) => value,
        Err(_) => {
            return Err(PiConfigError::Corrupt {
                detail: "pi config is not valid JSON",
            });
        }
    };
    if !value.is_object() {
        return Err(PiConfigError::Corrupt {
            detail: "pi config is not a JSON object",
        });
    }
    if value["schema_version"].as_u64() != Some(PI_CONFIG_SCHEMA_VERSION) {
        return Err(PiConfigError::Corrupt {
            detail: "pi config declares an unsupported schema_version",
        });
    }
    if value["surface"].as_str() != Some(PI_CONFIG_SURFACE) {
        return Err(PiConfigError::Corrupt {
            detail: "pi config is not a personal-pi-config document",
        });
    }
    let allowed_fields = [
        "schema_version",
        "surface",
        "executable_path",
        "extension_entry_path",
        "candidate_adapter_path",
        "candidate_extension_entry_path",
    ];
    if !value.as_object().is_some_and(|object| {
        object
            .keys()
            .all(|field| allowed_fields.contains(&field.as_str()))
    }) {
        return Err(PiConfigError::Corrupt {
            detail: "pi config has unsupported fields",
        });
    }

    let executable_path = required_path(&value, "executable_path")?;
    let extension_entry_path = required_path(&value, "extension_entry_path")?;
    let candidate_adapter_path = optional_path(&value, "candidate_adapter_path")?;
    let candidate_extension_entry_path = optional_path(&value, "candidate_extension_entry_path")?;
    if candidate_adapter_path.is_some() != candidate_extension_entry_path.is_some() {
        return Err(PiConfigError::Corrupt {
            detail: "pi config requires both private candidate paths",
        });
    }
    Ok(PiConfig {
        executable_path,
        extension_entry_path,
        candidate_adapter_path,
        candidate_extension_entry_path,
    })
}

fn optional_path(value: &Value, field: &'static str) -> Result<Option<PathBuf>, PiConfigError> {
    if value.get(field).is_none() {
        return Ok(None);
    }
    required_path(value, field).map(Some)
}

fn required_path(value: &Value, field: &'static str) -> Result<PathBuf, PiConfigError> {
    let raw = match value[field].as_str() {
        Some(raw) if !raw.trim().is_empty() => raw.trim(),
        Some(_) => {
            return Err(PiConfigError::Corrupt {
                detail: "pi config path field is empty",
            });
        }
        None => {
            return Err(PiConfigError::Corrupt {
                detail: "pi config is missing a required path field",
            });
        }
    };
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        // A relative path would resolve against whatever directory the daemon
        // happens to be started from, which is not a reproducible fact.
        return Err(PiConfigError::Corrupt {
            detail: "pi config paths must be absolute",
        });
    }
    Ok(path)
}

/// Observe the Pi runtime for a Personal configuration directory.
pub fn observe_pi_runtime(config_dir: &Path) -> PiRuntimeObservation {
    let config = match load_pi_config(config_dir) {
        Ok(config) => config,
        Err(PiConfigError::NotFound) => return PiRuntimeObservation::NotConfigured,
        Err(PiConfigError::Corrupt { detail }) => {
            return PiRuntimeObservation::ConfigUnusable { detail };
        }
        Err(PiConfigError::Unreadable) => {
            return PiRuntimeObservation::ConfigUnusable {
                detail: "pi config could not be read",
            };
        }
    };
    observe_configured_pi_runtime(&config)
}

/// Observe a already-loaded configuration.
pub fn observe_configured_pi_runtime(config: &PiConfig) -> PiRuntimeObservation {
    if !config.executable_path.is_file() {
        return PiRuntimeObservation::ExecutableMissing;
    }
    if !config.extension_entry_path.is_file() {
        return PiRuntimeObservation::ExtensionMissing;
    }
    match probe_reported_version(&config.executable_path) {
        Ok(Some(reported)) => classify_reported_version(&reported),
        Ok(None) => PiRuntimeObservation::ProbeTimedOut,
        Err(()) => PiRuntimeObservation::ProbeFailed,
    }
}

/// Classify `pi --version` output against the pinned version.
///
/// A whitespace-delimited token comparison is used rather than a substring
/// match so that `0.81.10` cannot satisfy a `0.81.1` pin.
pub fn classify_reported_version(reported: &str) -> PiRuntimeObservation {
    if reported
        .split_whitespace()
        .any(|token| token.trim_matches(|character| character == 'v') == PINNED_PI_VERSION)
    {
        return PiRuntimeObservation::Ready;
    }
    let observed = reported
        .split_whitespace()
        .map(|token| token.trim_matches(|character| character == 'v'))
        .find(|token| is_semver_like(token))
        .unwrap_or("unparsed")
        .to_owned();
    PiRuntimeObservation::VersionMismatch { observed }
}

fn is_semver_like(token: &str) -> bool {
    let mut segments = 0_usize;
    for segment in token.split('.') {
        if segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
        segments += 1;
    }
    segments == 3
}

/// Environment variables handed to the probe child.
///
/// The daemon's own environment is never inherited wholesale: an ambient
/// Provider key must not reach a Pi process through a readiness check. Only OS
/// essentials needed to execute a binary are forwarded.
pub const PROBE_ENVIRONMENT_ALLOWLIST: [&str; 8] = [
    "ComSpec",
    "PATH",
    "PATHEXT",
    "SystemRoot",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "WINDIR",
];

/// Build the probe child's environment from a parent environment map.
#[cfg(test)]
fn probe_child_environment(parent: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut child = BTreeMap::new();
    for key in PROBE_ENVIRONMENT_ALLOWLIST {
        if let Some(value) = parent.get(key) {
            child.insert(key.to_owned(), value.clone());
        }
    }
    child
}

/// Run `<executable> --version` with a deadline.
///
/// `Ok(Some(text))` is combined stdout+stderr, `Ok(None)` is a timeout, and
/// `Err(())` is a spawn or read failure.
fn probe_reported_version(executable: &Path) -> Result<Option<String>, ()> {
    let mut command = Command::new(executable);
    command.env_clear();
    for key in PROBE_ENVIRONMENT_ALLOWLIST {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    let mut child = command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ())?;

    let stdout = child.stdout.take().ok_or(())?;
    let stderr = child.stderr.take().ok_or(())?;
    let stdout_reader = thread::spawn(move || read_to_string_lossy(stdout));
    let stderr_reader = thread::spawn(move || read_to_string_lossy(stderr));

    let started = Instant::now();
    let timed_out = loop {
        match child.try_wait() {
            Ok(Some(_)) => break false,
            Ok(None) if started.elapsed() >= PI_VERSION_PROBE_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                break true;
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => return Err(()),
        }
    };

    let stdout_text = stdout_reader.join().map_err(|_| ())?;
    let stderr_text = stderr_reader.join().map_err(|_| ())?;
    if timed_out {
        return Ok(None);
    }
    Ok(Some(format!("{stdout_text}\n{stderr_text}")))
}

fn read_to_string_lossy<R: Read>(mut reader: R) -> String {
    let mut buffer = Vec::new();
    let _ = reader.read_to_end(&mut buffer);
    String::from_utf8_lossy(&buffer).into_owned()
}

// Test setup needs direct assertion failures for filesystem fixture creation.
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn valid_document(executable: &str, extension: &str) -> String {
        format!(
            r#"{{"schema_version":1,"surface":"personal-pi-config","executable_path":"{executable}","extension_entry_path":"{extension}"}}"#
        )
    }

    fn absolute(sample: &str) -> String {
        if cfg!(windows) {
            format!("C:\\\\cognitiveos\\\\{sample}")
        } else {
            format!("/opt/cognitiveos/{sample}")
        }
    }

    #[test]
    fn pinned_pi_version_matches_the_adapter_pin() {
        // The adapter's PiCompatibilityPin is the single source of truth; this
        // mirror must never drift from it.
        let adapter_source = include_str!("../../../pi-agent-adapter/src/lib.rs");
        let needle = format!("package_version: \"{PINNED_PI_VERSION}\"");
        assert!(
            adapter_source.contains(&needle),
            "PINNED_PI_VERSION drifted from apps/pi-agent-adapter/src/lib.rs"
        );
    }

    #[test]
    fn private_candidate_response_rejects_authority_fields_and_unknown_data() {
        let response = r#"{
            "tool_ref": "operation://personal/filesystem/read",
            "action": "filesystem.read",
            "target": "file:///workspace/input.txt",
            "parameters_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "expected_state_version": 1,
            "operation_descriptor_id": "00000000-0000-7000-9000-000000000001",
            "progress": "agent_end"
        }"#;
        assert!(serde_json::from_str::<PrivatePiCandidateResponse>(response).is_err());
    }

    #[test]
    fn private_candidate_request_rejects_oversized_rendered_context_before_spawn() {
        let request = PrivatePiCandidateRequest {
            protocol: "cognitiveos.private-candidate/1",
            task_ref: "task://personal/test".to_owned(),
            contract_epoch: 1,
            rendered_context: "x".repeat(PRIVATE_PI_CANDIDATE_FRAME_LIMIT),
        };
        let config = PiConfig {
            executable_path: PathBuf::from("/does/not/exist"),
            extension_entry_path: PathBuf::from("/does/not/exist.js"),
            candidate_adapter_path: None,
            candidate_extension_entry_path: None,
        };
        let error = PrivatePiCandidateProcess::from_config(&config, Path::new("/tmp"))
            .propose(&request)
            .expect_err("oversized Context must be rejected before spawning Pi");
        assert_eq!(
            error,
            "private Pi candidate request exceeds transport limit"
        );
    }

    #[cfg(unix)]
    #[test]
    fn private_completion_socket_is_one_shot_and_cleans_its_runtime_directory() {
        let temporary_directory = std::env::temp_dir().join(format!(
            "cognitiveos-private-completion-test-{}-{}",
            std::process::id(),
            PRIVATE_SOCKET_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&temporary_directory).expect("create temporary config directory");

        let socket = PrivateCompletionSocket::create(&temporary_directory)
            .expect("create private completion socket");
        let socket_path = socket.path().to_path_buf();
        let runtime_directory = socket.runtime_dir().to_path_buf();
        let client_socket_path = socket_path.clone();
        let client = thread::spawn(move || {
            let mut stream = UnixStream::connect(client_socket_path)
                .expect("connect to private completion socket");
            stream
                .write_all(b"GET /chat/completions HTTP/1.1\r\nContent-Length: 0\r\n\r\n")
                .expect("write rejected private completion request");
        });

        let error = socket
            .finish()
            .expect_err("one malformed client request must be rejected");
        client.join().expect("private completion client panicked");
        assert_eq!(error, "private completion route is refused");
        assert!(!socket_path.exists());
        assert!(!runtime_directory.exists());
        assert!(UnixStream::connect(&socket_path).is_err());

        fs::remove_dir_all(&temporary_directory).expect("remove temporary config directory");
    }

    #[test]
    fn a_well_formed_configuration_parses() {
        let executable = absolute("bin/pi");
        let extension = absolute("pi-cognitiveos/dist/index.js");
        let parsed_config = parse_pi_config(&valid_document(&executable, &extension));
        assert!(parsed_config.is_ok(), "valid Pi configuration must parse");
        let Some(config) = parsed_config.ok() else {
            return;
        };
        assert!(config.executable_path.is_absolute());
        assert!(config.extension_entry_path.is_absolute());
    }

    #[test]
    fn optional_private_candidate_paths_must_be_absolute_when_present() {
        let executable = absolute("bin/pi");
        let extension = absolute("pi-cognitiveos/dist/index.js");
        let adapter = absolute("bin/pi-agent-adapter");
        let candidate_extension = absolute("pi-cognitiveos/private-candidate.mjs");
        let document = format!(
            r#"{{"schema_version":1,"surface":"personal-pi-config","executable_path":"{executable}","extension_entry_path":"{extension}","candidate_adapter_path":"{adapter}","candidate_extension_entry_path":"{candidate_extension}"}}"#
        );
        let config = parse_pi_config(&document).expect("private candidate paths must parse");
        assert_eq!(
            config.candidate_adapter_path,
            Some(PathBuf::from(adapter.clone()))
        );
        assert_eq!(
            config.candidate_extension_entry_path,
            Some(PathBuf::from(candidate_extension))
        );
        let relative_adapter = document.replace(&adapter, "relative/pi-agent-adapter");
        assert!(matches!(
            parse_pi_config(&relative_adapter),
            Err(PiConfigError::Corrupt { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn private_completion_parser_rejects_bearers_and_wrong_lengths() {
        let body = r#"{"model":"selected","stream":false,"messages":[]}"#;
        let valid = format!(
            "POST /chat/completions HTTP/1.1\r\nHost: private\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        assert_eq!(
            parse_private_completion_request(valid.as_bytes()),
            Ok(body.as_bytes())
        );
        let bearer = b"POST /chat/completions HTTP/1.1\r\nAuthorization: Bearer forbidden\r\nContent-Length: 2\r\n\r\n{}";
        assert!(parse_private_completion_request(bearer).is_err());
        let wrong_length = b"POST /chat/completions HTTP/1.1\r\nContent-Length: 3\r\n\r\n{}";
        assert!(parse_private_completion_request(wrong_length).is_err());
        let duplicate_length =
            b"POST /chat/completions HTTP/1.1\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{}";
        assert!(parse_private_completion_request(duplicate_length).is_err());
        let arbitrary_proxy_body =
            r#"{"model":"selected","stream":false,"messages":[],"tools":[]}"#;
        let arbitrary_proxy_request = format!(
            "POST /chat/completions HTTP/1.1\r\nContent-Length: {}\r\n\r\n{arbitrary_proxy_body}",
            arbitrary_proxy_body.len()
        );
        assert!(parse_private_completion_request(arbitrary_proxy_request.as_bytes()).is_err());
    }

    #[test]
    fn malformed_configurations_are_corrupt_not_ready() {
        let executable = absolute("bin/pi");
        let extension = absolute("pi-cognitiveos/dist/index.js");
        let rejected = [
            "not json".to_owned(),
            "[]".to_owned(),
            r#"{"schema_version":2,"surface":"personal-pi-config","executable_path":"/a","extension_entry_path":"/b"}"#.to_owned(),
            r#"{"schema_version":1,"surface":"other","executable_path":"/a","extension_entry_path":"/b"}"#.to_owned(),
            r#"{"schema_version":1,"surface":"personal-pi-config","extension_entry_path":"/b"}"#.to_owned(),
            r#"{"schema_version":1,"surface":"personal-pi-config","executable_path":"/a","extension_entry_path":"/b","api_key":"forbidden"}"#.to_owned(),
            r#"{"schema_version":1,"surface":"personal-pi-config","executable_path":"  ","extension_entry_path":"/b"}"#.to_owned(),
            format!(
                r#"{{"schema_version":1,"surface":"personal-pi-config","executable_path":"relative/pi","extension_entry_path":"{extension}"}}"#
            ),
            format!(
                r#"{{"schema_version":1,"surface":"personal-pi-config","executable_path":"{executable}","extension_entry_path":"relative/index.js"}}"#
            ),
        ];
        for document in rejected {
            assert!(matches!(
                parse_pi_config(&document),
                Err(PiConfigError::Corrupt { .. })
            ));
        }
    }

    #[test]
    fn a_missing_configuration_reads_as_not_configured() {
        let directory = std::env::temp_dir().join(format!(
            "cos-pi-runtime-absent-{}-{}",
            std::process::id(),
            PI_CONFIG_SCHEMA_VERSION
        ));
        let _ = fs::remove_dir_all(&directory);
        assert!(fs::create_dir_all(&directory).is_ok());
        assert_eq!(
            observe_pi_runtime(&directory),
            PiRuntimeObservation::NotConfigured
        );
        assert_eq!(
            PiRuntimeObservation::NotConfigured.error_class(),
            Some("pi_not_configured")
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_configured_but_absent_executable_is_blocked() {
        let directory =
            std::env::temp_dir().join(format!("cos-pi-runtime-missing-exe-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        assert!(fs::create_dir_all(&directory).is_ok());
        let executable = directory.join("pi-does-not-exist");
        let extension = directory.join("index.js");
        assert!(fs::write(&extension, "export default () => {};\n").is_ok());
        let document = valid_document(
            &executable.display().to_string().replace('\\', "\\\\"),
            &extension.display().to_string().replace('\\', "\\\\"),
        );
        assert!(fs::write(directory.join(PI_CONFIG_FILE_NAME), document).is_ok());

        assert_eq!(
            observe_pi_runtime(&directory),
            PiRuntimeObservation::ExecutableMissing
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_present_executable_with_absent_extension_is_blocked() {
        let directory =
            std::env::temp_dir().join(format!("cos-pi-runtime-missing-ext-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        assert!(fs::create_dir_all(&directory).is_ok());
        let executable = directory.join("pi-stub");
        assert!(fs::write(&executable, "stub\n").is_ok());
        let extension = directory.join("does-not-exist.js");
        let document = valid_document(
            &executable.display().to_string().replace('\\', "\\\\"),
            &extension.display().to_string().replace('\\', "\\\\"),
        );
        assert!(fs::write(directory.join(PI_CONFIG_FILE_NAME), document).is_ok());

        assert_eq!(
            observe_pi_runtime(&directory),
            PiRuntimeObservation::ExtensionMissing
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn only_the_exact_pinned_version_token_is_ready() {
        assert_eq!(
            classify_reported_version("0.81.1"),
            PiRuntimeObservation::Ready
        );
        assert_eq!(
            classify_reported_version("pi v0.81.1 (build abc)"),
            PiRuntimeObservation::Ready
        );
        assert_eq!(
            classify_reported_version("0.81.10"),
            PiRuntimeObservation::VersionMismatch {
                observed: "0.81.10".to_owned()
            }
        );
        assert_eq!(
            classify_reported_version("pi 0.82.0"),
            PiRuntimeObservation::VersionMismatch {
                observed: "0.82.0".to_owned()
            }
        );
        assert_eq!(
            classify_reported_version("no version here"),
            PiRuntimeObservation::VersionMismatch {
                observed: "unparsed".to_owned()
            }
        );
        assert_eq!(
            classify_reported_version("0.81.1.2"),
            PiRuntimeObservation::VersionMismatch {
                observed: "unparsed".to_owned()
            }
        );
    }

    #[test]
    fn the_probe_child_never_receives_a_provider_credential() {
        let mut parent = BTreeMap::new();
        parent.insert("PATH".to_owned(), "/usr/bin".to_owned());
        parent.insert(
            "DEEPSEEK_API_KEY".to_owned(),
            "must-not-propagate".to_owned(),
        );
        parent.insert("OPENAI_API_KEY".to_owned(), "must-not-propagate".to_owned());
        parent.insert(
            "ANTHROPIC_API_KEY".to_owned(),
            "must-not-propagate".to_owned(),
        );
        parent.insert(
            "COGNITIVEOS_BOOTSTRAP".to_owned(),
            "must-not-propagate".to_owned(),
        );

        let child = probe_child_environment(&parent);
        assert_eq!(child.get("PATH").map(String::as_str), Some("/usr/bin"));
        for forbidden in [
            "DEEPSEEK_API_KEY",
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "COGNITIVEOS_BOOTSTRAP",
        ] {
            assert!(
                !child.contains_key(forbidden),
                "{forbidden} must not reach the probe child"
            );
        }
        assert!(
            child
                .keys()
                .all(|key| PROBE_ENVIRONMENT_ALLOWLIST.contains(&key.as_str())),
            "the probe child environment must be allowlisted"
        );
    }

    #[test]
    fn observation_error_classes_and_statuses_are_stable() {
        let cases = [
            (
                PiRuntimeObservation::NotConfigured,
                "pi_not_configured",
                "not_configured",
            ),
            (
                PiRuntimeObservation::ConfigUnusable { detail: "x" },
                "pi_config_unusable",
                "config_unusable",
            ),
            (
                PiRuntimeObservation::ExecutableMissing,
                "pi_executable_missing",
                "executable_missing",
            ),
            (
                PiRuntimeObservation::ExtensionMissing,
                "pi_extension_missing",
                "extension_missing",
            ),
            (
                PiRuntimeObservation::ProbeFailed,
                "pi_probe_failed",
                "probe_failed",
            ),
            (
                PiRuntimeObservation::ProbeTimedOut,
                "pi_probe_timeout",
                "probe_timeout",
            ),
            (
                PiRuntimeObservation::VersionMismatch {
                    observed: "0.82.0".to_owned(),
                },
                "pi_version_mismatch",
                "version_mismatch",
            ),
        ];
        for (observation, error_class, package_status) in cases {
            assert_eq!(observation.error_class(), Some(error_class));
            assert_eq!(observation.package_status(), package_status);
        }
        assert_eq!(PiRuntimeObservation::Ready.error_class(), None);
        assert_eq!(PiRuntimeObservation::Ready.package_status(), "ready");
        assert_eq!(
            PiRuntimeObservation::Ready.observed_version(),
            Some(PINNED_PI_VERSION)
        );
    }
}
