#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P11-T08 D01: Routine revision + Trigger no-overlap / missed ledger.
//!
//! Failure-first: overlap rejected; silent drop forbidden; stale policy
//! fail-closed; checkpoint is not completion; consequential auto-resume
//! forbidden. Green path proves queue-latest + daemon scheduler reuse.

use cognitive_store::{
    ConfirmCaller, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    RoutineRevisionSpec, RoutineStore, RoutineTriggerSpec, SqliteAuthorityStore,
    prepare_personal_databases, routine_scheduler_task_ref,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    SqliteAuthorityStore,
    ProjectAggregateStore,
    RoutineStore,
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
    let authority = SqliteAuthorityStore::open(&path).expect("authority");
    let projects = ProjectAggregateStore::open_path(&path).expect("projects");
    let routines = RoutineStore::from_authority_store(&authority);
    (temporary, authority, projects, routines)
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

fn publish(
    routines: &RoutineStore,
    project_id: &str,
    risk_class: &str,
    now_ms: i64,
) -> (String, String) {
    let published = routines
        .publish_revision(
            ConfirmCaller::OwnerManagement,
            &RoutineRevisionSpec {
                project_id,
                routine_id: None,
                body_json: r#"{"cadence":"manual","title":"nightly notes"}"#,
                risk_class,
                now_ms,
            },
        )
        .expect("publish");
    (published.routine_id, published.revision_id)
}

fn trigger(
    routines: &RoutineStore,
    routine_id: &str,
    revision_id: &str,
    force_parallel: bool,
    host_unavailable: bool,
    now_ms: i64,
) -> Result<cognitive_store::RoutineOccurrence, ProjectAggregateError> {
    routines.admit_trigger(
        ConfirmCaller::OwnerManagement,
        &RoutineTriggerSpec {
            routine_id,
            revision_id,
            trigger_kind: "manual",
            trigger_source: "owner-run",
            force_parallel,
            host_unavailable,
            now_ms,
        },
    )
}

#[test]
fn p11_t08_overlap_is_rejected() {
    let (_tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let (routine_id, revision_id) = publish(&routines, &project_id, "internal", 20);
    let first = trigger(&routines, &routine_id, &revision_id, false, false, 21).expect("first");
    assert_eq!(first.disposition, "active");
    let overlap = trigger(&routines, &routine_id, &revision_id, true, false, 22);
    match overlap {
        Err(ProjectAggregateError::Conflict { detail }) => {
            assert!(detail.contains("overlap rejected"), "{detail}");
        }
        other => panic!("expected overlap conflict, got {other:?}"),
    }
    let queued = trigger(&routines, &routine_id, &revision_id, false, false, 23).expect("queued");
    assert_eq!(queued.disposition, "queued");
}

#[test]
fn p11_t08_secret_shape_is_rejected() {
    let (_tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let body = routines.publish_revision(
        ConfirmCaller::OwnerManagement,
        &RoutineRevisionSpec {
            project_id: &project_id,
            routine_id: None,
            body_json: r#"{"title":"x","api_key":"sk-test"}"#,
            risk_class: "internal",
            now_ms: 80,
        },
    );
    match body {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(detail.contains("secret-shaped"), "{detail}");
        }
        other => panic!("expected secret body reject, got {other:?}"),
    }
    let (routine_id, revision_id) = publish(&routines, &project_id, "internal", 81);
    let source = routines.admit_trigger(
        ConfirmCaller::OwnerManagement,
        &RoutineTriggerSpec {
            routine_id: &routine_id,
            revision_id: &revision_id,
            trigger_kind: "schedule",
            trigger_source: "bearer sk-leak",
            force_parallel: false,
            host_unavailable: false,
            now_ms: 82,
        },
    );
    match source {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(detail.contains("secret-shaped"), "{detail}");
        }
        other => panic!("expected secret source reject, got {other:?}"),
    }
    let active = trigger(&routines, &routine_id, &revision_id, false, false, 83).expect("active");
    let checkpoint = routines.record_checkpoint(
        ConfirmCaller::OwnerManagement,
        &active.occurrence_id,
        r#"{"token":"ssv1:abc"}"#,
        false,
    );
    match checkpoint {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(detail.contains("secret-shaped"), "{detail}");
        }
        other => panic!("expected secret checkpoint reject, got {other:?}"),
    }
    let assistant = routines.admit_trigger(
        ConfirmCaller::Assistant,
        &RoutineTriggerSpec {
            routine_id: &routine_id,
            revision_id: &revision_id,
            trigger_kind: "qualified-event",
            trigger_source: "owner-run",
            force_parallel: false,
            host_unavailable: false,
            now_ms: 84,
        },
    );
    match assistant {
        Err(ProjectAggregateError::Forbidden { .. }) => {}
        other => panic!("expected assistant forbidden, got {other:?}"),
    }
}

#[test]
fn p11_t08_silent_drop_is_forbidden() {
    let (_tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let (routine_id, revision_id) = publish(&routines, &project_id, "internal", 30);
    let missed = trigger(&routines, &routine_id, &revision_id, false, true, 31).expect("missed");
    assert_eq!(missed.disposition, "missed");
    assert_eq!(missed.miss_reason.as_deref(), Some("host-unavailable"));
    let ledger = routines
        .list_ledger(&project_id, &routine_id)
        .expect("ledger");
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger[0].disposition, "missed");
    assert!(ledger[0].scheduler_task_ref.is_none());
    let resumed = routines
        .resume_missed(ConfirmCaller::OwnerManagement, &missed.occurrence_id, 32)
        .expect("resume internal missed");
    assert_eq!(resumed.disposition, "active");
    assert!(resumed.scheduler_task_ref.is_some());
}

