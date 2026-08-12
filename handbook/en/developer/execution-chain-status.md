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
tests:
  - apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - apps/kernel-server/src/personal/tool_executor/tests.rs
fingerprint: "sha256:8f94f4031710d5d6e3e7a5e5a1a4076a2293f281514fef017d48856ce85f0137"
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
| Sealed ContextRequest/View before Pi, per-body reauthorization | implemented | kernel-server scheduler_authority tests over real SQLite |
| Locked-down Pi candidate process over a one-shot private socket | implemented | pi-agent-adapter protocol/launch tests |
| Candidate admission bundle (Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT, all-or-nothing) | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead / WorkspaceSearch / ProcessCheck executors with persist-before-dispatch and original-key reconciliation | implemented, test-called only | `tool_executor/tests.rs` |
| WorkspaceWrite / WorkspacePatch mutation executor: expected-preimage compare-and-swap, staging-file plus rename publish, reconciliation by re-reading the target | implemented, test-called only | `tool_executor/tests.rs` |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, test-called only | `tool_executor/tests.rs`; loopback TLS proof in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Independent verifier seam (fixed post-state, append-only reports, CAS-backed evidence) | implemented, test-called only | verifier module tests |
| Recovery of consumed handoffs at startup | implemented | daemon startup path |

## The four wiring gaps (verified at the baseline)

1. **No bootstrap row**: Task admission persists contract + context + policy but
   inserts no scheduler row. Rows are created by `ProtocolStore::insert_intent`
   (same transaction as a task-bound Intent) — which is only reached from
   candidate admission, which itself requires an existing leased row. Production
   callers of `SchedulerRepository::upsert`: none (tests + benchmark only).
2. **One tick, no loop**: the daemon executes
   `run_private_scheduler_tick_with_store` once during startup; no periodic
   scheduler thread exists.
3. **Executors unwired**: all six registered families now have an assembled
   sink (P2-T10), so `ASSEMBLED_EXECUTOR_FAMILIES` lists all six and the
   resource projection reports each as `execution_ready`. Read that fact
   narrowly: it means *this binary contains an executor for the family*, not
   that an Agent can reach one. No `dispatch_staged_*_effect` function has a
   production caller — the sinks are reachable only from tests and, once gaps 1
   and 2 close, from the daemon's own worker path.
4. **Verifier unwired**: `record_independent_verification` and loop-continuation
   entry are exercised by tests only; no production route advances verification
   or Task acceptance.

Additional cross-module nuance: scheduler closure treats
`RECONCILED/VERIFIED/VERIFY_FAILED` as closed, while management stop counts them
as pending — a deliberate conservative asymmetry to keep in mind when wiring.

When any of this changes, update this page (and
[`user/tasks-and-execution`](../user/tasks-and-execution.md) +
[`ref.capability-status`](../reference/capability-status.md)) in the same PR — the
fingerprint on this page will force the review.
