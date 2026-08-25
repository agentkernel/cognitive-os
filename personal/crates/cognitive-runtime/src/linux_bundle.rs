//! Offline validation and activation primitives for the Personal Linux bundle.
//!
//! This module intentionally does not download artifacts, call `systemctl`, or
//! start a daemon. It establishes the failure-first bundle boundary used by a
//! future inspected installer: validate a local release directory, stage it,
//! run a caller-supplied health check, and only then atomically replace the
//! active-version pointer.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, VerifyingKey};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use tar::Archive as TarArchive;
use thiserror::Error;

const EXPECTED_PRODUCT: &str = "cognitiveos-personal";
const EXPECTED_PLATFORM: &str = "linux-x86_64";
const ATTESTATION_SCHEMA: &str = "cognitiveos.personal.linux-bundle-attestation";
const SIGNATURE_SCHEMA: &str = "cognitiveos.personal.linux-bundle-signature";
const ED25519_ALGORITHM: &str = "Ed25519";
const ACTIVE_VERSION_FILE: &str = "active-version";
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_ATTESTATION_STATEMENT_BYTES: usize = 64 * 1024;
const MAX_SIGNATURE_ENVELOPE_BYTES: usize = 16 * 1024;
const MAX_COMPRESSED_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_EXPANDED_ARTIFACT_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_REGULAR_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_REGULAR_FILE_ENTRIES: u64 = 1024;
const MAX_DIRECTORY_ENTRIES: u64 = 128;
const MAX_ARCHIVE_PATH_BYTES: usize = 4096;
const REQUIRED_KERNEL_SERVER_PATH: &str = "bin/kernel-server";
const REQUIRED_COGNITIVE_CLI_PATH: &str = "bin/cognitive";
const REQUIRED_EXTENSION_ROOT_PATH: &str = "extensions/pi-cognitiveos/dist";
const REQUIRED_EXTENSION_ENTRY_PATH: &str = "extensions/pi-cognitiveos/dist/index.js";
const REQUIRED_EXECUTABLE_MODE: u32 = 0o100;
const FORBIDDEN_EXECUTABLE_MODE: u32 = 0o7000;

