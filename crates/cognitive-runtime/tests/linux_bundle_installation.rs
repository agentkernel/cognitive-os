//! Failure-first coverage for Personal Linux bundle installation orchestration.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::linux_bundle::{
    ExpectedPiCompatibility, LinuxBundleDeployment, LinuxBundleError, LinuxBundleManifest,
    TrustedKeyInput, TrustedKeyStatus, TrustedKeyring, verify_linux_bundle,
};
use cognitive_runtime::linux_bundle_installation::install_linux_bundle;
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::fs;
use std::path::Path;
use tar::{Builder as TarBuilder, EntryType, Header as TarHeader};

const TEST_ONLY_PRIVATE_SIGNING_SEED: [u8; 32] = [0x39; 32];
const TEST_ONLY_KEY_ID: &str = "p1t08-installation-test-key";
const TEST_ONLY_KEYRING_VERSION: &str = "p1t08-installation-test-keyring-v1";
const PRODUCT: &str = "cognitiveos-personal";
const PLATFORM: &str = "linux-x86_64";
const PI_VERSION: &str = "0.81.1";
const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
const STATEMENT_FILE: &str = "attestation.statement.json";
const SIGNATURE_FILE: &str = "attestation.signature.json";
const KERNEL_SERVER_CONTENT: &[u8] = b"p1t08-test-kernel-server";
const USER_DATA_SENTINEL: &[u8] = b"p1t08-private-user-data-sentinel";

struct SignedBundleFixture {
    temporary_directory: tempfile::TempDir,
    signing_key: SigningKey,
    manifest: LinuxBundleManifest,
    statement: Value,
}

impl SignedBundleFixture {
    fn new(version: &str) -> Self {
        // Deterministic test-only signing material never leaves this test binary.
        let signing_key = SigningKey::from_bytes(&TEST_ONLY_PRIVATE_SIGNING_SEED);
        let temporary_directory = tempfile::tempdir().unwrap();
        let artifact_bytes = runnable_archive_bytes();
        let manifest = LinuxBundleManifest {
            schema_version: 1,
            product: PRODUCT.to_owned(),
            platform: PLATFORM.to_owned(),
            version: version.to_owned(),
            artifact_file: "cognitiveos-linux-x86_64.tar.gz".to_owned(),
            artifact_sha256: sha256_digest(&artifact_bytes),
            attestation_reference: format!("https://example.invalid/provenance/{version}"),
            attestation_statement_file: STATEMENT_FILE.to_owned(),
            attestation_signature_file: SIGNATURE_FILE.to_owned(),
            pi_version: PI_VERSION.to_owned(),
            pi_integrity: PI_INTEGRITY.to_owned(),
        };
        fs::write(
            temporary_directory.path().join(&manifest.artifact_file),
            artifact_bytes,
        )
        .unwrap();
        let statement = statement_for_manifest(&manifest);
        let fixture = Self {
            temporary_directory,
            signing_key,
            manifest,
            statement,
        };
        fixture.write_manifest();
        fixture.sign_and_write_statement();
        fixture
    }

    fn bundle_directory(&self) -> &Path {
        self.temporary_directory.path()
    }

    fn deployment_root(&self, name: &str) -> std::path::PathBuf {
        self.temporary_directory.path().join(name)
    }

    fn write_manifest(&self) {
        fs::write(
            self.bundle_directory().join("manifest.json"),
            serde_json::to_vec(&self.manifest).unwrap(),
        )
        .unwrap();
    }

    fn sign_and_write_statement(&self) {
        let statement_bytes = serde_json_canonicalizer::to_vec(&self.statement).unwrap();
        let detached_signature = self.signing_key.sign(&statement_bytes);
        fs::write(
            self.bundle_directory().join(STATEMENT_FILE),
            statement_bytes,
        )
        .unwrap();
        self.write_signature_envelope(json!({
            "algorithm": "Ed25519",
            "key_id": TEST_ONLY_KEY_ID,
            "schema": "cognitiveos.personal.linux-bundle-signature",
            "schema_version": 1,
            "signature": URL_SAFE_NO_PAD.encode(detached_signature.to_bytes()),
        }));
    }

