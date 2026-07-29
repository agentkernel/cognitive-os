//! Failure-first coverage for the P1-T08 user-service lifecycle transaction.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use cognitive_runtime::{
    ExpectedPiCompatibility, LinuxBundleManifest, LinuxBundleServiceController,
    LinuxBundleServiceError, LinuxBundleServiceReceipt, LinuxBundleSingleServiceController,
    PersonalUserServiceUnitKind, SystemdUserServiceController, TrustedKeyInput, TrustedKeyStatus,
    TrustedKeyring, install_linux_bundle_service, probe_personal_health,
    render_personal_user_service_unit, write_rendered_personal_user_service_unit,
};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{Compression, write::GzEncoder};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tar::{Builder as TarBuilder, Header as TarHeader};

#[test]
fn service_lifecycle_api_is_explicit_and_separate_from_offline_installation() {
    let _controller_type = std::any::type_name::<dyn LinuxBundleServiceController>();
    let _receipt_type = std::any::type_name::<LinuxBundleServiceReceipt>();
}

#[test]
fn checked_in_user_unit_is_unrendered_and_rejected_before_systemctl() {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let template = repository_root.join("deploy/linux/cognitiveos-personal.service");
    let source = fs::read_to_string(&template).unwrap();
    assert!(source.contains("@COGNITIVEOS_RELEASE_ROOT@"));
    assert!(source.contains("--personal --bind 127.0.0.1:"));
    for forbidden in ["sudo", "User=root", "systemctl", "sh -c", "eval "] {
        assert!(
            !source.contains(forbidden),
            "unexpected unit fragment: {forbidden}"
        );
    }
    let mut controller = SystemdUserServiceController::new(
        tempfile::tempdir().unwrap().path(),
        template,
        "127.0.0.1:48181".parse().unwrap(),
    )
    .unwrap();
    assert!(
        controller
            .start_candidate("2.0.0", std::path::Path::new("."))
            .is_err()
    );
}

#[test]
fn rendered_units_use_only_fixed_candidate_and_active_inputs() {
    let deployment_root = tempfile::tempdir().unwrap();
    let candidate_unit = render_personal_user_service_unit(
        PersonalUserServiceUnitKind::Candidate,
        deployment_root.path(),
        "2.0.0",
    )
    .unwrap();
    let active_unit = render_personal_user_service_unit(
        PersonalUserServiceUnitKind::Active,
        deployment_root.path(),
        "2.0.0",
    )
    .unwrap();

    assert!(candidate_unit.contains("staged/2.0.0/bin/kernel-server"));
    assert!(candidate_unit.contains("--bind 127.0.0.1:48182"));
    assert!(candidate_unit.contains("runtime/candidate"));
    assert!(active_unit.contains("versions/2.0.0/bin/kernel-server"));
    assert!(active_unit.contains("--bind 127.0.0.1:48181"));
    assert!(!active_unit.contains("--runtime-root"));
    for unit in [&candidate_unit, &active_unit] {
        assert!(!unit.contains('@'));
        assert!(!unit.contains("sudo"));
        assert!(!unit.contains("systemctl"));
        assert!(!unit.contains("sh -c"));
    }

    for unsafe_version in ["../2.0.0", "2.0.0/escape", "2.0.0\nExecStart=/bin/sh"] {
        assert!(
            render_personal_user_service_unit(
                PersonalUserServiceUnitKind::Candidate,
                deployment_root.path(),
                unsafe_version,
            )
            .is_err()
        );
    }

    let escaped_path = tempfile::tempdir().unwrap().path().join("space % value");
    let escaped_unit = render_personal_user_service_unit(
        PersonalUserServiceUnitKind::Active,
        &escaped_path,
        "2.0.0",
    )
    .unwrap();
    assert!(escaped_unit.contains("space\\x20%%\\x20value"));
}