/// A release manifest deliberately limited to non-secret distribution facts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LinuxBundleManifest {
    pub schema_version: u32,
    pub product: String,
    pub platform: String,
    pub version: String,
    pub artifact_file: String,
    pub artifact_sha256: String,
    pub attestation_reference: String,
    pub attestation_statement_file: String,
    pub attestation_signature_file: String,
    pub pi_version: String,
    pub pi_integrity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LinuxBundleAttestationStatement {
    schema: String,
    schema_version: u32,
    product: String,
    platform: String,
    version: String,
    artifact_file: String,
    artifact_sha256: String,
    pi_version: String,
    pi_integrity: String,
    provenance_reference: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LinuxBundleSignatureEnvelope {
    schema: String,
    schema_version: u32,
    key_id: String,
    algorithm: String,
    signature: String,
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
    #[error("Linux bundle attestation is malformed: {0}")]
    MalformedAttestation(&'static str),
    #[error("Linux bundle attestation version or algorithm is unsupported: {0}")]
    UnsupportedAttestation(&'static str),
    #[error("Linux bundle attestation statement is not canonical")]
    NonCanonicalAttestation,
    #[error("Linux bundle attestation references an unknown or untrusted key")]
    UnknownOrUntrustedKey,
    #[error("Linux bundle trusted keyring is invalid: {0}")]
    InvalidTrustedKeyring(&'static str),
    #[error("Linux bundle attestation signature does not verify")]
    SignatureMismatch,
    #[error("Linux bundle attestation statement does not match its manifest")]
    StatementBindingMismatch,
    #[error("Linux bundle attestation reference is missing or unsupported")]
    InvalidAttestationReference,
    #[error("Linux bundle Pi compatibility pin does not match the expected pin")]
    PiCompatibilityMismatch,
    #[error("Linux bundle contains a forbidden vendored runtime payload: {0}")]
    ForbiddenPayload(String),
    #[error("Linux bundle path is unsafe: {0}")]
    UnsafePath(String),
    #[error("Linux bundle archive is invalid or unsafe")]
    UnsafeArchive,
    #[error("Linux bundle activation health check failed")]
    HealthCheckFailed,
    #[error("Linux bundle active version could not be confirmed after activation")]
    ActiveVersionConfirmationFailed,
    #[error("Linux bundle deployment is already being installed by another process")]
    InstallationLeaseHeld,
    #[cfg(feature = "test-fault-injection")]
    #[error("Linux bundle test fault injected at {0}")]
    FaultInjected(&'static str),
    #[error("Linux bundle filesystem operation failed: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustedKeyStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedKeyInput {
    pub key_id: String,
    pub algorithm: String,
    pub public_key_base64url: String,
    pub status: TrustedKeyStatus,
}

#[derive(Clone)]
struct TrustedKey {
    verifying_key: VerifyingKey,
    status: TrustedKeyStatus,
}

/// Product-owned trust roots validated before any bundle is considered.
#[derive(Clone)]
pub struct TrustedKeyring {
    version: String,
    keys: BTreeMap<String, TrustedKey>,
}

impl TrustedKeyring {
    pub fn new(
        version: impl Into<String>,
        trusted_keys: Vec<TrustedKeyInput>,
    ) -> Result<Self, LinuxBundleError> {
        let version = version.into();
        if version.is_empty() || version.len() > 128 || !version.is_ascii() {
            return Err(LinuxBundleError::InvalidTrustedKeyring(
                "keyring version must be bounded non-empty ASCII",
            ));
        }
        if trusted_keys.is_empty() {
            return Err(LinuxBundleError::InvalidTrustedKeyring(
                "at least one trusted key is required",
            ));
        }

        let mut keys = BTreeMap::new();
        for trusted_key in trusted_keys {
            validate_key_id(&trusted_key.key_id).map_err(|_| {
                LinuxBundleError::InvalidTrustedKeyring("trusted key ID is invalid")
            })?;
            if trusted_key.algorithm != ED25519_ALGORITHM {
                return Err(LinuxBundleError::InvalidTrustedKeyring(
                    "trusted key algorithm is unsupported",
                ));
            }
            let public_key_bytes = decode_canonical_base64url(&trusted_key.public_key_base64url)
                .map_err(|_| {
                    LinuxBundleError::InvalidTrustedKeyring(
                        "trusted public key encoding is invalid",
                    )
                })?;
            let public_key_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
                LinuxBundleError::InvalidTrustedKeyring("trusted public key length is invalid")
            })?;
            let verifying_key = VerifyingKey::from_bytes(&public_key_array).map_err(|_| {
                LinuxBundleError::InvalidTrustedKeyring("trusted public key is invalid")
            })?;
            if keys
                .insert(
                    trusted_key.key_id,
                    TrustedKey {
                        verifying_key,
                        status: trusted_key.status,
                    },
                )
                .is_some()
            {
                return Err(LinuxBundleError::InvalidTrustedKeyring(
                    "trusted key IDs must be unique",
                ));
            }
        }
        Ok(Self { version, keys })
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Proof that manifest, artifact, Pi pin, statement, trust root and signature
/// passed the complete offline verifier. Fields are private by design.
#[derive(Debug, Clone)]
pub struct VerifiedLinuxBundle {
    manifest: LinuxBundleManifest,
    trusted_key_id: String,
}

impl VerifiedLinuxBundle {
    pub fn manifest(&self) -> &LinuxBundleManifest {
        &self.manifest
    }

    pub fn trusted_key_id(&self) -> &str {
        &self.trusted_key_id
    }
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
    trusted_keyring: &TrustedKeyring,
) -> Result<VerifiedLinuxBundle, LinuxBundleError> {
    let manifest_path = bundle_directory.join("manifest.json");
    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES)?;
    let manifest_text = std::str::from_utf8(&manifest_bytes)
        .map_err(|_| LinuxBundleError::InvalidManifest("manifest must be UTF-8 JSON".to_owned()))?;
    let manifest: LinuxBundleManifest = deserialize_strict_json(manifest_text)
        .map_err(|_| LinuxBundleError::InvalidManifest("strict JSON parsing failed".to_owned()))?;

    if manifest.schema_version != 1
        || manifest.product != EXPECTED_PRODUCT
        || manifest.version.is_empty()
    {
        return Err(LinuxBundleError::InvalidManifest(
            "schema_version, product, or version is unsupported".to_owned(),
        ));
    }
    if manifest.platform != EXPECTED_PLATFORM {
        return Err(LinuxBundleError::UnsupportedPlatform {
            actual: manifest.platform,
        });
    }
    if !is_strict_https_reference(&manifest.attestation_reference) {
        return Err(LinuxBundleError::InvalidAttestationReference);
    }
    if manifest.pi_version != expected_pi.version || manifest.pi_integrity != expected_pi.integrity
    {
        return Err(LinuxBundleError::PiCompatibilityMismatch);
    }
    validate_manifest_file_layout(&manifest)?;

    let artifact_path = checked_child_path(bundle_directory, &manifest.artifact_file)?;
    let statement_path =
        checked_child_path(bundle_directory, &manifest.attestation_statement_file)?;
    let signature_path =
        checked_child_path(bundle_directory, &manifest.attestation_signature_file)?;
    reject_vendored_runtime_payloads(bundle_directory)?;
    let mut artifact_file = open_regular_file(&artifact_path)?;
    if sha256_digest_reader(&mut artifact_file)? != manifest.artifact_sha256 {
        return Err(LinuxBundleError::ArtifactDigestMismatch);
    }

    let statement_bytes = read_bounded_file(&statement_path, MAX_ATTESTATION_STATEMENT_BYTES)?;
    let statement_text = std::str::from_utf8(&statement_bytes)
        .map_err(|_| LinuxBundleError::MalformedAttestation("statement must be UTF-8 JSON"))?;
    let statement: LinuxBundleAttestationStatement = deserialize_strict_json(statement_text)
        .map_err(|_| LinuxBundleError::MalformedAttestation("statement JSON is invalid"))?;
    let canonical_statement = serde_json_canonicalizer::to_vec(&statement)
        .map_err(|_| LinuxBundleError::MalformedAttestation("statement cannot be canonicalized"))?;
    if canonical_statement != statement_bytes {
        return Err(LinuxBundleError::NonCanonicalAttestation);
    }
    validate_statement_binding(&manifest, &statement)?;

    let signature_bytes = read_bounded_file(&signature_path, MAX_SIGNATURE_ENVELOPE_BYTES)?;
    let signature_text = std::str::from_utf8(&signature_bytes)
        .map_err(|_| LinuxBundleError::MalformedAttestation("signature envelope must be UTF-8"))?;
    let signature_envelope: LinuxBundleSignatureEnvelope = deserialize_strict_json(signature_text)
        .map_err(|_| {
            LinuxBundleError::MalformedAttestation("signature envelope JSON is invalid")
        })?;
    if signature_envelope.schema != SIGNATURE_SCHEMA || signature_envelope.schema_version != 1 {
        return Err(LinuxBundleError::UnsupportedAttestation(
            "signature envelope version",
        ));
    }
    if signature_envelope.algorithm != ED25519_ALGORITHM {
        return Err(LinuxBundleError::UnsupportedAttestation(
            "signature algorithm",
        ));
    }
    validate_key_id(&signature_envelope.key_id)?;
    let trusted_key = trusted_keyring
        .keys
        .get(&signature_envelope.key_id)
        .filter(|trusted_key| trusted_key.status == TrustedKeyStatus::Active)
        .ok_or(LinuxBundleError::UnknownOrUntrustedKey)?;
    let signature_bytes = decode_canonical_base64url(&signature_envelope.signature)
        .map_err(|_| LinuxBundleError::MalformedAttestation("signature encoding is invalid"))?;
    let signature_array: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| LinuxBundleError::MalformedAttestation("signature length is invalid"))?;
    let signature = Signature::from_bytes(&signature_array);
    trusted_key
        .verifying_key
        .verify_strict(&statement_bytes, &signature)
        .map_err(|_| LinuxBundleError::SignatureMismatch)?;

    Ok(VerifiedLinuxBundle {
        manifest,
        trusted_key_id: signature_envelope.key_id,
    })
}

/// Verify a local bundle and bind it to an inspected release version before
/// any deployment, lease, service, or user data state can be touched.
pub fn verify_linux_bundle_for_release(
    bundle_directory: &Path,
    expected_release_version: &str,
    expected_pi: &ExpectedPiCompatibility,
    trusted_keyring: &TrustedKeyring,
) -> Result<VerifiedLinuxBundle, LinuxBundleError> {
    let verified_bundle = verify_linux_bundle(bundle_directory, expected_pi, trusted_keyring)?;
    if verified_bundle.manifest().version != expected_release_version {
        return Err(LinuxBundleError::InvalidManifest(
            "bundle version does not match the inspected release".to_owned(),
        ));
    }
    Ok(verified_bundle)
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

    /// Read the active pointer only when it names an existing, safe version
    /// directory. Service rollback must never turn pointer text into a path
    /// without this validation.
    pub(crate) fn validated_active_version(&self) -> Result<Option<String>, LinuxBundleError> {
        let Some(version) = self.active_version()? else {
            return Ok(None);
        };
        let version_directory = safe_version_directory(&version)?;
        if !is_real_directory(&self.root.join("versions").join(version_directory))? {
            return Err(LinuxBundleError::InvalidManifest(
                "active version does not name an installed version".to_owned(),
            ));
        }
        Ok(Some(version))
    }

    /// Extracts only a verified archive to a version-specific staging location.
    /// It never changes the active pointer, so process interruption after this
    /// method leaves the prior version active.
    pub fn stage_verified_bundle(
        &self,
        bundle_directory: &Path,
        verified_bundle: &VerifiedLinuxBundle,
    ) -> Result<PathBuf, LinuxBundleError> {
        let manifest = verified_bundle.manifest();
        let version_directory = safe_version_directory(&manifest.version)?;
        let source_artifact = checked_child_path(bundle_directory, &manifest.artifact_file)?;
        let mut source_artifact_file = open_regular_file(&source_artifact)?;
        if sha256_digest_reader(&mut source_artifact_file)? != manifest.artifact_sha256 {
            return Err(LinuxBundleError::ArtifactDigestMismatch);
        }
        let staging_directory = self.root.join("staged").join(version_directory);
        let private_staging_directory = self
            .root
            .join("staged")
            .join(format!(".extracting-{version_directory}"));
        if staging_directory.exists() {
            fs::remove_dir_all(&staging_directory)?;
        }
        if private_staging_directory.exists() {
            fs::remove_dir_all(&private_staging_directory)?;
        }
        fs::create_dir(&private_staging_directory)?;
        if let Err(error) =
            extract_verified_archive(&mut source_artifact_file, &private_staging_directory)
        {
            let _ = fs::remove_dir_all(&private_staging_directory);
            return Err(error);
        }
        fs::rename(&private_staging_directory, &staging_directory)?;
        Ok(staging_directory)
    }

    /// Promotes a staged version only after the caller has performed its bounded
    /// health check. A health failure retains both the old active pointer and
    /// the staged candidate for operator inspection or explicit cleanup.
    pub fn activate_after_health_check(
        &self,
        verified_bundle: &VerifiedLinuxBundle,
        health_check: impl FnOnce(&Path) -> bool,
    ) -> Result<(), LinuxBundleError> {
        let manifest = verified_bundle.manifest();
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

        self.activate_staged_bundle(verified_bundle)
    }

    /// Promotes a candidate after its caller-owned health result has already
    /// succeeded. Keeping this step separate lets the installer place a
    /// deterministic interruption boundary between health and activation.
    pub(crate) fn activate_staged_bundle(
        &self,
        verified_bundle: &VerifiedLinuxBundle,
    ) -> Result<(), LinuxBundleError> {
        self.publish_staged_bundle(verified_bundle)?;
        self.activate_published_version(&verified_bundle.manifest().version)
    }

    /// Publish verified executable bytes as an immutable version without
    /// changing the active pointer. The single-service installer uses this
    /// boundary to render and health-check the canonical service before the
    /// new version becomes the committed active selection.
    pub(crate) fn publish_staged_bundle(
        &self,
        verified_bundle: &VerifiedLinuxBundle,
    ) -> Result<PathBuf, LinuxBundleError> {
        let manifest = verified_bundle.manifest();
        let version_directory = safe_version_directory(&manifest.version)?;
        let staging_directory = self.root.join("staged").join(version_directory);
        if !staging_directory.is_dir() {
            return Err(LinuxBundleError::InvalidManifest(
                "verified staged version is missing".to_owned(),
            ));
        }

        let version_directory_path = self.root.join("versions").join(version_directory);
        match fs::symlink_metadata(&version_directory_path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::rename(&staging_directory, &version_directory_path)?;
            }
            Ok(metadata) if metadata.file_type().is_dir() => {
                if !version_directories_match(&staging_directory, &version_directory_path)? {
                    return Err(LinuxBundleError::InvalidManifest(
                        "published version differs from the verified candidate".to_owned(),
                    ));
                }
                fs::remove_dir_all(&staging_directory)?;
            }
            Ok(_) => {
                return Err(LinuxBundleError::InvalidManifest(
                    "published version is not a real directory".to_owned(),
                ));
            }
            Err(error) => return Err(error.into()),
        }
        Ok(version_directory_path)
    }

    /// Atomically select an already-published immutable version.
    pub(crate) fn activate_published_version(&self, version: &str) -> Result<(), LinuxBundleError> {
        let version_directory = safe_version_directory(version)?;
        if !is_real_directory(&self.root.join("versions").join(version_directory))? {
            return Err(LinuxBundleError::InvalidManifest(
                "active version is not published".to_owned(),
            ));
        }
        self.replace_active_version(version)
    }

    /// Restore a previously validated active version while the caller holds
    /// the installer lifecycle lease.
    pub(crate) fn restore_active_version(&self, version: &str) -> Result<(), LinuxBundleError> {
        let version_directory = safe_version_directory(version)?;
        if !is_real_directory(&self.root.join("versions").join(version_directory))? {
            return Err(LinuxBundleError::InvalidManifest(
                "rollback version is not installed".to_owned(),
            ));
        }
        self.replace_active_version(version)
    }

    /// Remove an active pointer after a failed first install. The candidate
    /// version directory remains inspectable; user data is never removed.
    pub(crate) fn clear_active_version(&self) -> Result<(), LinuxBundleError> {
        match fs::remove_file(self.root.join(ACTIVE_VERSION_FILE)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn replace_active_version(&self, version: &str) -> Result<(), LinuxBundleError> {
        let temporary_path = self.root.join(format!("{ACTIVE_VERSION_FILE}.new"));
        fs::write(&temporary_path, format!("{version}\n"))?;
        fs::rename(temporary_path, self.root.join(ACTIVE_VERSION_FILE))?;
        Ok(())
    }
}

/// Extract the only supported Personal release layout. The caller has already
/// verified the artifact digest and holds the installation lifecycle lease.
fn extract_verified_archive(
    artifact_file: &mut fs::File,
    private_staging_directory: &Path,
) -> Result<(), LinuxBundleError> {
    if artifact_file.metadata()?.len() > MAX_COMPRESSED_ARTIFACT_BYTES {
        return Err(LinuxBundleError::UnsafeArchive);
    }
    artifact_file.seek(SeekFrom::Start(0))?;

    let gzip_decoder = GzDecoder::new(artifact_file);
    let mut archive = TarArchive::new(gzip_decoder);
    let entries = archive
        .entries()
        .map_err(|_| LinuxBundleError::UnsafeArchive)?;
    let mut canonical_paths = BTreeSet::new();
    let mut regular_file_entries = 0_u64;
    let mut directory_entries = 0_u64;
    let mut expanded_bytes = 0_u64;
    let mut kernel_server_was_written = false;
    let mut cognitive_cli_was_written = false;
    let mut extension_entry_was_written = false;

    for entry_result in entries {
        let entry = entry_result.map_err(|_| LinuxBundleError::UnsafeArchive)?;
        let entry_path = entry.path().map_err(|_| LinuxBundleError::UnsafeArchive)?;
        let canonical_path = validate_archive_entry_path(&entry_path)?;
        if !canonical_paths.insert(canonical_path.clone()) {
            return Err(LinuxBundleError::UnsafeArchive);
        }

        if entry.header().entry_type().is_dir() {
            directory_entries = directory_entries.saturating_add(1);
            if directory_entries > MAX_DIRECTORY_ENTRIES
                || !is_allowed_archive_directory(&canonical_path)
            {
                return Err(LinuxBundleError::UnsafeArchive);
            }
            fs::create_dir_all(private_staging_directory.join(canonical_path))?;
            continue;
        }
        if !entry.header().entry_type().is_file() || !is_allowed_archive_file(&canonical_path) {
            return Err(LinuxBundleError::UnsafeArchive);
        }

        let archive_mode = entry
            .header()
            .mode()
            .map_err(|_| LinuxBundleError::UnsafeArchive)?;
        let requires_executable_mode = is_required_executable_path(&canonical_path);
        if archive_mode & FORBIDDEN_EXECUTABLE_MODE != 0
            || (requires_executable_mode && archive_mode & REQUIRED_EXECUTABLE_MODE == 0)
            || (!requires_executable_mode && archive_mode & REQUIRED_EXECUTABLE_MODE != 0)
        {
            return Err(LinuxBundleError::UnsafeArchive);
        }
        let declared_size = entry.size();
        if declared_size > MAX_REGULAR_FILE_BYTES {
            return Err(LinuxBundleError::UnsafeArchive);
        }
        regular_file_entries = regular_file_entries.saturating_add(1);
        if regular_file_entries > MAX_REGULAR_FILE_ENTRIES {
            return Err(LinuxBundleError::UnsafeArchive);
        }
        let remaining_expanded_bytes = MAX_EXPANDED_ARTIFACT_BYTES
            .checked_sub(expanded_bytes)
            .ok_or(LinuxBundleError::UnsafeArchive)?;
        if declared_size > remaining_expanded_bytes {
            return Err(LinuxBundleError::UnsafeArchive);
        }

        let output_path = private_staging_directory.join(&canonical_path);
        let output_parent = output_path
            .parent()
            .ok_or(LinuxBundleError::UnsafeArchive)?;
        fs::create_dir_all(output_parent)?;
        let mut output_file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&output_path)?;
        let copied_bytes = io::copy(
            &mut entry.take(declared_size.saturating_add(1)),
            &mut output_file,
        )?;
        if copied_bytes != declared_size || copied_bytes > MAX_REGULAR_FILE_BYTES {
            return Err(LinuxBundleError::UnsafeArchive);
        }
        expanded_bytes = expanded_bytes
            .checked_add(copied_bytes)
            .filter(|total| *total <= MAX_EXPANDED_ARTIFACT_BYTES)
            .ok_or(LinuxBundleError::UnsafeArchive)?;
        if requires_executable_mode {
            set_executable_permissions(&output_path)?;
        } else {
            set_readonly_data_permissions(&output_path)?;
        }
        kernel_server_was_written |= canonical_path == Path::new(REQUIRED_KERNEL_SERVER_PATH);
        cognitive_cli_was_written |= canonical_path == Path::new(REQUIRED_COGNITIVE_CLI_PATH);
        extension_entry_was_written |= canonical_path == Path::new(REQUIRED_EXTENSION_ENTRY_PATH);
    }

    if !kernel_server_was_written || !cognitive_cli_was_written || !extension_entry_was_written {
        return Err(LinuxBundleError::UnsafeArchive);
    }
    validate_extracted_layout(private_staging_directory)
}

fn validate_archive_entry_path(entry_path: &Path) -> Result<PathBuf, LinuxBundleError> {
    let entry_path_text = entry_path.to_str().ok_or(LinuxBundleError::UnsafeArchive)?;
    if entry_path_text.is_empty()
        || entry_path_text.len() > MAX_ARCHIVE_PATH_BYTES
        || entry_path_text.contains('\\')
        || entry_path_text.starts_with('/')
        || entry_path_text.contains(":/")
        || entry_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(LinuxBundleError::UnsafeArchive);
    }
    if !is_allowed_archive_directory(&PathBuf::from(entry_path_text))
        && !is_allowed_archive_file(&PathBuf::from(entry_path_text))
    {
        return Err(LinuxBundleError::UnsafeArchive);
    }
    Ok(PathBuf::from(entry_path_text))
}

fn validate_extracted_layout(staging_directory: &Path) -> Result<(), LinuxBundleError> {
    for required_executable_path in [REQUIRED_KERNEL_SERVER_PATH, REQUIRED_COGNITIVE_CLI_PATH] {
        validate_regular_file(staging_directory.join(required_executable_path))?;
    }
    validate_regular_file(staging_directory.join(REQUIRED_EXTENSION_ENTRY_PATH))?;
    validate_extension_directory(&staging_directory.join(REQUIRED_EXTENSION_ROOT_PATH))?;
    Ok(())
}

fn version_directories_match(
    staged_directory: &Path,
    published_directory: &Path,
) -> Result<bool, LinuxBundleError> {
    validate_extracted_layout(staged_directory)?;
    validate_extracted_layout(published_directory)?;
    Ok(directory_file_digests(staged_directory)? == directory_file_digests(published_directory)?)
}

fn is_allowed_archive_directory(path: &Path) -> bool {
    matches!(
        path.to_str(),
        Some("bin" | "extensions" | "extensions/pi-cognitiveos" | "extensions/pi-cognitiveos/dist")
    ) || path.starts_with(REQUIRED_EXTENSION_ROOT_PATH)
}

fn is_allowed_archive_file(path: &Path) -> bool {
    is_required_executable_path(path) || path.starts_with(REQUIRED_EXTENSION_ROOT_PATH)
}

fn is_required_executable_path(path: &Path) -> bool {
    path == Path::new(REQUIRED_KERNEL_SERVER_PATH) || path == Path::new(REQUIRED_COGNITIVE_CLI_PATH)
}

fn validate_regular_file(path: PathBuf) -> Result<(), LinuxBundleError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| LinuxBundleError::UnsafeArchive)?;
    if metadata.file_type().is_file() {
        Ok(())
    } else {
        Err(LinuxBundleError::UnsafeArchive)
    }
}

