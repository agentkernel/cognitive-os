//! Isolated positive transaction coverage for the Linux installer adapter.

#![cfg(unix)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::{
    LinuxBundleManifest, LinuxBundleServiceError, LinuxBundleSingleServiceController,
    install_linux_bundle_with_controller,
};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tar::{Builder as TarBuilder, Header as TarHeader};

const KEY_ID: &str = "p1t08-adapter-test-key";
const KEYRING_VERSION: &str = "p1t08-adapter-test-keyring-v1";
const PI_VERSION: &str = "0.81.1";
const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
const VERSION: &str = "1.2.3";

#[test]
fn adapter_runs_a_positive_canonical_transaction_with_an_isolated_controller() {
    let fixture_root = tempfile::tempdir().unwrap();
    let xdg_data_home = fixture_root.path().join("xdg-data");
    let deployment_root = xdg_data_home.join("cognitiveos/deployment");
    let bundle_directory = fixture_root.path().join("bundle");
    fs::create_dir_all(&bundle_directory).unwrap();
    write_signed_bundle(&bundle_directory, &SigningKey::from_bytes(&[0x51; 32]));

    let mut controller = RecordingController::default();
    let arguments = installer_arguments(&bundle_directory);
    let receipt =
        install_linux_bundle_with_controller(&arguments, &deployment_root, &mut controller)
            .unwrap();

    assert_eq!(receipt.installed_version, VERSION);
    assert_eq!(receipt.previous_active_version, None);
    assert_eq!(receipt.resulting_active_version, VERSION);
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.2.3\n"
    );
    assert_eq!(
        controller.actions,
        ["publish:1.2.3", "restart", "confirm:1.2.3", "confirm:1.2.3",]
    );
}

#[derive(Default)]
struct RecordingController {
    actions: Vec<String>,
}

impl LinuxBundleSingleServiceController for RecordingController {
    fn publish_active_unit(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.actions.push(format!("publish:{version}"));
        Ok(())
    }

    fn restart_active_service(&mut self) -> Result<(), LinuxBundleServiceError> {
        self.actions.push("restart".to_owned());
        Ok(())
    }

    fn stop_active_service(&mut self) -> Result<(), LinuxBundleServiceError> {
        self.actions.push("stop".to_owned());
        Ok(())
    }

    fn confirm_active_service(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.actions.push(format!("confirm:{version}"));
        Ok(())
    }

    fn remove_active_unit(&mut self) -> Result<(), LinuxBundleServiceError> {
        self.actions.push("remove-unit".to_owned());
        Ok(())
    }
}

fn installer_arguments(bundle_directory: &Path) -> Vec<String> {
    let public_key = URL_SAFE_NO_PAD.encode(
        SigningKey::from_bytes(&[0x51; 32])
            .verifying_key()
            .to_bytes(),
    );
    [
        "--bundle-directory",
        bundle_directory.to_str().unwrap(),
        "--expected-release-version",
        VERSION,
        "--expected-pi-version",
        PI_VERSION,
        "--expected-pi-integrity",
        PI_INTEGRITY,
        "--keyring-version",
        KEYRING_VERSION,
        "--key-id",
        KEY_ID,
        "--public-key-base64url",
        &public_key,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn write_signed_bundle(bundle_directory: &Path, signing_key: &SigningKey) {
    let artifact = runnable_archive_bytes();
    let manifest = LinuxBundleManifest {
        schema_version: 1,
        product: "cognitiveos-personal".to_owned(),
        platform: "linux-x86_64".to_owned(),
        version: VERSION.to_owned(),
        artifact_file: "bundle.tar.gz".to_owned(),
        artifact_sha256: format!("sha256:{:x}", Sha256::digest(&artifact)),
        attestation_reference: "https://example.invalid/adapter/1.2.3".to_owned(),
        attestation_statement_file: "statement.json".to_owned(),
        attestation_signature_file: "signature.json".to_owned(),
        pi_version: PI_VERSION.to_owned(),
        pi_integrity: PI_INTEGRITY.to_owned(),
    };
    let statement = json!({"artifact_file":manifest.artifact_file,"artifact_sha256":manifest.artifact_sha256,"pi_integrity":manifest.pi_integrity,"pi_version":manifest.pi_version,"platform":manifest.platform,"product":manifest.product,"provenance_reference":manifest.attestation_reference,"schema":"cognitiveos.personal.linux-bundle-attestation","schema_version":1,"version":manifest.version});
    let statement_bytes = serde_json_canonicalizer::to_vec(&statement).unwrap();
    let signature = signing_key.sign(&statement_bytes);
    fs::write(bundle_directory.join("bundle.tar.gz"), artifact).unwrap();
    fs::write(
        bundle_directory.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(bundle_directory.join("statement.json"), statement_bytes).unwrap();
    fs::write(bundle_directory.join("signature.json"), serde_json::to_vec(&json!({"algorithm":"Ed25519","key_id":KEY_ID,"schema":"cognitiveos.personal.linux-bundle-signature","schema_version":1,"signature":URL_SAFE_NO_PAD.encode(signature.to_bytes())})).unwrap()).unwrap();
}

fn runnable_archive_bytes() -> Vec<u8> {
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    append_file(
        &mut tar_builder,
        "bin/kernel-server",
        b"p1t08-adapter-fixture-kernel-server",
        0o755,
    );
    append_file(
        &mut tar_builder,
        "bin/cognitive",
        b"p1t08-adapter-fixture-cognitive",
        0o755,
    );
    append_file(
        &mut tar_builder,
        "extensions/pi-cognitiveos/dist/index.js",
        b"export {};\n",
        0o644,
    );
    tar_builder.into_inner().unwrap().finish().unwrap()
}

fn append_file(
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
