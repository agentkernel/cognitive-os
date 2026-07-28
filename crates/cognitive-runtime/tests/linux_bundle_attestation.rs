//! Failure-first coverage for the offline Linux bundle attestation boundary.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::linux_bundle::{
    ExpectedPiCompatibility, LinuxBundleDeployment, LinuxBundleError, LinuxBundleManifest,
    TrustedKeyInput, TrustedKeyStatus, TrustedKeyring, verify_linux_bundle,
};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tar::{Builder as TarBuilder, Header as TarHeader};

const TEST_ONLY_KEY_ID: &str = "p1t08-test-only-key-2026";
const PRODUCT: &str = "cognitiveos-personal";
const PLATFORM: &str = "linux-x86_64";
const PI_VERSION: &str = "0.81.1";
const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
const STATEMENT_FILE: &str = "attestation.statement.json";
const SIGNATURE_FILE: &str = "attestation.signature.json";

struct AttestedBundleFixture {
    temporary_directory: tempfile::TempDir,
    signing_key: SigningKey,
    manifest: LinuxBundleManifest,
    statement: Value,
}

impl AttestedBundleFixture {
    fn new() -> Self {
        // Fixed test-only material keeps the suite deterministic. This seed is
        // never written to a bundle, release path, log, or production keyring.
        let signing_key = SigningKey::from_bytes(&[0x5a; 32]);
        let temporary_directory = tempfile::tempdir().unwrap();
        let artifact = runnable_archive_bytes();
        let manifest = LinuxBundleManifest {
            schema_version: 1,
            product: PRODUCT.to_owned(),
            platform: PLATFORM.to_owned(),
            version: "1.2.3".to_owned(),
            artifact_file: "cognitiveos-linux-x86_64.tar.gz".to_owned(),
            artifact_sha256: sha256_digest(&artifact),
            attestation_reference: "https://example.invalid/provenance/1.2.3".to_owned(),
            attestation_statement_file: STATEMENT_FILE.to_owned(),
            attestation_signature_file: SIGNATURE_FILE.to_owned(),
            pi_version: PI_VERSION.to_owned(),
            pi_integrity: PI_INTEGRITY.to_owned(),
        };
        fs::write(
            temporary_directory.path().join(&manifest.artifact_file),
            artifact,
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

    fn path(&self) -> &Path {
        self.temporary_directory.path()
    }

    fn write_manifest(&self) {
        fs::write(
            self.path().join("manifest.json"),
            serde_json::to_vec(&self.manifest).unwrap(),
        )
        .unwrap();
    }

    fn sign_and_write_statement(&self) {
        let statement_bytes = serde_json_canonicalizer::to_vec(&self.statement).unwrap();
        let signature = self.signing_key.sign(&statement_bytes);
        fs::write(self.path().join(STATEMENT_FILE), &statement_bytes).unwrap();
        self.write_signature_envelope(json!({
            "algorithm": "Ed25519",
            "key_id": TEST_ONLY_KEY_ID,
            "schema": "cognitiveos.personal.linux-bundle-signature",
            "schema_version": 1,
            "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        }));
    }

    fn write_signature_envelope(&self, envelope: Value) {
        fs::write(
            self.path().join(SIGNATURE_FILE),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
    }

    fn trusted_keyring(&self) -> TrustedKeyring {
        trusted_keyring_for(&self.signing_key, TEST_ONLY_KEY_ID)
    }

    fn verify(
        &self,
    ) -> Result<cognitive_runtime::linux_bundle::VerifiedLinuxBundle, LinuxBundleError> {
        verify_linux_bundle(self.path(), &expected_pi(), &self.trusted_keyring())
    }
}

fn runnable_archive_bytes() -> Vec<u8> {
    let executable_contents = b"attestation-test-kernel-server";
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_mode(0o755);
    header.set_size(executable_contents.len() as u64);
    header.set_cksum();
    tar_builder
        .append_data(&mut header, "bin/kernel-server", &executable_contents[..])
        .unwrap();
    let gzip_encoder = tar_builder.into_inner().unwrap();
    gzip_encoder.finish().unwrap()
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

fn trusted_keyring_for(signing_key: &SigningKey, key_id: &str) -> TrustedKeyring {
    TrustedKeyring::new(
        "p1t08-test-keyring-v1",
        vec![TrustedKeyInput {
            key_id: key_id.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Active,
        }],
    )
    .unwrap()
}

fn expected_pi() -> ExpectedPiCompatibility {
    ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY)
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[test]
fn valid_statement_signature_and_trusted_key_produce_verified_bundle() {
    let fixture = AttestedBundleFixture::new();
    let verified_bundle = fixture.verify().unwrap();

    assert_eq!(verified_bundle.manifest().product, PRODUCT);
    assert_eq!(verified_bundle.manifest().version, "1.2.3");
    assert_eq!(verified_bundle.trusted_key_id(), TEST_ONLY_KEY_ID);
}

#[test]
fn artifact_or_signed_statement_tampering_is_rejected() {
    let fixture = AttestedBundleFixture::new();
    fs::write(
        fixture.path().join(&fixture.manifest.artifact_file),
        b"cognitiveos daemon bundlf",
    )
    .unwrap();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::ArtifactDigestMismatch)
    ));

    let fixture = AttestedBundleFixture::new();
    let mut statement_bytes = fs::read(fixture.path().join(STATEMENT_FILE)).unwrap();
    let last_byte = statement_bytes.last_mut().unwrap();
    *last_byte = b' ';
    fs::write(fixture.path().join(STATEMENT_FILE), statement_bytes).unwrap();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::MalformedAttestation(_))
            | Err(LinuxBundleError::NonCanonicalAttestation)
            | Err(LinuxBundleError::SignatureMismatch)
    ));
}

#[test]
fn every_manifest_statement_binding_mismatch_is_rejected() {
    let mismatches = [
        ("product", "another-product"),
        ("platform", "linux-aarch64"),
        ("version", "9.9.9"),
        ("artifact_file", "another.tar.gz"),
        (
            "artifact_sha256",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        ("pi_version", "0.82.0"),
        ("pi_integrity", "sha512:another-integrity"),
    ];

    for (field_name, replacement) in mismatches {
        let mut fixture = AttestedBundleFixture::new();
        fixture.statement[field_name] = json!(replacement);
        fixture.sign_and_write_statement();
        assert!(
            matches!(
                fixture.verify(),
                Err(LinuxBundleError::StatementBindingMismatch)
            ),
            "field {field_name} must remain bound"
        );
    }
}

#[test]
fn wrong_unknown_and_bundle_selected_keys_are_rejected() {
    let fixture = AttestedBundleFixture::new();
    let wrong_key = SigningKey::from_bytes(&[0x6b; 32]);
    let wrong_keyring = trusted_keyring_for(&wrong_key, TEST_ONLY_KEY_ID);
    assert!(matches!(
        verify_linux_bundle(fixture.path(), &expected_pi(), &wrong_keyring),
        Err(LinuxBundleError::SignatureMismatch)
    ));

    let unknown_keyring = trusted_keyring_for(&fixture.signing_key, "different-trusted-key");
    assert!(matches!(
        verify_linux_bundle(fixture.path(), &expected_pi(), &unknown_keyring),
        Err(LinuxBundleError::UnknownOrUntrustedKey)
    ));

    let attacker_key = SigningKey::from_bytes(&[0x7c; 32]);
    let mut self_selected = AttestedBundleFixture::new();
    self_selected.signing_key = attacker_key;
    self_selected.sign_and_write_statement();
    let attacker_public_key =
        URL_SAFE_NO_PAD.encode(self_selected.signing_key.verifying_key().to_bytes());
    let envelope = fs::read_to_string(self_selected.path().join(SIGNATURE_FILE)).unwrap();
    let envelope_with_key = envelope.replacen(
        "{",
        &format!("{{\"public_key\":\"{attacker_public_key}\","),
        1,
    );
    fs::write(self_selected.path().join(SIGNATURE_FILE), envelope_with_key).unwrap();
    assert!(matches!(
        self_selected.verify(),
        Err(LinuxBundleError::MalformedAttestation(_))
    ));
}

#[test]
fn unsupported_malformed_and_noncanonical_signatures_are_rejected() {
    let fixture = AttestedBundleFixture::new();
    fixture.write_signature_envelope(json!({
        "algorithm": "RSA-PSS",
        "key_id": TEST_ONLY_KEY_ID,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": URL_SAFE_NO_PAD.encode([0_u8; 64]),
    }));
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::UnsupportedAttestation(_))
    ));

    let fixture = AttestedBundleFixture::new();
    fixture.write_signature_envelope(json!({
        "algorithm": "Ed25519",
        "key_id": TEST_ONLY_KEY_ID,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": "not-base64!",
    }));
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::MalformedAttestation(_))
    ));

    let fixture = AttestedBundleFixture::new();
    let statement_bytes = fs::read(fixture.path().join(STATEMENT_FILE)).unwrap();
    let signature = fixture.signing_key.sign(&statement_bytes);
    fixture.write_signature_envelope(json!({
        "algorithm": "Ed25519",
        "key_id": TEST_ONLY_KEY_ID,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": format!("{}=", URL_SAFE_NO_PAD.encode(signature.to_bytes())),
    }));
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::MalformedAttestation(_))
    ));

    let fixture = AttestedBundleFixture::new();
    let pretty_statement = serde_json::to_vec_pretty(&fixture.statement).unwrap();
    let pretty_signature = fixture.signing_key.sign(&pretty_statement);
    fs::write(fixture.path().join(STATEMENT_FILE), pretty_statement).unwrap();
    fixture.write_signature_envelope(json!({
        "algorithm": "Ed25519",
        "key_id": TEST_ONLY_KEY_ID,
        "schema": "cognitiveos.personal.linux-bundle-signature",
        "schema_version": 1,
        "signature": URL_SAFE_NO_PAD.encode(pretty_signature.to_bytes()),
    }));
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::NonCanonicalAttestation)
    ));
}

