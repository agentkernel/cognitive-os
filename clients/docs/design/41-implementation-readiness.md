# 41 — Implementation Readiness (Phase 2.5 final)

- Phase 2.5 (audit/planning only — no implementation authorized, no code changed, no commits)
- Date: 2026-08-24
- Evidence base: six read-only audits this phase + Phase-1/2 design contract. Audits: WebUI code (`pc/web` @ `0320c1a`/`db56374`), daemon HTTP surface (`apps/kernel-server/src/personal/`), store/domain/runtime internals (`crates/cognitive-store|domain|runtime|kernel`, `specs/transitions/`), TS layer (`packages/{contracts-ts,sdk-ts,pi-cognitiveos,dsh-akp-adapter}` + clients repo outside `pc/web`), product/plan docs, governance/lease state.

---

## 1. Repository Reality

Two repos (`26`): **A `agentkernel/cognitive-os`** = all authority (Rust daemon, contracts, store, CLI; serves the static bundle at `/ui/`); **B `agentkernel/cognitiveos-clients`** = the SPA at `pc/web/` (the only code in B; `shared/` is docs-only; `pc/app/` is a blocked stub; `apps/cognitiveos-console` in A is a deprecated stub that must not be revived). Governance: kernel changes via PR + required CI; contract changes via Lane-CTR; **client-repo automation write access is currently blocked (cursor[bot] 403 — owner remediation pending)**.

## 2. Current Web Architecture

React 18.3.1 + TS 5.6.3 strict + Vite 5.4.11, HashRouter, pnpm-pinned; single-file view layer (`App.tsx` ~1485 lines, 10 pages); **no component library, no tokens, no store**; hand-rolled fetch with channel-scoped memory-only bearers; strong tested logic layer (`channels/session/policy/probe/watch/identities/taskDraft`); raw-JSON-dominant presentation; manual refresh only; 9 unit/DOM test files (no network/page tests). Full map: `27`; per-page audit: `34`.

## 3. Current Backend Capability

Daemon authority chain fully implemented: intent record → interpretation → preview → admit (CAS+principal) → scheduler → candidates → effects (14-state machine) → independent verification → acceptance (CAS artifact) → completion. Provider control plane complete (accounts/keys/models/bindings/usage/budgets/alerts/audit). Resources: memory remember/forget/explain, skill import/bind/revoke/explain, tool catalog/lifecycle/exposure/selection. Backup/restore. Observation plane O2–O13. Readiness/doctor core. **Gaps:** task control verbs, agent lifecycle, rich task inventory, live watch deltas, unified audit, memory search — all HTTP-absent (BD register, `37`).

## 4. Current API Capability

~60 verified routes (`28`). Channel-scoped bearer auth (task/management), loopback-only, no cookies/CORS, Origin allowlist. Three error-envelope shapes (R-2); 200-stub fallthrough on unmatched management/task routes (R-1); `secret_ref` serialized in account responses (R-5, display-policy handled); SSE exists but process-local with empty task snapshot and inert resource watch (R-3/R-4).

## 5. Work / Task / Run Reality

**Task:** real, 9-state persisted machine (DRAFT→READY→ACTIVE→BLOCKED→CANDIDATE_COMPLETE→COMPLETED/FAILED/CANCELLED/ESCALATED) + immutable contract epoch chain; list is envelope-only (limit 64, no state/objective — BD-3); per-task truth via `GET /task/evidence`. **Run:** **no first-class entity** — composed from evidence transitions + observation families + watch + effects (DD-07 validated as the only honest and sufficient model). **Preview:** ephemeral (not persisted) — digest-bound into admission. Full detail: `30`.

## 6. Agent Reality

Eight identities classified (persistent/runtime/observation) in `31`. Lifecycle verbs (install/register/activate/pause/resume/stop/recover/quarantine-root/health/upgrade/rollback/uninstall) are **CLI + library only — zero HTTP routes** (BD-2). HTTP-visible agent facts: runtime inventory/inspect envelopes, provider bindings, dsh runtime snapshot. Dossier design is read-mostly with class-C honesty — fully consistent with reality.

## 7. Provider Reality

