# P2-T03 scheduler ceiling-admission handoff

- Date: 2026-08-01
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/ceiling-authority` (closed)
- Branch: `lane/personal-p2-t03-ceiling-authority`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Delivered slice

`SchedulerService` now evaluates a supplied authority-fact snapshot before a
caller begins another scheduler dispatch. It fails closed for negative facts,
parses deadline instants rather than comparing timestamp text, retains the
previous monotonic wall-clock clamp, and returns the first inclusive stop
reason across deadline, retry, step, and cost ceilings.

The slice introduces only runtime-local types:

- `SchedulerCeilingFacts`; and
- `SchedulerStopReason`.

It does not create a second authority writer, modify contracts, schemas,
transitions, registry entries, golden vectors, SQLite state, or external
dispatch behavior.

## Failure-first and Linux-host evidence

Before implementation,
`cargo test -p cognitive-runtime --test p2_t03_scheduler_ceiling_authority`
failed to compile because `SchedulerCeilingFacts`, `SchedulerStopReason`, and
`SchedulerService::evaluate_authority_ceilings` did not exist. After commits
`c18fe61` and `fb2baa8`, the following ran against a no-secret archive snapshot
on `wuz@192.168.1.2` (Linux native host, Rust 1.97.1):

| Check | Result |
|---|---|
| `cargo test -p cognitive-runtime --test p2_t03_scheduler_ceiling_authority` | pass; 2/2 |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy -p cognitive-runtime --test p2_t03_scheduler_ceiling_authority -- -D warnings` | pass |
| `cargo test -p cognitive-runtime --test p2_t03_scheduler_lease_timer` | not-run for this slice; passed 5/5 in the preceding scheduler-service slice |
| `cargo test -p cognitive-store` | not-run for this slice; passed in the preceding scheduler-service slice |
| required CI (Ubuntu + Windows/MSVC) | not-run; requires push and PR |

## Remaining work

- `blocked_paths`: none
- `blocked_task_ids`: none
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX
- owner: next P2-T03/P2-T04 implementation session
- next action: introduce the daemon-owned adapter that reloads TaskContract,
  durable progress/retry counters, and budget facts; persist a stop authority
  fact before a worker can dispatch, then connect the fenced worker path to
  `BoundedHarness` with stale-lease, cancel-propagation, no-progress, and
  ceiling-focused negative tests.

## Non-claims

This is not evidence that ceiling facts are yet loaded from durable authority
records or that a stop fact is persisted. It adds no worker, external dispatch,
Task completion transition, budget debit, Gate result, release claim, or
Profile claim. The Rust daemon remains the sole authority writer.
