#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P11-T11 D01: scoped episodic recall, secret/PII, Letta/self, forget/rebuild.
//!
//! N5 (vault file cannot enter Memory as authority) remains the T10 test
//! `p11_t10_memory_admission_cannot_swallow_vault_files` — not duplicated here.
//! N4 FTS resurrection against `workspace://` remains `p4_t02_memory_search`;
//! this file covers the scoped `opc://project/{id}/employee/{id}` path.

use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::ObjectId;
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::intent_chain::seal_governed_object_content_digest;
use cognitive_kernel::ports::{
    ContextStore, MemoryAdmissionDecisionRow, MemoryCandidateRow, MemoryObjectRow, MemoryStore,
    MemoryTombstoneRow, WorkspaceContextSourceRow,
};
use cognitive_store::{
    ConfirmCaller, EmployeeStore, EpisodicRecallSpec, PersonalDataLayout, ProjectAggregateError,
    ProjectAggregateStore, RosterProposal, SqliteAuthorityStore, StageSpec,
    canonical_episodic_scope, forget_episodic_memory, prepare_personal_databases,
    rebuild_episodic_memory_index, recall_episodic_memory, screen_memory_admission,
};
use serde_json::json;
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    SqliteAuthorityStore,
    ProjectAggregateStore,
    EmployeeStore,
) {
    let temporary = TempDir::new().expect("temp");
    let root = temporary.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    );
    prepare_personal_databases(&layout).expect("prepare");
    let path = layout.authority_database_path();
    let authority = SqliteAuthorityStore::open(&path).expect("authority");
    let projects = ProjectAggregateStore::open_path(&path).expect("projects");
    let employees = EmployeeStore::open_path(&path).expect("employees");
    (temporary, authority, projects, employees)
}

fn stage(id: &str, title: &str, slot: &str) -> StageSpec {
    StageSpec {
        stage_id: id.to_owned(),
        title: title.to_owned(),
        objective: format!("{title} objective"),
        output_contract_digest: ProjectAggregateStore::digest_hex(format!("out-{id}").as_bytes()),
        acceptance_spec_ref: Some(format!("cas:spec-{id}")),
        cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
        responsible_slot: slot.to_owned(),
        blocking_gap: None,
    }
}

fn activate(projects: &ProjectAggregateStore) -> String {
    let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
    projects
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = projects
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("G1")
        .new_ref
}

fn plan_two_slots(projects: &ProjectAggregateStore, project_id: &str) -> String {
    projects
        .apply_plan_revision(
            project_id,
            project_id,
            &[
                stage("s1", "Manage", "manager"),
                stage("s2", "Research", "researcher"),
            ],
            20,
        )
        .expect("plan")
}

fn proposals() -> [RosterProposal; 2] {
    [
        RosterProposal {
            slot: "manager".to_owned(),
            specialization: "project-manager".to_owned(),
            prompt: "coordinate".to_owned(),
            tools_declared: vec!["workspace-write".to_owned()],
        },
        RosterProposal {
            slot: "researcher".to_owned(),
            specialization: "member".to_owned(),
            prompt: "file notes".to_owned(),
            tools_declared: vec!["workspace-write".to_owned()],
        },
    ]
}

fn object_id(sequence: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
}

