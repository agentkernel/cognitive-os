# P2-T03 durable scheduler handoff

- Date: 2026-08-01
- Task: P2-T03 durable scheduler、lease 与 timer
- Lease: `lease/personal/P2-T03/durable-scheduler`
- Branch: `lane/personal-p2-t03-durable-scheduler`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Delivered slice

The durable scheduler persistence layer over the authority SQLite database:

- `scheduler_entries` table (migration v2, appended after the P1-T01 v1 full
  schema) with task_ref, state, lease owner/epoch/expiry, next-eligible,
  attempt count and cancel flag;
- `SchedulerRepository` in `crates/cognitive-store/src/scheduler.rs`:
  - `upsert` — insert/replace one row;
  - `acquire_lease` — transactional CAS: refuses a duplicate/leased owner and
    a cancelled task, advances `attempt_count`, returns the updated row;
  - `release_lease` — owner-bound, fails closed on mismatch, makes the task
    runnable again for takeover;
  - `load` / `request_cancel` — read-only projection and durable cancel.
- `authority_migration_plan` now contains v1 + v2.

No second authority exists: state transitions stay in the kernel; the
scheduler repository only persists runnable/lease facts behind product-owned
SQL.

## Evidence

| Check | Result |
|---|---|
| `cargo test -p cognitive-runtime --test p2_t03_scheduler_lease_timer` (Linux host) | pass; 4/4 |
| `cargo test -p cognitive-store` full suite | pass; migration suite `p1_t01_layout_migrations` 7/7 |
| `cargo clippy -p cognitive-store --all-targets` | pass |
| `cargo clippy -p cognitive-runtime --test p2_t03_scheduler_lease_timer` | pass |
| `cargo fmt --all -- --check` | pass |
| Required CI (Ubuntu + Windows/MSVC) | pass |
| PR | [#128](https://github.com/agentkernel/cognitive-os/pull/128) merged as `main@f3bacbe` |

## Coverage of P2-T03 acceptance bullets

- crash-safe takeover: `release_lease` is owner-bound and fails closed;
  rows survive reopen (`scheduler_rows_survive_reopen_like_a_crash_replay`);
- duplicate lease impossible: `acquire_lease` CAS refuses a leased task
  (`lease_acquire_is_exclusive_and_rejects_duplicate_owner`);
- cancel request durable and blocks re-dispatch
  (`cancel_request_blocks_future_lease_acquisition`);
- attempt accounting: `attempt_count` advances per acquire.

The timer/next-eligible and wall/monotonic clock policy, plus the budget
ceiling enforcement that stops work and records authority facts, remain for
the scheduler service slice that consumes this repository (P2-T03 continued
or the worker slice P2-T04); clock-shift no-double-dispatch tests are
not-run in this repository-only slice.

## Non-claims

No P2 acceptance Gate (B02/B04/B05/B12) result, no Profile claim, no release
claim. The scheduler repository is not an authority writer; it records
durable facts consumed by product-owned service code.
