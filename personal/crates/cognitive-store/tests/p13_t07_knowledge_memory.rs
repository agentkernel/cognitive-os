#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T07: Knowledge fragment labels + Memory promote / chat auto-admission.
//!
//! Failure-first: vault.index labels, not-indexed import visibility, Owner
//! promote preview, Agent self-admission and secret-shaped chat refused.
//! Files are not Project authority. Host FS/privacy E2E is out of this file.

use cognitive_domain::ObjectId;
use cognitive_kernel::ports::MemoryTombstoneRow;
use cognitive_store::memory_store::KnowledgeMemoryStore;
use cognitive_store::{
    CONVERSATION_ARCHIVE_PROJECTION_ID, ConfirmCaller, ConversationStore, EmployeeStore,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, RosterProposal,
    SqliteAuthorityStore, StageSpec, VaultImportSpec, VaultReadSpec, VaultStore,
    forget_episodic_memory, prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    ProjectAggregateStore,
    VaultStore,
    ConversationStore,
    EmployeeStore,
    KnowledgeMemoryStore,
    std::path::PathBuf,
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
    let projects = ProjectAggregateStore::open_path(&path).expect("projects");
    let vault = VaultStore::open_path(&path).expect("vault");
    let conversations = ConversationStore::open_path(&path).expect("conversations");
    let employees = EmployeeStore::open_path(&path).expect("employees");
    let knowledge = KnowledgeMemoryStore::open_path(&path).expect("knowledge");
    (
        temporary,
        projects,
        vault,
        conversations,
        employees,
        knowledge,
        path,
    )
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

fn seat_manager(
    projects: &ProjectAggregateStore,
    employees: &EmployeeStore,
    project_id: &str,
) -> String {
    let plan_id = projects
        .apply_plan_revision(
            project_id,
            project_id,
            &[stage("s1", "Manage", "manager")],
            20,
        )
        .expect("plan");
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            project_id,
            &plan_id,
            &[RosterProposal {
                slot: "manager".to_owned(),
                specialization: "project-manager".to_owned(),
                prompt: "coordinate".to_owned(),
                tools_declared: vec!["workspace-write".to_owned()],
            }],
            21,
        )
        .expect("roster");
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 22)
        .expect("seating");
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            23,
        )
        .expect("seat");
    ids[0].clone()
}

fn same_project(project_id: &str) -> VaultReadSpec<'_> {
    VaultReadSpec {
        caller_project_id: project_id,
        target_project_id: project_id,
    }
}

fn import_spec<'a>(
    project_id: &'a str,
    relative_path: &'a str,
    rights_class: &'a str,
    provenance_json: &'a str,
    body: &'a str,
    now_ms: i64,
) -> VaultImportSpec<'a> {
    VaultImportSpec {
        project_id,
        relative_path,
        rights_class,
        provenance_json,
        source_kind: "owner-paste",
        body,
        cas_ref: None,
        conflict_policy: None,
        now_ms,
    }
}

