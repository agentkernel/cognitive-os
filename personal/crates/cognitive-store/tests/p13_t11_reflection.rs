#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T11 reflection candidates from Attempt / verification / evidence facts
//! and versioned Member Runtime improvement. Failure-first: model self-report
//! is not an improvement; implicit Blueprint upgrade is refused; running
//! Attempt prompt injection is refused; reflection is never completion;
//! Members are not silently reused across Projects.

use cognitive_store::personal_db::reflection::{
    MEMBER_RUNTIME_SUBJECT_KIND, ROLE_TEMPLATE_SUBJECT_KIND, ReflectionStore,
    RuntimeImprovementSpec,
};
use cognitive_store::{
    ConfirmCaller, EmployeeStore, HOSTED_DSH_ARTIFACT_DIGEST, HostedArtifactObservation,
    HostedAttemptFrameSpec, HostedAttemptIntentSpec, HostedAttemptTerminalSpec,
    HostedDshAttemptStore, HostedDshPlane, PersonalDataLayout, ProjectAggregateError,
    ProjectAggregateStore, RosterProposal, StageSpec, prepare_personal_databases,
};
use rusqlite::Connection;
use tempfile::TempDir;

#[allow(dead_code)]
struct Fixture {
    _tmp: TempDir,
    projects: ProjectAggregateStore,
    employees: EmployeeStore,
    attempts: HostedDshAttemptStore,
    reflections: ReflectionStore,
    path: std::path::PathBuf,
    project_id: String,
    manager_id: String,
    researcher_id: String,
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

fn fixture() -> Fixture {
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
    let attempts = HostedDshAttemptStore::open_path(&path).expect("attempts");
    let reflections = ReflectionStore::open_path(&path).expect("reflections");

    let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
    projects
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = projects
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    let project_id = projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("G1")
        .new_ref;
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
    for stage_id in ["s1", "s2"] {
        let row = projects
            .get_stage(&plan_id, stage_id)
            .expect("stage")
            .expect("row");
        projects
            .confirm_stage(
                ConfirmCaller::OwnerManagement,
                &project_id,
                &plan_id,
                stage_id,
                &row.stage_digest,
            )
            .expect("confirm stage");
    }
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
                    tools_declared: vec!["workspace-write".to_owned()],
                },
                RosterProposal {
                    slot: "researcher".to_owned(),
                    specialization: "member".to_owned(),
                    prompt: "research".to_owned(),
                    tools_declared: vec!["workspace-write".to_owned()],
                },
            ],
            21,
        )
        .expect("roster");
    for id in &ids {
        employees
            .request_seating(ConfirmCaller::OwnerManagement, id, 30)
            .expect("seating");
        employees
            .confirm_seating(ConfirmCaller::OwnerManagement, id, Some("flash"), true, 31)
            .expect("seat");
    }
    attempts
        .record_artifact_observation(
            ConfirmCaller::OwnerManagement,
            &HostedArtifactObservation {
                configured_revision: Some(HOSTED_DSH_ARTIFACT_DIGEST.to_owned()),
                pin_file_revision: Some(HOSTED_DSH_ARTIFACT_DIGEST.to_owned()),
                health: "pinned".to_owned(),
                child_script_digest: Some("a".repeat(64)),
                detail: "config, pin file and child script agree".to_owned(),
            },
            1,
        )
        .expect("pinned");
    Fixture {
        _tmp: temporary,
        projects,
        employees,
        attempts,
        reflections,
        path,
        project_id,
        manager_id: ids[0].clone(),
        researcher_id: ids[1].clone(),
    }
}

