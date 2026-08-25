//! P1-T07 daemon-owned Provider proxy boundaries.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
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

fn create_runtime_root() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p1t07-provider-proxy-{}-{}",
        std::process::id(),
        free_port()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn spawn_personal_daemon(port: u16, runtime_root: &std::path::Path) -> Child {
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

fn exchange_http_request(port: u16, request: &str) -> String {
    let mut stream = common::connect_when_ready(port);
    // A daemon regression must fail this integration test instead of leaving a
    // Windows CI worker blocked forever while waiting for connection closure.
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn issue_management_token(port: u16, bootstrap_secret: &str) -> String {
    let body = format!(
        "{{\"channel\":\"management\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{bootstrap_secret}\"}}"
    );
    let request = format!(
        "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = exchange_http_request(port, &request);
    assert!(response.contains("HTTP/1.1 200"), "{response}");
    let token_key = "\"token\":\"";
    let token_start = response.find(token_key).expect("token field") + token_key.len();
    let token_end = response[token_start..].find('"').expect("token end") + token_start;
    response[token_start..token_end].to_owned()
}

#[test]
fn provider_proxy_requires_management_auth_and_fails_closed_without_provider_config() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let runtime_root = create_runtime_root();
    let mut daemon = spawn_personal_daemon(port, &runtime_root);
    let bootstrap_secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_management_token(port, &bootstrap_secret);
    let request_body = "{\"model\":\"test-model\",\"stream\":false,\"messages\":[]}";

    let unauthorized_request = format!(
        "POST /provider/v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{request_body}",
        request_body.len()
    );
    let unauthorized_response = exchange_http_request(port, &unauthorized_request);
    assert!(
        unauthorized_response.contains("LOCAL_SESSION_UNAUTHORIZED"),
        "{unauthorized_response}"
    );

    let authorized_request = format!(
        "POST /provider/v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{request_body}",
        request_body.len()
    );
    let authorized_response = exchange_http_request(port, &authorized_request);
    assert!(
        authorized_response.contains("PERSONAL_PROVIDER_NOT_CONFIGURED"),
        "{authorized_response}"
    );
    assert!(
        !authorized_response.contains(&management_token),
        "session credential leaked in proxy response: {authorized_response}"
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(&runtime_root);
}

#[test]
fn selected_model_projection_requires_management_auth_without_secret_resolution() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let runtime_root = create_runtime_root();
    let config_directory = runtime_root.join("config").join("cognitiveos");
    std::fs::create_dir_all(&config_directory).unwrap();
    std::fs::write(
        config_directory.join("selected-model.json"),
        "{\n  \"schema_version\": 1,\n  \"selected_model\": \"approved-model\",\n  \"selected_snapshot_digest\": \"fnv1a64:approved\",\n  \"chat_capable\": true\n}\n",
    )
    .unwrap();
    let mut daemon = spawn_personal_daemon(port, &runtime_root);
    let bootstrap_secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_management_token(port, &bootstrap_secret);

    let unauthorized_request =
        "GET /provider/v1/selected-model HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    let unauthorized_response = exchange_http_request(port, unauthorized_request);
    assert!(
        unauthorized_response.contains("LOCAL_SESSION_UNAUTHORIZED"),
        "{unauthorized_response}"
    );

    let authorized_request = format!(
        "GET /provider/v1/selected-model HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
    );
    let authorized_response = exchange_http_request(port, &authorized_request);
    assert!(
        authorized_response.contains("HTTP/1.1 200"),
        "{authorized_response}"
    );
    assert!(
        authorized_response.contains("\"selected_model\":\"approved-model\""),
        "{authorized_response}"
    );
    assert!(
        authorized_response.contains("\"chat_capable\":true"),
        "{authorized_response}"
    );
    assert!(
        !authorized_response.contains("secret_ref"),
        "{authorized_response}"
    );
    assert!(
        !authorized_response.contains(&management_token),
        "session credential leaked in selected-model response: {authorized_response}"
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(&runtime_root);
}
