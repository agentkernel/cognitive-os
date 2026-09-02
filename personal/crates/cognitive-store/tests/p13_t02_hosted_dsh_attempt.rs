#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T02 hosted DSH Attempt ledger: persist-before-dispatch, process death ≠
//! completion, unknown output ≠ success, frames are observations that never
//! write authority, crash-shaped rows reconcile to unknown-outcome, artifact
//! health / update / rollback facts gate the spawn, task channel is forbidden.

use cognitive_store::{
    ConfirmCaller, EmployeeStore, HOSTED_ATTEMPT_CONTEXT_MAX_BYTES, HOSTED_DSH_ARTIFACT_DIGEST,
    HOSTED_DSH_WIN_GNU_FENCE, HostedArtifactObservation, HostedAttemptFrameSpec,
    HostedAttemptIntentSpec, HostedAttemptTerminalSpec, HostedDshAttemptStore, HostedDshPlane,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, RosterProposal, StageSpec,
    prepare_personal_databases,
};
use rusqlite::Connection;
use tempfile::TempDir;

struct Fixture {
    _tmp: TempDir,
    employees: EmployeeStore,
    attempts: HostedDshAttemptStore,
    path: std::path::PathBuf,
    project_id: String,
    seated_employee_id: String,
    proposed_employee_id: String,
    revision_id: String,
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
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
        .expect("seating");
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            31,
        )
        .expect("seat");
    let revision_id = employees
        .latest_revision_id(&ids[0])
        .expect("rev")
        .expect("id");
    Fixture {
        _tmp: temporary,
        employees,
        attempts,
        path,
        project_id,
        seated_employee_id: ids[0].clone(),
        proposed_employee_id: ids[1].clone(),
        revision_id,
    }
}

fn pinned_observation() -> HostedArtifactObservation {
    HostedArtifactObservation {
        configured_revision: Some(HOSTED_DSH_ARTIFACT_DIGEST.to_owned()),
        pin_file_revision: Some(HOSTED_DSH_ARTIFACT_DIGEST.to_owned()),
        health: "pinned".to_owned(),
        child_script_digest: Some("a".repeat(64)),
        detail: "config, pin file and child script agree".to_owned(),
    }
}

fn mismatch_observation() -> HostedArtifactObservation {
    HostedArtifactObservation {
        configured_revision: Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_owned()),
        pin_file_revision: Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_owned()),
        health: "mismatch".to_owned(),
        child_script_digest: Some("a".repeat(64)),
        detail: "configured revision is not the product pin".to_owned(),
    }
}

fn intent<'a>(fixture: &'a Fixture, context: &'a str) -> HostedAttemptIntentSpec<'a> {
    HostedAttemptIntentSpec {
        employee_id: &fixture.seated_employee_id,
        employee_revision_id: &fixture.revision_id,
        task_ref: "task://personal/p13-t02-attempt",
        bounded_context: context,
        artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
        now_ms: 40,
    }
}

fn terminal<'a>(
    kind: &'a str,
    exit_code: Option<i32>,
    response: Option<&'a str>,
) -> HostedAttemptTerminalSpec<'a> {
    HostedAttemptTerminalSpec {
        terminal_kind: kind,
        exit_code,
        response_status: response,
        candidate_count: 1,
        observation_count: 2,
        rejected_frame_count: 1,
        unknown_line_count: 3,
        stdout_bytes: 512,
        stdout_truncated: false,
        stderr_tail_redacted: "Bearer abc.def tail",
        elapsed_ms: 12,
        now_ms: 60,
    }
}

fn fenced_or_persist(fixture: &Fixture, context: &str) -> Option<String> {
    let outcome = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &intent(fixture, context));
    if HostedDshPlane::isolated_spawn_is_fenced() {
        let error = outcome.expect_err("gnu fence");
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return None;
    }
    Some(outcome.expect("persist").attempt_id)
}

