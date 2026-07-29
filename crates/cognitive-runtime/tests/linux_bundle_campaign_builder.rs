//! Contract coverage for non-production native installer campaign artifacts.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::{
    ExpectedPiCompatibility, TrustedKeyInput, TrustedKeyStatus, TrustedKeyring,
    verify_linux_bundle_for_release,
};
use ed25519_dalek::SigningKey;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

const RELEASE_VERSION: &str = "0.0.0-campaign.1";
const KEY_ID: &str = "p1t08-campaign-key";
const KEYRING_VERSION: &str = "p1t08-campaign-keyring-v1";
const PI_VERSION: &str = "0.81.1";
const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";

#[test]
fn campaign_builder_emits_an_offline_verifiable_release_without_unrendered_bootstrap_policy() {
    let fixture_directory = tempfile::tempdir().unwrap();
    let kernel_server_binary = fixture_directory.path().join("kernel-server");
    let installer_binary = fixture_directory.path().join("linux-bundle-installer");
    let signing_seed_file = fixture_directory.path().join("campaign-signing-seed");
    let output_directory = fixture_directory.path().join("release");
    write_elf_fixture(&kernel_server_binary);
    write_elf_fixture(&installer_binary);
    fs::write(&signing_seed_file, [0x62; 32]).unwrap();
    #[cfg(unix)]
    fs::set_permissions(&signing_seed_file, std::fs::Permissions::from_mode(0o600)).unwrap();

    let output = run_campaign_builder(
        &kernel_server_binary,
        &installer_binary,
        &signing_seed_file,
        &output_directory,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read(output_directory.join("cognitiveos-linux-bundle-installer")).unwrap(),
        fs::read(&installer_binary).unwrap()
    );
    verify_linux_bundle_for_release(
        &output_directory,
        RELEASE_VERSION,
        &ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY),
        &trusted_campaign_keyring(),
    )
    .unwrap();

    let rendered_bootstrap = fs::read_to_string(output_directory.join("install.sh")).unwrap();
    assert!(!rendered_bootstrap.contains("=\"@COGNITIVEOS_"));
    assert!(rendered_bootstrap.contains("https://release.example.test/campaign"));
    assert!(rendered_bootstrap.contains("--expected-release-version \"$RELEASE_VERSION\""));

    let overwrite_attempt = run_campaign_builder(
        &kernel_server_binary,
        &installer_binary,
        &signing_seed_file,
        &output_directory,
    );
    assert!(!overwrite_attempt.status.success());
    assert_eq!(
        stderr(&overwrite_attempt),
        "CognitiveOS campaign release build failed\n"
    );
}

fn run_campaign_builder(
    kernel_server_binary: &Path,
    installer_binary: &Path,
    signing_seed_file: &Path,
    output_directory: &Path,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_linux_bundle_campaign_builder"))
        .args([
            "--kernel-server-binary",
            kernel_server_binary.to_str().unwrap(),
            "--installer-binary",
            installer_binary.to_str().unwrap(),
            "--campaign-signing-seed-file",
            signing_seed_file.to_str().unwrap(),
            "--output-directory",
            output_directory.to_str().unwrap(),
            "--release-version",
            RELEASE_VERSION,
            "--release-object-directory",
            "https://release.example.test/campaign",
            "--allowed-redirect-host",
            "release.example.test",
            "--keyring-version",
            KEYRING_VERSION,
            "--key-id",
            KEY_ID,
            "--expected-pi-version",
            PI_VERSION,
            "--expected-pi-integrity",
            PI_INTEGRITY,
        ])
        .output()
        .unwrap()
}

fn trusted_campaign_keyring() -> TrustedKeyring {
    let signing_key = SigningKey::from_bytes(&[0x62; 32]);
    TrustedKeyring::new(
        KEYRING_VERSION,
        vec![TrustedKeyInput {
            key_id: KEY_ID.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Active,
        }],
    )
    .unwrap()
}

fn write_elf_fixture(path: &Path) {
    fs::write(path, [0x7f, b'E', b'L', b'F', 2, 1, 1, 0]).unwrap();
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
