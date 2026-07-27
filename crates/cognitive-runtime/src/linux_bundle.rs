//! Offline validation and activation primitives for the Personal Linux bundle.
//!
//! This module intentionally does not download artifacts, call `systemctl`, or
//! start a daemon. It establishes the failure-first bundle boundary used by a
//! future inspected installer: validate a local release directory, stage it,
//! run a caller-supplied health check, and only then atomically replace the
//! active-version pointer.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

const EXPECTED_PLATFORM: &str = "linux-x86_64";
const ACTIVE_VERSION_FILE: &str = "active-version";

/// A release manifest deliberately limited to non-secret distribution facts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LinuxBundleManifest {
    pub schema_version: u32,
    pub platform: String,
    pub version: String,
    pub artifact_file: String,
    pub artifact_sha256: String,
    pub attestation_reference: String,
    pub pi_version: String,
    pub pi_integrity: String,
}

/// Errors intentionally distinguish an untrusted bundle from an interrupted
/// or unhealthy activation. None include artifact contents or secret material.
#[derive(Debug, Error)]
pub enum LinuxBundleError {
    #[error("Linux bundle manifest is invalid: {0}")]
    InvalidManifest(String),
    #[error("Linux bundle is unsupported on platform {actual}")]
    UnsupportedPlatform { actual: String },
    #[error("Linux bundle artifact digest does not match its manifest")]
    ArtifactDigestMismatch,
    #[error("Linux bundle attestation reference is missing or unsupported")]
    InvalidAttestationReference,
    #[error("Linux bundle Pi compatibility pin does not match the expected pin")]
    PiCompatibilityMismatch,
    #[error("Linux bundle contains a forbidden vendored runtime payload: {0}")]
    ForbiddenPayload(String),
    #[error("Linux bundle path is unsafe: {0}")]
    UnsafePath(String),
    #[error("Linux bundle activation health check failed")]
    HealthCheckFailed,
    #[error("Linux bundle filesystem operation failed: {0}")]
    Io(#[from] io::Error),
}

/// Non-secret compatibility facts fixed by the product's Pi compatibility pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedPiCompatibility {
    pub version: String,
    pub integrity: String,
}

impl ExpectedPiCompatibility {
    pub fn new(version: impl Into<String>, integrity: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            integrity: integrity.into(),
        }
    }
}