The deepest, most complete domain (`32` §3): all governance mutations API-real with CAS and trust reconfirm; secrets one-way into SecretStore (never read back); budgets observe-only (BD-8, design renders advisory); usage/audit unfiltered; capability probe beyond discovery not exposed (renders not-run). **Zero design conflicts.**

## 8. Evidence / Activity Reality

Evidence chain is the backend's strongest asset (contracts → fixed post-states → verification requests/reports → effects closed → CAS acceptance → terminal transition; `33` §3). Events: one durable append-only log, 6 closed types, **library-unified but not HTTP-unified**; memory/skill/tool/backup mutations emit no events (BD-5); provider plane has its own audit table. Activity design (honest composition + coverage banner) matches reality exactly.

## 9. Session Reality

In-process channel sessions (12 h absolute / 30 min idle), bootstrap-secret issuance, memory-only browser custody, no logout/introspection routes (BD-7), no browser-specific bootstrap (BD-9, owner+security decision). UI session ≠ authority session — never conflated; principal binding enforced server-side.

## 10. Design Conflicts (DESIGN ↔ IMPLEMENTATION register)

| # | Conflict | Severity | Resolution |
|---|---|---|---|
| DC-a | Phase-1 `03` documented 3 task states / 10 effect stages; store reality is 9/14 registered states | low | REFINE — mapping-table extension (`38` C11); state system absorbs by construction (verbatim labels) |
| DC-b | Phase-2 `15` §5 implied preview artifacts viewable later; previews are ephemeral | low | copy line: "previews are ephemeral by design; the admitted contract is the durable record" (`30` §5) |
| DC-c | Home "current work" assumed inventory; inventory is envelope-only | medium | Tier-1/Tier-2 split + honesty footer (already designed, `14`) — BD-3 for depth |
| DC-d | Activity unified feed assumed; no unified HTTP feed | medium | coverage banner + per-object timelines (already designed, `19`) — BD-5 for depth |
| DC-e | Agent "current work" assumed; Pi sidecar state not HTTP-exposed | medium | S7 honest state (already designed, `16`) — BD-2 |

**No conflict requires redesign. Zero BLOCK verdicts.**

## 11. Frontend-only Work (no backend change needed)

Shell; tokens; state system; data/projection layer; error normalization + route whitelist; Providers complete; Home (Tier-1); Work inventory Tier-1 + creation flow; Work detail (all six sections, per-task_ref); Run timeline (composed); Agents dossier (honest depth); Resources family pages; Activity (honest composition); System; command palette; watch EventSource refactor; accessibility pass.

## 12. Backend-required Work (design depth gated, not blocked)

BD-3 task inventory projection (Work Tier-2) · BD-4 watch deltas (live depth) · BD-5 unified audit (Activity feed) · BD-2 agent lifecycle HTTP (dossier depth/controls) · BD-1 task control HTTP (intervention) · BD-6 memory search/review · BD-7 session logout/introspection · BD-10 projection-plane reconciliation (hygiene).

## 13. Architecture-required Work

None for the approved design. No contract change, no new route family, no architectural refactor is required for waves 1–12 (`39`). BD items are additive routes/projections through normal Lane-CTR process, not architecture changes.

## 14. Blocked Work

