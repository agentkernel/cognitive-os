---
doc_id: dev.execution-chain-status
locale: en
kind: concept
audience: [developer, ai]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick_with_store"]
  - path: apps/kernel-server/src/personal/scheduler_authority/worker.rs
  - path: apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: apps/kernel-server/src/personal/verification_executor.rs
  - path: crates/cognitive-store/src/sqlite/protocol.rs
    symbols: ["insert_intent"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
tests:
  - apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - apps/kernel-server/src/personal/tool_executor/tests.rs
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:288693d9e894b2d2dab872aed713af5949aa871fa7d532cae1a197fa1f2e2403"
non_claims:
  - This page records gaps as facts at the recorded baseline; it neither predicts schedules nor downgrades the tested components.
---

# Execution-chain status

The single most drift-sensitive handbook page. Designed chain:

scheduler lease → sealed Context → Pi candidate → candidate admission
(Intent + Effect + one-time WIA) → governed tool execution → independent
verification → verified continuation or ceiling STOP.

## What each stage has today

| Stage | Status | Evidence |
|---|---|---|
| Scheduler persistence, CAS leases, fencing, ceilings | implemented | store scheduler tests; `SchedulerService` ceiling tests |
| Task-admission scheduler bootstrap | implemented | one fenced SQLite transaction publishes TaskContract + `START` Loop + hard Budget + current-epoch runnable row; crash/duplicate/rollback negatives |
| Sealed ContextRequest/View before Pi, per-body reauthorization | implemented | kernel-server scheduler_authority tests over real SQLite |
| Locked-down Pi candidate process over a one-shot private socket | implemented | pi-agent-adapter protocol/launch tests |
| Candidate admission bundle (Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT, all-or-nothing) | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead / WorkspaceSearch / ProcessCheck executors with persist-before-dispatch and original-key reconciliation | implemented, test-called only | immutable catalog equality is rechecked at every sink; search uses handle-relative no-follow opens, post-open type/reparse verification, and enumeration-time visit ceilings |
| WorkspaceWrite / WorkspacePatch mutation executor | implemented, test-called only | handle-anchored no-follow parent/target/staging operations; per-target OS lock closes the final CAS window; streamed write preimages, bounded patch preimages, durable key-bound attempts/receipts in a store outside the approved workspace, and orphan cleanup |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, test-called only | attempted/completed state survives restart; timeout/network attempts and missing durable state reconcile `Indeterminate`, while completed key-bound receipts reconcile executed; loopback TLS proof remains in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Independent verifier seam (fixed post-state, append-only reports, CAS-backed evidence) | implemented, test-called only | verifier module tests |
| Recovery of consumed handoffs at startup | implemented | daemon startup path |

## Remaining production wiring gaps

The former bootstrap gap is closed in the admission path without adding a
parallel scheduler: successful `TaskApplicationService::admit` atomically
publishes the contract-named Loop and Budget beside the runnable scheduler row.
The remaining gaps are:

1. **One tick, no loop**: the daemon executes
   `run_private_scheduler_tick_with_store` once during startup; no periodic
   scheduler thread exists.
2. **Executors unwired**: all six registered families now have an assembled
   sink (P2-T10), so `ASSEMBLED_EXECUTOR_FAMILIES` lists all six and the
   resource projection reports each as `execution_ready`. Read that fact
   narrowly: it means *this binary contains an executor for the family*, not
   that an Agent can reach one. No `dispatch_staged_*_effect` function has a
   production caller — the sinks remain reachable only from tests until the
   periodic daemon worker dispatch is wired.
3. **Verifier unwired**: `record_independent_verification` and loop-continuation
   entry are exercised by tests only; no production route advances verification
   or Task acceptance.

Additional cross-module nuance: scheduler closure treats
`RECONCILED/VERIFIED/VERIFY_FAILED` as closed, while management stop counts them
as pending — a deliberate conservative asymmetry to keep in mind when wiring.

When any of this changes, update this page (and
[`user/tasks-and-execution`](../user/tasks-and-execution.md) +
[`ref.capability-status`](../reference/capability-status.md)) in the same PR — the
fingerprint on this page will force the review.
