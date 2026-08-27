# 38 — Phase-2 Design Challenges (design vs actual implementation)

- Phase 2.5 (audit/planning only)
- Date: 2026-08-24
- Method: the brief's ten named challenges, each checked against implementation-verified reality (`28`/`29`/`32`; store-level audit cross-cited where landed). Verdicts: **KEEP DESIGN** · **REFINE DESIGN** · **BLOCK DESIGN** · **REQUIRES BACKEND**. No silent redesign: every refinement cites where the Phase-1/2 document already anticipated the issue (or says so if it did not).

## Current implementation (frozen challenge record)

The C1–C11 findings below remain the 2026-08-24 review of the earlier design
against P7-T05 implementation.

## Personal 2.0 target delta / supersession

The closing claim below that "no Phase-1/2 design element is contradicted" does
not assess the adopted Personal 2.0 target. The new target deliberately adds
conversation, Adapter capability projection, Goal/Plan/attempt, multi-Agent,
MCP, Account Hub tiers, federated writeback and provenance Activity. Their
gaps are registered in
[Backend Dependency Matrix](37-backend-dependency-matrix.md) and all remain
`Requires-backend`. The historical challenge outcomes are preserved but cannot
be cited as target implementation readiness.

---

## C1. Work may be ahead of backend capability

**Reality:** the governed chain (record→interpret→preview→admit) and per-task projections (evidence/effects/observation/watch) are fully implemented. The **inventory** is not: `list?family=task` is envelope-only (limit 64, no objective, no state field). 
**Verdict: KEEP DESIGN + REQUIRES BACKEND (BD-3).** Phase 1 anticipated this exactly (Tier-1/Tier-2 split in `14` §1; honesty footer). No design change; the inventory's depth is backend-gated, its honesty is not.

## C2. Run may not be a real domain object

**Reality:** confirmed — no first-class persisted Run/execution listing on the operator API. What exists: task lifecycle transitions (evidence), effects, observation families, watch deltas, dsh session snapshots. 
**Verdict: KEEP DESIGN.** Phase 1 already decided this (DD-07: Run = task_ref-scoped presentation composition, dual lanes). The audit validates the decision: both lanes have real sources (`/task/evidence` transitions = authority lane; `/task/observation` + watch = observation lane). REQUIRES BACKEND only for cross-task run inventory (BD-3) and live run updates (BD-4) — neither blocks the per-task Run reading.

## C3. Evidence may not be sufficiently authoritative

**Reality:** evidence is the strongest part of the backend: digest-bound terminal evidence (`TerminalTaskEvidence` with lifecycle transitions, intent/effect refs, latest_verification with report digest + currency, latest_acceptance, durable cursor), independent-verification semantics in the store (`complete_task_from_persisted_verification`). 
**Verdict: KEEP DESIGN.** The Evidence Block (`15` §4, `23`) maps 1:1 onto real fields. No conflict.

## C4. Activity may have insufficient event coverage

**Reality:** confirmed partial — provider-plane audit exists; per-task transitions/effects exist; O13 replay exists per task; **no unified cross-domain feed; non-provider management mutations are not audited over HTTP**. 
**Verdict: KEEP DESIGN + REQUIRES BACKEND (BD-5).** Phase 2 pre-committed to honesty here (DD-11 coverage banner; per-object timelines first). The design ships truthfully in wave 1 and deepens when BD-5 lands.

## C5. Agent lifecycle may be CLI-only

**Reality:** confirmed — install/register/activate/pause/resume/stop/recover/upgrade/rollback/uninstall exist only as admin-cli verbs operating store-direct with a `PrivilegedManagementSession` file (`apps/admin-cli/src/main.rs:111-128`) plus runtime library calls. No HTTP routes. 
**Verdict: KEEP DESIGN + REQUIRES BACKEND (BD-2).** The dossier (`16`) is read-mostly by design; class-C honesty blocks with CLI paths are specified (DD-08). No fake controls exist anywhere in the design.

## C6. Provider capabilities may differ from UI assumptions

**Reality:** mostly aligned (accounts/keys/models/bindings/usage/alerts/audit all real). Differences: budgets **observe-only** (no enforcement hook); capability probe beyond discovery not exposed; usage/audit queries have **no filters**; `secret_ref` serialized (display policy needed); dsh apply has a 4 s acknowledgement wait. 
**Verdict: REFINE DESIGN (already refined).** `17` carries the advisory annotation (budgets never block), capability `not-run`, no filter UI, presence-only secret display, and the apply gate. No further change.

