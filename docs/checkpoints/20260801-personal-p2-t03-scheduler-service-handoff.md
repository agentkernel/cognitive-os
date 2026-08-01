# P2-T03 scheduler-service handoff

- Date: 2026-08-01
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/scheduler-service`
- Branch: `lane/personal-p2-t03-scheduler-service`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Delivered slice

`SchedulerService` is the deterministic eligibility layer over the durable
SQLite scheduler repository. It:

- validates canonical wall-clock input and clamps a backwards observation to
  the worker's last trusted wall time;
- computes a positive, deterministic RFC 3339 lease expiry from the configured
  TTL;
- dispatches only via `SchedulerRepository::acquire_eligible_lease`; and
- permits a takeover only after durable expiry and at a strictly higher lease
  epoch, fencing the prior worker.

The repository now validates scheduler timestamps and atomically refuses a
non-eligible, cancelled, duplicate, or stale-epoch lease. It uses instant-aware
prechecks and SQLite `julianday` eligibility predicates so canonical fractional
timestamps are not compared lexically.

## Failure-first evidence

Before the implementation, Linux-host execution of
`cargo test -p cognitive-runtime --test p2_t03_scheduler_lease_timer --message-format short`
failed with `E0432`: `cognitive_runtime::SchedulerService` did not exist.
After implementation, the same focused suite passed **5/5**.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` (Linux host) | pass |
| `cargo test -p cognitive-runtime --test p2_t03_scheduler_lease_timer` (Linux host) | pass; 5/5 |
| `cargo test -p cognitive-store` (Linux host) | pass |
| `cargo clippy -p cognitive-runtime --test p2_t03_scheduler_lease_timer` (Linux host) | pass |
| `cargo clippy -p cognitive-store --all-targets` (Linux host) | pass |
| required CI (Ubuntu + Windows/MSVC) | not-run; requires pushed PR |

## Remaining work

- `blocked_paths`: none
- `blocked_task_ids`: none
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX
- owner: next P2-T03/P2-T04 implementation session
- next action: bind deadline, retry, step, and cost ceilings to durable
  authority facts and connect the bounded worker path to `BoundedHarness`.

## Non-claims

This slice does not add a worker, external dispatch, completion transition,
budget debit, Gate result, release claim, or Profile claim. The Rust daemon
remains the sole authority writer; scheduler clients only receive fenced lease
facts.
