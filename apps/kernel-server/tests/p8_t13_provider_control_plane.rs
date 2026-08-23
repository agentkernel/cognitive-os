//! P8-T13 Provider Control Plane failure-first coverage.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::ops::{Deref, DerefMut};
use std::process::{Child, Command};
use std::time::Duration;

use serde_json::{Value, json};

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

struct PersonalProcess(Child);

impl Deref for PersonalProcess {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PersonalProcess {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for PersonalProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_personal(port: u16, runtime_root: &std::path::Path) -> PersonalProcess {
    PersonalProcess(
        Command::new(env!("CARGO_BIN_EXE_kernel-server"))
            .args([
                "--personal",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--runtime-root",
                runtime_root.to_str().unwrap(),
            ])
            .spawn()
            .unwrap(),
    )
}

fn issue_token(port: u16, secret: &str, channel: &str) -> String {
    let body = format!(
        "{{\"channel\":\"{channel}\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{secret}\"}}"
    );
    let marker = "\"token\":\"";
    let mut last_response = String::new();
    for _ in 0..100 {
        last_response = request(
            port,
            &format!(
                "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            ),
        );
        if let Some(start) = last_response
            .find(marker)
            .map(|offset| offset + marker.len())
            && let Some(length) = last_response[start..].find('"')
        {
            return last_response[start..start + length].to_owned();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("personal daemon did not issue a {channel} token; last response: {last_response}")
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

fn get(port: u16, path: &str, token: &str) -> String {
    request(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    )
}

fn response_json(response: &str) -> Value {
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response has a header/body separator");
    serde_json::from_str(body).expect("HTTP response body is JSON")
}

fn create_account(port: u16, token: &str, body: &Value) -> Value {
    let response = send_json(port, "POST", "/management/providers/accounts", token, body);
    assert!(response.contains("200 OK"), "{response}");
    response_json(&response)
}

fn add_manual_model(port: u16, token: &str, account_id: &str, model_id: &str) {
    let response = send_json(
        port,
        "POST",
        "/management/providers/models/add",
        token,
        &json!({"account_id": account_id, "model_id": model_id}),
    );
    assert!(response.contains("200 OK"), "{response}");
}

fn set_binding(port: u16, token: &str, agent: &str, account_id: &str, model_id: &str) {
    let response = send_json(
        port,
        "POST",
        "/management/agent-bindings",
        token,
        &json!({
            "agent": agent,
            "account_id": account_id,
            "model_id": model_id
        }),
    );
    assert!(response.contains("200 OK"), "{response}");
}

#[test]
fn control_plane_refuses_unauth_task_channel_and_untrusted_endpoints() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t13-negatives-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let task_token = issue_token(port, &secret, "task");

    let unauth = request(
        port,
        "GET /management/providers/accounts HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(unauth.contains("401 Unauthorized"), "{unauth}");

    let task_management = get(port, "/management/providers/accounts", &task_token);
    assert!(
        task_management.contains("403 Forbidden"),
        "{task_management}"
    );

    let task_channel = get(port, "/task/providers/accounts", &task_token);
    assert!(task_channel.contains("403 Forbidden"), "{task_channel}");
    assert!(task_channel.contains("PROVIDER_CONTROL_CHANNEL_FORBIDDEN"));

    let embedded = send_json(
        port,
        "POST",
        "/management/providers/accounts",
        &management_token,
        &json!({
            "display_name": "embedded-creds",
            "provider_kind": "openai_compatible",
            "endpoint": "https://user:pass@api.example.test/v1"
        }),
    );
    assert!(embedded.contains("400 Bad Request"), "{embedded}");
    assert!(embedded.contains("PROVIDER_ENDPOINT_EMBEDDED_CREDENTIALS"));

    let http = send_json(
        port,
        "POST",
        "/management/providers/accounts",
        &management_token,
        &json!({
            "display_name": "plain-http",
            "provider_kind": "openai_compatible",
            "endpoint": "http://api.example.test/v1"
        }),
    );
    assert!(http.contains("400 Bad Request"), "{http}");
    assert!(http.contains("PROVIDER_ENDPOINT_HTTP_REQUIRES_GRANT"));

    let private = send_json(
        port,
        "POST",
        "/management/providers/accounts",
        &management_token,
        &json!({
            "display_name": "lan-box",
            "provider_kind": "openai_compatible",
            "endpoint": "https://10.0.0.8/v1"
        }),
    );
    assert!(private.contains("400 Bad Request"), "{private}");
    assert!(private.contains("PROVIDER_ENDPOINT_PRIVATE_REQUIRES_GRANT"));

    let anthropic_compatible = send_json(
        port,
        "POST",
        "/management/providers/accounts",
        &management_token,
        &json!({
            "display_name": "third-party-anthropic",
            "provider_kind": "anthropic_compatible",
            "endpoint": "https://api.example.test/v1"
        }),
    );
    assert!(
        anthropic_compatible.contains("400 Bad Request"),
        "{anthropic_compatible}"
    );
    assert!(anthropic_compatible.contains("PROVIDER_ENDPOINT_ANTHROPIC_COMPATIBLE_FORBIDDEN"));

    let headers = send_json(
        port,
        "POST",
        "/management/providers/accounts",
        &management_token,
        &json!({
            "display_name": "header-injection",
            "provider_kind": "openai_official",
            "headers": {"Authorization": "Bearer injected"}
        }),
    );
    assert!(headers.contains("400 Bad Request"), "{headers}");
    assert!(headers.contains("PROVIDER_ENDPOINT_HEADER_INJECTION_FORBIDDEN"));

    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn create_without_key_preserves_manual_catalog_and_blocks_delete_with_binding() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t13-catalog-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");

    let created = create_account(
        port,
        &management_token,
        &json!({
            "display_name": "openai-work",
            "provider_kind": "openai_official"
        }),
    );
    let account_id = created["account"]["id"].as_str().unwrap().to_owned();
    assert_eq!(created["account"]["status"], "revoked");
    assert!(created["account"]["secret_ref"].is_null());

    add_manual_model(port, &management_token, &account_id, "gpt-4o-mini");
    set_binding(port, &management_token, "pi", &account_id, "gpt-4o-mini");

    let refresh = send_json(
        port,
        "POST",
        "/management/providers/models/refresh",
        &management_token,
        &json!({"id": account_id}),
    );
    assert!(refresh.contains("409 Conflict"), "{refresh}");
    assert!(refresh.contains("PROVIDER_KEY_MISSING"));

    let models = get(
        port,
        &format!("/management/providers/models?account_id={account_id}"),
        &management_token,
    );
    assert!(models.contains("200 OK"), "{models}");
    let models_json = response_json(&models);
    let listed = models_json["models"].as_array().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["model_id"], "gpt-4o-mini");
    assert_eq!(listed[0]["source"], "manually_configured");

    let delete = send_json(
        port,
        "POST",
        "/management/providers/accounts/delete",
        &management_token,
        &json!({"id": account_id}),
    );
    assert!(delete.contains("409 Conflict"), "{delete}");
    assert!(delete.contains("PROVIDER_CONTROL_CONFLICT"));

    let sqlite = runtime_root
        .join("data")
        .join("cognitiveos")
        .join("authority.sqlite");
    let bytes = std::fs::read(&sqlite).expect("authority sqlite");
    let haystack = String::from_utf8_lossy(&bytes);
    assert!(
        !haystack.contains("sk-"),
        "authority sqlite must not contain API key material"
    );
    assert!(!haystack.contains("Bearer "));

    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn pi_and_dsh_bindings_are_isolated_before_secret_store() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t13-bind-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");

    let pi_account = create_account(
        port,
        &management_token,
        &json!({
            "display_name": "openai-work",
            "provider_kind": "openai_official"
        }),
    );
    let dsh_account = create_account(
        port,
        &management_token,
        &json!({
            "display_name": "openai-lab",
            "provider_kind": "openai_official"
        }),
    );
    let pi_id = pi_account["account"]["id"].as_str().unwrap().to_owned();
    let dsh_id = dsh_account["account"]["id"].as_str().unwrap().to_owned();
    add_manual_model(port, &management_token, &pi_id, "gpt-4o-mini");
    add_manual_model(port, &management_token, &dsh_id, "deepseek-chat");
    set_binding(port, &management_token, "pi", &pi_id, "gpt-4o-mini");
    set_binding(port, &management_token, "dsh", &dsh_id, "deepseek-chat");

    let pi_mismatch = send_json(
        port,
        "POST",
        "/provider/v1/chat/completions",
        &management_token,
        &json!({
            "model": "deepseek-chat",
            "messages": [{"role":"user","content":"hi"}]
        }),
    );
    assert!(pi_mismatch.contains("400 Bad Request"), "{pi_mismatch}");
    assert!(pi_mismatch.contains("PERSONAL_PROVIDER_BINDING_MISMATCH"));

    let dsh_rewritten = send_json(
        port,
        "POST",
        "/provider/v1/dsh/chat/completions",
        &management_token,
        &json!({
            "model": "gpt-4o-mini",
            "messages": [{"role":"user","content":"hi"}]
        }),
    );
    assert!(dsh_rewritten.contains("409 Conflict"), "{dsh_rewritten}");
    assert!(dsh_rewritten.contains("PERSONAL_PROVIDER_ACCOUNT_UNAVAILABLE"));
    assert!(!dsh_rewritten.contains("PERSONAL_PROVIDER_BINDING_MISMATCH"));

    let pi_bound = send_json(
        port,
        "POST",
        "/provider/v1/chat/completions",
        &management_token,
        &json!({
            "model": "gpt-4o-mini",
            "messages": [{"role":"user","content":"hi"}]
        }),
    );
    assert!(pi_bound.contains("409 Conflict"), "{pi_bound}");
    assert!(pi_bound.contains("PERSONAL_PROVIDER_ACCOUNT_UNAVAILABLE"));

    let selected_pi = get(port, "/provider/v1/selected-model", &management_token);
    assert!(selected_pi.contains("200 OK"), "{selected_pi}");
    let selected_pi_json = response_json(&selected_pi);
    assert_eq!(selected_pi_json["selected_model"], "gpt-4o-mini");
    assert_eq!(selected_pi_json["selected_snapshot_digest"], "binding");

    let selected_dsh = get(port, "/provider/v1/dsh/selected-model", &management_token);
    assert!(selected_dsh.contains("200 OK"), "{selected_dsh}");
    let selected_dsh_json = response_json(&selected_dsh);
    assert_eq!(selected_dsh_json["selected_model"], "deepseek-chat");
    assert_eq!(selected_dsh_json["binding_agent"], "agent://personal/dsh");

    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn binding_expected_revision_rejects_stale_cas() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t13-cas-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");

    let created = create_account(
        port,
        &management_token,
        &json!({
            "display_name": "openai-cas",
            "provider_kind": "openai_official"
        }),
    );
    let account_id = created["account"]["id"].as_str().unwrap().to_owned();
    add_manual_model(port, &management_token, &account_id, "gpt-4o-mini");

    let first = send_json(
        port,
        "POST",
        "/management/agent-bindings",
        &management_token,
        &json!({
            "agent": "pi",
            "account_id": account_id,
            "model_id": "gpt-4o-mini",
            "expected_revision": 0
        }),
    );
    assert!(first.contains("200 OK"), "{first}");

    let stale = send_json(
        port,
        "POST",
        "/management/agent-bindings",
        &management_token,
        &json!({
            "agent": "pi",
            "account_id": account_id,
            "model_id": "gpt-4o-mini",
            "expected_revision": 0
        }),
    );
    assert!(stale.contains("409 Conflict"), "{stale}");
    assert!(stale.contains("PROVIDER_BINDING_REVISION_STALE"), "{stale}");

    let current = send_json(
        port,
        "POST",
        "/management/agent-bindings",
        &management_token,
        &json!({
            "agent": "pi",
            "account_id": account_id,
            "model_id": "gpt-4o-mini",
            "expected_revision": 1
        }),
    );
    assert!(current.contains("200 OK"), "{current}");

    let _ = std::fs::remove_dir_all(runtime_root);
}