    fn write_signature_envelope(&self, envelope: Value) {
        fs::write(
            self.bundle_directory().join(SIGNATURE_FILE),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
    }

    fn replace_with_signed_artifact(&mut self, artifact_bytes: Vec<u8>) {
        self.manifest.artifact_sha256 = sha256_digest(&artifact_bytes);
        self.statement = statement_for_manifest(&self.manifest);
        fs::write(
            self.bundle_directory().join(&self.manifest.artifact_file),
            artifact_bytes,
        )
        .unwrap();
        self.write_manifest();
        self.sign_and_write_statement();
    }

    fn trusted_keyring(&self) -> TrustedKeyring {
        TrustedKeyring::new(
            TEST_ONLY_KEYRING_VERSION,
            vec![TrustedKeyInput {
                key_id: TEST_ONLY_KEY_ID.to_owned(),
                algorithm: "Ed25519".to_owned(),
                public_key_base64url: URL_SAFE_NO_PAD
                    .encode(self.signing_key.verifying_key().to_bytes()),
                status: TrustedKeyStatus::Active,
            }],
        )
        .unwrap()
    }
}

fn runnable_archive_bytes() -> Vec<u8> {
    archive_with_regular_entry("bin/kernel-server", 0o755)
}

fn archive_with_regular_entry(entry_path: &str, mode: u32) -> Vec<u8> {
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_mode(mode);
    header.set_size(KERNEL_SERVER_CONTENT.len() as u64);
    header.set_cksum();
    tar_builder
        .append_data(&mut header, entry_path, KERNEL_SERVER_CONTENT)
        .unwrap();
    let gzip_encoder = tar_builder.into_inner().unwrap();
    gzip_encoder.finish().unwrap()
}

fn archive_with_symbolic_link() -> Vec<u8> {
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_entry_type(EntryType::Symlink);
    header.set_size(0);
    header.set_link_name("/outside-staging-root").unwrap();
    header.set_cksum();
    tar_builder
        .append_data(&mut header, "bin/kernel-server", &[][..])
        .unwrap();
    let gzip_encoder = tar_builder.into_inner().unwrap();
    gzip_encoder.finish().unwrap()
}

fn archive_with_raw_traversal_entry() -> Vec<u8> {
    let entry_path = b"../escape";
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_mode(0o755);
    header.set_size(KERNEL_SERVER_CONTENT.len() as u64);
    header.as_mut_bytes()[..entry_path.len()].copy_from_slice(entry_path);
    header.set_cksum();
    tar_builder.append(&header, KERNEL_SERVER_CONTENT).unwrap();
    let gzip_encoder = tar_builder.into_inner().unwrap();
    gzip_encoder.finish().unwrap()
}

#[test]
fn verified_archive_with_wrong_layout_fails_without_pointer_or_receipt() {
    let mut fixture = SignedBundleFixture::new("2.0.0");
    fixture
        .replace_with_signed_artifact(archive_with_regular_entry("bin/not-kernel-server", 0o755));
    let deployment_root = fixture.deployment_root("wrong-layout-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");

    let result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &fixture.trusted_keyring(),
        |_| true,
    );

    assert!(matches!(result, Err(LinuxBundleError::UnsafeArchive)));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert!(!deployment_root.join("staged/2.0.0").exists());
}

#[test]
fn traversal_link_and_non_executable_archives_fail_without_active_state_mutation() {
    let unsafe_archives = [
        archive_with_raw_traversal_entry(),
        archive_with_symbolic_link(),
        archive_with_regular_entry("bin/kernel-server", 0o644),
    ];

    for (case_index, unsafe_archive) in unsafe_archives.into_iter().enumerate() {
        let mut fixture = SignedBundleFixture::new("2.0.0");
        fixture.replace_with_signed_artifact(unsafe_archive);
        let deployment_root = fixture.deployment_root(&format!("unsafe-archive-{case_index}"));
        prepare_existing_installation(&deployment_root, "1.0.0");

        let result = install_linux_bundle(
            fixture.bundle_directory(),
            &deployment_root,
            &expected_pi(),
            &fixture.trusted_keyring(),
            |_| true,
        );

        assert!(matches!(result, Err(LinuxBundleError::UnsafeArchive)));
        assert_eq!(
            fs::read_to_string(deployment_root.join("active-version")).unwrap(),
            "1.0.0\n"
        );
        assert!(!deployment_root.join("staged/2.0.0").exists());
    }
}

fn statement_for_manifest(manifest: &LinuxBundleManifest) -> Value {
    json!({
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
    })
}

