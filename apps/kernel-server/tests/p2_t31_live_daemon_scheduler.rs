//! P2-T31 live `kernel-server` HTTP admit → periodic scheduler tick.
//!
//! EVAL-006 skip class `scheduler_row_skip_before_lease` was observed on the
//! production daemon, not on P2-T30's in-process `TaskApi::handle` plus
//! `DeterministicProductionChainProposer` fixture. This test drives the public
//! HTTP path against a spawned Personal daemon, installs a Unix stub private-
//! candidate adapter (not bash/edit/write), and retains scheduler stderr.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

static P2_T31_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

const SEARCH_PARAMETERS_DIGEST: &str =
    "sha256:fa38ed3a81b5d77594862fe780acd8c0382b96171f007eb7a07916f7beba4fd5";
const SEARCH_DESCRIPTOR_ID: &str = "00000000-0000-7000-8000-000000002002";

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t31-{}-{}", std::process::id(), free_port()));
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

fn spawn_personal(port: u16, runtime_root: &Path) -> (Child, Arc<Mutex<String>>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args([
            "--personal",
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--runtime-root",
            runtime_root.to_str().unwrap(),
        ])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().expect("piped stderr");
    let captured = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&captured);
    thread::spawn(move || {
        let mut reader = stderr;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    if let Ok(mut held) = sink.lock() {
                        held.push_str(&String::from_utf8_lossy(&buffer[..count]));
                    }
                }
            }
        }
    });
    (child, captured)
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

fn get(port: u16, path: &str, token: &str) -> String {
    request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    )
}

fn redact_scheduler_stderr(raw: &str) -> String {
    raw.lines()
        .filter(|line| line.contains("scheduler tick") || line.contains("skip row"))
        .take(8)
        .collect::<Vec<_>>()
        .join("\n")
}

fn uuid7_like(kind: &str) -> String {
    let n = u64::from(std::process::id()) ^ u64::from(kind.len() as u32);
    let suffix = format!("{:012x}", n & 0x0000_ffff_ffff_ffff);
    let variant = if kind == "budget" { "8" } else { "9" };
    format!("00000000-0000-7000-{variant}000-{suffix}")
}

