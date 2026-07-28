//! Cross-process lifecycle and interruption coverage for Linux bundle installation.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::linux_bundle::{
    ExpectedPiCompatibility, LinuxBundleError, LinuxBundleManifest, TrustedKeyInput,
    TrustedKeyStatus, TrustedKeyring, verify_linux_bundle,
};
use cognitive_runtime::linux_bundle_installation::install_linux_bundle;
#[cfg(feature = "test-fault-injection")]
use cognitive_runtime::{InstallFaultPoint, install_linux_bundle_with_fault_injection};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tar::{Builder as TarBuilder, Header as TarHeader};

const TEST_ONLY_PRIVATE_SIGNING_SEED: [u8; 32] = [0x4d; 32];
const TEST_ONLY_KEY_ID: &str = "p1t08-lifecycle-test-key";
const TEST_ONLY_KEYRING_VERSION: &str = "p1t08-lifecycle-test-keyring-v1";
const PRODUCT: &str = "cognitiveos-personal";
const PLATFORM: &str = "linux-x86_64";
const PI_VERSION: &str = "0.81.1";
const PI_INTEGRITY: &str = "sha512:pinned-pi-integrity";
const STATEMENT_FILE: &str = "attestation.statement.json";
const SIGNATURE_FILE: &str = "attestation.signature.json";
const KERNEL_SERVER_CONTENT: &[u8] = b"p1t08-lifecycle-kernel-server";
const USER_DATA_SENTINEL: &[u8] = b"p1t08-lifecycle-user-data-sentinel";
const CHILD_MODE_ENVIRONMENT: &str = "COGNITIVEOS_P1T08_CHILD_MODE";
const CHILD_BUNDLE_DIRECTORY_ENVIRONMENT: &str = "COGNITIVEOS_P1T08_BUNDLE_DIRECTORY";
const CHILD_DEPLOYMENT_ROOT_ENVIRONMENT: &str = "COGNITIVEOS_P1T08_DEPLOYMENT_ROOT";
const CHILD_READY_MARKER_ENVIRONMENT: &str = "COGNITIVEOS_P1T08_READY_MARKER";
const CHILD_RELEASE_MARKER_ENVIRONMENT: &str = "COGNITIVEOS_P1T08_RELEASE_MARKER";
const CHILD_HEALTH_MARKER_ENVIRONMENT: &str = "COGNITIVEOS_P1T08_HEALTH_MARKER";
const CHILD_WAIT_TIMEOUT: Duration = Duration::from_secs(20);
const INSTALLER_LEASE_FILE_PREFIX: &str = ".cognitiveos-personal-installer-lease-v1-";

struct SignedBundleFixture {
    temporary_directory: tempfile::TempDir,
    signing_key: SigningKey,
    manifest: LinuxBundleManifest,
    statement: Value,
}

