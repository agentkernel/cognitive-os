#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T10 Markdown Vault: secret-shape, file-as-authority, LWW, overreach,
//! Memory admission fence, rebuildable index.

use cognitive_store::{
    CONTEXT_INJECT_ORDER, ConfirmCaller, PersonalDataLayout, ProjectAggregateError,
    ProjectAggregateStore, VAULT_PROJECTION_ID, VaultImportSpec, VaultReadSpec, VaultStore,
    prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (TempDir, ProjectAggregateStore, VaultStore) {
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
    (temporary, projects, vault)
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

fn import_spec<'a>(
    project_id: &'a str,
    relative_path: &'a str,
    body: &'a str,
    conflict_policy: Option<&'a str>,
    now_ms: i64,
) -> VaultImportSpec<'a> {
    VaultImportSpec {
        project_id,
        relative_path,
        rights_class: "owner-owned",
        provenance_json: r#"{"source_uri":"owner-paste:notes"}"#,
        source_kind: "markdown-file",
        body,
        cas_ref: None,
        conflict_policy,
        now_ms,
    }
}

fn same_project<'a>(project_id: &'a str) -> VaultReadSpec<'a> {
    VaultReadSpec {
        caller_project_id: project_id,
        target_project_id: project_id,
    }
}

#[test]
fn p11_t10_secret_shape_is_rejected_on_import() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    let error = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/secret.md",
                "token api_key=sk-p11t10-fixture-not-a-real-key",
                None,
                40,
            ),
        )
        .expect_err("secret");
    assert!(
        matches!(error, ProjectAggregateError::Invalid { detail } if detail.contains("secret-shaped"))
    );
}

#[test]
fn p11_t10_file_cannot_confirm_or_apply_project_authority() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    let document_id = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/charter-lookalike.md",
                "# Charter\n\nThis markdown must not activate or confirm a Project.",
                None,
                41,
            ),
        )
        .expect("import");
    let apply = vault
        .apply_as_project_authority(&document_id)
        .expect_err("authority");
    assert!(matches!(
        apply,
        ProjectAggregateError::Invalid { detail } if detail.contains("not Project authority")
    ));
    let listed = projects.list_projects(16).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].project_id, project_id);
}

#[test]
fn p11_t10_last_write_wins_without_conflict_record_is_rejected() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(&project_id, "notes/a.md", "version one", None, 42),
        )
        .expect("first");
    let lww = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/a.md",
                "version two silently overwrites",
                Some("last-write-wins"),
                43,
            ),
        )
        .expect_err("lww");
    assert!(matches!(
        lww,
        ProjectAggregateError::Invalid { detail } if detail.contains("last-write-wins")
    ));
    let omitted = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/a.md",
                "version two omitted policy",
                None,
                44,
            ),
        )
        .expect_err("omitted");
    assert!(matches!(omitted, ProjectAggregateError::Invalid { .. }));
}

#[test]
fn p11_t10_conflict_record_keeps_both_and_index_rebuilds() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    let first = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/a.md",
                "version one\n\nolder narrative",
                None,
                45,
            ),
        )
        .expect("first");
    let second = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(
                &project_id,
                "notes/a.md",
                "version two\n\nreplacement excerpt",
                Some("record"),
                46,
            ),
        )
        .expect("second");
    assert_ne!(first, second);
    let conflicts = vault
        .list_conflicts(&same_project(&project_id))
        .expect("conflicts");
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].resolution, "open");
    assert_eq!(conflicts[0].incumbent_document_id, first);
    assert_eq!(conflicts[0].incoming_document_id, second);
    let before_fts = vault.memory_fts_row_count().expect("fts before");
    let written = vault
        .rebuild_index(ConfirmCaller::OwnerManagement, &project_id, 47)
        .expect("rebuild");
    assert!(written >= 2);
    let after_fts = vault.memory_fts_row_count().expect("fts after");
    assert_eq!(after_fts, before_fts);
    vault
        .rebuild_index(ConfirmCaller::OwnerManagement, &project_id, 48)
        .expect("rebuild again");
    let index = vault.read_index(&same_project(&project_id)).expect("index");
    assert!(!index.is_empty());
    assert!(index.iter().all(|entry| entry.layer == "sourced-excerpt"));
    let plan = vault
        .assemble_context_inject_order(&same_project(&project_id))
        .expect("inject");
    assert_eq!(
        plan.order,
        CONTEXT_INJECT_ORDER
            .iter()
            .map(|layer| (*layer).to_owned())
            .collect::<Vec<_>>()
    );
    assert!(!plan.excerpts.is_empty());
    assert_eq!(
        VAULT_PROJECTION_ID,
        "cognitiveos.personal.markdown-vault/0.1"
    );
}