#[test]
fn invalid_trusted_keyrings_fail_closed() {
    assert!(matches!(
        TrustedKeyring::new("test-v1", Vec::new()),
        Err(LinuxBundleError::InvalidTrustedKeyring(_))
    ));

    let signing_key = SigningKey::from_bytes(&[0x5a; 32]);
    let input = TrustedKeyInput {
        key_id: TEST_ONLY_KEY_ID.to_owned(),
        algorithm: "Ed25519".to_owned(),
        public_key_base64url: URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes()),
        status: TrustedKeyStatus::Active,
    };
    assert!(matches!(
        TrustedKeyring::new("test-v1", vec![input.clone(), input]),
        Err(LinuxBundleError::InvalidTrustedKeyring(_))
    ));

    assert!(matches!(
        TrustedKeyring::new(
            "test-v1",
            vec![TrustedKeyInput {
                key_id: TEST_ONLY_KEY_ID.to_owned(),
                algorithm: "Ed25519".to_owned(),
                public_key_base64url: URL_SAFE_NO_PAD.encode([0_u8; 31]),
                status: TrustedKeyStatus::Active,
            }],
        ),
        Err(LinuxBundleError::InvalidTrustedKeyring(_))
    ));

    let padded_key = format!(
        "{}=",
        URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes())
    );
    assert!(matches!(
        TrustedKeyring::new(
            "test-v1",
            vec![TrustedKeyInput {
                key_id: TEST_ONLY_KEY_ID.to_owned(),
                algorithm: "Ed25519".to_owned(),
                public_key_base64url: padded_key,
                status: TrustedKeyStatus::Active,
            }],
        ),
        Err(LinuxBundleError::InvalidTrustedKeyring(_))
    ));
}

