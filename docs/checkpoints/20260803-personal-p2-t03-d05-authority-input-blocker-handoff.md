# P2-T03/D05 scheduler worker authority-input blocker

- Date: 2026-08-03
- Task / slice: `P2-T03/D05` scheduler-to-BoundedHarness worker and restart-safe recovery
- Lease: `lease/personal/P2-T03/scheduler-bounded-harness-worker-recovery` (closed as blocked)
- Branch: `lane/run-p2-t03-d05-worker-recovery`
- Latest checkpoint: `cc006c162da52372b0128c1d816b1a68fec26dd7`
- PR: #148 (Draft)
- Change class: `implementation-only`
- Normative surface: unchanged

## Checkpointed implementation and validation

The scheduler persistence path now binds durable work to immutable
`(task_ref, contract_epoch)` identity, retains v2 leases as epoch 1 through a
versioned migration, resolves durable task-to-Effect bindings, and refuses a
superseded TaskContract epoch before scheduler admission. The new focused
negative covers that stale-epoch fence.

Exact immutable Linux worktree validation used commit
`cc006c162da52372b0128c1d816b1a68fec26dd7` and passed:

```text
cargo test -p kernel-server scheduler_authority --locked
14 passed; 0 failed
```

`cargo fmt --check` and `git diff --check` passed locally. Required PR CI for
this checkpoint remains pending at the time of this handoff; it is not claimed
as passing.

## Bounded blocker

- `blocked_paths`: `apps/kernel-server/src/personal/server.rs`; daemon-owned
  scheduler lifecycle and the worker call to `BoundedHarness::drive_iteration`.
- `blocked_task_ids`: `P2-T03/D05`.
- `blocked_gate_ids`: `B02`, `B04`, `B05`, `B12` (all remain `not-run`).
- owner: product/architecture owner.
- evidence: the existing worker boundary requires `expected_loop_version`,
  `iteration`, `BudgetCharge`, `ProgressStatus`, and `evidence_refs`; no
  canonical persisted authority source currently defines these inputs or the
  selected authorized candidate. `TaskApi::admit` creates a TaskContract only;
  it does not create a Loop, Budget, Checkpoint, Intent, Effect, or scheduler
  work. A daemon supervisor must not fabricate any of them.
- next action: choose and document one canonical persisted authority source for
  the candidate, charge, progress/evidence and expected loop version, or
  explicitly revise the D05 task order/scope to place that producer before the
  worker. Then claim a new exact D05 lease and add end-to-end failure injection
  for restart, duplicate, clock and ceiling recovery.

## Non-claims

No daemon worker lifecycle was activated, no scheduler lease was dispatched by
daemon startup, and no Task acceptance or completion path changed. No Provider,
secret, privileged action, B01 guest, or external operation was used.