fn expected_pi() -> ExpectedPiCompatibility {
    ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY)
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn prepare_existing_installation(deployment_root: &Path, version: &str) {
    fs::create_dir_all(deployment_root.join("staged")).unwrap();
    fs::create_dir_all(deployment_root.join("versions").join(version)).unwrap();
    fs::create_dir_all(deployment_root.join("user-data")).unwrap();
    fs::write(
        deployment_root.join("active-version"),
        format!("{version}\n"),
    )
    .unwrap();
    fs::write(
        deployment_root.join("user-data").join("personal.sqlite"),
        USER_DATA_SENTINEL,
    )
    .unwrap();
}

fn assert_deployment_was_not_created(deployment_root: &Path) {
    assert!(!deployment_root.exists());
}

#[test]
fn valid_signed_bundle_runs_the_complete_order_and_returns_non_secret_receipt() {
    let fixture = SignedBundleFixture::new("1.2.3");
    let deployment_root = fixture.deployment_root("deployment");
    let health_check_calls = Cell::new(0_u32);

    let receipt = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &fixture.trusted_keyring(),
        |staged_candidate| {
            health_check_calls.set(health_check_calls.get() + 1);
            staged_candidate.join("bin/kernel-server").is_file()
        },
    )
    .unwrap();

    assert_eq!(health_check_calls.get(), 1);
    assert!(deployment_root.join("versions/1.2.3").is_dir());
    assert!(
        deployment_root
            .join("versions/1.2.3/bin/kernel-server")
            .is_file()
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.2.3\n"
    );
    assert_eq!(receipt.installed_version, "1.2.3");
    assert_eq!(receipt.previous_active_version, None);
    assert_eq!(receipt.resulting_active_version, "1.2.3");
    assert_eq!(receipt.trusted_key_id, TEST_ONLY_KEY_ID);
    assert_eq!(receipt.trusted_keyring_version, TEST_ONLY_KEYRING_VERSION);
}

#[test]
fn invalid_signature_fails_before_deployment_mutation_or_health_check() {
    let fixture = SignedBundleFixture::new("1.2.3");
    let deployment_root = fixture.deployment_root("invalid-signature-deployment");
    let health_check_calls = Cell::new(0_u32);
    fixture.write_signature_envelope(json!({
        "algorithm": "Ed25519",
        "key_id": TEST_ONLY_KEY_ID,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": URL_SAFE_NO_PAD.encode([0_u8; 64]),
    }));

    let result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &fixture.trusted_keyring(),
        |_| {
            health_check_calls.set(health_check_calls.get() + 1);
            true
        },
    );

    assert!(matches!(result, Err(LinuxBundleError::SignatureMismatch)));
    assert_eq!(health_check_calls.get(), 0);
    assert_deployment_was_not_created(&deployment_root);
}