#[test]
fn p13_t07_labeled_fragments_expose_provenance_rights_freshness_exclusion() {
    let (_tmp, projects, vault, _, _, _, _) = stores();
    let project_id = activate(&projects);
    vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/owned.md",
                "owner-owned",
                r#"{"source_uri":"owner-paste:owned"}"#,
                "Owned excerpt one.",
                1_000,
            ),
        )
        .expect("import owned");
    vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/cite.md",
                "citation-only",
                r#"{"source_uri":"https://example.invalid/cite"}"#,
                "Citation excerpt two.",
                1_100,
            ),
        )
        .expect("import cite");
    vault
        .rebuild_index(ConfirmCaller::OwnerManagement, &project_id, 1_200)
        .expect("rebuild");
    let mut owned_v2 = import_spec(
        &project_id,
        "notes/owned.md",
        "owner-owned",
        r#"{"source_uri":"owner-paste:owned-v2"}"#,
        "Owned excerpt one revised.",
        1_300,
    );
    owned_v2.conflict_policy = Some("record");
    vault
        .import(ConfirmCaller::OwnerManagement, &owned_v2)
        .expect("import owned v2");
    vault
        .rebuild_index(ConfirmCaller::OwnerManagement, &project_id, 1_400)
        .expect("rebuild v2");

    let labels = vault
        .read_labeled_index(&same_project(&project_id))
        .expect("labels");
    assert!(
        labels.iter().any(|row| {
            row.provenance_source_uri == "owner-paste:owned-v2"
                && row.rights_class == "owner-owned"
                && row.freshness == "current"
                && row.exclusion == "included"
                && !row.untrusted_observation
        }),
        "current owner-owned fragment missing labels: {labels:?}"
    );
    assert!(
        labels.iter().any(|row| {
            row.provenance_source_uri == "owner-paste:owned" && row.freshness == "superseded"
        }),
        "superseded owner-owned fragment missing: {labels:?}"
    );
    assert!(
        labels.iter().any(|row| {
            row.rights_class == "citation-only"
                && row.exclusion == "excluded"
                && row.exclusion_reason == "citation-only"
                && row.untrusted_observation
                && row.provenance_source_uri == "https://example.invalid/cite"
        }),
        "citation-only fragment must be excluded and untrusted: {labels:?}"
    );
    assert!(
        labels.iter().all(|row| !row.is_authority),
        "Vault fragments must never claim Project authority: {labels:?}"
    );
}

#[test]
fn p13_t07_import_without_rebuild_keeps_not_indexed_document_visible() {
    let (_tmp, projects, vault, _, _, _, _) = stores();
    let project_id = activate(&projects);
    let document_id = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/pending.md",
                "owner-owned",
                r#"{"source_uri":"owner-paste:pending"}"#,
                "Stored but not indexed yet.",
                2_000,
            ),
        )
        .expect("import");
    let statuses = vault
        .list_document_statuses(&same_project(&project_id))
        .expect("statuses");
    let pending = statuses
        .iter()
        .find(|row| row.document_id == document_id)
        .expect("imported document remains visible");
    assert_eq!(pending.index_status, "not-indexed");
    assert_eq!(pending.provenance_source_uri, "owner-paste:pending");
    assert_eq!(pending.relative_path, "notes/pending.md");
}

#[test]
fn p13_t07_cross_project_labeled_read_is_forbidden() {
    let (_tmp, projects, vault, _, _, _, _) = stores();
    let project_id = activate(&projects);
    let error = vault
        .read_labeled_index(&VaultReadSpec {
            caller_project_id: "task://personal/other",
            target_project_id: &project_id,
        })
        .expect_err("overreach");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p13_t07_vault_file_still_cannot_become_project_authority() {
    let (_tmp, projects, vault, _, _, _, _) = stores();
    let project_id = activate(&projects);
    let document_id = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/note.md",
                "owner-owned",
                r#"{"source_uri":"owner-paste:note"}"#,
                "Note.",
                3_000,
            ),
        )
        .expect("import");
    let error = vault
        .apply_as_project_authority(&document_id)
        .expect_err("file-as-authority");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
}

#[test]
fn p13_t07_agent_cannot_self_admit_chat_into_memory() {
    let (_tmp, projects, _, conversations, employees, knowledge, _) = stores();
    let project_id = activate(&projects);
    let employee_id = seat_manager(&projects, &employees, &project_id);
    let record_id = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &cognitive_store::ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: &employee_id,
                kind: "note",
                body: "Please remember this coordination note.",
                now_ms: 4_000,
            },
        )
        .expect("archive");
    let error = knowledge
        .auto_admit_chat(
            ConfirmCaller::Assistant,
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &record_id,
            4_100,
        )
        .expect_err("agent self-admission");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p13_t07_secret_shaped_chat_is_not_admitted() {
    let (_tmp, projects, _, conversations, employees, knowledge, _) = stores();
    let project_id = activate(&projects);
    let employee_id = seat_manager(&projects, &employees, &project_id);
    let record_id = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &cognitive_store::ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: &employee_id,
                kind: "note",
                body: "sk-ant-api03-not-a-real-secret-but-shaped",
                now_ms: 5_000,
            },
        )
        .expect_err("secret-shaped archive should already fail")
        .to_string();
    if record_id.contains("secret") || record_id.contains("shape") {
        return;
    }
    let error = knowledge
        .auto_admit_chat(
            ConfirmCaller::OwnerManagement,
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            "missing-record",
            5_100,
        )
        .expect_err("secret or missing");
    assert!(matches!(
        error,
        ProjectAggregateError::Invalid { .. } | ProjectAggregateError::NotFound { .. }
    ));
}

