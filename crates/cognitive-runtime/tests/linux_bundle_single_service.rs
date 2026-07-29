//! Failure-first coverage for the Personal single-service MVP transaction.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::{
    ExpectedPiCompatibility, LinuxBundleManifest, LinuxBundleServiceError,
    LinuxBundleSingleServiceController, TrustedKeyInput, TrustedKeyStatus, TrustedKeyring,
    install_linux_bundle_single_service,
};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use tar::{Builder as TarBuilder, Header as TarHeader};

const TEST_KEY_ID: &str = "p1t08-single-service-test-key";
const TEST_KEYRING_VERSION: &str = "p1t08-single-service-test-keyring-v1";
const PI_VERSION: &str = "0.81.1";
const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
const ARTIFACT_FILENAME: &str = "cognitiveos-linux-x86_64.tar.gz";
const USER_DATA_SENTINEL: &[u8] = b"single-service-user-data";

#[test]
fn upgrade_confirms_service_before_and_after_pointer_publication() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller =
        RecordingSingleServiceController::with_outcomes([Ok(()), Ok(()), Ok(()), Ok(())]);

    let receipt = install_linux_bundle_single_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    )
    .unwrap();

    assert_eq!(receipt.previous_active_version.as_deref(), Some("1.0.0"));
    assert_eq!(receipt.resulting_active_version, "2.0.0");
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
    assert!(
        deployment_root
            .join("versions/2.0.0/bin/kernel-server")
            .is_file()
    );
    assert_eq!(
        controller.actions,
        ["publish:2.0.0", "restart", "confirm:2.0.0", "confirm:2.0.0"]
    );
}

#[test]
fn health_failure_before_pointer_switch_restores_the_previous_unit_and_service() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller = RecordingSingleServiceController::with_outcomes([
        Ok(()),
        Ok(()),
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
    ]);

    let result = install_linux_bundle_single_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    );

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed)
    ));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert_eq!(
        controller.actions,
        [
            "publish:2.0.0",
            "restart",
            "confirm:2.0.0",
            "stop",
            "publish:1.0.0",
            "restart",
            "confirm:1.0.0"
        ]
    );
    assert_eq!(
        fs::read(deployment_root.join("user-data/personal.sqlite")).unwrap(),
        USER_DATA_SENTINEL
    );
}

#[test]
fn final_confirmation_failure_after_pointer_switch_restores_the_previous_version() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller = RecordingSingleServiceController::with_outcomes([
        Ok(()),
        Ok(()),
        Ok(()),
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
    ]);

    let result = install_linux_bundle_single_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    );

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed)
    ));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert_eq!(
        controller.actions,
        [
            "publish:2.0.0",
            "restart",
            "confirm:2.0.0",
            "confirm:2.0.0",
            "stop",
            "publish:1.0.0",
            "restart",
            "confirm:1.0.0"
        ]
    );
}

#[test]
fn failed_first_install_stops_service_removes_unit_and_leaves_no_pointer() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("deployment");
    let mut controller = RecordingSingleServiceController::with_outcomes([
        Ok(()),
        Ok(()),
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed),
        Ok(()),
        Ok(()),
    ]);

    let result = install_linux_bundle_single_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    );

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed)
    ));
    assert!(!deployment_root.join("active-version").exists());
    assert_eq!(
        controller.actions,
        [
            "publish:2.0.0",
            "restart",
            "confirm:2.0.0",
            "stop",
            "remove-unit"
        ]
    );
}

#[test]
fn incomplete_rollback_returns_rollback_incomplete_without_a_receipt() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller = RecordingSingleServiceController::with_outcomes([
        Ok(()),
        Ok(()),
        Err(LinuxBundleServiceError::FinalServiceConfirmationFailed),
        Err(LinuxBundleServiceError::CandidateStartFailed),
    ]);

    let result = install_linux_bundle_single_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    );

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::RollbackIncomplete)
    ));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
}

struct RecordingSingleServiceController {
    actions: Vec<String>,
    outcomes: VecDeque<Result<(), LinuxBundleServiceError>>,
}

impl RecordingSingleServiceController {
    fn with_outcomes(
        outcomes: impl IntoIterator<Item = Result<(), LinuxBundleServiceError>>,
    ) -> Self {
        Self {
            actions: Vec::new(),
            outcomes: outcomes.into_iter().collect(),
        }
    }

