#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T05 D01 failure-first: Routine arming after G2 and the scheduler-
//! driven occurrence ledger.
//!
//! Negatives: arming before G2 / unseated / non-responsible / stale revision
//! / task channel / secret shape; a second scheduler cannot lease or bind;
//! overlap never dispatches twice; a schedule firing on a paused host or a
//! manual trigger on an un-armed Routine is a visible `missed` fact, never a
//! silent drop; checkpoint and process exit are not completion; a new
//! instruction never touches the running occurrence. Green path proves
//! queue-latest promotion and the Today overview counts.

use cognitive_store::{
    CloseRequestSpec, ConfirmCaller, DaemonBindSpec, EmployeeStore, HomeAdmitSpec,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, ROUTINE_ATTEMPT_OUTCOMES,
    ROUTINE_SCHEDULER_LEASE_OWNER, RosterProposal, RoutineArmSpec, RoutineArmingStore,
    RoutineInstructionSpec, RoutineRevisionSpec, RoutineStore, RoutineTriggerSpec, SeatingFacts,
    StageSpec, StageTestOracle, WindowsHostStore, canonical_timestamp_from_ms,
    prepare_personal_databases, routine_scheduler_task_ref,
    scheduler::{SchedulerRepository, SchedulerState, SchedulerWorkKey},
};
use rusqlite::Connection;
use tempfile::TempDir;

struct Fixture {
    _tmp: TempDir,
    path: std::path::PathBuf,
    projects: ProjectAggregateStore,
    routines: RoutineStore,
    armings: RoutineArmingStore,
    host: WindowsHostStore,
    project_id: String,
    plan_id: String,
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

fn pass_stage(store: &ProjectAggregateStore, project_id: &str, plan_id: &str, stage_id: &str) {
    let ring = store.get_stage(plan_id, stage_id).expect("s").expect("row");
    store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            project_id,
            plan_id,
            stage_id,
            &ring.stage_digest,
        )
        .expect("confirm");
    store
        .derive_stage_test_passed(&StageTestOracle {
            project_id: project_id.to_owned(),
            plan_revision_id: plan_id.to_owned(),
            stage_id: stage_id.to_owned(),
            task_ref: format!("task://personal/{stage_id}"),
            seating: SeatingFacts { seated: true },
            verification_current: true,
            verification_report_ref: format!("cas:report-{stage_id}"),
            openable: true,
            checks_passed: true,
            effects_closed: true,
            now_ms: 60,
        })
        .expect("fact");
}

/// G1 project with a two-ring plan, the manager seated and the researcher
/// only proposed. `accept` drives G2 joint acceptance.
fn fixture(accept: bool) -> Fixture {
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
    let routines = RoutineStore::open_path(&path).expect("routines");
    let armings = RoutineArmingStore::open_path(&path).expect("armings");
    let host = WindowsHostStore::open_path(&path).expect("host");

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
    if accept {
        pass_stage(&projects, &project_id, &plan_id, "s1");
        pass_stage(&projects, &project_id, &plan_id, "s2");
        let (preview_id, preview_digest) = projects
            .request_preview("acceptance", &project_id, b"g2-ok", 80)
            .expect("g2 preview");
        let result = projects
            .confirm_preview(
                ConfirmCaller::OwnerManagement,
                &preview_id,
                &preview_digest,
                81,
            )
            .expect("G2");
        assert_eq!(result.kind, "accepted");
    }
    Fixture {
        _tmp: temporary,
        path,
        projects,
        routines,
        armings,
        host,
        project_id,
        plan_id,
        manager_id: ids[0].clone(),
        researcher_id: ids[1].clone(),
    }
}

fn publish(fixture: &Fixture, body: &str, now_ms: i64) -> (String, String) {
    let published = fixture
        .routines
        .publish_revision(
            ConfirmCaller::OwnerManagement,
            &RoutineRevisionSpec {
                project_id: &fixture.project_id,
                routine_id: None,
                body_json: body,
                risk_class: "internal",
                now_ms,
            },
        )
        .expect("publish");
    (published.routine_id, published.revision_id)
}

fn revise(fixture: &Fixture, routine_id: &str, body: &str, now_ms: i64) -> String {
    fixture
        .routines
        .publish_revision(
            ConfirmCaller::OwnerManagement,
            &RoutineRevisionSpec {
                project_id: &fixture.project_id,
                routine_id: Some(routine_id),
                body_json: body,
                risk_class: "internal",
                now_ms,
            },
        )
        .expect("revise")
        .revision_id
}

const INTERVAL_BODY: &str = r#"{"cadence":"interval","interval_ms":5000,"bounded_context":"summarize the day","attempt_timeout_ms":20000}"#;
const MANUAL_BODY: &str = r#"{"cadence":"manual","bounded_context":"run once when asked"}"#;

fn arm<'a>(
    fixture: &'a Fixture,
    routine_id: &'a str,
    revision_id: &'a str,
    stage_id: &'a str,
    employee_id: &'a str,
    now_ms: i64,
) -> Result<cognitive_store::RoutineArming, ProjectAggregateError> {
    fixture.armings.arm(
        ConfirmCaller::OwnerManagement,
        &RoutineArmSpec {
            project_id: &fixture.project_id,
            routine_id,
            revision_id,
            stage_id,
            employee_id,
            now_ms,
        },
    )
}