## C7. Session model may differ

**Reality:** sessions are in-process (restart clears all), 12 h absolute / 30 min idle, no logout/revoke route, no introspection route (expiry known only from issuance response fields), no browser-specific bootstrap flow. 
**Verdict: REFINE DESIGN (already refined).** Session chrome (`12` §5, `20` §5) shows expiry from issuance fields, states the no-revoke truth (BD-7), and keeps the memory-only gate. BD-9 (bootstrap ergonomics) is an owner+security decision, not a design assumption.

## C8. Watch/realtime may not exist

**Reality:** SSE exists on three endpoints but: task watch snapshot `tasks:[]` always empty; resource watch publishes only `projection.initialized` (inert after startup); everything is process-local (daemon restart = fresh cursors). The shipped SPA parses SSE offline from manual GETs. 
**Verdict: KEEP DESIGN + REQUIRES BACKEND (BD-4).** The design's watch states (live/stale/disconnected/reconciling/unknown) and "never fabricate finals" rules are exactly calibrated to this reality. Real streaming (EventSource) is a client refactor (`36` §1); *meaningful* live depth is backend-gated.

## C9. Resource model may be incomplete

**Reality:** two planes exist and disagree: the projection plane self-declares memory/skill/context `not-backed`, while authority-backed memory/skill reads exist on other routes; Resource Manager lists are envelope-only (limit 64); context/runtime lists return empty `projection-only`. 
**Verdict: REFINE DESIGN (already refined).** Family pages (`18`) consume the **authority-backed** routes (memory object explain, skill binding/revision explain, tool catalog) and label envelope limits; the projection plane is not used as a source of truth. A backend reconciliation of the two planes is recorded as a candidate BD item (see `37` BD-10, new).

## C10. System diagnostics may be CLI-first

**Reality:** mixed — readiness/doctor core is HTTP and live; doctor sub-sections are static placeholders over HTTP; backup/restore are HTTP; upgrade/uninstall and daemon service control are CLI-only; `doctor --bundle` is CLI-only. 
**Verdict: KEEP DESIGN.** `20` renders sub-sections in their true placeholder state, keeps stewardship on real routes, and renders upgrade/service control as CLI guidance (class-C).

---

## C11 — ADDENDUM (store-level audit): state vocabularies are deeper than the HTTP surface showed

**Reality:** the registered Task machine has **9 states** (`DRAFT, READY, ACTIVE, BLOCKED, CANDIDATE_COMPLETE, COMPLETED, FAILED, CANCELLED, ESCALATED` — `specs/transitions/task.transitions.json:5-27`; DRAFT is persisted at contract mint), and the Effect machine has **14 states** (`effect.transitions.json:5-7`). Phase-1 `03` §3 documented the completion-path slice (3 task states) and the observation-plane's projected effect subset (10 stages). 
**Verdict: REFINE DESIGN (mapping extension, zero structural change).** The state system (`22`) is category×verbatim-label by construction and absorbs the full sets; the mapping table gains rows (`READY`→S3 Waiting, `BLOCKED`→S5, `ESCALATED`→S5, `COMPENSATING`→S3, `COMMITTED`→S1, `ABORTED`/`QUARANTINED`→S5, etc.). Assigned to the implementation phase's state-system component work; verbatim labels guarantee no diagnostic loss. Also noted: **previews are not persisted** (no preview table — `task_api.rs:532-566`); Work detail §5 renders the persisted chain (record→interpretation→contract) with the preview shown as digest + "ephemeral by design" copy.

## Summary

| Verdict | Count | Items |
|---|---|---|
| KEEP DESIGN | 5 | C2, C3, C5*, C8*, C10 (*with backend deps named) |
| REFINE DESIGN (already refined in Phase-2 docs) | 3 | C6, C7, C9 |
| REFINE DESIGN (mapping/copy extension, recorded) | 1 | C11 |
| REQUIRES BACKEND (design kept; depth gated) | 4 | C1 (BD-3), C4 (BD-5), C5 (BD-2), C8 (BD-4) |
| BLOCK DESIGN | 0 | — |

**No Phase-1/2 design element is contradicted by implementation reality.** Four elements have their *depth* gated by named backend dependencies; zero require redesign. The honesty machinery (Tier splits, coverage banners, class-C blocks, S7 states) absorbed every gap the audit found — evidence that the Phase-1 capability-honesty contract was the right load-bearing decision.

---

*Backend dependencies verified one-by-one in `37-backend-dependency-matrix.md`; implementation ordering in `39`.*