#[test]
fn p13_t02_artifact_facts_record_health_update_and_rollback() {
    let fixture = fixture();
    let health = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("health");
    assert_eq!(health.kind, "health-check");
    assert_eq!(health.health, "pinned");
    assert!(health.admits_spawn());
    assert_eq!(health.expected_revision, HOSTED_DSH_ARTIFACT_DIGEST);
    assert!(health.previous_fact_id.is_none());

    let again = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 2)
        .expect("health again");
    assert_eq!(again.kind, "health-check");
    assert_eq!(
        again.previous_fact_id.as_deref(),
        Some(health.fact_id.as_str())
    );

    let update = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &mismatch_observation(), 3)
        .expect("update");
    assert_eq!(update.kind, "update");
    assert_eq!(update.health, "mismatch");
    assert!(!update.admits_spawn());
    assert_eq!(
        update.previous_fact_id.as_deref(),
        Some(again.fact_id.as_str())
    );

    let rollback = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 4)
        .expect("rollback");
    assert_eq!(rollback.kind, "rollback");
    assert!(rollback.admits_spawn());
    assert_eq!(
        rollback.previous_fact_id.as_deref(),
        Some(update.fact_id.as_str())
    );

    let history = fixture.attempts.list_artifact_facts(10).expect("history");
    assert_eq!(history.len(), 4);
    assert_eq!(history[0].fact_id, rollback.fact_id);
    assert_eq!(history[3].fact_id, health.fact_id);
    assert_eq!(
        fixture
            .attempts
            .latest_artifact_fact()
            .expect("latest")
            .expect("row")
            .fact_id,
        rollback.fact_id
    );

    let mut lying = mismatch_observation();
    lying.health = "pinned".to_owned();
    let error = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &lying, 5)
        .expect_err("pinned requires agreement");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    let mut no_script = pinned_observation();
    no_script.child_script_digest = None;
    let error = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &no_script, 6)
        .expect_err("pinned requires the child script");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    let error = fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::TaskChannel, &pinned_observation(), 7)
        .expect_err("task channel");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));

    let conn = Connection::open(&fixture.path).expect("open");
    let error = conn
        .execute(
            "UPDATE p13_hosted_dsh_artifact_fact SET health = 'pinned' WHERE fact_id = ?1",
            [&update.fact_id],
        )
        .expect_err("facts are append-only");
    assert!(format!("{error}").contains("append-only"));
}

#[test]
fn p13_t02_unhealthy_or_unknown_artifact_refuses_spawn() {
    let fixture = fixture();
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(fenced_or_persist(&fixture, "do the task").is_none());
        return;
    }
    let error = fixture
        .attempts
        .persist_intent(
            ConfirmCaller::OwnerManagement,
            &intent(&fixture, "do the task"),
        )
        .expect_err("unknown health");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("health is unknown"))
    );
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &mismatch_observation(), 1)
        .expect("mismatch");
    let error = fixture
        .attempts
        .persist_intent(
            ConfirmCaller::OwnerManagement,
            &intent(&fixture, "do the task"),
        )
        .expect_err("unhealthy");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("not pinned"))
    );
    assert_eq!(
        fixture
            .attempts
            .list_attempts(&fixture.project_id, 10)
            .expect("list")
            .len(),
        0
    );
}

