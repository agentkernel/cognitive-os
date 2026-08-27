# 34 — Web UI Current-State Audit (per page)

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Subject: `pc/web/src/App.tsx` @ `0320c1a` (branch `personal/P7-T05-dsh-binding-cas`; superset of `main`). Line numbers cite that revision.
- Per-page record format: Route · Purpose · Component tree · Data source · State · User actions · Problems · Reusable · Replaceable · Missing.

## Current implementation (frozen audit baseline)

The per-page observations below are preserved without reinterpretation as
pre-closure P7-T05 evidence at `0320c1a`. Route names, line numbers, actions,
omissions and reusable logic refer to that revision. The accepted current SPA
now has seven routes: Home / Work / Agents / Providers / Resources / Activity /
System; that later integration does not erase the older observations.

## Personal 2.0 target delta

The target six-space IA and global Agent Shell supersede this page map for
design: Providers/System move under Settings, Resources becomes Library,
embedded conversation/history lives under Agents, managed Goal/Plan/Task Work
gains Context/orchestration, MCP is first-class, and Activity gains provenance.
Those changes are not present in this audit. Use
[Traceability](35-design-to-code-traceability.md) for explicit gaps; do not
rewrite the historical findings or infer an API.

---

## Page: Session

- **Route:** `#/session` (also rendered inline by `RequireSession` gates on every page)
- **Purpose:** bootstrap-secret → channel session issuance
- **Component tree:** `SessionPage` → `SessionForm` (shared with gates)
- **Data source:** `POST /local/session` ×2 (management + task)
- **State:** local form state; session stored in module memory (`session.ts`)
- **Actions:** issue sessions; clear memory session
- **Problems:** re-paste on every reload (by design, costly); no expiry display; principal is a free-text default
- **Reusable:** `SessionForm`, memory-only field pattern, "not a Provider key" copy
- **Replaceable:** page-as-destination (demote to chrome per DD-05)
- **Missing:** expiry introspection, proactive re-auth (BD-7/BD-9)

## Page: Home

- **Route:** `#/` (`HomePage`, `App.tsx:254-304`)
- **Purpose:** daemon health/readiness/doctor overview
- **Component tree:** `HomePage` → 4 × `StateNote` + 2 × `JsonPanel`
- **Data source:** `GET /personal/health` (unauth), `/personal/status`, `/personal/readiness`, `/personal/doctor`
- **State:** `LoadState` per projection; load-once
- **Actions:** none (no refresh button even)
- **Problems:** raw JSON is the main content; no attention synthesis; no current work; no auto/manual refresh; readiness vs status duplicate call (identical projections, both fetched)
- **Reusable:** the four-endpoint fetch pattern; status-line concept
- **Replaceable:** the entire presentation (raw JSON → attention surface per `13`)
- **Missing:** attention queue, current work, recent evidence, alerts surfacing

## Page: Agents (list)

- **Route:** `#/agents` (`AgentsPage`, `320-381`)
- **Purpose:** runtime-family inventory
- **Component tree:** `AgentsPage` → table + 2 × `JsonPanel` (bindings, dsh runtime)
- **Data source:** `resource/v1/list?family=runtime`, `agent-bindings`, `/personal/dsh/runtime`
- **State:** `LoadState`; load-once
- **Actions:** Inspect link per row
- **Problems:** no lifecycle verbs (honest `not-run` labels present but as dead text); no binding state per row; no current-work linkage; JSON panels dominate
- **Reusable:** table markup pattern; the not-run label discipline
- **Replaceable:** table content model (needs dossier-oriented rows per `16`)
- **Missing:** agent state semantics (source-labeled), binding dispatchability, activity links

## Page: Agent detail

- **Route:** `#/agents/:id` (`AgentDetailPage`, `383-427`)
- **Purpose:** identity breakdown of one runtime resource
- **Component tree:** 9 identity cards (`identities.ts`) + "Typed lifecycle" not-run list + inspect `JsonPanel`
- **Data source:** `resource/v1/inspect?family=runtime&id=…`
- **State:** `LoadState`
- **Actions:** none
- **Problems:** no binding section, no current work, no activity/evidence; identity cards carry no source labels
- **Reusable:** `identities.ts` merge (the 9-identity model is correct and matches the canonical decomposition)
- **Replaceable:** page composition (→ dossier per `16`)
- **Missing:** binding/capabilities/activity/evidence sections; class-C control honesty block

## Page: Providers (list + create)

