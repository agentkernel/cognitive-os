# 35 — Design → Real Code Traceability Matrix

- Phase 2.5 (audit/planning only)
- Date: 2026-08-24
- Maps each Phase-2 design element to: existing SPA route/component → existing API → existing backend capability → **required change** (SPA-side unless marked BACKEND). "Existing component" cites `pc/web/src/` @ `0320c1a`; API cites `28`. Backend gaps reference BD-n (verified in `37`).

---

## 1. Shell & cross-cutting

| Phase-2 design | Existing SPA | Existing API | Backend capability | Required change |
|---|---|---|---|---|
| App shell: status strip + sidebar + MI/MID regions (`12`) | `Shell` + flat sidebar (`App.tsx:116-150`) | strip cells map to `/personal/health` + `/personal/status` + `/management/alerts` + watch state | all exist | REPLACE Shell (new layout; strip new); sidebar rework (7 spaces, counts, footer caption) |
| Command palette ⌘K (`21`) | none | composed from existing list/inspect routes | exists (client-side index over loaded projections) | NEW component + NEW client index layer; no backend change |
| State system (`22`) | `LoadState` union + `StateNote` text line (`App.tsx:27-90`) | n/a (presentation) | vocabularies exist in API payloads | NEW StateChip/dot/label components; REPLACE StateNote; client normalization layer for the 3 error envelopes (R-2) |
| Design tokens (`11`) | 155-line global CSS | n/a | n/a | REPLACE styling architecture (token layer + per-component styles) |
| Session gate/chrome (DD-05) | `SessionPage` + `SessionForm` + `RequireSession` | `POST /local/session` | exists | KEEP logic; REFACTOR presentation (gate inline over destination; strip session cell with expiry from `absolute_expiry_secs`/`idle_expiry_secs`) |
| Honesty plumbing (redaction, channel binding, escaping, route whitelist vs 200-stubs) | `policy.ts`, `channels.ts`, `api.ts`, `session.ts` | n/a | n/a | KEEP + EXTEND: add route-whitelist + 200-stub detection (R-1); keep all existing negatives |

## 2. Home (`13`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Readiness line + component row | HomePage 4 status lines + JSON | `/personal/status|readiness|doctor` | exists (sub-sections placeholder) | REPLACE presentation; dedupe status/readiness double-fetch |
| Needs-attention queue | none | composed: readiness + `/task/evidence`(unknown outcomes) + `/management/alerts` + bindings dispatchability | composed from existing routes; **no server-side attention aggregation** | NEW composition layer (client-side, honesty-labeled); alert ack via existing `POST /management/alerts/acknowledge` |
| Current work strip | none | session-observed task_refs + `list?family=task` envelope | PARTIAL (BD-3) | NEW component; Tier-1 honesty footer per `14` |
| Recent evidence | none | `GET /task/evidence` per known task_ref | exists (per-task only) | NEW component (limited to session-known tasks in Tier-1) |
| Critical changes group | none | `/management/audit` (provider plane) | PARTIAL (provider-only audit) | NEW rows from audit; coverage labeled |

## 3. Work (`14`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Inventory master | none (Tasks page requires known task_ref) | `list?family=task` (envelope, limit 64) + session-observed refs | PARTIAL (BD-3) | NEW page; Tier-1/Tier-2 column contract; honesty footer |
| Filters/saved views | none | client-side over loaded set | exists client-side | NEW (session-local, labeled) |
| Inspector | none | composed: evidence + effects + bindings + watch | exists per task_ref | NEW component |
| New-task governed flow | TasksPage chain (record→interpret→preview→admit, `App.tsx:1162-1247`) | `/task/{intent.record,intent.interpret,preview,admit}` | exists | REFACTOR into full-route flow; surface ambiguities/`clarification_required` (currently unrendered); generalize beyond hardcoded draft only when contract supports |
| Watch controls | manual poll buttons (`watch.ts`, `watchSse.ts`) | `GET /task/watch` | PARTIAL (process-local, empty snapshot) | REFACTOR to EventSource streaming (parser exists, never opened); keep stale/gap semantics; keep detach-never-cancels |

## 4. Work Detail (`15`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Persistent header (state+disposition+actions) | none | evidence + admit facts | exists per task_ref | NEW |
| Run timeline (dual-lane) | none | `/task/evidence` transitions (authority) + `/task/observation` o4/o5 + watch deltas (observation) | exists per task_ref; process facts bounded | NEW component (the signature); lanes fed by two real sources |
| Effects section | raw JSON panel | `/task/effects` | exists | NEW structured table |
| Evidence section | raw JSON panel | `/task/evidence` | exists | NEW EvidenceBlock component |
| Intent & Contract chain | raw JSON of preview result only | record/interpret/preview/admit results + evidence `intent_refs` | exists | NEW ProvenanceChain component |
| Context section | none | consumption pins (`/task/resource/v1/consumption`); context view projection `not-backed` | PARTIAL | NEW section with S7 honesty state + pins when present |
| Class-C control block (cancel/pause) | bare `not-run` text (`App.tsx:1259`) | none (BD-1) | FORBIDDEN routes confirmed | KEEP honesty; upgrade presentation per DD-08 |

