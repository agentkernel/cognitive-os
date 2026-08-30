#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T05 conversation archive: T04-N9 + T05-N1..N6.

use cognitive_store::{
    ArchiveAppendSpec, ArchiveReadSpec, CONVERSATION_ARCHIVE_PROJECTION_ID,
    CONVERSATION_BODY_LIMIT, CONVERSATION_RESUME_LIMIT, ConfirmCaller, ConversationStore,
    EmployeeStore, LEGACY_CONVERSATION_PROJECTION_ID, PersonalDataLayout, ProjectAggregateError,
    ProjectAggregateStore, RosterProposal, SeatingFacts, SpeechArchiveSpec, StageSpec,
    StageTestOracle, prepare_personal_databases,
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

fn index_spec<'a>(
    projection_id: &'a str,
    caller: &'a str,
    target: &'a str,
    employee: Option<&'a str>,
    limit: u32,
) -> ArchiveReadSpec<'a> {
    ArchiveReadSpec {
        projection_id,
        caller_project_id: caller,
        target_project_id: target,
        employee_id: employee,
        limit,
        resume_from: None,
        include_bodies: false,
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
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("read");
    assert!(after_chatter.records.is_empty());
    let deliverable = conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "deliverable", "openable note", 51),
        )
        .expect("deliverable");
    assert!(deliverable.delivered);
    assert!(deliverable.record_id.is_some());
    let rows = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            Some(&ids[1]),
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("scoped");
    assert_eq!(rows.records.len(), 1);
    assert_eq!(rows.records[0].kind, "deliverable");
    assert_eq!(
        rows.records[0].projection_id,
        CONVERSATION_ARCHIVE_PROJECTION_ID
    );
    let fetched = conversations
        .read_record(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &rows.records[0].record_id,
        )
        .expect("record");
    assert_eq!(fetched.body_redacted, "openable note");
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
            .read_index(&index_spec(
                legacy,
                &project_id,
                &project_id,
                None,
                CONVERSATION_RESUME_LIMIT,
            ))
            .expect_err("N1 read");
        assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    }
    let rows = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("empty");
    assert!(rows.records.is_empty());
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
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("empty");
    assert!(rows.records.is_empty());
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
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_b,
            &project_a,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect_err("N3 caller/target");
    assert!(matches!(
        cross_project,
        ProjectAggregateError::Forbidden { .. }
    ));
    let cross_employee = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_b,
            &project_b,
            Some(&ids_a[1]),
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect_err("N3 employee");
    assert!(matches!(
        cross_employee,
        ProjectAggregateError::Forbidden { .. }
    ));
    let b_rows = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_b,
            &project_b,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("b empty");
    assert!(b_rows.records.is_empty());
    let a_rows = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_a,
            &project_a,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("a");
    assert_eq!(a_rows.records.len(), 1);
    let fetched = conversations
        .read_record(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_a,
            &a_rows.records[0].record_id,
        )
        .expect("record");
    assert_eq!(fetched.body_redacted, "only in a");
}

#[test]
fn p11_t05_unbounded_resume_rejected() {
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
    conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "deliverable", "first", 90),
        )
        .expect("first");
    conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "deliverable", "second", 91),
        )
        .expect("second");
    conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "deliverable", "third", 92),
        )
        .expect("third");
    for bad_limit in [0_u32, CONVERSATION_RESUME_LIMIT + 1] {
        let error = conversations
            .read_index(&index_spec(
                CONVERSATION_ARCHIVE_PROJECTION_ID,
                &project_id,
                &project_id,
                None,
                bad_limit,
            ))
            .expect_err("N4");
        assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
        assert!(format!("{error}").contains("unbounded conversation resume"));
    }
    let first = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
            1,
        ))
        .expect("page1");
    assert_eq!(first.records.len(), 1);
    assert!(first.truncated);
    let cursor = first.next_cursor.expect("cursor");
    let mut page2 = index_spec(
        CONVERSATION_ARCHIVE_PROJECTION_ID,
        &project_id,
        &project_id,
        None,
        1,
    );
    page2.resume_from = Some(&cursor);
    let second = conversations.read_index(&page2).expect("page2");
    assert_eq!(second.records.len(), 1);
    assert_ne!(second.records[0].record_id, first.records[0].record_id);
}

#[test]
fn p11_t05_index_does_not_embed_bodies() {
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
    let record_id = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: &ids[1],
                kind: "note",
                body: "owner note body",
                now_ms: 100,
            },
        )
        .expect("append");
    let chatter = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: &ids[1],
                kind: "chatter",
                body: "must not land",
                now_ms: 101,
            },
        )
        .expect_err("chatter");
    assert!(matches!(chatter, ProjectAggregateError::Invalid { .. }));
    let oversize = "x".repeat(CONVERSATION_BODY_LIMIT + 1);
    let dumped = conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: &ids[1],
                kind: "note",
                body: &oversize,
                now_ms: 102,
            },
        )
        .expect_err("N5 oversize");
    assert!(matches!(dumped, ProjectAggregateError::Invalid { .. }));
    assert!(format!("{dumped}").contains("full-archive injection"));
    let mut inject = index_spec(
        CONVERSATION_ARCHIVE_PROJECTION_ID,
        &project_id,
        &project_id,
        None,
        CONVERSATION_RESUME_LIMIT,
    );
    inject.include_bodies = true;
    let error = conversations.read_index(&inject).expect_err("N5 bodies");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    let page = conversations
        .read_index(&index_spec(
            CONVERSATION_ARCHIVE_PROJECTION_ID,
            &project_id,
            &project_id,
            None,
            CONVERSATION_RESUME_LIMIT,
        ))
        .expect("index");
    assert_eq!(page.records.len(), 1);
    assert_eq!(page.records[0].record_id, record_id);
    assert_eq!(page.records[0].body_digest.len(), 64);
    let encoded = format!("{:?}", page.records[0]);
    assert!(!encoded.contains("owner note body"));
    let one = conversations
        .read_record(CONVERSATION_ARCHIVE_PROJECTION_ID, &project_id, &record_id)
        .expect("one");
    assert_eq!(one.body_redacted, "owner note body");
}

#[test]
fn p11_t05_archive_is_not_completion() {
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
    let landed = conversations
        .land_speech(
            &employees,
            &spec(&project_id, &ids[1], "deliverable", "not a pass", 110),
        )
        .expect("land");
    let record_id = landed.record_id.expect("id");
    let before = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    let ring = projects.get_stage(&plan_id, "s1").expect("s").expect("row");
    projects
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            "s1",
            &ring.stage_digest,
        )
        .expect("confirm");
    let error = projects
        .derive_stage_test_passed(&StageTestOracle {
            project_id: project_id.clone(),
            plan_revision_id: plan_id,
            stage_id: "s1".to_owned(),
            task_ref: "task://personal/p11-t05-n6".to_owned(),
            seating: SeatingFacts { seated: true },
            verification_current: true,
            verification_report_ref: record_id,
            openable: true,
            checks_passed: true,
            effects_closed: true,
            now_ms: 111,
        })
        .expect_err("N6");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    assert!(format!("{error}").contains("observation-only"));
    let after = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_eq!(after.state, before.state);
    assert_eq!(after.accepted_at, before.accepted_at);
}
