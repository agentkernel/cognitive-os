//! P8-T12 Resource Manager failure-first coverage.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::ops::{Deref, DerefMut};
use std::process::{Child, Command};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cognitive_domain::ObjectId;
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

fn object_id(serial: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{serial:012x}")).unwrap()
}

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

#[test]
fn resource_manager_refuses_unknown_family_generic_create_task_channel_and_malformed_mutations() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t12-negatives-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let task_token = issue_token(port, &secret, "task");

    let unknown_family = get(
        port,
        "/management/resource/v1/list?family=widget",
        &management_token,
    );
    assert!(
        unknown_family.contains("400 Bad Request"),
        "{unknown_family}"
    );
    assert!(unknown_family.contains("RESOURCE_MANAGER_FAMILY_UNKNOWN"));

    let generic_create = send_json(
        port,
        "POST",
        "/management/resource/v1/create",
        &management_token,
        &json!({"family":"tool","id":"native.workspace.read","expected_version":1,"idempotency_key":"x"}),
    );
    assert!(
        generic_create.contains("400 Bad Request"),
        "{generic_create}"
    );
    assert!(generic_create.contains("RESOURCE_MANAGER_OPERATION_FORBIDDEN"));

    let task_list = get(
        port,
        "/management/resource/v1/list?family=tool",
        &task_token,
    );
    assert!(task_list.contains("403 Forbidden"), "{task_list}");

    let task_channel = get(port, "/task/resource/v1/list?family=tool", &task_token);
    assert!(task_channel.contains("403 Forbidden"), "{task_channel}");
    assert!(task_channel.contains("RESOURCE_MANAGER_CHANNEL_FORBIDDEN"));

    let missing_id = send_json(
        port,
        "POST",
        "/management/resource/v1/disable",
        &management_token,
        &json!({"family":"tool","expected_version":1,"idempotency_key":"k"}),
    );
    assert!(missing_id.contains("400 Bad Request"), "{missing_id}");
    assert!(missing_id.contains("RESOURCE_MANAGER_ID_REQUIRED"));

    let missing_version = send_json(
        port,
        "POST",
        "/management/resource/v1/disable",
        &management_token,
        &json!({"family":"tool","id":"native.workspace.read","idempotency_key":"k"}),
    );
    assert!(
        missing_version.contains("400 Bad Request"),
        "{missing_version}"
    );
    assert!(missing_version.contains("RESOURCE_MANAGER_VERSION_INVALID"));

    let missing_idempotency = send_json(
        port,
        "POST",
        "/management/resource/v1/disable",
        &management_token,
        &json!({"family":"tool","id":"native.workspace.read","expected_version":1}),
    );
    assert!(
        missing_idempotency.contains("400 Bad Request"),
        "{missing_idempotency}"
    );
    assert!(missing_idempotency.contains("RESOURCE_MANAGER_IDEMPOTENCY_REQUIRED"));

    let unsupported = send_json(
        port,
        "POST",
        "/management/resource/v1/bind",
        &management_token,
        &json!({
            "family":"memory",
            "id":"00000000-0000-7000-9000-000000000001",
            "expected_version":0,
            "idempotency_key":"mem-bind"
        }),
    );
    assert!(unsupported.contains("400 Bad Request"), "{unsupported}");
    assert!(unsupported.contains("RESOURCE_MANAGER_OPERATION_UNSUPPORTED"));

    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn resource_manager_lists_inspects_and_mutates_native_tools_with_version_guards() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t12-tools-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");

    let list = get(
        port,
        "/management/resource/v1/list?family=tool",
        &management_token,
    );
    assert!(list.contains("200 OK"), "{list}");
    let list_body = response_json(&list);
    assert_eq!(list_body["kind"], "resource.manager.list");
    assert_eq!(list_body["authority_source"], "daemon-native-tool-registry");
    assert!(
        list_body["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == "native.workspace.read")
    );

    let inspect = get(
        port,
        "/management/resource/v1/inspect?family=tool&id=native.workspace.read",
        &management_token,
    );
    assert!(inspect.contains("200 OK"), "{inspect}");
    assert_eq!(response_json(&inspect)["resource"]["object_version"], 1);

    let stale = send_json(
        port,
        "POST",
        "/management/resource/v1/disable",
        &management_token,
        &json!({
            "family":"tool",
            "id":"native.workspace.read",
            "expected_version":99,
            "idempotency_key":"stale-disable"
        }),
    );
    assert!(stale.contains("409 Conflict"), "{stale}");
    assert!(stale.contains("RESOURCE_MANAGER_VERSION_STALE"));

    let disable = send_json(
        port,
        "POST",
        "/management/resource/v1/disable",
        &management_token,
        &json!({
            "family":"tool",
            "id":"native.workspace.read",
            "expected_version":1,
            "idempotency_key":"disable-1"
        }),
    );
    assert!(disable.contains("200 OK"), "{disable}");
    assert_eq!(response_json(&disable)["object_version"], 2);

    let inspect_disabled = get(
        port,
        "/management/resource/v1/inspect?family=tool&id=native.workspace.read",
        &management_token,
    );
    assert_eq!(
        response_json(&inspect_disabled)["resource"]["health"],
        "disabled"
    );

    let enable = send_json(
        port,
        "POST",
        "/management/resource/v1/enable",
        &management_token,
        &json!({
            "family":"tool",
            "id":"native.workspace.read",
            "expected_version":2,
            "idempotency_key":"enable-1"
        }),
    );
    assert!(enable.contains("200 OK"), "{enable}");
    assert_eq!(response_json(&enable)["object_version"], 1);

    let context = get(
        port,
        "/management/resource/v1/list?family=context",
        &management_token,
    );
    assert!(context.contains("200 OK"), "{context}");
    assert_eq!(
        response_json(&context)["authority_source"],
        "projection-only"
    );
    assert_eq!(
        response_json(&context)["resources"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn resource_manager_lists_memory_and_binds_then_revokes_skill_through_existing_sinks() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p8t12-memory-skill-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret_from(&mut daemon, &runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let remember = send_json(
        port,
        "POST",
        "/management/resource/v1/memory/remember",
        &management_token,
        &json!({
            "text": "p8-t12 remembered procedure",
            "governance_scope": "workspace://personal/p8-t12",
            "target_scope": "workspace://personal/p8-t12",
            "purpose": "task_execution",
            "retention_expires_at_unix_seconds": now + 3_600
        }),
    );
    assert!(remember.contains("HTTP/1.1 201 "), "{remember}");
    let memory_id = response_json(&remember)["memory_id"]
        .as_str()
        .expect("remember returns memory_id")
        .to_owned();

    let memory_list = get(
        port,
        "/management/resource/v1/list?family=memory",
        &management_token,
    );
    assert!(memory_list.contains("200 OK"), "{memory_list}");
    let memory_body = response_json(&memory_list);
    assert_eq!(
        memory_body["authority_source"],
        "sqlite-authority-memory-objects"
    );
    assert!(
        memory_body["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == memory_id)
    );

    let memory_inspect = get(
        port,
        &format!("/management/resource/v1/inspect?family=memory&id={memory_id}"),
        &management_token,
    );
    assert!(memory_inspect.contains("200 OK"), "{memory_inspect}");
    assert_eq!(response_json(&memory_inspect)["resource"]["id"], memory_id);

    let package_id = object_id(200);
    let revision_id = object_id(201);
    let binding_id = object_id(202);
    let import = send_json(
        port,
        "POST",
        "/management/resource/v1/skill/import",
        &management_token,
        &json!({
            "package_id": package_id.to_string(),
            "revision_id": revision_id.to_string(),
            "workspace_scope": "workspace://personal/p8-t12",
            "local_source_path": "skills/p8-t12/SKILL.md",
            "provenance_ref": "file://workspace/skills/p8-t12/SKILL.md",
            "manifest_digest": digest('d'),
            "content_digest": digest('e'),
            "compatibility": "compatible",
            "instructions": "use only the reviewed p8-t12 skill",
        }),
    );
    assert!(import.contains("HTTP/1.1 201 "), "{import}");

    let bind = send_json(
        port,
        "POST",
        "/management/resource/v1/bind",
        &management_token,
        &json!({
            "family": "skill",
            "id": binding_id.to_string(),
            "expected_version": 0,
            "idempotency_key": "skill-bind-1",
            "revision_id": revision_id.to_string(),
            "workspace_scope": "workspace://personal/p8-t12",
            "target_kind": "task",
            "target_ref": "task://personal/p8-t12",
        }),
    );
    assert!(bind.contains("200 OK"), "{bind}");
    assert_eq!(response_json(&bind)["object_version"], 1);

    let skill_list = get(
        port,
        "/management/resource/v1/list?family=skill",
        &management_token,
    );
    assert!(skill_list.contains("200 OK"), "{skill_list}");
    assert!(
        response_json(&skill_list)["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == binding_id.to_string() && item["health"] == "bound")
    );

    let revoke = send_json(
        port,
        "POST",
        "/management/resource/v1/revoke",
        &management_token,
        &json!({
            "family": "skill",
            "id": binding_id.to_string(),
            "expected_version": 1,
            "idempotency_key": "skill-revoke-1",
            "reason": "owner revoked the p8-t12 binding",
        }),
    );
    assert!(revoke.contains("200 OK"), "{revoke}");
    assert_eq!(response_json(&revoke)["object_version"], 2);

    let inspect_revoked = get(
        port,
        &format!(
            "/management/resource/v1/inspect?family=skill&id={}",
            binding_id
        ),
        &management_token,
    );
    assert_eq!(
        response_json(&inspect_revoked)["resource"]["health"],
        "revoked"
    );

    let _ = std::fs::remove_dir_all(runtime_root);
}
