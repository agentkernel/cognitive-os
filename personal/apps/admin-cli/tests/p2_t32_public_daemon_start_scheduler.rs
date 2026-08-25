//! P2-T32 public `cognitive daemon start` HTTP admit → scheduler lease.
//!
//! EVAL-007 skip class was observed on the product launcher, not on P2-T31's
//! test spawn of `kernel-server --personal` with piped stderr. This test starts
//! the daemon only through `cognitive daemon start`, installs a Unix stub
//! Workspace* private-candidate adapter (not bash/edit/write), and reads skip
//! diagnostics from the product `daemon.log`.
#![allow(
    dead_code,
    unused_imports,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

static P2_T32_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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
        std::env::temp_dir().join(format!("cos-p2t32-{}-{}", std::process::id(), free_port()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn cognitive() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cognitive"))
}

fn run_cognitive(args: &[&str]) -> Output {
    cognitive().args(args).output().expect("spawn cognitive")
}

fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[cfg(unix)]
fn kernel_server_binary() -> PathBuf {
    let cognitive = PathBuf::from(env!("CARGO_BIN_EXE_cognitive"));
    let sibling = cognitive.with_file_name("kernel-server");
    assert!(
        sibling.is_file(),
        "kernel-server binary missing at {}; build with `cargo build -p kernel-server` \
         before this suite (CI workspace builds both)",
        sibling.display()
    );
    sibling
}

fn daemon_log_path(runtime_root: &Path) -> PathBuf {
    runtime_root
        .join("state")
        .join("cognitiveos")
        .join("daemon.log")
}

fn bootstrap_secret_path(runtime_root: &Path) -> PathBuf {
    runtime_root
        .join("cognitiveos")
        .join("local-bootstrap.secret")
}

fn request(port: u16, wire: &str) -> String {
    let mut stream = connect_when_ready(port);
    stream.write_all(wire.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn connect_when_ready(port: u16) -> TcpStream {
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            return stream;
        }
        if std::time::Instant::now() >= deadline {
            panic!("daemon did not accept connections on 127.0.0.1:{port}");
        }
        thread::sleep(Duration::from_millis(20));
    }
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
    let start = response
        .find(marker)
        .unwrap_or_else(|| panic!("token missing from session response"))
        + marker.len();
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

fn redact_scheduler_log(raw: &str) -> String {
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
    fs::create_dir_all(&config_dir).unwrap();
    let adapter = config_dir.join("p2-t32-stub-adapter");
    let extension = config_dir.join("p2-t32-stub-extension.js");
    let executable = config_dir.join("p2-t32-stub-pi");
    fs::write(&extension, b"// stub candidate extension\n").unwrap();
    fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    let candidate = format!(
        "{{\"tool_ref\":\"native.workspace.search\",\"action\":\"search\",\"target\":\"workspace://\",\"parameters\":{{\"family\":\"WorkspaceSearch\",\"query\":\"needle\"}},\"parameters_digest\":\"{SEARCH_PARAMETERS_DIGEST}\",\"expected_state_version\":1,\"operation_descriptor_id\":\"{SEARCH_DESCRIPTOR_ID}\"}}"
    );
    let script = format!("#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{candidate}'\n");
    fs::write(&adapter, script).unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let pi_json = format!(
        "{{\"schema_version\":1,\"surface\":\"personal-pi-config\",\"executable_path\":\"{}\",\"extension_entry_path\":\"{}\",\"candidate_adapter_path\":\"{}\",\"candidate_extension_entry_path\":\"{}\"}}",
        json_path(&executable),
        json_path(&extension),
        json_path(&adapter),
        json_path(&extension)
    );
    fs::write(config_dir.join("pi.json"), pi_json).unwrap();
    fs::write(
        config_dir.join("selected-model.json"),
        "{\n  \"schema_version\": 1,\n  \"selected_model\": \"p2-t32-stub\",\n  \"selected_snapshot_digest\": \"fnv1a64:p2t32stub\",\n  \"chat_capable\": true\n}\n",
    )
    .unwrap();
}

#[cfg(unix)]
#[test]
fn public_cognitive_daemon_start_admit_leaves_draft_until_scheduler_acquires_lease() {
    let _guard = P2_T32_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let root = runtime_root();
    let log_path = daemon_log_path(&root);
    let kernel_server = kernel_server_binary();
    let workspace = root.join("data").join("cognitiveos").join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    fs::write(workspace.join("search.txt"), b"contains needle\n").unwrap();

    let init = run_cognitive(&["init", "--runtime-root", root.to_str().unwrap()]);
    assert!(
        init.status.success(),
        "init stdout={} stderr={}",
        stdout_str(&init),
        stderr_str(&init)
    );
    install_unix_stub_candidate_transport(&root);

    let start = run_cognitive(&[
        "daemon",
        "start",
        "--runtime-root",
        root.to_str().unwrap(),
        "--bind",
        &bind,
        "--kernel-server",
        kernel_server.to_str().unwrap(),
    ]);
    if !start.status.success() {
        let log_text = fs::read_to_string(&log_path).unwrap_or_default();
        let _ = run_cognitive(&["daemon", "stop", "--runtime-root", root.to_str().unwrap()]);
        let _ = fs::remove_dir_all(&root);
        panic!(
            "cognitive daemon start failed: stdout={} stderr={} daemon.log:\n{}",
            stdout_str(&start),
            stderr_str(&start),
            redact_scheduler_log(&log_text)
        );
    }
    let start_out = stdout_str(&start);
    assert!(
        start_out.contains("\"log_path\""),
        "start JSON must name daemon.log: {start_out}"
    );

    let secret = fs::read_to_string(bootstrap_secret_path(&root))
        .expect("bootstrap secret after daemon start")
        .trim()
        .to_owned();
    assert!(!secret.is_empty(), "bootstrap secret must be non-empty");
    let task_token = issue_token(port, &secret, "task");

    let recorded = response_json(&send_json(
        port,
        "/task/intent.record",
        &task_token,
        &json!({
            "conversation_or_scope_ref": "conversation://personal/p2-t32",
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
        "task_ref": "task://personal/p2-t32-public-daemon-start"
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
    assert_eq!(
        admitted["task_ref"],
        "task://personal/p2-t32-public-daemon-start"
    );

    thread::sleep(Duration::from_millis(3000));

    let evidence = response_json(&get(
        port,
        "/task/evidence?task_ref=task%3A%2F%2Fpersonal%2Fp2-t32-public-daemon-start",
        &task_token,
    ));
    let observation = response_json(&get(
        port,
        "/task/observation?family=o4&task_ref=task%3A%2F%2Fpersonal%2Fp2-t32-public-daemon-start",
        &task_token,
    ));
    let log_text = fs::read_to_string(&log_path).unwrap_or_default();
    let skip_lines = redact_scheduler_log(&log_text);
    let log_head = log_text.lines().take(6).collect::<Vec<_>>().join("\n");
    let _ = run_cognitive(&["daemon", "stop", "--runtime-root", root.to_str().unwrap()]);
    let _ = fs::remove_dir_all(&root);

    let state = evidence["lifecycle"]["current_state"]
        .as_str()
        .unwrap_or("");
    let lease_acquired = observation["counters"]["lease_acquired"]["count"]
        .as_u64()
        .unwrap_or(0);
    assert_ne!(
        state, "DRAFT",
        "EVAL-007 skip: public cognitive daemon start stayed DRAFT; lease_acquired={lease_acquired}; daemon.log scheduler:\n{skip_lines}\ndaemon.log head:\n{log_head}"
    );
    assert!(
        lease_acquired >= 1,
        "public launcher must acquire a scheduler lease; lease_acquired={lease_acquired}; daemon.log scheduler:\n{skip_lines}\ndaemon.log head:\n{log_head}"
    );
}

#[cfg(not(unix))]
#[test]
fn public_cognitive_daemon_start_unix_scheduler_not_run_on_windows() {
    // Owner-directed Linux-only route: Windows GitHub MSVC compiles this crate
    // but p1_t06 already avoids live `cognitive daemon start` on MSVC job objects.
}
