//! P2-T24/D01 public fault-profile and Effect-history HTTP guards.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use serde_json::{Value, json};

static P2_T24_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t24-{}-{}", std::process::id(), free_port()));
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

fn issue_token(port: u16, secret: &str, channel: &str) -> String {
    let body = format!(
        "{{\"channel\":\"{channel}\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{secret}\"}}"
    );
    let response = request(
        port,
        &format!(
            "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    let marker = "\"token\":\"";
    let start = response.find(marker).expect("token") + marker.len();
    let end = start + response[start..].find('"').unwrap();
    response[start..end].to_owned()
}

fn response_json(response: &str) -> Value {
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response has a header/body separator");
    serde_json::from_str(body).expect("HTTP response body is JSON")
}

#[test]
fn public_fault_profile_denies_task_channel_and_unauthorized_campaign() {
    let _guard = P2_T24_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let mut daemon = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let task_token = issue_token(port, &secret, "task");
    let management_token = issue_token(port, &secret, "management");

    let task_enable = request(
        port,
        &format!(
            "POST /task/resource/v1/fault-profile HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(
        task_enable.contains("RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN"),
        "{task_enable}"
    );
    assert!(task_enable.starts_with("HTTP/1.1 403 "), "{task_enable}");

    let unauthorized = request(
        port,
        &format!(
            "POST /management/resource/v1/fault-profile HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            json!({
                "task_ref": "task://personal/example",
                "campaign_id": "owner-local",
                "case_ref": "BR-04-D01",
                "faults_enabled": true,
                "fault_point": "dispatch_before"
            })
            .to_string()
            .len(),
            json!({
                "task_ref": "task://personal/example",
                "campaign_id": "owner-local",
                "case_ref": "BR-04-D01",
                "faults_enabled": true,
                "fault_point": "dispatch_before"
            })
        ),
    );
    let unauthorized_json = response_json(&unauthorized);
    assert!(unauthorized.starts_with("HTTP/1.1 403 "), "{unauthorized}");
    assert_eq!(
        unauthorized_json["code"],
        "RESOURCE_FAULT_PROFILE_UNAUTHORIZED"
    );

    let history = request(
        port,
        &format!(
            "GET /task/effects?task_ref=task%3A%2F%2Fpersonal%2Fexample HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(history.starts_with("HTTP/1.1 404 "), "{history}");
    assert!(
        history.contains("TASK_EFFECT_HISTORY_NOT_FOUND"),
        "{history}"
    );

    let restatement = request(
        port,
        &format!(
            "GET /task/effects?task_ref=task%3A%2F%2Fpersonal%2Fexample&receipt=receipt%3A%2F%2Fraw HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(restatement.starts_with("HTTP/1.1 400 "), "{restatement}");
    assert!(
        restatement.contains("TASK_EFFECT_HISTORY_QUERY_FORBIDDEN"),
        "{restatement}"
    );

    daemon.kill().unwrap();
    let _ = daemon.wait();
}
