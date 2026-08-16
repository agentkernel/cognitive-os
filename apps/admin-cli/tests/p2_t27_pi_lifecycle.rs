//! P2-T27/D02 current-revision public `admin-cli` Pi lifecycle.
//!
//! Reuses the P5-T01/T02/T05 durable install/register/activate/pause/resume/
//! upgrade/rollback/stop/uninstall/recover stack. Does not invent a second
//! lifecycle. Never asserts secret material.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_runtime::{OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION};
use cognitive_store::SqliteAuthorityStore;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

const ADAPTER: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SANDBOX: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const COMPAT: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const PROTOCOL: &str = "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const POLICY: &str = "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

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

fn official_package_ref() -> String {
    format!("pkg://{OFFICIAL_PI_PACKAGE}@{OFFICIAL_PI_VERSION}")
}

fn write_session(dir: &Path) -> PathBuf {
    let path = dir.join("session.json");
    let value = json!({
        "schema_version": "cognitiveos.privileged-management-session/0.1",
        "session_id": "pms_p2-t27-lifecycle-01",
        "object_version": 1,
        "management_domain": "cognitiveos.management",
        "session_authority": "authority://tenant-a/management-authority",
        "human_principal": "principal://tenant-a/verified-operator",
        "actor_chain_digest": format!("sha256:{}", "ab12".repeat(16)),
        "authentication_context_ref": "authn://tenant-a/webauthn-9",
        "activity_context_ref": "activity://tenant-a/p2-t27-lifecycle",
        "scope": {
            "domains": ["cognitiveos.management"],
            "actions": [
                "agent.install",
                "agent.register",
                "agent.activate",
                "agent.pause",
                "agent.resume",
                "agent.stop",
                "agent.recover",
                "agent.health",
                "agent.rollback",
                "agent.uninstall"
            ],
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
        "authority_signature": "sig-p2-t27-lifecycle-0001"
    });
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    path
}

fn write_staged(dir: &Path) -> (PathBuf, PathBuf) {
    let artifact = dir.join("pi.tgz");
    let lock = dir.join("deps.lock");
    std::fs::write(&artifact, b"staged-official-pi-package").unwrap();
    std::fs::write(&lock, b"locked-dependencies").unwrap();
    (artifact, lock)
}

fn assert_ok(result: &CliResult, context: &str) -> Value {
    assert_eq!(
        result.code, 0,
        "{context} stdout={} stderr={}",
        result.stdout, result.stderr
    );
    serde_json::from_str(result.stdout.trim()).unwrap()
}

fn error_code(result: &CliResult) -> String {
    let parsed: Value = serde_json::from_str(result.stderr.trim()).unwrap_or(json!({}));
    parsed["error"]["code"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

struct Harness {
    session: PathBuf,
    install_db: PathBuf,
    authority_db: PathBuf,
    artifact: PathBuf,
    lock: PathBuf,
}

impl Harness {
    fn new(dir: &Path) -> Self {
        let install_db = dir.join("install.db");
        let authority_db = dir.join("authority.db");
        SqliteAuthorityStore::open(&authority_db).unwrap();
        let (artifact, lock) = write_staged(dir);
        Self {
            session: write_session(dir),
            install_db,
            authority_db,
            artifact,
            lock,
        }
    }

    fn session(&self) -> &str {
        self.session.to_str().unwrap()
    }

    fn install_db(&self) -> &str {
        self.install_db.to_str().unwrap()
    }

    fn authority_db(&self) -> &str {
        self.authority_db.to_str().unwrap()
    }

    fn official_install(&self) -> CliResult {
        let package = official_package_ref();
        run_cli(&[
            "install",
            "--mode",
            "official",
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--package-id",
            &package,
            "--staged-artifact",
            self.artifact.to_str().unwrap(),
            "--dependency-lock",
            self.lock.to_str().unwrap(),
            "--node-version",
            "22.19.0",
            "--signed-lock-ref",
            "attestation://pi/lock-01",
            "--adapter-digest",
            ADAPTER,
            "--sandbox-digest",
            SANDBOX,
            "--compatibility-digest",
            COMPAT,
        ])
    }

    fn activate_root(&self, expected: Option<u64>) -> CliResult {
        let package = official_package_ref();
        let expected_text = expected.map(|value| value.to_string());
        let mut args = vec![
            "activate-root",
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--installation-root",
            OFFICIAL_PI_INSTALLATION_ROOT,
            "--package-ref",
            &package,
            "--compatibility-accepted",
            "yes",
            "--health-accepted",
            "yes",
        ];
        if let Some(version) = expected_text.as_deref() {
            args.push("--expected-activation-version");
            args.push(version);
        }
        run_cli(&args)
    }

    fn rollback(&self, expected: u64, target: u64) -> CliResult {
        let expected_text = expected.to_string();
        let target_text = target.to_string();
        run_cli(&[
            "rollback",
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--installation-root",
            OFFICIAL_PI_INSTALLATION_ROOT,
            "--expected-activation-version",
            &expected_text,
            "--target-activation-version",
            &target_text,
        ])
    }

    fn register(&self, activation_version: u64) -> CliResult {
        let version = activation_version.to_string();
        run_cli(&[
            "register",
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--installation-root",
            OFFICIAL_PI_INSTALLATION_ROOT,
            "--expected-activation-version",
            &version,
            "--adapter-digest",
            ADAPTER,
            "--protocol-digest",
            PROTOCOL,
            "--policy-digest",
            POLICY,
        ])
    }

    fn lifecycle(&self, verb: &str, epoch: u64) -> CliResult {
        let epoch_text = epoch.to_string();
        run_cli(&[
            verb,
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--installation-root",
            OFFICIAL_PI_INSTALLATION_ROOT,
            "--expected-fencing-epoch",
            &epoch_text,
            "--protocol-digest",
            PROTOCOL,
        ])
    }

    fn health(&self) -> CliResult {
        run_cli(&[
            "agent-health",
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--installation-root",
            OFFICIAL_PI_INSTALLATION_ROOT,
        ])
    }

    fn uninstall(&self, activation_version: u64) -> CliResult {
        let version = activation_version.to_string();
        run_cli(&[
            "uninstall",
            "--store",
            self.authority_db(),
            "--session",
            self.session(),
            "--installation-store",
            self.install_db(),
            "--installation-root",
            OFFICIAL_PI_INSTALLATION_ROOT,
            "--expected-activation-version",
            &version,
            "--lifecycle-precondition",
            "stopped",
        ])
    }
}

#[test]
fn managed_pi_install_through_recover_upgrade_rollback_and_orphan() {
    let directory = tempfile::tempdir().unwrap();
    let harness = Harness::new(directory.path());

    let installed = assert_ok(&harness.official_install(), "official install");
    assert_eq!(installed["source_mode"], "official_pi");
    assert_eq!(installed["capability_grants"], 0);

    let root_v1 = assert_ok(&harness.activate_root(None), "activate-root v1");
    assert_eq!(root_v1["activation_version"], 1);
    assert_eq!(root_v1["package_ref"], official_package_ref());

    let registered = assert_ok(&harness.register(1), "register");
    assert_eq!(registered["lifecycle_state"], "registered");
    assert_eq!(registered["fencing_epoch"], 1);
    assert_eq!(registered["capability_grants"], 0);

    let activated = assert_ok(&harness.lifecycle("activate", 1), "activate");
    assert_eq!(activated["lifecycle_state"], "active");
    assert_eq!(activated["fencing_epoch"], 2);
    let health = assert_ok(&harness.health(), "health after activate");
    assert_eq!(health["process_bound"], true);

    let blocked_upgrade = harness.activate_root(Some(1));
    assert_eq!(blocked_upgrade.code, 1, "{}", blocked_upgrade.stderr);
    assert_eq!(error_code(&blocked_upgrade), "STATE_CONFLICT");
    assert!(blocked_upgrade.stderr.contains("process-bound"));

    let blocked_rollback = harness.rollback(1, 1);
    assert_eq!(blocked_rollback.code, 1, "{}", blocked_rollback.stderr);
    assert_eq!(error_code(&blocked_rollback), "STATE_CONFLICT");

    let paused = assert_ok(&harness.lifecycle("agent-pause", 2), "pause");
    assert_eq!(paused["lifecycle_state"], "paused");
    let resumed = assert_ok(&harness.lifecycle("agent-resume", 2), "resume");
    assert_eq!(resumed["lifecycle_state"], "active");
    assert_eq!(resumed["fencing_epoch"], 3);
    let stopped = assert_ok(&harness.lifecycle("agent-stop", 3), "stop");
    assert_eq!(stopped["lifecycle_state"], "stopped");

    let stale = harness.lifecycle("agent-recover", 2);
    assert_eq!(stale.code, 1, "{}", stale.stderr);
    assert_eq!(error_code(&stale), "STATE_CONFLICT");

    let root_v2 = assert_ok(&harness.activate_root(Some(1)), "upgrade to v2");
    assert_eq!(root_v2["activation_version"], 2);
    let rolled = assert_ok(&harness.rollback(2, 1), "rollback to v1 package");
    assert_eq!(rolled["activation_version"], 3);
    assert_eq!(rolled["package_ref"], official_package_ref());

    let recovered = assert_ok(&harness.lifecycle("agent-recover", 3), "recover");
    assert_eq!(recovered["lifecycle_state"], "active");
    assert_eq!(recovered["fencing_epoch"], 4);
    assert_eq!(recovered["capability_grants"], 0);
    assert_eq!(recovered["effects_created"], 0);
    assert_eq!(recovered["tasks_completed"], 0);
    assert_ne!(
        recovered["sidecar_session_id"],
        activated["sidecar_session_id"]
    );

    let stopped_again = assert_ok(&harness.lifecycle("agent-stop", 4), "stop before uninstall");
    assert_eq!(stopped_again["lifecycle_state"], "stopped");
    let uninstalled = assert_ok(&harness.uninstall(3), "uninstall");
    assert_eq!(uninstalled["activation_version"], 3);
    assert_eq!(uninstalled["capability_grants"], 0);
}
