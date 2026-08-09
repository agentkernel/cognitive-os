//! P4-T05/D01 failure-first coverage for task-bound resource projection.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::Duration;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn request(port: u16, wire: &str) -> String {
    let mut stream = loop {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            break stream;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
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

fn bootstrap_secret(runtime_root: &std::path::Path) -> String {
    let path = runtime_root
        .join("cognitiveos")
        .join("local-bootstrap.secret");
    for _ in 0..100 {
        if let Ok(secret) = std::fs::read_to_string(&path) {
            return secret.trim().to_owned();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("personal daemon did not create its bootstrap secret")
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
    let start = response.find(marker).unwrap() + marker.len();
    let end = start + response[start..].find('"').unwrap();
    response[start..end].to_owned()
}

#[test]
fn task_projection_requires_task_reference_and_management_cannot_cross_task_boundary() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p4t05-resource-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = bootstrap_secret(&runtime_root);
    let task_token = issue_token(port, &secret, "task");
    let management_token = issue_token(port, &secret, "management");

    let missing_reference = request(
        port,
        &format!(
            "GET /task/resource/v1/projection?family=skill&version=1 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        missing_reference.contains("400 Bad Request"),
        "{missing_reference}"
    );
    assert!(missing_reference.contains("RESOURCE_TASK_REFERENCE_REQUIRED"));

    let task_projection = request(
        port,
        &format!(
            "GET /task/resource/v1/projection?family=skill&version=1&task_ref=task://local/one HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(task_projection.contains("200 OK"), "{task_projection}");
    assert!(task_projection.contains("\"task_ref\":\"task://local/one\""));
    assert!(task_projection.contains("\"family\":\"skill\""));

    let management_crossing = request(
        port,
        &format!(
            "GET /task/resource/v1/projection?family=skill&version=1&task_ref=task://local/one HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        management_crossing.contains("403 Forbidden"),
        "{management_crossing}"
    );
    assert!(management_crossing.contains("SHELL_CHANNEL_BINDING_MISMATCH"));

    let missing_memory_id = request(
        port,
        &format!(
            "GET /management/resource/v1/memory/object HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        missing_memory_id.contains("400 Bad Request"),
        "{missing_memory_id}"
    );
    assert!(missing_memory_id.contains("RESOURCE_OBJECT_ID_REQUIRED"));

    let task_memory_explain = request(
        port,
        &format!(
            "GET /management/resource/v1/memory/object?id=00000000-0000-7000-9000-000000000001 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        task_memory_explain.contains("403 Forbidden"),
        "{task_memory_explain}"
    );
    assert!(task_memory_explain.contains("SHELL_CHANNEL_BINDING_MISMATCH"));

    let malformed_forget = request(
        port,
        &format!(
            "POST /management/resource/v1/memory/forget HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{{"
        ),
    );
    assert!(
        malformed_forget.contains("400 Bad Request"),
        "{malformed_forget}"
    );
    assert!(malformed_forget.contains("RESOURCE_MEMORY_PAYLOAD_INVALID"));

    let task_revoke = request(
        port,
        &format!(
            "POST /management/resource/v1/skill/binding/revoke HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{{"
        ),
    );
    assert!(task_revoke.contains("403 Forbidden"), "{task_revoke}");
    assert!(task_revoke.contains("SHELL_CHANNEL_BINDING_MISMATCH"));

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}
