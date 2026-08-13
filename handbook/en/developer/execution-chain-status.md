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
  - path: apps/kernel-server/src/personal/campaign_observation.rs
    symbols: ["CampaignMutationObservationService", "CampaignExternalStateFixture"]
  - path: crates/cognitive-store/src/sqlite/protocol.rs
    symbols: ["insert_intent"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
tests:
  - apps/kernel-server/src/personal/p2_t17_a7_failure_first.rs
  - apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - apps/kernel-server/src/personal/tool_executor/tests.rs
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:7fed070e0df1a87f9d41bed363c8af6efea21f81a21cd4d8893c0bbab6943ad4"
non_claims:
  - This page records gaps as facts at the recorded baseline; it neither predicts schedules nor downgrades the tested components.
  - A7 campaign fixture and local/CI observation evidence never promote Gate, release, Profile, B01, or EVAL-003 results.
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
| WorkspaceWrite / WorkspacePatch mutation executor | implemented, test-called only | handle-anchored no-follow parent/target/staging operations; per-target OS lock closes the final CAS window; streamed write preimages, bounded patch preimages, durable key-bound attempts/receipts in a store outside the approved workspace, and orphan cleanup |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, test-called only | attempted/completed state survives restart; timeout/network attempts and missing durable state reconcile `Indeterminate`, while completed key-bound receipts reconcile executed; loopback TLS proof remains in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Fixed post-state + verification-request + Loop `ACT -> VERIFY` publication | implemented, production-called | after WorkspaceRead reconciliation, one fenced SQLite transaction validates the current closed Effect and commits both append-only rows with the registered Loop transition |
| Independent verifier + continuation loop | implemented, production-called | criteria derive only from current Acceptance conditions; the registered fixed-Effect verifier emits CAS-backed evidence, persists the report, enters `VERIFY -> CONTINUE`, then checkpoint-bound one-time authority is consumed through `CONTINUE -> OBSERVE` without Task completion |
| A7 campaign loopback external-mutation observation | implemented, test-called only | campaign-owned idempotent fixture with bounded mutate/query/reset/cleanup; persist-before-dispatch Effect; default-off authorized fault points; restart queries only the original key and reconciles exactly once; independent verification is bound, `acceptance_ref` stays absent. Local/fixture evidence is not a Gate, release, Profile, B01, or EVAL-003 result |
| Startup recovery | implemented | consumed handoffs reconcile; current admitted contracts idempotently repair only missing Loop/Budget/scheduler prerequisites without replacing existing authority |

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
   WorkspaceRead through the durable Effect protocol. WorkspaceSearch,
   WorkspaceWrite/Patch, ProcessCheck, and HttpFetchReadOnly still fail before
   Effect authorization because production has no separately governed
   payload/preimage, supervised-process, or registered-origin carrier for them;
   their sinks remain test-called only.
2. **Task completion remains separate**: production now closes
   `ACT -> VERIFY -> CONTINUE -> OBSERVE`, including checkpoint and one-time
   continuation authority. No report, checkpoint, continuation, or A7
   campaign observation completes a Task; acceptance remains P2-T14 scope.
   A7 fixture/local evidence must not be promoted to Gate, release, Profile,
   B01, or EVAL-003 campaign results.

Additional cross-module nuance: scheduler closure treats
`RECONCILED/VERIFIED/VERIFY_FAILED` as closed, while management stop counts them
as pending — a deliberate conservative asymmetry to keep in mind when wiring.

When any of this changes, update this page (and
[`user/tasks-and-execution`](../user/tasks-and-execution.md) +
[`ref.capability-status`](../reference/capability-status.md)) in the same PR — the
fingerprint on this page will force the review.
