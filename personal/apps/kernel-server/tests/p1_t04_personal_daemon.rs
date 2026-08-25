//! P1-T04 Personal bounded daemon auth/bounds/lifecycle evidence.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

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
        "cos-p1t04-{}-{}-{}",
        label,
        std::process::id(),
        free_port()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn spawn_personal(port: u16, runtime_root: &std::path::Path, once: bool) -> Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kernel-server"));
    command.args([
        "--personal",
        "--bind",
        &format!("127.0.0.1:{port}"),
        "--runtime-root",
        runtime_root.to_str().unwrap(),
    ]);
    if once {
        command.arg("--once");
    }
    command.spawn().unwrap()
}

fn http_exchange(port: u16, wire: &str) -> String {
    let mut stream = common::connect_when_ready(port);
    // Oversized requests may reset mid-write; treat partial write/reset as a closed door.
    let _ = stream.write_all(wire.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut out = String::new();
    let _ = stream.read_to_string(&mut out);
    out
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
fn bad_auth_and_wrong_channel_fail_closed() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let root = runtime_root("auth");
    let mut child = spawn_personal(port, &root, false);
    let secret = common::wait_for_bootstrap_secret_from(&mut child, &root);
    let management_token = issue_token(port, &secret, "management");

    let unauth = http_exchange(
        port,
        "POST /management/inspect HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(unauth.contains("LOCAL_SESSION_UNAUTHORIZED"), "{unauth}");

    let wrong = format!(
        "POST /task/noop HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let wrong_response = http_exchange(port, &wrong);
    assert!(
        wrong_response.contains("SHELL_CHANNEL_BINDING_MISMATCH"),
        "{wrong_response}"
    );

    let ok = format!(
        "POST /management/inspect HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let ok_response = http_exchange(port, &ok);
    assert!(ok_response.contains("\"status\":\"ok\""), "{ok_response}");

    child.kill().unwrap();
    child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn oversized_body_is_rejected() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let root = runtime_root("body");
    let mut child = spawn_personal(port, &root, false);
    let _ = common::wait_for_bootstrap_secret_from(&mut child, &root);
    let body = "x".repeat((1024 * 1024) + 1);
    let wire = format!(
        "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = http_exchange(port, &wire);
    let rejected = response.contains("HTTP/1.1 400 Bad Request")
        || response.contains("REQUEST_BODY_TOO_LARGE")
        || response.contains("PERSONAL_HTTP_PARSE_ERROR")
        || response.is_empty();
    assert!(rejected, "expected body size rejection, got: {response}");
    child.kill().unwrap();
    child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn cookie_auth_and_bad_host_are_rejected() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let root = runtime_root("csrf");
    let mut child = spawn_personal(port, &root, false);
    let _ = common::wait_for_bootstrap_secret_from(&mut child, &root);
    let cookie = http_exchange(
        port,
        "GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: session=1\r\nConnection: close\r\n\r\n",
    );
    assert!(cookie.contains("LOCAL_COOKIE_AUTH_FORBIDDEN"), "{cookie}");
    let host = http_exchange(
        port,
        "GET /personal/health HTTP/1.1\r\nHost: evil.example\r\nConnection: close\r\n\r\n",
    );
    assert!(host.contains("LOCAL_HOST_HEADER_REJECTED"), "{host}");
    let health = http_exchange(
        port,
        "GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(health.contains("HTTP/1.1 200 OK"), "{health}");
    assert!(health.contains("\"schema_version\":1"), "{health}");
    assert!(
        health.contains("\"surface\":\"personal-health\""),
        "{health}"
    );
    assert!(health.contains("\"status\":\"ok\""), "{health}");
    assert!(
        health.contains("\"authority_side_effects\":false"),
        "{health}"
    );
    assert!(
        health.contains("\"readiness_claim\":\"not-claimed\""),
        "{health}"
    );
    assert!(
        health.contains("\"profile_claim\":\"not-claimed\""),
        "{health}"
    );
    assert!(!health.contains("session_count"), "{health}");
    assert!(!health.contains("auth_required"), "{health}");
    child.kill().unwrap();
    child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn second_instance_lock_and_restart() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port_a = free_port();
    let port_b = free_port();
    let root = runtime_root("lock");
    let mut first = spawn_personal(port_a, &root, false);
    let _ = common::wait_for_bootstrap_secret_from(&mut first, &root);
    // Second process same runtime root must fail on daemon.lock.
    let mut second = spawn_personal(port_b, &root, false);
    let status = second.wait().unwrap();
    assert!(!status.success(), "second instance should fail closed");
    first.kill().unwrap();
    first.wait().unwrap();
    // SIGKILL skips Drop, so the create-new lock file can remain. Operators
    // remove a confirmed-stale lock; tests do the same after forced kill.
    let stale_lock = root.join("cognitiveos").join("daemon.lock");
    let _ = std::fs::remove_file(&stale_lock);
    // Restart after forced stop + stale-lock cleanup should succeed.
    let mut third = spawn_personal(port_a, &root, false);
    let secret = common::wait_for_bootstrap_secret_from(&mut third, &root);
    let token = issue_token(port_a, &secret, "task");
    let ok = format!(
        "POST /task/noop HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let response = http_exchange(port_a, &ok);
    assert!(response.contains("\"status\":\"ok\""), "{response}");
    third.kill().unwrap();
    third.wait().unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn production_xdg_daemon_publishes_its_bound_loopback_endpoint() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let root = runtime_root("production-xdg");
    let mut command = Command::new(env!("CARGO_BIN_EXE_kernel-server"));
    command
        .args(["--personal", "--bind", &format!("127.0.0.1:{port}")])
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("XDG_RUNTIME_DIR", root.join("runtime"));
    let mut child = command.spawn().unwrap();

    let endpoint_path = root
        .join("state")
        .join("cognitiveos")
        .join("daemon-endpoint.json");
    let published = common::poll_until(
        &format!("daemon endpoint at {}", endpoint_path.display()),
        common::READY_TIMEOUT,
        || std::fs::read_to_string(&endpoint_path).ok(),
    );

    child.kill().unwrap();
    child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&root);

    let document = published.unwrap_or_else(|diagnostic| panic!("{diagnostic}"));
    assert!(document.contains("\"schema_version\": 1"), "{document}");
    assert!(
        document.contains(&format!("\"endpoint\": \"127.0.0.1:{port}\"")),
        "{document}"
    );
    assert!(
        document.contains("\"surface\": \"personal-daemon-endpoint\""),
        "{document}"
    );
}
