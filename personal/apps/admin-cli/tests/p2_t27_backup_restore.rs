//! P2-T27/D01 public `cognitive backup` / `restore` caller.
//!
//! Hermetic `--runtime-root` trees. Never asserts secret material values.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p2t27-{}-{}-{}",
        label,
        std::process::id(),
        free_port()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn cognitive() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cognitive"))
}

fn run_cognitive(args: &[&str]) -> Output {
    cognitive().args(args).output().expect("spawn cognitive")
}

fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn seed_layout(root: &std::path::Path) {
    let config = root.join("config").join("cognitiveos");
    let data = root.join("data").join("cognitiveos");
    let state = root.join("state").join("cognitiveos");
    let cache = root.join("cache").join("cognitiveos");
    let runtime = root.join("cognitiveos");
    fs::create_dir_all(&config).unwrap();
    fs::create_dir_all(&data).unwrap();
    fs::create_dir_all(&state).unwrap();
    fs::create_dir_all(&cache).unwrap();
    fs::create_dir_all(&runtime).unwrap();
    fs::write(config.join("ui.json"), b"{\"theme\":\"dark\"}").unwrap();
    fs::write(
        config.join("provider-config.json"),
        b"{\"secret_ref\":\"ssv1:should-not-copy\"}",
    )
    .unwrap();
    fs::write(runtime.join("local-bootstrap.secret"), b"bootstrap-secret").unwrap();
    fs::write(data.join("authority.sqlite"), b"sqlite-bytes").unwrap();
}

#[test]
fn backup_restore_roundtrip_excludes_secrets_and_sqlite() {
    let root = runtime_root("roundtrip");
    seed_layout(&root);
    let archive = std::env::temp_dir().join(format!(
        "cos-p2t27-archive-{}-{}",
        std::process::id(),
        free_port()
    ));
    let _ = fs::remove_dir_all(&archive);

    let backup = run_cognitive(&[
        "backup",
        "--runtime-root",
        root.to_str().unwrap(),
        "--output",
        archive.to_str().unwrap(),
    ]);
    assert!(
        backup.status.success(),
        "stdout={} stderr={}",
        stdout_str(&backup),
        stderr_str(&backup)
    );
    let backup_json: serde_json::Value = serde_json::from_str(&stdout_str(&backup)).unwrap();
    assert_eq!(backup_json["sqlite_copied"], false);
    assert!(!archive.join("parts/config/provider-config.json").exists());

    fs::write(
        root.join("config").join("cognitiveos").join("ui.json"),
        b"mutated",
    )
    .unwrap();

    let preflight = run_cognitive(&[
        "restore",
        "--runtime-root",
        root.to_str().unwrap(),
        "--archive",
        archive.to_str().unwrap(),
        "--preflight",
    ]);
    assert!(
        preflight.status.success(),
        "stdout={} stderr={}",
        stdout_str(&preflight),
        stderr_str(&preflight)
    );

    let restore = run_cognitive(&[
        "restore",
        "--runtime-root",
        root.to_str().unwrap(),
        "--archive",
        archive.to_str().unwrap(),
    ]);
    assert!(
        restore.status.success(),
        "stdout={} stderr={}",
        stdout_str(&restore),
        stderr_str(&restore)
    );
    assert_eq!(
        fs::read(root.join("config").join("cognitiveos").join("ui.json")).unwrap(),
        b"{\"theme\":\"dark\"}"
    );
    assert_eq!(
        fs::read(
            root.join("data")
                .join("cognitiveos")
                .join("authority.sqlite")
        )
        .unwrap(),
        b"sqlite-bytes"
    );
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&archive);
}

#[test]
fn restore_preflight_rejects_tampered_archive() {
    let root = runtime_root("tamper");
    seed_layout(&root);
    let archive = std::env::temp_dir().join(format!(
        "cos-p2t27-tamper-{}-{}",
        std::process::id(),
        free_port()
    ));
    let _ = fs::remove_dir_all(&archive);
    let backup = run_cognitive(&[
        "backup",
        "--runtime-root",
        root.to_str().unwrap(),
        "--output",
        archive.to_str().unwrap(),
    ]);
    assert!(backup.status.success(), "{}", stderr_str(&backup));
    fs::write(
        archive
            .join("parts")
            .join("authority-db")
            .join("export.json"),
        b"{\"tampered\":true}",
    )
    .unwrap();
    let restore = run_cognitive(&[
        "restore",
        "--runtime-root",
        root.to_str().unwrap(),
        "--archive",
        archive.to_str().unwrap(),
        "--preflight",
    ]);
    assert_eq!(restore.status.code(), Some(1));
    assert!(stderr_str(&restore).contains("digest") || stderr_str(&restore).contains("tamper"));
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&archive);
}