fn json_path(path: &Path) -> String {
    path.to_str()
        .unwrap()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

/// Install a daemon-governed Unix stub adapter plus `pi.json` and selected-model
/// so the live periodic tick can invoke private-candidate transport without
/// bash/edit/write or a real Provider.
#[cfg(unix)]
fn install_unix_stub_candidate_transport(runtime_root: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let config_dir = runtime_root.join("config").join("cognitiveos");
    std::fs::create_dir_all(&config_dir).unwrap();
    let adapter = config_dir.join("p2-t31-stub-adapter");
    let extension = config_dir.join("p2-t31-stub-extension.js");
    let executable = config_dir.join("p2-t31-stub-pi");
    std::fs::write(&extension, b"// stub candidate extension\n").unwrap();
    std::fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    let candidate = format!(
        "{{\"tool_ref\":\"native.workspace.search\",\"action\":\"search\",\"target\":\"workspace://\",\"parameters\":{{\"family\":\"WorkspaceSearch\",\"query\":\"needle\"}},\"parameters_digest\":\"{SEARCH_PARAMETERS_DIGEST}\",\"expected_state_version\":1,\"operation_descriptor_id\":\"{SEARCH_DESCRIPTOR_ID}\"}}"
    );
    let script = format!("#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{candidate}'\n");
    std::fs::write(&adapter, script).unwrap();
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let pi_json = format!(
        "{{\"schema_version\":1,\"surface\":\"personal-pi-config\",\"executable_path\":\"{}\",\"extension_entry_path\":\"{}\",\"candidate_adapter_path\":\"{}\",\"candidate_extension_entry_path\":\"{}\"}}",
        json_path(&executable),
        json_path(&extension),
        json_path(&adapter),
        json_path(&extension)
    );
    std::fs::write(config_dir.join("pi.json"), pi_json).unwrap();
    std::fs::write(
        config_dir.join("selected-model.json"),
        "{\n  \"schema_version\": 1,\n  \"selected_model\": \"p2-t31-stub\",\n  \"selected_snapshot_digest\": \"fnv1a64:p2t31stub\",\n  \"chat_capable\": true\n}\n",
    )
    .unwrap();
}

#[cfg(unix)]
#[test]
fn live_http_admit_c1_search_leaves_draft_until_scheduler_acquires_lease() {
    let _guard = P2_T31_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let workspace = root.join("data").join("cognitiveos").join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(workspace.join("search.txt"), b"contains needle\n").unwrap();
    install_unix_stub_candidate_transport(&root);

    let (mut daemon, stderr) = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let task_token = issue_token(port, &secret, "task");

    let recorded = response_json(&send_json(
        port,
        "/task/intent.record",
        &task_token,
        &json!({
            "conversation_or_scope_ref": "conversation://personal/p2-t31",
            "raw_expression": "search the workspace for needle",
            "schema_version": "cognitiveos.task-intent-record-request/0.1"
        }),
    ));
    let user_intent_record_id = recorded["user_intent_record_id"].as_str().unwrap();
    let interpreted = response_json(&send_json(
        port,
        "/task/intent.interpret",
        &task_token,
        &json!({
            "schema_version": "cognitiveos.task-intent-interpret-request/0.1",
            "user_intent_record_id": user_intent_record_id,
            "candidate": {
                "objectives": ["search the workspace for needle"],
                "constraints": [],
                "forbidden": ["bash", "edit", "write"],
                "assumptions": [],
                "ambiguities": [],
                "information_gaps": []
            }
        }),
    ));
    let loop_object_id = uuid7_like("loop");
    let budget_id = uuid7_like("budget");
    let draft = json!({
        "allowed_state_domains": ["task", "effect"],
        "allowed_tools": ["native.workspace.search"],
        "budget": {"semantic_calls": 4, "tool_calls": 4},
        "budget_id": budget_id,
        "conditions": [{
            "description": "independent fixed-effect verification",
            "id": "acceptance",
            "kind": "acceptance",
            "verifier_ref": "verifier://personal/fixed-effect"
        }],
        "deadline": "2027-12-31T00:00:00Z",
        "loop_object_id": loop_object_id,
        "max_iterations": 4,
        "max_retries": 0,
        "objective": "search the workspace for needle",
        "scope": {
            "in_scope": ["workspace search"],
            "out_of_scope": ["bash", "edit", "write"]
        },
        "task_ref": "task://personal/p2-t31-live-daemon"
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
    assert_eq!(admitted["task_ref"], "task://personal/p2-t31-live-daemon");

    // Several 250 ms production ticks: stub Pi on tick 1, lease on tick 2.
    thread::sleep(Duration::from_millis(3000));

    let evidence = response_json(&get(
        port,
        "/task/evidence?task_ref=task%3A%2F%2Fpersonal%2Fp2-t31-live-daemon",
        &task_token,
    ));
    let observation = response_json(&get(
        port,
        "/task/observation?family=o4&task_ref=task%3A%2F%2Fpersonal%2Fp2-t31-live-daemon",
        &task_token,
    ));
    let skip_lines = redact_scheduler_stderr(&stderr.lock().unwrap());
    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = std::fs::remove_dir_all(&root);

    let state = evidence["lifecycle"]["current_state"]
        .as_str()
        .unwrap_or("");
    let lease_acquired = observation["counters"]["lease_acquired"]["count"]
        .as_u64()
        .unwrap_or(0);
    assert_ne!(
        state, "DRAFT",
        "EVAL-006 skip: live HTTP admit stayed DRAFT; lease_acquired={lease_acquired}; scheduler stderr:\n{skip_lines}"
    );
    assert!(
        lease_acquired >= 1,
        "live daemon must acquire a scheduler lease; lease_acquired={lease_acquired}; scheduler stderr:\n{skip_lines}"
    );
}

#[cfg(not(unix))]
#[test]
fn live_http_admit_unix_private_candidate_transport_not_run_on_windows() {
    // Owner-directed Linux-only route: Windows GitHub MSVC compiles this crate
    // but cannot invoke Unix-domain private-candidate transport.
}
