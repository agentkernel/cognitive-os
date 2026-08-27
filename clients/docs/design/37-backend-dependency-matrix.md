# 37 — Backend Dependency Matrix (BD register, verified against the real repository)

- Phase 2.5 (audit/planning only)
- Date: 2026-08-24
- Method: BD-1..BD-9 were registered in Phase 1 (`03` §5). Here each is **re-verified against actual code** — not copied from Phase-1 assumptions. Status values: **CONFIRMED GAP** (verified absent) · **PARTIAL** (exists with material restriction) · **RESOLVED** (actually exists — Phase-1 assumption was wrong) · **NEW** (found by this audit).
- For each: what exists today (evidence) · what the design needs · what backend work would close it (contract lane implications) · which Phase-2 surface is gated.

## Current implementation (frozen BD-1..BD-10 baseline)

The verified dependencies below remain the P7-T05 as-of register. Their code
citations and statuses are preserved. Target design does not change an absent
handler into an API.

## Personal 2.0 target gap register

These rows identify semantic gaps only. They intentionally define no route,
DTO, storage schema, lifecycle state or implementation schedule.

| Gap | Target outcome gated | Current evidence | Required disposition |
|---|---|---|---|
| P2-BD-A Adapter conversation projection | embedded native conversation/history and first real chat | no Control Plane transcript/send projection | Requires-backend; define source identity, ordering, freshness, errors, three-axis capability matrix, and display-only render-slot boundary |
| P2-BD-B Agent catalog/install/connect | signed catalog and ≤3-step setup | lifecycle is CLI/library only; runtime envelopes partial | Requires-backend; separate install, connect, disconnect and uninstall semantics |
| P2-BD-C Goal/Plan/attempt | explicit Manage with Personal and Work hierarchy | current Task chain only | Requires-core + Requires-backend for durable authority, revisions, linkage and evidence semantics |
| P2-BD-D Multi-Agent orchestration | roles, handoffs, disagreement and dependency supervision | architecture target only | Requires-backend; daemon remains sole scheduler/authority |
| P2-BD-E Rich Work inventory | Goal/Plan/Task/attempt list and attention | envelope-only task list | Requires-backend; no synthesized completeness |
| P2-BD-F MCP family | first-class MCP identity, discovery, policy, binding and observation | Tool/MCP fixture/design evidence; no operator family surface | Requires-core + Requires-backend; candidate routing, support-path preference, exact-grant reconciliation and re-confirmation triggers required; MCP cannot control host Agent session |
| P2-BD-G Seven-family placement depth | Memory/Skill/Tool/Context/Task/Runtime/MCP family facts in Library/Work/Agents | current six-family depth varies; MCP family absent | Requires-backend where facts/actions are absent; MCP also Requires-core |
| P2-BD-H Federated observation | Personal and Agent-native revisions/conflicts | no general cross-source projection | Requires-backend; preserve source/provenance/freshness |
| P2-BD-I Governed writeback | Shell suggestion -> daemon preview/confirm/Effect -> verified receipt | no general typed path | Requires-backend; no optimistic client mutation |
| P2-BD-J Native/unified Activity | one four-provenance timeline | provider audit + bounded per-task sources | Requires-backend; ordering/coverage contract required |
| P2-BD-K OAuth/subscription accounts | supported consent and credential lifecycle | absent in P7-T05 | Requires-backend; daemon custody only |
| P2-BD-L Credential import | ADR-0055 exact-source import and redacted receipt | governance authorization only | Requires-backend per source; consent and source disposition mandatory |
| P2-BD-M Custom gateways | supported gateway account lifecycle | openai-compatible subset only | Requires-backend for every unsupported gateway |
| P2-BD-N Model capability/quota/cost | honest normalized model, quota and spend facts | models/usage/advisory budgets partial | Requires-backend for missing sources; unknown never zero/free |
| P2-BD-O Target controls/progress | pause/cancel/re-plan/retry/orchestration and real progress | existing BD-1/2/3/4 gaps | Requires-backend; no active-looking control or fabricated percentage |

Existing BD-1..BD-10 still apply where they overlap these broader gaps. A future
delivery must reconcile names rather than treating the P2 rows as duplicate API
requests. Until then, every gated target surface uses `Requires-backend`.

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