/// Validate a downloaded release directory before it can be staged.
///
/// The caller supplies the Pi pin from the single authoritative compatibility
/// source; a manifest is never allowed to select its own accepted Pi version.
pub fn verify_linux_bundle(
    bundle_directory: &Path,
    expected_pi: &ExpectedPiCompatibility,
) -> Result<LinuxBundleManifest, LinuxBundleError> {
    let manifest_path = bundle_directory.join("manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest: LinuxBundleManifest = serde_json::from_str(&manifest_text)
        .map_err(|error| LinuxBundleError::InvalidManifest(error.to_string()))?;

    if manifest.schema_version != 1 || manifest.version.is_empty() {
        return Err(LinuxBundleError::InvalidManifest(
            "schema_version must be 1 and version must be non-empty".to_owned(),
        ));
    }
    if manifest.platform != EXPECTED_PLATFORM {
        return Err(LinuxBundleError::UnsupportedPlatform {
            actual: manifest.platform,
        });
    }
    if !manifest.attestation_reference.starts_with("https://") {
        return Err(LinuxBundleError::InvalidAttestationReference);
    }
    if manifest.pi_version != expected_pi.version || manifest.pi_integrity != expected_pi.integrity
    {
        return Err(LinuxBundleError::PiCompatibilityMismatch);
    }

    let artifact_path = checked_child_path(bundle_directory, &manifest.artifact_file)?;
    reject_vendored_runtime_payloads(bundle_directory)?;
    let artifact = fs::read(artifact_path)?;
    if sha256_digest(&artifact) != manifest.artifact_sha256 {
        return Err(LinuxBundleError::ArtifactDigestMismatch);
    }
    Ok(manifest)
}

/// Filesystem deployment state. `active-version` is a small text pointer that
/// is atomically replaced with `rename`, which keeps this model testable on the
/// supported Windows CI host without pretending it is a systemd service.
#[derive(Debug, Clone)]
pub struct LinuxBundleDeployment {
    root: PathBuf,
}

impl LinuxBundleDeployment {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, LinuxBundleError> {
        let root = root.into();
        fs::create_dir_all(root.join("staged"))?;
        fs::create_dir_all(root.join("versions"))?;
        Ok(Self { root })
    }

    pub fn active_version(&self) -> Result<Option<String>, LinuxBundleError> {
        match fs::read_to_string(self.root.join(ACTIVE_VERSION_FILE)) {
            Ok(version) => Ok(Some(version.trim().to_owned())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Copies only a verified bundle to a version-specific staging location.
    /// It never changes the active pointer, so process interruption after this
    /// method leaves the prior version active.
    pub fn stage_verified_bundle(
        &self,
        bundle_directory: &Path,
        manifest: &LinuxBundleManifest,
    ) -> Result<PathBuf, LinuxBundleError> {
        let version_directory = safe_version_directory(&manifest.version)?;
        let source_artifact = checked_child_path(bundle_directory, &manifest.artifact_file)?;
        let staging_directory = self.root.join("staged").join(version_directory);
        if staging_directory.exists() {
            fs::remove_dir_all(&staging_directory)?;
        }
        fs::create_dir_all(&staging_directory)?;
        fs::copy(
            source_artifact,
            staging_directory.join(&manifest.artifact_file),
        )?;
        fs::write(
            staging_directory.join("manifest.json"),
            serde_json::to_vec(manifest)
                .map_err(|error| LinuxBundleError::InvalidManifest(error.to_string()))?,
        )?;
        Ok(staging_directory)
    }

    /// Promotes a staged version only after the caller has performed its bounded
    /// health check. A health failure retains both the old active pointer and
    /// the staged candidate for operator inspection or explicit cleanup.
    pub fn activate_after_health_check(
        &self,
        manifest: &LinuxBundleManifest,
        health_check: impl FnOnce(&Path) -> bool,
    ) -> Result<(), LinuxBundleError> {
        let version_directory = safe_version_directory(&manifest.version)?;
        let staging_directory = self.root.join("staged").join(version_directory);
        if !staging_directory.is_dir() {
            return Err(LinuxBundleError::InvalidManifest(
                "verified staged version is missing".to_owned(),
            ));
        }
        if !health_check(&staging_directory) {
            return Err(LinuxBundleError::HealthCheckFailed);
        }

        let version_directory_path = self.root.join("versions").join(version_directory);
        if !version_directory_path.exists() {
            fs::rename(&staging_directory, &version_directory_path)?;
        } else {
            fs::remove_dir_all(&staging_directory)?;
        }
        self.replace_active_version(&manifest.version)
    }

    fn replace_active_version(&self, version: &str) -> Result<(), LinuxBundleError> {
        let temporary_path = self.root.join(format!("{ACTIVE_VERSION_FILE}.new"));
        fs::write(&temporary_path, format!("{version}\n"))?;
        fs::rename(temporary_path, self.root.join(ACTIVE_VERSION_FILE))?;
        Ok(())
    }
}

fn checked_child_path(root: &Path, child: &str) -> Result<PathBuf, LinuxBundleError> {
    let child_path = Path::new(child);
    if child_path.as_os_str().is_empty()
        || child_path.is_absolute()
        || child_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(LinuxBundleError::UnsafePath(child.to_owned()));
    }
    Ok(root.join(child_path))
}

fn safe_version_directory(version: &str) -> Result<&str, LinuxBundleError> {
    if version.is_empty()
        || version.contains('/')
        || version.contains('\\')
        || version.contains("..")
        || version
            .chars()
            .any(|character| !(character.is_ascii_alphanumeric() || ".-_+".contains(character)))
    {
        return Err(LinuxBundleError::UnsafePath(version.to_owned()));
    }
    Ok(version)
}

fn reject_vendored_runtime_payloads(bundle_directory: &Path) -> Result<(), LinuxBundleError> {
    for entry in fs::read_dir(bundle_directory)? {
        let entry = entry?;
        let lower_case_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if lower_case_name == "node"
            || lower_case_name == "node.exe"
            || lower_case_name.starts_with("node-")
            || lower_case_name == "pi"
            || lower_case_name == "pi.exe"
            || lower_case_name.starts_with("pi-")
        {
            return Err(LinuxBundleError::ForbiddenPayload(lower_case_name));
        }
    }
    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    const PI_VERSION: &str = "0.81.1";
    const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";

    fn expected_pi() -> ExpectedPiCompatibility {
        ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY)
    }

    fn write_bundle(directory: &Path, version: &str) -> LinuxBundleManifest {
        let artifact = b"cognitiveos daemon bundle";
        let manifest = LinuxBundleManifest {
            schema_version: 1,
            platform: EXPECTED_PLATFORM.to_owned(),
            version: version.to_owned(),
            artifact_file: "cognitiveos-linux-x86_64.tar.gz".to_owned(),
            artifact_sha256: sha256_digest(artifact),
            attestation_reference: "https://example.invalid/attestations/v1".to_owned(),
            pi_version: PI_VERSION.to_owned(),
            pi_integrity: PI_INTEGRITY.to_owned(),
        };
        fs::write(directory.join(&manifest.artifact_file), artifact).unwrap();
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        manifest
    }

    #[test]
    fn rejects_tampered_artifacts_missing_attestations_and_wrong_pi_integrity() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let manifest = write_bundle(temporary_directory.path(), "1.0.0");
        fs::write(
            temporary_directory.path().join(&manifest.artifact_file),
            b"tampered bundle",
        )
        .unwrap();
        assert!(matches!(
            verify_linux_bundle(temporary_directory.path(), &expected_pi()),
            Err(LinuxBundleError::ArtifactDigestMismatch)
        ));

        let manifest = write_bundle(temporary_directory.path(), "1.0.0");
        let no_attestation = LinuxBundleManifest {
            attestation_reference: String::new(),
            ..manifest.clone()
        };
        fs::write(
            temporary_directory.path().join("manifest.json"),
            serde_json::to_vec(&no_attestation).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            verify_linux_bundle(temporary_directory.path(), &expected_pi()),
            Err(LinuxBundleError::InvalidAttestationReference)
        ));

        let wrong_pi = LinuxBundleManifest {
            pi_integrity: "sha512:wrong".to_owned(),
            ..manifest
        };
        fs::write(
            temporary_directory.path().join("manifest.json"),
            serde_json::to_vec(&wrong_pi).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            verify_linux_bundle(temporary_directory.path(), &expected_pi()),
            Err(LinuxBundleError::PiCompatibilityMismatch)
        ));
    }

    #[test]
    fn rejects_vendored_node_or_pi_payloads() {
        let temporary_directory = tempfile::tempdir().unwrap();
        write_bundle(temporary_directory.path(), "1.0.0");
        fs::write(temporary_directory.path().join("node"), b"forbidden").unwrap();
        assert!(matches!(
            verify_linux_bundle(temporary_directory.path(), &expected_pi()),
            Err(LinuxBundleError::ForbiddenPayload(payload)) if payload == "node"
        ));
    }

    #[test]
    fn interruption_and_failed_health_preserve_active_version_and_user_data() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let deployment =
            LinuxBundleDeployment::open(temporary_directory.path().join("deploy")).unwrap();
        deployment.replace_active_version("1.0.0").unwrap();
        let user_data = temporary_directory.path().join("user-data.sqlite");
        fs::write(&user_data, b"do-not-migrate-before-verification").unwrap();

        let bundle_directory = temporary_directory.path().join("bundle");
        fs::create_dir(&bundle_directory).unwrap();
        let manifest = write_bundle(&bundle_directory, "2.0.0");
        let verified_manifest = verify_linux_bundle(&bundle_directory, &expected_pi()).unwrap();
        deployment
            .stage_verified_bundle(&bundle_directory, &verified_manifest)
            .unwrap();
        assert_eq!(
            deployment.active_version().unwrap().as_deref(),
            Some("1.0.0")
        );
        assert_eq!(
            fs::read(&user_data).unwrap(),
            b"do-not-migrate-before-verification"
        );

        assert!(matches!(
            deployment.activate_after_health_check(&manifest, |_| false),
            Err(LinuxBundleError::HealthCheckFailed)
        ));
        assert_eq!(
            deployment.active_version().unwrap().as_deref(),
            Some("1.0.0")
        );
        assert_eq!(
            fs::read(&user_data).unwrap(),
            b"do-not-migrate-before-verification"
        );
    }

    #[test]
    fn successful_health_check_atomically_switches_and_retains_previous_version() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let deployment =
            LinuxBundleDeployment::open(temporary_directory.path().join("deploy")).unwrap();
        deployment.replace_active_version("1.0.0").unwrap();
        fs::create_dir_all(temporary_directory.path().join("deploy/versions/1.0.0")).unwrap();
        let bundle_directory = temporary_directory.path().join("bundle");
        fs::create_dir(&bundle_directory).unwrap();
        let manifest = write_bundle(&bundle_directory, "2.0.0");
        let verified_manifest = verify_linux_bundle(&bundle_directory, &expected_pi()).unwrap();
        deployment
            .stage_verified_bundle(&bundle_directory, &verified_manifest)
            .unwrap();

        deployment
            .activate_after_health_check(&manifest, |staged_directory| {
                staged_directory.join("manifest.json").is_file()
            })
            .unwrap();

        assert_eq!(
            deployment.active_version().unwrap().as_deref(),
            Some("2.0.0")
        );
        assert!(
            temporary_directory
                .path()
                .join("deploy/versions/1.0.0")
                .is_dir()
        );
        assert!(
            temporary_directory
                .path()
                .join("deploy/versions/2.0.0")
                .is_dir()
        );
    }
}