- **Route:** `#/providers` (`ProvidersPage`, `429-586`)
- **Purpose:** create + list provider accounts
- **Component tree:** create form (kind select, endpoint, trust checkboxes, conditional confirmation) + accounts table
- **Data source:** `GET/POST /management/providers/accounts`
- **State:** `LoadState`; form state; refresh after mutation
- **Actions:** create account; open detail
- **Problems:** no triage ordering (broken accounts don't float); secret presence is the only credential fact; no probe/catalog summary per row
- **Reusable:** the trust-gate form logic (`probe.ts:111-120`), secret presence rendering (`App.tsx:101-106`)
- **Replaceable:** row grammar (→ `17` §1)
- **Missing:** status cause class per row, catalog revision, last probe, attention sort

## Page: Provider detail

- **Route:** `#/providers/:id` (`ProviderDetailPage`, `588-785`)
- **Purpose:** key handoff, probe, catalog, delete
- **Component tree:** info panel + key form + probe button + manual model form + delete + catalog table
- **Data source:** `accounts/inspect`, `models?account_id=`, `accounts/key`, `models/refresh`, `models/add`, `accounts/delete`
- **State:** `LoadState` ×2; form states
- **Actions:** set/rotate/remove key; refresh models; add model; delete account
- **Problems:** capability always `not-run` (honest but unexplained); no usage/audit/bindings sections; no trust-grant history; delete consequence is copy-only (no binding list)
- **Reusable:** key handoff pattern (memory-only, op by presence), probe classification (`probe.ts:33-94`), cost display honesty (`policy.ts:65-73`)
- **Replaceable:** page sections (→ five-section governance detail per `17` §2)
- **Missing:** usage/budgets/alerts/audit sections, bindings management, trust reconfirm flow surfacing

## Page: Bindings

- **Route:** `#/bindings` (`BindingsPage`, `787-1091`)
- **Purpose:** fixed agent↔account↔model bindings + dsh apply
- **Component tree:** set-binding form (agent/account/model selects, CAS revision, 3 checkboxes) + active-bindings table + dsh-apply section
- **Data source:** `agent-bindings` (GET/POST), `agent-bindings/remove`, accounts, models, `/personal/dsh/runtime`, `/provider/v1/dsh/selected-model`
- **State:** `LoadState`; models reload on account change
- **Actions:** set binding (CAS), remove binding, apply to dsh
- **Problems:** must re-establish all context on every visit (the DD-04 defect); negative checkboxes (fallback/override) are traps-as-education; agent select hardcoded pi/dsh
- **Reusable:** `bindingRevisionForCas` (`policy.ts:91-100`), `acceptDshApply` gate (`policy.ts:122-149`), `dispatchAllowed` (`policy.ts:151-168`) — the CAS/gate logic is solid
- **Replaceable:** page as a destination (fold into Providers + Agent dossier per DD-04)
- **Missing:** consequence preview richness (running-work impact), binding history

## Page: Tasks

- **Route:** `#/tasks` (`TasksPage`, `1093-1341`) — the only task-channel page
- **Purpose:** governed task creation + per-task projection inspection
- **Component tree:** objective form → chain buttons → task_ref form → projection buttons → 3 × `JsonPanel` (effects/evidence/observation)
- **Data source:** `intent.record` → `intent.interpret` → `preview` → `admit`; `effects`/`observation`/`evidence`; `watch` (manual poll)
- **State:** `LoadState`; `WatchState` via `watch.ts`
- **Actions:** record/interpret/preview chain; admit; load projections; watch poll; reconnect snapshot; simulate cursor gap; detach
- **Problems:** **no task list** (must know a task_ref); hardcoded workspace-search draft + deadline 2027-12-31; watch is manual polling despite SSE parser; "Simulate cursor gap" is a debug affordance in product UI; projections are raw JSON
- **Reusable:** `taskDraft.ts` (uuidV7, draft builder), `watch.ts` state machine, `watchSse.ts` parser, `escapeUntrustedText`
- **Replaceable:** the entire page composition (→ Work inventory + detail per `14`/`15`)
- **Missing:** inventory, run timeline, evidence-first rendering, interpretation ambiguity surfacing, cancel honesty block (currently a bare `not-run` text)

## Page: Activity

- **Route:** `#/activity` (`ActivityPage`, `1343-1368`)
- **Purpose:** usage/budgets/alerts/audit viewing
- **Component tree:** 4 × `JsonPanel`
- **Data source:** `GET /management/{usage,budgets,alerts,audit}`
- **State:** `LoadState` ×4
- **Actions:** none (not even ack)
- **Problems:** not an activity surface (no events/effects/evidence); no filters; no ack action despite the route existing; raw JSON only
- **Reusable:** the four-endpoint fetch set
- **Replaceable:** entire presentation (→ evidence stream per `19`)
- **Missing:** event typing, object links, coverage honesty, per-object timelines

## Page: Resources

- **Route:** `#/resources` (`ResourcesPage`, `1370-1398`)
- **Purpose:** six-family browsing
- **Component tree:** family select + 1 × `JsonPanel`
- **Data source:** `resource/v1/list?family=…`
- **State:** `LoadState`
- **Actions:** family select only
- **Problems:** no inspect links, no actions despite existing verbs (memory remember/forget, skill import/bind/revoke, tool enable/disable/quarantine/revoke are all HTTP-available); context/runtime families return empty projection-only (not explained)
- **Reusable:** family select concept
- **Replaceable:** everything (→ family hub + family pages per `18`)
- **Missing:** family-native depth, explain views, mutation flows, honesty labels for not-backed facets

## Cross-cutting findings

1. **No `*` route** — unknown hashes render an empty main area.
2. **No global refresh/invalidation strategy** — every page is load-once.
3. **No notifications/alerts surfacing** — alerts exist only as Activity JSON.
4. **No settings surface** (acceptable — System owns stewardship; but session/status display is missing from chrome).
5. **The strongest code in the app is the policy layer** (`policy.ts`, `probe.ts`, `channels.ts`, `session.ts`, `watch*.ts`) — channel discipline, redaction, CAS gates, watch state machine. These are the reuse core for any refactor.
6. **The weakest code is the view layer** — single-file pages, JsonPanel-as-presentation, untyped coercion.

---

*Subsystem-level KEEP/REFACTOR/REPLACE/REWRITE/NEW decisions built on this audit: `36-refactor-vs-rewrite.md`.*
