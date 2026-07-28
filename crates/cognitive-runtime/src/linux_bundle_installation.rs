//! Failure-closed orchestration for local Personal Linux bundle installation.
//!
//! This module composes the offline attestation verifier with staged
//! activation. It intentionally does not download bundles, discover trust
//! roots, spawn a daemon, call a service manager, or create authority state.
//! The installer lifecycle is serialized per deployment root by a stable,
//! product-owned OS file lock.

use crate::linux_bundle::{
    ExpectedPiCompatibility, LinuxBundleDeployment, LinuxBundleError, TrustedKeyring,
    verify_linux_bundle,
};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

const INSTALLER_LEASE_FILE_PREFIX: &str = ".cognitiveos-personal-installer-lease-v1-";
const INSTALLER_LEASE_FILE_SUFFIX: &str = ".lock";

/// Crate-private guard for every mutable installer lifecycle transaction.
///
/// The lock stays intentionally hidden from public callers: service-aware
/// orchestration can share this OS-backed lease without exposing lock paths,
/// owner data, or a second locking model.
pub(crate) struct InstallerLifecycleLease {
    lock_file: File,
}

impl InstallerLifecycleLease {
    pub(crate) fn acquire(deployment_root: &Path) -> Result<Self, LinuxBundleError> {
        let lock_path = installer_lease_path(deployment_root)?;
        let lock_file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;

        // `flock` on Unix and `LockFileEx` on Windows/MSVC are process-backed
        // advisory locks. The empty, product-owned file is intentionally
        // persistent so stale metadata never needs TTL-based reclamation.
        match lock_file.try_lock() {
            Ok(()) => Ok(Self { lock_file }),
            Err(std::fs::TryLockError::WouldBlock) => Err(LinuxBundleError::InstallationLeaseHeld),
            Err(std::fs::TryLockError::Error(error)) => Err(LinuxBundleError::Io(error)),
        }
    }
}

impl Drop for InstallerLifecycleLease {
    fn drop(&mut self) {
        // The OS releases the lock when this descriptor is closed, including
        // panic unwinding and abnormal process termination. Do not unlink the
        // stable lock path: a successor may already have opened the same file.
        let _ = self.lock_file.unlock();
    }
}

fn installer_lease_path(deployment_root: &Path) -> Result<PathBuf, LinuxBundleError> {
    let root_parent = deployment_root.parent().unwrap_or_else(|| Path::new("."));
    let root_name = deployment_root.file_name().ok_or_else(|| {
        LinuxBundleError::UnsafePath("deployment root must have a stable name".to_owned())
    })?;
    let canonical_parent = fs::canonicalize(root_parent)?;
    let canonical_root = if deployment_root.exists() {
        fs::canonicalize(deployment_root)?
    } else {
        canonical_parent.join(root_name)
    };
    let path_digest = Sha256::digest(canonical_root.to_string_lossy().as_bytes());
    Ok(canonical_parent.join(format!(
        "{INSTALLER_LEASE_FILE_PREFIX}{path_digest:x}{INSTALLER_LEASE_FILE_SUFFIX}"
    )))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "test-fault-injection")]
pub enum InstallFaultPoint {
    LeaseAcquiredBeforeDeploymentOpen,
    DeploymentOpenedBeforeStage,
    StageCompletedBeforeHealth,
    HealthSucceededBeforeActivation,
    ActivationCompletedBeforeReceiptConfirmation,
    PanicAfterLeaseAcquired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InternalFaultPoint {
    LeaseAcquiredBeforeDeploymentOpen,
    DeploymentOpenedBeforeStage,
    StageCompletedBeforeHealth,
    HealthSucceededBeforeActivation,
    ActivationCompletedBeforeReceiptConfirmation,
}

#[cfg(feature = "test-fault-injection")]
fn fault_injection_error(point: InstallFaultPoint) -> LinuxBundleError {
    LinuxBundleError::FaultInjected(match point {
        InstallFaultPoint::LeaseAcquiredBeforeDeploymentOpen => "fault injected after lease",
        InstallFaultPoint::DeploymentOpenedBeforeStage => "fault injected after deployment open",
        InstallFaultPoint::StageCompletedBeforeHealth => "fault injected after staging",
        InstallFaultPoint::HealthSucceededBeforeActivation => "fault injected after health",
        InstallFaultPoint::ActivationCompletedBeforeReceiptConfirmation => {
            "fault injected after activation"
        }
        InstallFaultPoint::PanicAfterLeaseAcquired => "panic fault requires unwinding",
    })
}

/// Non-secret facts confirmed after a Linux bundle becomes the active version.
///
/// The receipt deliberately excludes manifests, artifact bytes, attestation
/// statements, detached signatures, public-key bytes, private-key material,
/// health-check output, and user data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxBundleInstallationReceipt {
    pub installed_version: String,
    pub previous_active_version: Option<String>,
    pub resulting_active_version: String,
    pub trusted_key_id: String,
    pub trusted_keyring_version: String,
}

