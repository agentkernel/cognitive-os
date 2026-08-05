//! Durable Context request/view admission regressions.
//!
//! Context records are daemon-owned governed objects. These tests prove that
//! the SQLite persistence boundary accepts only sealed payloads whose durable
//! row metadata and strong request reference name the exact same objects.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{ObjectId, UriRef, WallTimestamp};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, MembershipFacts, ObjectGovernance, PrincipalFacts, authorize,
};
use cognitive_kernel::intent_chain::seal_governed_object_content_digest;
use cognitive_kernel::ports::{
    ContextAuthorizationFactStore, ContextAuthorizationFactsRow, ContextCandidateQuery,
    ContextRequestRow, ContextRevocationFactRow, ContextStore, ContextViewRow, StorePortError,
    WorkspaceContextSourceRow,
};
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore, prepare_personal_databases};
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
    let root = directory.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    );
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    (directory, store)
}

fn workspace_source_row(
    identifier: &ObjectId,
    conversation_ref: Option<&str>,
) -> WorkspaceContextSourceRow {
    let canonical_payload = json!({
        "header": governed_header(identifier, "WorkspaceContextSource"),
        "tenant_id": "tenant-a",
        "owner_ref": "principal://tenant-a/daemon",
        "resource_scope": "workspace://tenant-a/project/alpha",
        "conversation_ref": conversation_ref,
        "role": "working",
        "trust_level": "verified",
        "representation": "text",
        "provenance_ref": "admission://tenant-a/daemon/test",
        "content_bytes": 12,
        "content_tokens": 3,
        "body": {"text": "trusted text"},
    });
    let (canonical_json, source_digest) = sealed_payload(canonical_payload);
    WorkspaceContextSourceRow {
        source_id: identifier.clone(),
        source_digest,
        governance: ObjectGovernance {
            object_ref: identifier.as_str().to_owned(),
            tenant_id: Some("tenant-a".to_owned()),
            owner_ref: "principal://tenant-a/daemon".to_owned(),
            resource_scope: "workspace://tenant-a/project/alpha".to_owned(),
            conversation_ref: conversation_ref.map(str::to_owned),
        },
        role: LoadedContextItemRole::Working,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        provenance_ref: "admission://tenant-a/daemon/test".to_owned(),
        content_bytes: 12,
        content_tokens: Some(3),
        canonical_json,
    }
}

fn timestamp(value: &str) -> WallTimestamp {
    WallTimestamp::parse(value).unwrap()
}

fn context_authorization_facts_row(identifier: &ObjectId) -> ContextAuthorizationFactsRow {
    let principal = PrincipalFacts {
        principal_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
        authenticated: true,
        active: true,
        tenant_id: Some("tenant-a".to_owned()),
    };
    let actor_chain = ActorChainFacts {
        chain_digest: format!("sha256:{}", "d".repeat(64)),
        resolved: true,
    };
    let membership = Some(MembershipFacts {
        valid: true,
        roles: ["owner".to_owned()].into(),
    });
    let capability_links = vec![CapabilityConstraints {
        subject: principal.principal_ref.as_str().to_owned(),
        audience: "daemon://tenant-a/context".to_owned(),
        resource: "workspace://tenant-a/project".to_owned(),
        purpose: "task_execution".to_owned(),
        actions: ["read_body".to_owned()].into(),
        parameter_bounds: Default::default(),
        lease: LeaseWindow {
            not_before: timestamp("2026-08-05T00:00:00Z"),
            expires: timestamp("2026-08-06T00:00:00Z"),
        },
        depth_remaining: 1,
        issued_epoch: 1,
    }];
    let payload = json!({
        "header": governed_header(identifier, "ContextAuthorizationFacts"),
        "fact_set_id": identifier.as_str(),
        "subject_ref": principal.principal_ref.as_str(),
        "tenant_id": "tenant-a",
        "principal": principal,
        "actor_chain": actor_chain,
        "membership": membership,
        "capability_links": capability_links,
        "explicit_denies": [],
        "capability_set_version": 1,
        "issued_revocation_epoch": 1,
    });
    let (canonical_json, _) = sealed_payload(payload);
    ContextAuthorizationFactsRow {
        fact_set_id: identifier.clone(),
        subject_ref: "principal://tenant-a/daemon".to_owned(),
        tenant_id: "tenant-a".to_owned(),
        principal,
        actor_chain,
        membership,
        capability_links,
        explicit_denies: vec![],
        capability_set_version: 1,
        issued_revocation_epoch: 1,
        canonical_json,
    }
}

fn context_revocation_fact_row(identifier: &ObjectId, epoch: i64) -> ContextRevocationFactRow {
    let payload = json!({
        "header": governed_header(identifier, "ContextRevocationFact"),
        "revocation_fact_id": identifier.as_str(),
        "tenant_id": "tenant-a",
        "revocation_epoch": epoch,
        "revoked_subject_ref": null,
        "revoked_capability_ref": null,
    });
    let (canonical_json, _) = sealed_payload(payload);
    ContextRevocationFactRow {
        revocation_fact_id: identifier.clone(),
        tenant_id: "tenant-a".to_owned(),
        revocation_epoch: epoch,
        revoked_subject_ref: None,
        revoked_capability_ref: None,
        canonical_json,
    }
}

