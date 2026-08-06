//! P2-T02/D02 process evidence for private resource projections.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static RESOURCE_PROJECTION_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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

#[test]
fn resource_projection_is_private_versioned_and_management_channel_bound() {
    let _guard = RESOURCE_PROJECTION_PROCESS_LOCK.lock().unwrap();
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p2t02-resource-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = bootstrap_secret(&runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let task_token = issue_token(port, &secret, "task");

    let path = "/resource/v1/projection?family=memory&version=1";
    let management_response = request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        management_response.contains("200 OK"),
        "{management_response}"
    );
    assert!(management_response.contains("personal-resource-projection/1"));
    assert!(management_response.contains("\"family\":\"memory\""));
    assert!(management_response.contains("\"availability\":\"not-backed\""));

    let tool_response = request(
        port,
        &format!(
            "GET /resource/v1/projection?family=tool&version=1 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(tool_response.contains("200 OK"), "{tool_response}");
    assert!(tool_response.contains("daemon-native-tool-registry"));
    assert!(tool_response.contains("native.workspace.read"));
    assert!(tool_response.contains("native.http.fetch"));
    assert!(tool_response.contains("descriptor_digest"));

    let task_response = request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(task_response.contains("403 Forbidden"), "{task_response}");
    assert!(task_response.contains("SHELL_CHANNEL_BINDING_MISMATCH"));

    let watch = request(
        port,
        &format!(
            "GET /resource/v1/watch?family=memory&version=1 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(watch.contains("Content-Type: text/event-stream"), "{watch}");
    assert!(watch.contains("\"kind\":\"snapshot\""));
    assert!(watch.contains("\"family\":\"memory\""));

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}