fn source_row(identifier: &ObjectId, scope: &str, text: &str) -> WorkspaceContextSourceRow {
    let payload = json!({
        "header": {
            "id": identifier.as_str(), "type": "WorkspaceContextSource", "schema_version": "cognitiveos.context/0.1", "object_version": 1,
            "scope_domain": "tenant", "tenant_id": "00000000-0000-7000-9000-0000000000f1", "resource_scope_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000101","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "owner_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000102","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "authority_ref": {"kind":"strong","id":"00000000-0000-7000-9000-000000000103","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))}, "policy_refs": [], "purpose_constraints": ["task_execution"], "sensitivity": "internal", "compartments": [], "retention": {"policy":"standard","expires_at":null,"legal_hold":false}, "provenance": {"created_by":"principal://tenant-a/daemon","source_refs":[]}, "lineage": {"parents":[],"transform":"p11-t11-memory"}, "content_digest":format!("sha256:{}", "0".repeat(64)), "created_at":"2026-08-31T00:00:00Z", "valid_time":{"from":"2026-08-31T00:00:00Z","until":null}
        },
        "tenant_id":"personal", "owner_ref":"principal://local/owner", "resource_scope":scope, "conversation_ref":null, "role":"working", "trust_level":"verified", "representation":"text", "provenance_ref":"management://personal/memory/remember", "content_bytes":text.len(), "content_tokens":3, "body":{"text":text}
    });
    let (sealed, source_digest) = seal_governed_object_content_digest(payload).unwrap();
    WorkspaceContextSourceRow {
        source_id: identifier.clone(),
        source_digest,
        governance: ObjectGovernance {
            object_ref: identifier.as_str().to_owned(),
            tenant_id: Some("personal".to_owned()),
            owner_ref: "principal://local/owner".to_owned(),
            resource_scope: scope.to_owned(),
            conversation_ref: None,
        },
        role: LoadedContextItemRole::Working,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        provenance_ref: "management://personal/memory/remember".to_owned(),
        content_bytes: i64::try_from(text.len()).unwrap(),
        content_tokens: Some(3),
        canonical_json: serde_json::to_string(&sealed).unwrap(),
    }
}

fn admit_scoped(
    store: &SqliteAuthorityStore,
    identifier_offset: u64,
    project_id: &str,
    employee_id: &str,
    text: &str,
) -> ObjectId {
    screen_memory_admission(text, "{}").expect("screen");
    let scope = canonical_episodic_scope(project_id, employee_id);
    let source = source_row(&object_id(identifier_offset), &scope, text);
    store.append_workspace_context_source(&source).unwrap();
    let candidate_id = object_id(identifier_offset + 1);
    let candidate_payload = json!({"header":{"id":candidate_id.as_str(),"type":"MemoryCandidate","schema_version":"cognitiveos.memory/0.1","object_version":1,"scope_domain":"tenant","tenant_id":"00000000-0000-7000-9000-0000000000f1","resource_scope_ref":{"kind":"strong","id":"00000000-0000-7000-9000-000000000101","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))},"owner_ref":{"kind":"strong","id":"00000000-0000-7000-9000-000000000102","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))},"authority_ref":{"kind":"strong","id":"00000000-0000-7000-9000-000000000103","object_version":1,"content_digest":format!("sha256:{}", "a".repeat(64))},"policy_refs":[],"purpose_constraints":["task_execution"],"sensitivity":"internal","compartments":[],"retention":{"policy":"standard","expires_at":null,"legal_hold":false},"provenance":{"created_by":"principal://local/owner","source_refs":[]},"lineage":{"parents":[],"transform":"p11-t11-memory"},"content_digest":format!("sha256:{}", "0".repeat(64)),"created_at":"2026-08-31T00:00:00Z","valid_time":{"from":"2026-08-31T00:00:00Z","until":null}},"memory_kind":"working","governance_scope":scope,"purpose":"task_execution","retention":{"policy":"standard","expires_at":null,"legal_hold":false},"source_evidence_refs":[],"conflict_refs":[],"admission_status":"proposed","content_ref":"artifact://memory-content","target_scope":scope});
    let (sealed_candidate, candidate_digest) =
        seal_governed_object_content_digest(candidate_payload).unwrap();
    let candidate = MemoryCandidateRow {
        candidate_id: candidate_id.clone(),
        candidate_digest: candidate_digest.clone(),
        source_id: source.source_id.clone(),
        source_digest: source.source_digest.clone(),
        source_provenance_ref: source.provenance_ref.clone(),
        governance_scope: scope.clone(),
        target_scope: scope,
        purpose: "task_execution".to_owned(),
        retention_expires_at_unix_seconds: 4_000_000_000,
        observed_at_unix_seconds: 100,
        canonical_json: serde_json::to_string(&sealed_candidate).unwrap(),
    };
    let decision_id = object_id(identifier_offset + 2);
    let memory_id = object_id(identifier_offset + 3);
    store
        .append_memory_admission(
            &candidate,
            &MemoryAdmissionDecisionRow {
                decision_id: decision_id.clone(),
                candidate_id: candidate_id.clone(),
                candidate_digest,
                decision: "admit".to_owned(),
                policy_version: 1,
                reason_codes_json: "[\"MEMORY_ADMISSION_ACCEPTED\"]".to_owned(),
                canonical_json: "{}".to_owned(),
            },
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

fn recall_spec<'a>(
    caller_project: &'a str,
    target_project: &'a str,
    caller_employee: &'a str,
    target_employee: &'a str,
    query: &'a str,
) -> EpisodicRecallSpec<'a> {
    EpisodicRecallSpec {
        caller_project_id: caller_project,
        target_project_id: target_project,
        caller_employee_id: caller_employee,
        target_employee_id: target_employee,
        query_text: query,
        purpose: "task_execution",
        observed_at_unix_seconds: 150,
        maximum_results: 8,
    }
}