    fn record(&mut self, action: String) -> Result<(), LinuxBundleServiceError> {
        self.actions.push(action);
        self.outcomes.pop_front().unwrap_or(Ok(()))
    }
}

impl LinuxBundleSingleServiceController for RecordingSingleServiceController {
    fn publish_active_unit(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.record(format!("publish:{version}"))
    }

    fn restart_active_service(&mut self) -> Result<(), LinuxBundleServiceError> {
        self.record("restart".to_owned())
    }

    fn stop_active_service(&mut self) -> Result<(), LinuxBundleServiceError> {
        self.record("stop".to_owned())
    }

    fn confirm_active_service(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.record(format!("confirm:{version}"))
    }

    fn remove_active_unit(&mut self) -> Result<(), LinuxBundleServiceError> {
        self.record("remove-unit".to_owned())
    }
}

struct BundleFixture {
    temporary_directory: tempfile::TempDir,
    keyring: TrustedKeyring,
}

fn signed_bundle(version: &str) -> BundleFixture {
    let temporary_directory = tempfile::tempdir().unwrap();
    let signing_key = SigningKey::from_bytes(&[0x62; 32]);
    let artifact = runnable_archive_bytes();
    let manifest = LinuxBundleManifest {
        schema_version: 1,
        product: "cognitiveos-personal".to_owned(),
        platform: "linux-x86_64".to_owned(),
        version: version.to_owned(),
        artifact_file: ARTIFACT_FILENAME.to_owned(),
        artifact_sha256: sha256_bytes(&artifact),
        attestation_reference: format!("https://example.invalid/provenance/{version}"),
        attestation_statement_file: "attestation.statement.json".to_owned(),
        attestation_signature_file: "attestation.signature.json".to_owned(),
        pi_version: PI_VERSION.to_owned(),
        pi_integrity: PI_INTEGRITY.to_owned(),
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
    let statement_bytes = serde_json_canonicalizer::to_vec(&statement).unwrap();
    let signature = signing_key.sign(&statement_bytes);
    fs::write(temporary_directory.path().join(ARTIFACT_FILENAME), artifact).unwrap();
    fs::write(
        temporary_directory.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        temporary_directory
            .path()
            .join("attestation.statement.json"),
        statement_bytes,
    )
    .unwrap();
    fs::write(
        temporary_directory
            .path()
            .join("attestation.signature.json"),
        serde_json::to_vec(&json!({
            "algorithm": "Ed25519",
            "key_id": TEST_KEY_ID,
            "schema": "cognitiveos.personal.linux-bundle-signature",
            "schema_version": 1,
            "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        }))
        .unwrap(),
    )
    .unwrap();
    let keyring = TrustedKeyring::new(
        TEST_KEYRING_VERSION,
        vec![TrustedKeyInput {
            key_id: TEST_KEY_ID.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Active,
        }],
    )
    .unwrap();
    BundleFixture {
        temporary_directory,
        keyring,
    }
}

fn runnable_archive_bytes() -> Vec<u8> {
    let executable = b"single-service-kernel-server";
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_mode(0o755);
    header.set_size(executable.len() as u64);
    header.set_cksum();
    tar_builder
        .append_data(&mut header, "bin/kernel-server", &executable[..])
        .unwrap();
    tar_builder.into_inner().unwrap().finish().unwrap()
}

fn expected_pi() -> ExpectedPiCompatibility {
    ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY)
}

fn prepare_existing_installation(deployment_root: &Path, version: &str) {
    fs::create_dir_all(deployment_root.join("versions").join(version).join("bin")).unwrap();
    fs::create_dir_all(deployment_root.join("staged")).unwrap();
    fs::create_dir_all(deployment_root.join("user-data")).unwrap();
    fs::write(
        deployment_root
            .join("versions")
            .join(version)
            .join("bin/kernel-server"),
        b"previous-kernel-server",
    )
    .unwrap();
    fs::write(
        deployment_root.join("active-version"),
        format!("{version}\n"),
    )
    .unwrap();
    fs::write(
        deployment_root.join("user-data/personal.sqlite"),
        USER_DATA_SENTINEL,
    )
    .unwrap();
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
