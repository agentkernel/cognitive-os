//! User-service lifecycle transaction for a verified Personal Linux bundle.
//!
//! This module owns neither download nor trust-root selection. It reuses the
//! offline verifier and installer lease, then delegates only fixed service
//! lifecycle actions to a narrow controller. A controller never receives a
//! manifest, keyring, artifact bytes, user data, or arbitrary command text.

use crate::linux_bundle::{
    ExpectedPiCompatibility, LinuxBundleDeployment, LinuxBundleError, TrustedKeyring,
    verify_linux_bundle,
};
use crate::linux_bundle_installation::InstallerLifecycleLease;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

const ACTIVE_SYSTEMD_USER_UNIT: &str = "cognitiveos-personal.service";
const CANDIDATE_SYSTEMD_USER_UNIT: &str = "cognitiveos-personal-candidate.service";
const ACTIVE_HEALTH_PORT: u16 = 48181;
const CANDIDATE_HEALTH_PORT: u16 = 48182;
const MAX_SYSTEMCTL_OUTPUT_BYTES: usize = 8 * 1024;
const MAX_HEALTH_RESPONSE_BYTES: usize = 4 * 1024;
const MAX_HEALTH_ATTEMPTS: u8 = 3;

/// Stable, non-secret failure categories for the user-service boundary.
#[derive(Debug, Error)]
pub enum LinuxBundleServiceError {
    #[error("Linux bundle service controller rejected the candidate")]
    CandidateStartFailed,
    #[error("Linux bundle candidate health check failed")]
    CandidateHealthFailed,
    #[error("Linux bundle active service could not be confirmed")]
    FinalServiceConfirmationFailed,
    #[error("Linux bundle service rollback is incomplete")]
    RollbackIncomplete,
    #[error("Linux bundle user-service template is not safely rendered")]
    UnsafeUnitTemplate,
    #[error("Linux bundle service configuration is unsafe")]
    UnsafeServiceConfiguration,
    #[error("Linux bundle service command exceeded its deadline")]
    ServiceCommandTimedOut,
    #[error("Linux bundle health response was invalid")]
    InvalidHealthResponse,
    #[error("Linux bundle installation failed: {0}")]
    Installation(#[from] LinuxBundleError),
}

/// The only two product-owned user-unit roles. Bundle metadata never chooses
/// a unit identity, bind address, runtime root, executable name, or argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalUserServiceUnitKind {
    Candidate,
    Active,
}

impl PersonalUserServiceUnitKind {
    fn unit_name(self) -> &'static str {
        match self {
            Self::Candidate => CANDIDATE_SYSTEMD_USER_UNIT,
            Self::Active => ACTIVE_SYSTEMD_USER_UNIT,
        }
    }

    fn health_port(self) -> u16 {
        match self {
            Self::Candidate => CANDIDATE_HEALTH_PORT,
            Self::Active => ACTIVE_HEALTH_PORT,
        }
    }

    fn executable_directory(self) -> &'static str {
        match self {
            Self::Candidate => "staged",
            Self::Active => "versions",
        }
    }

    fn runtime_directory(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Active => "active",
        }
    }
}

/// Render one complete fixed user unit from product-owned values.
///
/// This renderer deliberately accepts only the checked deployment root and a
/// constrained version. It does not read a bundle, manifest, environment, or
/// source template, so none of those inputs can select a command or address.
pub fn render_personal_user_service_unit(
    unit_kind: PersonalUserServiceUnitKind,
    deployment_root: &Path,
    version: &str,
) -> Result<String, LinuxBundleServiceError> {
    if !safe_service_version(version) || !safe_unit_path(deployment_root) {
        return Err(LinuxBundleServiceError::UnsafeServiceConfiguration);
    }
    let executable = deployment_root
        .join(unit_kind.executable_directory())
        .join(version)
        .join("bin/kernel-server");
    let runtime_root = deployment_root
        .join("runtime")
        .join(unit_kind.runtime_directory());
    let executable = executable.to_string_lossy();
    let runtime_root = runtime_root.to_string_lossy();
    Ok(format!(
        "[Unit]\nDescription=CognitiveOS Personal daemon ({})\nAfter=default.target\n\n[Service]\nType=simple\nExecStart={executable} --personal --bind 127.0.0.1:{} --runtime-root {runtime_root}\nRestart=on-failure\nRestartSec=2\nTimeoutStartSec=15\nTimeoutStopSec=15\nNoNewPrivileges=true\n\n[Install]\nWantedBy=default.target\n",
        unit_kind.runtime_directory(),
        unit_kind.health_port(),
    ))
}