#[test]
fn rendered_unit_publication_uses_fixed_names_and_never_leaves_a_partial_file() {
    let deployment_root = tempfile::tempdir().unwrap();
    let unit_root = tempfile::tempdir().unwrap();
    let unit_path = write_rendered_personal_user_service_unit(
        PersonalUserServiceUnitKind::Candidate,
        unit_root.path(),
        deployment_root.path(),
        "2.0.0",
    )
    .unwrap();

    assert_eq!(
        unit_path.file_name().unwrap(),
        "cognitiveos-personal-candidate.service"
    );
    let unit_contents = fs::read_to_string(&unit_path).unwrap();
    assert!(unit_contents.contains("staged/2.0.0/bin/kernel-server"));
    assert!(
        !unit_root
            .path()
            .join(".cognitiveos-personal-candidate.service-0.tmp")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn fixture_controller_publishes_candidate_then_reloads_before_fixed_start() {
    use std::os::unix::fs::PermissionsExt;

    let fixture_root = tempfile::tempdir().unwrap();
    let deployment_root = fixture_root.path().join("deployment");
    let candidate_executable = deployment_root.join("staged/2.0.0/bin/kernel-server");
    fs::create_dir_all(candidate_executable.parent().unwrap()).unwrap();
    fs::write(&candidate_executable, b"fixture executable").unwrap();
    let action_log = fixture_root.path().join("systemctl-actions.log");
    let fake_systemctl = fixture_root.path().join("fake-systemctl");
    fs::write(
        &fake_systemctl,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n",
            action_log.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&fake_systemctl, fs::Permissions::from_mode(0o700)).unwrap();

    let mut controller = SystemdUserServiceController::new_fixture(
        &deployment_root,
        fixture_root.path().join("units"),
        &fake_systemctl,
        "127.0.0.1:48181".parse().unwrap(),
    )
    .unwrap();

    controller
        .start_candidate("2.0.0", candidate_executable.parent().unwrap())
        .unwrap();
    let candidate_unit = fs::read_to_string(
        fixture_root
            .path()
            .join("units/cognitiveos-personal-candidate.service"),
    )
    .unwrap();
    assert!(candidate_unit.contains("staged/2.0.0/bin/kernel-server"));
    assert!(candidate_unit.contains("--bind 127.0.0.1:48182"));
    assert!(
        !fixture_root
            .path()
            .join("units/cognitiveos-personal.service")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(action_log).unwrap(),
        "--user --no-ask-password --no-pager daemon-reload\n--user --no-ask-password --no-pager start cognitiveos-personal-candidate.service\n"
    );
}

#[cfg(unix)]
#[test]
fn fixture_controller_publishes_only_the_canonical_unit_before_fixed_restart() {
    use std::os::unix::fs::PermissionsExt;

    let fixture_root = tempfile::tempdir().unwrap();
    let deployment_root = fixture_root.path().join("deployment");
    let active_executable = deployment_root.join("versions/2.0.0/bin/kernel-server");
    fs::create_dir_all(active_executable.parent().unwrap()).unwrap();
    fs::write(&active_executable, b"fixture executable").unwrap();
    let action_log = fixture_root.path().join("systemctl-actions.log");
    let fake_systemctl = fixture_root.path().join("fake-systemctl");
    fs::write(
        &fake_systemctl,
        format!(
            "#!/bin/sh\nprintf '%s\n' \"$*\" >> '{}'\n",
            action_log.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&fake_systemctl, fs::Permissions::from_mode(0o700)).unwrap();

    let unit_directory = fixture_root.path().join("units");
    let mut controller = SystemdUserServiceController::new_fixture(
        &deployment_root,
        &unit_directory,
        &fake_systemctl,
        "127.0.0.1:48181".parse().unwrap(),
    )
    .unwrap();

    controller.publish_active_unit("2.0.0").unwrap();
    controller.restart_active_service().unwrap();

    let active_unit =
        fs::read_to_string(unit_directory.join("cognitiveos-personal.service")).unwrap();
    assert!(active_unit.contains("versions/2.0.0/bin/kernel-server"));
    assert!(active_unit.contains("--bind 127.0.0.1:48181"));
    assert!(!active_unit.contains("--runtime-root"));
    assert!(
        !unit_directory
            .join("cognitiveos-personal-candidate.service")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(action_log).unwrap(),
        "--user --no-ask-password --no-pager daemon-reload\n--user --no-ask-password --no-pager restart cognitiveos-personal.service\n"
    );
}

#[cfg(unix)]
#[test]
fn systemctl_timeout_kills_pipe_holding_descendants_before_releasing_the_controller() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::Instant;

    let fixture_root = tempfile::tempdir().unwrap();
    let fake_systemctl = fixture_root.path().join("fake-systemctl");
    fs::write(
        &fake_systemctl,
        "#!/bin/sh\n(sleep 30) &\nwhile :; do sleep 30; done\n",
    )
    .unwrap();
    fs::set_permissions(&fake_systemctl, fs::Permissions::from_mode(0o700)).unwrap();
    let mut controller = SystemdUserServiceController::new_fixture_with_command_timeout(
        fixture_root.path().join("deployment"),
        fixture_root.path().join("units"),
        &fake_systemctl,
        "127.0.0.1:48181".parse().unwrap(),
        Duration::from_millis(100),
    )
    .unwrap();

    let started_at = Instant::now();
    let result = controller.restart_active_service();

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::ServiceCommandTimedOut)
    ));
    assert!(
        started_at.elapsed() < Duration::from_secs(2),
        "timeout must include pipe-holder cleanup"
    );
}

