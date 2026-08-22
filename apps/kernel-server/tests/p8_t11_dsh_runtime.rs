//! P8-T11 daemon-owned dsh runtime inspect (`GET`/`POST /personal/dsh/runtime`).
//!
//! Observation-only: sessions, fencing epoch, and `/proc/{pid}` existence.
//! Never opens cmdline/environ. A dsh response is never Task completion.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use cognitive_akp::deepseek_harness::PINNED_DSH_REVISION;
use serde_json::{Value, json};

static P8_T11_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p8t11-{}-{}", std::process::id(), free_port()));
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

fn get(port: u16, path: &str, token: Option<&str>) -> String {
    let authorization = token.map_or_else(String::new, |value| {
        format!("Authorization: Bearer {value}\r\n")
    });
    request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}Connection: close\r\n\r\n"
        ),
    )
}

#[test]
fn dsh_runtime_inspect_is_management_observation_only() {
    let _guard = P8_T11_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let mut daemon = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let management = issue_token(port, &secret, "management");
    let task = issue_token(port, &secret, "task");

    let unauthenticated = get(port, "/personal/dsh/runtime", None);
    assert!(
        unauthenticated.contains("HTTP/1.1 401"),
        "{unauthenticated}"
    );

    let task_channel = get(port, "/personal/dsh/runtime", Some(&task));
    assert!(
        task_channel.contains("HTTP/1.1 403") || task_channel.contains("HTTP/1.1 401"),
        "{task_channel}"
    );

    let inactive = response_json(&get(port, "/personal/dsh/runtime", Some(&management)));
    assert_eq!(inactive["surface"], "personal-dsh-runtime");
    assert_eq!(inactive["state"], "INACTIVE");
    assert_eq!(inactive["session_count"], 0);
    assert_eq!(inactive["candidate_only"], true);
    assert_eq!(inactive["dsh_response_is_not_task_completion"], true);

    let refused = send_json(
        port,
        "/personal/dsh/runtime",
        &management,
        &json!({"schema_version": 1, "surface": "personal-dsh-runtime", "op": "bind"}),
    );
    assert!(refused.contains("HTTP/1.1 400"), "{refused}");

    let activate = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task,
        &json!({
            "op": "activate",
            "dsh_version": PINNED_DSH_REVISION,
            "session_id": "dsh-session-runtime",
            "fencing_epoch": 1
        }),
    ));
    assert_eq!(activate["accepted"], true, "{activate}");

    let bound = response_json(&send_json(
        port,
        "/personal/dsh/runtime",
        &management,
        &json!({
            "schema_version": 1,
            "surface": "personal-dsh-runtime",
            "op": "bind",
            "process_id": std::process::id()
        }),
    ));
    assert_eq!(bound["state"], "ACTIVE");
    assert_eq!(bound["session_count"], 1);
    assert_eq!(bound["sessions"][0]["session_id"], "dsh-session-runtime");
    assert_eq!(bound["sessions"][0]["fencing_epoch"], 1);
    assert_eq!(bound["process_id"], std::process::id());
    assert_eq!(bound["authority_side_effects"], false);

    let heartbeat = response_json(&send_json(
        port,
        "/personal/dsh/runtime",
        &management,
        &json!({
            "schema_version": 1,
            "surface": "personal-dsh-runtime",
            "op": "heartbeat"
        }),
    ));
    assert_eq!(heartbeat["state"], "ACTIVE");
    assert!(heartbeat["last_heartbeat_unix_ms"].as_u64().unwrap() > 0);

    let stop = response_json(&send_json(
        port,
        "/task/akp/dsh",
        &task,
        &json!({
            "op": "stop",
            "session_id": "dsh-session-runtime"
        }),
    ));
    assert_eq!(stop["accepted"], true, "{stop}");

    let cleared = response_json(&send_json(
        port,
        "/personal/dsh/runtime",
        &management,
        &json!({
            "schema_version": 1,
            "surface": "personal-dsh-runtime",
            "op": "clear"
        }),
    ));
    assert_eq!(cleared["state"], "INACTIVE");
    assert_eq!(cleared["process_id"], Value::Null);

    #[cfg(unix)]
    {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let pid = child.id();
        child.kill().unwrap();
        let _ = child.wait();
        let crashed = response_json(&send_json(
            port,
            "/personal/dsh/runtime",
            &management,
            &json!({
                "schema_version": 1,
                "surface": "personal-dsh-runtime",
                "op": "bind",
                "process_id": pid
            }),
        ));
        assert_eq!(crashed["state"], "CRASHED", "{crashed}");
        assert_eq!(crashed["process_alive"], false);
        let _ = send_json(
            port,
            "/personal/dsh/runtime",
            &management,
            &json!({
                "schema_version": 1,
                "surface": "personal-dsh-runtime",
                "op": "clear"
            }),
        );
    }

    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = std::fs::remove_dir_all(&root);
}