fn validate_extension_directory(path: &Path) -> Result<(), LinuxBundleError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| LinuxBundleError::UnsafeArchive)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(LinuxBundleError::UnsafeArchive);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink()
            || (!metadata.file_type().is_file() && !metadata.file_type().is_dir())
        {
            return Err(LinuxBundleError::UnsafeArchive);
        }
        if metadata.file_type().is_dir() {
            validate_extension_directory(&entry.path())?;
        }
    }
    Ok(())
}

fn directory_file_digests(directory: &Path) -> Result<BTreeMap<PathBuf, String>, LinuxBundleError> {
    let mut digests = BTreeMap::new();
    collect_directory_file_digests(directory, Path::new(""), &mut digests)?;
    Ok(digests)
}

fn collect_directory_file_digests(
    directory: &Path,
    relative_directory: &Path,
    digests: &mut BTreeMap<PathBuf, String>,
) -> Result<(), LinuxBundleError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let relative_path = relative_directory.join(entry.file_name());
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(LinuxBundleError::UnsafeArchive);
        }
        if metadata.file_type().is_dir() {
            collect_directory_file_digests(&entry.path(), &relative_path, digests)?;
        } else if metadata.file_type().is_file() {
            let mut file = open_regular_file(&entry.path())?;
            digests.insert(relative_path, sha256_digest_reader(&mut file)?);
        } else {
            return Err(LinuxBundleError::UnsafeArchive);
        }
    }
    Ok(())
}