#[test]
fn bounded_loopback_health_requires_the_exact_liveness_contract() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || respond_once(listener, valid_health_response()));

    assert!(probe_personal_health(address, Duration::from_secs(1)).is_ok());
    server.join().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        respond_once(
            listener,
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}".to_owned(),
        )
    });

    assert!(probe_personal_health(address, Duration::from_millis(250)).is_err());
    server.join().unwrap();
}

#[test]
fn candidate_health_failure_stops_candidate_and_restores_previous_service_and_pointer() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller = RecordingController::with_outcomes([
        Ok(()),
        Err(LinuxBundleServiceError::CandidateHealthFailed),
    ]);

    let result = install_linux_bundle_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    );

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::CandidateHealthFailed)
    ));
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "1.0.0\n"
    );
    assert_eq!(
        controller.actions,
        [
            "start-candidate:2.0.0",
            "candidate-health:2.0.0",
            "stop-candidate:2.0.0",
            "start-active:1.0.0",
            "confirm-active:1.0.0"
        ]
    );
    assert!(deployment_root.join("staged/2.0.0").is_dir());
}

#[test]
fn first_install_failure_clears_pointer_without_inventing_a_rollback_target() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture.temporary_directory.path().join("first-deployment");
    let mut controller =
        RecordingController::with_outcomes([Err(LinuxBundleServiceError::CandidateStartFailed)]);

    let result = install_linux_bundle_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    );

    assert!(matches!(
        result,
        Err(LinuxBundleServiceError::CandidateStartFailed)
    ));
    assert!(!deployment_root.join("active-version").exists());
    assert_eq!(controller.actions, ["start-candidate:2.0.0"]);
}