fn manual_trigger(fixture: &Fixture, routine_id: &str, revision_id: &str, now_ms: i64) -> String {
    fixture
        .routines
        .admit_trigger(
            ConfirmCaller::OwnerManagement,
            &RoutineTriggerSpec {
                routine_id,
                revision_id,
                trigger_kind: "manual",
                trigger_source: "owner-run",
                force_parallel: false,
                host_unavailable: false,
                now_ms,
            },
        )
        .expect("trigger")
        .occurrence_id
}

/// Take the daemon scheduler lease exactly as the tick does. The scheduler
/// row eligibility is a wall-clock fact (P11-T08 seeds `next_eligible` in
/// 2026), so the lease uses the real clock while the ledger uses test instants.
fn lease(fixture: &Fixture, occurrence_id: &str) -> Result<i64, String> {
    let mut repository = SchedulerRepository::open(&fixture.path).expect("repo");
    let key = SchedulerWorkKey {
        task_ref: routine_scheduler_task_ref(occurrence_id),
        contract_epoch: 1,
    };
    let row = repository
        .load(&key)
        .expect("load")
        .ok_or_else(|| "no scheduler row".to_owned())?;
    let epoch = row.lease_epoch + 1;
    let wall = cognitive_store::now_ms();
    repository
        .acquire_eligible_lease(
            &key,
            ROUTINE_SCHEDULER_LEASE_OWNER,
            epoch,
            &canonical_timestamp_from_ms(wall),
            &canonical_timestamp_from_ms(wall + 60_000),
        )
        .map(|_| epoch)
        .map_err(|error| error.to_string())
}

fn release(fixture: &Fixture, occurrence_id: &str, epoch: i64, state: SchedulerState) {
    let mut repository = SchedulerRepository::open(&fixture.path).expect("repo");
    repository
        .release_lease(
            &SchedulerWorkKey {
                task_ref: routine_scheduler_task_ref(occurrence_id),
                contract_epoch: 1,
            },
            ROUTINE_SCHEDULER_LEASE_OWNER,
            epoch,
            state,
            &canonical_timestamp_from_ms(cognitive_store::now_ms()),
        )
        .expect("release");
}

#[test]
fn p13_t05_arming_before_g2_is_refused_and_after_g2_is_armed() {
    let creating = fixture(false);
    let (routine_id, revision_id) = publish(&creating, INTERVAL_BODY, 100);
    let before = arm(
        &creating,
        &routine_id,
        &revision_id,
        "s1",
        &creating.manager_id,
        101,
    );
    match before {
        Err(ProjectAggregateError::Unconfirmed { detail }) => {
            assert!(detail.contains("ROUTINE_ARM_BEFORE_G2"), "{detail}");
        }
        other => panic!("expected G2 refusal, got {other:?}"),
    }
    assert!(creating.armings.live_arming(&routine_id).unwrap().is_none());

    let accepted = fixture(true);
    let project = accepted
        .projects
        .get_project(&accepted.project_id)
        .unwrap()
        .unwrap();
    assert_eq!(project.state, "active");
    let (routine_id, revision_id) = publish(&accepted, INTERVAL_BODY, 100);
    let arming = arm(
        &accepted,
        &routine_id,
        &revision_id,
        "s1",
        &accepted.manager_id,
        1_000,
    )
    .expect("armed after G2");
    assert_eq!(arming.state, "armed");
    assert_eq!(arming.armed_after, "G2");
    assert_eq!(arming.cadence_kind, "interval");
    assert_eq!(arming.interval_ms, Some(5_000));
    assert_eq!(arming.next_due_at, Some(6_000));
    assert_eq!(arming.attempt_timeout_ms, 20_000);
    assert_eq!(arming.plan_revision_id, accepted.plan_id);
    assert_eq!(arming.seq, 1);
    // Arming twice is refused: revisions go through an instruction.
    let twice = arm(
        &accepted,
        &routine_id,
        &revision_id,
        "s1",
        &accepted.manager_id,
        1_001,
    );
    assert!(matches!(twice, Err(ProjectAggregateError::Conflict { .. })));
    let listed = accepted
        .armings
        .list_armings(&accepted.project_id, 10)
        .unwrap();
    assert_eq!(listed.len(), 1);
}