#[test]
fn p13_t02_persist_before_dispatch_then_daemon_terminal_observation() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    let Some(attempt_id) = fenced_or_persist(&fixture, "summarize README.md") else {
        return;
    };
    let persisted = fixture
        .attempts
        .get_attempt(&attempt_id)
        .expect("get")
        .expect("row");
    assert_eq!(persisted.state, "persisted");
    assert!(persisted.intent_persisted);
    assert_eq!(persisted.terminal_kind, "pending");
    assert_eq!(persisted.response_status, "pending");
    assert!(!persisted.completion_claimed);
    assert_eq!(persisted.verification_status, "not-run");
    assert!(persisted.pid.is_none());
    assert_eq!(persisted.project_id, fixture.project_id);
    assert_eq!(
        persisted.context_digest,
        HostedDshAttemptStore::context_digest("summarize README.md")
    );
    assert_eq!(persisted.context_bytes, 19);

    fixture
        .attempts
        .bind_child_identity(&attempt_id, "dshchild-test")
        .expect("bind child");
    let dispatched = fixture
        .attempts
        .mark_dispatched(&attempt_id, None, 4242, 50)
        .expect("dispatched");
    assert_eq!(dispatched.state, "dispatched");
    assert_eq!(dispatched.pid, Some(4242));
    assert_eq!(dispatched.child_id.as_deref(), Some("dshchild-test"));
    assert_eq!(dispatched.dispatched_at, Some(50));
    let error = fixture
        .attempts
        .mark_dispatched(&attempt_id, None, 4243, 51)
        .expect_err("second dispatch");
    assert!(matches!(error, ProjectAggregateError::Conflict { .. }));

    let frames = vec![
        HostedAttemptFrameSpec {
            seq: 1,
            kind: "observation".to_owned(),
            operation: None,
            payload_digest: None,
            text_redacted: "child.started".to_owned(),
            reject_reason: None,
        },
        HostedAttemptFrameSpec {
            seq: 2,
            kind: "heartbeat".to_owned(),
            operation: None,
            payload_digest: None,
            text_redacted: "alive".to_owned(),
            reject_reason: None,
        },
        HostedAttemptFrameSpec {
            seq: 3,
            kind: "candidate".to_owned(),
            operation: Some("DeliverableDraft".to_owned()),
            payload_digest: Some("b".repeat(64)),
            text_redacted: "draft Bearer abc.def".to_owned(),
            reject_reason: None,
        },
        HostedAttemptFrameSpec {
            seq: 4,
            kind: "rejected".to_owned(),
            operation: None,
            payload_digest: None,
            text_redacted: "{\"frame\":\"provider_request\"}".to_owned(),
            reject_reason: Some("child-direct-provider".to_owned()),
        },
        HostedAttemptFrameSpec {
            seq: 5,
            kind: "response".to_owned(),
            operation: None,
            payload_digest: None,
            text_redacted: "done".to_owned(),
            reject_reason: None,
        },
    ];
    let written = fixture
        .attempts
        .record_frames(&attempt_id, &frames, 55)
        .expect("frames");
    assert_eq!(written, 5);
    let stored = fixture
        .attempts
        .list_frames(&attempt_id, 100)
        .expect("list frames");
    assert_eq!(stored.len(), 5);
    assert!(stored.iter().all(|frame| !frame.authority_written));
    assert_eq!(stored[2].kind, "candidate");
    assert_eq!(stored[2].operation.as_deref(), Some("DeliverableDraft"));
    assert!(stored[2].text_redacted.contains("Bearer [redacted]"));
    assert!(!stored[2].text_redacted.contains("abc.def"));
    assert_eq!(
        stored[3].reject_reason.as_deref(),
        Some("child-direct-provider")
    );

    let closed = fixture
        .attempts
        .record_terminal(&attempt_id, &terminal("exited", Some(0), Some("done")))
        .expect("terminal");
    assert_eq!(closed.state, "terminal");
    assert_eq!(closed.terminal_kind, "exited");
    assert_eq!(closed.exit_code, Some(0));
    assert_eq!(closed.response_status, "done");
    assert!(!closed.completion_claimed);
    assert_eq!(closed.verification_status, "not-run");
    assert!(closed.pid.is_none());
    assert_eq!(closed.candidate_count, 1);
    assert_eq!(closed.observation_count, 2);
    assert_eq!(closed.rejected_frame_count, 1);
    assert_eq!(closed.unknown_line_count, 3);
    assert_eq!(closed.terminal_at, Some(60));
    assert!(closed.stderr_tail_redacted.contains("Bearer [redacted]"));
    assert!(!closed.stderr_tail_redacted.contains("abc.def"));

    let error = fixture
        .attempts
        .record_terminal(&attempt_id, &terminal("exited", Some(1), None))
        .expect_err("double terminal");
    assert!(matches!(error, ProjectAggregateError::Conflict { .. }));
    let error = fixture
        .attempts
        .record_frames(&attempt_id, &frames[..1], 70)
        .expect_err("frames after terminal");
    assert!(matches!(error, ProjectAggregateError::Conflict { .. }));

    let history = fixture
        .attempts
        .list_attempts(&fixture.project_id, 10)
        .expect("runs read");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].attempt_id, attempt_id);
    assert_eq!(history[0].task_ref, "task://personal/p13-t02-attempt");
}

#[test]
fn p13_t02_process_death_is_never_completion() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    for (kind, code) in [
        ("exited", Some(0)),
        ("exited", Some(1)),
        ("signaled", None),
        ("timed-out", None),
        ("spawn-failed", None),
    ] {
        let Some(attempt_id) = fenced_or_persist(&fixture, "do the task") else {
            return;
        };
        let closed = fixture
            .attempts
            .record_terminal(&attempt_id, &terminal(kind, code, Some("done")))
            .expect("terminal");
        assert_eq!(closed.terminal_kind, kind);
        assert_ne!(closed.terminal_kind, "success");
        assert!(!closed.completion_claimed);
        assert_eq!(closed.verification_status, "not-run");
    }
    let Some(attempt_id) = fenced_or_persist(&fixture, "do the task") else {
        return;
    };
    let error = fixture
        .attempts
        .record_terminal(&attempt_id, &terminal("success", Some(0), Some("done")))
        .expect_err("success is not a terminal kind");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));

    let conn = Connection::open(&fixture.path).expect("open");
    for statement in [
        "UPDATE p13_hosted_dsh_attempt SET terminal_kind = 'success' WHERE attempt_id = ?1",
        "UPDATE p13_hosted_dsh_attempt SET completion_claimed = 1 WHERE attempt_id = ?1",
        "UPDATE p13_hosted_dsh_attempt SET verification_status = 'passed' WHERE attempt_id = ?1",
        "UPDATE p13_hosted_dsh_attempt SET state = 'completed' WHERE attempt_id = ?1",
    ] {
        let error = conn
            .execute(statement, [&attempt_id])
            .expect_err("schema forbids completion claims");
        assert!(format!("{error}").contains("CHECK"), "{statement}: {error}");
    }
    let error = conn
        .execute(
            "DELETE FROM p13_hosted_dsh_attempt WHERE attempt_id = ?1",
            [&attempt_id],
        )
        .expect_err("attempts are never deleted");
    assert!(format!("{error}").contains("append-only"));
}