#[test]
fn p11_t11_cross_scope_episodic_recall_is_rejected() {
    let (_tmp, authority, projects, employees) = stores();
    let project_a = activate(&projects);
    let project_b = activate(&projects);
    let plan_a = plan_two_slots(&projects, &project_a);
    let plan_b = plan_two_slots(&projects, &project_b);
    let ids_a = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_a,
            &plan_a,
            &proposals(),
            21,
        )
        .expect("a");
    let ids_b = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_b,
            &plan_b,
            &proposals(),
            22,
        )
        .expect("b");
    let memory_id = admit_scoped(&authority, 100, &project_a, &ids_a[1], "lantern hangs east");
    let same = recall_episodic_memory(
        &authority,
        &employees,
        &recall_spec(&project_a, &project_a, &ids_a[1], &ids_a[1], "lantern"),
    )
    .expect("same scope");
    assert_eq!(same.len(), 1);
    assert_eq!(same[0].memory_id, memory_id);
    let cross_project = recall_episodic_memory(
        &authority,
        &employees,
        &recall_spec(&project_b, &project_a, &ids_b[1], &ids_a[1], "lantern"),
    )
    .expect_err("N1 project");
    assert!(matches!(
        cross_project,
        ProjectAggregateError::Forbidden { .. }
    ));
    let cross_employee = recall_episodic_memory(
        &authority,
        &employees,
        &recall_spec(&project_a, &project_a, &ids_a[0], &ids_a[1], "lantern"),
    )
    .expect_err("N1 employee");
    assert!(matches!(
        cross_employee,
        ProjectAggregateError::Forbidden { .. }
    ));
}

#[test]
fn p11_t11_secret_and_pii_shaped_candidate_is_denied() {
    let (_tmp, _authority, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    let _ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &proposals(),
            21,
        )
        .expect("roster");
    let secret = screen_memory_admission("api_key=sk-p11t11-fixture-not-a-real-key", "{}")
        .expect_err("N2 secret");
    assert!(matches!(secret, ProjectAggregateError::Invalid { .. }));
    let pii =
        screen_memory_admission("email owner@example.com in the note", "{}").expect_err("N2 pii");
    assert!(matches!(pii, ProjectAggregateError::Invalid { .. }));
    assert!(!projects.leak_scan_contains("sk-p11t11").expect("scan"));
}

#[test]
fn p11_t11_agent_self_and_letta_mem0_direct_write_is_rejected() {
    let letta = screen_memory_admission(
        "ordinary lantern",
        r#"{"source":"letta","text":"ordinary lantern"}"#,
    )
    .expect_err("N3 letta");
    assert!(matches!(letta, ProjectAggregateError::Invalid { .. }));
    let mem0 =
        screen_memory_admission("ordinary lantern", r#"{"engine":"mem0"}"#).expect_err("N3 mem0");
    assert!(matches!(mem0, ProjectAggregateError::Invalid { .. }));
    let self_admit = screen_memory_admission(
        "ordinary lantern",
        r#"{"self_admit":true,"admitted_by":"agent"}"#,
    )
    .expect_err("N3 self");
    assert!(matches!(self_admit, ProjectAggregateError::Invalid { .. }));
}

#[test]
fn p11_t11_forget_then_index_rebuild_cannot_resurrect_scoped_memory() {
    let (_tmp, authority, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &proposals(),
            21,
        )
        .expect("roster");
    let memory_id = admit_scoped(
        &authority,
        200,
        &project_id,
        &ids[1],
        "forgettable compass memory",
    );
    assert_eq!(
        recall_episodic_memory(
            &authority,
            &employees,
            &recall_spec(&project_id, &project_id, &ids[1], &ids[1], "compass"),
        )
        .expect("before forget")
        .len(),
        1
    );
    forget_episodic_memory(
        &authority,
        &employees,
        &project_id,
        &ids[1],
        &MemoryTombstoneRow {
            lifecycle_id: object_id(210),
            memory_id: memory_id.clone(),
            action: "forget".to_owned(),
            occurred_at_unix_seconds: 200,
            reason: "owner forgot scoped Memory".to_owned(),
            canonical_json: "{\"action\":\"forget\"}".to_owned(),
        },
    )
    .expect("forget");
    rebuild_episodic_memory_index(&authority).expect("rebuild");
    assert!(
        recall_episodic_memory(
            &authority,
            &employees,
            &recall_spec(&project_id, &project_id, &ids[1], &ids[1], "compass"),
        )
        .expect("after rebuild")
        .is_empty()
    );
}
