//! P2-T03 failure-first behavior: durable scheduler lease, takeover and
//! cancellation against the real SQLite WAL scheduler repository.
//!
//! Acceptance invariants exercised:
//! - a lease is exclusive (duplicate worker acquire is refused by CAS);
//! - after a worker crash the lease can be taken over only by an explicit
//!   release path that fails closed on owner or epoch mismatch;
//! - attempt_count advances per acquire;
//! - cancel_requested is durable and blocks re-acquire;
//! - a fresh repository reopen (crash/replay) sees only committed rows.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_runtime::SchedulerService;
use cognitive_store::scheduler::{SchedulerRepository, SchedulerRow, SchedulerState};

fn open_repo(dir: &tempfile::TempDir) -> SchedulerRepository {
    SchedulerRepository::open(&dir.path().join("scheduler.db")).unwrap()
}

fn runnable_row(task_ref: &str) -> SchedulerRow {
    SchedulerRow {
        task_ref: task_ref.to_owned(),
        state: SchedulerState::Runnable.as_str().to_owned(),
        lease_owner: None,
        lease_epoch: 0,
        lease_expires: None,
        next_eligible: "2026-08-01T12:00:00Z".to_owned(),
        attempt_count: 0,
        cancel_requested: false,
    }
}

#[test]
fn durable_discovery_reopens_only_non_terminal_scheduler_work() {
    let dir = tempfile::tempdir().unwrap();
    let mut repository = open_repo(&dir);
    repository
        .upsert(&runnable_row("task://tenant-a/runnable"))
        .unwrap();
    repository
        .upsert(&SchedulerRow {
            task_ref: "task://tenant-a/leased".to_owned(),
            state: SchedulerState::Leased.as_str().to_owned(),
            lease_owner: Some("crashed-worker".to_owned()),
            lease_epoch: 4,
            lease_expires: Some("2026-08-01T12:00:30Z".to_owned()),
            next_eligible: "2026-08-01T12:00:00Z".to_owned(),
            attempt_count: 1,
            cancel_requested: false,
        })
        .unwrap();
    repository
        .upsert(&SchedulerRow {
            task_ref: "task://tenant-a/closed".to_owned(),
            state: SchedulerState::Succeeded.as_str().to_owned(),
            lease_owner: None,
            lease_epoch: 2,
            lease_expires: None,
            next_eligible: "2026-08-01T12:00:00Z".to_owned(),
            attempt_count: 1,
            cancel_requested: false,
        })
        .unwrap();
    drop(repository);

    // Reopen models a daemon crash: discovery includes runnable and leased
    // recovery candidates, but never a terminal row.
    let recovered = open_repo(&dir).list_recoverable().unwrap();
    assert_eq!(
        recovered
            .iter()
            .map(|row| row.task_ref.as_str())
            .collect::<Vec<_>>(),
        vec!["task://tenant-a/leased", "task://tenant-a/runnable"]
    );
}

// ---------------------------------------------------------------------
// 1. lease is exclusive (duplicate worker acquire refused by CAS)
// ---------------------------------------------------------------------

#[test]
fn lease_acquire_is_exclusive_and_rejects_duplicate_owner() {
    let dir = tempfile::tempdir().unwrap();
    let mut repo = open_repo(&dir);
    repo.upsert(&runnable_row("task://tenant-a/rollout-v2"))
        .unwrap();

    let leased = repo
        .acquire_lease(
            "task://tenant-a/rollout-v2",
            "worker-1",
            1,
            "2026-08-01T12:10:00Z",
        )
        .unwrap();
    assert_eq!(leased.state, SchedulerState::Leased.as_str());
    assert_eq!(leased.lease_owner.as_deref(), Some("worker-1"));
    assert_eq!(leased.lease_epoch, 1);
    assert_eq!(leased.attempt_count, 1);

    // A second worker (or the same worker twice) is refused while leased.
    let duplicate = repo.acquire_lease(
        "task://tenant-a/rollout-v2",
        "worker-2",
        1,
        "2026-08-01T12:10:00Z",
    );
    assert!(
        matches!(
            duplicate,
            Err(cognitive_store::scheduler::SchedulerRepositoryError::LeaseConflict(_))
        ),
        "duplicate lease must be refused"
    );
}

// ---------------------------------------------------------------------
// 2. crash takeover: release is owner- and epoch-bound and fails closed
// ---------------------------------------------------------------------

#[test]
fn release_lease_fails_closed_on_owner_or_epoch_mismatch_and_releases_on_match() {
    let dir = tempfile::tempdir().unwrap();
    let mut repo = open_repo(&dir);
    repo.upsert(&runnable_row("task://tenant-a/rollout-v2"))
        .unwrap();
    repo.acquire_lease(
        "task://tenant-a/rollout-v2",
        "worker-1",
        1,
        "2026-08-01T12:10:00Z",
    )
    .unwrap();

    // Wrong owner release is refused (a crashed worker's identity cannot
    // release someone else's lease).
    let wrong = repo.release_lease(
        "task://tenant-a/rollout-v2",
        "worker-2",
        1,
        SchedulerState::Runnable,
        "2026-08-01T12:05:00Z",
    );
    assert!(wrong.is_err(), "owner mismatch on release must be refused");

    // Correct owner release makes the task runnable again for takeover.
    let released = repo
        .release_lease(
            "task://tenant-a/rollout-v2",
            "worker-1",
            1,
            SchedulerState::Runnable,
            "2026-08-01T12:05:00Z",
        )
        .unwrap();
    assert_eq!(released.state, SchedulerState::Runnable.as_str());
    assert_eq!(released.lease_owner, None);
    assert_eq!(released.attempt_count, 1);

    // Takeover now succeeds and advances the attempt counter.
    let taken = repo
        .acquire_lease(
            "task://tenant-a/rollout-v2",
            "worker-2",
            2,
            "2026-08-01T12:20:00Z",
        )
        .unwrap();
    assert_eq!(taken.lease_owner.as_deref(), Some("worker-2"));
    assert_eq!(taken.lease_epoch, 2);
    assert_eq!(taken.attempt_count, 2);
}

