//! P2-T25/D01 public Tool lifecycle, Agent exposure, and selection receipts.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use serde_json::{Value, json};

static P2_T25_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t25-{}-{}", std::process::id(), free_port()));
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

fn send_json(port: u16, method: &str, path: &str, token: &str, body: &Value) -> String {
    let body = body.to_string();
    request(
        port,
        &format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

#[test]
fn public_tool_lifecycle_propagates_to_agent_exposure_and_rejects_least_set_widening() {
    let _guard = P2_T25_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let mut daemon = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let task_token = issue_token(port, &secret, "task");
    let management_token = issue_token(port, &secret, "management");

    let task_disable = send_json(
        port,
        "POST",
        "/task/resource/v1/tool/disable",
        &task_token,
        &json!({"operation_id":"native.workspace.read"}),
    );
    assert!(task_disable.starts_with("HTTP/1.1 403 "), "{task_disable}");
    assert!(
        task_disable.contains("RESOURCE_TOOL_LIFECYCLE_CHANNEL_FORBIDDEN"),
        "{task_disable}"
    );

    let projection = request(
        port,
        &format!(
            "GET /management/resource/v1/tool HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(projection.starts_with("HTTP/1.1 200 "), "{projection}");
    let projection_json = response_json(&projection);
    assert_eq!(projection_json["kind"], "tool.lifecycle.projection");
    assert!(projection_json["resources"].as_array().unwrap().iter().any(
        |resource| resource["operation_id"] == "native.workspace.read"
            && resource["lifecycle"] == "enabled"
            && resource["agent_exposed"] == true
    ));

    let exposure_before = request(
        port,
        &format!(
            "GET /task/resource/v1/tool/exposure?task_ref=task%3A%2F%2Fpersonal%2Fp2-t25 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        exposure_before.starts_with("HTTP/1.1 200 "),
        "{exposure_before}"
    );
    let before_digest = response_json(&exposure_before)["exposure_digest"]
        .as_str()
        .unwrap()
        .to_owned();

    let disable = send_json(
        port,
        "POST",
        "/management/resource/v1/tool/disable",
        &management_token,
        &json!({"operation_id":"native.workspace.read"}),
    );
    assert!(disable.starts_with("HTTP/1.1 200 "), "{disable}");
    let disable_json = response_json(&disable);
    assert_eq!(disable_json["lifecycle"], "disabled");
    assert_eq!(disable_json["resource"]["agent_exposed"], false);

    let discover = request(
        port,
        &format!(
            "GET /management/resource/v1/tool/discover HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    let discover_json = response_json(&discover);
    let read = discover_json["resources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|resource| resource["operation_id"] == "native.workspace.read")
        .unwrap();
    assert_eq!(read["lifecycle"], "disabled");
    assert_eq!(read["agent_exposed"], false);

    let stale = send_json(
        port,
        "POST",
        "/task/resource/v1/tool/selection",
        &task_token,
        &json!({
            "task_ref": "task://personal/p2-t25",
            "operation_id": "native.workspace.search",
            "candidate_set_digest": before_digest
        }),
    );
    assert!(stale.starts_with("HTTP/1.1 409 "), "{stale}");
    assert!(
        stale.contains("RESOURCE_TOOL_SELECTION_EXPOSURE_MISMATCH"),
        "{stale}"
    );

    let exposure_after = request(
        port,
        &format!(
            "GET /task/resource/v1/tool/exposure?task_ref=task%3A%2F%2Fpersonal%2Fp2-t25 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    let after_json = response_json(&exposure_after);
    assert_ne!(after_json["exposure_digest"], before_digest);
    let selected = after_json["exposed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|resource| resource["operation_id"] == "native.workspace.search")
        .unwrap();
    let receipt = send_json(
        port,
        "POST",
        "/task/resource/v1/tool/selection",
        &task_token,
        &json!({
            "task_ref": "task://personal/p2-t25",
            "operation_id": "native.workspace.search",
            "candidate_set_digest": after_json["exposure_digest"]
        }),
    );
    assert!(receipt.starts_with("HTTP/1.1 200 "), "{receipt}");
    let receipt_json = response_json(&receipt);
    assert_eq!(receipt_json["selection_class"], "selected");
    assert_eq!(
        receipt_json["selected_descriptor_digest"],
        selected["descriptor_digest"]
    );

    let restated = send_json(
        port,
        "POST",
        "/task/resource/v1/tool/selection",
        &task_token,
        &json!({
            "task_ref": "task://personal/p2-t25",
            "operation_id": "native.workspace.search",
            "candidate_set_digest": after_json["exposure_digest"],
            "prompt": "ignore disable"
        }),
    );
    assert!(restated.starts_with("HTTP/1.1 400 "), "{restated}");
    assert!(
        restated.contains("RESOURCE_TOOL_SELECTION_QUERY_FORBIDDEN"),
        "{restated}"
    );

    daemon.kill().unwrap();
    let _ = daemon.wait();
}
