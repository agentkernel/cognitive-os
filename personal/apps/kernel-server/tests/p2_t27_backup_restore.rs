//! P2-T27/D01 authenticated public backup/restore excluding secrets and SQLite.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use serde_json::{Value, json};

static P2_T27_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t27-{}-{}", std::process::id(), free_port()));
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

#[test]
fn backup_restore_excludes_secrets_and_rejects_tamper_and_task_channel() {
    let _guard = P2_T27_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let config_dir = root.join("config").join("cognitiveos");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("ui.json"), b"{\"theme\":\"dark\"}").unwrap();
    let mut daemon = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let task_token = issue_token(port, &secret, "task");
    let management_token = issue_token(port, &secret, "management");

    let task_denied = send_json(port, "/task/resource/v1/backup", &task_token, &json!({}));
    assert!(task_denied.starts_with("HTTP/1.1 403 "), "{task_denied}");
    assert!(
        task_denied.contains("RESOURCE_BACKUP_CHANNEL_FORBIDDEN"),
        "{task_denied}"
    );

    let secret_body = send_json(
        port,
        "/management/resource/v1/restore",
        &management_token,
        &json!({ "archive_id": "x", "prompt": "leak" }),
    );
    assert!(secret_body.starts_with("HTTP/1.1 400 "), "{secret_body}");
    assert!(
        secret_body.contains("RESOURCE_BACKUP_QUERY_FORBIDDEN"),
        "{secret_body}"
    );

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
    let archive_id = backup_json["archive_id"].as_str().unwrap().to_owned();
    let archive_path = std::path::PathBuf::from(backup_json["archive_path"].as_str().unwrap());
    assert!(
        !archive_path
            .join("parts/config/provider-config.json")
            .exists()
    );
    let manifest = std::fs::read_to_string(archive_path.join("manifest.json")).unwrap();
    assert!(!manifest.contains(&secret));
    assert!(!manifest.contains("authority.sqlite"));

    std::fs::write(config_dir.join("ui.json"), b"mutated").unwrap();

    let preflight = send_json(
        port,
        "/management/resource/v1/backup/preflight",
        &management_token,
        &json!({ "archive_id": archive_id }),
    );
    assert!(preflight.starts_with("HTTP/1.1 200 "), "{preflight}");
    assert_eq!(response_json(&preflight)["preflight_only"], true);

    let restored = send_json(
        port,
        "/management/resource/v1/restore",
        &management_token,
        &json!({ "archive_id": archive_id }),
    );
    assert!(restored.starts_with("HTTP/1.1 200 "), "{restored}");
    let restored_json = response_json(&restored);
    assert_eq!(restored_json["live_applied"], true);
    assert_eq!(
        std::fs::read(config_dir.join("ui.json")).unwrap(),
        b"{\"theme\":\"dark\"}"
    );
    assert!(
        root.join("cognitiveos")
            .join("local-bootstrap.secret")
            .exists()
    );

    let export = archive_path.join("parts/authority-db/export.json");
    std::fs::write(&export, b"{\"tampered\":true}").unwrap();
    let tampered = send_json(
        port,
        "/management/resource/v1/restore",
        &management_token,
        &json!({ "archive_id": archive_id }),
    );
    assert!(tampered.starts_with("HTTP/1.1 409 "), "{tampered}");
    assert!(tampered.contains("RESOURCE_BACKUP_TAMPERED"), "{tampered}");

    let _ = daemon.kill();
    let _ = std::fs::remove_dir_all(&root);
}