#[test]
fn p13_t02_unknown_child_output_is_not_success() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    for response in [
        None,
        Some("success"),
        Some("ok"),
        Some("agent_end"),
        Some("complete"),
    ] {
        let Some(attempt_id) = fenced_or_persist(&fixture, "do the task") else {
            return;
        };
        let closed = fixture
            .attempts
            .record_terminal(&attempt_id, &terminal("exited", Some(0), response))
            .expect("terminal");
        assert_eq!(closed.response_status, "unknown", "{response:?}");
        assert!(!closed.completion_claimed);
    }
}

#[test]
fn p13_t02_frames_never_write_authority() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    let Some(attempt_id) = fenced_or_persist(&fixture, "do the task") else {
        return;
    };
    let before = fixture
        .employees
        .get_employee(&fixture.seated_employee_id)
        .expect("get")
        .expect("row");
    let conn = Connection::open(&fixture.path).expect("open");
    let project_before: String = conn
        .query_row(
            "SELECT state FROM p11_project WHERE project_id = ?1",
            [&fixture.project_id],
            |row| row.get(0),
        )
        .expect("project state");
    let frames: Vec<HostedAttemptFrameSpec> = (1..=40)
        .map(|seq| HostedAttemptFrameSpec {
            seq,
            kind: if seq % 2 == 0 {
                "heartbeat".to_owned()
            } else {
                "observation".to_owned()
            },
            operation: None,
            payload_digest: None,
            text_redacted: format!("beat {seq}"),
            reject_reason: None,
        })
        .collect();
    fixture
        .attempts
        .record_frames(&attempt_id, &frames, 55)
        .expect("frames");
    let after = fixture
        .employees
        .get_employee(&fixture.seated_employee_id)
        .expect("get")
        .expect("row");
    assert_eq!(before, after);
    let project_after: String = conn
        .query_row(
            "SELECT state FROM p11_project WHERE project_id = ?1",
            [&fixture.project_id],
            |row| row.get(0),
        )
        .expect("project state");
    assert_eq!(project_before, project_after);
    let stage_tests: i64 = conn
        .query_row("SELECT COUNT(*) FROM p11_stage_test_fact", [], |row| {
            row.get(0)
        })
        .expect("stage tests");
    assert_eq!(stage_tests, 0);
    let acceptance: i64 = conn
        .query_row("SELECT COUNT(*) FROM p11_acceptance_fact", [], |row| {
            row.get(0)
        })
        .expect("acceptance");
    assert_eq!(acceptance, 0);
    let attempt = fixture
        .attempts
        .get_attempt(&attempt_id)
        .expect("get")
        .expect("row");
    assert_eq!(attempt.state, "persisted");
    assert!(!attempt.completion_claimed);

    let stored = fixture
        .attempts
        .list_frames(&attempt_id, 512)
        .expect("frames");
    assert_eq!(stored.len(), 40);
    let error = conn
        .execute(
            "UPDATE p13_hosted_dsh_attempt_frame SET authority_written = 1 WHERE frame_id = ?1",
            [&stored[0].frame_id],
        )
        .expect_err("frames are append-only");
    assert!(format!("{error}").contains("append-only"));
    let error = fixture
        .attempts
        .record_frames(
            &attempt_id,
            &[HostedAttemptFrameSpec {
                seq: 99,
                kind: "authority".to_owned(),
                operation: None,
                payload_digest: None,
                text_redacted: "x".to_owned(),
                reject_reason: None,
            }],
            56,
        )
        .expect_err("authority frame kind");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
}

