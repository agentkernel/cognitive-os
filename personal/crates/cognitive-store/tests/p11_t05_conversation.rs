#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T05 conversation archive: T04-N9 handoff + T05-N1/N2/N3.

use cognitive_store::{
    CONVERSATION_ARCHIVE_PROJECTION_ID, ConfirmCaller, ConversationStore, EmployeeStore,
    LEGACY_CONVERSATION_PROJECTION_ID, PersonalDataLayout, ProjectAggregateError,
    ProjectAggregateStore, RosterProposal, SpeechArchiveSpec, StageSpec,
    prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    ProjectAggregateStore,
    EmployeeStore,
    ConversationStore,
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
    let employees = EmployeeStore::open_path(&path).expect("employees");
    let conversations = ConversationStore::open_path(&path).expect("conversations");
    (temporary, projects, employees, conversations)
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
    let result = projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("G1");
    result.new_ref
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
            prompt: "use workspace-write to file notes".to_owned(),
            tools_declared: vec!["workspace-write".to_owned()],
        },
    ]
}

fn seat_manager(employees: &EmployeeStore, ids: &[String]) {
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
        .expect("request");
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            31,
        )
        .expect("seat");
}

fn spec<'a>(
    project_id: &'a str,
    employee_id: &'a str,
    kind: &'a str,
    body: &'a str,
    now_ms: i64,
) -> SpeechArchiveSpec<'a> {
    SpeechArchiveSpec {
        projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
        project_id,
        employee_id,
        kind,
        mentioned: false,
        body,
        now_ms,
    }
}

#[test]
fn p11_t05_deliverable_lands_chatter_does_not() {
    let (_tmp, projects, employees, conversations) = stores();
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
    seat_manager(&employees, &ids);
    let chatter = conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "chatter", "side talk", 50),
        )
        .expect("chatter");
    assert!(!chatter.delivered);
    assert!(chatter.record_id.is_none());
    assert!(!chatter.audit_id.is_empty());
    let after_chatter = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
        )
        .expect("read");
    assert!(after_chatter.is_empty());
    let deliverable = conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "deliverable", "openable note", 51),
        )
        .expect("deliverable");
    assert!(deliverable.delivered);
    assert!(deliverable.record_id.is_some());
    let rows = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            Some(&ids[1]),
        )
        .expect("scoped");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, "deliverable");
    assert_eq!(rows[0].body_redacted, "openable note");
    assert_eq!(rows[0].projection_id, CONVERSATION_ARCHIVE_PROJECTION_ID);
    let project = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_ne!(project.state, "accepted");
}

#[test]
fn p11_t05_legacy_projection_not_coerced() {
    let (_tmp, projects, employees, conversations) = stores();
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
    seat_manager(&employees, &ids);
    for legacy in [LEGACY_CONVERSATION_PROJECTION_ID, "v01"] {
        let mut rejected = spec(&project_id, &ids[1], "deliverable", "note", 60);
        rejected.projection_id = legacy;
        let error = conversations
            .land_speech(&employees, &rejected)
            .expect_err("N1 land");
        assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
        let error = conversations
            .read_scoped(legacy, &project_id, &project_id, None)
            .expect_err("N1 read");
        assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    }
    let rows = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
        )
        .expect("empty");
    assert!(rows.is_empty());
}

#[test]
fn p11_t05_append_rejects_secret_shape() {
    let (_tmp, projects, employees, conversations) = stores();
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
    seat_manager(&employees, &ids);
    let error = conversations
        .land_speech(
            &employees,
            &spec(
                &project_id,
                &ids[1],
                "deliverable",
                "api_key=sk-p11t05-fixture-not-a-real-key",
                70,
            ),
        )
        .expect_err("N2");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    assert!(!projects.leak_scan_contains("sk-p11t05").expect("scan"));
    let rows = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
        )
        .expect("empty");
    assert!(rows.is_empty());
}

#[test]
fn p11_t05_cross_scope_read_rejected() {
    let (_tmp, projects, employees, conversations) = stores();
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
    seat_manager(&employees, &ids_a);
    seat_manager(&employees, &ids_b);
    conversations
        .land_speech(
            &employees,
            &spec(&project_a, &ids_a[1], "deliverable", "only in a", 80),
        )
        .expect("land a");
    let cross_project = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_b,
            &project_a,
            None,
        )
        .expect_err("N3 caller/target");
    assert!(matches!(
        cross_project,
        ProjectAggregateError::Forbidden { .. }
    ));
    let cross_employee = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_b,
            &project_b,
            Some(&ids_a[1]),
        )
        .expect_err("N3 employee");
    assert!(matches!(
        cross_employee,
        ProjectAggregateError::Forbidden { .. }
    ));
    let b_rows = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_b,
            &project_b,
            None,
        )
        .expect("b empty");
    assert!(b_rows.is_empty());
    let a_rows = conversations
        .read_scoped(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_a,
            &project_a,
            None,
        )
        .expect("a");
    assert_eq!(a_rows.len(), 1);
    assert_eq!(a_rows[0].body_redacted, "only in a");
}
