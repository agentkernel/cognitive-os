#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T09/D01 HITL ApprovalPreview: reject, narrow, mechanical stale, channel fail-closed.
//! Chat/task cannot complete approval. Host UI E2E is not this crate.

use cognitive_store::{
    ConfirmCaller, EmployeeStore, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    RosterProposal, STANDING_POLICY_MAX_TTL_MS, StageSpec, prepare_personal_databases,
};
use tempfile::TempDir;

fn store() -> (TempDir, ProjectAggregateStore) {
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
    let store = ProjectAggregateStore::open_path(&layout.authority_database_path()).expect("open");
    (temporary, store)
}

fn pending_activation(store: &ProjectAggregateStore) -> (String, String, String) {
    let (draft_id, _) = store.create_draft(b"charter-v1", 10).expect("draft");
    store
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = store
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    (draft_id, preview_id, preview_digest)
}

#[test]
fn chat_and_task_channel_cannot_complete_approval() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    for caller in [ConfirmCaller::TaskChannel, ConfirmCaller::Assistant] {
        let confirm = store
            .confirm_preview(caller, &preview_id, &preview_digest, 13)
            .expect_err("chat confirm");
        assert!(matches!(confirm, ProjectAggregateError::Forbidden { .. }));
        let reject = store
            .reject_preview(caller, &preview_id, &preview_digest, 14)
            .expect_err("chat reject");
        assert!(matches!(reject, ProjectAggregateError::Forbidden { .. }));
        let narrow = store
            .narrow_preview(caller, &preview_id, &preview_digest, b"narrower", 15)
            .expect_err("chat narrow");
        assert!(matches!(narrow, ProjectAggregateError::Forbidden { .. }));
    }
    let still = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(still.status, "pending");
}

#[test]
fn stale_is_mechanical_base_digest_mismatch_not_time() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    let later_ms = 12 + 8 * 24 * 60 * 60 * 1000;
    store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            later_ms,
        )
        .expect("elapsed wall clock is not stale");
}

#[test]
fn wrong_digest_fail_closed_on_confirm_reject_narrow() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    let wrong = "0".repeat(64);
    let confirm = store
        .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &wrong, 20)
        .expect_err("wrong confirm digest");
    assert!(matches!(confirm, ProjectAggregateError::Stale { .. }));
    let reject = store
        .reject_preview(ConfirmCaller::OwnerManagement, &preview_id, &wrong, 21)
        .expect_err("wrong reject digest");
    assert!(matches!(reject, ProjectAggregateError::Stale { .. }));
    let narrow = store
        .narrow_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &wrong,
            b"narrower",
            22,
        )
        .expect_err("wrong narrow digest");
    assert!(matches!(narrow, ProjectAggregateError::Stale { .. }));
    let still = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(still.status, "pending");
    assert_eq!(still.preview_digest, preview_digest);
}

#[test]
fn reject_leaves_receipt_and_rejected_digest_is_not_confirmable() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    let receipt = store
        .reject_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            30,
        )
        .expect("reject");
    assert!(receipt.contains(&preview_id));
    let detail = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.status, "rejected");
    assert_eq!(detail.receipt_ref.as_deref(), Some(receipt.as_str()));
    let confirm = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            31,
        )
        .expect_err("rejected digest");
    assert!(matches!(confirm, ProjectAggregateError::Invalid { .. }));
    assert!(store.list_projects(16).expect("list").is_empty());
}

#[test]
fn narrow_mints_new_preview_and_freezes_old_digest() {
    let (_tmp, store) = store();
    let (draft_id, old_id, old_digest) = pending_activation(&store);
    let narrowed = store
        .narrow_preview(
            ConfirmCaller::OwnerManagement,
            &old_id,
            &old_digest,
            b"narrowed-activation-preview",
            40,
        )
        .expect("narrow");
    assert_ne!(narrowed.preview_id, old_id);
    assert_ne!(narrowed.preview_digest, old_digest);
    assert_eq!(narrowed.superseded_preview_id, old_id);
    let old = store.preview_detail(&old_id).expect("old").expect("row");
    assert_eq!(old.status, "superseded");
    assert_eq!(
        old.superseded_by.as_deref(),
        Some(narrowed.preview_id.as_str())
    );
    let old_confirm = store
        .confirm_preview(ConfirmCaller::OwnerManagement, &old_id, &old_digest, 41)
        .expect_err("old digest");
    assert!(matches!(old_confirm, ProjectAggregateError::Invalid { .. }));
    let pending = store.list_pending_previews(&draft_id).expect("pending");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].preview_id, narrowed.preview_id);
    let result = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &narrowed.preview_id,
            &narrowed.preview_digest,
            42,
        )
        .expect("new pending still confirmable");
    assert_eq!(result.kind, "activated");
}