/// Atomically publish one rendered unit into a product-selected user-unit
/// directory. Test fixtures may inject an isolated directory; callers must
/// never derive this path from a bundle or manifest.
pub fn write_rendered_personal_user_service_unit(
    unit_kind: PersonalUserServiceUnitKind,
    unit_directory: &Path,
    deployment_root: &Path,
    version: &str,
) -> Result<PathBuf, LinuxBundleServiceError> {
    if !safe_unit_path(unit_directory) {
        return Err(LinuxBundleServiceError::UnsafeServiceConfiguration);
    }
    fs::create_dir_all(unit_directory).map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?;
    let directory_metadata = fs::symlink_metadata(unit_directory)
        .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?;
    if !directory_metadata.file_type().is_dir() || directory_metadata.file_type().is_symlink() {
        return Err(LinuxBundleServiceError::UnsafeUnitTemplate);
    }
    set_private_directory_permissions(unit_directory)?;

    let unit_path = unit_directory.join(unit_kind.unit_name());
    if unit_path.exists()
        && !fs::symlink_metadata(&unit_path)
            .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?
            .file_type()
            .is_file()
    {
        return Err(LinuxBundleServiceError::UnsafeUnitTemplate);
    }
    let temporary_path = unit_directory.join(format!(
        ".{}-{}.tmp",
        unit_kind.unit_name(),
        std::process::id()
    ));
    let unit_contents = render_personal_user_service_unit(unit_kind, deployment_root, version)?;
    let mut temporary_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)
        .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?;
    temporary_file
        .write_all(unit_contents.as_bytes())
        .and_then(|()| temporary_file.sync_all())
        .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?;
    set_private_file_permissions(&temporary_path)?;
    fs::rename(&temporary_path, &unit_path)
        .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?;
    Ok(unit_path)
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> Result<(), LinuxBundleServiceError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)
}

#[cfg(not(unix))]
fn set_private_directory_permissions(_path: &Path) -> Result<(), LinuxBundleServiceError> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<(), LinuxBundleServiceError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> Result<(), LinuxBundleServiceError> {
    Ok(())
}

/// A controller may perform only product-selected, non-secret service actions.
pub trait LinuxBundleServiceController {
    fn start_candidate(
        &mut self,
        version: &str,
        candidate_directory: &Path,
    ) -> Result<(), LinuxBundleServiceError>;
    fn stop_candidate(&mut self, version: &str) -> Result<(), LinuxBundleServiceError>;
    fn confirm_candidate_health(&mut self, version: &str) -> Result<(), LinuxBundleServiceError>;
    fn start_active(&mut self, version: &str) -> Result<(), LinuxBundleServiceError>;
    fn confirm_active(&mut self, version: &str) -> Result<(), LinuxBundleServiceError>;
}

/// Non-secret receipt issued only after both pointer and active service agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxBundleServiceReceipt {
    pub installed_version: String,
    pub previous_active_version: Option<String>,
    pub resulting_active_version: String,
    pub trusted_key_id: String,
    pub trusted_keyring_version: String,
}

