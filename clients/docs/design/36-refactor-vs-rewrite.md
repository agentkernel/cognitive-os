# 36 — Refactor vs Rewrite (per frontend subsystem)

- Phase 2.5 (audit/planning only)
- Date: 2026-08-24
- Evidence: `27` (architecture map), `34` (per-page audit), `35` (traceability). Evaluation axes: coupling, state management, router, API abstraction, component reuse, test coverage, design-system compatibility, migration risk. **"Rewrite" is rejected as a default; every verdict is argued from evidence.**
- Verdicts: KEEP (use as-is) · REFACTOR (same code, improved structure) · REPLACE (new implementation of the same responsibility) · REWRITE (discard subsystem, rebuild its responsibility from scratch) · NEW (does not exist).

---

## 1. Verdict matrix

| Subsystem | Current reality | Verdict | Why |
|---|---|---|---|
| Toolchain (pnpm/Vite/TS strict/React 18/HashRouter) | modern, pinned, ADR-0053-sanctioned | **KEEP** | no evidence of inadequacy; HashRouter retained per DD-13 (no daemon SPA fallback) |
| `api.ts` readJson + header-injection guards | small, correct, tested (negatives) | **KEEP + extend** | add: stub-note detection (R-1), error normalization (R-2) |
| `channels.ts` path→channel classification + bearer injection | correct, tested, security-critical | **KEEP** | the channel map needs extension only if new route families are consumed |
| `session.ts` memory-only store + self-check | correct, tested, ADR-compliant | **KEEP** | add expiry introspection display data (fields already in issuance response) |
| `policy.ts` (redaction, CAS gates, dispatch/apply gates, cost honesty, escaping) | correct, well-tested, encodes real product rules | **KEEP** | this is the app's best code; the redesign depends on it |
| `probe.ts` (probe classification, trust gate) | correct, tested | **KEEP** | feeds Provider flows as-is |
| `taskDraft.ts` (uuidV7 + draft builder) | correct but hardcoded to workspace-search | **KEEP + generalize carefully** | draft generalization only as contract support widens; do not fake breadth |
| `watch.ts` + `watchSse.ts` | correct state machine + parser, **never opened as a stream** | **REFACTOR** | wire real `EventSource`; keep controller semantics (live/stale/disconnected/reconciling/unknown) and resume-stale handling |
| `identities.ts` (9-identity merge) | correct, matches canonical identity model | **KEEP** | reused by the Agent dossier overview |
| Routing (`App.tsx:1400-1484`) | HashRouter, 10 routes, no `*` route | **REFACTOR** | new route map per `06`; add designed 404; keep HashRouter; move routes out of the monolith |
| App shell (`Shell`, sidebar) | minimal flat sidebar | **REPLACE** | new shell per `12` (status strip, 7 spaces, counts, command trigger); current one cannot express the IA |
| `RequireSession` + `SessionForm` | inline gate pattern (sidebar-fix branch) — correct UX direction | **KEEP pattern, REFACTOR** into shell session layer | gate-over-destination is the designed behavior |
| `StateNote` + `LoadState` | single muted text line for 7 states | **REPLACE** | superseded by the state system (`22`): category×label components; `LoadState` union survives as the *loading* facet model |
| `JsonPanel` | the dominant presentation component | **REPLACE as default; KEEP as inspector "Raw projection" tab** | DD-10: raw access has debugging value; it is demoted, not deleted |
| Pages: Home / Agents / AgentDetail / Tasks / Activity / Resources | JSON-panel-driven, load-once | **REPLACE (page-level)** | their *content models* are wrong (raw JSON, no inventory, mis-scoped Activity), not merely their styling — see §2 why this is not a "rewrite" |
| Page: Providers (list+detail) | the most complete real forms/tables | **REFACTOR** | content model is right; restructure into the five-section governance detail; reuse trust-gate/key-handoff/probe logic |
| Page: Bindings | functionally complete with strong gates | **REPLACE as destination; KEEP logic** | DD-04 fold: `bindingRevisionForCas`/`acceptDshApply`/`dispatchAllowed` move with the flow into Providers/Agent contexts |
| State management (none) | per-page `useState` | **REPLACE with a minimal query/projection layer** | see §3 — the absence of a data layer is the root cause of load-once/manual-refresh behavior |
| Component library (none) | inline HTML per page | **NEW** | build the `23` taxonomy (Foundation→Evidence) as the component layer |
| Design tokens (none) | 155-line global CSS | **NEW** | `11` token layer (dark+light) |
| Tests (9 files, unit/DOM) | good coverage of logic modules; none on network/pages | **KEEP + extend** | keep all existing tests; add network-layer tests (mock fetch), component tests per `23`, journey tests per `07` |

## 2. Why page-level REPLACE is not a "rewrite"

The distinction that matters: **the hard, correct, security-bearing logic survives** (§1 KEEP rows: channels/session/policy/probe/watch/identities/draft). What is discarded is the *presentation composition* — JsonPanel-as-default, single-file pages, load-once data handling. No authority behavior, no security negative, no honesty rule is rewritten; they are re-housed. Migration risk is therefore concentrated in the view layer, which has the least behavioral coupling. A full rewrite (discard logic modules) is **rejected**: it would throw away tested security/honesty behavior for aesthetic reasons — exactly what the brief forbids.

## 3. The one structural addition: a data/projection layer

Evidence for NEW (not refactor): today's load-once `useState` per page cannot express Phase-2 requirements (master stability across refresh, stale markers, selection preservation by ID, watch-driven updates, shared projections across Home/Work/Activity). Options considered:

- (a) Adopt TanStack Query — mature, but adds a dependency whose cache semantics must be bent to the stale/cursor honesty model.
- (b) Hand-rolled minimal projection store (subscribable, keyed by route+params, carrying `{data, cursor, stale, source}` triples) — ~small, exact fit to the honesty model (stale markers, gap detection, named-zero preservation), zero new runtime dependencies.
- **Recommendation: (b)**, with (a) as fallback if (b) grows past its remit. Recorded as an implementation-phase decision (OQ-6); ADR-0053's dependency discipline (OSI-permissive, pinned) applies either way.

## 4. Migration-risk ordering (input to waves, `39`)

1. **Lowest risk, highest foundation value:** tokens + state system + data layer + shell (no page behavior changes required to land them).
2. **Medium:** Providers refactor (logic reuse is highest; flows already worked end-to-end in D08/D09 evidence); Resources/Activity/System NEW pages (no existing behavior to break).
3. **Highest:** Work inventory + Work Detail + Run timeline (new composition; BD-3 honesty tiering; watch streaming). Scheduled where its dependencies (data layer, state system, watch refactor) are already landed.

## 5. Explicit non-decisions (deferred to authorized implementation)

- Component implementation technology (plain React + CSS custom properties vs CSS modules etc.) — token format decision (`11` outro).
- Whether `shared/` in the clients repo hosts the typed envelopes (pending TS-layer audit; `27` §6).
- Test-runner expansion (e2e driver choice) — `41` §validation flags the constraint set only.

---

*Feeding: `39-control-plane-implementation-waves.md` (ordering), `40-phase3-first-slice.md` (slice selection), `37` (backend gates).*