#[test]
fn unknown_revoked_and_bundle_selected_keys_fail_before_deployment_mutation() {
    let unknown_fixture = SignedBundleFixture::new("1.2.3");
    let unknown_root = unknown_fixture.deployment_root("unknown-key-deployment");
    let different_signing_key = SigningKey::from_bytes(&[0x47; 32]);
    let unknown_keyring = TrustedKeyring::new(
        "different-test-keyring",
        vec![TrustedKeyInput {
            key_id: "different-trusted-key".to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD
                .encode(different_signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Active,
        }],
    )
    .unwrap();
    assert!(matches!(
        install_linux_bundle(
            unknown_fixture.bundle_directory(),
            &unknown_root,
            &expected_pi(),
            &unknown_keyring,
            |_| true,
        ),
        Err(LinuxBundleError::UnknownOrUntrustedKey)
    ));
    assert_deployment_was_not_created(&unknown_root);

    let revoked_fixture = SignedBundleFixture::new("1.2.3");
    let revoked_root = revoked_fixture.deployment_root("revoked-key-deployment");
    let revoked_keyring = TrustedKeyring::new(
        TEST_ONLY_KEYRING_VERSION,
        vec![TrustedKeyInput {
            key_id: TEST_ONLY_KEY_ID.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD
                .encode(revoked_fixture.signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Revoked,
        }],
    )
    .unwrap();
    assert!(matches!(
        install_linux_bundle(
            revoked_fixture.bundle_directory(),
            &revoked_root,
            &expected_pi(),
            &revoked_keyring,
            |_| true,
        ),
        Err(LinuxBundleError::UnknownOrUntrustedKey)
    ));
    assert_deployment_was_not_created(&revoked_root);

    let selected_fixture = SignedBundleFixture::new("1.2.3");
    let selected_root = selected_fixture.deployment_root("selected-key-deployment");
    let signature_envelope =
        fs::read_to_string(selected_fixture.bundle_directory().join(SIGNATURE_FILE)).unwrap();
    let envelope_with_untrusted_key = signature_envelope.replacen(
        '{',
        &format!(
            "{{\"public_key\":\"{}\",",
            URL_SAFE_NO_PAD.encode(selected_fixture.signing_key.verifying_key().to_bytes())
        ),
        1,
    );
    fs::write(
        selected_fixture.bundle_directory().join(SIGNATURE_FILE),
        envelope_with_untrusted_key,
    )
    .unwrap();
    assert!(matches!(
        install_linux_bundle(
            selected_fixture.bundle_directory(),
            &selected_root,
            &expected_pi(),
            &selected_fixture.trusted_keyring(),
            |_| true,
        ),
        Err(LinuxBundleError::MalformedAttestation(_))
    ));
    assert_deployment_was_not_created(&selected_root);
}

#[test]
fn artifact_and_statement_tampering_fail_before_deployment_or_health_check() {
    let artifact_fixture = SignedBundleFixture::new("1.2.3");
    let artifact_root = artifact_fixture.deployment_root("artifact-tamper-deployment");
    let health_check_calls = Cell::new(0_u32);
    fs::write(
        artifact_fixture
            .bundle_directory()
            .join(&artifact_fixture.manifest.artifact_file),
        b"tampered-artifact",
    )
    .unwrap();
    assert!(matches!(
        install_linux_bundle(
            artifact_fixture.bundle_directory(),
            &artifact_root,
            &expected_pi(),
            &artifact_fixture.trusted_keyring(),
            |_| {
                health_check_calls.set(health_check_calls.get() + 1);
                true
            },
        ),
        Err(LinuxBundleError::ArtifactDigestMismatch)
    ));
    assert_eq!(health_check_calls.get(), 0);
    assert_deployment_was_not_created(&artifact_root);

    let statement_fixture = SignedBundleFixture::new("1.2.3");
    let statement_root = statement_fixture.deployment_root("statement-tamper-deployment");
    fs::write(
        statement_fixture.bundle_directory().join(STATEMENT_FILE),
        b"statement-tampering-sentinel",
    )
    .unwrap();
    assert!(
        install_linux_bundle(
            statement_fixture.bundle_directory(),
            &statement_root,
            &expected_pi(),
            &statement_fixture.trusted_keyring(),
            |_| {
                health_check_calls.set(health_check_calls.get() + 1);
                true
            },
        )
        .is_err()
    );
    assert_eq!(health_check_calls.get(), 0);
    assert_deployment_was_not_created(&statement_root);
}

#[test]
fn wrong_pi_pin_and_unsupported_platform_fail_before_deployment_mutation() {
    let wrong_pi_fixture = SignedBundleFixture::new("1.2.3");
    let wrong_pi_root = wrong_pi_fixture.deployment_root("wrong-pi-deployment");
    let wrong_pi = ExpectedPiCompatibility::new(PI_VERSION, "sha512:wrong-product-pin");
    assert!(matches!(
        install_linux_bundle(
            wrong_pi_fixture.bundle_directory(),
            &wrong_pi_root,
            &wrong_pi,
            &wrong_pi_fixture.trusted_keyring(),
            |_| true,
        ),
        Err(LinuxBundleError::PiCompatibilityMismatch)
    ));
    assert_deployment_was_not_created(&wrong_pi_root);

    let mut platform_fixture = SignedBundleFixture::new("1.2.3");
    let platform_root = platform_fixture.deployment_root("platform-deployment");
    platform_fixture.manifest.platform = "linux-aarch64".to_owned();
    platform_fixture.write_manifest();
    assert!(matches!(
        install_linux_bundle(
            platform_fixture.bundle_directory(),
            &platform_root,
            &expected_pi(),
            &platform_fixture.trusted_keyring(),
            |_| true,
        ),
        Err(LinuxBundleError::UnsupportedPlatform { .. })
    ));
    assert_deployment_was_not_created(&platform_root);
}

#[test]
fn health_failure_preserves_previous_version_user_data_and_staged_candidate() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("health-failure-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let health_check_calls = Cell::new(0_u32);

    let result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &fixture.trusted_keyring(),
        |staged_candidate| {
            health_check_calls.set(health_check_calls.get() + 1);
            assert!(staged_candidate.join("bin/kernel-server").is_file());
            false
        },
    );

    assert!(matches!(result, Err(LinuxBundleError::HealthCheckFailed)));
    assert_eq!(health_check_calls.get(), 1);
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert!(deployment_root.join("versions/1.0.0").is_dir());
    assert!(deployment_root.join("staged/2.0.0").is_dir());
    assert_eq!(
        fs::read(deployment_root.join("user-data/personal.sqlite")).unwrap(),
        USER_DATA_SENTINEL
    );
}

#[test]
fn successful_upgrade_retains_previous_version_and_user_data() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("upgrade-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");

    let receipt = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &fixture.trusted_keyring(),
        |staged_candidate| staged_candidate.join("bin/kernel-server").is_file(),
    )
    .unwrap();

    assert_eq!(receipt.previous_active_version.as_deref(), Some("1.0.0"));
    assert_eq!(receipt.resulting_active_version, "2.0.0");
    assert!(deployment_root.join("versions/1.0.0").is_dir());
    assert!(deployment_root.join("versions/2.0.0").is_dir());
    assert_eq!(
        fs::read(deployment_root.join("user-data/personal.sqlite")).unwrap(),
        USER_DATA_SENTINEL
    );
}

