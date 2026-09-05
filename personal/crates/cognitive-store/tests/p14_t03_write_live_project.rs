#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P14-T03 failure-first Dual Track: Write Project must mint a titled live
//! Project with a PlanRevision axis and must leave `creating`.
//! EVAL-016 J1 blocker 1.

use cognitive_store::{
    ConfirmCaller, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    prepare_personal_databases,
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

fn dual_track_charter(title: &str) -> String {
    format!(
        "title: {title}\n\n\
         goal_and_trigger_confirmed: yes\n\n\
         process:\n\
         - collect (收集): confirmed; input=facts; method=read; rights=owner\n\
         - analyze (分析): confirmed; input=facts; method=think; rights=owner\n\
         - draft (起草): confirmed; input=analysis; method=write; rights=owner\n\n\
         honesty: owner-recorded Dual Track draft; local notes are not Project authority.\n"
    )
}

fn confirm_activation(
    store: &ProjectAggregateStore,
    payload: &[u8],
    charter: &[u8],
) -> Result<String, ProjectAggregateError> {
    let (draft_id, _) = store.create_draft(payload, 10)?;
    store.put_draft_charter(&draft_id, charter, 11)?;
    let (preview_id, preview_digest) =
        store.request_preview("activation", &draft_id, b"activation-preview", 12)?;
    store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .map(|result| result.new_ref)
}

#[test]
fn empty_title_must_not_activate() {
    let (_tmp, store) = store();
    let error = confirm_activation(&store, b"", dual_track_charter("Alpha").as_bytes())
        .expect_err("empty title must fail closed");
    assert!(
        matches!(
            error,
            ProjectAggregateError::Invalid { .. } | ProjectAggregateError::Rejected { .. }
        ),
        "{error}"
    );
    assert!(store.list_projects(16).expect("list").is_empty());
}

#[test]
fn unknown_title_must_not_activate() {
    let (_tmp, store) = store();
    let error = confirm_activation(&store, b"unknown", dual_track_charter("unknown").as_bytes())
        .expect_err("title unknown must fail closed");
    assert!(
        matches!(
            error,
            ProjectAggregateError::Invalid { .. } | ProjectAggregateError::Rejected { .. }
        ),
        "{error}"
    );
    assert!(store.list_projects(16).expect("list").is_empty());
}

#[test]
fn write_project_must_leave_creating_with_axis() {
    let (_tmp, store) = store();
    let project_id = confirm_activation(&store, b"Alpha", dual_track_charter("Alpha").as_bytes())
        .expect("titled Dual Track Write Project");
    let project = store.get_project(&project_id).expect("get").expect("row");
    assert_ne!(
        project.state, "creating",
        "EVAL-016 J1 blocker 1: Write Project must leave creating"
    );
    assert!(
        project.accepted_at.is_some(),
        "live Project requires accepted_at"
    );
    assert!(
        project.current_plan_revision_id.is_some(),
        "EVAL-016 J1 blocker 1: PlanRevision axis must exist"
    );
    assert_eq!(project.state, "active");
    assert_eq!(project.title_summary, "Alpha");
}

#[test]
fn live_without_axis_must_be_refused() {
    let (_tmp, store) = store();
    let charter = "title: Alpha\n\ngoal_and_trigger_confirmed: yes\n\nprocess:\n";
    let error = confirm_activation(&store, b"Alpha", charter.as_bytes())
        .expect_err("no process axis must not mark live");
    assert!(
        matches!(
            error,
            ProjectAggregateError::Invalid { .. } | ProjectAggregateError::Rejected { .. }
        ),
        "{error}"
    );
    assert!(store.list_projects(16).expect("list").is_empty());
}
