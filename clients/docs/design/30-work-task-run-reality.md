# 30 — Work / Task / Run Reality (highest-priority audit)

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Question (brief §9): Phase 1/2 made **Work** the primary operational object — what does the REAL backend provide? Sources: store audit (`crates/cognitive-store`, `crates/cognitive-domain`, `specs/transitions/`) + HTTP audit (`28`). Every claim cites implementation.

---

## 1. The true Task entity (A. True task entity)

A Task is **two persisted parts sharing one identity**:

1. **Governed lifecycle row** — `governed_objects(object_id, domain='task', state, version, body_json, …)` (`crates/cognitive-store/src/sqlite/schema.rs:61-69`). `object_id == contract_id`. The body is minimal: `{contract_epoch, task_contract_digest, task_contract_id, task_ref}` (`intent_chain.rs:61-66`). **No objective text is persisted on the Task object** — the objective lives in the contract's `canonical_json`.
2. **Immutable contract chain** — `task_contracts(contract_seq, contract_id UNIQUE, task_ref, contract_epoch, user_intent_record_id, interpretation_id, accepted_by, contract_digest, canonical_json, UNIQUE(task_ref, contract_epoch))`, append-only via triggers; epoch CAS (+1 exactly) enforced in-transaction (`schema.rs:208-227`, `intent_chain.rs:158-212`).

### The real Task state machine (registered, compile-time embedded)

`specs/transitions/task.transitions.json:5-27` — **9 states**, not the 3 the HTTP surface shows:

`DRAFT (initial) · READY · ACTIVE · BLOCKED · CANDIDATE_COMPLETE · COMPLETED · FAILED · CANCELLED · ESCALATED` — terminal: COMPLETED, FAILED, CANCELLED, ESCALATED.

Guarded edges, e.g.: `DRAFT→READY` on `CONTRACT_ACCEPTED` (guards `task_contract_complete`, `acceptance_criteria_fixed`); `CANDIDATE_COMPLETE→COMPLETED` on `ACCEPTANCE_GRANTED` (guards `acceptance_authority_matches`, `verification_passed_and_current`, `fixed_post_state_unchanged`).

**DRAFT is persisted** at contract mint (`insert_draft_task_projection_in_tx`, `intent_chain.rs:56-92`).

### DESIGN ↔ IMPLEMENTATION refinement (recorded, not silently applied)

Phase-1 `03` §3 stated "Task lifecycle states in store: `ACTIVE → CANDIDATE_COMPLETE → COMPLETED`… no DRAFT state is API-visible". The store reality is the 9-state machine above (the 3 states were the *completion-path* slice cited from `continuation.rs`). Similarly, Effect stages are the registered **14-state** set (`effect.transitions.json:5-7`: PROPOSED, AUTHORIZED, DENIED, EXECUTING, EXECUTED, OUTCOME_UNKNOWN, RECONCILED, VERIFIED, VERIFY_FAILED, COMPENSATING, NOT_EXECUTED, COMMITTED, ABORTED, QUARANTINED) of which the observation plane projects a subset.

**Impact on design:** none structural. The state system (`22`) is category×verbatim-label and absorbs the full vocabularies by construction; the mapping table gains rows (e.g. `READY`→S3, `BLOCKED`→S5-with-reason, `ESCALATED`→S5, `COMPENSATING`→S3, `COMMITTED`→S1, `ABORTED`/`QUARANTINED`→S5). Verdict: REFINE (mapping extension), recorded as C11 in `38`'s addendum. **No redesign.**

### What a list can actually read

- `list?family=task` reads `list_current_task_contracts()` — current-epoch-per-task_ref rows; envelope `{id: task_ref, object_version: contract_epoch, health:"contracted", revision_digest: contract_digest, allowed_actions:["inspect"]}`, limit 64 (`resource_manager.rs:682-738`). **No state, no objective.**
- Richer per-task state exists (`governed_objects.state`) but is **not** exposed as a list; per-task it's visible via `GET /task/evidence` `lifecycle.current_state`.
- Port-level `list_objects_in_states(domain, states)` exists for recovery (`protocol.rs:218-229`) — not HTTP-exposed.

## 2. Task projection (B) / session-backed work (C) / CLI-only (D) / none (E)

