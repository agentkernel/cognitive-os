//! P2-T28/D02 public UJ journeys: each required family has a live caller
//! and a mechanical oracle on one hermetic daemon. Web UI/Multi-Agent stay
//! scope-excluded and are not exercised. Nested Pi timing and managed Pi
//! install→recover remain covered by their named D01 oracles on linux-002.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use serde_json::{Value, json};

static P2_T28_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t28-{}-{}", std::process::id(), free_port()));
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

fn send_json(port: u16, path: &str, token: &str, body: &Value) -> String {
    let body = body.to_string();
    request(
        port,
        &format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

fn get(port: u16, path: &str, token: &str) -> String {
    request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    )
}

struct Hermetic {
    daemon: Child,
    root: std::path::PathBuf,
}

impl Drop for Hermetic {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn required_uj_public_callers_return_mechanical_oracles_and_cleanup() {
    let _guard = P2_T28_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let daemon = spawn_personal(port, &root);
    let mut hermetic = Hermetic { daemon, root };
    let secret = common::wait_for_bootstrap_secret_from(&mut hermetic.daemon, &hermetic.root);
    let task_token = issue_token(port, &secret, "task");
    let management_token = issue_token(port, &secret, "management");

    // UJ1 / UJ3 — install/init/first-response and status/doctor.
    let status = get(port, "/personal/status", &management_token);
    assert!(status.starts_with("HTTP/1.1 200 "), "{status}");
    let status_json = response_json(&status);
    assert_eq!(status_json["profile_claim"], "not-claimed");
    assert_eq!(status_json["first_conversation_ready"], false);
    assert!(!status.contains(&secret), "{status}");

    let doctor = get(port, "/personal/doctor", &management_token);
    assert!(doctor.starts_with("HTTP/1.1 200 "), "{doctor}");
    let doctor_json = response_json(&doctor);
    assert!(doctor_json.get("overall").is_some(), "{doctor}");
    assert_eq!(doctor_json["first_conversation_ready"], false);

    // UJ3 — bounded observation replay surface (empty window is a named zero).
    let observation = get(
        port,
        "/task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t28",
        &task_token,
    );
    assert!(observation.starts_with("HTTP/1.1 200 "), "{observation}");
    let o2 = response_json(&observation);
    assert_eq!(o2["family"], "o2");
    assert_eq!(o2["observed_zero"], true);
    assert!(o2.get("negative_control").is_some(), "{observation}");

    // UJ4 — durable terminal query fails closed without a task_ref.
    let evidence = get(port, "/task/evidence", &task_token);
    assert!(evidence.starts_with("HTTP/1.1 400 "), "{evidence}");

    // UJ5 — Effect history is task-channel, unknown task is not found, receipt
    // restatement is forbidden.
    let effects = get(
        port,
        "/task/effects?task_ref=task%3A%2F%2Fpersonal%2Fp2-t28",
        &task_token,
    );
    assert!(effects.starts_with("HTTP/1.1 404 "), "{effects}");
    assert!(
        effects.contains("TASK_EFFECT_HISTORY_NOT_FOUND"),
        "{effects}"
    );
    let restated_effects = get(
        port,
        "/task/effects?task_ref=task%3A%2F%2Fpersonal%2Fp2-t28&receipt=receipt%3A%2F%2Fraw",
        &task_token,
    );
    assert!(
        restated_effects.starts_with("HTTP/1.1 400 "),
        "{restated_effects}"
    );
    assert!(
        restated_effects.contains("TASK_EFFECT_HISTORY_QUERY_FORBIDDEN"),
        "{restated_effects}"
    );

    // UJ6 Memory — consumption query does not restate user text.
    let restated_consumption = get(
        port,
        "/task/resource/v1/consumption?task_ref=task://local/missing&query_text=restated",
        &task_token,
    );
    assert!(
        restated_consumption.contains("RESOURCE_CONSUMPTION_RESTATEMENT_FORBIDDEN"),
        "{restated_consumption}"
    );

    // UJ6 backup — public management caller excludes SQLite/secrets.
    let backup = send_json(
        port,
        "/management/resource/v1/backup",
        &management_token,
        &json!({}),
    );
    assert!(backup.starts_with("HTTP/1.1 200 "), "{backup}");
    let backup_json = response_json(&backup);
    assert_eq!(backup_json["sqlite_copied"], false);
    assert!(backup_json["excluded_secret_count"].as_u64().unwrap() >= 1);
    assert!(!backup.contains(&secret), "{backup}");

    let root_path = hermetic.root.clone();
    drop(hermetic);
    assert!(
        !root_path.exists(),
        "hermetic runtime root must be removed after the sample"
    );
}
