#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T04 Employee / Blueprint / Assignment: 22 §3 negatives + card extras.

use cognitive_store::{
    ConfirmCaller, EmployeeStore, HandoffSpec, PersonalDataLayout, ProjectAggregateError,
    ProjectAggregateStore, RosterProposal, StageSpec, prepare_personal_databases,
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

fn seat_first(
    employees: &EmployeeStore,
    ids: &[String],
    model: &str,
) -> Result<String, ProjectAggregateError> {
    employees.request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)?;
    employees.confirm_seating(
        ConfirmCaller::OwnerManagement,
        &ids[0],
        Some(model),
        true,
        31,
    )
}

#[test]
fn p11_t04_employee_survives_process_death() {
    let (_tmp, projects, employees) = stores();
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
    seat_first(&employees, &ids, "flash").expect("seat");
    employees
        .bind_runtime(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            "adapter:dsh-akp",
            32,
        )
        .expect("bind");
    let before = employees.get_employee(&ids[0]).expect("get").expect("row");
    employees
        .observe_attempt_process_exit(&ids[0])
        .expect("process death");
    let after = employees.get_employee(&ids[0]).expect("get").expect("row");
    assert_eq!(before, after);
    assert_eq!(after.state, "seated");
    assert_eq!(
        after.runtime_binding_ref.as_deref(),
        Some("adapter:dsh-akp")
    );
    let columns = employees.employee_column_names().expect("cols");
    assert!(!columns.iter().any(|name| name.contains("process")));
}

#[test]
fn p11_t04_recipe_mention_grants_nothing() {
    let (_tmp, projects, employees) = stores();
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
    let declared = employees.recipe_declared_tools(&ids[1]).expect("declared");
    assert!(declared.contains(&"workspace-write".to_owned()));
    let catalog = employees
        .tool_catalog(&project_id, &ids[1])
        .expect("catalog");
    assert!(catalog.is_empty());
    let error = employees
        .invoke_tool(&project_id, &ids[1], "workspace-write")
        .expect_err("N2");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p11_t04_roster_must_cover_all_slots() {
    let (_tmp, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    let missing = [proposals()[0].clone()];
    let error = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &missing,
            21,
        )
        .expect_err("missing slot");
    assert!(format!("{error}").contains("missing slot"));
    let surplus = [
        proposals()[0].clone(),
        proposals()[1].clone(),
        RosterProposal {
            slot: "invented".to_owned(),
            specialization: "member".to_owned(),
            prompt: "extra".to_owned(),
            tools_declared: vec![],
        },
    ];
    let error = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &surplus,
            22,
        )
        .expect_err("surplus");
    assert!(format!("{error}").contains("surplus"));
}

#[test]
fn p11_t04_sequential_seating_enforced() {
    let (_tmp, projects, employees) = stores();
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
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
        .expect("first seating");
    let error = employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[1], 31)
        .expect_err("N4");
    assert!(matches!(error, ProjectAggregateError::Conflict { .. }));
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            32,
        )
        .expect("seat first");
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[1], 33)
        .expect("second seating after first seated");
}

#[test]
fn p11_t04_progress_reads_committed_facts_only() {
    let (_tmp, projects, employees) = stores();
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
    let before = employees.seating_progress(&project_id).expect("progress");
    assert_eq!(before.seated, 0);
    assert_eq!(before.roster, 2);
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
        .expect("seating");
    employees
        .observe_attempt_process_exit(&ids[0])
        .expect("kill assistant");
    let mid = employees.seating_progress(&project_id).expect("mid");
    assert_eq!(mid.seated, 0);
    assert_eq!(mid.roster, 2);
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            31,
        )
        .expect("seat");
    let after = employees.seating_progress(&project_id).expect("after");
    assert_eq!(after.seated, 1);
    assert_eq!(after.roster, 2);
}

