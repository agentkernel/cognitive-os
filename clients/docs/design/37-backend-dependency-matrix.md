# 37 — Backend Dependency Matrix (BD register, verified against the real repository)

- Phase 2.5 (audit/planning only)
- Date: 2026-08-24
- Method: BD-1..BD-9 were registered in Phase 1 (`03` §5). Here each is **re-verified against actual code** — not copied from Phase-1 assumptions. Status values: **CONFIRMED GAP** (verified absent) · **PARTIAL** (exists with material restriction) · **RESOLVED** (actually exists — Phase-1 assumption was wrong) · **NEW** (found by this audit).
- For each: what exists today (evidence) · what the design needs · what backend work would close it (contract lane implications) · which Phase-2 surface is gated.

---

## BD-1 — Task control over HTTP (cancel/pause/resume/retry)

- **Status: CONFIRMED GAP.** No route in `task_api.rs`/`server.rs`; `POST /task/cancel` and `/task/complete` are explicitly forbidden routes (`web-ui-route-inventory.json`); `TaskApplicationService.control` exists without a typed HTTP route (handbook known-limitations).
- Design needs: Work detail class-C block (DD-08) — ships honestly without the backend.
- To close: typed control route(s) through Lane-CTR (envelope + errors + negatives), daemon authorization semantics for cancel/pause, reconciliation behavior on in-flight effects.
- Gates: Work detail intervention verbs; Home "stop this" actions.

## BD-2 — Agent lifecycle over HTTP

- **Status: CONFIRMED GAP.** Lifecycle verbs are admin-cli store-direct (`main.rs:111-128`) + runtime library (`installer.rs:622-673`; adapter states `agent_adapter_manifest.rs:50`). No HTTP. SidecarSession not exposed over HTTP (store record `installation.rs:537-574`).
- Design needs: Agent dossier read depth (current work, session state) + class-C control honesty.
- To close: read projection first (instance/sidecar state, current execution link), then typed lifecycle verbs with fencing/epoch semantics.
- Gates: Agent dossier "current work" section; any agent control.

## BD-3 — Task inventory projection

- **Status: CONFIRMED GAP (partial substitute exists).** Only `list?family=task` envelope (limit 64; `{id, health:"contracted", revision_digest}` — no objective, no lifecycle state field) (`resource_manager.rs:723-738`).
- Design needs: Work inventory columns (state, objective, agent, age, evidence disposition).
- To close: a read projection route (list tasks with state/objective/agent/updated) — read-only, no authority change; contract-light but still Lane-CTR.
- Gates: Work inventory Tier-2; Home current-work strip depth; Activity stream breadth.

## BD-4 — Live watch deltas

- **Status: CONFIRMED GAP.** Task watch snapshot `tasks:[]` always empty (`task_api.rs:1047-1050`); resource watch publishes only `projection.initialized` (`resource_api.rs:60-70`); all watch state is process-local (restart resets cursors).
- Design needs: live Home/Work/Activity updates without manual refresh.
- To close: publish mutation deltas to the watch rings; populate task-watch snapshot; consider durability/cursor semantics across restart (or explicit reset semantics).
- Gates: real-time feel everywhere; OQ-2 (refresh policy) is the wave-1 workaround.

## BD-5 — Unified activity/audit feed

- **Status: CONFIRMED GAP.** `/management/audit` is provider-plane-only; memory/skill/tool/backup mutations write no API-visible audit rows; no cross-domain event listing.
- Design needs: Activity space's unified stream.
- To close: authority event projection across domains (the event log exists — O13 replays it per task; a cross-domain bounded query is the work).
- Gates: Activity unified feed (wave-1 ships honest composition + coverage banner).

## BD-6 — Memory search / proposal review over HTTP

- **Status: CONFIRMED GAP.** FTS5 is daemon-internal (`cognitive-resource-model.md:93-104`); no HTTP search; no proposal review queue route found.
- Design needs: Memory family search; proposal review flow.
- To close: read-only search route (authorization filters before ranking — the product rule); review queue projection.
- Gates: Resources→Memory depth.

## BD-7 — Session lifecycle endpoints