/// Verify, stage, start, health-check, activate, confirm, or compensate.
pub fn install_linux_bundle_service(
    bundle_directory: &Path,
    deployment_root: &Path,
    expected_pi_compatibility: &ExpectedPiCompatibility,
    trusted_keyring: &TrustedKeyring,
    service_controller: &mut impl LinuxBundleServiceController,
) -> Result<LinuxBundleServiceReceipt, LinuxBundleServiceError> {
    // Verification is intentionally complete before a lease, deployment root,
    // unit, service, or other installation state can be mutated.
    let verified_bundle =
        verify_linux_bundle(bundle_directory, expected_pi_compatibility, trusted_keyring)?;
    let target_version = verified_bundle.manifest().version.clone();
    let _installer_lease = InstallerLifecycleLease::acquire(deployment_root)?;
    let deployment = LinuxBundleDeployment::open(deployment_root)?;
    let previous_active_version = deployment.validated_active_version()?;

    if previous_active_version.as_deref() == Some(target_version.as_str())
        && service_controller.confirm_active(&target_version).is_ok()
    {
        return Ok(service_receipt(
            &target_version,
            previous_active_version,
            trusted_keyring,
            verified_bundle.trusted_key_id(),
        ));
    }

    let candidate_directory =
        deployment.stage_verified_bundle(bundle_directory, &verified_bundle)?;
    let candidate_started = service_controller
        .start_candidate(&target_version, &candidate_directory)
        .is_ok();
    if !candidate_started {
        return compensate_failure(
            &deployment,
            previous_active_version.as_deref(),
            &target_version,
            false,
            service_controller,
            LinuxBundleServiceError::CandidateStartFailed,
        );
    }
    if service_controller
        .confirm_candidate_health(&target_version)
        .is_err()
    {
        return compensate_failure(
            &deployment,
            previous_active_version.as_deref(),
            &target_version,
            true,
            service_controller,
            LinuxBundleServiceError::CandidateHealthFailed,
        );
    }
    if service_controller.stop_candidate(&target_version).is_err() {
        return compensate_failure(
            &deployment,
            previous_active_version.as_deref(),
            &target_version,
            true,
            service_controller,
            LinuxBundleServiceError::CandidateStartFailed,
        );
    }
    if deployment.activate_staged_bundle(&verified_bundle).is_err() {
        return compensate_failure(
            &deployment,
            previous_active_version.as_deref(),
            &target_version,
            false,
            service_controller,
            LinuxBundleServiceError::Installation(
                LinuxBundleError::ActiveVersionConfirmationFailed,
            ),
        );
    }
    let pointer_confirmed =
        deployment.validated_active_version()?.as_deref() == Some(target_version.as_str());
    let active_started = service_controller.start_active(&target_version).is_ok();
    let active_confirmed =
        active_started && service_controller.confirm_active(&target_version).is_ok();
    if !pointer_confirmed || !active_confirmed {
        return compensate_failure(
            &deployment,
            previous_active_version.as_deref(),
            &target_version,
            false,
            service_controller,
            LinuxBundleServiceError::FinalServiceConfirmationFailed,
        );
    }

    Ok(service_receipt(
        &target_version,
        previous_active_version,
        trusted_keyring,
        verified_bundle.trusted_key_id(),
    ))
}

fn service_receipt(
    target_version: &str,
    previous_active_version: Option<String>,
    trusted_keyring: &TrustedKeyring,
    trusted_key_id: &str,
) -> LinuxBundleServiceReceipt {
    LinuxBundleServiceReceipt {
        installed_version: target_version.to_owned(),
        previous_active_version,
        resulting_active_version: target_version.to_owned(),
        trusted_key_id: trusted_key_id.to_owned(),
        trusted_keyring_version: trusted_keyring.version().to_owned(),
    }
}

fn compensate_failure(
    deployment: &LinuxBundleDeployment,
    previous_active_version: Option<&str>,
    target_version: &str,
    candidate_started: bool,
    service_controller: &mut impl LinuxBundleServiceController,
    original_error: LinuxBundleServiceError,
) -> Result<LinuxBundleServiceReceipt, LinuxBundleServiceError> {
    // A fail-closed start preflight has not touched a unit. Stopping the
    // canonical unit in that case could interrupt the still-healthy release.
    let candidate_stopped =
        !candidate_started || service_controller.stop_candidate(target_version).is_ok();
    let rollback_succeeded = match previous_active_version {
        Some(previous_version) => {
            deployment.restore_active_version(previous_version).is_ok()
                && service_controller.start_active(previous_version).is_ok()
                && service_controller.confirm_active(previous_version).is_ok()
        }
        None => deployment.clear_active_version().is_ok(),
    };
    if candidate_stopped && rollback_succeeded {
        Err(original_error)
    } else {
        Err(LinuxBundleServiceError::RollbackIncomplete)
    }
}

/// Fixed, user-only systemd controller. It fails before unit mutation when a
/// release has not supplied the future safe extracted daemon layout.
pub struct SystemdUserServiceController {
    deployment_root: PathBuf,
    rendered_unit_template: PathBuf,
    active_health_address: SocketAddr,
    candidate_health_address: SocketAddr,
    command_timeout: Duration,
    health_timeout: Duration,
}

