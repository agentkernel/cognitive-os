---
doc_id: dev.execution-chain-status
locale: en
kind: concept
audience: [developer, ai]
status: partial
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
    symbols: ["PeriodicSchedulerWorker", "serve_personal_loopback"]
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick_with_store"]
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/worker.rs
  - path: personal/apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: personal/apps/kernel-server/src/personal/registered_check/mod.rs
  - path: personal/apps/kernel-server/src/personal/verification_executor.rs
  - path: personal/apps/kernel-server/src/personal/campaign_observation.rs
    symbols: ["CampaignMutationObservationService", "CampaignExternalStateFixture"]
  - path: personal/apps/kernel-server/src/personal/fault_profile.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/pinned_https.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/observation.rs
    symbols: ["handle"]
  - path: personal/crates/cognitive-store/src/sqlite/protocol.rs
    symbols: ["insert_intent"]
  - path: personal/crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: personal/crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
tests:
  - personal/apps/kernel-server/src/personal/p2_t17_a7_failure_first.rs
  - personal/apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - personal/apps/kernel-server/src/personal/tool_executor/tests.rs
  - personal/apps/kernel-server/tests/p2_t16_registered_check.rs
  - personal/apps/kernel-server/tests/p2_t24_effect_fault.rs
  - personal/apps/kernel-server/tests/p2_t25_tool_lifecycle.rs
  - personal/apps/kernel-server/tests/p2_t26_observation_plane.rs
  - personal/apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - personal/apps/kernel-server/src/personal/fault_profile.rs
  - personal/crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:204fbdf51e36c220ddafada6b67182f7b6b918cd34ed8cacd87cb7ae2d0a98d3"
non_claims:
  - This page records gaps as facts at the recorded baseline; it neither predicts schedules nor downgrades the tested components.
  - A7 campaign fixture and local/CI observation evidence never promote Gate, release, Profile, B01, or EVAL-003 results.
---

# Execution-chain status

P11-T08 Routine occurrences reuse this daemon scheduler (`scheduler_entries` via `task://personal/routine/{occurrence_id}`). There is no second Temporal scheduler.

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
| WorkspaceWrite / WorkspacePatch mutation executor | implemented, production-called | the production router carries the governed payload + expected preimage from the persisted Intent and stages it into the mutation sink; handle-anchored no-follow parent/target/staging operations; per-target OS lock closes the final CAS window; streamed write preimages, bounded patch preimages, durable key-bound attempts/receipts in a store outside the approved workspace, and orphan cleanup. Expected preimage `digest:sha256:<64 hex>` may name either the domain-tagged workspace-image digest or the SHA-256 of the raw file bytes (sha256sum / P-arm form). Independent verification starts only after the Effect is RECONCILED |
| HttpFetchReadOnly executor over the single audited Rustls boundary (GET only at dispatch; no caller headers, no redirects, no inherited proxy, registered origins) | implemented, production-called | the production router stages the pinned HTTPS target using the task/campaign origin registry; the allowlist is empty by default so staging fails closed until management pins an exact HTTPS origin (`host` or `host:port`); attempted/completed state survives restart; loopback TLS proof remains in `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| Public pinned HTTPS origin registry | implemented, HTTP-called; production consult | management `GET/POST /management/resource/v1/http-origin` is campaign-authorized (`P2-T25` or `PERSONAL-PERF-EVAL-*`); task callers are denied. Pins never carry credentials, headers, or bodies. Production HttpFetchReadOnly consults the registry by Intent `task_ref` |
| Fixed post-state + verification-request + Loop `ACT -> VERIFY` publication | implemented, production-called | after WorkspaceRead reconciliation, one fenced SQLite transaction validates the current closed Effect and commits both append-only rows with the registered Loop transition |
| Independent verifier + continuation loop | implemented, production-called | criteria derive only from current Acceptance conditions; fixed-Effect and RegisteredCheck verifiers accept only their registered identity. RegisteredCheck revalidates exact descriptor/file digests and every safety observation from CAS Evidence; a passed report enters `VERIFY -> CONTINUE`, then checkpoint-bound one-time authority is consumed through `CONTINUE -> OBSERVE` without Task completion. WorkspaceRead with the fixed-Effect verifier still publishes `ACT -> VERIFY`. On a RegisteredCheck-terminated Task, a closed intermediate WorkspaceWrite/Patch/Search Effect instead walks `ACT -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE` so a later tick can admit RegisteredCheckRun; only that check's independent verification may complete the Task |
| Personal 2.0 Attempt artifact verifier (`verifier://personal/attempt-artifact`) + last-ring acceptance | implemented, production-called (P13-T04) | the broker thread that writes a hosted Attempt terminal hands each `DeliverableDraft` candidate to the same daemon CAS and runs this registered verifier identity under `principal://personal/independent-verifier`: deterministic re-reads only (CAS digest, source-frame binding, terminal Attempt, UTF-8 / non-empty / no secret shape); the child's `response done`, exit code and prose are recorded `not-used`; evidence is append-only with its report in the CAS. P11-T03 StageTestPassed is derived from that evidence plus real seating and a CAS re-read (no caller `passed`); run acceptance is a `run-acceptance` ApprovalPreview refused off the last ring; nothing here touches the core Task/Effect verifier paths above or completes a core Task |
| A7 campaign loopback external-mutation observation | implemented, test-called only | campaign-owned idempotent fixture with bounded mutate/query/reset/cleanup and durable request/query counters; persist-before-dispatch Effect; default-off authorized fault points; a response dropped after durable mutation is reconciled by querying only the original key, with one applied mutation and no second POST; independent verification is bound, `acceptance_ref` stays absent. Local/fixture evidence is not a Gate, release, Profile, B01, or EVAL-003 result |
| Public Effect history and default-off fault profiles | implemented, HTTP-called; production consult | task-channel `GET /task/effects` returns opaque original-key digest, stage, outcome/reconcile class, mutation count 0/1 or absent, and report refs without receipts or parameters; management `POST/GET /management/resource/v1/fault-profile` is default-off and campaign-authorized; task callers are denied. Production native dispatch consults the persisted profile at the four fixed points; missing, default-off, and unauthorized file content never inject. Restart queries only the original idempotency key; a replacement key cannot bind a second Intent; Indeterminate/open Effects never complete a Task |
| Public Tool lifecycle, Agent exposure, and selection receipts | implemented, HTTP-called | management `GET/POST /management/resource/v1/tool*` overlays enabled/disabled/quarantined/revoked without mutating descriptor digests; `agent_exposed` follows overlay plus assembled-executor readiness; task callers cannot mutate lifecycle. `GET /task/resource/v1/tool/exposure` returns the least exposure set and digest; `POST /task/resource/v1/tool/selection` records a receipt only for that digest and an exposed operation_id. Prompt/body/receipt restatement and stale/widened candidate digests fail closed |
| Task candidate + acceptance authority | implemented; public C1 native-proven | the scheduler materializes/activates the governed Task, then only a latest current independent passed report, retrievable CAS evidence, unchanged fixed state, closed Effect set and the distinct daemon acceptance principal can commit the two registered Task transitions; missing report, duplicate acceptance, open Effect, superseded report, missing CAS evidence, and stale fixed post-state fail closed |
| Startup recovery | implemented | consumed handoffs reconcile; current admitted contracts idempotently repair only missing Task/Loop/Budget/scheduler prerequisites without replacing existing authority |