#[test]
fn p13_t05_arming_requires_seated_responsible_member_and_current_revision() {
    let fixture = fixture(true);
    let (routine_id, revision_id) = publish(&fixture, INTERVAL_BODY, 100);
    // Researcher is only proposed → unseated refused.
    let unseated = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s2",
        &fixture.researcher_id,
        101,
    );
    match unseated {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("seated"), "{detail}")
        }
        other => panic!("expected unseated refusal, got {other:?}"),
    }
    // Manager is seated but not responsible for s2.
    let wrong_stage = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s2",
        &fixture.manager_id,
        102,
    );
    match wrong_stage {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("responsible"), "{detail}");
        }
        other => panic!("expected responsibility refusal, got {other:?}"),
    }
    // Unknown stage.
    assert!(matches!(
        arm(
            &fixture,
            &routine_id,
            &revision_id,
            "s9",
            &fixture.manager_id,
            103
        ),
        Err(ProjectAggregateError::NotFound { .. })
    ));
    // Stale revision.
    let successor = revise(&fixture, &routine_id, INTERVAL_BODY, 104);
    assert!(matches!(
        arm(
            &fixture,
            &routine_id,
            &revision_id,
            "s1",
            &fixture.manager_id,
            105
        ),
        Err(ProjectAggregateError::Stale { .. })
    ));
    // Task channel / assistant may not arm.
    for caller in [ConfirmCaller::TaskChannel, ConfirmCaller::Assistant] {
        let forbidden = fixture.armings.arm(
            caller,
            &RoutineArmSpec {
                project_id: &fixture.project_id,
                routine_id: &routine_id,
                revision_id: &successor,
                stage_id: "s1",
                employee_id: &fixture.manager_id,
                now_ms: 106,
            },
        );
        assert!(matches!(
            forbidden,
            Err(ProjectAggregateError::Forbidden { .. })
        ));
    }
    // Invalid declarations fail closed at arm time.
    let (short_routine, short_revision) = publish(
        &fixture,
        r#"{"cadence":"interval","interval_ms":10,"bounded_context":"x"}"#,
        107,
    );
    assert!(matches!(
        arm(
            &fixture,
            &short_routine,
            &short_revision,
            "s1",
            &fixture.manager_id,
            108
        ),
        Err(ProjectAggregateError::Invalid { .. })
    ));
    let (odd_routine, odd_revision) = publish(&fixture, r#"{"cadence":"cron"}"#, 109);
    assert!(matches!(
        arm(
            &fixture,
            &odd_routine,
            &odd_revision,
            "s1",
            &fixture.manager_id,
            110
        ),
        Err(ProjectAggregateError::Invalid { .. })
    ));
    // Secret-shaped declaration never reaches the store (P11-T08 guard).
    let secret = fixture.routines.publish_revision(
        ConfirmCaller::OwnerManagement,
        &RoutineRevisionSpec {
            project_id: &fixture.project_id,
            routine_id: None,
            body_json: r#"{"cadence":"manual","bounded_context":"use sk-live-not-real"}"#,
            risk_class: "internal",
            now_ms: 111,
        },
    );
    assert!(matches!(secret, Err(ProjectAggregateError::Invalid { .. })));
    // Green: the seated manager arms s1 with the current revision.
    let armed = arm(
        &fixture,
        &routine_id,
        &successor,
        "s1",
        &fixture.manager_id,
        112,
    )
    .unwrap();
    assert_eq!(armed.revision_id, successor);
}

#[test]
fn p13_t05_second_scheduler_is_fenced_and_cannot_bind_twice() {
    let fixture = fixture(true);
    let (routine_id, revision_id) = publish(&fixture, MANUAL_BODY, 100);
    let arming = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s1",
        &fixture.manager_id,
        101,
    )
    .unwrap();
    let occurrence_id = manual_trigger(&fixture, &routine_id, &revision_id, 200);
    let dispatchable = fixture.armings.dispatchable_occurrences().unwrap();
    assert_eq!(dispatchable.len(), 1);
    assert_eq!(dispatchable[0].occurrence.occurrence_id, occurrence_id);

    // Binding without the daemon lease is refused: nothing else may dispatch.
    let unleased =
        fixture
            .armings
            .bind_attempt(&occurrence_id, &arming.arming_id, "dshattempt-fake", 1, 201);
    match unleased {
        Err(ProjectAggregateError::Conflict { detail }) => {
            assert!(detail.contains("second scheduler"), "{detail}");
        }
        other => panic!("expected lease refusal, got {other:?}"),
    }

    // The daemon scheduler leases the row; a second dispatcher is fenced.
    let epoch = lease(&fixture, &occurrence_id).expect("first lease");
    let second = lease(&fixture, &occurrence_id);
    assert!(
        second.is_err(),
        "second scheduler must be fenced: {second:?}"
    );
    // A stale epoch cannot bind either.
    assert!(matches!(
        fixture.armings.bind_attempt(
            &occurrence_id,
            &arming.arming_id,
            "dshattempt-a",
            epoch + 7,
            204
        ),
        Err(ProjectAggregateError::Conflict { .. })
    ));
    let bound = fixture
        .armings
        .bind_attempt(
            &occurrence_id,
            &arming.arming_id,
            "dshattempt-a",
            epoch,
            205,
        )
        .expect("bind under lease");
    assert_eq!(bound.attempt_id.as_deref(), Some("dshattempt-a"));
    assert_eq!(bound.lease_epoch, Some(epoch));
    // Binding a second Attempt to the same occurrence is an overlap.
    assert!(matches!(
        fixture.armings.bind_attempt(
            &occurrence_id,
            &arming.arming_id,
            "dshattempt-b",
            epoch,
            206
        ),
        Err(ProjectAggregateError::Conflict { .. })
    ));
    assert!(
        fixture
            .armings
            .dispatchable_occurrences()
            .unwrap()
            .is_empty()
    );
    assert_eq!(fixture.armings.in_flight_occurrences().unwrap().len(), 1);

    // Exactly one scheduler row per occurrence and no second scheduler table.
    let conn = Connection::open(&fixture.path).unwrap();
    let rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM scheduler_entries WHERE task_ref = ?1",
            [routine_scheduler_task_ref(&occurrence_id)],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(rows, 1);
    let foreign: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'
               AND (name LIKE '%temporal%' OR name LIKE '%cron%' OR name LIKE '%second_scheduler%')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(foreign, 0);
}