fn failed_terminal(fixture: &Fixture, employee_id: &str) -> Option<String> {
    let revision = fixture
        .employees
        .latest_revision_id(employee_id)
        .expect("rev")
        .expect("id");
    let outcome = fixture.attempts.persist_intent(
        ConfirmCaller::OwnerManagement,
        &HostedAttemptIntentSpec {
            employee_id,
            employee_revision_id: &revision,
            task_ref: "task://personal/p13-t11",
            bounded_context: "write the weekly report",
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
            now_ms: 40,
        },
    );
    if HostedDshPlane::isolated_spawn_is_fenced() {
        outcome.expect_err("gnu fence");
        return None;
    }
    let attempt_id = outcome.expect("persist").attempt_id;
    fixture
        .attempts
        .mark_dispatched(&attempt_id, Some("dshchild-p13-t11"), 4242, 50)
        .expect("dispatched");
    fixture
        .attempts
        .record_frames(
            &attempt_id,
            &[HostedAttemptFrameSpec {
                seq: 1,
                kind: "response".to_owned(),
                operation: None,
                payload_digest: None,
                reject_reason: None,
                text_redacted: "failed".to_owned(),
            }],
            55,
        )
        .expect("frames");
    fixture
        .attempts
        .record_terminal(
            &attempt_id,
            &HostedAttemptTerminalSpec {
                terminal_kind: "exited",
                exit_code: Some(6),
                response_status: Some("failed"),
                candidate_count: 0,
                observation_count: 0,
                rejected_frame_count: 0,
                unknown_line_count: 0,
                stdout_bytes: 64,
                stdout_truncated: false,
                stderr_tail_redacted: "",
                elapsed_ms: 12,
                now_ms: 60,
            },
        )
        .expect("terminal");
    Some(attempt_id)
}

#[test]
fn model_self_report_is_not_an_improvement() {
    let fixture = fixture();
    let error = fixture
        .reflections
        .admit_model_self_report(&fixture.project_id, "I improved my own prompt")
        .expect_err("self-report");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("self-report")),
        "{error}"
    );
    assert!(
        fixture
            .reflections
            .list_candidates(&fixture.project_id)
            .expect("list")
            .is_empty()
    );
}

#[test]
fn implicit_blueprint_upgrade_is_refused() {
    let fixture = fixture();
    let Some(_) = failed_terminal(&fixture, &fixture.researcher_id) else {
        return;
    };
    let generated = fixture
        .reflections
        .generate_from_facts(&fixture.project_id, 80)
        .expect("generate");
    assert!(
        !generated.is_empty(),
        "failed terminal must yield an incident"
    );
    let error = fixture
        .reflections
        .propose_runtime_improvement(
            ConfirmCaller::OwnerManagement,
            &RuntimeImprovementSpec {
                candidate_id: &generated[0].candidate_id,
                proposed_prompt: "tighten research",
                proposed_tools: &["workspace-write".to_owned()],
                new_blueprint_revision_id: Some("role-blueprint-rev:forged"),
                now_ms: 90,
            },
        )
        .expect_err("blueprint");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("Blueprint")),
        "{error}"
    );
}

#[test]
fn running_attempt_prompt_injection_is_refused() {
    let fixture = fixture();
    let revision = fixture
        .employees
        .latest_revision_id(&fixture.researcher_id)
        .expect("rev")
        .expect("id");
    let outcome = fixture.attempts.persist_intent(
        ConfirmCaller::OwnerManagement,
        &HostedAttemptIntentSpec {
            employee_id: &fixture.researcher_id,
            employee_revision_id: &revision,
            task_ref: "task://personal/p13-t11-running",
            bounded_context: "original bounded context",
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
            now_ms: 40,
        },
    );
    if HostedDshPlane::isolated_spawn_is_fenced() {
        outcome.expect_err("gnu fence");
        return;
    }
    let attempt_id = outcome.expect("persist").attempt_id;
    fixture
        .attempts
        .mark_dispatched(&attempt_id, Some("dshchild-p13-t11-run"), 7, 50)
        .expect("dispatched");
    let error = fixture
        .reflections
        .overwrite_running_attempt_context(&attempt_id, "injected prompt")
        .expect_err("inject");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("injection")),
        "{error}"
    );
    let generated = fixture
        .reflections
        .generate_from_facts(&fixture.project_id, 80)
        .expect("generate");
    if let Some(candidate) = generated.first() {
        let propose = fixture.reflections.propose_runtime_improvement(
            ConfirmCaller::OwnerManagement,
            &RuntimeImprovementSpec {
                candidate_id: &candidate.candidate_id,
                proposed_prompt: "should not land while running",
                proposed_tools: &[],
                new_blueprint_revision_id: None,
                now_ms: 90,
            },
        );
        assert!(
            matches!(
                propose,
                Err(ProjectAggregateError::Rejected { detail })
                    if detail.contains("injection")
            ),
            "{propose:?}"
        );
    }
}