- **Status: CONFIRMED GAP.** No logout/revoke; no introspection route; in-process sessions (`auth.rs:155-163, 372-375`).
- Design needs: session chrome honesty (already designed around it); logout button.
- To close: revoke + introspect routes (small, but security-reviewed).
- Gates: System→Session "sign out" affordance (wave-1 shows expiry + clear-local only).

## BD-8 — Budget enforcement

- **Status: CONFIRMED GAP (and possibly never wanted).** Budgets/alerts exist; no enforcement hook in the proxy path (module docstring + `plan_bound_proxy` review).
- Design needs: nothing — the design renders budgets advisory (correct).
- To close (if ever desired): enforcement decision is a **product-semantic** owner decision first, then backend.
- Gates: nothing in the approved design.

## BD-9 — Browser session bootstrap ergonomics

- **Status: CONFIRMED GAP.** No browser-specific flow; bootstrap secret is file-based; sessions die on reload (memory-only); no CORS (same-origin only).
- Design needs: first-run/reload friction reduction.
- To close: owner + security review; options (none assumed): short-lived re-issue within idle window, OS-keychain-held bootstrap handoff via CLI helper, etc.
- Gates: session UX polish; wave-1 ships the honest gate.

## BD-10 — NEW: Resource projection-plane reconciliation

- **Status: NEW (found by this audit).** Projection plane self-declares memory/skill/context `not-backed` (`resource_api.rs:1567-1646`) while authority-backed reads exist elsewhere; context/runtime Resource Manager lists are empty `projection-only`; the two planes disagree.
- Design needs: nothing directly (family pages consume authority-backed routes), but the disagreement is a trap for future consumers.
- To close: reconcile planes (back the projection plane with authority reads or mark it deprecated).
- Gates: none in the approved design; flagged for backend hygiene.

## Non-dependencies confirmed (Phase-1 caution that turned out fine)

| Assumed risk | Reality check | Result |
|---|---|---|
| "Evidence may not be authoritative enough" | digest-bound terminal evidence with verification+acceptance records | **RESOLVED — no dependency** |
| "Provider surface may be too thin for the governance design" | accounts/keys/models/bindings/usage/budgets/alerts/audit all implemented | **RESOLVED** |
| "Backup/restore may be CLI-only" | full HTTP surface (mgmt channel) | **RESOLVED** |
| "Readiness may be too thin for Home" | 6-component projection + first_conversation_ready + provider secret-resolution honesty | **RESOLVED** (doctor sub-sections placeholder — labeled) |

## Feature → backend matrix (brief §19 format)

| Phase-2 feature | Frontend-only? | Existing API? | API composition? | Backend modification? | New capability? | Blocked? |
|---|---|---|---|---|---|---|
| Shell + strip + nav | yes | — | — | — | — | no |
| State system + tokens | yes | — | — | — | — | no |
| Home (Tier-1) | composition | yes (readiness/alerts/evidence-per-known-task) | yes (client compose, labeled) | — | — | no |
| Work inventory Tier-1 | yes (envelope + session-observed) | partial | yes | — | — | no (thin) |
| Work inventory Tier-2 | — | — | — | BD-3 | yes | backend-gated |
| Work detail (all six sections) | composition | yes (per-task_ref) | yes | — | — | no |
| Run timeline | composition | yes (evidence+observation+watch) | yes | — | — | no (live depth: BD-4) |
| New-task flow | yes | yes | — | — | — | no |
| Agents dossier | composition | partial (BD-2 for current work/session) | yes | — | — | no (honest depth) |
| Providers (all sections) | yes | yes | yes | — | — | no |
| Resources family pages | yes | yes (authority routes) | yes | — | — | no |
| Activity (wave-1 honest composition) | composition | partial | yes | — | — | no |
| Activity unified feed | — | — | — | BD-5 | yes | backend-gated |
| System (readiness/doctor/stewardship/session) | yes | yes | yes | — | — | no |
| Command palette | yes (client index) | existing routes | yes | — | — | no |
| Live updates everywhere | — | — | — | BD-4 | yes | backend-gated |
| Task/agent controls | — | — | — | BD-1/BD-2 | yes | backend-gated |

---

*Ordering of the frontend-buildable set vs backend-gated set drives the waves in `39`; the first slice is `40`.*