## 5. Agents (`16`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Inventory rows (actor+binding+current work) | table + JSON (`AgentsPage`) | `list?family=runtime` + `agent-bindings` + `/personal/dsh/runtime` | exists; current-work PARTIAL (BD-2/BD-3) | REPLACE row grammar |
| Dossier: 7-identity cards | 9-card merge (`identities.ts`) | `inspect?family=runtime` | exists | KEEP identities logic; REFACTOR presentation (source labels, not-confused-with captions) |
| Binding section | none on this page | `agent-bindings` | exists | NEW section + contextual entry to Providers flow |
| Capabilities section | none | `tool/exposure?task_ref=` (task-scoped only) | PARTIAL | NEW section; exposure shown per-task-honest, else S7 |
| Activity/Evidence slices | none | per-task projections | PARTIAL (BD-2/BD-3) | NEW links/sections with honesty states |
| Lifecycle class-C block | "Typed lifecycle" not-run list (`App.tsx:416-423`) | none (CLI-only, verified) | CLI+LIB only | KEEP honesty; upgrade copy per DD-08 (CLI path named) |

## 6. Providers (`17`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Accounts master (triage order) | `ProvidersPage` table | accounts list/inspect | exists | REFACTOR rows (status cause, catalog rev, probe fact, attention-first sort) |
| Create-account flow | form + trust gate (`probe.ts:111-120`) | `POST accounts` | exists | KEEP logic; REFACTOR to staged flow per Flow 3 |
| Key handoff | key form (memory-only, op by presence) | `accounts/key` | exists | KEEP (pattern is correct); REFACTOR copy/consequence |
| Models section | catalog table + refresh + manual add | models routes | exists | KEEP logic; REFACTOR presentation (source honesty, cost_unavailable) |
| Bindings section | separate BindingsPage | `agent-bindings` routes + dsh runtime/apply | exists | MOVE into account detail + agent dossier context; KEEP `bindingRevisionForCas`/`acceptDshApply`/`dispatchAllowed` gates |
| Usage/Budgets/Alerts/Audit sections | raw JSON on Activity page | `/management/{usage,budgets,alerts,audit}` | exists (observe-only; no filters) | NEW structured sections with advisory annotations |
| Delete guard | copy note | delete route (binding-blocked) | exists | KEEP; surface blocking bindings in confirm |

## 7. Resources (`18`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Family hub (index rows) | family select + JSON | `list?family=` | PARTIAL (context/runtime empty projection-only) | NEW hub; honesty labels per family |
| Memory list/explain/remember/forget | none | memory routes | exists | NEW family page (all real routes, unused by current UI) |
| Skills list/import/bind/revoke/explain | none | skill routes | exists | NEW family page |
| Tools catalog/lifecycle | none | tool routes | exists | NEW family page (readiness caveat annotation) |
| Context page | none | none standalone | NONE | honest pointer page → Work (no fake browser) |

## 8. Activity (`19`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Evidence stream (7 kinds) | 4 raw JSON panels | composed: audit + alerts + per-task evidence/effects + watch-observed | PARTIAL (no unified feed — BD-5) | NEW page; coverage banner persistent |
| Per-object timelines | none | same sources per object | exists per object | NEW shared timeline component |
| Alert ack | none in UI | `alerts/acknowledge` | exists | NEW inline class-B action |

## 9. System (`20`)

| Design element | Existing | API | Backend | Required change |
|---|---|---|---|---|
| Readiness detail | Home JSON panels | status/doctor | exists (sub-sections placeholder — render as such) | NEW page |
| Stewardship (backup/restore) | none in UI | backup/restore routes | exists | NEW flows (preview-first, 409-class copy) |
| Session detail | none | session facts (expiry fields at issuance) | PARTIAL (no introspection route — BD-7) | NEW section; client-held expiry display |
| About/diagnostics | none | health + build facts | PARTIAL | NEW section |

## 10. Cross-cutting reality checks

1. **Every Phase-2 surface is buildable on existing routes** except where BD-1..BD-6 are named; no surface requires a forbidden route.
2. **No new backend route is required for Wave-1 shells + Providers + Resources + System + Work creation.** The first backend dependency that blocks a *designed* surface is BD-3 (Work inventory depth) — see `37`.
3. **Client must add:** route whitelist + stub detector (R-1), error normalizer (R-2), `secret_ref` display policy (R-5), real EventSource watch (R-3/R-4 honest labeling), typed envelopes (currently untyped coercion).

---

*Refactor-vs-rewrite per subsystem: `36`. Backend dependency verification: `37`. Design challenges against this reality: `38`.*