impl SignedBundleFixture {
    fn new(version: &str) -> Self {
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

    fn deployment_root(&self, name: &str) -> PathBuf {
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
        fs::write(
            self.bundle_directory().join(SIGNATURE_FILE),
            serde_json::to_vec(&json!({
                "algorithm": "Ed25519",
                "key_id": TEST_ONLY_KEY_ID,
                "schema": "cognitiveos.personal.linux-bundle-signature",
                "schema_version": 1,
                "signature": URL_SAFE_NO_PAD.encode(detached_signature.to_bytes()),
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

fn runnable_archive_bytes() -> Vec<u8> {
    let gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar_builder = TarBuilder::new(gzip_encoder);
    let mut header = TarHeader::new_gnu();
    header.set_mode(0o755);
    header.set_size(KERNEL_SERVER_CONTENT.len() as u64);
    header.set_cksum();
    tar_builder
        .append_data(&mut header, "bin/kernel-server", KERNEL_SERVER_CONTENT)
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

fn expected_pi() -> ExpectedPiCompatibility {
    ExpectedPiCompatibility::new(PI_VERSION, PI_INTEGRITY)
}

fn trusted_keyring() -> TrustedKeyring {
    let signing_key = SigningKey::from_bytes(&TEST_ONLY_PRIVATE_SIGNING_SEED);
    TrustedKeyring::new(
        TEST_ONLY_KEYRING_VERSION,
        vec![TrustedKeyInput {
            key_id: TEST_ONLY_KEY_ID.to_owned(),
            algorithm: "Ed25519".to_owned(),
            public_key_base64url: URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes()),
            status: TrustedKeyStatus::Active,
        }],
    )
    .unwrap()
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
        deployment_root.join("user-data/personal.sqlite"),
        USER_DATA_SENTINEL,
    )
    .unwrap();
}

fn spawn_child(
    mode: &str,
    bundle_directory: &Path,
    deployment_root: &Path,
    ready_marker: Option<&Path>,
    release_marker: Option<&Path>,
    health_marker: Option<&Path>,
) -> Child {
    let mut command = Command::new(env::current_exe().unwrap());
    command
        .arg("--ignored")
        .arg("--exact")
        .arg("lifecycle_child_process_entrypoint")
        .arg("--nocapture")
        .env(CHILD_MODE_ENVIRONMENT, mode)
        .env(CHILD_BUNDLE_DIRECTORY_ENVIRONMENT, bundle_directory)
        .env(CHILD_DEPLOYMENT_ROOT_ENVIRONMENT, deployment_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(marker) = ready_marker {
        command.env(CHILD_READY_MARKER_ENVIRONMENT, marker);
    }
    if let Some(marker) = release_marker {
        command.env(CHILD_RELEASE_MARKER_ENVIRONMENT, marker);
    }
    if let Some(marker) = health_marker {
        command.env(CHILD_HEALTH_MARKER_ENVIRONMENT, marker);
    }
    command.spawn().unwrap()
}

fn wait_for_marker(marker: &Path, child: &mut Child, timeline: &str) {
    let deadline = Instant::now() + CHILD_WAIT_TIMEOUT;
    loop {
        if marker.is_file() {
            return;
        }
        if let Some(status) = child.try_wait().unwrap() {
            panic!("{timeline}: child exited before marker with {status}");
        }
        assert!(
            Instant::now() < deadline,
            "{timeline}: timed out waiting for deterministic marker"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_success(child: Child, timeline: &str) {
    let output = child.wait_with_output().unwrap();
    assert_child_success(output.status, &output.stdout, &output.stderr, timeline);
}

fn assert_child_success(status: ExitStatus, stdout: &[u8], stderr: &[u8], timeline: &str) {
    assert!(
        status.success(),
        "{timeline}: child failed with {status}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    );
}

fn installer_lease_files(parent_directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(parent_directory)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(INSTALLER_LEASE_FILE_PREFIX)
        })
        .map(|entry| entry.path())
        .collect()
}

#[test]
#[ignore = "invoked only as an isolated child process"]
fn lifecycle_child_process_entrypoint() {
    let mode = env::var(CHILD_MODE_ENVIRONMENT).expect("child mode must be provided");
    let bundle_directory = PathBuf::from(
        env::var_os(CHILD_BUNDLE_DIRECTORY_ENVIRONMENT)
            .expect("child bundle directory must be provided"),
    );
    let deployment_root = PathBuf::from(
        env::var_os(CHILD_DEPLOYMENT_ROOT_ENVIRONMENT)
            .expect("child deployment root must be provided"),
    );
    verify_linux_bundle(&bundle_directory, &expected_pi(), &trusted_keyring()).unwrap();

    match mode.as_str() {
        "hold" => {
            let ready_marker = PathBuf::from(
                env::var_os(CHILD_READY_MARKER_ENVIRONMENT).expect("ready marker must be provided"),
            );
            let release_marker = PathBuf::from(
                env::var_os(CHILD_RELEASE_MARKER_ENVIRONMENT)
                    .expect("release marker must be provided"),
            );
            install_linux_bundle(
                &bundle_directory,
                &deployment_root,
                &expected_pi(),
                &trusted_keyring(),
                |_| {
                    fs::write(&ready_marker, b"lease-held-and-staged").unwrap();
                    let deadline = Instant::now() + CHILD_WAIT_TIMEOUT;
                    while !release_marker.is_file() {
                        assert!(
                            Instant::now() < deadline,
                            "child timed out waiting for release marker"
                        );
                        thread::sleep(Duration::from_millis(10));
                    }
                    true
                },
            )
            .unwrap();
        }
        "expect-lease-held" => {
            let health_marker = PathBuf::from(
                env::var_os(CHILD_HEALTH_MARKER_ENVIRONMENT)
                    .expect("health marker must be provided"),
            );
            let result = install_linux_bundle(
                &bundle_directory,
                &deployment_root,
                &expected_pi(),
                &trusted_keyring(),
                |_| {
                    fs::write(&health_marker, b"unexpected-health").unwrap();
                    true
                },
            );
            assert!(matches!(
                result,
                Err(LinuxBundleError::InstallationLeaseHeld)
            ));
            assert!(!health_marker.exists());
        }
        "install" => {
            install_linux_bundle(
                &bundle_directory,
                &deployment_root,
                &expected_pi(),
                &trusted_keyring(),
                |_| true,
            )
            .unwrap();
        }
        other => panic!("unsupported child mode: {other}"),
    }
}

#[test]
fn same_deployment_root_allows_only_one_cross_process_installer() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("same-root-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let ready_marker = fixture.deployment_root("holder-ready");
    let release_marker = fixture.deployment_root("release-holder");
    let loser_health_marker = fixture.deployment_root("loser-health");

    let mut holder = spawn_child(
        "hold",
        fixture.bundle_directory(),
        &deployment_root,
        Some(&ready_marker),
        Some(&release_marker),
        None,
    );
    wait_for_marker(
        &ready_marker,
        &mut holder,
        "holder acquired lease and completed staging",
    );

    let contender = spawn_child(
        "expect-lease-held",
        fixture.bundle_directory(),
        &deployment_root,
        None,
        None,
        Some(&loser_health_marker),
    );
    wait_for_success(contender, "same-root contender must fail closed");
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert!(!loser_health_marker.exists());

    fs::write(&release_marker, b"release").unwrap();
    wait_for_success(holder, "holder completes after deterministic release");
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
    assert!(deployment_root.join("versions/2.0.0").is_dir());
}

#[test]
fn different_versions_remain_mutually_exclusive_per_deployment_root() {
    let fixture_v2 = SignedBundleFixture::new("2.0.0");
    let fixture_v3 = SignedBundleFixture::new("3.0.0");
    let deployment_parent = tempfile::tempdir().unwrap();
    let deployment_root = deployment_parent.path().join("shared-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let ready_marker = deployment_parent.path().join("holder-ready");
    let release_marker = deployment_parent.path().join("release-holder");
    let loser_health_marker = deployment_parent.path().join("loser-health");

    let mut holder = spawn_child(
        "hold",
        fixture_v2.bundle_directory(),
        &deployment_root,
        Some(&ready_marker),
        Some(&release_marker),
        None,
    );
    wait_for_marker(&ready_marker, &mut holder, "version 2 holder staged");

    let contender = spawn_child(
        "expect-lease-held",
        fixture_v3.bundle_directory(),
        &deployment_root,
        None,
        None,
        Some(&loser_health_marker),
    );
    wait_for_success(contender, "version 3 contender must observe root lease");
    assert!(!deployment_root.join("staged/3.0.0").exists());
    assert!(!loser_health_marker.exists());

    fs::write(&release_marker, b"release").unwrap();
    wait_for_success(holder, "version 2 holder completes");
}

#[test]
fn different_deployment_roots_do_not_block_each_other() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let first_root = fixture.deployment_root("first-deployment");
    let second_root = fixture.deployment_root("second-deployment");
    let ready_marker = fixture.deployment_root("first-holder-ready");
    let release_marker = fixture.deployment_root("release-first-holder");

    let mut holder = spawn_child(
        "hold",
        fixture.bundle_directory(),
        &first_root,
        Some(&ready_marker),
        Some(&release_marker),
        None,
    );
    wait_for_marker(&ready_marker, &mut holder, "first root holder staged");

    let independent_installer = spawn_child(
        "install",
        fixture.bundle_directory(),
        &second_root,
        None,
        None,
        None,
    );
    wait_for_success(
        independent_installer,
        "second root install must not wait for first root",
    );
    assert_eq!(
        fs::read_to_string(second_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );

    fs::write(&release_marker, b"release").unwrap();
    wait_for_success(holder, "first root holder completes");
}

#[test]
fn successful_process_releases_lease_for_the_next_process() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("sequential-deployment");

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "first process installation",
    );
    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "same-version second process installation",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}

#[test]
fn verifier_failure_creates_neither_deployment_nor_lease_mutation() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("invalid-deployment");
    fs::write(
        fixture.bundle_directory().join(SIGNATURE_FILE),
        b"{\"invalid\":true}",
    )
    .unwrap();
    let health_calls = std::cell::Cell::new(0_u32);

    let result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring(),
        |_| {
            health_calls.set(health_calls.get() + 1);
            true
        },
    );

    assert!(matches!(
        result,
        Err(LinuxBundleError::MalformedAttestation(_))
    ));
    assert_eq!(health_calls.get(), 0);
    assert!(!deployment_root.exists());
    assert!(installer_lease_files(fixture.bundle_directory()).is_empty());
}

#[test]
fn missing_lease_parent_fails_without_creating_deployment_state() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let absent_parent = fixture.deployment_root("absent-lease-parent");
    let deployment_root = absent_parent.join("deployment");
    let health_calls = std::cell::Cell::new(0_u32);

    let result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring(),
        |_| {
            health_calls.set(health_calls.get() + 1);
            true
        },
    );

    assert!(matches!(result, Err(LinuxBundleError::Io(_))));
    assert_eq!(health_calls.get(), 0);
    assert!(!absent_parent.exists());
    assert!(installer_lease_files(fixture.bundle_directory()).is_empty());
}

#[test]
fn staging_failure_releases_lease_for_a_successor_process() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("staging-failure-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let invalid_staging_leaf = deployment_root.join("staged/2.0.0");
    fs::write(&invalid_staging_leaf, b"not-a-staging-directory").unwrap();
    let health_calls = std::cell::Cell::new(0_u32);

    let failed_result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring(),
        |_| {
            health_calls.set(health_calls.get() + 1);
            true
        },
    );

    assert!(matches!(failed_result, Err(LinuxBundleError::Io(_))));
    assert_eq!(health_calls.get(), 0);
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    fs::remove_file(invalid_staging_leaf).unwrap();

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "successor after staging failure",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}

#[test]
fn persistent_lock_file_and_stale_contents_do_not_represent_a_live_lease() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("stale-lock-file-deployment");

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "first installation creates stable empty lock file",
    );
    let lease_files = installer_lease_files(fixture.bundle_directory());
    assert_eq!(lease_files.len(), 1);
    assert_eq!(fs::metadata(&lease_files[0]).unwrap().len(), 0);

    fs::write(&lease_files[0], b"stale-content-has-no-ownership-authority").unwrap();
    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "installation ignores stale lock-file contents",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}

#[test]
fn health_failure_releases_lease_and_preserves_inspectable_staging() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("health-failure-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");

