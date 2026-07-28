//! Failure-closed orchestration for local Personal Linux bundle installation.
//!
//! This module composes the offline attestation verifier with staged
//! activation. It intentionally does not download bundles, discover trust
//! roots, spawn a daemon, call a service manager, or create authority state.

use crate::linux_bundle::{
    ExpectedPiCompatibility, LinuxBundleDeployment, LinuxBundleError, TrustedKeyring,
    verify_linux_bundle,
};
use std::path::Path;

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
///
/// The order is fixed and failure-closed:
///
/// 1. [`verify_linux_bundle`] performs the complete offline verification.
/// 2. Only successful verification permits opening or creating deployment
///    state.
/// 3. The previous active version is read before staging.
/// 4. The privately constructed verified value is staged, including its
///    pre-write artifact re-hash.
/// 5. The staged candidate is passed to the caller's health check.
/// 6. A successful check permits atomic active-pointer replacement.
/// 7. The active pointer is re-read and must name the verified version before
///    a receipt is returned.
///
/// Reinstalling the already active version is idempotent: verification,
/// staging, and health checking still run; the existing version directory is
/// retained, temporary staging is removed, and the confirmed receipt records
/// the same previous and resulting version.
///
/// This API makes no downloader, service lifecycle, cross-process lease,
/// uninstall, release, Gate, Profile, containment, capability, Intent,
/// Effect, Task transition, or authority-completion claim.
pub fn install_linux_bundle(
    bundle_directory: &Path,
    deployment_root: &Path,
    expected_pi_compatibility: &ExpectedPiCompatibility,
    trusted_keyring: &TrustedKeyring,
    health_check: impl FnOnce(&Path) -> bool,
) -> Result<LinuxBundleInstallationReceipt, LinuxBundleError> {
    let verified_bundle =
        verify_linux_bundle(bundle_directory, expected_pi_compatibility, trusted_keyring)?;

    // No deployment path is opened or created before complete verification.
    let deployment = LinuxBundleDeployment::open(deployment_root)?;
    let previous_active_version = deployment.active_version()?;
    deployment.stage_verified_bundle(bundle_directory, &verified_bundle)?;
    deployment.activate_after_health_check(&verified_bundle, health_check)?;

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