impl SystemdUserServiceController {
    pub fn new(
        deployment_root: impl Into<PathBuf>,
        rendered_unit_template: impl Into<PathBuf>,
        health_address: SocketAddr,
    ) -> Result<Self, LinuxBundleServiceError> {
        if !health_address.ip().is_loopback() || health_address.port() != ACTIVE_HEALTH_PORT {
            return Err(LinuxBundleServiceError::UnsafeServiceConfiguration);
        }
        let candidate_health_address = SocketAddr::new(health_address.ip(), CANDIDATE_HEALTH_PORT);
        Ok(Self {
            deployment_root: deployment_root.into(),
            rendered_unit_template: rendered_unit_template.into(),
            active_health_address: health_address,
            candidate_health_address,
            command_timeout: Duration::from_secs(15),
            health_timeout: Duration::from_secs(10),
        })
    }

    fn ensure_rendered_unit_and_layout(
        &self,
        version: &str,
        unit_kind: PersonalUserServiceUnitKind,
    ) -> Result<(), LinuxBundleServiceError> {
        if !safe_service_version(version) {
            return Err(LinuxBundleServiceError::UnsafeServiceConfiguration);
        }
        let template = fs::read_to_string(&self.rendered_unit_template)
            .map_err(|_| LinuxBundleServiceError::UnsafeUnitTemplate)?;
        if template.contains('@')
            || !template.contains("[Service]")
            || template.contains("sudo")
            || template.contains("/root")
            || template.contains("systemctl")
        {
            return Err(LinuxBundleServiceError::UnsafeUnitTemplate);
        }
        let executable = self
            .deployment_root
            .join(unit_kind.executable_directory())
            .join(version)
            .join("bin/kernel-server");
        if !executable.is_file() {
            return Err(LinuxBundleServiceError::UnsafeServiceConfiguration);
        }
        Ok(())
    }

    fn invoke_systemctl(
        &self,
        action: &str,
        unit_kind: PersonalUserServiceUnitKind,
    ) -> Result<(), LinuxBundleServiceError> {
        let mut child = Command::new("systemctl")
            .args([
                "--user",
                "--no-ask-password",
                "--no-pager",
                action,
                unit_kind.unit_name(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| LinuxBundleServiceError::CandidateStartFailed)?;
        let stdout = child
            .stdout
            .take()
            .ok_or(LinuxBundleServiceError::CandidateStartFailed)?;
        let stderr = child
            .stderr
            .take()
            .ok_or(LinuxBundleServiceError::CandidateStartFailed)?;
        let stdout_reader = thread::spawn(move || drain_capped_output(stdout));
        let stderr_reader = thread::spawn(move || drain_capped_output(stderr));
        let deadline = Instant::now() + self.command_timeout;
        let timed_out = loop {
            match child.try_wait() {
                Ok(Some(_)) => break false,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    break true;
                }
                Ok(None) => thread::sleep(Duration::from_millis(20)),
                Err(_) => return Err(LinuxBundleServiceError::CandidateStartFailed),
            }
        };
        let status = child
            .wait()
            .map_err(|_| LinuxBundleServiceError::CandidateStartFailed)?;
        let stdout_exceeded_cap = stdout_reader
            .join()
            .map_err(|_| LinuxBundleServiceError::CandidateStartFailed)?;
        let stderr_exceeded_cap = stderr_reader
            .join()
            .map_err(|_| LinuxBundleServiceError::CandidateStartFailed)?;
        if timed_out {
            return Err(LinuxBundleServiceError::ServiceCommandTimedOut);
        }
        if status.success() && !stdout_exceeded_cap && !stderr_exceeded_cap {
            Ok(())
        } else {
            Err(LinuxBundleServiceError::CandidateStartFailed)
        }
    }
}

/// Drain output fully so a hostile or misconfigured child cannot block on a
/// full pipe. Only the cap outcome survives; no output becomes an error,
/// receipt, diagnostic, or log payload.
fn drain_capped_output(mut reader: impl Read) -> bool {
    let mut buffer = [0_u8; 1024];
    let mut total_bytes = 0_usize;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => return total_bytes > MAX_SYSTEMCTL_OUTPUT_BYTES,
            Ok(read_bytes) => total_bytes = total_bytes.saturating_add(read_bytes),
        }
    }
}

impl LinuxBundleServiceController for SystemdUserServiceController {
    fn start_candidate(
        &mut self,
        version: &str,
        _candidate_directory: &Path,
    ) -> Result<(), LinuxBundleServiceError> {
        // A canonical active unit cannot launch a staged candidate without
        // changing the active pointer. Future safe extraction must supply a
        // separately reviewed candidate unit/layout before this call gains a
        // systemctl action; the current opaque archive fails closed here.
        self.ensure_rendered_unit_and_layout(version, PersonalUserServiceUnitKind::Candidate)?;
        self.invoke_systemctl("start", PersonalUserServiceUnitKind::Candidate)
    }

