#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P4-T02 metadata-first Memory FTS retrieval regressions.

use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::ObjectId;
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::intent_chain::seal_governed_object_content_digest;
use cognitive_kernel::ports::{
    ContextStore, MemoryAdmissionDecisionRow, MemoryCandidateRow, MemoryObjectRow,
    MemorySearchQuery, MemoryStore, WorkspaceContextSourceRow,
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

fn source_row(identifier: &ObjectId, scope: &str, text: &str) -> WorkspaceContextSourceRow {
    let payload = json!({
        "header": {
            "id": identifier.as_str(), "type": "WorkspaceContextSource", "schema_version": "cognitiveos.context/0.1", "object_version": 1,
            "scope_domain": "tenant", "tenant_id": "00000000-0000-7000-9000-0000000000f1", "resource_scope_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000101","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "owner_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000102","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "authority_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000103","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "policy_refs": [], "purpose_constraints": ["task_execution"], "sensitivity": "internal", "compartments": [], "retention": {"policy":"standard","expires_at":null,"legal_hold":false}, "provenance": {"created_by":"principal://tenant-a/daemon","source_refs":[]}, "lineage": {"parents":[],"transform":"memory-search-test"}, "content_digest":format!("sha256:{}", "0".repeat(64)), "created_at":"2026-08-09T00:00:00Z", "valid_time":{"from":"2026-08-09T00:00:00Z","until":null}
        },
        "tenant_id":"tenant-a", "owner_ref":"principal://tenant-a/daemon", "resource_scope":scope, "conversation_ref":null, "role":"working", "trust_level":"verified", "representation":"text", "provenance_ref":"source://context/1", "content_bytes":text.len(), "content_tokens":3, "body":{"text":text}
    });
    let (sealed, source_digest) = seal_governed_object_content_digest(payload).unwrap();
    WorkspaceContextSourceRow {
        source_id: identifier.clone(),
        source_digest,
        governance: ObjectGovernance {
            object_ref: identifier.as_str().to_owned(),
            tenant_id: Some("tenant-a".to_owned()),
            owner_ref: "principal://tenant-a/daemon".to_owned(),
            resource_scope: scope.to_owned(),
            conversation_ref: None,
        },
        role: LoadedContextItemRole::Working,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        provenance_ref: "source://context/1".to_owned(),
        content_bytes: i64::try_from(text.len()).unwrap(),
        content_tokens: Some(3),
        canonical_json: serde_json::to_string(&sealed).unwrap(),
    }
}

fn admit_memory(
    store: &SqliteAuthorityStore,
    identifier_offset: u64,
    source: &WorkspaceContextSourceRow,
    purpose: &str,
    expires_at: i64,
) -> ObjectId {
    store.append_workspace_context_source(source).unwrap();
    let candidate_id = object_id(identifier_offset + 1);
    let candidate_payload = json!({"header":{"id":candidate_id.as_str(),"type":"MemoryCandidate","schema_version":"cognitiveos.memory/0.1","object_version":1,"scope_domain":"tenant","tenant_id":"00000000-0000-7000-9000-0000000000f1","resource_scope_ref":{"kind":"strong","id":"00000000-0000-7000-9000-000000000101","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))},"owner_ref":{"kind":"strong","id":"00000000-0000-7000-9000-000000000102","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))},"authority_ref":{"kind":"strong","id":"00000000-0000-7000-9000-000000000103","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))},"policy_refs":[],"purpose_constraints":["task_execution"],"sensitivity":"internal","compartments":[],"retention":{"policy":"standard","expires_at":null,"legal_hold":false},"provenance":{"created_by":"principal://tenant-a/daemon","source_refs":[]},"lineage":{"parents":[],"transform":"memory-search-test"},"content_digest":format!("sha256:{}", "0".repeat(64)),"created_at":"2026-08-09T00:00:00Z","valid_time":{"from":"2026-08-09T00:00:00Z","until":null}},"memory_kind":"working","governance_scope":source.governance.resource_scope,"purpose":purpose,"retention":{"policy":"standard","expires_at":null,"legal_hold":false},"source_evidence_refs":[],"conflict_refs":[],"admission_status":"proposed","content_ref":"artifact://memory-content","target_scope":source.governance.resource_scope});
    let (sealed_candidate, candidate_digest) =
        seal_governed_object_content_digest(candidate_payload).unwrap();
    let candidate = MemoryCandidateRow {
        candidate_id: candidate_id.clone(),
        candidate_digest: candidate_digest.clone(),
        source_id: source.source_id.clone(),
        source_digest: source.source_digest.clone(),
        source_provenance_ref: source.provenance_ref.clone(),
        governance_scope: source.governance.resource_scope.clone(),
        target_scope: source.governance.resource_scope.clone(),
        purpose: purpose.to_owned(),
        retention_expires_at_unix_seconds: expires_at,
        observed_at_unix_seconds: 100,
        canonical_json: serde_json::to_string(&sealed_candidate).unwrap(),
    };
    let decision_id = object_id(identifier_offset + 2);
    let decision = MemoryAdmissionDecisionRow {
        decision_id: decision_id.clone(),
        candidate_id: candidate_id.clone(),
        candidate_digest,
        decision: "admit".to_owned(),
        policy_version: 1,
        reason_codes_json: "[\"MEMORY_ADMISSION_ACCEPTED\"]".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    let memory_id = object_id(identifier_offset + 3);
    store
        .append_memory_admission(
            &candidate,
            &decision,
            Some(&MemoryObjectRow {
                memory_id: memory_id.clone(),
                candidate_id,
                decision_id,
                canonical_json: "{}".to_owned(),
            }),
        )
        .unwrap();
    memory_id
}

fn search(scope: &str, purpose: &str, observed_at: i64, text: &str) -> MemorySearchQuery {
    MemorySearchQuery {
        governance_scope: scope.to_owned(),
        purpose: purpose.to_owned(),
        observed_at_unix_seconds: observed_at,
        query_text: text.to_owned(),
        maximum_results: 10,
    }
}

#[test]
fn search_filters_authoritative_scope_purpose_and_retention_before_fts_ranking() {
    let (_directory, store) = fresh_store();
    let matching_source = source_row(
        &object_id(1),
        "workspace://tenant-a/project",
        "durable garden planning note",
    );
    let other_scope_source = source_row(
        &object_id(10),
        "workspace://tenant-a/private",
        "durable garden secret",
    );
    let expired_source = source_row(
        &object_id(20),
        "workspace://tenant-a/project",
        "durable garden expired",
    );
    let matching_memory = admit_memory(&store, 100, &matching_source, "task fact", 200);
    admit_memory(&store, 200, &other_scope_source, "task fact", 200);
    admit_memory(&store, 300, &expired_source, "task fact", 100);

    let results = store
        .search_memory_candidates(&search(
            "workspace://tenant-a/project",
            "task fact",
            150,
            "garden",
        ))
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory_id, matching_memory);
    assert_eq!(results[0].source_id, matching_source.source_id);
    assert_eq!(results[0].source_digest, matching_source.source_digest);
}

#[test]
fn rebuild_restores_derived_fts_rows_without_changing_authoritative_memory() {
    let (_directory, store) = fresh_store();
    let source = source_row(
        &object_id(30),
        "workspace://tenant-a/project",
        "rebuildable lantern memory",
    );
    let memory_id = admit_memory(&store, 400, &source, "task fact", 200);

    store.rebuild_memory_search_index().unwrap();

    assert_eq!(
        store
            .search_memory_candidates(&search(
                "workspace://tenant-a/project",
                "task fact",
                150,
                "lantern"
            ))
            .unwrap()[0]
            .memory_id,
        memory_id
    );
    assert!(store.load_memory_object(&memory_id).unwrap().is_some());
}
