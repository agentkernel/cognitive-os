#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P14-T04 failure-first Dual Track: after Write Project activation,
//! PlanRevision responsible slots must be the ③ posts (collect / analyze /
//! draft), and write-join must seat on those slots. EVAL-016 J4.

use cognitive_store::{
    ConfirmCaller, EmployeeStore, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    RosterProposal, prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (TempDir, ProjectAggregateStore, EmployeeStore) {
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

fn dual_track_charter(title: &str) -> String {
    format!(
        "title: {title}\n\n\
         goal_and_trigger_confirmed: yes\n\n\
         process:\n\
         - collect (收集): confirmed; input=facts; method=read; rights=owner; slot=collect\n\
         - analyze (分析): confirmed; input=facts; method=think; rights=owner; slot=analyze\n\
         - draft (起草): confirmed; input=analysis; method=write; rights=owner; slot=draft\n\n\
         members:\n\
         - 收集岗 stage=collect model=draft-bound seat=seated\n\
         - 分析岗 stage=analyze model=draft-bound seat=seated\n\
         - 起草岗 stage=draft model=draft-bound seat=seated\n\n\
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

fn slot_proposals() -> [RosterProposal; 3] {
    [
        RosterProposal {
            slot: "collect".to_owned(),
            specialization: "project-manager".to_owned(),
            prompt: "collect facts".to_owned(),
            tools_declared: vec![],
        },
        RosterProposal {
            slot: "analyze".to_owned(),
            specialization: "member".to_owned(),
            prompt: "analyze".to_owned(),
            tools_declared: vec![],
        },
        RosterProposal {
            slot: "draft".to_owned(),
            specialization: "member".to_owned(),
            prompt: "draft".to_owned(),
            tools_declared: vec![],
        },
    ]
}

#[test]
fn dual_track_activation_mints_responsible_slots() {
    let (_tmp, projects, _employees) = stores();
    let project_id =
        confirm_activation(&projects, b"Alpha", dual_track_charter("Alpha").as_bytes())
            .expect("Dual Track Write Project");
    let project = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    let plan_id = project
        .current_plan_revision_id
        .expect("PlanRevision axis after T03");
    let stages = projects.list_stages(&plan_id).expect("stages");
    let mut slots: Vec<&str> = stages
        .iter()
        .map(|stage| stage.responsible_slot.as_str())
        .collect();
    slots.sort_unstable();
    assert_eq!(
        slots,
        ["analyze", "collect", "draft"],
        "EVAL-016 J4: ③ posts must be PlanRevision responsible slots, not a collapsed owner slot: {stages:?}"
    );
}

#[test]
fn write_join_seats_members_on_plan_revision_slots() {
    let (_tmp, projects, employees) = stores();
    let project_id =
        confirm_activation(&projects, b"Alpha", dual_track_charter("Alpha").as_bytes())
            .expect("Dual Track Write Project");
    let plan_id = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row")
        .current_plan_revision_id
        .expect("plan");
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &slot_proposals(),
            21,
        )
        .expect("write join register");
    assert_eq!(ids.len(), 3, "one Employee per ③ slot");
    for (index, employee_id) in ids.iter().enumerate() {
        employees
            .request_seating(
                ConfirmCaller::OwnerManagement,
                employee_id,
                30 + index as i64,
            )
            .expect("seat.request");
        let state = employees
            .confirm_seating(
                ConfirmCaller::OwnerManagement,
                employee_id,
                Some("draft-bound"),
                true,
                40 + index as i64,
            )
            .expect("seat.confirm");
        assert_eq!(state, "seated", "{employee_id}");
    }
    let roster = employees.list_roster(&project_id).expect("roster");
    assert_eq!(roster.len(), 3);
    assert!(roster.iter().all(|row| row.state == "seated"));
    assert_eq!(
        roster.iter().filter(|row| row.is_current_manager).count(),
        1
    );
}

#[test]
fn no_slot_fake_join_is_refused() {
    let (_tmp, projects, employees) = stores();
    let project_id = confirm_activation(
        &projects,
        b"Alpha",
        b"title: Alpha\n\ngoal_and_trigger_confirmed: yes\n",
    )
    .expect("G1 without process stays creating");
    let project = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_eq!(project.state, "creating");
    assert!(project.current_plan_revision_id.is_none());
    let error = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            "plan-invented",
            &slot_proposals(),
            21,
        )
        .expect_err("fake join without PlanRevision slots must fail closed");
    assert!(
        matches!(
            error,
            ProjectAggregateError::NotFound { .. }
                | ProjectAggregateError::Invalid { .. }
                | ProjectAggregateError::Rejected { .. }
        ),
        "{error}"
    );
    assert!(
        employees
            .list_roster(&project_id)
            .expect("roster")
            .is_empty()
    );
}

#[test]
fn surplus_slot_join_does_not_seat() {
    let (_tmp, projects, employees) = stores();
    let project_id =
        confirm_activation(&projects, b"Alpha", dual_track_charter("Alpha").as_bytes())
            .expect("Dual Track Write Project");
    let plan_id = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row")
        .current_plan_revision_id
        .expect("plan");
    let error = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &[RosterProposal {
                slot: "shop-install".to_owned(),
                specialization: "member".to_owned(),
                prompt: "install store".to_owned(),
                tools_declared: vec![],
            }],
            21,
        )
        .expect_err("Install-store / unknown slot must not join");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    assert!(
        employees
            .list_roster(&project_id)
            .expect("roster")
            .is_empty()
    );
}

#[test]
fn chat_approve_must_not_join() {
    let (_tmp, projects, employees) = stores();
    let project_id =
        confirm_activation(&projects, b"Alpha", dual_track_charter("Alpha").as_bytes())
            .expect("Dual Track Write Project");
    let plan_id = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row")
        .current_plan_revision_id
        .expect("plan");
    let error = employees
        .register_roster(
            ConfirmCaller::TaskChannel,
            &project_id,
            &plan_id,
            &slot_proposals(),
            21,
        )
        .expect_err("chat Approve must not register");
    assert!(
        matches!(error, ProjectAggregateError::Forbidden { .. }),
        "{error}"
    );
    assert!(
        employees
            .list_roster(&project_id)
            .expect("roster")
            .is_empty()
    );
}
