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
    symbols: ["mint_task_contract", "validate_context_request_binding"]
contracts:
  - specs/schemas/task-preview-request.schema.json
  - specs/schemas/task-admit-request.schema.json
  - specs/schemas/task-contract.schema.json
tests:
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - apps/kernel-server/tests/p2_t02_task_api_watch.rs
fingerprint: "sha256:12e3d2171a8af959a8d92cff9296ab770ed5ff27ae9ab65815ccd60f8050c465"
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
| `admit` | `POST /task/admit` | recompute preview digest (`PreviewDigestMismatch` on drift) → `admit_interpretation` → `mint_task_contract` under contract-epoch CAS |
| watch | `GET /task/watch` | snapshot-first bounded stream (process-local 128-event replay; stale `resume_from` → `TASK_WATCH_RESUME_STALE`) |

Contract versioning: minting with a `context_request_ref` produces schema
`cognitiveos.task-contract/0.4` after `validate_context_request_binding` checks the
durable ContextRequest row (task/digest/type/perspective consistency); without it,
v0.3. The contract pins loop/budget IDs, allowed state domains and tools, deadline
and ceilings, and its own ID becomes the WIA namespace root.

Implemented-but-unexposed: `control` (supersession/cancel via
`supersede_task_contract`) and `query_intent` exist on the service trait with full
tests, but no Personal HTTP route calls them yet. Corrections through HTTP are
therefore not available; the fencing machinery (`INTENT_VERSION_SUPERSEDED`) is
fully tested at the kernel/store level.

Also honest: `POST /task/*` unknown paths return 200 with a "no Task API operation
matched" note (not 404), and the watch event source is process-local — no durable
event outbox is consumed by this surface yet.

Downstream native-tool staging accepts only a descriptor exactly equal to the
immutable daemon catalog entry. Executor attempts remain Effect-owned: uncertain
HTTP attempts survive restart as indeterminate (missing durable state also fails
indeterminate), and workspace mutation completion requires a durable receipt bound
to the original idempotency key in a state store outside the approved workspace.
None of these executor guarantees turns admission, a Tool receipt, or matching
workspace bytes into Task completion.