#[test]
fn missing_unsafe_or_non_https_attestation_inputs_are_rejected() {
    let fixture = AttestedBundleFixture::new();
    fs::remove_file(fixture.path().join(STATEMENT_FILE)).unwrap();
    assert!(matches!(fixture.verify(), Err(LinuxBundleError::Io(_))));

    let mut fixture = AttestedBundleFixture::new();
    fixture.manifest.attestation_statement_file = "../statement.json".to_owned();
    fixture.write_manifest();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::UnsafePath(_))
    ));

    let mut fixture = AttestedBundleFixture::new();
    fixture.manifest.attestation_reference = "http://example.invalid/provenance".to_owned();
    fixture.write_manifest();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::InvalidAttestationReference)
    ));

    let mut fixture = AttestedBundleFixture::new();
    fixture.manifest.attestation_reference = "https://".to_owned();
    fixture.write_manifest();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::InvalidAttestationReference)
    ));

    let mut fixture = AttestedBundleFixture::new();
    fixture.manifest.attestation_reference =
        "https://user:password@example.invalid/provenance".to_owned();
    fixture.write_manifest();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::InvalidAttestationReference)
    ));

    let mut fixture = AttestedBundleFixture::new();
    fixture.manifest.attestation_statement_file = fixture.manifest.artifact_file.clone();
    fixture.write_manifest();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::InvalidManifest(_))
    ));
}

