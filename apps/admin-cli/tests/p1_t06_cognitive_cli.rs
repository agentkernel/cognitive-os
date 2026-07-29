//! P1-T06 Personal `cognitive` CLI evidence.
//!
//! Spawns the real `cognitive` binary (and `kernel-server` for daemon paths)
//! under hermetic `--runtime-root` trees. Never prints or asserts secret
//! material values beyond redaction negatives.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Stdio;
use std::process::{Command, Output};
use std::sync::{LazyLock, Mutex};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::Duration;

static PERSONAL_CLI_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p1t06-{}-{}-{}",
        label,
        std::process::id(),
        free_port()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(unix)]
fn kernel_server_binary() -> PathBuf {
    // Workspace `cargo test` places bins side-by-side under target/{debug,release}.
    let cognitive = PathBuf::from(env!("CARGO_BIN_EXE_cognitive"));
    let sibling = cognitive.with_file_name(if cfg!(windows) {
        "kernel-server.exe"
    } else {
        "kernel-server"
    });
    assert!(
        sibling.is_file(),
        "kernel-server binary missing at {}; build with `cargo build -p kernel-server` \
         before this suite (CI workspace builds both)",
        sibling.display()
    );
    sibling
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

fn write_key_file(dir: &Path, contents: &str) -> PathBuf {
    let path = dir.join("api-key.txt");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    path
}

#[test]
fn init_prepares_layout_and_is_idempotent_without_provider_egress() {
    let _guard = PERSONAL_CLI_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = runtime_root("init-idempotent");
    let first = run_cognitive(&["init", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        first.status.success(),
        "stdout={} stderr={}",
        stdout_str(&first),
        stderr_str(&first)
    );
    let first_out = stdout_str(&first);
    assert!(first_out.contains("\"status\": \"ok\""), "{first_out}");
    assert!(first_out.contains("authority_database"), "{first_out}");
    assert!(first_out.contains("\"configured\": false"), "{first_out}");

    let authority = root
        .join("data")
        .join("cognitiveos")
        .join("authority.sqlite");
    assert!(authority.is_file(), "authority db missing at {authority:?}");
    // Re-init remains local and does not initiate Provider egress without flags.
    let second = run_cognitive(&["init", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        second.status.success(),
        "stdout={} stderr={}",
        stdout_str(&second),
        stderr_str(&second)
    );
    let second_out = stdout_str(&second);
    assert!(
        second_out.contains("\"idempotent_reinit\": true"),
        "{second_out}"
    );
    assert!(
        second_out.contains("skipped")
            || second_out.contains("unchanged")
            || second_out.contains("preserved"),
        "{second_out}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn init_rejects_http_base_url_with_actionable_error() {
    let _guard = PERSONAL_CLI_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = runtime_root("init-http");
    let key_path = write_key_file(&root, "unused-secret\n");
    let output = run_cognitive(&[
        "init",
        "--runtime-root",
        root.to_str().unwrap(),
        "--provider",
        "deepseek",
        "--base-url",
        "http://api.deepseek.com/v1",
        "--api-key-file",
        key_path.to_str().unwrap(),
        "--allow-ephemeral-secret-backend",
    ]);
    assert!(!output.status.success(), "http URL must fail");
    let err = stderr_str(&output);
    assert!(err.contains("https://"), "{err}");
    assert!(err.contains("http://") || err.contains("rejected"), "{err}");
    assert!(
        !err.contains("unused-secret"),
        "secret leaked in error: {err}"
    );
    let _ = fs::remove_dir_all(&root);
}

/// Doctor/status against a Child-owned Personal daemon (Unix).
///
/// Windows MSVC CI has hung for 10+ minutes inside this package's
/// process-spawning tests (job-object / process-tree interaction with
/// `kernel-server --personal`). Ubuntu remains the authoritative host for
/// live daemon + CLI projection evidence; Windows still builds the `cognitive`
/// binary and runs init/usage tests below.
#[cfg(unix)]
#[test]
fn doctor_and_status_against_child_owned_daemon() {
    let _guard = PERSONAL_CLI_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = runtime_root("doctor-child-daemon");
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let kernel_server = kernel_server_binary();

    let init = run_cognitive(&["init", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        init.status.success(),
        "stdout={} stderr={}",
        stdout_str(&init),
        stderr_str(&init)
    );

    let mut daemon = Command::new(&kernel_server)
        .args([
            "--personal",
            "--bind",
            &bind,
            "--runtime-root",
            root.to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn kernel-server --personal");

    let bootstrap = root.join("cognitiveos").join("local-bootstrap.secret");
    let mut ready = false;
    for _ in 0..250 {
        if bootstrap.is_file() {
            ready = true;
            break;
        }
        if let Ok(Some(status)) = daemon.try_wait() {
            panic!("kernel-server exited before ready: {status:?}");
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(ready, "daemon did not publish bootstrap secret");

    let doctor = run_cognitive(&[
        "doctor",
        "--runtime-root",
        root.to_str().unwrap(),
        "--endpoint",
        &bind,
    ]);
    assert!(
        doctor.status.success(),
        "stdout={} stderr={}",
        stdout_str(&doctor),
        stderr_str(&doctor)
    );
    let doctor_out = stdout_str(&doctor);
    assert!(
        doctor_out.contains("personal-doctor") || doctor_out.contains("\"overall\""),
        "{doctor_out}"
    );
    assert!(
        doctor_out.contains("\"profile_claim\":\"not-claimed\"")
            || doctor_out.contains("\"profile_claim\": \"not-claimed\""),
        "{doctor_out}"
    );

    let personal_status = run_cognitive(&[
        "status",
        "--runtime-root",
        root.to_str().unwrap(),
        "--endpoint",
        &bind,
    ]);
    assert!(
        personal_status.status.success(),
        "stdout={} stderr={}",
        stdout_str(&personal_status),
        stderr_str(&personal_status)
    );
    assert!(
        stdout_str(&personal_status).contains("\"overall\""),
        "{}",
        stdout_str(&personal_status)
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = fs::remove_dir_all(&root);
}

/// Full `cognitive daemon start|status|stop` product path (Unix CI).
///
/// Windows MSVC runners have hung indefinitely when the CLI detaches
/// `kernel-server` under the Actions job object; the Child-owned path above
/// covers doctor/status on Windows, and P1-T04 covers daemon process control.
#[cfg(unix)]
#[test]
fn cognitive_daemon_start_status_stop_product_path() {
    let _guard = PERSONAL_CLI_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = runtime_root("daemon-product-path");
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let kernel_server = kernel_server_binary();

    let init = run_cognitive(&["init", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        init.status.success(),
        "stdout={} stderr={}",
        stdout_str(&init),
        stderr_str(&init)
    );

    let start = run_cognitive(&[
        "daemon",
        "start",
        "--runtime-root",
        root.to_str().unwrap(),
        "--bind",
        &bind,
        "--kernel-server",
        kernel_server.to_str().unwrap(),
    ]);
    assert!(
        start.status.success(),
        "stdout={} stderr={}",
        stdout_str(&start),
        stderr_str(&start)
    );

    let status = run_cognitive(&["daemon", "status", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        status.status.success(),
        "stdout={} stderr={}",
        stdout_str(&status),
        stderr_str(&status)
    );
    assert!(
        stdout_str(&status).contains("\"process_alive\": true"),
        "{}",
        stdout_str(&status)
    );

    let stop = run_cognitive(&["daemon", "stop", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        stop.status.success(),
        "stdout={} stderr={}",
        stdout_str(&stop),
        stderr_str(&stop)
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn usage_error_on_unknown_verb() {
    let output = run_cognitive(&["not-a-verb"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_str(&output).contains("unknown verb"));
}