#[test]
fn p11_t04_blueprint_upgrade_is_opt_in() {
    let (_tmp, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    employees.ensure_builtins(5).expect("builtins");
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &proposals(),
            21,
        )
        .expect("roster");
    let before = employees
        .get_employee(&ids[0])
        .expect("get")
        .expect("row")
        .blueprint_revision_id;
    let new_revision = employees
        .publish_blueprint_revision(
            ConfirmCaller::OwnerManagement,
            cognitive_store::PROJECT_MANAGER_BLUEPRINT_ID,
            br#"{"duty":"Project Manager","prompt":"v2","tools":[]}"#,
            40,
        )
        .expect("publish");
    let after_publish = employees
        .get_employee(&ids[0])
        .expect("get")
        .expect("row")
        .blueprint_revision_id;
    assert_eq!(before, after_publish);
    let error = employees
        .upgrade_employee_blueprint(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            &new_revision,
            false,
            41,
        )
        .expect_err("implicit");
    assert!(format!("{error}").contains("opt-in"));
    employees
        .upgrade_employee_blueprint(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            &new_revision,
            true,
            42,
        )
        .expect("opt-in");
    let upgraded = employees
        .get_employee(&ids[0])
        .expect("get")
        .expect("row")
        .blueprint_revision_id;
    assert_eq!(upgraded, new_revision);
}

#[test]
fn p11_t04_employee_not_shared_across_projects() {
    let (_tmp, projects, employees) = stores();
    let project_a = activate(&projects);
    let project_b = activate(&projects);
    let plan_a = plan_two_slots(&projects, &project_a);
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_a,
            &plan_a,
            &proposals(),
            21,
        )
        .expect("roster");
    let error = employees
        .reuse_employee_in_project(&ids[0], &project_b)
        .expect_err("N7");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p11_t04_one_current_manager() {
    let (_tmp, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = projects
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[
                stage("s1", "Lead", "lead"),
                stage("s2", "Also lead", "colead"),
            ],
            20,
        )
        .expect("plan");
    let dual_managers = [
        RosterProposal {
            slot: "lead".to_owned(),
            specialization: "project-manager".to_owned(),
            prompt: "lead".to_owned(),
            tools_declared: vec![],
        },
        RosterProposal {
            slot: "colead".to_owned(),
            specialization: "project-manager".to_owned(),
            prompt: "colead".to_owned(),
            tools_declared: vec![],
        },
    ];
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &dual_managers,
            21,
        )
        .expect("roster");
    seat_first(&employees, &ids, "flash").expect("first manager");
    assert_eq!(employees.current_manager_count(&project_id).expect("n"), 1);
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[1], 40)
        .expect("second seating");
    let error = employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[1],
            Some("flash"),
            true,
            41,
        )
        .expect_err("N8");
    assert!(format!("{error}").contains("current manager"));
}

#[test]
fn p11_t04_speech_whitelist_enforced() {
    let (_tmp, projects, employees) = stores();
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
    seat_first(&employees, &ids, "flash").expect("manager seated");
    let filtered = employees
        .route_speech(&project_id, &ids[1], "chatter", false, 50)
        .expect("filter");
    assert!(!filtered.delivered);
    assert_eq!(filtered.reason, "speech-filtered");
    let deliverable = employees
        .route_speech(&project_id, &ids[1], "deliverable", false, 51)
        .expect("deliverable");
    assert!(deliverable.delivered);
    let manager = employees
        .route_speech(&project_id, &ids[0], "reply", false, 52)
        .expect("manager");
    assert!(manager.delivered);
}

