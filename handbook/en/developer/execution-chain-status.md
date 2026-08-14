---
doc_id: dev.execution-chain-status
locale: en
kind: concept
audience: [developer, ai]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
    symbols: ["PeriodicSchedulerWorker", "serve_personal_loopback"]
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
fingerprint: "sha256:a24fbe111ac9ef29f335d79e1bb81044d831205bad2f6666fa04777ba099d940"
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
| Periodic daemon scheduler worker | implemented | starts only after bind/endpoint publication; one serial fixed-delay worker rejects reentry, survives pass errors, and cancels/joins on orderly exit |
| Sealed ContextRequest/View before Pi, per-body reauthorization | implemented | kernel-server scheduler_authority tests over real SQLite |
| Locked-down Pi candidate process over a one-shot private socket | implemented | pi-agent-adapter protocol/launch tests |
| Candidate admission bundle (Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT, all-or-nothing) | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead executor with persist-before-dispatch and original-key reconciliation | implemented, production-called | the periodic worker reloads WIA/candidate/Intent/persisted descriptor, rechecks its exact scheduler lease and current authorization, stages under the daemon data workspace, and enters the existing Effect protocol; interrupted leased rows query the original key and never re-dispatch |
| WorkspaceSearch executor | implemented, production-called | the production router carries the governed query from the persisted Intent and stages it into the search sink; handle-relative no-follow opens, post-open type/reparse verification, and enumeration-time visit ceilings |
| ProcessCheck executor | implemented, test-called only | immutable catalog equality is rechecked at every sink; bounded process-tree supervision remains test-called |
| WorkspaceWrite / WorkspacePatch mutation executor | implemented, production-called | the production router carries the governed payload + expected preimage from the persisted Intent and stages it into the mutation sink; handle-anchored no-follow parent/target/staging operations; per-target OS lock closes the final CAS window; streamed write preimages, bounded patch preimages, durable key-bound attempts/receipts in a store outside the approved workspace, and orphan cleanup |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, test-called only | attempted/completed state survives restart; timeout/network attempts and missing durable state reconcile `Indeterminate`, while completed key-bound receipts reconcile executed; loopback TLS proof remains in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Fixed post-state + verification-request + Loop `ACT -> VERIFY` publication | implemented, production-called | after WorkspaceRead reconciliation, one fenced SQLite transaction validates the current closed Effect and commits both append-only rows with the registered Loop transition |
| Independent verifier + continuation loop | implemented, production-called | criteria derive only from current Acceptance conditions; the registered fixed-Effect verifier emits CAS-backed evidence, persists the report, enters `VERIFY -> CONTINUE`, then checkpoint-bound one-time authority is consumed through `CONTINUE -> OBSERVE` without Task completion |
| Task candidate + acceptance authority | implemented; public C1 native-proven | the scheduler materializes/activates the governed Task, then only a latest current independent passed report, retrievable CAS evidence, unchanged fixed state, closed Effect set and the distinct daemon acceptance principal can commit the two registered Task transitions; missing report, duplicate acceptance, open Effect, superseded report, missing CAS evidence, and stale fixed post-state fail closed |
| Startup recovery | implemented | consumed handoffs reconcile; current admitted contracts idempotently repair only missing Task/Loop/Budget/scheduler prerequisites without replacing existing authority |

## Remaining production wiring gaps

The former bootstrap gap is closed in the admission path without adding a
parallel scheduler: successful `TaskApplicationService::admit` atomically
publishes the contract-named Loop and Budget beside the runnable scheduler row.
A zero-Intent row now enters the pre-admission candidate branch instead of
raising `MissingEffectBinding`; that pass returns after issuing the WIA, so it
cannot consume its own worker authority. Row-local failures are isolated and do
not abort later rows in the bounded pass. The daemon now starts one
non-reentrant, cancellable periodic worker only after bind and endpoint
publication; pass-level failures are retried and cannot prevent listening.
The remaining gaps are:

1. **Executor wiring is partial**: all six registered families have an
   assembled sink (P2-T10), so `execution_ready` still means that the binary
   contains one. The periodic worker now production-dispatches parameter-free
   WorkspaceRead, query-bearing WorkspaceSearch, and preimage-bearing
   WorkspaceWrite/Patch (query, payload, and expected preimage carried in the
   persisted Intent) through the durable Effect protocol. ProcessCheck and
   HttpFetchReadOnly still fail before Effect authorization because production
   has no separately governed supervised-process or registered-origin carrier
   for them; their sinks remain test-called only.
2. **Task completion is implemented and public C1 is native-proven**: the
   P2-T14 code reuses the registered `completion_claim` / `fixed_post_state` /
   `verification_report` / `acceptance_decision` slots; canonical decision
   bytes live in Artifact CAS and a daemon-private acceptance principal is
   distinct from worker and verifier identities. SQLite rechecks currentness
   and the complete Effect set in both transition transactions. Exact native
   `95f402d3` (merged `main@b30386be`) passed scheduler authority 57/57,
   verification executor 12/12 and Clippy. All D02 negatives pass: missing
   report/non-authority, duplicate acceptance, open Effect, superseded report,
   missing CAS, and stale fixed post-state. Other Tool request carriers remain
   unwired.

Additional cross-module nuance: scheduler closure treats
`RECONCILED/VERIFIED/VERIFY_FAILED` as closed, while management stop counts them
as pending — a deliberate conservative asymmetry to keep in mind when wiring.

When any of this changes, update this page (and
[`user/tasks-and-execution`](../user/tasks-and-execution.md) +
[`ref.capability-status`](../reference/capability-status.md)) in the same PR — the
fingerprint on this page will force the review.
