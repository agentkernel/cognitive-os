#![allow(clippy::expect_used, clippy::unwrap_used)]

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

struct CliResult {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_cli(args: &[&str], npm_dir: Option<&Path>) -> CliResult {
    let mut command = Command::new(env!("CARGO_BIN_EXE_admin-cli"));
    command.args(args);
    if let Some(npm_dir) = npm_dir {
        let inherited = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = npm_dir.as_os_str().to_owned();
        paths.push(if cfg!(windows) { ";" } else { ":" });
        paths.push(inherited);
        command.env("PATH", paths);
    }
    let output = command.output().expect("admin-cli runs");
    CliResult {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn write_fake_npm(dir: &Path) {
    #[cfg(windows)]
    std::fs::write(
        dir.join("npm.cmd"),
        "@echo off\r\nif not \"%1\"==\"ci\" exit /b 11\r\nif not \"%2\"==\"--ignore-scripts\" exit /b 12\r\nexit /b 0\r\n",
    )
    .unwrap();
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("npm");
        std::fs::write(
            &path,
            "#!/bin/sh\n[ \"$1\" = \"ci\" ] || exit 11\n[ \"$2\" = \"--ignore-scripts\" ] || exit 12\nexit 0\n",
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn write_session(dir: &Path) -> PathBuf {
    let path = dir.join("session.json");
    let value = json!({
        "schema_version": "cognitiveos.privileged-management-session/0.1",
        "session_id": "pms_custom-install-01",
        "object_version": 1,
        "management_domain": "cognitiveos.management",
        "session_authority": "authority://tenant-a/management-authority",
        "human_principal": "principal://tenant-a/verified-operator",
        "actor_chain_digest": format!("sha256:{}", "ab12".repeat(16)),
        "authentication_context_ref": "authn://tenant-a/webauthn-9",
        "activity_context_ref": "activity://tenant-a/custom-install",
        "scope": {
            "domains": ["cognitiveos.management"],
            "actions": ["agent.install"],
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
        "authority_signature": "sig-custom-install-fixture-0001"
    });
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    path
}

fn write_locked_project(dir: &Path) -> PathBuf {
    let project = dir.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join("package.json"),
        r#"{"name":"custom-agent","version":"1.0.0","dependencies":{}}"#,
    )
    .unwrap();
    std::fs::write(
        project.join("package-lock.json"),
        r#"{"name":"custom-agent","version":"1.0.0","lockfileVersion":3,"requires":true,"packages":{"":{"name":"custom-agent","version":"1.0.0"}}}"#,
    )
    .unwrap();
    project
}

fn standard_args<'a>(session: &'a Path, database: &'a Path, project: &'a Path) -> Vec<&'a str> {
    vec![
        "install",
        "--mode",
        "custom",
        "--session",
        session.to_str().unwrap(),
        "--installation-store",
        database.to_str().unwrap(),
        "--project",
        project.to_str().unwrap(),
        "--package-id",
        "pkg://tenant-a/custom-agent@1.0.0",
        "--adapter-digest",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--sandbox-digest",
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "--compatibility-digest",
        "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    ]
}

#[test]
fn custom_install_requires_an_explicit_confirmation_and_shows_the_fixed_notice() {
    let directory = tempfile::tempdir().unwrap();
    let session = write_session(directory.path());
    let project = write_locked_project(directory.path());
    let database = directory.path().join("install.db");
    let args = standard_args(&session, &database, &project);

    let result = run_cli(&args, None);

    assert_eq!(
        result.code, 1,
        "stdout: {} stderr: {}",
        result.stdout, result.stderr
    );
    assert!(
        result
            .stderr
            .contains("This project does not have trusted publisher provenance.")
    );
    assert!(result.stderr.contains("after installation it follows the same authorization and execution policy as a normal installation."));
    assert!(
        !database.exists(),
        "unconfirmed source must not open a durable store"
    );
}

#[test]
fn confirmed_custom_install_uses_authenticated_principal_and_commits_no_capability_or_effect() {
    let directory = tempfile::tempdir().unwrap();
    let session = write_session(directory.path());
    let project = write_locked_project(directory.path());
    let database = directory.path().join("install.db");
    let npm_dir = directory.path().join("fake-npm");
    std::fs::create_dir(&npm_dir).unwrap();
    write_fake_npm(&npm_dir);
    let mut args = standard_args(&session, &database, &project);
    args.extend(["--confirm-custom-source", "yes"]);

    let result = run_cli(&args, Some(&npm_dir));

    assert_eq!(
        result.code, 0,
        "stdout: {} stderr: {}",
        result.stdout, result.stderr
    );
    let value: Value = serde_json::from_str(result.stdout.trim()).unwrap();
    assert_eq!(value["source_mode"], "custom_user_provided");
    assert_eq!(
        value["operator_ref"],
        "principal://tenant-a/verified-operator"
    );
    assert!(
        value["bundle_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert!(
        value["lockfile_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(value["capability_grants"], 0);
    assert_eq!(value["effects_created"], 0);
    assert_eq!(value["tasks_completed"], 0);
}
