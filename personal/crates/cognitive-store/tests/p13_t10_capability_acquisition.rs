#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T10/D01 failure-first: install ≠ grant; unreviewed install, chat
//! Approve, and ambient grant are refused. No second grant table.

use cognitive_store::{
    ConfirmCaller, EmployeeStore, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    RosterProposal, SecurityReview, StageSpec, prepare_personal_databases,
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

fn seat_researcher(
    projects: &ProjectAggregateStore,
    employees: &EmployeeStore,
) -> (String, String) {
    let project_id = activate(projects);
    let plan_id = projects
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[
                stage("s1", "Manage", "manager"),
                stage("s2", "Research", "researcher"),
            ],
            20,
        )
        .expect("plan");
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &[
                RosterProposal {
                    slot: "manager".to_owned(),
                    specialization: "project-manager".to_owned(),
                    prompt: "coordinate".to_owned(),
                    tools_declared: vec![],
                },
                RosterProposal {
                    slot: "researcher".to_owned(),
                    specialization: "member".to_owned(),
                    prompt: "research".to_owned(),
                    tools_declared: vec!["mcp:search".to_owned()],
                },
            ],
            21,
        )
        .expect("roster");
    (project_id, ids[1].clone())
}

fn passed_mcp_review(capability_ref: &str, version_pin: &str) -> SecurityReview {
    SecurityReview {
        capability_ref: capability_ref.to_owned(),
        kind: "mcp".to_owned(),
        version_pin: version_pin.to_owned(),
        source: "https://example.invalid/mcp/search".to_owned(),
        license: "MIT".to_owned(),
        hidden_instruction: "none".to_owned(),
        prompt_injection: "none".to_owned(),
        file_intent: "none".to_owned(),
        network_intent: "declared".to_owned(),
        command_intent: "none".to_owned(),
        dependencies: Some("none".to_owned()),
        executable_code: Some("none".to_owned()),
        secret_access: Some("none".to_owned()),
        tool_permissions: Some("search".to_owned()),
        supply_chain: Some("pinned-origin".to_owned()),
        sources: vec!["https://example.invalid/mcp/search".to_owned()],
    }
}

#[test]
fn unreviewed_install_is_refused() {
    let (_tmp, projects, employees) = stores();
    let (_project_id, _employee_id) = seat_researcher(&projects, &employees);
    let mut review = passed_mcp_review("mcp:search", "1.0.0");
    review.hidden_instruction.clear();
    let error = employees
        .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &review, 30)
        .expect_err("unreviewed");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    assert!(employees.list_install_facts().expect("list").is_empty());
}

#[test]
fn injection_or_hidden_instruction_refuses_install() {
    let (_tmp, _projects, employees) = stores();
    let mut hidden = passed_mcp_review("mcp:search", "1.0.0");
    hidden.hidden_instruction = "found".to_owned();
    assert!(matches!(
        employees
            .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &hidden, 31)
            .expect_err("hidden"),
        ProjectAggregateError::Rejected { .. }
    ));
    let mut injected = passed_mcp_review("mcp:search", "1.0.0");
    injected.prompt_injection = "found".to_owned();
    assert!(matches!(
        employees
            .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &injected, 32)
            .expect_err("injection"),
        ProjectAggregateError::Rejected { .. }
    ));
}

#[test]
fn reviewed_install_is_not_a_grant() {
    let (_tmp, projects, employees) = stores();
    let (project_id, employee_id) = seat_researcher(&projects, &employees);
    let review = passed_mcp_review("mcp:search", "1.0.0");
    let install_id = employees
        .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &review, 40)
        .expect("install");
    assert!(install_id.starts_with("install-"));
    let catalog = employees
        .tool_catalog(&project_id, &employee_id)
        .expect("catalog");
    assert!(
        catalog.is_empty(),
        "InstallFact must not authorize: {catalog:?}"
    );
    employees
        .invoke_tool(&project_id, &employee_id, "mcp:search")
        .expect_err("recipe mention is not a grant");
}

#[test]
fn chat_and_task_cannot_install_or_approve() {
    let (_tmp, projects, employees) = stores();
    let (project_id, employee_id) = seat_researcher(&projects, &employees);
    let review = passed_mcp_review("mcp:search", "1.0.0");
    for caller in [ConfirmCaller::Assistant, ConfirmCaller::TaskChannel] {
        let install = employees
            .record_reviewed_install_fact(caller, &review, 50)
            .expect_err("chat install");
        assert!(matches!(install, ProjectAggregateError::Forbidden { .. }));
        let acquire = employees
            .admit_discovery(caller, &review)
            .expect_err("chat discover");
        assert!(matches!(acquire, ProjectAggregateError::Forbidden { .. }));
    }
    employees
        .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &review, 51)
        .expect("owner install");
    let ambient = employees
        .refuse_ambient_grant(&project_id, &employee_id, "mcp:search", "project")
        .expect_err("ambient");
    assert!(matches!(ambient, ProjectAggregateError::Forbidden { .. }));
    assert!(
        employees
            .tool_catalog(&project_id, &employee_id)
            .expect("still empty")
            .is_empty()
    );
}

#[test]
fn marketplace_engine_store_and_unpinned_sources_are_refused() {
    let (_tmp, _projects, employees) = stores();
    for (capability, source) in [
        ("marketplace:search", "https://example.invalid/ok"),
        ("mcp:search", "engine-store://search"),
        ("mcp:search", "ambient://local"),
    ] {
        let mut review = passed_mcp_review(capability, "1.0.0");
        review.source = source.to_owned();
        review.sources = vec![source.to_owned()];
        let error = employees
            .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &review, 60)
            .expect_err(capability);
        assert!(matches!(
            error,
            ProjectAggregateError::Rejected { .. } | ProjectAggregateError::Invalid { .. }
        ));
    }
}

#[test]
fn update_review_compat_and_rollback_do_not_silent_grant() {
    let (_tmp, projects, employees) = stores();
    let (project_id, employee_id) = seat_researcher(&projects, &employees);
    let review = passed_mcp_review("mcp:search", "1.0.0");
    employees
        .record_reviewed_install_fact(ConfirmCaller::OwnerManagement, &review, 70)
        .expect("v1");
    let compatible = employees
        .compat_test("mcp:search", "1.0.0", "1.0.1")
        .expect("compat");
    assert_eq!(compatible, "compatible");
    let incompatible = employees
        .compat_test("mcp:search", "1.0.0", "2.0.0")
        .expect("major");
    assert_eq!(incompatible, "incompatible");
    let mut next = passed_mcp_review("mcp:search", "1.0.1");
    next.license = "MIT".to_owned();
    employees
        .review_update(ConfirmCaller::OwnerManagement, &next, 71)
        .expect("update review");
    assert!(
        employees
            .tool_catalog(&project_id, &employee_id)
            .expect("no silent grant")
            .is_empty()
    );
    employees
        .rollback_install(ConfirmCaller::OwnerManagement, "mcp:search", "1.0.0", 72)
        .expect("rollback");
    let rolled = employees
        .record_reviewed_install_fact(
            ConfirmCaller::OwnerManagement,
            &passed_mcp_review("mcp:search", "1.0.0"),
            73,
        )
        .expect_err("rolled-back pin");
    assert!(matches!(rolled, ProjectAggregateError::Rejected { .. }));
}
