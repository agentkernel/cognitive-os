//! P2-T02/D03 deterministic CLI parity evidence.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::Duration;

static CLI_PARITY_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p2t02-cli-{}-{}",
        std::process::id(),
        free_port()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn cognitive() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cognitive"))
}

fn kernel_server_binary() -> PathBuf {
    let cognitive_binary = PathBuf::from(env!("CARGO_BIN_EXE_cognitive"));
    let kernel_server = cognitive_binary.with_file_name("kernel-server");
    assert!(kernel_server.is_file(), "missing kernel-server binary");
    kernel_server
}

fn run_cognitive(arguments: &[&str]) -> Output {
    cognitive().args(arguments).output().unwrap()
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn cognitive_uses_isolated_task_and_resource_daemon_channels() {
    let _guard = CLI_PARITY_TEST_LOCK.lock().unwrap();
    let root = runtime_root();
    let port = free_port();
    let bind_address = format!("127.0.0.1:{port}");

    let init = run_cognitive(&["init", "--runtime-root", root.to_str().unwrap()]);
    assert!(init.status.success(), "{}", output_text(&init));

    let mut daemon = Command::new(kernel_server_binary())
        .args([
            "--personal",
            "--bind",
            &bind_address,
            "--runtime-root",
            root.to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let bootstrap_secret = root.join("cognitiveos").join("local-bootstrap.secret");
    for _ in 0..250 {
        if bootstrap_secret.is_file() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(bootstrap_secret.is_file(), "daemon did not start");

    let resource = run_cognitive(&[
        "resource",
        "get",
        "--family",
        "memory",
        "--runtime-root",
        root.to_str().unwrap(),
        "--endpoint",
        &bind_address,
    ]);
    assert!(resource.status.success(), "{}", output_text(&resource));
    assert!(
        String::from_utf8_lossy(&resource.stdout).contains("\"availability\":\"not-backed\""),
        "{}",
        output_text(&resource)
    );

    let task_watch = run_cognitive(&[
        "task",
        "watch",
        "--runtime-root",
        root.to_str().unwrap(),
        "--endpoint",
        &bind_address,
    ]);
    assert!(task_watch.status.success(), "{}", output_text(&task_watch));
    assert!(
        String::from_utf8_lossy(&task_watch.stdout).contains("event: snapshot"),
        "{}",
        output_text(&task_watch)
    );

    let invalid_cursor = run_cognitive(&[
        "resource",
        "get",
        "--family",
        "memory",
        "--resume-from",
        "1",
        "--runtime-root",
        root.to_str().unwrap(),
        "--endpoint",
        &bind_address,
    ]);
    assert_eq!(
        invalid_cursor.status.code(),
        Some(2),
        "{}",
        output_text(&invalid_cursor)
    );

    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = fs::remove_dir_all(root);
}