#[test]
fn context_authorization_facts_reconstruct_only_against_durable_current_epoch() {
    let (_directory, store) = fresh_store();
    let facts = context_authorization_facts_row(&object_id(40));
    let initial_epoch = context_revocation_fact_row(&object_id(41), 1);
    store
        .append_context_revocation_fact(&initial_epoch)
        .unwrap();
    store.append_context_authorization_facts(&facts).unwrap();

    let loaded_facts = store
        .load_latest_context_authorization_facts(&facts.subject_ref, &facts.tenant_id)
        .unwrap()
        .unwrap();
    assert_eq!(loaded_facts, facts);
    assert_eq!(
        store
            .load_current_context_revocation_epoch("tenant-a")
            .unwrap(),
        Some(1)
    );
    assert!(
        loaded_facts
            .reconstruct_snapshot(1, timestamp("2026-08-05T12:00:00Z"))
            .is_ok()
    );

    store
        .append_context_revocation_fact(&context_revocation_fact_row(&object_id(42), 2))
        .unwrap();
    assert_eq!(
        store
            .load_current_context_revocation_epoch("tenant-a")
            .unwrap(),
        Some(2)
    );
}

#[test]
fn later_durable_revocation_epoch_denies_previously_allowed_context_body_access() {
    let (_directory, store) = fresh_store();
    let facts = context_authorization_facts_row(&object_id(43));
    let source = workspace_source_row(&object_id(44), Some("conversation://tenant-a/one"));
    store
        .append_context_revocation_fact(&context_revocation_fact_row(&object_id(45), 1))
        .unwrap();
    store.append_context_authorization_facts(&facts).unwrap();
    store.append_workspace_context_source(&source).unwrap();

    let access_request = AccessRequest {
        action: "read_body".to_owned(),
        purpose: "task_execution".to_owned(),
    };
    let initial_snapshot = facts
        .reconstruct_snapshot(1, timestamp("2026-08-05T12:00:00Z"))
        .unwrap();
    assert!(authorize(&initial_snapshot, &source.governance, &access_request).is_ok());

    store
        .append_context_revocation_fact(&context_revocation_fact_row(&object_id(46), 2))
        .unwrap();
    let current_epoch = store
        .load_current_context_revocation_epoch("tenant-a")
        .unwrap()
        .unwrap();
    let revoked_snapshot = facts
        .reconstruct_snapshot(current_epoch, timestamp("2026-08-05T12:00:00Z"))
        .unwrap();
    assert!(authorize(&revoked_snapshot, &source.governance, &access_request).is_err());
}

#[test]
fn context_authorization_fact_append_rejects_tampering_and_epoch_conflicts() {
    let (_directory, store) = fresh_store();
    let mut tampered_facts = context_authorization_facts_row(&object_id(50));
    let mut payload: Value = serde_json::from_str(&tampered_facts.canonical_json).unwrap();
    payload["subject_ref"] = json!("principal://tenant-a/other");
    tampered_facts.canonical_json = serde_json::to_string(&payload).unwrap();
    assert!(matches!(
        store.append_context_authorization_facts(&tampered_facts),
        Err(StorePortError::Unavailable { .. })
    ));

    let first_epoch = context_revocation_fact_row(&object_id(51), 1);
    store.append_context_revocation_fact(&first_epoch).unwrap();
    let conflicting_epoch = context_revocation_fact_row(&object_id(52), 1);
    assert!(matches!(
        store.append_context_revocation_fact(&conflicting_epoch),
        Err(StorePortError::Conflict { .. })
    ));
}

#[test]
fn workspace_context_source_discovery_returns_metadata_before_body_load() {
    let (_directory, store) = fresh_store();
    let matching_source = workspace_source_row(&object_id(30), Some("conversation://tenant-a/one"));
    let other_conversation =
        workspace_source_row(&object_id(31), Some("conversation://tenant-a/two"));
    store
        .append_workspace_context_source(&matching_source)
        .unwrap();
    store
        .append_workspace_context_source(&other_conversation)
        .unwrap();

    let metadata = store
        .query_context_candidate_metadata(&ContextCandidateQuery {
            tenant_id: "tenant-a".to_owned(),
            resource_scope_prefix: "workspace://tenant-a/project".to_owned(),
            conversation_ref: Some("conversation://tenant-a/one".to_owned()),
            limit: 10,
        })
        .unwrap();

    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].source_id, matching_source.source_id);
    assert!(!format!("{:?}", metadata[0]).contains("trusted text"));
    assert_eq!(
        store
            .load_workspace_context_source_body(&matching_source.source_id)
            .unwrap(),
        Some(matching_source)
    );
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
