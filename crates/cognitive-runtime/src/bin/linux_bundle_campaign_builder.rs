//! Builds a non-production, release-shaped Linux installer campaign artifact.
//!
//! This tool is deliberately separate from production release automation. It
//! accepts only a campaign signing seed file, never writes that seed into the
//! output, and emits a directory that the existing offline verifier validates
//! before publication. It does not upload, serve, or install the artifact.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::{
    ExpectedPiCompatibility, LinuxBundleManifest, TrustedKeyInput, TrustedKeyStatus,
    TrustedKeyring, verify_linux_bundle_for_release,
};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Builder as TarBuilder, Header as TarHeader};
use url::Url;

const INSTALLER_FILENAME: &str = "cognitiveos-linux-bundle-installer";
const ARTIFACT_FILENAME: &str = "cognitiveos-linux-x86_64.tar.gz";
const MANIFEST_FILENAME: &str = "manifest.json";
const STATEMENT_FILENAME: &str = "attestation.statement.json";
const SIGNATURE_FILENAME: &str = "attestation.signature.json";
const BOOTSTRAP_FILENAME: &str = "install.sh";
const MAX_INSTALLER_BYTES: u64 = 32 * 1024 * 1024;
const MAX_KERNEL_SERVER_BYTES: u64 = 512 * 1024 * 1024;
const BOOTSTRAP_TEMPLATE: &str = include_str!("../../../../deploy/linux/install.sh");

#[derive(Default)]
struct CampaignArguments {
    kernel_server_binary: Option<PathBuf>,
    installer_binary: Option<PathBuf>,
    campaign_signing_seed_file: Option<PathBuf>,
    output_directory: Option<PathBuf>,
    release_version: Option<String>,
    release_object_directory: Option<String>,
    allowed_redirect_host: Option<String>,
    keyring_version: Option<String>,
    key_id: Option<String>,
    expected_pi_version: Option<String>,
    expected_pi_integrity: Option<String>,
}

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if build_campaign_release(&arguments).is_err() {
        // Paths, signing material, and input errors are intentionally not
        // surfaced through this command's stable public boundary.
        eprintln!("CognitiveOS campaign release build failed");
        std::process::exit(1);
    }
}

fn build_campaign_release(arguments: &[String]) -> Result<(), CampaignBuildError> {
    let parsed_arguments = parse_arguments(arguments).map_err(|_| CampaignBuildError::Arguments)?;
    let kernel_server_binary = parsed_arguments
        .kernel_server_binary
        .ok_or(CampaignBuildError::Arguments)?;
    let installer_binary = parsed_arguments
        .installer_binary
        .ok_or(CampaignBuildError::Arguments)?;
    let campaign_signing_seed_file = parsed_arguments
        .campaign_signing_seed_file
        .ok_or(CampaignBuildError::Arguments)?;
    let output_directory = parsed_arguments
        .output_directory
        .ok_or(CampaignBuildError::Arguments)?;
    let release_version = parsed_arguments
        .release_version
        .ok_or(CampaignBuildError::Arguments)?;
    let release_object_directory = parsed_arguments
        .release_object_directory
        .ok_or(CampaignBuildError::Arguments)?;
    let allowed_redirect_host = parsed_arguments
        .allowed_redirect_host
        .ok_or(CampaignBuildError::Arguments)?;
    let keyring_version = parsed_arguments
        .keyring_version
        .ok_or(CampaignBuildError::Arguments)?;
    let key_id = parsed_arguments
        .key_id
        .ok_or(CampaignBuildError::Arguments)?;
    let expected_pi_version = parsed_arguments
        .expected_pi_version
        .ok_or(CampaignBuildError::Arguments)?;
    let expected_pi_integrity = parsed_arguments
        .expected_pi_integrity
        .ok_or(CampaignBuildError::Arguments)?;

    validate_campaign_arguments(
        &release_version,
        &release_object_directory,
        &allowed_redirect_host,
        &keyring_version,
        &key_id,
        &expected_pi_version,
        &expected_pi_integrity,
    )
    .map_err(|_| CampaignBuildError::Arguments)?;
    reject_existing_output_directory(&output_directory).map_err(|_| CampaignBuildError::Output)?;
    validate_input_binary(&kernel_server_binary, MAX_KERNEL_SERVER_BYTES)
        .map_err(|_| CampaignBuildError::KernelServer)?;
    validate_input_binary(&installer_binary, MAX_INSTALLER_BYTES)
        .map_err(|_| CampaignBuildError::Installer)?;
    let signing_key = read_campaign_signing_key(&campaign_signing_seed_file)
        .map_err(|_| CampaignBuildError::SigningKey)?;

    let output_parent = output_directory
        .parent()
        .ok_or(CampaignBuildError::Output)?;
    if !output_parent.is_dir() {
        return Err(CampaignBuildError::Output);
    }
    let temporary_output_directory = output_parent.join(format!(
        ".cognitiveos-campaign-release-{}",
        std::process::id()
    ));
    if temporary_output_directory.exists() {
        return Err(CampaignBuildError::Output);
    }
    fs::create_dir(&temporary_output_directory).map_err(|_| CampaignBuildError::Output)?;

    let result = write_campaign_release(
        &temporary_output_directory,
        &kernel_server_binary,
        &installer_binary,
        &signing_key,
        &release_version,
        &release_object_directory,
        &allowed_redirect_host,
        &keyring_version,
        &key_id,
        &expected_pi_version,
        &expected_pi_integrity,
    );
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary_output_directory);
        return Err(CampaignBuildError::ReleaseContents(error));
    }
    fs::rename(&temporary_output_directory, &output_directory).map_err(|_| {
        let _ = fs::remove_dir_all(&temporary_output_directory);
        CampaignBuildError::Output
    })?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