| Need | Reality | Class |
|---|---|---|
| Create governed task | HTTP task channel (record/interpret/preview/admit) | **A — true entity, API-backed** |
| Per-task truth (state, transitions, evidence) | `GET /task/evidence` per task_ref | **A/B — true entity, projection-backed** |
| Inventory of tasks | envelope-only list (64) | **B — thin projection; BD-3 for depth** |
| Task control (cancel/pause/resume/retry) | no route; forbidden by inventory | **E — no current capability (BD-1)** |
| Autonomous progression | daemon-internal scheduler tick (250 ms, non-HTTP-addressable) | **D-ish — daemon-owned; observable only via projections** |

## 3. Run reality (the brief's explicit question)

**There is no first-class persisted Run entity.** Full table inventory across `crates/cognitive-store/src/` contains no run/execution table; `struct Run`/`run_id` grep finds only test fixtures.

What exists instead (the composition a Run timeline reads from — all real):

| Run facet | Real source | HTTP-reachable? |
|---|---|---|
| Lifecycle transitions (authority lane) | `events` + `transition_records` for the task's object ids → `GET /task/evidence` `lifecycle.transitions[]` | yes (`task_api.rs:1586-1783`) |
| Loop progress (phase machine) | Loop governed object, 15 phases (`loop.transitions.json`: START/OBSERVE/RESOLVE/ORIENT/DECIDE/ACT/VERIFY/CONTINUE/DIAGNOSE/WAIT/QUARANTINE/RECONCILE/ESCALATE/STOP/END); `loop_progress_facts` per iteration | **no direct HTTP** (facts surface only via observation families) |
| Checkpoints | `checkpoints` table (event high-watermark, fencing epoch) | no direct HTTP |
| Worker iterations | `worker_iteration_authorizations` / `_consumptions` (WIA) | no direct HTTP |
| Effects | Effect governed objects (14-state machine) → `GET /task/effects` | yes |
| Process/observation facts | observation plane O2/O3/O4/O5/O13 → `GET /task/observation` | yes (bounded) |
| Watch deltas | in-process watch ring → `GET /task/watch` | yes (process-local; snapshot empty) |
| AgentExecution | registered domain (9 states: CREATED→…→TERMINATED) — management-plane/conformance usage; **daemon production-path usage UNKNOWN** | no |

**Conclusion for the design:** Phase-1 DD-07 (Run = task_ref-scoped *presentation composition*) is validated as the only honest option — and it is sufficient: the dual-lane timeline's authority lane (evidence transitions) and observation lane (O-family + watch + dsh snapshot) are both HTTP-real. What wave-1 cannot show: Loop phase detail and WIA rows (no HTTP surface) — the Run timeline marks their absence as part of its bounded-coverage honesty (GapSpan "no recorded facts" covers unobserved spans).

## 4. The completion authority chain (what "verified" really means)

`complete_task_from_persisted_verification` (`task_completion.rs:199-378`) requires ALL of:

1. writer fencing epoch current; current contract epoch match;
2. verification report is the **latest** for its request, `status="passed"`, recorded at the **current fencing epoch**;
3. request/report/fixed-post-state all bind the same TaskBinding + verifier identity;
4. artifact evidence re-validated from CAS;
5. **every** task-bound Effect in `RECONCILED|VERIFIED|VERIFY_FAILED`;
6. fixed-subject Effect version unchanged;
then two atomic commits (claim → acceptance) with the acceptance decision as a daemon-authored CAS artifact (`artifact://sha256/…`).

The Evidence Block (`15` §4) renders exactly this chain's outputs (`latest_verification`, `latest_acceptance`, `durable_cursor`). **The design's evidence-first posture is fully backed.**

## 5. Preview is ephemeral (design-relevant store fact)

No preview table exists: preview computes governance context and returns it (`task_api.rs:532-566`). The admitted contract persists; the preview does not. Consequence for `15` §5 (Intent & Contract section): persisted artifacts shown = intent record → interpretation → contract chain (all digest-linked); the preview artifact renders from creation-flow context, and historically as "preview digest `p-…` (previews are ephemeral by design; the admitted contract is the durable record)". **No design change — one copy line.**

---

*Agent-side reality: `31`. Event/activity/evidence sources: `33`. What this means for sequencing: `39`; first slice: `40`.*
