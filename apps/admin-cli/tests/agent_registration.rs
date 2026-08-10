#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, PackageInstallRequest, PiInstallationRootActivationRequest,
    acquire_official_pi_durable, activate_official_pi_root_durable, package_artifact_digest,
    package_sha256_digest, package_sri_sha512,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

struct CliResult {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_cli(args: &[&str]) -> CliResult {
    let output = Command::new(env!("CARGO_BIN_EXE_admin-cli"))
        .args(args)
        .output()
        .expect("admin-cli runs");
    CliResult {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn write_session(dir: &Path) -> PathBuf {
    let path = dir.join("session.json");
    let value = json!({
        "schema_version": "cognitiveos.privileged-management-session/0.1",
        "session_id": "pms-register-01",
        "object_version": 1,
        "management_domain": "cognitiveos.management",
        "session_authority": "authority://tenant-a/management-authority",
        "human_principal": "principal://tenant-a/verified-operator",
        "actor_chain_digest": format!("sha256:{}", "ab12".repeat(16)),
        "authentication_context_ref": "authn://tenant-a/webauthn-9",
        "activity_context_ref": "activity://tenant-a/agent-register",
        "scope": {
            "domains": ["cognitiveos.management"],
            "actions": ["agent.register"],
            "resources": ["agent-installation://"]
        },
        "risk_ceiling": "R1",
        "policy_version": 1,
        "revocation_epoch": 41,
        "issued_at": "2026-07-24T12:00:00Z",
        "last_activity_at": "2026-07-24T12:00:00Z",
        "idle_timeout_seconds": 3600,
        "absolute_expires_at": "2030-01-01T00:00:00Z",
        "state": "active",
        "session_digest": format!("sha256:{}", "cd34".repeat(16)),
        "authority_signature": "sig-register-fixture-0001"
    });
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    path
}

fn prepare_active_official_root(database: &Path) -> u64 {
    let authority = DurableInstallationAuthority::open(database).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let artifact = b"staged-official-pi-package".to_vec();
    let dependency_lock = b"locked-dependencies".to_vec();
    let artifact_digest = package_artifact_digest(&artifact).unwrap();
    let lock_digest = package_artifact_digest(&dependency_lock).unwrap();
    let request = OfficialPiAcquisitionRequest {
        install: PackageInstallRequest {
            package_id: format!("pkg://{OFFICIAL_PI_PACKAGE}@{OFFICIAL_PI_VERSION}"),
            publisher: OFFICIAL_PI_PACKAGE.to_owned(),
            package_version: OFFICIAL_PI_VERSION.to_owned(),
            artifact: artifact.clone(),
            declared_artifact_digest: artifact_digest,
            signature_ref: "official-lock".to_owned(),
            provenance_ref: OFFICIAL_NPM_ORIGIN.to_owned(),
            adapter_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_owned(),
            sandbox_digest: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_owned(),
            compatibility_digest:
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_owned(),
            lockfile_digest: lock_digest.clone(),
            expected_adapter_digest:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            expected_sandbox_digest:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            expected_compatibility_digest:
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_owned(),
        },
        registry_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        resolved_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        sri_sha512: package_sri_sha512(&artifact),
        declared_package_sha256: package_sha256_digest(&artifact),
        dependency_lock,
        declared_dependency_lock_digest: lock_digest,
        node_version: "22.19.0".to_owned(),
        signed_acquisition_lock_ref: "attestation://pi/lock-01".to_owned(),
    };
    acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();
    activate_official_pi_root_durable(
        &manager,
        &PiInstallationRootActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            package_ref: request.install.package_id,
            expected_activation_version: None,
            compatibility_accepted: true,
            health_accepted: true,
        },
    )
    .unwrap()
    .activation_version()
}

#[test]
fn management_register_persists_inactive_instance_without_sidecar() {
    let directory = tempfile::tempdir().unwrap();
    let session = write_session(directory.path());
    let database = directory.path().join("install.db");
    let activation_version = prepare_active_official_root(&database);
    let version = activation_version.to_string();

    let result = run_cli(&[
        "register",
        "--session",
        session.to_str().unwrap(),
        "--installation-store",
        database.to_str().unwrap(),
        "--installation-root",
        OFFICIAL_PI_INSTALLATION_ROOT,
        "--expected-activation-version",
        &version,
        "--adapter-digest",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--protocol-digest",
        "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "--policy-digest",
        "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    ]);

    assert_eq!(
        result.code, 0,
        "stdout: {} stderr: {}",
        result.stdout, result.stderr
    );
    let value: Value = serde_json::from_str(result.stdout.trim()).unwrap();
    assert_eq!(value["lifecycle_state"], "registered");
    assert_eq!(value["fencing_epoch"], 1);
    assert_eq!(value["capability_grants"], 0);
    assert_eq!(value["sidecar_sessions"], 0);
    assert_eq!(value["effects_created"], 0);
    assert_eq!(value["tasks_completed"], 0);
    assert!(value["registration_id"].as_str().unwrap().len() > 8);
    assert!(value["instance_id"].as_str().unwrap().len() > 8);
}