#[test]
fn revoked_trusted_key_is_rejected_even_when_signature_is_valid() {
    let fixture = AttestedBundleFixture::new();
    let revoked_keyring = TrustedKeyring::new(
        "p1t08-test-keyring-v1",
        vec![TrustedKeyInput {
            key_id: TEST_ONLY_KEY_ID.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD
                .encode(fixture.signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Revoked,
        }],
    )
    .unwrap();
    assert!(matches!(
        verify_linux_bundle(fixture.path(), &expected_pi(), &revoked_keyring),
        Err(LinuxBundleError::UnknownOrUntrustedKey)
    ));
}

#[test]
fn statement_unknown_and_duplicate_fields_are_rejected() {
    let fixture = AttestedBundleFixture::new();
    let canonical = fs::read_to_string(fixture.path().join(STATEMENT_FILE)).unwrap();
    let with_unknown = canonical.replacen("{", "{\"public_key\":\"untrusted\",", 1);
    fs::write(fixture.path().join(STATEMENT_FILE), with_unknown).unwrap();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::MalformedAttestation(_))
    ));

    let fixture = AttestedBundleFixture::new();
    let canonical = fs::read_to_string(fixture.path().join(STATEMENT_FILE)).unwrap();
    let with_duplicate = canonical.replacen("{", "{\"product\":\"attacker-selected-product\",", 1);
    fs::write(fixture.path().join(STATEMENT_FILE), with_duplicate).unwrap();
    assert!(matches!(
        fixture.verify(),
        Err(LinuxBundleError::MalformedAttestation(_))
    ));
}

#[test]
fn attestation_errors_do_not_disclose_inputs_or_key_material() {
    let fixture = AttestedBundleFixture::new();
    let sensitive_statement = b"statement-secret-user-data";
    fs::write(fixture.path().join(STATEMENT_FILE), sensitive_statement).unwrap();
    let error_text = fixture.verify().unwrap_err().to_string();

    assert!(!error_text.contains("statement-secret-user-data"));
    assert!(!error_text.contains("cognitiveos daemon bundle"));
    assert!(!error_text.contains(TEST_ONLY_KEY_ID));
    assert!(!error_text.contains("private"));
    assert!(!error_text.contains("public_key"));
    assert!(!error_text.contains("user-data"));
}

#[test]
fn attestation_failure_precedes_staging_and_preserves_active_state_and_user_data() {
    let fixture = AttestedBundleFixture::new();
    let deployment_root = fixture.path().join("deployment");
    let deployment = LinuxBundleDeployment::open(&deployment_root).unwrap();
    fs::write(deployment_root.join("active-version"), b"1.0.0\n").unwrap();
    let user_data = fixture.path().join("user-data.sqlite");
    fs::write(&user_data, b"preserve-this-user-data").unwrap();

    fs::write(fixture.path().join(SIGNATURE_FILE), b"malformed-signature").unwrap();
    assert!(fixture.verify().is_err());

    assert_eq!(
        deployment.active_version().unwrap().as_deref(),
        Some("1.0.0")
    );
    assert_eq!(fs::read(&user_data).unwrap(), b"preserve-this-user-data");
    assert_eq!(
        fs::read_dir(deployment_root.join("staged"))
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        fs::read_dir(deployment_root.join("versions"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn only_cryptographically_verified_value_can_be_staged_and_activated() {
    let fixture = AttestedBundleFixture::new();
    let verified_bundle = fixture.verify().unwrap();
    let deployment = LinuxBundleDeployment::open(fixture.path().join("deployment")).unwrap();

    let staged_directory = deployment
        .stage_verified_bundle(fixture.path(), &verified_bundle)
        .unwrap();
    assert!(staged_directory.is_dir());
    deployment
        .activate_after_health_check(&verified_bundle, |candidate| candidate.is_dir())
        .unwrap();
    assert_eq!(
        deployment.active_version().unwrap().as_deref(),
        Some("1.2.3")
    );
}

#[test]
fn artifact_tampering_after_verification_is_rejected_before_staging() {
    let fixture = AttestedBundleFixture::new();
    let verified_bundle = fixture.verify().unwrap();
    fs::write(
        fixture.path().join(&fixture.manifest.artifact_file),
        b"tampered after verification",
    )
    .unwrap();

    let deployment_root = tempfile::tempdir().unwrap();
    let deployment = LinuxBundleDeployment::open(deployment_root.path()).unwrap();
    assert!(matches!(
        deployment.stage_verified_bundle(fixture.path(), &verified_bundle),
        Err(LinuxBundleError::ArtifactDigestMismatch)
    ));
    assert!(!deployment_root.path().join("staged/1.2.3").exists());
}

#[cfg(unix)]
#[test]
fn bundle_file_symlinks_are_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = AttestedBundleFixture::new();
    let artifact_path = fixture.path().join(&fixture.manifest.artifact_file);
    let external_artifact = tempfile::NamedTempFile::new().unwrap();
    fs::write(external_artifact.path(), b"cognitiveos daemon bundle").unwrap();
    fs::remove_file(&artifact_path).unwrap();
    symlink(external_artifact.path(), artifact_path).unwrap();

    assert!(matches!(fixture.verify(), Err(LinuxBundleError::Io(_))));
}