#[test]
fn confirm_still_works_for_a_fresh_pending_after_reject() {
    let (_tmp, store) = store();
    let (draft_id, first_id, first_digest) = pending_activation(&store);
    store
        .reject_preview(ConfirmCaller::OwnerManagement, &first_id, &first_digest, 50)
        .expect("reject first");
    let (second_id, second_digest) = store
        .request_preview("activation", &draft_id, b"second-preview", 51)
        .expect("second preview");
    let result = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &second_id,
            &second_digest,
            52,
        )
        .expect("second confirm");
    assert_eq!(result.kind, "activated");
}

fn stores_pair() -> (TempDir, ProjectAggregateStore, EmployeeStore) {
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
    let employees = EmployeeStore::open_path(&path).expect("employees");
    (temporary, projects, employees)
}

fn grant_subject(project_id: &str, employee_id: &str) -> String {
    serde_json::json!({
        "project_id": project_id,
        "employee_id": employee_id,
        "capability_ref": "mcp:search",
        "scope": "project-a"
    })
    .to_string()
}

fn seated_employee(
    projects: &ProjectAggregateStore,
    employees: &EmployeeStore,
) -> (String, String) {
    let project_id = {
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
    };
    let plan_id = projects
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[StageSpec {
                stage_id: "s1".to_owned(),
                title: "Manage".to_owned(),
                objective: "manage".to_owned(),
                output_contract_digest: ProjectAggregateStore::digest_hex(b"out"),
                acceptance_spec_ref: Some("cas:spec".to_owned()),
                cadence_json: None,
                responsible_slot: "manager".to_owned(),
                blocking_gap: None,
            }],
            20,
        )
        .expect("plan");
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
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
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
        .expect("seat request");
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            31,
        )
        .expect("seat confirm");
    employees
        .record_install_fact("mcp:search", "1.0.0", 32)
        .expect("install");
    (project_id, ids[0].clone())
}

#[test]
fn standing_policy_missing_expires_at_is_rejected() {
    let (_tmp, store) = store();
    let err = store
        .create_standing_policy(
            ConfirmCaller::OwnerManagement,
            "outbound",
            "grant-expansion",
            None,
            1_000,
        )
        .expect_err("missing expires_at");
    assert!(matches!(err, ProjectAggregateError::Invalid { .. }));
    assert!(format!("{err}").contains("expires_at required"));
}

#[test]
fn standing_policy_over_seven_days_is_rejected() {
    let (_tmp, store) = store();
    let now = 1_000;
    let err = store
        .create_standing_policy(
            ConfirmCaller::OwnerManagement,
            "outbound",
            "grant-expansion",
            Some(now + STANDING_POLICY_MAX_TTL_MS + 1),
            now,
        )
        .expect_err(">7d");
    assert!(matches!(err, ProjectAggregateError::Invalid { .. }));
    assert!(format!("{err}").contains("7-day"));
    let chat = store
        .create_standing_policy(
            ConfirmCaller::Assistant,
            "outbound",
            "grant-expansion",
            Some(now + 60_000),
            now,
        )
        .expect_err("chat");
    assert!(matches!(chat, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn standing_policy_lists_and_revokes_on_owner_path() {
    let (_tmp, store) = store();
    let now = 1_000;
    let expires = now + 3 * 24 * 60 * 60 * 1000;
    let policy_id = store
        .create_standing_policy(
            ConfirmCaller::OwnerManagement,
            "outbound",
            "grant-expansion",
            Some(expires),
            now,
        )
        .expect("create");
    let listed = store.list_standing_policies(now + 1).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].policy_id, policy_id);
    assert!(listed[0].active);
    assert_eq!(listed[0].expires_at, expires);
    store
        .revoke_standing_policy(ConfirmCaller::TaskChannel, &policy_id, now + 2)
        .expect_err("task cannot revoke");
    store
        .revoke_standing_policy(ConfirmCaller::OwnerManagement, &policy_id, now + 3)
        .expect("revoke");
    assert!(
        store
            .list_standing_policies(now + 4)
            .expect("empty")
            .is_empty()
    );
}

#[test]
fn grant_expansion_preview_confirmable_on_owner_not_chat() {
    let (_tmp, projects, employees) = stores_pair();
    let (project_id, employee_id) = seated_employee(&projects, &employees);
    let subject = grant_subject(&project_id, &employee_id);
    let (preview_id, digest) = projects
        .request_preview("grant-expansion", &subject, b"expand-mcp-search", 40)
        .expect("preview");
    let chat = projects
        .confirm_preview(ConfirmCaller::Assistant, &preview_id, &digest, 41)
        .expect_err("chat");
    assert!(matches!(chat, ProjectAggregateError::Forbidden { .. }));
    let result = projects
        .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 42)
        .expect("owner confirm");
    assert_eq!(result.kind, "granted");
    let catalog = employees
        .tool_catalog(&project_id, &employee_id)
        .expect("catalog");
    assert_eq!(catalog, vec!["mcp:search".to_owned()]);
}

#[test]
fn unsupported_subject_kind_is_rejected() {
    let (_tmp, store) = store();
    let err = store
        .request_preview("inbox", "x", b"nope", 1)
        .expect_err("inbox kind");
    assert!(matches!(err, ProjectAggregateError::Invalid { .. }));
}
