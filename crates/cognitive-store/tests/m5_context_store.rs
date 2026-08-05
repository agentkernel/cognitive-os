//! Durable Context request/view admission regressions.
//!
//! Context records are daemon-owned governed objects. These tests prove that
//! the SQLite persistence boundary accepts only sealed payloads whose durable
//! row metadata and strong request reference name the exact same objects.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_domain::ObjectId;
use cognitive_kernel::intent_chain::seal_governed_object_content_digest;
use cognitive_kernel::ports::{ContextRequestRow, ContextStore, ContextViewRow, StorePortError};
use cognitive_store::SqliteAuthorityStore;
use serde_json::{Value, json};

fn object_id(sequence: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
}

fn strong_reference(identifier: &str, digest: &str) -> Value {
    json!({
        "kind": "strong",
        "id": identifier,
        "object_version": 1,
        "content_digest": digest,
    })
}

fn governed_header(identifier: &ObjectId, object_type: &str) -> Value {
    let reference_digest = format!("sha256:{}", "a".repeat(64));
    json!({
        "id": identifier.as_str(),
        "type": object_type,
        "schema_version": "cognitiveos.context/0.1",
        "object_version": 1,
        "scope_domain": "tenant",
        "tenant_id": "00000000-0000-7000-9000-0000000000f1",
        "resource_scope_ref": strong_reference("00000000-0000-7000-9000-000000000101", &reference_digest),
        "owner_ref": strong_reference("00000000-0000-7000-9000-000000000102", &reference_digest),
        "authority_ref": strong_reference("00000000-0000-7000-9000-000000000103", &reference_digest),
        "policy_refs": [],
        "purpose_constraints": ["task_execution"],
        "sensitivity": "internal",
        "compartments": [],
        "retention": {"policy": "standard", "expires_at": null, "legal_hold": false},
        "provenance": {"created_by": "principal://tenant-a/daemon", "source_refs": []},
        "lineage": {"parents": [], "transform": "context-store-test"},
        "content_digest": format!("sha256:{}", "0".repeat(64)),
        "created_at": "2026-08-05T00:00:00Z",
        "valid_time": {"from": "2026-08-05T00:00:00Z", "until": null},
    })
}

fn sealed_payload(payload: Value) -> (String, String) {
    let (sealed, digest) = seal_governed_object_content_digest(payload).unwrap();
    (serde_json::to_string(&sealed).unwrap(), digest)
}

fn context_request_row(identifier: &ObjectId, task_ref: &str) -> ContextRequestRow {
    let (canonical_json, request_digest) = sealed_payload(json!({
        "header": governed_header(identifier, "ContextRequest"),
        "purpose": "task execution",
        "perspective": {
            "principal": "principal://tenant-a/daemon",
            "task": task_ref,
            "episode": "episode://tenant-a/context-test",
        },
        "budget": {},
        "priority": ["task"],
        "required": [],
        "forbidden": [],
        "freshness": {"world_max_age_ms": 0},
        "sensitivity": {"max_input": "internal", "egress": "none"},
        "target_profile": {"kind": "structured", "schema": "context-test/v1"},
        "allow_partial": false,
    }));
    ContextRequestRow {
        request_id: identifier.clone(),
        task_ref: task_ref.to_owned(),
        request_digest,
        canonical_json,
    }
}

fn context_view_row(identifier: &ObjectId, request: &ContextRequestRow) -> ContextViewRow {
    let (canonical_json, view_digest) = sealed_payload(json!({
        "header": governed_header(identifier, "ContextView"),
        "request_ref": strong_reference(request.request_id.as_str(), &request.request_digest),
        "complete": true,
        "loaded": [],
        "rejected": [],
        "loss_declaration": [],
        "pinned_versions": {"task": 1},
        "cost": {"bytes": 0, "resolve_ms": 0},
        "activity_bound": "activity://tenant-a/context-test",
    }));
    ContextViewRow {
        view_id: identifier.clone(),
        request_id: request.request_id.clone(),
        view_digest,
        canonical_json,
    }
}

fn fresh_store() -> (tempfile::TempDir, SqliteAuthorityStore) {
    let directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&directory.path().join("authority.db")).unwrap();
    (directory, store)
}

#[test]
fn context_store_accepts_matching_sealed_request_and_view() {
    let (_directory, store) = fresh_store();
    let request = context_request_row(&object_id(1), "task://tenant-a/context-test");
    let view = context_view_row(&object_id(2), &request);

    store.append_context_request(&request).unwrap();
    store.append_context_view(&view).unwrap();

    assert_eq!(
        store.load_context_request(&request.request_id).unwrap(),
        Some(request)
    );
    assert_eq!(store.load_context_view(&view.view_id).unwrap(), Some(view));
}

#[test]
fn context_store_rejects_tampered_or_mislabeled_request_before_insert() {
    let (_directory, store) = fresh_store();
    let mut tampered_request = context_request_row(&object_id(10), "task://tenant-a/context-test");
    let mut payload: Value = serde_json::from_str(&tampered_request.canonical_json).unwrap();
    payload["purpose"] = json!("tampered after sealing");
    tampered_request.canonical_json = serde_json::to_string(&payload).unwrap();

    assert!(matches!(
        store.append_context_request(&tampered_request),
        Err(StorePortError::Unavailable { .. })
    ));
    assert!(
        store
            .load_context_request(&tampered_request.request_id)
            .unwrap()
            .is_none()
    );

    let mut mislabeled_request =
        context_request_row(&object_id(11), "task://tenant-a/context-test");
    mislabeled_request.task_ref = "task://tenant-a/other-task".to_owned();
    assert!(matches!(
        store.append_context_request(&mislabeled_request),
        Err(StorePortError::Unavailable { .. })
    ));
    assert!(
        store
            .load_context_request(&mislabeled_request.request_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn context_store_requires_view_request_strong_reference_to_match_persisted_request() {
    let (_directory, store) = fresh_store();
    let request = context_request_row(&object_id(20), "task://tenant-a/context-test");
    store.append_context_request(&request).unwrap();

    let mut mismatched_view = context_view_row(&object_id(21), &request);
    let mut payload: Value = serde_json::from_str(&mismatched_view.canonical_json).unwrap();
    payload["request_ref"]["content_digest"] = json!(format!("sha256:{}", "b".repeat(64)));
    let (canonical_json, view_digest) = sealed_payload(payload);
    mismatched_view.canonical_json = canonical_json;
    mismatched_view.view_digest = view_digest;

    assert!(matches!(
        store.append_context_view(&mismatched_view),
        Err(StorePortError::Unavailable { .. })
    ));
    assert!(
        store
            .load_context_view(&mismatched_view.view_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn context_store_rejects_view_for_unknown_request_identity() {
    let (_directory, store) = fresh_store();
    let unknown_request = context_request_row(&object_id(30), "task://tenant-a/context-test");
    let view = context_view_row(&object_id(31), &unknown_request);

    assert!(matches!(
        store.append_context_view(&view),
        Err(StorePortError::Conflict { .. })
    ));
    assert!(store.load_context_view(&view.view_id).unwrap().is_none());
}
