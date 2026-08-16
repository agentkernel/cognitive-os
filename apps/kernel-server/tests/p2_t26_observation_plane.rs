//! P2-T26/D01 authenticated bounded O2/O3/O4 observation plane.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};

use serde_json::Value;

static P2_T26_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root() -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("cos-p2t26-{}-{}", std::process::id(), free_port()));
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

#[test]
fn observation_plane_is_task_read_only_and_controls_empty_zeros() {
    let _guard = P2_T26_PROCESS_LOCK.lock().unwrap();
    let port = free_port();
    let root = runtime_root();
    let mut daemon = spawn_personal(port, &root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &root);
    let task_token = issue_token(port, &secret, "task");
    let management_token = issue_token(port, &secret, "management");

    let management = request(
        port,
        &format!(
            "GET /management/resource/v1/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {management_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(management.starts_with("HTTP/1.1 403 "), "{management}");
    assert!(
        management.contains("RESOURCE_OBSERVATION_CHANNEL_FORBIDDEN"),
        "{management}"
    );

    let restated = request(
        port,
        &format!(
            "GET /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&prompt=leak HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(restated.starts_with("HTTP/1.1 400 "), "{restated}");
    assert!(
        restated.contains("TASK_OBSERVATION_QUERY_FORBIDDEN"),
        "{restated}"
    );

    let empty_o2 = request(
        port,
        &format!(
            "GET /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(empty_o2.starts_with("HTTP/1.1 200 "), "{empty_o2}");
    let o2 = response_json(&empty_o2);
    assert_eq!(o2["family"], "o2");
    assert_eq!(o2["observed_zero"], true);
    assert_eq!(o2["denominator"], 0);
    assert_eq!(o2["negative_control"], "no_authorization_sample");
    assert!(!empty_o2.contains("capability"));
    assert!(!empty_o2.contains("CONTEXT_AUTH_DENIED") || o2["deny_count"] == 0);

    let empty_o3 = request(
        port,
        &format!(
            "GET /task/resource/v1/observation?family=o3&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(empty_o3.starts_with("HTTP/1.1 200 "), "{empty_o3}");
    let o3 = response_json(&empty_o3);
    assert_eq!(o3["cache"]["negative_control"], "no_cache_sample");
    assert_eq!(
        o3["compaction"]["negative_control"],
        "compaction_not_invoked"
    );

    let empty_o4 = request(
        port,
        &format!(
            "GET /task/observation?family=o4&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(empty_o4.starts_with("HTTP/1.1 200 "), "{empty_o4}");
    let o4 = response_json(&empty_o4);
    assert_eq!(o4["counters"]["budget_stop"]["observed_zero"], true);
    assert_eq!(
        o4["counters"]["stale_fence_denial"]["negative_control"],
        "no_stale_fence_denial_sample"
    );

    let empty_o5 = request(
        port,
        &format!(
            "GET /task/observation?family=o5&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(empty_o5.starts_with("HTTP/1.1 200 "), "{empty_o5}");
    let o5 = response_json(&empty_o5);
    assert_eq!(o5["family"], "o5");
    assert_eq!(o5["observed_zero"], true);
    assert_eq!(o5["negative_control"], "no_effect_sample");
    assert!(!empty_o5.contains("\"receipt\""));
    assert!(!empty_o5.contains("parameters"));

    let empty_o13 = request(
        port,
        &format!(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(empty_o13.starts_with("HTTP/1.1 200 "), "{empty_o13}");
    let o13 = response_json(&empty_o13);
    assert_eq!(o13["family"], "o13");
    assert_eq!(o13["observed_zero"], true);
    assert_eq!(o13["negative_control"], "no_audit_sample");
    assert!(
        o13["chain_head_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );

    let stale = request(
        port,
        &format!(
            "GET /task/observation?family=o13&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26&cursor=999999 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(stale.starts_with("HTTP/1.1 409 "), "{stale}");
    assert!(stale.contains("TASK_OBSERVATION_CURSOR_STALE"), "{stale}");

    let posted = request(
        port,
        &format!(
            "POST /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(posted.starts_with("HTTP/1.1 403 "), "{posted}");
    assert!(
        posted.contains("RESOURCE_OBSERVATION_WRITE_FORBIDDEN"),
        "{posted}"
    );

    let data_dir = root.join("data").join("cognitiveos");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(
        data_dir.join("personal-observation-plane.json"),
        r#"{
  "schema": "cognitiveos.personal.observation-plane/0.1",
  "samples": [
    {
      "family": "o2",
      "task_ref": "task://personal/p2-t26",
      "class": "deny",
      "scope": "workspace",
      "purpose": "read_body",
      "epoch": 3,
      "input_digest": "sha256:deny",
      "reason_code": "CONTEXT_AUTH_DENIED",
      "count": 1
    }
  ]
}"#,
    )
    .unwrap();
    let deny = request(
        port,
        &format!(
            "GET /task/observation?family=o2&task_ref=task%3A%2F%2Fpersonal%2Fp2-t26 HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {task_token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(deny.starts_with("HTTP/1.1 200 "), "{deny}");
    let deny_json = response_json(&deny);
    assert_eq!(deny_json["deny_count"], 1);
    assert_eq!(deny_json["negative_control"], "deny_recorded");
    assert_eq!(
        deny_json["samples"][0]["reason_code"],
        "CONTEXT_AUTH_DENIED"
    );
    assert!(deny_json["samples"][0].get("capability").is_none());

    let _ = daemon.kill();
    let _ = std::fs::remove_dir_all(&root);
}