#[test]
fn p13_t05_overlap_never_dispatches_twice_and_queue_latest_promotes() {
    let fixture = fixture(true);
    let (routine_id, revision_id) = publish(&fixture, INTERVAL_BODY, 100);
    let arming = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s1",
        &fixture.manager_id,
        1_000,
    )
    .unwrap();
    let host = fixture.armings.host_dispatch_availability().unwrap();
    assert!(host.available);

    // Nothing is due before next_due_at.
    assert!(
        fixture
            .armings
            .due_schedule_armings(5_999)
            .unwrap()
            .is_empty()
    );
    let due = fixture.armings.due_schedule_armings(6_000).unwrap();
    assert_eq!(due.len(), 1);
    let first = fixture
        .armings
        .fire_schedule(&due[0], &host, 6_000)
        .unwrap();
    assert_eq!(first.disposition, "active");
    assert_eq!(first.trigger_kind, "schedule");
    let advanced = fixture.armings.get_arming(&arming.arming_id).unwrap();
    assert_eq!(advanced.next_due_at, Some(11_000));
    assert_eq!(advanced.last_fired_at, Some(6_000));

    let epoch = lease(&fixture, &first.occurrence_id).unwrap();
    fixture
        .armings
        .bind_attempt(
            &first.occurrence_id,
            &arming.arming_id,
            "dshattempt-1",
            epoch,
            6_002,
        )
        .unwrap();

    // Second and third firings while the first runs: queue-latest, no dispatch.
    let due = fixture.armings.due_schedule_armings(11_000).unwrap();
    let second = fixture
        .armings
        .fire_schedule(&due[0], &host, 11_000)
        .unwrap();
    assert_eq!(second.disposition, "queued");
    let due = fixture.armings.due_schedule_armings(16_000).unwrap();
    let third = fixture
        .armings
        .fire_schedule(&due[0], &host, 16_000)
        .unwrap();
    assert_eq!(third.disposition, "queued");
    assert!(
        fixture
            .armings
            .dispatchable_occurrences()
            .unwrap()
            .is_empty()
    );
    let coalesced = fixture
        .armings
        .get_ledger_row(&second.occurrence_id)
        .unwrap();
    assert_eq!(coalesced.occurrence.disposition, "coalesced");
    assert_eq!(
        coalesced.occurrence.coalesced_by.as_deref(),
        Some(third.occurrence_id.as_str())
    );
    // Promotion is impossible while the first is still active.
    assert!(
        fixture
            .armings
            .promote_queued(&routine_id, 16_001)
            .unwrap()
            .is_none()
    );

    // Daemon observes the Attempt terminal → occurrence attempted, never completed.
    let attempted = fixture
        .armings
        .record_attempt_terminal(&first.occurrence_id, "done", None, Some(1_234), 17_000)
        .unwrap();
    assert_eq!(attempted.occurrence.disposition, "attempted");
    assert_eq!(attempted.attempt_outcome.as_deref(), Some("done"));
    assert!(!attempted.completion_claimed);
    assert_eq!(attempted.elapsed_ms, Some(1_234));
    release(
        &fixture,
        &first.occurrence_id,
        epoch,
        SchedulerState::Succeeded,
    );

    let promoted = fixture
        .armings
        .promote_queued(&routine_id, 17_001)
        .unwrap()
        .expect("latest queued promoted");
    assert_eq!(promoted.occurrence.occurrence_id, third.occurrence_id);
    assert_eq!(promoted.occurrence.disposition, "active");
    assert!(promoted.attempt_id.is_none());
    assert_eq!(fixture.armings.dispatchable_occurrences().unwrap().len(), 1);
    let ledger = fixture
        .armings
        .list_project_ledger(&fixture.project_id, 32)
        .unwrap();
    let dispositions: Vec<&str> = ledger
        .iter()
        .map(|row| row.occurrence.disposition.as_str())
        .collect();
    assert_eq!(dispositions.len(), 3);
    assert!(dispositions.contains(&"attempted"));
    assert!(dispositions.contains(&"coalesced"));
    assert!(dispositions.contains(&"active"));
    let active_count = dispositions.iter().filter(|d| **d == "active").count();
    assert_eq!(
        active_count, 1,
        "one Routine never has two active occurrences"
    );
}