enum CampaignBuildError {
    Arguments,
    KernelServer,
    Installer,
    SigningKey,
    Output,
    ReleaseContents(CampaignReleaseError),
}

#[allow(dead_code)]
#[derive(Debug)]
enum CampaignReleaseError {
    Archive,
    CanonicalStatement,
    ReadInstaller,
    RenderBootstrap(&'static str),
    WriteOutput,
    TrustedKeyring,
    Verification(cognitive_runtime::LinuxBundleError),
}

#[allow(clippy::too_many_arguments)]
fn write_campaign_release(
    output_directory: &Path,
    kernel_server_binary: &Path,
    installer_binary: &Path,
    signing_key: &SigningKey,
    release_version: &str,
    release_object_directory: &str,
    allowed_redirect_host: &str,
    keyring_version: &str,
    key_id: &str,
    expected_pi_version: &str,
    expected_pi_integrity: &str,
) -> Result<(), CampaignReleaseError> {
    let artifact =
        archive_kernel_server(kernel_server_binary).map_err(|_| CampaignReleaseError::Archive)?;
    let artifact_sha256 = sha256_digest(&artifact);
    let manifest = LinuxBundleManifest {
        schema_version: 1,
        product: "cognitiveos-personal".to_owned(),
        platform: "linux-x86_64".to_owned(),
        version: release_version.to_owned(),
        artifact_file: ARTIFACT_FILENAME.to_owned(),
        artifact_sha256,
        attestation_reference: format!("{release_object_directory}/provenance/{release_version}"),
        attestation_statement_file: STATEMENT_FILENAME.to_owned(),
        attestation_signature_file: SIGNATURE_FILENAME.to_owned(),
        pi_version: expected_pi_version.to_owned(),
        pi_integrity: expected_pi_integrity.to_owned(),
    };
    let statement = json!({
        "artifact_file": manifest.artifact_file,
        "artifact_sha256": manifest.artifact_sha256,
        "pi_integrity": manifest.pi_integrity,
        "pi_version": manifest.pi_version,
        "platform": manifest.platform,
        "product": manifest.product,
        "provenance_reference": manifest.attestation_reference,
        "schema": "cognitiveos.personal.linux-bundle-attestation",
        "schema_version": 1,
        "version": manifest.version,
    });
    let statement_bytes = serde_json_canonicalizer::to_vec(&statement)
        .map_err(|_| CampaignReleaseError::CanonicalStatement)?;
    let signature = signing_key.sign(&statement_bytes);
    let public_key_base64url = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
    let signature_envelope = json!({
        "algorithm": "Ed25519",
        "key_id": key_id,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    });
    let installer_bytes =
        fs::read(installer_binary).map_err(|_| CampaignReleaseError::ReadInstaller)?;
    let rendered_bootstrap = render_bootstrap(
        release_version,
        release_object_directory,
        allowed_redirect_host,
        &sha256_digest(&installer_bytes),
        keyring_version,
        key_id,
        &public_key_base64url,
        expected_pi_version,
        expected_pi_integrity,
    )
    .map_err(CampaignReleaseError::RenderBootstrap)?;

    write_private_file(
        output_directory.join(INSTALLER_FILENAME),
        &installer_bytes,
        true,
    )
    .map_err(|_| CampaignReleaseError::WriteOutput)?;
    write_private_file(output_directory.join(ARTIFACT_FILENAME), &artifact, false)
        .map_err(|_| CampaignReleaseError::WriteOutput)?;
    write_private_file(
        output_directory.join(MANIFEST_FILENAME),
        &serde_json::to_vec(&manifest).map_err(|_| CampaignReleaseError::WriteOutput)?,
        false,
    )
    .map_err(|_| CampaignReleaseError::WriteOutput)?;
    write_private_file(
        output_directory.join(STATEMENT_FILENAME),
        &statement_bytes,
        false,
    )
    .map_err(|_| CampaignReleaseError::WriteOutput)?;
    write_private_file(
        output_directory.join(SIGNATURE_FILENAME),
        &serde_json::to_vec(&signature_envelope).map_err(|_| CampaignReleaseError::WriteOutput)?,
        false,
    )
    .map_err(|_| CampaignReleaseError::WriteOutput)?;
    write_private_file(
        output_directory.join(BOOTSTRAP_FILENAME),
        rendered_bootstrap.as_bytes(),
        true,
    )
    .map_err(|_| CampaignReleaseError::WriteOutput)?;

    let trusted_keyring = TrustedKeyring::new(
        keyring_version,
        vec![TrustedKeyInput {
            key_id: key_id.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url,
            status: TrustedKeyStatus::Active,
        }],
    )
    .map_err(|_| CampaignReleaseError::TrustedKeyring)?;
    verify_linux_bundle_for_release(
        output_directory,
        release_version,
        &ExpectedPiCompatibility::new(expected_pi_version, expected_pi_integrity),
        &trusted_keyring,
    )
    .map_err(CampaignReleaseError::Verification)?;
    Ok(())
}

fn archive_kernel_server(kernel_server_binary: &Path) -> Result<Vec<u8>, ()> {
    let binary_bytes = fs::read(kernel_server_binary).map_err(|_| ())?;
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_mode(0o755);
    header.set_size(binary_bytes.len().try_into().map_err(|_| ())?);
    header.set_cksum();
    tar_builder
        .append_data(&mut header, "bin/kernel-server", binary_bytes.as_slice())
        .map_err(|_| ())?;
    tar_builder
        .into_inner()
        .map_err(|_| ())?
        .finish()
        .map_err(|_| ())
}

#[allow(clippy::too_many_arguments)]
fn render_bootstrap(
    release_version: &str,
    release_object_directory: &str,
    allowed_redirect_host: &str,
    installer_sha256: &str,
    keyring_version: &str,
    key_id: &str,
    public_key_base64url: &str,
    expected_pi_version: &str,
    expected_pi_integrity: &str,
) -> Result<String, &'static str> {
    let replacements = [
        ("@COGNITIVEOS_RELEASE_VERSION@", release_version),
        (
            "@COGNITIVEOS_RELEASE_OBJECT_DIRECTORY@",
            release_object_directory,
        ),
        ("@COGNITIVEOS_ALLOWED_REDIRECT_HOST@", allowed_redirect_host),
        ("@COGNITIVEOS_INSTALLER_SHA256@", installer_sha256),
        ("@COGNITIVEOS_TRUSTED_KEYRING_VERSION@", keyring_version),
        ("@COGNITIVEOS_TRUSTED_KEY_ID@", key_id),
        (
            "@COGNITIVEOS_TRUSTED_PUBLIC_KEY_BASE64URL@",
            public_key_base64url,
        ),
        ("@COGNITIVEOS_EXPECTED_PI_VERSION@", expected_pi_version),
        ("@COGNITIVEOS_EXPECTED_PI_INTEGRITY@", expected_pi_integrity),
    ];
    let mut rendered_bootstrap = BOOTSTRAP_TEMPLATE.to_owned();
    for (placeholder, replacement) in replacements {
        if rendered_bootstrap.matches(placeholder).count() != 1 {
            return Err(placeholder);
        }
        rendered_bootstrap = rendered_bootstrap.replacen(placeholder, replacement, 1);
    }
    // The bootstrap deliberately retains a shell pattern that recognizes an
    // unrendered value at runtime. Only a policy assignment is evidence that
    // release rendering was incomplete.
    if rendered_bootstrap.contains("=\"@COGNITIVEOS_") {
        return Err("unrendered policy assignment");
    }
    Ok(rendered_bootstrap)
}