/// Verifies, stages, health-checks, and atomically activates one local bundle.
///
/// The caller supplies only the local bundle and deployment paths, the
/// product-fixed Pi compatibility pin, the product-owned trusted keyring, and
/// a health check whose duration and resource bounds are enforced by the
/// caller. The health check is invoked exactly once, and only after staging.
/// The deployment root may be absent, but its immediate parent must already
/// exist because the stable lease file is deliberately kept outside the root.
///
/// The order is fixed and failure-closed:
///
/// 1. [`verify_linux_bundle`] performs the complete offline verification.
/// 2. Only successful verification permits acquiring the product-owned,
///    cross-process deployment lifecycle lease.
/// 3. Only the lease holder may open or create the deployment root.
/// 4. The previous active version is read before staging.
/// 5. The privately constructed verified value is staged, including its
///    pre-write artifact re-hash.
/// 6. The staged candidate is passed exactly once to the caller's health check.
/// 7. A successful check permits atomic active-pointer replacement.
/// 8. The active pointer is re-read and must name the verified version before
///    a receipt is returned.
///
/// Reinstalling the already active version is idempotent: verification,
/// staging, and health checking still run; the existing version directory is
/// retained, temporary staging is removed, and the confirmed receipt records
/// the same previous and resulting version.
///
/// This API makes no downloader, service lifecycle, uninstall, release, Gate,
/// Profile, containment, capability, Intent,
/// Effect, Task transition, or authority-completion claim.
pub fn install_linux_bundle(
    bundle_directory: &Path,
    deployment_root: &Path,
    expected_pi_compatibility: &ExpectedPiCompatibility,
    trusted_keyring: &TrustedKeyring,
    health_check: impl FnOnce(&Path) -> bool,
) -> Result<LinuxBundleInstallationReceipt, LinuxBundleError> {
    install_linux_bundle_internal(
        bundle_directory,
        deployment_root,
        expected_pi_compatibility,
        trusted_keyring,
        |_| Ok(()),
        health_check,
    )
}

#[cfg(feature = "test-fault-injection")]
pub fn install_linux_bundle_with_fault_injection(
    bundle_directory: &Path,
    deployment_root: &Path,
    expected_pi_compatibility: &ExpectedPiCompatibility,
    trusted_keyring: &TrustedKeyring,
    fault_point: InstallFaultPoint,
    health_check: impl FnOnce(&Path) -> bool,
) -> Result<LinuxBundleInstallationReceipt, LinuxBundleError> {
    let internal_fault_point = match fault_point {
        InstallFaultPoint::LeaseAcquiredBeforeDeploymentOpen => {
            InternalFaultPoint::LeaseAcquiredBeforeDeploymentOpen
        }
        InstallFaultPoint::DeploymentOpenedBeforeStage => {
            InternalFaultPoint::DeploymentOpenedBeforeStage
        }
        InstallFaultPoint::StageCompletedBeforeHealth => {
            InternalFaultPoint::StageCompletedBeforeHealth
        }
        InstallFaultPoint::HealthSucceededBeforeActivation => {
            InternalFaultPoint::HealthSucceededBeforeActivation
        }
        InstallFaultPoint::ActivationCompletedBeforeReceiptConfirmation => {
            InternalFaultPoint::ActivationCompletedBeforeReceiptConfirmation
        }
        InstallFaultPoint::PanicAfterLeaseAcquired => {
            InternalFaultPoint::LeaseAcquiredBeforeDeploymentOpen
        }
    };
    install_linux_bundle_internal(
        bundle_directory,
        deployment_root,
        expected_pi_compatibility,
        trusted_keyring,
        |current_point| {
            if current_point == internal_fault_point {
                if fault_point == InstallFaultPoint::PanicAfterLeaseAcquired {
                    std::panic::resume_unwind(Box::new(
                        "deterministic installer lease panic fault",
                    ));
                }
                Err(fault_injection_error(fault_point))
            } else {
                Ok(())
            }
        },
        health_check,
    )
}

fn install_linux_bundle_internal(
    bundle_directory: &Path,
    deployment_root: &Path,
    expected_pi_compatibility: &ExpectedPiCompatibility,
    trusted_keyring: &TrustedKeyring,
    fault_callback: impl Fn(InternalFaultPoint) -> Result<(), LinuxBundleError>,
    health_check: impl FnOnce(&Path) -> bool,
) -> Result<LinuxBundleInstallationReceipt, LinuxBundleError> {
    let verified_bundle =
        verify_linux_bundle(bundle_directory, expected_pi_compatibility, trusted_keyring)?;

    // Complete verification precedes both lease creation and deployment-root
    // mutation. The lease is process-backed and has no TTL or owner metadata.
    let _installer_lease = InstallerLifecycleLease::acquire(deployment_root)?;
    fault_callback(InternalFaultPoint::LeaseAcquiredBeforeDeploymentOpen)?;

    let deployment = LinuxBundleDeployment::open(deployment_root)?;
    fault_callback(InternalFaultPoint::DeploymentOpenedBeforeStage)?;
    let previous_active_version = deployment.active_version()?;
    deployment.stage_verified_bundle(bundle_directory, &verified_bundle)?;
    fault_callback(InternalFaultPoint::StageCompletedBeforeHealth)?;

    let staged_candidate = deployment_root
        .join("staged")
        .join(&verified_bundle.manifest().version);
    if !health_check(&staged_candidate) {
        return Err(LinuxBundleError::HealthCheckFailed);
    }
    fault_callback(InternalFaultPoint::HealthSucceededBeforeActivation)?;
    deployment.activate_staged_bundle(&verified_bundle)?;
    fault_callback(InternalFaultPoint::ActivationCompletedBeforeReceiptConfirmation)?;

    let installed_version = verified_bundle.manifest().version.clone();
    let resulting_active_version = deployment
        .active_version()?
        .filter(|active_version| active_version == &installed_version)
        .ok_or(LinuxBundleError::ActiveVersionConfirmationFailed)?;

    Ok(LinuxBundleInstallationReceipt {
        installed_version,
        previous_active_version,
        resulting_active_version,
        trusted_key_id: verified_bundle.trusted_key_id().to_owned(),
        trusted_keyring_version: trusted_keyring.version().to_owned(),
    })
}
