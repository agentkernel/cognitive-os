---
doc_id: dev.task-pipeline
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService", "contract_preview_digest"]
  - path: crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["mint_schedulable_task_contract", "validate_context_request_binding"]
contracts:
  - specs/schemas/task-preview-request.schema.json
  - specs/schemas/task-admit-request.schema.json
  - specs/schemas/task-contract.schema.json
tests:
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - apps/kernel-server/tests/p2_t24_effect_fault.rs
fingerprint: "sha256:6a655a2ac22cd6f5ff5e424bab8391b695ae324dcd6fd2ffdb245c1cb1a3f7d4"
non_claims:
  - Admission does not start autonomous execution; that gap is documented in execution-chain-status.
---

# Task pipeline

HTTP (`TaskApi`, task channel) → `KernelTaskApplicationService` → kernel intent
chain → SQLite, with generated request/result DTOs at the wire.

| Operation | Route | Kernel composition |
|---|---|---|
| `propose` | `POST /task/intent.record` | `record_user_intent` — raw text fixed durably first |
| `clarify` | `POST /task/intent.interpret` | `record_interpretation_candidate` — status derived, never chosen |
| `preview` | `POST /task/preview` | local canonical digest over the typed draft (`cognitiveos.personal.task-contract-preview` domain); persists nothing |
| `admit` | `POST /task/admit` | recompute preview digest (`PreviewDigestMismatch` on drift) → `admit_interpretation` → one fenced contract-epoch-CAS transaction for TaskContract + `START` Loop + hard Budget + runnable scheduler row |
| evidence | `GET /task/evidence?task_ref=...` | reconstruct a bounded redacted lifecycle, Effect reconciliation class, current verification/Artifact availability, acceptance transition, and durable event cursor from SQLite authority plus Artifact CAS |
| effects | `GET /task/effects?task_ref=...` | reconstruct bounded Effect history (opaque original-key digest, stage, outcome/reconcile class, mutation count 0/1 or absent, report refs) without receipts or raw parameters |
| watch | `GET /task/watch` | snapshot-first bounded stream (process-local 128-event replay; stale `resume_from` → `TASK_WATCH_RESUME_STALE`) |

Contract versioning: minting with a `context_request_ref` produces schema
`cognitiveos.task-contract/0.4` after `validate_context_request_binding` checks the
durable ContextRequest row (task/digest/type/perspective consistency); without it,
v0.3. The contract pins loop/budget IDs, allowed state domains and tools, deadline
and ceilings, and its own ID becomes the WIA namespace root.

The admission publication is all-or-nothing in the authority SQLite file. A
late Loop/Budget/scheduler conflict rolls back the contract and event; a crash
after a successful response reopens every member. It does not create the
candidate Intent/Effect or run a Tool—the periodic worker path remains separate.
At daemon startup, the current immutable contract can reconstruct the same
bootstrap and idempotently restore only a missing Loop, Budget, or scheduler
row; existing authority is never reset.
When that admitted Task later resolves Context, the daemon loads currently
eligible Memory objects and exact Skill pins into the sealed view and writes an
append-only consumption record; a later session reuses those pins without
chat restatement, and forget/revoke/digest drift fail closed. This does not
complete the Task.

When a scheduler pass first observes that row with zero Intents, it selects the
pre-admission candidate path rather than treating the absent Effect binding as
corruption. Candidate admission may issue one WIA, but the same pass returns
without consuming it; a later pass must reload it under the scheduler lease.

Implemented-but-unexposed: `control` (supersession/cancel via
`supersede_task_contract`) and `query_intent` exist on the service trait with full
tests, but no Personal HTTP route calls them yet. Corrections through HTTP are
therefore not available; the fencing machinery (`INTENT_VERSION_SUPERSEDED`) is
fully tested at the kernel/store level.

Also honest: `POST /task/*` unknown paths return 200 with a "no Task API operation
matched" note (not 404), and the watch event source remains process-local. The
separate evidence query is restart-durable and read-only; it does not return
candidate parameters, workspace bytes, receipts, Provider/Pi content, or secrets.

Downstream native-tool staging accepts only a descriptor exactly equal to the
immutable daemon catalog entry. Executor attempts remain Effect-owned: uncertain
HTTP attempts survive restart as indeterminate (missing durable state also fails
indeterminate), and workspace mutation completion requires a durable receipt bound
to the original idempotency key in a state store outside the approved workspace.
RegisteredCheckRun adds one production carrier whose payload is only `check_id`;
its separate immutable registry fixes executable, argv, cwd, empty environment and
all process/output/write/network bounds. The result is CAS Evidence and still
requires the registered independent verifier.
A later tick can admit RegisteredCheckRun after an intermediate WorkspaceWrite
on a RegisteredCheck-terminated Task returns the Loop to `DECIDE` through
registered edges; only that check's independent verification plus acceptance
may complete the Task. Public C1 WorkspaceRead with the fixed-Effect verifier
still completes through `ACT -> VERIFY`.
None of these executor guarantees turns admission, a Tool receipt, or matching
workspace bytes into Task completion. P2-T14 keeps that boundary: a public C1
WorkspaceRead can complete only from current closed Effects, the exact fixed
post-state, the newest independent passed report, retrievable Artifact CAS
evidence, and a daemon-private acceptance principal. A missing report leaves
the Task `DRAFT`; a second acceptance after `COMPLETED` is rejected. Exact
native `22c3f502` proved the public C1 path. Open-Effect, superseded-report and
missing-CAS negatives are written; stale fixed post-state remains open, so this
is not yet a fully accepted D02 matrix.