#[test]
fn stale_epoch_cannot_release_a_successor_lease_owned_by_the_same_worker() {
    let dir = tempfile::tempdir().unwrap();
    let mut repo = open_repo(&dir);
    repo.upsert(&runnable_row("task://tenant-a/stale-release"))
        .unwrap();

    let mut first_worker = SchedulerService::new("worker-1", 60).unwrap();
    first_worker
        .claim_eligible(
            &mut repo,
            "task://tenant-a/stale-release",
            1,
            "2026-08-01T12:00:00Z",
        )
        .unwrap();

    let mut successor_worker = SchedulerService::new("worker-1", 60).unwrap();
    let successor_dispatch = successor_worker
        .claim_eligible(
            &mut repo,
            "task://tenant-a/stale-release",
            2,
            "2026-08-01T12:01:01Z",
        )
        .unwrap();

    let stale_release = repo.release_lease(
        "task://tenant-a/stale-release",
        "worker-1",
        1,
        SchedulerState::Succeeded,
        "2026-08-01T12:01:01Z",
    );
    assert!(
        stale_release.is_err(),
        "stale epoch release must be refused"
    );

    let current = repo
        .load("task://tenant-a/stale-release")
        .unwrap()
        .expect("the successor lease must remain durable");
    assert_eq!(current.state, SchedulerState::Leased.as_str());
    assert_eq!(current.lease_owner.as_deref(), Some("worker-1"));
    assert_eq!(current.lease_epoch, successor_dispatch.lease_epoch);
}

// ---------------------------------------------------------------------
// 3. durable across reopen (crash/replay sees only committed rows)
// ---------------------------------------------------------------------

#[test]
fn scheduler_rows_survive_reopen_like_a_crash_replay() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut repo = open_repo(&dir);
        repo.upsert(&runnable_row("task://tenant-a/rollout-v2"))
            .unwrap();
        repo.acquire_lease(
            "task://tenant-a/rollout-v2",
            "worker-1",
            1,
            "2026-08-01T12:10:00Z",
        )
        .unwrap();
        repo.request_cancel("task://tenant-a/rollout-v2").unwrap();
    }
    // Drop = crash. Reopen sees the committed lease + cancel.
    let mut repo = open_repo(&dir);
    let row = repo.load("task://tenant-a/rollout-v2").unwrap().unwrap();
    assert_eq!(row.lease_owner.as_deref(), Some("worker-1"));
    assert_eq!(row.attempt_count, 1);
    assert!(row.cancel_requested);

    // Cancelled tasks cannot be re-acquired.
    let acquire = repo.acquire_lease(
        "task://tenant-a/rollout-v2",
        "worker-2",
        2,
        "2026-08-01T12:20:00Z",
    );
    assert!(acquire.is_err(), "cancelled task must not be re-acquired");
}

// ---------------------------------------------------------------------
// 4. cancel request is durable and blocks further dispatch
// ---------------------------------------------------------------------

#[test]
fn cancel_request_blocks_future_lease_acquisition() {
    let dir = tempfile::tempdir().unwrap();
    let mut repo = open_repo(&dir);
    repo.upsert(&runnable_row("task://tenant-a/rollout-v2"))
        .unwrap();
    repo.request_cancel("task://tenant-a/rollout-v2").unwrap();

    let acquire = repo.acquire_lease(
        "task://tenant-a/rollout-v2",
        "worker-1",
        1,
        "2026-08-01T12:10:00Z",
    );
    assert!(acquire.is_err(), "cancelled task must refuse dispatch");
}

// ---------------------------------------------------------------------
// 5. the service clamps backwards wall time and permits expired takeover
// ---------------------------------------------------------------------

#[test]
fn scheduler_service_prevents_clock_rollback_double_dispatch_and_reclaims_expired_lease() {
    let dir = tempfile::tempdir().unwrap();
    let mut repo = open_repo(&dir);
    repo.upsert(&runnable_row("task://tenant-a/rollout-v2"))
        .unwrap();
    let mut scheduler = SchedulerService::new("worker-1", 60).unwrap();

    let first_dispatch = scheduler
        .claim_eligible(
            &mut repo,
            "task://tenant-a/rollout-v2",
            1,
            "2026-08-01T12:00:00Z",
        )
        .unwrap();
    assert_eq!(first_dispatch.lease_epoch, 1);

    // A backwards wall-clock observation stays at the last trusted value,
    // so it cannot make an earlier work window eligible again.
    assert_eq!(
        scheduler.observe_wall_time("2026-08-01T11:59:00Z").unwrap(),
        "2026-08-01T12:00:00Z"
    );
    assert!(
        scheduler
            .claim_eligible(
                &mut repo,
                "task://tenant-a/rollout-v2",
                2,
                "2026-08-01T11:59:00Z",
            )
            .is_err()
    );

    let mut takeover_scheduler = SchedulerService::new("worker-2", 60).unwrap();
    let takeover = takeover_scheduler
        .claim_eligible(
            &mut repo,
            "task://tenant-a/rollout-v2",
            2,
            "2026-08-01T12:01:01Z",
        )
        .unwrap();
    assert_eq!(takeover.lease_owner, "worker-2");
    assert_eq!(takeover.attempt_count, 2);
}
