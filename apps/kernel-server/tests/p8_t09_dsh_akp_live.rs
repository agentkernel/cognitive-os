//! P8-T09 live `POST /task/akp/dsh` → public WorkspaceRead admission.
//!
//! dsh remains candidate-only. A dsh response is never Task completion.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use cognitive_akp::deepseek_harness::{BRIDGE_PROTOCOL, PINNED_DSH_REVISION};
use serde_json::{Value, json};

static P8_T09_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p8t09-{}-{}", std::process::id(), free_port()));
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
    let encoded = body.to_string();
    request(
        port,
        &format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{encoded}",
            encoded.len()
        ),
    )
}

fn uuid7_like(kind: &str) -> String {
    let n = u64::from(std::process::id()) ^ u64::from(kind.len() as u32);
    let suffix = format!("{:012x}", n & 0x0000_ffff_ffff_ffff);
    let variant = if kind == "budget" { "8" } else { "9" };
    format!("00000000-0000-7000-{variant}000-{suffix}")
}

fn admit_read_task(port: u16, token: &str) -> String {
    let recorded = response_json(&send_json(
        port,
        "/task/intent.record",
        token,
        &json!({
            "conversation_or_scope_ref": "conversation://personal/p8-t09",
            "raw_expression": "read README.md",
            "schema_version": "cognitiveos.task-intent-record-request/0.1"
        }),
    ));
    let interpreted = response_json(&send_json(
        port,
        "/task/intent.interpret",
        token,
        &json!({
            "schema_version": "cognitiveos.task-intent-interpret-request/0.1",
            "user_intent_record_id": recorded["user_intent_record_id"],
            "candidate": {
                "objectives": ["read README.md"],
                "constraints": [],
                "forbidden": ["bash", "edit", "write"],
                "assumptions": [],
                "ambiguities": [],
                "information_gaps": []
            }
        }),
    ));
    let draft = json!({
        "allowed_state_domains": ["task", "effect"],
        "allowed_tools": ["native.workspace.read"],
        "budget": {"semantic_calls": 4, "tool_calls": 4},
        "budget_id": uuid7_like("budget"),
        "conditions": [{
            "description": "independent fixed-effect verification",
            "id": "acceptance",
            "kind": "acceptance",
            "verifier_ref": "verifier://personal/fixed-effect"
        }],
        "deadline": "2027-12-31T00:00:00Z",
        "loop_object_id": uuid7_like("loop"),
        "max_iterations": 4,
        "max_retries": 0,
        "objective": "read README.md",
        "scope": {
            "in_scope": ["workspace read"],
            "out_of_scope": ["bash", "edit", "write"]
        },
        "task_ref": "task://personal/p8-t09-dsh-read"
    });
    let previewed = response_json(&send_json(
        port,
        "/task/preview",
        token,
        &json!({
            "schema_version": "cognitiveos.task-preview-request/0.1",
            "task_contract_draft": draft
        }),
    ));
    let admitted = response_json(&send_json(
        port,
        "/task/admit",
        token,
        &json!({
            "schema_version": "cognitiveos.task-admit-request/0.1",
            "expected_current_epoch": 0,
            "preview_digest": previewed["preview_digest"],
            "task_contract_draft": draft,
            "acceptance": {
                "accepted_by": "principal://local/owner",
                "accepted_digest": interpreted["interpretation_digest"],
                "interpretation_id": interpreted["interpretation_id"]
            }
        }),
    ));
    admitted["task_ref"].as_str().expect("task_ref").to_owned()
}

fn dsh_event(sequence: u64, operation: &str, kind: &str, payload: Value) -> Value {
    json!({
        "op": "event",
        "bridge_protocol": BRIDGE_PROTOCOL,
        "dsh_version": PINNED_DSH_REVISION,
        "schema_digest": cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST,
        "session_id": "dsh-session-1",
        "fencing_epoch": 1,
        "sequence": sequence,
        "plugin_id": "plugin.core",
        "correlation_id": format!("dsh-session-1:{sequence}"),
        "deadline": "2030-01-01T00:00:00Z",
        "event": {
            "kind": kind,
            "operation": operation,
            "payload": payload,
            "authority_claim": false,
            "secret_shaped": false
        }
    })
}

#[test]
fn live_dsh_activate_workspace_read_stays_candidate_only() {
    let _guard = P8_T09_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let workspace = root.join("data").join("cognitiveos").join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(workspace.join("README.md"), b"p8-t09 disposable read\n").unwrap();

    let mut daemon = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let task_token = issue_token(port, &secret, "task");
    let task_ref = admit_read_task(port, &task_token);

    let wrong_version = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task_token,
        &json!({
            "op": "activate",
            "dsh_version": "0.0.0",
            "session_id": "dsh-session-1",
            "fencing_epoch": 1,
            "task_ref": task_ref
        }),
    ));
    assert_eq!(wrong_version["accepted"], false);
    assert_eq!(wrong_version["error"], "DSH_VERSION_MISMATCH");
    assert_eq!(wrong_version["candidate_only"], true);

    let activate = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task_token,
        &json!({
            "op": "activate",
            "dsh_version": PINNED_DSH_REVISION,
            "session_id": "dsh-session-1",
            "fencing_epoch": 1,
            "task_ref": task_ref
        }),
    ));
    assert_eq!(activate["accepted"], true);
    assert_eq!(activate["candidate_only"], true);

    let read = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task_token,
        &dsh_event(
            1,
            "WorkspaceRead",
            "candidate",
            json!({"target": "README.md"}),
        ),
    ));
    assert_eq!(read["accepted"], true);
    assert_eq!(read["candidate_only"], true);
    assert_eq!(read["sequence"], 1);
    assert_eq!(read["result"]["admission"]["admitted"], true);
    assert_eq!(read["result"]["candidate_only"], true);

    let duplicate = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task_token,
        &dsh_event(
            1,
            "WorkspaceRead",
            "candidate",
            json!({"target": "README.md"}),
        ),
    ));
    assert_eq!(duplicate["accepted"], false);
    assert_eq!(duplicate["error"], "SEQUENCE_NOT_MONOTONIC");

    let secret_shaped = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task_token,
        &dsh_event(
            2,
            "lifecycle.observe",
            "observation",
            json!({"api_key": "sk-example"}),
        ),
    ));
    assert_eq!(secret_shaped["accepted"], false);
    assert_eq!(secret_shaped["error"], "SECRET_SHAPED_PAYLOAD");

    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = std::fs::remove_dir_all(&root);
}