fn validate_campaign_arguments(
    release_version: &str,
    release_object_directory: &str,
    allowed_redirect_host: &str,
    keyring_version: &str,
    key_id: &str,
    expected_pi_version: &str,
    expected_pi_integrity: &str,
) -> Result<(), ()> {
    validate_token(release_version)?;
    validate_token(keyring_version)?;
    validate_token(key_id)?;
    validate_ascii_value(expected_pi_version)?;
    validate_ascii_value(expected_pi_integrity)?;
    if allowed_redirect_host.is_empty()
        || allowed_redirect_host.len() > 253
        || allowed_redirect_host.starts_with('.')
        || allowed_redirect_host.contains("..")
        || !allowed_redirect_host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return Err(());
    }
    let release_url = Url::parse(release_object_directory).map_err(|_| ())?;
    if release_url.scheme() != "https"
        || release_url.host_str().is_none()
        || !release_url.username().is_empty()
        || release_url.password().is_some()
        || release_url.query().is_some()
        || release_url.fragment().is_some()
        || release_url.path().is_empty()
        || release_url.path() == "/"
    {
        return Err(());
    }
    if release_url.host_str() != Some(allowed_redirect_host) {
        return Err(());
    }
    Ok(())
}

fn validate_token(value: &str) -> Result<(), ()> {
    (!value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')))
    .then_some(())
    .ok_or(())
}

fn validate_ascii_value(value: &str) -> Result<(), ()> {
    (!value.is_empty()
        && value.len() <= 512
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' '))
    .then_some(())
    .ok_or(())
}

