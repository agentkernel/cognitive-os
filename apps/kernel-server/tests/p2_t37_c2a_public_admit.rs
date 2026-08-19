//! P2-T37 D02 helper: admit a public WorkspaceWrite/Patch Task against an
//! already running Personal daemon.
//!
//! This is not part of required CI. Invoke it only on exact-revision
//! `DEV-LINUX-NATIVE-01` with `--ignored` after the disposable public daemon is
//! ready. It uses the same HTTP record/interpret/preview/admit path as P2-T31
//! and P2-T32. It never prints bootstrap material, session tokens, or Provider
//! state.
#![allow(
    dead_code,
    unused_imports,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

fn request(port: u16, wire: &str) -> String {
    let mut stream = common::connect_when_ready(port);
    stream.write_all(wire.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
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
    let start = response.find(marker).unwrap_or_else(|| {
        panic!(
            "session response missing token field; http={}",
            http_status(&response)
        )
    });
    let start = start + marker.len();
    let end = start + response[start..].find('"').unwrap();
    response[start..end].to_owned()
}

fn http_status(response: &str) -> String {
    response
        .lines()
        .next()
        .unwrap_or("missing-status")
        .to_owned()
}

fn response_json(response: &str) -> Value {
    let status = http_status(response);
    assert!(
        status.contains(" 200 "),
        "HTTP request failed: {status}; error={}",
        redacted_error(response)
    );
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response has a header/body separator");
    serde_json::from_str(body).expect("HTTP response body is JSON")
}

fn redacted_error(response: &str) -> String {
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| {
                    value
                        .get("error_class")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
        })
        .unwrap_or_else(|| "unredacted-body-omitted".to_owned())
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

fn get(port: u16, path: &str, token: &str) -> String {
    request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    )
}

fn uuid7_like(kind: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let suffix = format!(
        "{:012x}",
        (nanos ^ u128::from(kind.len() as u64)) & 0x0000_ffff_ffff_ffff
    );
    let variant = if kind == "budget" { "8" } else { "9" };
    format!("00000000-0000-7000-{variant}000-{suffix}")
}

fn encode_task_ref(task_ref: &str) -> String {
    let mut encoded = String::new();
    for byte in task_ref.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[test]
#[ignore = "live public-daemon admit; set COS_P2_T37_RUNTIME_ROOT and COS_P2_T37_PORT"]
fn admit_public_c2a_mutation_task_against_running_daemon() {
    let root = PathBuf::from(
        env::var("COS_P2_T37_RUNTIME_ROOT")
            .expect("COS_P2_T37_RUNTIME_ROOT must name the disposable runtime"),
    );
    let port: u16 = env::var("COS_P2_T37_PORT")
        .expect("COS_P2_T37_PORT must name the disposable loopback port")
        .parse()
        .expect("COS_P2_T37_PORT must be a u16");
    let task_ref = env::var("COS_P2_T37_TASK_REF")
        .unwrap_or_else(|_| "task://personal/p2-t37-public-write".to_owned());
    let tool = env::var("COS_P2_T37_TOOL").unwrap_or_else(|_| "native.workspace.write".to_owned());
    assert!(
        tool == "native.workspace.write" || tool == "native.workspace.patch",
        "COS_P2_T37_TOOL must be native.workspace.write or native.workspace.patch"
    );
    let (family, objective) = if tool.ends_with("write") {
        (
            "write",
            "mutate workspace through daemon-governed WorkspaceWrite",
        )
    } else {
        (
            "patch",
            "mutate workspace through daemon-governed WorkspacePatch",
        )
    };
    let secret = common::wait_for_bootstrap_secret(&root);
    let task_token = issue_token(port, &secret, "task");

    let recorded = response_json(&send_json(
        port,
        "/task/intent.record",
        &task_token,
        &json!({
            "conversation_or_scope_ref": format!("conversation://personal/p2-t37-{family}"),
            "raw_expression": objective,
            "schema_version": "cognitiveos.task-intent-record-request/0.1"
        }),
    ));
    let interpreted = response_json(&send_json(
        port,
        "/task/intent.interpret",
        &task_token,
        &json!({
            "schema_version": "cognitiveos.task-intent-interpret-request/0.1",
            "user_intent_record_id": recorded["user_intent_record_id"],
            "candidate": {
                "objectives": [objective],
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
        "allowed_tools": [tool],
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
        "objective": objective,
        "scope": {
            "in_scope": [format!("workspace {family}")],
            "out_of_scope": ["bash", "edit", "write"]
        },
        "task_ref": task_ref
    });
    let previewed = response_json(&send_json(
        port,
        "/task/preview",
        &task_token,
        &json!({
            "schema_version": "cognitiveos.task-preview-request/0.1",
            "task_contract_draft": draft
        }),
    ));
    let admitted = response_json(&send_json(
        port,
        "/task/admit",
        &task_token,
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
    assert_eq!(admitted["task_ref"].as_str().unwrap(), task_ref);

    let evidence = response_json(&get(
        port,
        &format!("/task/evidence?task_ref={}", encode_task_ref(&task_ref)),
        &task_token,
    ));
    let lifecycle = evidence["lifecycle"]["current_state"]
        .as_str()
        .unwrap_or("absent");
    let preview_digest_present = previewed
        .get("preview_digest")
        .and_then(Value::as_str)
        .map(|value| !value.is_empty())
        .unwrap_or(false);
    println!(
        "{}",
        json!({
            "admitted_task_ref": task_ref,
            "lifecycle": lifecycle,
            "has_preview_digest": preview_digest_present,
            "tool": tool
        })
    );
}
