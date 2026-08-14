//! P4-T05/D01 failure-first coverage for task-bound resource projection.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
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
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
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

    // `skill/bind` is a prefix of `skill/binding/revoke`. A channel-binding
    // assertion passes whichever handler runs, so this discriminates by a code
    // only the revoke handler can produce: the payload carries a valid
    // `binding_id` but no `revocation_id` and no `revision_id`, so the revoke
    // handler answers `RESOURCE_SKILL_REVOCATION_ID_INVALID` while the bind
    // handler would answer `RESOURCE_SKILL_ID_INVALID`.
    let revoke_route_body =
        "{\"binding_id\":\"00000000-0000-7000-8000-000000000001\",\"reason\":\"route probe\"}";
    let revoke_reaches_revoke_handler = request(
        port,
        &format!(
            "POST /management/resource/v1/skill/binding/revoke HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{revoke_route_body}",
            revoke_route_body.len()
        ),
    );
    assert!(
        revoke_reaches_revoke_handler.contains("RESOURCE_SKILL_REVOCATION_ID_INVALID"),
        "revoke route must reach the revoke handler, not the bind handler: {revoke_reaches_revoke_handler}"
    );
    assert!(
        !revoke_reaches_revoke_handler.contains("RESOURCE_SKILL_ID_INVALID"),
        "revoke route was shadowed by the bind prefix: {revoke_reaches_revoke_handler}"
    );

    let management_consumption_crossing = request(
        port,
        &format!(
            "POST /task/resource/v1/consumption HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{{"
        ),
    );
    assert!(
        management_consumption_crossing.contains("403 Forbidden"),
        "{management_consumption_crossing}"
    );
    assert!(management_consumption_crossing.contains("SHELL_CHANNEL_BINDING_MISMATCH"));

    let malformed_consumption = request(
        port,
        &format!(
            "POST /task/resource/v1/consumption HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{{"
        ),
    );
    assert!(
        malformed_consumption.contains("400 Bad Request"),
        "{malformed_consumption}"
    );
    assert!(malformed_consumption.contains("RESOURCE_CONSUMPTION_PAYLOAD_INVALID"));

    let unknown_task_body = "{\"task_ref\":\"task://local/missing\",\"query_text\":\"fact\",\"skill_binding_id\":\"00000000-0000-7000-9000-000000000001\"}";
    let unknown_task_consumption = request(
        port,
        &format!(
            "POST /task/resource/v1/consumption HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{unknown_task_body}",
            unknown_task_body.len()
        ),
    );
    assert!(
        unknown_task_consumption.contains("404 Not Found"),
        "{unknown_task_consumption}"
    );
    assert!(unknown_task_consumption.contains("RESOURCE_TASK_NOT_FOUND"));

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}