#[test]
fn p13_t07_cross_project_promote_requires_owner_confirm() {
    let (_tmp, projects, _, conversations, employees, knowledge, _) = stores();
    let from_project = activate(&projects);
    let to_project = activate(&projects);
    let from_employee = seat_manager(&projects, &employees, &from_project);
    let to_employee = seat_manager(&projects, &employees, &to_project);
    let record_id = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &cognitive_store::ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &from_project,
                employee_id: &from_employee,
                kind: "note",
                body: "Admitted coordination note for promote.",
                now_ms: 6_000,
            },
        )
        .expect("archive");
    let admitted = knowledge
        .auto_admit_chat(
            ConfirmCaller::OwnerManagement,
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &from_project,
            &record_id,
            6_100,
        )
        .expect("owner auto-admit");
    let pending = knowledge
        .request_promote(
            ConfirmCaller::OwnerManagement,
            &admitted.memory_id,
            &from_project,
            &to_project,
            &to_employee,
            6_200,
        )
        .expect("promote preview");
    assert_eq!(pending.status, "pending");
    assert_ne!(pending.from_project_id, pending.to_project_id);
    let target_before = knowledge
        .list_promotes(&to_project)
        .expect("target list")
        .into_iter()
        .filter(|row| row.status == "confirmed")
        .count();
    assert_eq!(target_before, 0, "unconfirmed promote must not copy Memory");
    let confirmed = knowledge
        .confirm_promote(
            ConfirmCaller::OwnerManagement,
            &pending.promote_id,
            &pending.preview_digest,
            6_300,
        )
        .expect("confirm");
    assert_eq!(confirmed.status, "confirmed");
    assert_eq!(confirmed.to_project_id, to_project);
    assert_ne!(
        confirmed.memory_id,
        confirmed.promoted_memory_id.as_deref().unwrap_or("")
    );
}

#[test]
fn p13_t07_tombstoned_memory_cannot_be_promoted() {
    let (_tmp, projects, _, conversations, employees, knowledge, path) = stores();
    let from_project = activate(&projects);
    let to_project = activate(&projects);
    let from_employee = seat_manager(&projects, &employees, &from_project);
    let to_employee = seat_manager(&projects, &employees, &to_project);
    let record_id = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &cognitive_store::ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &from_project,
                employee_id: &from_employee,
                kind: "note",
                body: "Note that will be forgotten.",
                now_ms: 7_000,
            },
        )
        .expect("archive");
    let admitted = knowledge
        .auto_admit_chat(
            ConfirmCaller::OwnerManagement,
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &from_project,
            &record_id,
            7_100,
        )
        .expect("admit");
    let store = SqliteAuthorityStore::open(&path).expect("reopen");
    let lifecycle = ObjectId::parse(&uuid::Uuid::now_v7().as_hyphenated().to_string()).expect("id");
    forget_episodic_memory(
        &store,
        &employees,
        &from_project,
        &from_employee,
        &MemoryTombstoneRow {
            lifecycle_id: lifecycle,
            memory_id: ObjectId::parse(&admitted.memory_id).expect("memory"),
            action: "forget".to_owned(),
            occurred_at_unix_seconds: 8,
            reason: "owner forget".to_owned(),
            canonical_json: "{}".to_owned(),
        },
    )
    .expect("tombstone");
    let error = knowledge
        .request_promote(
            ConfirmCaller::OwnerManagement,
            &admitted.memory_id,
            &from_project,
            &to_project,
            &to_employee,
            7_200,
        )
        .expect_err("tombstone must not promote");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
}
