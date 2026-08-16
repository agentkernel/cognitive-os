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
  - path: apps/kernel-server/src/personal/campaign_observation.rs
    symbols: ["CampaignMutationObservationService", "CampaignExternalStateFixture"]
  - path: apps/kernel-server/src/personal/fault_profile.rs
    symbols: ["handle"]
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
  - apps/kernel-server/tests/p2_t16_registered_check.rs
  - apps/kernel-server/tests/p2_t24_effect_fault.rs
  - apps/kernel-server/src/personal/fault_profile.rs
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:57132739c4b3305055ad5b644e4c2f0a332ac7c7fde49dbbf74cf23d36990bed"
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
| Sealed ContextRequest/View before Pi, per-body reauthorization | implemented | kernel-server scheduler_authority tests over real SQLite; production also loads eligible Memory/Skill pins after current forget/revoke and digest revalidation, and those pins replace identical raw workspace bodies so the governed identity reaches Pi |
| Locked-down Pi candidate process over a one-shot private socket | implemented | pi-agent-adapter protocol/launch tests |
| Candidate admission bundle (Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT, all-or-nothing) | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead executor with persist-before-dispatch and original-key reconciliation | implemented, production-called | the periodic worker reloads WIA/candidate/Intent/persisted descriptor, rechecks its exact scheduler lease and current authorization, stages under the daemon data workspace, and enters the existing Effect protocol; interrupted leased rows query the original key and never re-dispatch |
| WorkspaceSearch executor | implemented, production-called | the production router carries the governed query from the persisted Intent and stages it into the search sink; handle-relative no-follow opens, post-open type/reparse verification, and enumeration-time visit ceilings |
| ProcessCheck executor | implemented, production-called | the production router stages the bounded process check; dispatch fails closed until the daemon supervised-process registry is wired (no ambient process observation) |
| RegisteredCheckRun executor | implemented, production-called | caller payload is exactly `check_id`; an immutable daemon registry fixes the current-binary helper, argv, workspace-root cwd, empty environment, timeout, output/process/write/network bounds and descriptor digest. The frozen catalog binds `c2a.repair.typescript` (descriptor_version 2, public + hidden tests) and `c2a.repair.rust`; oracle equality is file-digest, so gutting a hidden test fails even when source and public tests match. Intent/Effect reaches durable `EXECUTING` before spawn, original-key state survives restart, and bounded output becomes CAS Evidence for the registered independent verifier |
| WorkspaceWrite / WorkspacePatch mutation executor | implemented, production-called | the production router carries the governed payload + expected preimage from the persisted Intent and stages it into the mutation sink; handle-anchored no-follow parent/target/staging operations; per-target OS lock closes the final CAS window; streamed write preimages, bounded patch preimages, durable key-bound attempts/receipts in a store outside the approved workspace, and orphan cleanup |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, production-called | the production router stages the pinned HTTPS target; the registered-origin allowlist is empty by default so staging fails closed until an origin is registered; attempted/completed state survives restart; loopback TLS proof remains in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Fixed post-state + verification-request + Loop `ACT -> VERIFY` publication | implemented, production-called | after WorkspaceRead reconciliation, one fenced SQLite transaction validates the current closed Effect and commits both append-only rows with the registered Loop transition |
| Independent verifier + continuation loop | implemented, production-called | criteria derive only from current Acceptance conditions; fixed-Effect and RegisteredCheck verifiers accept only their registered identity. RegisteredCheck revalidates exact descriptor/file digests and every safety observation from CAS Evidence; a passed report enters `VERIFY -> CONTINUE`, then checkpoint-bound one-time authority is consumed through `CONTINUE -> OBSERVE` without Task completion. WorkspaceRead with the fixed-Effect verifier still publishes `ACT -> VERIFY`. On a RegisteredCheck-terminated Task, a closed intermediate WorkspaceWrite/Patch/Search Effect instead walks `ACT -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE` so a later tick can admit RegisteredCheckRun; only that check's independent verification may complete the Task |
| A7 campaign loopback external-mutation observation | implemented, test-called only | campaign-owned idempotent fixture with bounded mutate/query/reset/cleanup and durable request/query counters; persist-before-dispatch Effect; default-off authorized fault points; a response dropped after durable mutation is reconciled by querying only the original key, with one applied mutation and no second POST; independent verification is bound, `acceptance_ref` stays absent. Local/fixture evidence is not a Gate, release, Profile, B01, or EVAL-003 result |
| Public Effect history and default-off fault profiles | implemented, HTTP-called; production consult | task-channel `GET /task/effects` returns opaque original-key digest, stage, outcome/reconcile class, mutation count 0/1 or absent, and report refs without receipts or parameters; management `POST/GET /management/resource/v1/fault-profile` is default-off and campaign-authorized; task callers are denied. Production native dispatch consults the persisted profile at the four fixed points; missing, default-off, and unauthorized file content never inject. Restart queries only the original idempotency key; a replacement key cannot bind a second Intent; Indeterminate/open Effects never complete a Task |
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

1. **Executor wiring is complete across all seven registered families**: the six
   original families (P2-T10) plus RegisteredCheckRun (P2-T16) all have a
   production request carrier. The periodic worker production-dispatches
   parameter-free WorkspaceRead, query-bearing WorkspaceSearch, preimage-bearing
   WorkspaceWrite/Patch (query, payload, and expected preimage carried in the
   persisted Intent), bounded ProcessCheck, origin-gated HttpFetchReadOnly, and
   `check_id`-only RegisteredCheckRun through the durable Effect protocol.
   ProcessCheck dispatch fails closed until the daemon supervised-process registry
   is wired, and HttpFetchReadOnly staging fails closed until an origin is
   registered — neither fabricates input or bypasses the Effect protocol.
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
   A7 fixture/local evidence must not be promoted to Gate, release, Profile,
   unwired.
3. **Governed software-repair journey is one Task (P2-T22/D02)**: after a
   closed intermediate mutation on a RegisteredCheck-terminated Task the Loop
   returns to `DECIDE` through registered edges; a later tick admits
   `check_id`-only RegisteredCheckRun against the workspace capability, and
   only that check's independent verifier plus acceptance may `COMPLETED` the
   Task. Public C1 WorkspaceRead with the fixed-Effect verifier is unchanged.
   When several Intents share one contract epoch, an unconsumed WIA selects the
   current Intent instead of treating the set as ambiguous. Journey tests read
   Loop `DECIDE` from the contract-pinned Loop object. Hidden-test gutting, public-test weakening, and out-of-scope writes fail
   closed. D03 still owns the exact-revision linux-002
   restart/unknown-outcome/resource/secret/cleanup matrix.

Additional cross-module nuance: scheduler closure treats
`RECONCILED/VERIFIED/VERIFY_FAILED` as closed, while management stop counts them
as pending — a deliberate conservative asymmetry to keep in mind when wiring.

When any of this changes, update this page (and
[`user/tasks-and-execution`](../user/tasks-and-execution.md) +
[`ref.capability-status`](../reference/capability-status.md)) in the same PR — the
fingerprint on this page will force the review.