## Remaining production wiring gaps

The former bootstrap gap is closed in the admission path without adding a
parallel scheduler: successful `TaskApplicationService::admit` atomically
publishes the contract-named Loop and Budget beside the runnable scheduler row.
Public `POST /task/admit` also persists daemon-owned Context authorization facts
and the tenant `personal` revocation epoch. A zero-Intent row now enters the
pre-admission candidate branch instead of raising `MissingEffectBinding`; that
first tick walks Loop `START -> DECIDE` from the sealed ContextView, admits one
private Pi candidate, and returns after issuing the WIA, so it cannot consume
its own worker authority or acquire a scheduler lease. A later tick reloads the
durable WIA under a lease and activates the Task. Row-local failures are isolated and do
not abort later rows in the bounded pass. The daemon now starts one
non-reentrant, cancellable periodic worker only after bind and endpoint
publication; pass-level failures are retried and cannot prevent listening.
Live HTTP `TaskApi` clones the daemon-owned `SqliteAuthorityStore` handle so that
tick sees the Context facts admit persisted; opening a second writer per request
is the EVAL-006 skip. A contract `max_retries` of 0 still allows the first
scheduler dispatch: retry count 0 is not a reached ceiling, so the later WIA
tick can acquire a lease instead of calling `stop_for_ceiling` with no
checkpoint. The remaining gaps are:

1. **Executor wiring is complete across all seven registered families**: the six
   original families (P2-T10) plus RegisteredCheckRun (P2-T16) all have a
   production request carrier. The periodic worker production-dispatches
   parameter-free WorkspaceRead, query-bearing WorkspaceSearch, preimage-bearing
   WorkspaceWrite/Patch (query, payload, and expected preimage carried in the
   persisted Intent), bounded ProcessCheck, origin-gated HttpFetchReadOnly, and
   `check_id`-only RegisteredCheckRun through the durable Effect protocol.
   ProcessCheck dispatch fails closed until the daemon supervised-process registry
   is wired, and HttpFetchReadOnly staging fails closed until a campaign-authorized
   origin is pinned — neither fabricates input or bypasses the Effect protocol.
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
O2/O3/O4/O5/O13 observation is a task-channel read plane
(`GET /task/observation?family=o2|o3|o4|o5|o13&task_ref=…`). O2–O4 samples are
daemon-authored redacted receipts in `personal-observation-plane.json`. O5
reuses bounded `GET /task/effects` history. O13 is durable audit cursor/replay
with fail-closed stale-cursor, missing-event, digest-break, and gap handling.
Empty collectors return `observed_zero` with a named negative control. This is
not a second authority API and does not promote Gate, release, Profile, B01,
or EVAL results.

When any of this changes, update this page (and
[`user/tasks-and-execution`](../user/tasks-and-execution.md) +
[`ref.capability-status`](../reference/capability-status.md)) in the same PR — the
fingerprint on this page will force the review.
