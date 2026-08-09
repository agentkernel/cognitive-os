#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P4-T01 durable Memory admission regressions.

use cognitive_domain::ObjectId;
use cognitive_kernel::intent_chain::seal_governed_object_content_digest;
use cognitive_kernel::ports::{
    MemoryAdmissionDecisionRow, MemoryCandidateRow, MemoryObjectRow, MemoryStore, StorePortError,
};
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore, prepare_personal_databases};
use serde_json::json;

fn object_id(sequence: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
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

fn memory_candidate(identifier: &ObjectId, source_identifier: &ObjectId) -> MemoryCandidateRow {
    let payload = json!({
        "header": {
            "id": identifier.as_str(), "type": "MemoryCandidate", "schema_version": "cognitiveos.memory/0.1", "object_version": 1,
            "scope_domain": "tenant", "tenant_id": "00000000-0000-7000-9000-0000000000f1", "resource_scope_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000101","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "owner_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000102","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "authority_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000103","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "policy_refs": [], "purpose_constraints": ["task_execution"], "sensitivity": "internal", "compartments": [], "retention": {"policy":"standard","expires_at":null,"legal_hold":false}, "provenance": {"created_by":"principal://tenant-a/daemon","source_refs":[]}, "lineage": {"parents":[],"transform":"memory-store-test"}, "content_digest":format!("sha256:{}", "0".repeat(64)), "created_at":"2026-08-09T00:00:00Z", "valid_time":{"from":"2026-08-09T00:00:00Z","until":null}
        },
        "memory_kind":"working", "governance_scope":"workspace://tenant-a/project", "purpose":"task fact", "retention":{"policy":"standard","expires_at":null,"legal_hold":false}, "source_evidence_refs":[], "conflict_refs":[], "admission_status":"proposed", "content_ref":"artifact://memory-content", "target_scope":"workspace://tenant-a/project"
    });
    let (sealed, candidate_digest) = seal_governed_object_content_digest(payload).unwrap();
    MemoryCandidateRow {
        candidate_id: identifier.clone(),
        candidate_digest,
        source_id: source_identifier.clone(),
        source_digest: format!("sha256:{}", "b".repeat(64)),
        source_provenance_ref: "source://context/1".to_owned(),
        governance_scope: "workspace://tenant-a/project".to_owned(),
        target_scope: "workspace://tenant-a/project".to_owned(),
        purpose: "task fact".to_owned(),
        retention_expires_at_unix_seconds: 200,
        observed_at_unix_seconds: 100,
        canonical_json: serde_json::to_string(&sealed).unwrap(),
    }
}

#[test]
fn stale_or_unknown_source_binding_rejects_without_memory_object() {
    let (_directory, store) = fresh_store();
    let candidate = memory_candidate(&object_id(1), &object_id(2));
    let decision = MemoryAdmissionDecisionRow {
        decision_id: object_id(3),
        candidate_id: candidate.candidate_id.clone(),
        candidate_digest: candidate.candidate_digest.clone(),
        decision: "admit".to_owned(),
        policy_version: 1,
        reason_codes_json: "[\"MEMORY_ADMISSION_ACCEPTED\"]".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    let memory_object = MemoryObjectRow {
        memory_id: object_id(4),
        candidate_id: candidate.candidate_id.clone(),
        decision_id: decision.decision_id.clone(),
        canonical_json: "{}".to_owned(),
    };

    assert!(matches!(
        store.append_memory_admission(&candidate, &decision, Some(&memory_object)),
        Err(StorePortError::Conflict { .. })
    ));
    assert_eq!(
        store.load_memory_object(&memory_object.memory_id).unwrap(),
        None
    );
}

#[test]
fn reject_decision_cannot_create_memory_object() {
    let (_directory, store) = fresh_store();
    let candidate = memory_candidate(&object_id(11), &object_id(12));
    let decision = MemoryAdmissionDecisionRow {
        decision_id: object_id(13),
        candidate_id: candidate.candidate_id.clone(),
        candidate_digest: candidate.candidate_digest.clone(),
        decision: "reject".to_owned(),
        policy_version: 1,
        reason_codes_json: "[\"MEMORY_ADMISSION_DENIED\"]".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    let memory_object = MemoryObjectRow {
        memory_id: object_id(14),
        candidate_id: candidate.candidate_id.clone(),
        decision_id: decision.decision_id.clone(),
        canonical_json: "{}".to_owned(),
    };

    assert!(matches!(
        store.append_memory_admission(&candidate, &decision, Some(&memory_object)),
        Err(StorePortError::Unavailable { .. })
    ));
    assert_eq!(
        store.load_memory_object(&memory_object.memory_id).unwrap(),
        None
    );
}