#[test]
fn rollback_restart_failure_is_reported_without_a_success_receipt() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture
        .temporary_directory
        .path()
        .join("rollback-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller = RecordingController::with_outcomes([
        Ok(()),
        Err(LinuxBundleServiceError::CandidateHealthFailed),
        Ok(()),
        Err(LinuxBundleServiceError::CandidateStartFailed),
    ]);

    let result = install_linux_bundle_service(
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

#[test]
fn upgrade_stops_candidate_before_starting_and_confirming_canonical_active_service() {
    let fixture = signed_bundle("2.0.0");
    let deployment_root = fixture
        .temporary_directory
        .path()
        .join("upgrade-deployment");
    prepare_existing_installation(&deployment_root, "1.0.0");
    let mut controller =
        RecordingController::with_outcomes([Ok(()), Ok(()), Ok(()), Ok(()), Ok(())]);

    let receipt = install_linux_bundle_service(
        fixture.temporary_directory.path(),
        &deployment_root,
        &expected_pi(),
        &fixture.keyring,
        &mut controller,
    )
    .unwrap();

    assert_eq!(receipt.resulting_active_version, "2.0.0");
    assert_eq!(
        fs::read_to_string(deployment_root.join("active-version")).unwrap(),
        "2.0.0\n"
    );
    assert_eq!(
        controller.actions,
        [
            "start-candidate:2.0.0",
            "candidate-health:2.0.0",
            "stop-candidate:2.0.0",
            "start-active:2.0.0",
            "confirm-active:2.0.0"
        ]
    );
}

fn respond_once(listener: TcpListener, response: String) {
    let (mut stream, _) = listener.accept().unwrap();
    let mut request = [0_u8; 512];
    let _ = stream.read(&mut request).unwrap();
    stream.write_all(response.as_bytes()).unwrap();
}

fn valid_health_response() -> String {
    let body = r#"{"authority_side_effects":false,"profile_claim":"not-claimed","readiness_claim":"not-claimed","schema_version":1,"status":"ok","surface":"personal-health"}"#;
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

const KERNEL_SERVER_CONTENT: &[u8] = b"p1t08-service-lifecycle-kernel-server";

struct BundleFixture {
    temporary_directory: tempfile::TempDir,
    keyring: TrustedKeyring,
}

fn signed_bundle(version: &str) -> BundleFixture {
    let temporary_directory = tempfile::tempdir().unwrap();
    let signing_key = SigningKey::from_bytes(&[0x61; 32]);
    let artifact_bytes = runnable_archive_bytes();
    let manifest = LinuxBundleManifest {
        schema_version: 1,
        product: "cognitiveos-personal".to_owned(),
        platform: "linux-x86_64".to_owned(),
        version: version.to_owned(),
        artifact_file: "bundle.tar.gz".to_owned(),
        artifact_sha256: format!("sha256:{:x}", Sha256::digest(&artifact_bytes)),
        attestation_reference: format!("https://example.invalid/{version}"),
        attestation_statement_file: "statement.json".to_owned(),
        attestation_signature_file: "signature.json".to_owned(),
        pi_version: "0.81.1".to_owned(),
        pi_integrity: "sha512:test-pin".to_owned(),
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
    fs::write(
        temporary_directory.path().join("bundle.tar.gz"),
        artifact_bytes,
    )
    .unwrap();
    fs::write(
        temporary_directory.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        temporary_directory.path().join("statement.json"),
        statement_bytes,
    )
    .unwrap();
    fs::write(
        temporary_directory.path().join("signature.json"),
        serde_json::to_vec(&json!({
            "algorithm": "Ed25519",
            "key_id": "service-test-key",
            "schema": "cognitiveos.personal.linux-bundle-signature",
            "schema_version": 1,
            "signature": URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        }))
        .unwrap(),
    )
    .unwrap();
    let keyring = TrustedKeyring::new(
        "service-test-keyring",
        vec![TrustedKeyInput {
            key_id: "service-test-key".to_owned(),
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

fn expected_pi() -> ExpectedPiCompatibility {
    ExpectedPiCompatibility::new("0.81.1", "sha512:test-pin")
}

fn prepare_existing_installation(deployment_root: &std::path::Path, version: &str) {
    fs::create_dir_all(deployment_root.join("versions").join(version)).unwrap();
    fs::create_dir_all(deployment_root.join("staged")).unwrap();
    fs::create_dir_all(deployment_root.join("user-data")).unwrap();
    fs::write(
        deployment_root.join("active-version"),
        format!("{version}\n"),
    )
    .unwrap();
    fs::write(
        deployment_root.join("user-data/sentinel"),
        b"private-user-data",
    )
    .unwrap();
}

struct RecordingController {
    actions: Vec<String>,
    outcomes: VecDeque<Result<(), LinuxBundleServiceError>>,
}

impl RecordingController {
    fn with_outcomes(
        outcomes: impl IntoIterator<Item = Result<(), LinuxBundleServiceError>>,
    ) -> Self {
        Self {
            actions: Vec::new(),
            outcomes: outcomes.into_iter().collect(),
        }
    }

    fn record(&mut self, action: &str, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.actions.push(format!("{action}:{version}"));
        self.outcomes.pop_front().unwrap_or(Ok(()))
    }
}

impl LinuxBundleServiceController for RecordingController {
    fn start_candidate(
        &mut self,
        version: &str,
        _candidate_directory: &std::path::Path,
    ) -> Result<(), LinuxBundleServiceError> {
        self.record("start-candidate", version)
    }
    fn stop_candidate(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.record("stop-candidate", version)
    }
    fn confirm_candidate_health(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.record("candidate-health", version)
    }
    fn start_active(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.record("start-active", version)
    }
    fn confirm_active(&mut self, version: &str) -> Result<(), LinuxBundleServiceError> {
        self.record("confirm-active", version)
    }
}