    let failed_result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring(),
        |_| false,
    );
    assert!(matches!(
        failed_result,
        Err(LinuxBundleError::HealthCheckFailed)
    ));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert!(deployment_root.join("staged/2.0.0").is_dir());
    assert_eq!(
        fs::read(deployment_root.join("user-data/personal.sqlite")).unwrap(),
        USER_DATA_SENTINEL
    );

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "installation after health failure",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}

#[test]
fn activation_failure_releases_lease_without_torn_pointer_or_receipt() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("activation-failure-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");

    let failed_result = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring(),
        |_| {
            fs::create_dir(deployment_root.join("active-version.new")).unwrap();
            true
        },
    );
    assert!(matches!(failed_result, Err(LinuxBundleError::Io(_))));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert!(deployment_root.join("versions/2.0.0").is_dir());
    fs::remove_dir(deployment_root.join("active-version.new")).unwrap();

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "installation after activation failure",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}

#[test]
fn operating_system_process_termination_releases_live_lease() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("terminated-holder-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let ready_marker = fixture.deployment_root("terminated-holder-ready");
    let never_release_marker = fixture.deployment_root("never-release-holder");
    let live_holder_health_marker = fixture.deployment_root("live-holder-contender-health");

    let mut holder = spawn_child(
        "hold",
        fixture.bundle_directory(),
        &deployment_root,
        Some(&ready_marker),
        Some(&never_release_marker),
        None,
    );
    wait_for_marker(
        &ready_marker,
        &mut holder,
        "holder staged before termination",
    );

    wait_for_success(
        spawn_child(
            "expect-lease-held",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            Some(&live_holder_health_marker),
        ),
        "live holder must keep lease",
    );
    assert!(!live_holder_health_marker.exists());
    holder.kill().unwrap();
    let terminated_status = holder.wait().unwrap();
    assert!(!terminated_status.success());
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "successor after operating-system termination",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}