#[test]
fn p13_t05_silent_drop_is_forbidden_missed_facts_carry_host_and_arming_reasons() {
    let fixture = fixture(true);
    // Un-armed Routine: a manual trigger is Intent, but nothing can dispatch it.
    let (unarmed_routine, unarmed_revision) = publish(&fixture, MANUAL_BODY, 100);
    let orphan = manual_trigger(&fixture, &unarmed_routine, &unarmed_revision, 200);
    assert!(
        fixture
            .armings
            .live_arming(&unarmed_routine)
            .unwrap()
            .is_none()
    );
    let not_armed = fixture.armings.mark_not_armed(&orphan, 201).unwrap();
    assert_eq!(not_armed.occurrence.disposition, "missed");
    assert_eq!(
        not_armed.occurrence.miss_reason.as_deref(),
        Some("not-armed")
    );
    assert!(not_armed.occurrence.scheduler_task_ref.is_none());
    let conn = Connection::open(&fixture.path).unwrap();
    let retired: String = conn
        .query_row(
            "SELECT state FROM scheduler_entries WHERE task_ref = ?1",
            [routine_scheduler_task_ref(&orphan)],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(retired, "failed");
    // Still visible in the Project ledger and resumable once armed (internal risk).
    let ledger = fixture
        .armings
        .list_project_ledger(&fixture.project_id, 32)
        .unwrap();
    assert!(
        ledger
            .iter()
            .any(|row| row.occurrence.occurrence_id == orphan)
    );

    // Close-window pause on the P11-T02 host → schedule firing lands as missed.
    let (routine_id, revision_id) = publish(&fixture, INTERVAL_BODY, 300);
    arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s1",
        &fixture.manager_id,
        1_000,
    )
    .unwrap();
    let home_id = fixture
        .host
        .admit_home(
            ConfirmCaller::OwnerManagement,
            &HomeAdmitSpec {
                install_root: r"C:\Users\owner\Personal Home",
                app_dir: r"C:\Users\owner\Personal Home\app",
                data_dir: r"C:\Users\owner\Personal Home\data",
                acl_policy: "owner-only-dacl",
                argv: &["--personal"],
                env_pairs: &[("PATH", r"C:\Windows\System32")],
                now_ms: 2_000,
            },
        )
        .unwrap()
        .home_id;
    fixture
        .host
        .bind_daemon(
            ConfirmCaller::OwnerManagement,
            &DaemonBindSpec {
                home_id: &home_id,
                can_honor_background: true,
                now_ms: 2_001,
            },
        )
        .unwrap();
    assert!(
        fixture
            .armings
            .host_dispatch_availability()
            .unwrap()
            .available
    );
    fixture
        .host
        .request_close(
            ConfirmCaller::OwnerManagement,
            &CloseRequestSpec {
                home_id: &home_id,
                choice: "pause",
                now_ms: 2_002,
            },
        )
        .unwrap();
    let paused = fixture.armings.host_dispatch_availability().unwrap();
    assert!(!paused.available);
    assert_eq!(paused.reason.as_deref(), Some("close-paused"));
    let due = fixture.armings.due_schedule_armings(6_000).unwrap();
    assert_eq!(due.len(), 1);
    let missed = fixture
        .armings
        .fire_schedule(&due[0], &paused, 6_000)
        .unwrap();
    assert_eq!(missed.disposition, "missed");
    assert_eq!(
        missed.miss_reason.as_deref(),
        Some("host-unavailable:close-paused")
    );
    assert!(missed.scheduler_task_ref.is_none());
    assert!(
        fixture
            .armings
            .dispatchable_occurrences()
            .unwrap()
            .is_empty()
    );
    // The schedule keeps advancing; nothing is dropped from the ledger.
    let advanced = fixture.armings.due_schedule_armings(11_000).unwrap();
    assert_eq!(advanced.len(), 1);
    assert_eq!(advanced[0].last_fired_at, Some(6_000));

    // Offline segment after resume: the reason names the cause.
    fixture
        .host
        .record_offline(ConfirmCaller::OwnerManagement, &home_id, "sleep", 7_000)
        .unwrap();
    let offline = fixture.armings.host_dispatch_availability().unwrap();
    assert!(!offline.available);
    assert_eq!(offline.reason.as_deref(), Some("offline:sleep"));

    // The missed internal occurrence resumes through the P11-T08 path (owner).
    let resumed = fixture
        .routines
        .resume_missed(ConfirmCaller::OwnerManagement, &missed.occurrence_id, 8_000)
        .unwrap();
    assert_eq!(resumed.disposition, "active");
    assert_eq!(fixture.armings.dispatchable_occurrences().unwrap().len(), 1);
}