#[test]
fn p11_t08_stale_policy_fail_closed() {
    let (_tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let (routine_id, revision_id) = publish(&routines, &project_id, "internal", 40);
    let successor = routines
        .publish_revision(
            ConfirmCaller::OwnerManagement,
            &RoutineRevisionSpec {
                project_id: &project_id,
                routine_id: Some(&routine_id),
                body_json: r#"{"cadence":"manual","title":"revised"}"#,
                risk_class: "internal",
                now_ms: 41,
            },
        )
        .expect("successor");
    assert_ne!(successor.revision_id, revision_id);
    let stale = trigger(&routines, &routine_id, &revision_id, false, false, 42);
    match stale {
        Err(ProjectAggregateError::Stale { detail }) => {
            assert!(detail.contains("stale"), "{detail}");
        }
        other => panic!("expected stale, got {other:?}"),
    }
}

#[test]
fn p11_t08_checkpoint_is_not_completion() {
    let (_tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let (routine_id, revision_id) = publish(&routines, &project_id, "internal", 50);
    let active = trigger(&routines, &routine_id, &revision_id, false, false, 51).expect("active");
    let forbidden = routines.record_checkpoint(
        ConfirmCaller::OwnerManagement,
        &active.occurrence_id,
        r#"{"step":1}"#,
        true,
    );
    match forbidden {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(detail.contains("checkpoint is not completion"), "{detail}");
        }
        other => panic!("expected invalid, got {other:?}"),
    }
    let saved = routines
        .record_checkpoint(
            ConfirmCaller::OwnerManagement,
            &active.occurrence_id,
            r#"{"step":1}"#,
            false,
        )
        .expect("checkpoint");
    assert_eq!(saved.disposition, "active");
    assert_eq!(saved.checkpoint_json.as_deref(), Some(r#"{"step":1}"#));
}

#[test]
fn p11_t08_consequential_auto_resume_is_forbidden() {
    let (_tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let (routine_id, revision_id) = publish(&routines, &project_id, "consequential", 60);
    let missed = trigger(&routines, &routine_id, &revision_id, false, true, 61).expect("missed");
    let resume = routines.resume_missed(ConfirmCaller::OwnerManagement, &missed.occurrence_id, 62);
    match resume {
        Err(ProjectAggregateError::Forbidden { detail }) => {
            assert!(detail.contains("consequential auto-resume"), "{detail}");
        }
        other => panic!("expected forbidden, got {other:?}"),
    }
    let task = routines.admit_trigger(
        ConfirmCaller::TaskChannel,
        &RoutineTriggerSpec {
            routine_id: &routine_id,
            revision_id: &revision_id,
            trigger_kind: "manual",
            trigger_source: "task",
            force_parallel: false,
            host_unavailable: false,
            now_ms: 63,
        },
    );
    match task {
        Err(ProjectAggregateError::Forbidden { .. }) => {}
        other => panic!("expected task-channel forbidden, got {other:?}"),
    }
}

#[test]
fn p11_t08_queue_latest_reuses_daemon_scheduler() {
    let (tmp, _authority, projects, routines) = stores();
    let project_id = activate(&projects);
    let (routine_id, revision_id) = publish(&routines, &project_id, "internal", 70);
    let first = trigger(&routines, &routine_id, &revision_id, false, false, 71).expect("active");
    assert_eq!(first.disposition, "active");
    let expected_ref = routine_scheduler_task_ref(&first.occurrence_id);
    assert_eq!(
        first.scheduler_task_ref.as_deref(),
        Some(expected_ref.as_str())
    );
    let queued_old = trigger(&routines, &routine_id, &revision_id, false, false, 72).expect("q1");
    assert_eq!(queued_old.disposition, "queued");
    let queued_new = trigger(&routines, &routine_id, &revision_id, false, false, 73).expect("q2");
    assert_eq!(queued_new.disposition, "queued");
    let ledger = routines
        .list_ledger(&project_id, &routine_id)
        .expect("ledger");
    let coalesced = ledger
        .iter()
        .find(|row| row.occurrence_id == queued_old.occurrence_id)
        .expect("old queued");
    assert_eq!(coalesced.disposition, "coalesced");
    assert_eq!(
        coalesced.coalesced_by.as_deref(),
        Some(queued_new.occurrence_id.as_str())
    );
    let layout_root = tmp.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        layout_root.join("config"),
        layout_root.join("data"),
        layout_root.join("state"),
        layout_root.join("cache"),
        layout_root.join("runtime"),
    );
    let conn = Connection::open(layout.authority_database_path()).expect("open");
    let state: String = conn
        .query_row(
            "SELECT state FROM scheduler_entries WHERE task_ref = ?1 AND contract_epoch = 1",
            [expected_ref],
            |row| row.get(0),
        )
        .expect("scheduler row");
    assert_eq!(state, "runnable");
    let extra: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name LIKE '%temporal%'",
            [],
            |row| row.get(0),
        )
        .expect("no temporal");
    assert_eq!(extra, 0);
}