#[test]
fn lease_failure_debug_output_contains_no_sensitive_material() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("redaction-deployment");
    let ready_marker = fixture.deployment_root("redaction-holder-ready");
    let release_marker = fixture.deployment_root("release-redaction-holder");
    let health_marker = fixture.deployment_root("redaction-contender-health");
    let mut holder = spawn_child(
        "hold",
        fixture.bundle_directory(),
        &deployment_root,
        Some(&ready_marker),
        Some(&release_marker),
        None,
    );
    wait_for_marker(&ready_marker, &mut holder, "redaction holder staged");

    let error = install_linux_bundle(
        fixture.bundle_directory(),
        &deployment_root,
        &expected_pi(),
        &trusted_keyring(),
        |_| {
            fs::write(&health_marker, b"unexpected-health").unwrap();
            true
        },
    )
    .unwrap_err();
    assert!(matches!(error, LinuxBundleError::InstallationLeaseHeld));
    let error_debug = format!("{error:?}");
    let error_display = error.to_string();
    let lease_path = installer_lease_files(fixture.bundle_directory())
        .into_iter()
        .next()
        .expect("live holder must have created its stable lease file");
    let sensitive_values = [
        URL_SAFE_NO_PAD.encode(TEST_ONLY_PRIVATE_SIGNING_SEED),
        URL_SAFE_NO_PAD.encode(
            SigningKey::from_bytes(&TEST_ONLY_PRIVATE_SIGNING_SEED)
                .verifying_key()
                .to_bytes(),
        ),
        String::from_utf8_lossy(KERNEL_SERVER_CONTENT).into_owned(),
        String::from_utf8_lossy(USER_DATA_SENTINEL).into_owned(),
        deployment_root.to_string_lossy().into_owned(),
        lease_path.to_string_lossy().into_owned(),
        std::process::id().to_string(),
    ];
    for sensitive_value in sensitive_values {
        assert!(!error_debug.contains(&sensitive_value));
        assert!(!error_display.contains(&sensitive_value));
    }
    assert!(!health_marker.exists());

    fs::write(&release_marker, b"release").unwrap();
    wait_for_success(holder, "redaction holder completes");
}