#[test]
fn p13_t02_crash_shaped_rows_reconcile_to_unknown_outcome_never_success() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    let Some(persisted_only) = fenced_or_persist(&fixture, "crash before spawn") else {
        return;
    };
    let dispatched = fenced_or_persist(&fixture, "crash after spawn").expect("persist");
    fixture
        .attempts
        .mark_dispatched(&dispatched, Some("dshchild-x"), 777, 50)
        .expect("dispatch");
    let closed = fenced_or_persist(&fixture, "already terminal").expect("persist");
    fixture
        .attempts
        .record_terminal(&closed, &terminal("exited", Some(2), Some("failed")))
        .expect("terminal");

    let reconciled = fixture
        .attempts
        .reconcile_unknown_outcomes(90)
        .expect("reconcile");
    assert_eq!(reconciled.len(), 2);
    assert!(reconciled.contains(&persisted_only));
    assert!(reconciled.contains(&dispatched));
    for attempt_id in [&persisted_only, &dispatched] {
        let row = fixture
            .attempts
            .get_attempt(attempt_id)
            .expect("get")
            .expect("row");
        assert_eq!(row.state, "unknown-outcome");
        assert_eq!(row.terminal_kind, "unknown-outcome");
        assert_eq!(row.response_status, "unknown");
        assert!(row.pid.is_none());
        assert!(!row.completion_claimed);
        assert_eq!(row.terminal_at, Some(90));
    }
    let untouched = fixture
        .attempts
        .get_attempt(&closed)
        .expect("get")
        .expect("row");
    assert_eq!(untouched.state, "terminal");
    assert_eq!(untouched.terminal_kind, "exited");
    assert_eq!(untouched.exit_code, Some(2));
    let error = fixture
        .attempts
        .record_terminal(&dispatched, &terminal("exited", Some(0), Some("done")))
        .expect_err("late terminal on reconciled row");
    assert!(matches!(error, ProjectAggregateError::Conflict { .. }));
    assert!(
        fixture
            .attempts
            .reconcile_unknown_outcomes(91)
            .expect("idempotent")
            .is_empty()
    );
}

#[test]
fn p13_t02_context_is_bounded_and_secret_free() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(fenced_or_persist(&fixture, "do the task").is_none());
        return;
    }
    let oversize = "x".repeat(HOSTED_ATTEMPT_CONTEXT_MAX_BYTES + 1);
    for context in [
        "",
        "   ",
        oversize.as_str(),
        "use sk-live-123 to call",
        "Bearer abc",
    ] {
        let error = fixture
            .attempts
            .persist_intent(ConfirmCaller::OwnerManagement, &intent(&fixture, context))
            .expect_err("bounded context");
        assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    }
    let exact = "y".repeat(HOSTED_ATTEMPT_CONTEXT_MAX_BYTES);
    let row = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &intent(&fixture, &exact))
        .expect("exact ceiling");
    assert_eq!(row.context_bytes, 65536);
    let mut spec = intent(&fixture, "do the task");
    spec.task_ref = "conversation://personal/not-a-task";
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &spec)
        .expect_err("task ref");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    let mut spec = intent(&fixture, "do the task");
    spec.artifact_digest = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &spec)
        .expect_err("digest");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("digest"))
    );
}

#[test]
fn p13_t02_task_channel_and_unseated_member_cannot_attempt() {
    let fixture = fixture();
    fixture
        .attempts
        .record_artifact_observation(ConfirmCaller::OwnerManagement, &pinned_observation(), 1)
        .expect("pinned");
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::TaskChannel, &intent(&fixture, "do the task"))
        .expect_err("task channel");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::Assistant, &intent(&fixture, "do the task"))
        .expect_err("assistant");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    if HostedDshPlane::isolated_spawn_is_fenced() {
        return;
    }
    let proposed_revision = fixture
        .employees
        .latest_revision_id(&fixture.proposed_employee_id)
        .expect("rev")
        .expect("id");
    let mut spec = intent(&fixture, "do the task");
    spec.employee_id = &fixture.proposed_employee_id;
    spec.employee_revision_id = &proposed_revision;
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &spec)
        .expect_err("unseated");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("seated"))
    );
    let mut spec = intent(&fixture, "do the task");
    spec.employee_revision_id = "employee-revision-stale";
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &spec)
        .expect_err("stale revision");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("revision"))
    );
    let mut spec = intent(&fixture, "do the task");
    spec.task_ref = "task://personal/hidden-pi-assistant/member-engine";
    let error = fixture
        .attempts
        .persist_intent(ConfirmCaller::OwnerManagement, &spec)
        .expect_err("pi member engine");
    assert!(format!("{error}").contains("Pi is not the Member"));
}
