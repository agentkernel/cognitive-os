//! P1-T05 Personal readiness / status / doctor evidence.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static PERSONAL_PROCESS_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p1t05-{}-{}-{}",
        label,
        std::process::id(),
        free_port()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
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

fn wait_connect(port: u16) -> TcpStream {
    for _ in 0..100 {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            return stream;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("personal daemon did not accept connections on {port}");
}

fn http_exchange(port: u16, wire: &str) -> String {
    let mut stream = wait_connect(port);
    let _ = stream.write_all(wire.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut out = String::new();
    let _ = stream.read_to_string(&mut out);
    out
}

fn bootstrap_secret(runtime_root: &std::path::Path) -> String {
    let path = runtime_root
        .join("cognitiveos")
        .join("local-bootstrap.secret");
    for _ in 0..100 {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            let trimmed = contents.trim().to_owned();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("bootstrap secret not found at {}", path.display());
}

fn issue_token(port: u16, secret: &str, channel: &str) -> String {
    let body = format!(
        "{{\"channel\":\"{channel}\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{secret}\"}}"
    );
    let wire = format!(
        "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = http_exchange(port, &wire);
    assert!(response.contains("HTTP/1.1 200"), "{response}");
    let token_key = "\"token\":\"";
    let start = response.find(token_key).expect("token field") + token_key.len();
    let end = response[start..].find('"').expect("token end") + start;
    response[start..end].to_owned()
}

#[test]
fn status_and_doctor_require_management_channel_and_report_blocked() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let root = runtime_root("auth-status");
    let mut child = spawn_personal(port, &root);
    let secret = bootstrap_secret(&root);
    let management_token = issue_token(port, &secret, "management");
    let task_token = issue_token(port, &secret, "task");

    let unauth = http_exchange(
        port,
        "GET /personal/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(unauth.contains("LOCAL_SESSION_UNAUTHORIZED"), "{unauth}");

    let wrong_channel = format!(
        "GET /personal/doctor HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
    );
    let wrong = http_exchange(port, &wrong_channel);
    assert!(wrong.contains("SHELL_CHANNEL_BINDING_MISMATCH"), "{wrong}");

    let status_wire = format!(
        "GET /personal/status HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
    );
    let status = http_exchange(port, &status_wire);
    assert!(status.contains("HTTP/1.1 200"), "{status}");
    assert!(status.contains("\"overall\":\"blocked\""), "{status}");
    assert!(
        status.contains("\"profile_claim\":\"not-claimed\""),
        "{status}"
    );
    assert!(
        status.contains("\"static_check_is_not_runtime_ready\":true"),
        "{status}"
    );
    assert!(
        status.contains("\"first_conversation_ready\":false"),
        "{status}"
    );

    let doctor_wire = format!(
        "GET /personal/doctor HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
    );
    let doctor = http_exchange(port, &doctor_wire);
    assert!(doctor.contains("HTTP/1.1 200"), "{doctor}");
    assert!(
        doctor.contains("\"surface\":\"personal-doctor\""),
        "{doctor}"
    );
    assert!(doctor.contains("\"component\":\"database\""), "{doctor}");
    assert!(
        doctor.contains("database_not_prepared")
            || doctor.contains("secret_store_unavailable")
            || doctor.contains("provider_config_missing"),
        "{doctor}"
    );
    assert!(
        doctor.contains("\"gate_claim\":\"not-claimed\""),
        "{doctor}"
    );
    // Bootstrap secret material must never appear in projections.
    assert!(!doctor.contains(&secret), "{doctor}");

    let _ = child.kill();
    let _ = child.wait();
}