#[cfg(feature = "test-fault-injection")]
#[test]
fn deterministic_fault_points_release_lease_and_preserve_pointer_boundaries() {
    let fault_points = [
        InstallFaultPoint::LeaseAcquiredBeforeDeploymentOpen,
        InstallFaultPoint::DeploymentOpenedBeforeStage,
        InstallFaultPoint::StageCompletedBeforeHealth,
        InstallFaultPoint::HealthSucceededBeforeActivation,
        InstallFaultPoint::ActivationCompletedBeforeReceiptConfirmation,
    ];

    for fault_point in fault_points {
        let fixture = SignedBundleFixture::new("2.0.0");
        let deployment_root = fixture.deployment_root("fault-deployment");
        prepare_existing_installation(&deployment_root, "1.0.0");
        let health_calls = std::cell::Cell::new(0_u32);
        let result = install_linux_bundle_with_fault_injection(
            fixture.bundle_directory(),
            &deployment_root,
            &expected_pi(),
            &trusted_keyring(),
            fault_point,
            |_| {
                health_calls.set(health_calls.get() + 1);
                true
            },
        );

        assert!(matches!(result, Err(LinuxBundleError::FaultInjected(_))));
        assert_eq!(
            fs::read_to_string(deployment_root.join("active-version")).unwrap(),
            if fault_point == InstallFaultPoint::ActivationCompletedBeforeReceiptConfirmation {
                "2.0.0\n"
            } else {
                "1.0.0\n"
            }
        );
        assert_eq!(
            health_calls.get(),
            if matches!(
                fault_point,
                InstallFaultPoint::HealthSucceededBeforeActivation
                    | InstallFaultPoint::ActivationCompletedBeforeReceiptConfirmation
            ) {
                1
            } else {
                0
            }
        );

        wait_for_success(
            spawn_child(
                "install",
                fixture.bundle_directory(),
                &deployment_root,
                None,
                None,
                None,
            ),
            "successor after deterministic fault",
        );
        assert_eq!(
            fs::read_to_string(deployment_root.join("active-version")).unwrap(),
            "2.0.0\n"
        );
    }
}

#[cfg(feature = "test-fault-injection")]
#[test]
fn panic_after_lease_acquisition_does_not_permanently_block_installation() {
    let fixture = SignedBundleFixture::new("2.0.0");
    let deployment_root = fixture.deployment_root("panic-fault-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");

    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        install_linux_bundle_with_fault_injection(
            fixture.bundle_directory(),
            &deployment_root,
            &expected_pi(),
            &trusted_keyring(),
            InstallFaultPoint::PanicAfterLeaseAcquired,
            |_| true,
        )
    }));
    assert!(panic_result.is_err());
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );

    wait_for_success(
        spawn_child(
            "install",
            fixture.bundle_directory(),
            &deployment_root,
            None,
            None,
            None,
        ),
        "successor after panic fault",
    );
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
}