fn is_real_directory(path: &Path) -> Result<bool, LinuxBundleError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_dir() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), LinuxBundleError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(unix)]
fn set_readonly_data_permissions(path: &Path) -> Result<(), LinuxBundleError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o644))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_readonly_data_permissions(_path: &Path) -> Result<(), LinuxBundleError> {
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), LinuxBundleError> {
    Ok(())
}

fn checked_child_path(root: &Path, child: &str) -> Result<PathBuf, LinuxBundleError> {
    let child_path = Path::new(child);
    if child_path.as_os_str().is_empty()
        || child_path.is_absolute()
        || child.contains('/')
        || child.contains('\\')
        || child == "."
        || child == ".."
        || child_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
                    | Component::CurDir
            )
        })
    {
        return Err(LinuxBundleError::UnsafePath(child.to_owned()));
    }
    Ok(root.join(child_path))
}

fn deserialize_strict_json<'de, Value>(input: &'de str) -> Result<Value, serde_json::Error>
where
    Value: Deserialize<'de>,
{
    // The closed derived structs use deny_unknown_fields and serde's generated
    // duplicate-field checks, so a single typed parse rejects both classes.
    serde_json::from_str(input)
}

fn read_bounded_file(path: &Path, maximum_bytes: usize) -> io::Result<Vec<u8>> {
    ensure_regular_file(path)?;
    let file = fs::File::open(path)?;
    let mut bounded_reader = file.take(maximum_bytes as u64 + 1);
    let mut bytes = Vec::with_capacity(maximum_bytes.min(16 * 1024));
    bounded_reader.read_to_end(&mut bytes)?;
    if bytes.len() > maximum_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Linux bundle metadata exceeds its size limit",
        ));
    }
    Ok(bytes)
}