#[test]
fn p11_t10_cross_project_vault_read_is_rejected() {
    let (_tmp, projects, vault) = stores();
    let project_a = activate(&projects);
    vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(&project_a, "notes/a.md", "private to A", None, 49),
        )
        .expect("import");
    vault
        .rebuild_index(ConfirmCaller::OwnerManagement, &project_a, 50)
        .expect("rebuild");
    let overreach = vault
        .read_index(&VaultReadSpec {
            caller_project_id: "task://personal/other",
            target_project_id: &project_a,
        })
        .expect_err("overreach");
    assert!(matches!(
        overreach,
        ProjectAggregateError::Forbidden { detail } if detail.contains("cross-project")
    ));
    let conflicts = vault
        .list_conflicts(&VaultReadSpec {
            caller_project_id: "task://personal/other",
            target_project_id: &project_a,
        })
        .expect_err("conflict overreach");
    assert!(matches!(conflicts, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p11_t10_memory_admission_cannot_swallow_vault_files() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    let document_id = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(&project_id, "notes/memory.md", "do not admit", None, 51),
        )
        .expect("import");
    let admit = vault.admit_as_memory(&document_id).expect_err("memory");
    assert!(matches!(
        admit,
        ProjectAggregateError::Invalid { detail } if detail.contains("Memory admission")
    ));
}

#[test]
fn p11_t10_conversation_and_cas_are_not_vault_files() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    let conversation = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &VaultImportSpec {
                project_id: &project_id,
                relative_path: "notes/from-chat.md",
                rights_class: "owner-owned",
                provenance_json: r#"{"source_uri":"conversation-archive"}"#,
                source_kind: "conversation-archive",
                body: "archive dump",
                cas_ref: None,
                conflict_policy: None,
                now_ms: 52,
            },
        )
        .expect_err("conversation");
    assert!(matches!(
        conversation,
        ProjectAggregateError::Invalid { detail } if detail.contains("conversation archive")
    ));
    let cas = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &VaultImportSpec {
                project_id: &project_id,
                relative_path: "notes/cas-only.md",
                rights_class: "owner-owned",
                provenance_json: r#"{"source_uri":"cas"}"#,
                source_kind: "markdown-file",
                body: "",
                cas_ref: Some("sha256:deadbeef"),
                conflict_policy: None,
                now_ms: 53,
            },
        )
        .expect_err("cas");
    assert!(matches!(
        cas,
        ProjectAggregateError::Invalid { detail } if detail.contains("artifact CAS")
    ));
}

#[test]
fn p11_t10_authority_sqlite_omits_secret_shape_bytes_after_import() {
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
    let project_id = activate(&projects);
    vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(&project_id, "notes/ok.md", "research notes only", None, 60),
        )
        .expect("import");
    vault
        .rebuild_index(ConfirmCaller::OwnerManagement, &project_id, 61)
        .expect("rebuild");
    let haystack = String::from_utf8_lossy(&std::fs::read(&path).expect("sqlite"));
    assert!(
        !haystack.contains("sk-"),
        "authority sqlite must not contain API key material"
    );
    assert!(!haystack.contains("Bearer "));
}

#[test]
fn p11_t10_path_traversal_is_rejected() {
    let (_tmp, projects, vault) = stores();
    let project_id = activate(&projects);
    let error = vault
        .import(
            ConfirmCaller::OwnerManagement,
            &import_spec(&project_id, "../etc/passwd", "nope", None, 54),
        )
        .expect_err("traversal");
    assert!(matches!(
        error,
        ProjectAggregateError::Invalid { detail } if detail.contains("traversal")
    ));
}