#[test]
fn reflection_is_never_completion() {
    let fixture = fixture();
    let error = fixture
        .reflections
        .claim_reflection_is_completion("reflect-none")
        .expect_err("completion");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("never completion")),
        "{error}"
    );
}

#[test]
fn member_is_not_reused_across_projects() {
    let fixture = fixture();
    let error = fixture
        .reflections
        .reuse_member_in_other_project(&fixture.researcher_id, "project-other")
        .expect_err("reuse");
    assert!(
        matches!(error, ProjectAggregateError::Forbidden { .. }),
        "{error}"
    );
}

#[test]
fn generate_from_failed_terminal_then_confirm_and_rollback_revision() {
    let fixture = fixture();
    let Some(attempt_id) = failed_terminal(&fixture, &fixture.researcher_id) else {
        return;
    };
    let first = fixture
        .reflections
        .generate_from_facts(&fixture.project_id, 80)
        .expect("generate");
    let incident = first
        .iter()
        .find(|row| row.kind == "incident")
        .expect("failed terminal must yield an incident");
    assert_eq!(incident.source, "attempt-terminal");
    assert_eq!(incident.attempt_id.as_deref(), Some(attempt_id.as_str()));
    assert!(!incident.completion_claimed);
    let daily = first
        .iter()
        .find(|row| row.kind == "daily")
        .expect("a terminal day must also yield a daily rollup");
    assert_eq!(daily.source, "attempt-terminal");
    assert_ne!(daily.kind, "key-result");
    assert_eq!(first.len(), 2);
    let again = fixture
        .reflections
        .generate_from_facts(&fixture.project_id, 81)
        .expect("idempotent");
    assert!(again.is_empty(), "same fact_digest must not duplicate");

    let base = fixture
        .employees
        .latest_revision_id(&fixture.researcher_id)
        .expect("base")
        .expect("id");
    let proposed = fixture
        .reflections
        .propose_runtime_improvement(
            ConfirmCaller::OwnerManagement,
            &RuntimeImprovementSpec {
                candidate_id: &incident.candidate_id,
                proposed_prompt: "cite sources before drafting",
                proposed_tools: &["workspace-write".to_owned()],
                new_blueprint_revision_id: None,
                now_ms: 90,
            },
        )
        .expect("propose");
    assert_eq!(proposed.state, "preview");
    assert_eq!(proposed.base_revision_id, base);
    assert!(proposed.applied_revision_id.is_none());
    assert_eq!(
        fixture
            .employees
            .latest_revision_id(&fixture.researcher_id)
            .expect("still base")
            .as_deref(),
        Some(base.as_str()),
        "preview must not change the current Employee revision"
    );

    let task_error = fixture
        .reflections
        .confirm_runtime_improvement(
            ConfirmCaller::TaskChannel,
            &proposed.preview_id,
            &proposed.preview_digest,
            100,
        )
        .expect_err("task channel");
    assert!(matches!(
        task_error,
        ProjectAggregateError::Forbidden { .. }
    ));

    let confirmed = fixture
        .reflections
        .confirm_runtime_improvement(
            ConfirmCaller::OwnerManagement,
            &proposed.preview_id,
            &proposed.preview_digest,
            100,
        )
        .expect("confirm");
    assert_eq!(confirmed.state, "active");
    let after_confirm = fixture
        .employees
        .latest_revision_id(&fixture.researcher_id)
        .expect("after")
        .expect("id");
    assert_ne!(after_confirm, base);
    assert_eq!(
        confirmed.applied_revision_id.as_deref(),
        Some(after_confirm.as_str())
    );

    let rolled = fixture
        .reflections
        .rollback_runtime_improvement(
            ConfirmCaller::OwnerManagement,
            &confirmed.improvement_id,
            110,
        )
        .expect("rollback");
    assert_eq!(rolled.state, "rolled-back");
    let after_rollback = fixture
        .employees
        .latest_revision_id(&fixture.researcher_id)
        .expect("rollback latest")
        .expect("id");
    assert_ne!(after_rollback, after_confirm);
    assert_ne!(after_rollback, base);
}