fn open_regular_file(path: &Path) -> io::Result<fs::File> {
    ensure_regular_file(path)?;
    fs::File::open(path)
}

fn ensure_regular_file(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Linux bundle input must be a regular file",
        ));
    }
    Ok(())
}

fn validate_statement_binding(
    manifest: &LinuxBundleManifest,
    statement: &LinuxBundleAttestationStatement,
) -> Result<(), LinuxBundleError> {
    if statement.schema != ATTESTATION_SCHEMA || statement.schema_version != 1 {
        return Err(LinuxBundleError::UnsupportedAttestation(
            "statement schema version",
        ));
    }
    let statement_matches_manifest = statement.product == manifest.product
        && statement.platform == manifest.platform
        && statement.version == manifest.version
        && statement.artifact_file == manifest.artifact_file
        && statement.artifact_sha256 == manifest.artifact_sha256
        && statement.pi_version == manifest.pi_version
        && statement.pi_integrity == manifest.pi_integrity
        && statement.provenance_reference == manifest.attestation_reference;
    if !statement_matches_manifest {
        return Err(LinuxBundleError::StatementBindingMismatch);
    }
    Ok(())
}

fn validate_manifest_file_layout(manifest: &LinuxBundleManifest) -> Result<(), LinuxBundleError> {
    let file_names = [
        manifest.artifact_file.as_str(),
        manifest.attestation_statement_file.as_str(),
        manifest.attestation_signature_file.as_str(),
    ];
    if file_names.contains(&"manifest.json")
        || file_names[0] == file_names[1]
        || file_names[0] == file_names[2]
        || file_names[1] == file_names[2]
    {
        return Err(LinuxBundleError::InvalidManifest(
            "bundle payload files must be distinct and cannot replace manifest.json".to_owned(),
        ));
    }
    Ok(())
}