#[test]
fn repeating_the_active_version_is_idempotent_and_leaves_no_partial_staging() {
    let fixture = SignedBundleFixture::new("1.2.3");
    let deployment_root = fixture.deployment_root("repeat-deployment");
    let trusted_keyring = fixture.trusted_keyring();
    install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring,
        |_| true,
    )
    .unwrap();

    let receipt = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring,
        |_| true,
    )
    .unwrap();

    assert_eq!(receipt.previous_active_version.as_deref(), Some("1.2.3"));
    assert_eq!(receipt.resulting_active_version, "1.2.3");
    assert!(deployment_root.join("versions/1.2.3").is_dir());
    assert!(!deployment_root.join("staged/1.2.3").exists());
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.2.3\n"
    );
}

#[test]
fn failures_and_receipts_do_not_disclose_bundle_key_or_user_data_bytes() {
    let fixture = SignedBundleFixture::new("1.2.3");
    let deployment_root = fixture.deployment_root("disclosure-deployment");
    let public_key_text = URL_SAFE_NO_PAD.encode(fixture.signing_key.verifying_key().to_bytes());
    let signature_text =
        fs::read_to_string(fixture.bundle_directory().join(SIGNATURE_FILE)).unwrap();
    let statement_text =
        fs::read_to_string(fixture.bundle_directory().join(STATEMENT_FILE)).unwrap();
    fixture.write_signature_envelope(json!({
        "algorithm": "Ed25519",
        "key_id": TEST_ONLY_KEY_ID,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": URL_SAFE_NO_PAD.encode([0_u8; 64]),
    }));
    let error_text = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &fixture.trusted_keyring(),
        |_| true,
    )
    .unwrap_err()
    .to_string();

    let sensitive_texts = [
        URL_SAFE_NO_PAD.encode(TEST_ONLY_PRIVATE_SIGNING_SEED),
        public_key_text,
        signature_text,
        String::from_utf8_lossy(KERNEL_SERVER_CONTENT).into_owned(),
        statement_text,
        String::from_utf8_lossy(USER_DATA_SENTINEL).into_owned(),
    ];
    for sensitive_text in &sensitive_texts {
        assert!(!error_text.contains(sensitive_text));
    }

    let valid_fixture = SignedBundleFixture::new("1.2.3");
    let receipt = install_linux_bundle(
        valid_fixture.bundle_directory(),
        &valid_fixture.deployment_root("receipt-deployment"),
        &expected_pi(),
        &valid_fixture.trusted_keyring(),
        |_| true,
    )
    .unwrap();
    let receipt_debug_text = format!("{receipt:?}");
    for sensitive_text in &sensitive_texts {
        assert!(!receipt_debug_text.contains(sensitive_text));
    }
}

#[test]
fn post_verification_artifact_mutation_cannot_create_a_staged_candidate() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("post-verification-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let verified_bundle = verify_linux_bundle(
        fixture.bundle_directory(),
        &expected_pi(),
        &fixture.trusted_keyring(),
    )
    .unwrap();
    fs::write(
        fixture
            .bundle_directory()
            .join(&fixture.manifest.artifact_file),
        b"artifact-mutated-after-verification",
    )
    .unwrap();

    let deployment = LinuxBundleDeployment::open(&deployment_root).unwrap();
    assert!(matches!(
        deployment.stage_verified_bundle(fixture.bundle_directory(), &verified_bundle),
        Err(LinuxBundleError::ArtifactDigestMismatch)
    ));
    assert!(!deployment_root.join("staged/2.0.0").exists());
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
}