#[test]
fn role_template_proposal_needs_owner_and_does_not_copy_employee() {
    let fixture = fixture();
    let assistant = fixture
        .reflections
        .propose_role_template(ConfirmCaller::Assistant, &fixture.researcher_id, 70)
        .expect_err("assistant");
    assert!(matches!(assistant, ProjectAggregateError::Forbidden { .. }));
    let (proposal_id, preview_id) = fixture
        .reflections
        .propose_role_template(ConfirmCaller::OwnerManagement, &fixture.researcher_id, 70)
        .expect("propose template");
    assert!(preview_id.starts_with("preview-"));
    fixture
        .reflections
        .confirm_role_template(ConfirmCaller::OwnerManagement, &proposal_id, 80)
        .expect("confirm template");
    let other = fixture
        .reflections
        .reuse_member_in_other_project(&fixture.researcher_id, "project-copied")
        .expect_err("still not copied");
    assert!(matches!(other, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn v40_tables_and_preview_kinds_exist() {
    let fixture = fixture();
    let conn = Connection::open(&fixture.path).expect("open");
    for table in [
        "p13_reflection_candidate",
        "p13_runtime_improvement",
        "p13_role_template_proposal",
    ] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .expect("table");
        assert_eq!(exists, 1, "{table}");
    }
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'p11_approval_preview'",
            [],
            |row| row.get(0),
        )
        .expect("preview sql");
    assert!(sql.contains(MEMBER_RUNTIME_SUBJECT_KIND) || sql.contains("member"));
    assert!(sql.contains(ROLE_TEMPLATE_SUBJECT_KIND) || sql.contains("template"));
    assert!(!sql.contains("sk-"));
}

#[test]
fn successful_looking_terminal_without_evidence_is_not_a_key_result() {
    let fixture = fixture();
    let revision = fixture
        .employees
        .latest_revision_id(&fixture.manager_id)
        .expect("rev")
        .expect("id");
    let outcome = fixture.attempts.persist_intent(
        ConfirmCaller::OwnerManagement,
        &HostedAttemptIntentSpec {
            employee_id: &fixture.manager_id,
            employee_revision_id: &revision,
            task_ref: "task://personal/p13-t11-done-shape",
            bounded_context: "child claimed done",
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
            now_ms: 40,
        },
    );
    if HostedDshPlane::isolated_spawn_is_fenced() {
        outcome.expect_err("gnu fence");
        return;
    }
    let attempt_id = outcome.expect("persist").attempt_id;
    fixture
        .attempts
        .mark_dispatched(&attempt_id, Some("dshchild-p13-t11-done"), 3, 50)
        .expect("dispatched");
    fixture
        .attempts
        .record_terminal(
            &attempt_id,
            &HostedAttemptTerminalSpec {
                terminal_kind: "exited",
                exit_code: Some(0),
                response_status: Some("done"),
                candidate_count: 1,
                observation_count: 0,
                rejected_frame_count: 0,
                unknown_line_count: 0,
                stdout_bytes: 16,
                stdout_truncated: false,
                stderr_tail_redacted: "",
                elapsed_ms: 4,
                now_ms: 60,
            },
        )
        .expect("terminal");
    let generated = fixture
        .reflections
        .generate_from_facts(&fixture.project_id, 80)
        .expect("generate");
    assert!(
        generated.iter().all(|row| row.kind != "key-result"),
        "exit 0 / response done without evidence must not become a key-result: {generated:?}"
    );
    assert!(
        generated.iter().any(|row| row.kind == "daily"),
        "a terminal calendar day must yield a daily candidate: {generated:?}"
    );
}

#[test]
fn empty_project_day_is_not_a_daily_candidate() {
    let fixture = fixture();
    let generated = fixture
        .reflections
        .generate_from_facts(&fixture.project_id, 80)
        .expect("generate");
    assert!(
        generated.iter().all(|row| row.kind != "daily"),
        "no Attempt terminal means no daily rollup: {generated:?}"
    );
}