fn validate_key_id(key_id: &str) -> Result<(), LinuxBundleError> {
    let key_id_is_valid = !key_id.is_empty()
        && key_id.len() <= 128
        && key_id.is_ascii()
        && key_id.bytes().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, b'.' | b'_' | b'-')
        });
    if !key_id_is_valid {
        return Err(LinuxBundleError::MalformedAttestation(
            "trusted key ID is invalid",
        ));
    }
    Ok(())
}

fn decode_canonical_base64url(encoded: &str) -> Result<Vec<u8>, ()> {
    let decoded = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| ())?;
    if URL_SAFE_NO_PAD.encode(&decoded) != encoded {
        return Err(());
    }
    Ok(decoded)
}

fn is_strict_https_reference(reference: &str) -> bool {
    let Ok(parsed_url) = url::Url::parse(reference) else {
        return false;
    };
    parsed_url.scheme() == "https"
        && parsed_url.host_str().is_some_and(|host| !host.is_empty())
        && parsed_url.username().is_empty()
        && parsed_url.password().is_none()
        && !reference.chars().any(char::is_control)
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

#[cfg(test)]
fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn sha256_digest_reader(reader: &mut impl Read) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use flate2::{Compression, write::GzEncoder};
    use tar::{Builder as TarBuilder, Header as TarHeader};

    const PI_VERSION: &str = "0.81.1";
    const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
    const TEST_ONLY_KEY_ID: &str = "linux-bundle-unit-test-key";

    fn expected_pi() -> ExpectedPiCompatibility {
        ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY)
    }

    fn test_only_signing_key() -> SigningKey {
        SigningKey::from_bytes(&[0x4d; 32])
    }

    fn test_only_keyring() -> TrustedKeyring {
        let signing_key = test_only_signing_key();
        TrustedKeyring::new(
            "linux-bundle-unit-test-keyring-v1",
            vec![TrustedKeyInput {
                key_id: TEST_ONLY_KEY_ID.to_owned(),
                algorithm: ED25519_ALGORITHM.to_owned(),
                public_key_base64url: URL_SAFE_NO_PAD
                    .encode(signing_key.verifying_key().to_bytes()),
                status: TrustedKeyStatus::Active,
            }],
        )
        .unwrap()
    }

    fn write_bundle(directory: &Path, version: &str) -> LinuxBundleManifest {
        let artifact = runnable_archive_bytes();
        let manifest = LinuxBundleManifest {
            schema_version: 1,
            product: EXPECTED_PRODUCT.to_owned(),
            platform: EXPECTED_PLATFORM.to_owned(),
            version: version.to_owned(),
            artifact_file: "cognitiveos-linux-x86_64.tar.gz".to_owned(),
            artifact_sha256: sha256_digest(&artifact),
            attestation_reference: "https://example.invalid/attestations/v1".to_owned(),
            attestation_statement_file: "attestation.statement.json".to_owned(),
            attestation_signature_file: "attestation.signature.json".to_owned(),
            pi_version: PI_VERSION.to_owned(),
            pi_integrity: PI_INTEGRITY.to_owned(),
        };
        fs::write(directory.join(&manifest.artifact_file), artifact).unwrap();
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let statement = LinuxBundleAttestationStatement {
            schema: ATTESTATION_SCHEMA.to_owned(),
            schema_version: 1,
            product: manifest.product.clone(),
            platform: manifest.platform.clone(),
            version: manifest.version.clone(),
            artifact_file: manifest.artifact_file.clone(),
            artifact_sha256: manifest.artifact_sha256.clone(),
            pi_version: manifest.pi_version.clone(),
            pi_integrity: manifest.pi_integrity.clone(),
            provenance_reference: manifest.attestation_reference.clone(),
        };
        let statement_bytes = serde_json_canonicalizer::to_vec(&statement).unwrap();
        let signature = test_only_signing_key().sign(&statement_bytes);
        fs::write(
            directory.join(&manifest.attestation_statement_file),
            statement_bytes,
        )
        .unwrap();
        fs::write(
            directory.join(&manifest.attestation_signature_file),
            serde_json::to_vec(&serde_json::json!({
                "algorithm": ED25519_ALGORITHM,
                "key_id": TEST_ONLY_KEY_ID,
                "schema": SIGNATURE_SCHEMA,
                "schema_version": 1,
                "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
            }))
            .unwrap(),
        )
        .unwrap();
        manifest
    }

    fn runnable_archive_bytes() -> Vec<u8> {
        let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut tar_builder = TarBuilder::new(gzip_encoder);
        append_test_archive_file(
            &mut tar_builder,
            REQUIRED_KERNEL_SERVER_PATH,
            b"unit-test-kernel-server",
            0o755,
        );
        append_test_archive_file(
            &mut tar_builder,
            REQUIRED_COGNITIVE_CLI_PATH,
            b"unit-test-cognitive",
            0o755,
        );
        append_test_archive_file(
            &mut tar_builder,
            REQUIRED_EXTENSION_ENTRY_PATH,
            b"export {};\n",
            0o644,
        );
        let gzip_encoder = tar_builder.into_inner().unwrap();
        gzip_encoder.finish().unwrap()
    }

    fn append_test_archive_file(
        tar_builder: &mut TarBuilder<GzEncoder<Vec<u8>>>,
        path: &str,
        contents: &[u8],
        mode: u32,
    ) {
        let mut header = TarHeader::new_gnu();
        header.set_mode(mode);
        header.set_size(contents.len() as u64);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, path, contents)
            .unwrap();
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
            verify_linux_bundle(
                temporary_directory.path(),
                &expected_pi(),
                &test_only_keyring(),
            ),
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
            verify_linux_bundle(
                temporary_directory.path(),
                &expected_pi(),
                &test_only_keyring(),
            ),
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
            verify_linux_bundle(
                temporary_directory.path(),
                &expected_pi(),
                &test_only_keyring(),
            ),
            Err(LinuxBundleError::PiCompatibilityMismatch)
        ));
    }

    #[test]
    fn rejects_vendored_node_or_pi_payloads() {
        let temporary_directory = tempfile::tempdir().unwrap();
        write_bundle(temporary_directory.path(), "1.0.0");
        fs::write(temporary_directory.path().join("node"), b"forbidden").unwrap();
        assert!(matches!(
            verify_linux_bundle(
                temporary_directory.path(),
                &expected_pi(),
                &test_only_keyring(),
            ),
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
        write_bundle(&bundle_directory, "2.0.0");
        let verified_bundle =
            verify_linux_bundle(&bundle_directory, &expected_pi(), &test_only_keyring()).unwrap();
        deployment
            .stage_verified_bundle(&bundle_directory, &verified_bundle)
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
            deployment.activate_after_health_check(&verified_bundle, |_| false),
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
        fs::create_dir_all(temporary_directory.path().join("personal/deploy/versions/1.0.0")).unwrap();
        let bundle_directory = temporary_directory.path().join("bundle");
        fs::create_dir(&bundle_directory).unwrap();
        write_bundle(&bundle_directory, "2.0.0");
        let verified_bundle =
            verify_linux_bundle(&bundle_directory, &expected_pi(), &test_only_keyring()).unwrap();
        deployment
            .stage_verified_bundle(&bundle_directory, &verified_bundle)
            .unwrap();

        deployment
            .activate_after_health_check(&verified_bundle, |staged_directory| {
                staged_directory.join(REQUIRED_KERNEL_SERVER_PATH).is_file()
            })
            .unwrap();

        assert_eq!(
            deployment.active_version().unwrap().as_deref(),
            Some("2.0.0")
        );
        assert!(
            temporary_directory
                .path()
                .join("personal/deploy/versions/1.0.0")
                .is_dir()
        );
        assert!(
            temporary_directory
                .path()
                .join("personal/deploy/versions/2.0.0")
                .is_dir()
        );
    }
}
