//! P4-T05/D01 failure-first coverage for task-bound resource projection.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::ops::{Deref, DerefMut};
use std::process::{Child, Command};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_domain::{ObjectId, WallTimestamp};
use cognitive_kernel::intent_chain::{
    GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
    strong_reference_to,
};
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

fn stop_for_restart(process: &mut PersonalProcess, runtime_root: &std::path::Path) {
    process.kill().unwrap();
    process.wait().unwrap();
    let _ = std::fs::remove_file(runtime_root.join("cognitiveos").join("daemon.lock"));
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

fn sealed_object(
    identifier: &ObjectId,
    object_type: &str,
    schema_version: &str,
    mut payload: Value,
) -> (Value, String) {
    let header = compose_governed_header(
        identifier,
        object_type,
        schema_version,
        &governance_seed(),
        Vec::new(),
        Vec::new(),
        "management-resource-lifecycle-test",
        &WallTimestamp::parse("2026-08-14T00:00:00Z").unwrap(),
    )
    .unwrap();
    payload["header"] = serde_json::to_value(header).unwrap();
    seal_governed_object_content_digest(payload).unwrap()
}

fn governance_seed() -> GovernanceSeed {
    GovernanceSeed {
        owner: strong_reference_to(&object_id(900), &digest('a')),
        authority: strong_reference_to(&object_id(901), &digest('b')),
        resource_scope: strong_reference_to(&object_id(902), &digest('c')),
        tenant_id: Some(object_id(903).to_string()),
        created_by: "principal://local/owner".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "owner_local".to_owned(),
    }
}

fn object_id(serial: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{serial:012x}")).unwrap()
}

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
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

#[test]
fn management_resource_lifecycle_preconditions_are_discoverable() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p2t19-resource-preconditions-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret(&runtime_root);
    let management_token = issue_token(port, &secret, "management");

    let response = get(
        port,
        "/resource/v1/projection?family=memory&version=1",
        &management_token,
    );

    assert!(response.contains("200 OK"), "{response}");
    let document = response_json(&response);
    assert_eq!(
        document["projection"]["lifecycle"]["remember"],
        "/management/resource/v1/memory/remember"
    );
    assert_eq!(
        document["projection"]["lifecycle"]["forget"],
        "/management/resource/v1/memory/forget"
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn management_memory_lifecycle_uses_canonical_source_and_survives_restart() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p2t19-memory-lifecycle-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret(&runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let scope = "workspace://personal/project/lifecycle";
    let provenance = "file://workspace/facts/lifecycle.txt";
    let source_id = object_id(100);
    let (source, source_digest) = sealed_object(
        &source_id,
        "WorkspaceContextSource",
        "cognitiveos.workspace-context-source/0.1",
        json!({
            "tenant_id": "personal",
            "owner_ref": "principal://local/owner",
            "resource_scope": scope,
            "conversation_ref": null,
            "role": "working",
            "trust_level": "verified",
            "representation": "text",
            "provenance_ref": provenance,
            "content_bytes": 19,
            "content_tokens": 4,
            "body": {"text": "durable owner fact"},
        }),
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let candidate_id = object_id(101);
    let (candidate, _) = sealed_object(
        &candidate_id,
        "MemoryCandidate",
        "cognitiveos.memory/0.1",
        json!({
            "source_id": source_id.to_string(),
            "source_digest": source_digest,
            "source_provenance_ref": provenance,
            "governance_scope": scope,
            "target_scope": scope,
            "purpose": "task_execution",
            "retention_expires_at_unix_seconds": now + 3_600,
            "observed_at_unix_seconds": now,
        }),
    );
    let remember_response = send_json(
        port,
        "POST",
        "/management/resource/v1/memory/remember",
        &management_token,
        &json!({"source": source, "candidate": candidate}),
    );
    assert!(
        remember_response.contains("HTTP/1.1 201 "),
        "{remember_response}"
    );
    let memory_id = response_json(&remember_response)["memory_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let review = get(
        port,
        &format!("/management/resource/v1/memory/object?id={memory_id}"),
        &management_token,
    );
    assert!(review.contains("200 OK"), "{review}");
    assert_eq!(response_json(&review)["memory"]["memory_id"], memory_id);

    let forget = send_json(
        port,
        "POST",
        "/management/resource/v1/memory/forget",
        &management_token,
        &json!({"memory_id": memory_id, "reason": "owner lifecycle test"}),
    );
    assert!(forget.contains("HTTP/1.1 201 "), "{forget}");
    assert_eq!(response_json(&forget)["status"], "forgotten");

    stop_for_restart(&mut daemon, &runtime_root);
    let restarted_port = free_port();
    let mut restarted = spawn_personal(restarted_port, &runtime_root);
    let restarted_secret = common::wait_for_bootstrap_secret(&runtime_root);
    let restarted_token = issue_token(restarted_port, &restarted_secret, "management");
    let after_restart = get(
        restarted_port,
        &format!("/management/resource/v1/memory/object?id={memory_id}"),
        &restarted_token,
    );
    assert!(after_restart.contains("200 OK"), "{after_restart}");

    restarted.kill().unwrap();
    restarted.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn rejected_memory_candidate_leaves_exact_source_retryable_without_partial_memory() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p2t19-memory-retry-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret(&runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let scope = "workspace://personal/project/retry";
    let provenance = "file://workspace/facts/retry.txt";
    let source_id = object_id(300);
    let (source, source_digest) = sealed_object(
        &source_id,
        "WorkspaceContextSource",
        "cognitiveos.workspace-context-source/0.1",
        json!({
            "tenant_id": "personal",
            "owner_ref": "principal://local/owner",
            "resource_scope": scope,
            "conversation_ref": null,
            "role": "working",
            "trust_level": "verified",
            "representation": "text",
            "provenance_ref": provenance,
            "content_bytes": 14,
            "content_tokens": 3,
            "body": {"text": "retryable fact"},
        }),
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let (invalid_candidate, _) = sealed_object(
        &object_id(301),
        "MemoryCandidate",
        "cognitiveos.memory/0.1",
        json!({
            "source_id": source_id.to_string(),
            "source_digest": source_digest,
            "source_provenance_ref": provenance,
            "governance_scope": scope,
            "target_scope": "workspace://personal/promoted",
            "purpose": "task_execution",
            "retention_expires_at_unix_seconds": now + 3_600,
            "observed_at_unix_seconds": now,
        }),
    );
    let rejected = send_json(
        port,
        "POST",
        "/management/resource/v1/memory/remember",
        &management_token,
        &json!({"source": source, "candidate": invalid_candidate}),
    );
    assert!(rejected.contains("409 Conflict"), "{rejected}");

    let (valid_candidate, _) = sealed_object(
        &object_id(302),
        "MemoryCandidate",
        "cognitiveos.memory/0.1",
        json!({
            "source_id": source_id.to_string(),
            "source_digest": source_digest,
            "source_provenance_ref": provenance,
            "governance_scope": scope,
            "target_scope": scope,
            "purpose": "task_execution",
            "retention_expires_at_unix_seconds": now + 3_600,
            "observed_at_unix_seconds": now,
        }),
    );
    let retried = send_json(
        port,
        "POST",
        "/management/resource/v1/memory/remember",
        &management_token,
        &json!({"source": source, "candidate": valid_candidate}),
    );
    assert!(
        retried.contains("HTTP/1.1 201 "),
        "an exact source may be retried after candidate rejection without raw cleanup: {retried}"
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}

#[test]
fn management_skill_lifecycle_imports_inspects_supersedes_and_revokes() {
    let runtime_root = std::env::temp_dir().join(format!(
        "cos-p2t19-skill-lifecycle-{}-{}",
        std::process::id(),
        free_port()
    ));
    std::fs::create_dir_all(&runtime_root).unwrap();
    let port = free_port();
    let mut daemon = spawn_personal(port, &runtime_root);
    let secret = common::wait_for_bootstrap_secret(&runtime_root);
    let management_token = issue_token(port, &secret, "management");
    let package_id = object_id(200);
    let revision_id = object_id(201);
    let binding_id = object_id(202);
    let replacement_id = object_id(203);
    let workspace_scope = "workspace://personal/project/lifecycle";
    let import = send_json(
        port,
        "POST",
        "/management/resource/v1/skill/import",
        &management_token,
        &json!({
            "package_id": package_id.to_string(),
            "revision_id": revision_id.to_string(),
            "workspace_scope": workspace_scope,
            "local_source_path": "skills/lifecycle/SKILL.md",
            "provenance_ref": "file://workspace/skills/lifecycle/SKILL.md",
            "manifest_digest": digest('d'),
            "content_digest": digest('e'),
            "compatibility": "compatible",
            "instructions": "use only the reviewed lifecycle skill",
        }),
    );
    assert!(import.contains("HTTP/1.1 201 "), "{import}");

    let inspect = get(
        port,
        &format!(
            "/management/resource/v1/skill/binding/explain?kind=revision&id={}",
            revision_id
        ),
        &management_token,
    );
    assert!(inspect.contains("200 OK"), "{inspect}");
    assert_eq!(
        response_json(&inspect)["revision"]["content_digest"],
        digest('e')
    );

    let bind = send_json(
        port,
        "POST",
        "/management/resource/v1/skill/bind",
        &management_token,
        &json!({
            "binding_id": binding_id.to_string(),
            "revision_id": revision_id.to_string(),
            "workspace_scope": workspace_scope,
            "target_kind": "task",
            "target_ref": "task://personal/lifecycle",
        }),
    );
    assert!(bind.contains("HTTP/1.1 201 "), "{bind}");
    let explain = get(
        port,
        &format!(
            "/management/resource/v1/skill/binding/explain?id={}",
            binding_id
        ),
        &management_token,
    );
    assert!(explain.contains("200 OK"), "{explain}");

    let supersede = send_json(
        port,
        "POST",
        "/management/resource/v1/skill/import",
        &management_token,
        &json!({
            "previous_revision_id": revision_id.to_string(),
            "revision_id": replacement_id.to_string(),
            "package_id": package_id.to_string(),
            "content_digest": digest('f'),
            "compatibility": "compatible",
            "instructions": "replacement remains an exact opt-in revision",
        }),
    );
    assert!(supersede.contains("HTTP/1.1 201 "), "{supersede}");
    let replacement = get(
        port,
        &format!(
            "/management/resource/v1/skill/binding/explain?kind=revision&id={}",
            replacement_id
        ),
        &management_token,
    );
    assert!(replacement.contains("200 OK"), "{replacement}");
    assert_eq!(
        response_json(&replacement)["revision"]["content_digest"],
        digest('f')
    );

    let revoke = send_json(
        port,
        "POST",
        "/management/resource/v1/skill/binding/revoke",
        &management_token,
        &json!({
            "revocation_id": object_id(204).to_string(),
            "binding_id": binding_id.to_string(),
            "reason": "owner revoked lifecycle binding",
        }),
    );
    assert!(revoke.contains("HTTP/1.1 201 "), "{revoke}");
    let revoked = get(
        port,
        &format!(
            "/management/resource/v1/skill/binding/explain?id={}",
            binding_id
        ),
        &management_token,
    );
    assert!(revoked.contains("200 OK"), "{revoked}");
    assert_eq!(
        response_json(&revoked)["binding"]["revocation_reason"],
        "owner revoked lifecycle binding"
    );

    stop_for_restart(&mut daemon, &runtime_root);
    let restarted_port = free_port();
    let mut restarted = spawn_personal(restarted_port, &runtime_root);
    let restarted_secret = common::wait_for_bootstrap_secret(&runtime_root);
    let restarted_token = issue_token(restarted_port, &restarted_secret, "management");
    let after_restart = get(
        restarted_port,
        &format!(
            "/management/resource/v1/skill/binding/explain?id={}",
            binding_id
        ),
        &restarted_token,
    );
    assert!(after_restart.contains("200 OK"), "{after_restart}");
    assert_eq!(
        response_json(&after_restart)["binding"]["revocation_reason"],
        "owner revoked lifecycle binding"
    );

    restarted.kill().unwrap();
    restarted.wait().unwrap();
    let _ = std::fs::remove_dir_all(runtime_root);
}