| Blocker | What it blocks | Owner/action |
|---|---|---|
| **Client-repo write access (cursor[bot] 403)** | publishing ANY client change (all waves' PRs) | owner remediation (P7-T05/D10 register) or owner-run publication |
| BD-3 | Work inventory Tier-2 (Tier-1 ships) | owner schedules backend slice |
| BD-1/BD-2 | intervention verbs, agent controls (design ships class-C honest) | owner schedules; bigger contract work |
| OQ-6 (data-layer approach) | Wave 1 start | implementation-phase decision (recommendation recorded) |
| OQ-1 (labels Work/Tasks, System) | sidebar labels | owner; can trail Wave 1 |

## 15. Refactor vs Rewrite Decision

**Option A — incremental frontend refactor with page-level replacement** (`36`): keep toolchain + tested logic layer; replace view layer page-by-page onto new foundation; no rewrite of security/honesty logic; no API changes; no new app.

## 16. Recommended First Implementation Slice

**"Foundation + Providers"** (`40`): shell + tokens + state system + data layer + the complete Providers space end-to-end (the deepest real domain, zero backend gates, hardest honesty case, highest logic reuse, executes the DD-04 Bindings fold). Other spaces ship as honest placeholders.

## 17. Phase 3 Implementation Waves

`39`: W0 governance/repo prep → W1 foundation → W2 Providers → W3 Home → W4 Work inventory + creation → W5 Work detail + Run timeline → W6 Agents → W7 Resources → W8 Activity → W9 System → W10 command layer → W11 watch streaming → W12 a11y/QA gate. Backend tracks (BD-*) run in parallel, owner-scheduled.

## 18. Validation plan (respecting repo rules)

| Layer | Plan | Environment rule |
|---|---|---|
| Frontend unit | keep 9 existing suites; add policy/state/token unit tests | local Windows OK (Node/pnpm) |
| Frontend integration | network-layer tests with mock fetch (channel binding, stub detection, error normalization, redaction negatives) | local OK |
| Component/a11y | Vitest + jsdom per `23` contract (keyboard, focus, ARIA, state non-color redundancy) | local OK |
| API contract | no new API in waves 1–12; route-whitelist tests against the frozen inventory | local OK |
| E2E journeys | `07` flows against daemon-served `/ui/` bundle | **exact-revision Linux** (`DEV-LINUX-NATIVE-01`) per ADR-0053 §6 |
| State transition | state-system mapping tests incl. full 9/14-state vocabularies (C11) | local OK |
| Error/honesty | 200-stub detection, three-envelope normalization, secret-shape negatives | local OK |
| Security | redaction/storage negatives; CSP; Origin rules | existing kernel CI + Linux route |
| Rust (if BD work lands) | build/test/clippy | **never on this Windows GNU host** (`RUST-LINK-DEV-WIN-GNU-01`) — CI or exact-revision Linux only |
| Visual regression | decided in implementation phase (tooling choice is OQ-adjacent) | — |

## 19. Risks

1. **Client-repo 403** turns all waves into local-only work until resolved (highest practical risk).
2. **Data-layer scope creep** (OQ-6): the projection store must stay minimal; TanStack Query fallback recorded.
3. **Dashboard drift on Home** under implementation pressure — DD-03 guardrail + review gate.
4. **BD-3 timing**: Work waves land better with it; without it, Tier-1 honesty footer must survive contact with reviewers.
5. **Two-plane inconsistency (BD-10)** could mislead future consumers if family pages accidentally read the projection plane — traceability (`35`) pins authority-backed routes.
6. **Scope creep into backend** ("just add one route") — every route addition is Lane-CTR work; the BD register is the only path.

## 20. Open Owner Decisions

OQ-1 labels (Work vs Tasks; System space) · OQ-2 refresh/polling policy · OQ-3 browser diagnostics export · OQ-4 final hues + contrast verification · OQ-5 session expiry UX · OQ-6 data-layer approach · BD scheduling (BD-3 first recommended).

---

## FINAL GATE

**DESIGN STATUS: PASS.**
Phase 1 + Phase 2 survive contact with the real repositories: 0 blocking conflicts, 5 recorded refinements (DC-a..DC-e), all absorbed by the designed honesty machinery. The design never requires an invented API, a fake capability, or a forbidden route.

**IMPLEMENTATION STATUS: PARTIALLY READY.**

- **READY (frontend-only set):** waves 1–12 as scoped in `39` are fully specified and backend-supported at their designed honesty tiers. First slice defined (`40`).
- **BLOCKED (publication):** client-repo write access (cursor[bot] 403) — owner remediation required before any client PR can land.
- **BACKEND-GATED (depth, not design):** BD-1, BD-2, BD-3, BD-4, BD-5, BD-6, BD-7, BD-9, BD-10 — each verified against code, each with its design already shipping honestly without it.

No hidden assumptions. No invented API. No fake capability. No code written. No packages installed. No commits. No push.

**FINAL STATUS: READY FOR PHASE 3 IMPLEMENTATION** — contingent on (1) owner resolution of the client-repo write access, and (2) owner sign-off on the open decisions (OQ-1..OQ-6). Backend-gated depth items are scheduled separately through the BD register.
