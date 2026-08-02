# P2-T03 durable task-to-Effect lookup handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/durable-task-effect-lookup` (closed)
- Branch: `main`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: unchanged (`tested-local` from prior Linux-host slices)
- Normative surface: unchanged

## Delivered implementation slice

`ProtocolStore` now exposes a deterministic reverse lookup for immutable
Intent rows bound to an exact `TaskBinding`. The SQLite authority adapter
queries the existing `task_ref` and `contract_epoch` columns in Intent identity
order, so this slice requires no schema migration.

The daemon-private resolver follows that read through to the governed Effect
object before a future worker can make a closure decision. It rejects empty or
non-positive bindings, no binding, multiple bindings, row inconsistency,
missing Effects and unknown Effect states. Only `RECONCILED`, `VERIFIED`, and
`VERIFY_FAILED` classify as `Closed`; all known in-flight states classify as
`PendingReconciliation`. It does not use process exits, external receipts or
Pi output, and it neither releases a scheduler lease nor accepts a Task.

Focused regressions were added before the implementation:

- persisted M5 Intent rows can be listed by their exact task/epoch binding in
  stable identity order;
- a terminal durable Effect state is closed, an executing state stays pending,
  and an unknown state fails closed.

## Checks

- Failure-first focused commands attempted:
  - `cargo test -p cognitive-store --test m5_intent_chain`
  - `cargo test -p kernel-server scheduler_authority::tests`
  Both stopped before crate compilation because the Windows GNU linker returned
  exit 121 while linking dependency build scripts.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `pnpm run check:consistency`: passed.
- IDE diagnostics for changed files: none.
- Clippy, complete workspace tests, protected CI, and Linux-native exact-
  revision validation: not-run. Local Rust compilation remains blocked by the
  Windows GNU linker; Linux-host validation requires a separate confirmation-
  gated SSH action after this commit is remotely reachable.

## Remaining work

- `blocked_paths`: no implementation path; Linux-host validation is
  confirmation-gated.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12 and GMVP-LINUX.
- owner: next P2-T03 Lane-RUN/CTR session.
- next action: bind one unambiguous durable Effect resolution to the concrete
  worker closure and owner/epoch-fenced scheduler release operation. Preserve
  pending reconciliation and independent Task verification boundaries.

## Non-claims

No Provider, secret, service, B01 guest, remote host or external operation was
used. This slice adds no implementation-evidence level, P2 Gate, release or
Profile claim.
