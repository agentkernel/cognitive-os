//! P2-T02/D01 process evidence for the authenticated daemon Task API.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

static TASK_API_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t02-{}-{}", std::process::id(), free_port()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn request(port: u16, wire: &str) -> String {
    let mut stream = common::connect_when_ready(port);
    stream.write_all(wire.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn spawn_personal(port: u16, runtime_root: &std::path::Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args([
            "--personal",
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--runtime-root",
            runtime_root.to_str().unwrap(),
        ])
        .spawn()
        .unwrap()
}

fn issue_task_token(port: u16, secret: &str) -> String {
    let body = format!(
        "{{\"channel\":\"task\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{secret}\"}}"
    );
    let response = request(
        port,
        &format!(
            "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    let marker = "\"token\":\"";
    let start = response.find(marker).unwrap() + marker.len();
    let end = start + response[start..].find('"').unwrap();
    response[start..end].to_owned()
}

#[test]
fn task_record_requires_task_auth_persists_daemon_root_and_watch_is_snapshot_first() {
    let _guard = TASK_API_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let mut daemon = spawn_personal(port, &root);
    let token = issue_task_token(
        port,
        &common::wait_for_bootstrap_secret_from(&mut daemon, &root),
    );
    let body = "{\"conversation_or_scope_ref\":\"conversation://local/thread-1\",\"raw_expression\":\"prepare a governed task\",\"schema_version\":\"cognitiveos.task-intent-record-request/0.1\"}";

    let unauthenticated = request(
        port,
        &format!(
            "POST /task/intent.record HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(unauthenticated.contains("LOCAL_SESSION_UNAUTHORIZED"));

    let recorded = request(
        port,
        &format!(
            "POST /task/intent.record HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(recorded.contains("user_intent_record_id"), "{recorded}");
    assert!(
        root.join("data")
            .join("cognitiveos")
            .join("personal-governance-root.json")
            .is_file()
    );

    let watch = request(
        port,
        &format!(
            "GET /task/watch HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(watch.contains("Content-Type: text/event-stream"), "{watch}");
    assert!(watch.contains("\"kind\":\"snapshot\""), "{watch}");
    assert!(watch.contains("\"kind\":\"delta\""), "{watch}");
    assert!(
        watch.find("\"kind\":\"snapshot\"").unwrap() < watch.find("\"kind\":\"delta\"").unwrap()
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(root);
}