#[test]
fn p13_t05_checkpoint_and_process_exit_are_not_completion() {
    let fixture = fixture(true);
    let (routine_id, revision_id) = publish(&fixture, MANUAL_BODY, 100);
    let arming = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s1",
        &fixture.manager_id,
        101,
    )
    .unwrap();
    let occurrence_id = manual_trigger(&fixture, &routine_id, &revision_id, 200);
    // P11-T08: a checkpoint never completes.
    let forbidden = fixture.routines.record_checkpoint(
        ConfirmCaller::OwnerManagement,
        &occurrence_id,
        r#"{"step":1}"#,
        true,
    );
    assert!(matches!(
        forbidden,
        Err(ProjectAggregateError::Invalid { .. })
    ));
    // Only daemon-observed terminals are outcomes; `success` / `completed` are not.
    for bogus in ["success", "completed", "complete", "ok"] {
        let refused =
            fixture
                .armings
                .record_attempt_terminal(&occurrence_id, bogus, None, None, 300);
        assert!(
            matches!(refused, Err(ProjectAggregateError::Invalid { .. })),
            "{bogus} must not be an outcome"
        );
    }
    assert!(!ROUTINE_ATTEMPT_OUTCOMES.contains(&"success"));
    // Secret-shaped detail is refused before it is stored.
    assert!(matches!(
        fixture.armings.record_attempt_terminal(
            &occurrence_id,
            "failed",
            Some("Authorization: Bearer sess-not-real"),
            None,
            301
        ),
        Err(ProjectAggregateError::Invalid { .. })
    ));
    // Every daemon terminal kind lands as an outcome fact with no completion.
    let epoch = lease(&fixture, &occurrence_id).unwrap();
    fixture
        .armings
        .bind_attempt(
            &occurrence_id,
            &arming.arming_id,
            "dshattempt-x",
            epoch,
            303,
        )
        .unwrap();
    let exited_zero = fixture
        .armings
        .record_attempt_terminal(
            &occurrence_id,
            "unknown",
            Some("child exited 0 without a response frame"),
            Some(50),
            400,
        )
        .unwrap();
    assert_eq!(exited_zero.occurrence.disposition, "attempted");
    assert_eq!(exited_zero.attempt_outcome.as_deref(), Some("unknown"));
    assert!(!exited_zero.completion_claimed);
    // A terminal cannot be recorded twice, and the schema refuses completion.
    assert!(matches!(
        fixture
            .armings
            .record_attempt_terminal(&occurrence_id, "done", None, None, 401),
        Err(ProjectAggregateError::Invalid { .. })
    ));
    let conn = Connection::open(&fixture.path).unwrap();
    let completed = conn.execute(
        "UPDATE p11_routine_occurrence SET completion_claimed = 1 WHERE occurrence_id = ?1",
        [&occurrence_id],
    );
    assert!(
        completed.is_err(),
        "schema must refuse completion_claimed = 1"
    );
    let success = conn.execute(
        "UPDATE p11_routine_occurrence SET attempt_outcome = 'success' WHERE occurrence_id = ?1",
        [&occurrence_id],
    );
    assert!(success.is_err(), "schema must refuse a success outcome");
    for outcome in ROUTINE_ATTEMPT_OUTCOMES {
        let occurrence = manual_trigger(&fixture, &routine_id, &revision_id, 500);
        let epoch = lease(&fixture, &occurrence).unwrap();
        fixture
            .armings
            .bind_attempt(
                &occurrence,
                &arming.arming_id,
                &format!("dshattempt-{outcome}"),
                epoch,
                502,
            )
            .unwrap();
        let row = fixture
            .armings
            .record_attempt_terminal(&occurrence, outcome, None, None, 503)
            .unwrap();
        assert_eq!(row.attempt_outcome.as_deref(), Some(outcome));
        assert!(!row.completion_claimed);
        release(&fixture, &occurrence, epoch, SchedulerState::Failed);
    }
}

