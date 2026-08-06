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
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

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

/// Non-secret Personal Pi configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiConfig {
    executable_path: PathBuf,
    extension_entry_path: PathBuf,
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrivatePiCandidateResponse {
    pub tool_ref: String,
    pub action: String,
    pub target: String,
    pub parameters_digest: String,
    pub expected_state_version: i64,
    pub operation_descriptor_id: String,
}

/// Placeholder for the daemon-private one-shot Pi candidate transport.
///
/// Its configuration shape is retained so a future pinned, evidenced protocol
/// can be introduced without broadening scheduler authority. Until then, it
/// must fail before it starts a child process or passes any input to Pi.
pub(crate) struct PrivatePiCandidateProcess {
    _configured_executable_path: PathBuf,
    _configured_extension_entry_path: PathBuf,
}

impl PrivatePiCandidateProcess {
    // The scheduler caller is added separately; retain this narrow
    // construction boundary instead of exposing Pi paths outside this module.
    #[allow(dead_code)]
    pub(crate) fn from_config(config: &PiConfig) -> Self {
        Self {
            _configured_executable_path: config.executable_path.clone(),
            _configured_extension_entry_path: config.extension_entry_path.clone(),
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
        // Pi 0.81.1 has no evidenced extension API that owns stdin/stdout,
        // and its documented --print path receives a positional prompt rather
        // than this JSON request. Do not spawn a TUI or rely on an unproven
        // extension-defined CLI flag on an authority-adjacent code path.
        Err("private Pi candidate protocol is unsupported by the pinned runtime".to_owned())
    }
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

    let executable_path = required_path(&value, "executable_path")?;
    let extension_entry_path = required_path(&value, "extension_entry_path")?;
    Ok(PiConfig {
        executable_path,
        extension_entry_path,
    })
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
        };
        let error = PrivatePiCandidateProcess::from_config(&config)
            .propose(&request)
            .expect_err("oversized Context must be rejected before spawning Pi");
        assert_eq!(
            error,
            "private Pi candidate request exceeds transport limit"
        );
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
    fn malformed_configurations_are_corrupt_not_ready() {
        let executable = absolute("bin/pi");
        let extension = absolute("pi-cognitiveos/dist/index.js");
        let rejected = [
            "not json".to_owned(),
            "[]".to_owned(),
            r#"{"schema_version":2,"surface":"personal-pi-config","executable_path":"/a","extension_entry_path":"/b"}"#.to_owned(),
            r#"{"schema_version":1,"surface":"other","executable_path":"/a","extension_entry_path":"/b"}"#.to_owned(),
            r#"{"schema_version":1,"surface":"personal-pi-config","extension_entry_path":"/b"}"#.to_owned(),
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
