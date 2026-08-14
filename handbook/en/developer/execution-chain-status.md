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
  - path: apps/kernel-server/src/personal/registered_check/mod.rs
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
  - apps/kernel-server/tests/p2_t16_registered_check.rs
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:eeae823d3ee38cb3dea59de20b0df13df3bc263dc3af24c33f4b9b966596bb4b"
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
| WorkspaceSearch / ProcessCheck executors | implemented, test-called only | immutable catalog equality is rechecked at every sink; search uses handle-relative no-follow opens, post-open type/reparse verification, and enumeration-time visit ceilings |
| RegisteredCheckRun executor | implemented, production-called | caller payload is exactly `check_id`; an immutable daemon registry fixes the current-binary helper, argv, workspace-root cwd, empty environment, timeout, output/process/write/network bounds and descriptor digest. Intent/Effect reaches durable `EXECUTING` before spawn, original-key state survives restart, and bounded output becomes CAS Evidence for the registered independent verifier |
| WorkspaceWrite / WorkspacePatch mutation executor | implemented, test-called only | handle-anchored no-follow parent/target/staging operations; per-target OS lock closes the final CAS window; streamed write preimages, bounded patch preimages, durable key-bound attempts/receipts in a store outside the approved workspace, and orphan cleanup |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, test-called only | attempted/completed state survives restart; timeout/network attempts and missing durable state reconcile `Indeterminate`, while completed key-bound receipts reconcile executed; loopback TLS proof remains in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Fixed post-state + verification-request + Loop `ACT -> VERIFY` publication | implemented, production-called | after WorkspaceRead reconciliation, one fenced SQLite transaction validates the current closed Effect and commits both append-only rows with the registered Loop transition |
| Independent verifier + continuation loop | implemented, production-called | criteria derive only from current Acceptance conditions; fixed-Effect and RegisteredCheck verifiers accept only their registered identity. RegisteredCheck revalidates exact descriptor/file digests and every safety observation from CAS Evidence; a passed report enters `VERIFY -> CONTINUE`, then checkpoint-bound one-time authority is consumed through `CONTINUE -> OBSERVE` without Task completion |
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

1. **Executor wiring is partial**: all seven registered families have an
   assembled sink (the original six in P2-T10 and RegisteredCheckRun in P2-T16),
   so `execution_ready` still means that the binary
   contains one. The periodic worker now production-dispatches parameter-free
   WorkspaceRead and `check_id`-only RegisteredCheckRun through the durable
   Effect protocol. WorkspaceSearch, WorkspaceWrite/Patch, ProcessCheck, and
   HttpFetchReadOnly still fail before Effect authorization because production
   has no separately governed payload/preimage, supervised-process, or
   registered-origin carrier for them; their sinks remain test-called only.
2. **Task completion remains separate**: production now closes
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