fn reject_existing_output_directory(output_directory: &Path) -> Result<(), ()> {
    (!output_directory.exists()).then_some(()).ok_or(())
}

fn validate_input_binary(binary_path: &Path, maximum_bytes: u64) -> Result<(), ()> {
    let metadata = fs::metadata(binary_path).map_err(|_| ())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum_bytes {
        return Err(());
    }
    let mut binary_file = File::open(binary_path).map_err(|_| ())?;
    let mut elf_magic = [0_u8; 4];
    binary_file.read_exact(&mut elf_magic).map_err(|_| ())?;
    (elf_magic == [0x7f, b'E', b'L', b'F'])
        .then_some(())
        .ok_or(())
}

fn read_campaign_signing_key(seed_file: &Path) -> Result<SigningKey, ()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::metadata(seed_file).map_err(|_| ())?.permissions();
        if permissions.mode() & 0o077 != 0 {
            return Err(());
        }
    }
    let seed = fs::read(seed_file).map_err(|_| ())?;
    let seed: [u8; 32] = seed.try_into().map_err(|_| ())?;
    Ok(SigningKey::from_bytes(&seed))
}

fn write_private_file(path: PathBuf, contents: &[u8], executable: bool) -> Result<(), ()> {
    let mut output_file = File::create(&path).map_err(|_| ())?;
    output_file.write_all(contents).map_err(|_| ())?;
    output_file.sync_all().map_err(|_| ())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o700 } else { 0o600 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|_| ())?;
    }
    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn parse_arguments(arguments: &[String]) -> Result<CampaignArguments, ()> {
    let mut parsed_arguments = CampaignArguments::default();
    let mut argument_index = 0;
    while argument_index < arguments.len() {
        let flag = &arguments[argument_index];
        let value = arguments.get(argument_index + 1).ok_or(())?.clone();
        argument_index += 2;
        match flag.as_str() {
            "--kernel-server-binary" => {
                set_path_once(&mut parsed_arguments.kernel_server_binary, value)?
            }
            "--installer-binary" => set_path_once(&mut parsed_arguments.installer_binary, value)?,
            "--campaign-signing-seed-file" => {
                set_path_once(&mut parsed_arguments.campaign_signing_seed_file, value)?
            }
            "--output-directory" => set_path_once(&mut parsed_arguments.output_directory, value)?,
            "--release-version" => set_once(&mut parsed_arguments.release_version, value)?,
            "--release-object-directory" => {
                set_once(&mut parsed_arguments.release_object_directory, value)?
            }
            "--allowed-redirect-host" => {
                set_once(&mut parsed_arguments.allowed_redirect_host, value)?
            }
            "--keyring-version" => set_once(&mut parsed_arguments.keyring_version, value)?,
            "--key-id" => set_once(&mut parsed_arguments.key_id, value)?,
            "--expected-pi-version" => set_once(&mut parsed_arguments.expected_pi_version, value)?,
            "--expected-pi-integrity" => {
                set_once(&mut parsed_arguments.expected_pi_integrity, value)?
            }
            _ => return Err(()),
        }
    }
    let all_required_arguments_present = parsed_arguments.kernel_server_binary.is_some()
        && parsed_arguments.installer_binary.is_some()
        && parsed_arguments.campaign_signing_seed_file.is_some()
        && parsed_arguments.output_directory.is_some()
        && parsed_arguments.release_version.is_some()
        && parsed_arguments.release_object_directory.is_some()
        && parsed_arguments.allowed_redirect_host.is_some()
        && parsed_arguments.keyring_version.is_some()
        && parsed_arguments.key_id.is_some()
        && parsed_arguments.expected_pi_version.is_some()
        && parsed_arguments.expected_pi_integrity.is_some();
    all_required_arguments_present
        .then_some(parsed_arguments)
        .ok_or(())
}

fn set_once(target: &mut Option<String>, value: String) -> Result<(), ()> {
    target.replace(value).is_none().then_some(()).ok_or(())
}

fn set_path_once(target: &mut Option<PathBuf>, value: String) -> Result<(), ()> {
    target
        .replace(PathBuf::from(value))
        .is_none()
        .then_some(())
        .ok_or(())
}