#[test]
fn p11_t04_mcp_grant_is_per_scope() {
    let (_tmp, projects, employees) = stores();
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
    employees
        .record_install_fact("mcp:search", "1.0.0", 23)
        .expect("install");
    employees
        .grant_capability(
            ConfirmCaller::OwnerManagement,
            &project_a,
            &ids_a[1],
            "mcp:search",
            "project-a",
            24,
        )
        .expect("grant a");
    let catalog_a = employees
        .tool_catalog(&project_a, &ids_a[1])
        .expect("cat a");
    let catalog_b = employees
        .tool_catalog(&project_b, &ids_b[1])
        .expect("cat b");
    assert_eq!(catalog_a, vec!["mcp:search".to_owned()]);
    assert!(catalog_b.is_empty());
    employees
        .invoke_tool(&project_b, &ids_b[1], "mcp:search")
        .expect_err("B has no grant");
}

#[test]
fn p11_t04_chat_cannot_transfer_authority() {
    let (_tmp, projects, employees) = stores();
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
    let before = employees.grant_count(&ids[1]).expect("before");
    let error = employees
        .apply_chat_authority_transfer("transfer authority to member X")
        .expect_err("N11");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    assert_eq!(employees.grant_count(&ids[1]).expect("after"), before);
    let columns = employees.handoff_column_names().expect("cols");
    assert!(!columns.iter().any(|name| name.contains("transfer")));
    assert!(columns.iter().any(|name| name == "authority_stays"));
    employees
        .record_handoff(
            ConfirmCaller::OwnerManagement,
            &HandoffSpec {
                project_id: &project_id,
                source_employee_id: &ids[0],
                target_employee_id: &ids[1],
                bounded_work_digest: &ProjectAggregateStore::digest_hex(b"bounded-work"),
                blocked_or_ready: "ready",
                now_ms: 60,
            },
        )
        .expect("handoff");
}

#[test]
fn p11_t04_role_is_not_agent() {
    let (_tmp, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    let mut bad = proposals();
    bad[1].specialization = "agent".to_owned();
    let error = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &bad,
            21,
        )
        .expect_err("role=agent");
    assert!(format!("{error}").contains("Agent"));
}

#[test]
fn p11_t04_blueprint_has_no_provider_binding() {
    let (_tmp, _projects, employees) = stores();
    employees.ensure_builtins(1).expect("builtins");
    let columns = employees.blueprint_column_names().expect("cols");
    assert!(!columns.iter().any(|name| name.contains("provider")));
    let error = employees
        .publish_blueprint_revision(
            ConfirmCaller::OwnerManagement,
            cognitive_store::MEMBER_BLUEPRINT_ID,
            br#"{"duty":"Member","provider_binding":"flash"}"#,
            2,
        )
        .expect_err("provider on blueprint");
    assert!(format!("{error}").contains("Provider"));
}

#[test]
fn p11_t04_removed_manager_keeps_history() {
    let (_tmp, projects, employees) = stores();
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
    seat_first(&employees, &ids, "flash").expect("seat");
    employees
        .remove_employee(ConfirmCaller::OwnerManagement, &ids[0], 70)
        .expect("remove");
    let row = employees.get_employee(&ids[0]).expect("get").expect("row");
    assert_eq!(row.state, "removed");
    assert!(!row.is_current_manager);
    assert_eq!(employees.current_manager_count(&project_id).expect("n"), 0);
}

#[test]
fn p11_t04_seated_facts_unblock_stage_predicate() {
    let (_tmp, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    assert!(
        !employees
            .stage_is_seated(&project_id, &plan_id, "s1")
            .expect("empty")
    );
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &proposals(),
            21,
        )
        .expect("roster");
    seat_first(&employees, &ids, "flash").expect("seat manager");
    assert!(
        employees
            .stage_is_seated(&project_id, &plan_id, "s1")
            .expect("manager seated")
    );
    assert!(
        !employees
            .stage_is_seated(&project_id, &plan_id, "s2")
            .expect("researcher still unseated")
    );
}

#[test]
fn p11_t04_task_channel_cannot_register_roster() {
    let (_tmp, projects, employees) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    let error = employees
        .register_roster(
            ConfirmCaller::TaskChannel,
            &project_id,
            &plan_id,
            &proposals(),
            21,
        )
        .expect_err("N12 discipline");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}