    fn stop_candidate(&mut self, _version: &str) -> Result<(), LinuxBundleServiceError> {
        self.invoke_systemctl("stop", PersonalUserServiceUnitKind::Candidate)
    }

    fn confirm_candidate_health(&mut self, _version: &str) -> Result<(), LinuxBundleServiceError> {
        probe_personal_health(self.candidate_health_address, self.health_timeout)
    }

    fn start_active(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.ensure_rendered_unit_and_layout(version, PersonalUserServiceUnitKind::Active)?;
        self.invoke_systemctl("restart", PersonalUserServiceUnitKind::Active)
    }

    fn confirm_active(&mut self, _version: &str) -> Result<(), LinuxBundleServiceError> {
        probe_personal_health(self.active_health_address, self.health_timeout)
    }
}

/// Strict bounded liveness probe; it is deliberately not a readiness check.
pub fn probe_personal_health(
    address: SocketAddr,
    overall_timeout: Duration,
) -> Result<(), LinuxBundleServiceError> {
    if !address.ip().is_loopback() {
        return Err(LinuxBundleServiceError::UnsafeServiceConfiguration);
    }
    let deadline = Instant::now() + overall_timeout;
    for _attempt in 0..MAX_HEALTH_ATTEMPTS {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        let per_attempt = remaining.min(Duration::from_secs(2));
        if health_attempt(address, per_attempt).is_ok() {
            return Ok(());
        }
    }
    Err(LinuxBundleServiceError::CandidateHealthFailed)
}

fn health_attempt(address: SocketAddr, timeout: Duration) -> Result<(), LinuxBundleServiceError> {
    let mut stream = TcpStream::connect_timeout(&address, timeout)
        .map_err(|_| LinuxBundleServiceError::CandidateHealthFailed)?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| LinuxBundleServiceError::CandidateHealthFailed)?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|_| LinuxBundleServiceError::CandidateHealthFailed)?;
    stream
        .write_all(b"GET /personal/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .map_err(|_| LinuxBundleServiceError::CandidateHealthFailed)?;
    let mut response = Vec::new();
    stream
        .take((MAX_HEALTH_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response)
        .map_err(|_| LinuxBundleServiceError::CandidateHealthFailed)?;
    if response.len() > MAX_HEALTH_RESPONSE_BYTES {
        return Err(LinuxBundleServiceError::InvalidHealthResponse);
    }
    validate_health_response(&response)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersonalHealthResponse {
    schema_version: u8,
    surface: String,
    status: String,
    authority_side_effects: bool,
    readiness_claim: String,
    profile_claim: String,
}

fn validate_health_response(response: &[u8]) -> Result<(), LinuxBundleServiceError> {
    let response_text = std::str::from_utf8(response)
        .map_err(|_| LinuxBundleServiceError::InvalidHealthResponse)?;
    let (header_text, body_text) = response_text
        .split_once("\r\n\r\n")
        .ok_or(LinuxBundleServiceError::InvalidHealthResponse)?;
    if !header_text.starts_with("HTTP/1.1 200 ") || header_text.contains("Transfer-Encoding") {
        return Err(LinuxBundleServiceError::InvalidHealthResponse);
    }
    let content_length = header_text
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or(LinuxBundleServiceError::InvalidHealthResponse)?;
    if content_length != body_text.len() {
        return Err(LinuxBundleServiceError::InvalidHealthResponse);
    }
    let health: PersonalHealthResponse = serde_json::from_str(body_text)
        .map_err(|_| LinuxBundleServiceError::InvalidHealthResponse)?;
    if health.schema_version == 1
        && health.surface == "personal-health"
        && health.status == "ok"
        && !health.authority_side_effects
        && health.readiness_claim == "not-claimed"
        && health.profile_claim == "not-claimed"
    {
        Ok(())
    } else {
        Err(LinuxBundleServiceError::InvalidHealthResponse)
    }
}

fn safe_service_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 128
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn safe_unit_path(path: &Path) -> bool {
    let path_text = path.to_string_lossy();
    !path_text.is_empty()
        && path_text.len() <= 4096
        && !path_text.chars().any(char::is_control)
        && !path_text.contains('@')
        && !path_text.contains('"')
        && !path_text.contains('\'')
}