#[test]
fn p13_t05_instruction_applies_at_safe_point_and_never_injects_running_prompt() {
    let fixture = fixture(true);
    let (routine_id, revision_id) = publish(&fixture, INTERVAL_BODY, 100);
    let arming = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s1",
        &fixture.manager_id,
        1_000,
    )
    .unwrap();
    let running = manual_trigger(&fixture, &routine_id, &revision_id, 2_000);
    let epoch = lease(&fixture, &running).unwrap();
    let bound = fixture
        .armings
        .bind_attempt(
            &running,
            &arming.arming_id,
            "dshattempt-running",
            epoch,
            2_002,
        )
        .unwrap();

    // A new Owner instruction = a new Routine revision.
    let revised = revise(
        &fixture,
        &routine_id,
        r#"{"cadence":"interval","interval_ms":7000,"bounded_context":"summarize the week instead"}"#,
        3_000,
    );
    // Applying a stale revision is refused; unknown apply is refused.
    assert!(matches!(
        fixture.armings.apply_instruction(
            ConfirmCaller::OwnerManagement,
            &RoutineInstructionSpec {
                arming_id: &arming.arming_id,
                revision_id: &revision_id,
                apply: "continue",
                now_ms: 3_001,
            },
        ),
        Err(ProjectAggregateError::Stale { .. })
    ));
    assert!(matches!(
        fixture.armings.apply_instruction(
            ConfirmCaller::OwnerManagement,
            &RoutineInstructionSpec {
                arming_id: &arming.arming_id,
                revision_id: &revised,
                apply: "inject",
                now_ms: 3_001,
            },
        ),
        Err(ProjectAggregateError::Invalid { .. })
    ));
    assert!(matches!(
        fixture.armings.apply_instruction(
            ConfirmCaller::TaskChannel,
            &RoutineInstructionSpec {
                arming_id: &arming.arming_id,
                revision_id: &revised,
                apply: "continue",
                now_ms: 3_001,
            },
        ),
        Err(ProjectAggregateError::Forbidden { .. })
    ));

    // continue: next occurrence uses the new revision; the running one is untouched.
    let continued = fixture
        .armings
        .apply_instruction(
            ConfirmCaller::OwnerManagement,
            &RoutineInstructionSpec {
                arming_id: &arming.arming_id,
                revision_id: &revised,
                apply: "continue",
                now_ms: 3_002,
            },
        )
        .unwrap();
    assert_eq!(continued.arming.state, "armed");
    assert_eq!(continued.arming.seq, 2);
    assert_eq!(continued.arming.revision_id, revised);
    assert_eq!(continued.arming.interval_ms, Some(7_000));
    assert_eq!(continued.arming.apply_mode, "continue");
    assert_eq!(
        continued.active_occurrence_id.as_deref(),
        Some(running.as_str())
    );
    assert!(continued.restart_occurrence.is_none());
    let untouched = fixture.armings.get_ledger_row(&running).unwrap();
    assert_eq!(untouched.occurrence.revision_id, revision_id);
    assert_eq!(untouched.attempt_id, bound.attempt_id);
    assert_eq!(untouched.occurrence.disposition, "active");
    assert_eq!(
        fixture.armings.get_arming(&arming.arming_id).unwrap().state,
        "superseded"
    );
    assert!(matches!(
        fixture.armings.apply_instruction(
            ConfirmCaller::OwnerManagement,
            &RoutineInstructionSpec {
                arming_id: &arming.arming_id,
                revision_id: &revised,
                apply: "pause",
                now_ms: 3_003,
            },
        ),
        Err(ProjectAggregateError::Stale { .. })
    ));

    // pause: no new occurrences fire; the running one still finishes on its own.
    let paused = fixture
        .armings
        .apply_instruction(
            ConfirmCaller::OwnerManagement,
            &RoutineInstructionSpec {
                arming_id: &continued.arming.arming_id,
                revision_id: &revised,
                apply: "pause",
                now_ms: 3_004,
            },
        )
        .unwrap();
    assert_eq!(paused.arming.state, "paused");
    assert!(paused.arming.next_due_at.is_none());
    assert!(
        fixture
            .armings
            .due_schedule_armings(1_000_000)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fixture
            .armings
            .get_ledger_row(&running)
            .unwrap()
            .occurrence
            .disposition,
        "active"
    );
    assert!(fixture.armings.in_flight_occurrences().unwrap().len() == 1);

    // resume: armed again with the same declaration.
    let resumed = fixture
        .armings
        .resume_arming(
            ConfirmCaller::OwnerManagement,
            &paused.arming.arming_id,
            4_000,
        )
        .unwrap();
    assert_eq!(resumed.state, "armed");
    assert_eq!(resumed.apply_mode, "resume");
    assert_eq!(resumed.next_due_at, Some(11_000));
    assert!(matches!(
        fixture
            .armings
            .resume_arming(ConfirmCaller::OwnerManagement, &resumed.arming_id, 4_001),
        Err(ProjectAggregateError::Invalid { .. })
    ));

    // restart: the new revision is queued behind the running occurrence.
    let restarted = fixture
        .armings
        .apply_instruction(
            ConfirmCaller::OwnerManagement,
            &RoutineInstructionSpec {
                arming_id: &resumed.arming_id,
                revision_id: &revised,
                apply: "restart",
                now_ms: 5_000,
            },
        )
        .unwrap();
    let queued = restarted.restart_occurrence.expect("restart occurrence");
    assert_eq!(queued.disposition, "queued");
    assert_eq!(queued.revision_id, revised);
    assert_eq!(queued.trigger_source, "instruction-restart");
    assert_eq!(
        restarted.active_occurrence_id.as_deref(),
        Some(running.as_str())
    );
    let still_running = fixture.armings.get_ledger_row(&running).unwrap();
    assert_eq!(still_running.occurrence.disposition, "active");
    assert_eq!(
        still_running.attempt_id.as_deref(),
        Some("dshattempt-running")
    );
    assert_eq!(still_running.occurrence.revision_id, revision_id);
    let queued_row = fixture
        .armings
        .get_ledger_row(&queued.occurrence_id)
        .unwrap();
    assert_eq!(
        queued_row.arming_id.as_deref(),
        Some(restarted.arming.arming_id.as_str())
    );
    // Safe point: the running Attempt terminates, then the restart occurrence
    // is promoted with the new revision.
    fixture
        .armings
        .record_attempt_terminal(&running, "done", None, Some(10), 6_000)
        .unwrap();
    release(&fixture, &running, epoch, SchedulerState::Succeeded);
    let promoted = fixture
        .armings
        .promote_queued(&routine_id, 6_001)
        .unwrap()
        .expect("promoted");
    assert_eq!(promoted.occurrence.occurrence_id, queued.occurrence_id);
    assert_eq!(promoted.occurrence.revision_id, revised);
    assert_eq!(
        fixture
            .armings
            .list_armings(&fixture.project_id, 10)
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn p13_t05_today_overview_counts_live_projects_and_periods() {
    let fixture = fixture(true);
    // Second Project stays creating (G1 only) → counted as created, no row.
    let (draft_id, _) = fixture.projects.create_draft(b"charter-v2", 90).unwrap();
    fixture
        .projects
        .put_draft_charter(&draft_id, b"charter-body-v2", 91)
        .unwrap();
    let (preview_id, digest) = fixture
        .projects
        .request_preview("activation", &draft_id, b"activation-2", 92)
        .unwrap();
    fixture
        .projects
        .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 93)
        .unwrap();

    let (routine_id, revision_id) = publish(&fixture, MANUAL_BODY, 100);
    let arming = arm(
        &fixture,
        &routine_id,
        &revision_id,
        "s1",
        &fixture.manager_id,
        101,
    )
    .unwrap();
    let day = 86_400_000_i64;
    let now = 10 * day + 3_600_000; // 01:00 UTC on day 10
    // Yesterday: one failed attempt.
    let yesterday = manual_trigger(&fixture, &routine_id, &revision_id, now - day);
    let epoch = lease(&fixture, &yesterday).unwrap();
    fixture
        .armings
        .bind_attempt(
            &yesterday,
            &arming.arming_id,
            "dshattempt-y",
            epoch,
            now - day + 2,
        )
        .unwrap();
    fixture
        .armings
        .record_attempt_terminal(
            &yesterday,
            "failed",
            Some("dsh-exit-1"),
            Some(2_000),
            now - day + 3,
        )
        .unwrap();
    release(&fixture, &yesterday, epoch, SchedulerState::Failed);
    // Today: two done attempts and one unknown-outcome.
    for (index, outcome) in ["done", "done", "unknown-outcome"].iter().enumerate() {
        let at = now - 600_000 + (index as i64) * 60_000;
        let occurrence = manual_trigger(&fixture, &routine_id, &revision_id, at);
        let epoch = lease(&fixture, &occurrence).unwrap();
        fixture
            .armings
            .bind_attempt(
                &occurrence,
                &arming.arming_id,
                &format!("dshattempt-{index}"),
                epoch,
                at + 2,
            )
            .unwrap();
        fixture
            .armings
            .record_attempt_terminal(&occurrence, outcome, None, Some(1_000), at + 3)
            .unwrap();
        release(&fixture, &occurrence, epoch, SchedulerState::Succeeded);
    }
    // One running occurrence and one queued behind it.
    let running = manual_trigger(&fixture, &routine_id, &revision_id, now - 1_000);
    let epoch = lease(&fixture, &running).unwrap();
    fixture
        .armings
        .bind_attempt(
            &running,
            &arming.arming_id,
            "dshattempt-run",
            epoch,
            now - 998,
        )
        .unwrap();
    let queued = manual_trigger(&fixture, &routine_id, &revision_id, now - 500);
    assert_eq!(
        fixture
            .armings
            .get_ledger_row(&queued)
            .unwrap()
            .occurrence
            .disposition,
        "queued"
    );

    let today = fixture.armings.today_overview("today", now).unwrap();
    assert_eq!(today.created_count, 1);
    assert_eq!(today.live_count, 1);
    assert_eq!(today.blocked_count, 0);
    assert_eq!(today.period_start_ms, 10 * day);
    assert_eq!(today.rows.len(), 1, "one row per live Project only");
    let row = &today.rows[0];
    assert_eq!(row.project_id, fixture.project_id);
    assert_eq!(row.attempts_total, 3);
    assert_eq!(row.attempts_done, 2);
    assert_eq!(row.attempts_failed, 0);
    assert_eq!(row.attempts_unknown, 1);
    assert_eq!(row.duration_ms, Some(3_000));
    assert_eq!(row.running_occurrence_id.as_deref(), Some(running.as_str()));
    assert_eq!(row.queued_count, 1);
    assert_eq!(row.armed_routines, 1);
    assert_eq!(row.current_stage_id.as_deref(), Some("s1"));
    assert_eq!(row.current_stage_title.as_deref(), Some("Manage"));

    let week = fixture.armings.today_overview("week", now).unwrap();
    assert_eq!(week.rows[0].attempts_total, 4);
    assert_eq!(week.rows[0].attempts_failed, 1);
    assert_eq!(week.rows[0].duration_ms, Some(5_000));
    let month = fixture.armings.today_overview("month", now).unwrap();
    assert_eq!(month.rows[0].attempts_total, 4);
    assert!(matches!(
        fixture.armings.today_overview("year", now),
        Err(ProjectAggregateError::Invalid { .. })
    ));

    // A Project with no attempts in the period reports no duration, not 0.
    let quiet = fixture
        .armings
        .today_overview("today", now + 2 * day)
        .unwrap();
    assert_eq!(quiet.rows[0].attempts_total, 0);
    assert_eq!(quiet.rows[0].duration_ms, None);
}
