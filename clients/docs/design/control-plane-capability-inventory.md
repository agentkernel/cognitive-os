# Control Plane Capability Inventory

- Phase: Product Redesign Phase 1 (design-only)
- Date: 2026-08-24
- Companion: [Current State Map](control-plane-current-state.md)
- Method: every capability is rated from **verified daemon/UI evidence only** (route handlers, store schemas, canonical docs, frozen route inventory). Ratings:
  - **AVAILABLE** — implemented and reachable by an operator through the daemon HTTP API (the SPA's only possible backend).
  - **PARTIAL** — implemented with material restrictions (process-local, observe-only, CLI-only, placeholder, or inconsistent planes).
  - **NOT AVAILABLE** — does not exist on any operator-reachable surface, or is deliberately refused/forbidden.
  - **UNKNOWN** — cannot be verified from the audited sources.
- Hard rule (inherited from `web-ui-architecture.md:98-106` + ADR-0053 §4): a capability that is PARTIAL/NOT AVAILABLE must be *rendered* as such; this inventory never licenses inventing routes.

---

## 1. Agents

| Capability | Rating | Evidence |
|---|---|---|
| List installed/registered agents (runtime inventory) | AVAILABLE | `GET /management/resource/v1/list?family=runtime` (`resource_manager.rs:143-161`); SPA Agents page |
| Inspect agent identity decomposition (package/installation/registration/instance/sidecar/execution/process/task/shell_session) | PARTIAL | `GET /management/resource/v1/inspect?family=runtime&id=`; 9-card merge is client-side (`identities.ts`); depth of per-identity facts limited to envelope |
| Agent Provider binding (view/set/remove, CAS) | AVAILABLE | `GET|POST /management/agent-bindings`, `POST /management/agent-bindings/remove` (409 `PROVIDER_BINDING_REVISION_STALE`) |
| Agent runtime state (Pi sidecar session) | PARTIAL | SidecarSession exists in store (`installation.rs:537-574`) but is **not exposed over HTTP**; only dsh has a runtime snapshot (`GET /personal/dsh/runtime`, state ACTIVE/INACTIVE/CRASHED) |
| Agent lifecycle over HTTP: install / register / activate / pause / resume / stop / recover / upgrade / rollback / uninstall | **NOT AVAILABLE** | admin-cli verbs operate directly on the SQLite store with a `PrivilegedManagementSession` file (`apps/admin-cli/src/main.rs:111-128`); runtime library states `Registered|Active|Paused|Stopped` (`agent_adapter_manifest.rs:50`); frozen inventory marks all five control verbs `unavailable` (`web-ui-route-inventory.json`) |
| Agent quarantine over HTTP | **NOT AVAILABLE** | Tool quarantine exists; Agent-instance quarantine does not (`web-ui-route-inventory.json` agent-quarantine row) |
| Multi-agent orchestration | **NOT AVAILABLE** | P6-T01..T04 all `not-started`, default-off, NO-GO is a legal result (`PERSONAL-DEVELOPMENT-PLAN.md:930-933`) |
| Non-Pi agents | PARTIAL (designed) | Codex fixture identity only; DeepSeek harness (dsh) runs as a candidate-only AKP adapter; neither inherits Pi qualification (handbook `capability-status.md`) |

## 2. Tasks

| Capability | Rating | Evidence |
|---|---|---|
| Record raw intent | AVAILABLE | `POST /task/intent.record` → `user_intent_record_id` + digest |
| Submit interpretation candidate (with ambiguities/information gaps) | AVAILABLE | `POST /task/intent.interpret` → `candidate|clarification_required` |
| Server-issued preview (digest-bound contract draft) | AVAILABLE | `POST /task/preview` (no authority write) |
| Admit task (CAS + principal-bound acceptance) | AVAILABLE | `POST /task/admit` (403 principal mismatch; 409 admission rejected) |
| List / inventory tasks | PARTIAL | `GET /management/resource/v1/list?family=task` returns current contracts, limit 64, envelope-only (no objective text, no state field beyond `health:"contracted"`) (`resource_manager.rs:723-738`) |
| Inspect task state / lifecycle | PARTIAL | `GET /task/evidence` exposes `lifecycle.current_state/current_version` + transitions; store states `ACTIVE → CANDIDATE_COMPLETE → COMPLETED`; no DRAFT state API-visible |
| Watch task events (SSE) | PARTIAL | `GET /task/watch` — process-local 128-event ring, snapshot `tasks:[]` always empty, stale resume → 409 (`task_api.rs:1029-1066`) |
| Task evidence (terminal, digest-bound) | AVAILABLE | `GET /task/evidence` (404 `TASK_EVIDENCE_NOT_FOUND`) |
| Task effect history | AVAILABLE | `GET /task/effects` (bounded, stage/outcome/reconcile classes) |
| Task observation (bounded O2/O3/O4/O5/O13) | AVAILABLE | `GET /task/observation?family=` with named zeros + negative controls |
| Task-scoped resource consumption (memory/skill pins) | AVAILABLE | `POST|GET /task/resource/v1/consumption` |
| **Task cancel** | **NOT AVAILABLE** | `TaskApplicationService.control` exists without a typed HTTP route; `POST /task/cancel` is a forbidden route by inventory |
| Task pause / resume / retry / delete | **NOT AVAILABLE** | no such routes; lifecycle is daemon-internal |
| Arbitrary task types from the UI | PARTIAL | the contract draft schema is general, but the shipped SPA can only build one fixed workspace-search draft (hardcoded deadline 2027-12-31) (`taskDraft.ts:39-65`) |

## 3. Runs

"Run" is a **product-level Activity concept** (Activity = Run, Process, Effect, Evidence — `product-design.md:213-220`), not a separately persisted HTTP object today.

| Capability | Rating | Evidence |
|---|---|---|
| Run/execution identity (AgentExecution, epoch-fenced) | PARTIAL | exists in the authority model (user-journeys §6.6) and dsh session snapshot (`fencing_epoch`, `last_sequence`, `task_ref`); no first-class `GET /runs` or run detail route |
| Run list / run history | **NOT AVAILABLE** | no cross-task execution listing; nearest facts are per-task effects/evidence and the o4/o5 observation families |
| Run timeline (ordered state transitions) | PARTIAL | per-task via `GET /task/evidence` `lifecycle.transitions[]` (+ `transitions_truncated`); no unified timeline |
| Process observation (spawn/alive/exit/samples) | PARTIAL | bounded process facts inside dsh runtime snapshot and o5/observation projections; not a general process browser |
| Run-level cancel/stop | **NOT AVAILABLE** | inherits task-cancel gap |
| Streaming run output | PARTIAL | Provider proxy SSE passthrough exists for chat completions; agent/run output streaming to the SPA does not (watch is polled SSE-as-text in the shipped UI) |

## 4. Resources (common envelope)

| Capability | Rating | Evidence |
|---|---|---|
| Six-family list (memory/skill/tool/context/task/runtime) | PARTIAL | `GET /management/resource/v1/list?family=` — tool/memory/skill/task backed (limit 64); **context/runtime return empty `projection-only`** (`resource_manager.rs:485-721`) |
| Inspect envelope | PARTIAL | `GET /management/resource/v1/inspect` (404 `RESOURCE_MANAGER_NOT_FOUND`); envelope only, domain depth varies |
| Resource projection + watch per family | PARTIAL | `/resource/v1/projection|watch`; memory/skill/context self-declare `availability:"not-backed"`; watch publishes only `projection.initialized` — **inert after startup** (`resource_api.rs:60-70, 1567-1646`) |
| Generic create / install / execute / complete | **NOT AVAILABLE (deliberately refused)** | 400 `RESOURCE_MANAGER_OPERATION_FORBIDDEN` — a designed invariant, not a gap to close (`resource_manager.rs:101-106`) |
| Cross-family search | **NOT AVAILABLE** | no search route; per-family list only |
| Bind/unbind/enable/disable/revoke via common envelope | PARTIAL | skill bind/unbind/revoke with CAS; tool enable/disable/revoke; other family×op combos → 400 `RESOURCE_MANAGER_OPERATION_UNSUPPORTED` |

## 5. Memory

| Capability | Rating | Evidence |
|---|---|---|
| Remember (explicit, governed headers daemon-composed) | AVAILABLE | `POST /management/resource/v1/memory/remember` → 201 (sealed + unsealed public paths; retention cap 31 536 000 s) |
| Forget (durable tombstone) | AVAILABLE | `POST /management/resource/v1/memory/forget` |
| Explain memory object (candidate/decision/provenance) | AVAILABLE | `GET /management/resource/v1/memory/object?id=` (`memory.explain`) |
| List memory | PARTIAL | Resource Manager list family=memory, non-tombstoned, limit 64, envelope-only |
| Search memory content (FTS) | PARTIAL | FTS5 exists daemon-side as derived index (`cognitive-resource-model.md:93-104`); **no HTTP search route** — content reachable only per-id via explain |
| Review Agent memory proposals | UNKNOWN | candidate→admission flow is documented and partially evidenced in eval records (MS-AUTH Memory 10/10); no dedicated HTTP review queue route found |
| Memory update/edit | **NOT AVAILABLE** | new versions via new remember only |

## 6. Skills

| Capability | Rating | Evidence |
|---|---|---|
| Import (immutable package/revision) | AVAILABLE | `POST /management/resource/v1/skill/import` (201; supersede path with `previous_revision_id`) |
| List bindings | AVAILABLE | Resource Manager list family=skill (limit 64) |
| Inspect binding / revision (digests, provenance, revocation reason) | AVAILABLE | `GET /management/resource/v1/skill/binding/explain[?kind=revision]` |
| Bind (CAS) | AVAILABLE | `POST /management/resource/v1/skill/bind`; envelope bind (object_version 0→1) |
| Revoke binding | AVAILABLE | `POST /management/resource/v1/skill/binding/revoke` (route-order defect fixed by P2-T11) |
| Enable/disable (eligibility without byte mutation) | PARTIAL | documented action (`cognitive-resource-model.md:118-127`); HTTP reachability not separately verified beyond lifecycle wording — folded into bind/revoke envelope semantics |
| Pin exact revision | PARTIAL | documented; expressed through binding revision selection |
| Marketplace / auto-download / chaining | **NOT AVAILABLE** | deferred beyond Linux 1.0 by design |

## 7. Tools

| Capability | Rating | Evidence |
|---|---|---|
| Catalog / discover (7 built-in families) | AVAILABLE | `GET /management|task/resource/v1/tool[/discover]` — workspace read/search/write/patch, process check, http fetch, registered-check run (`tool_registry.rs:128-217`) |
| Lifecycle overlay: enable / disable / quarantine / revoke | AVAILABLE | `POST /management/resource/v1/tool/{enable,disable,quarantine,revoke}`; illegal transitions 409 (revoked terminal; quarantined↛enabled) |
| Per-task exposure + bounded selection receipts | AVAILABLE | `GET …/tool/exposure?task_ref=`; `POST /task/resource/v1/tool/selection` (digest-gated) |
| Execution readiness projection | PARTIAL | registered/enabled ≠ execution-ready; executors assembled for all 7 families but "the production call chain from an admitted Task to one of these sinks does not exist yet" (`tool_executor/mod.rs:48-52`) |
| Dynamic/MCP tools | PARTIAL (designed) | MCP adapter is transport-only fixture (P5-T03); dynamic ecosystem delivered behind qualification (P5-T04); not a product operator surface |
| Tool un-quarantine | **NOT AVAILABLE** | only revoke escapes quarantine |

## 8. Providers

The most complete domain. Source: P8-T13 (`provider_control_plane.rs`), handbook usage page, shipped SPA pages.

| Capability | Rating | Evidence |
|---|---|---|
| Account create/list/inspect/update/delete | AVAILABLE | full route set; delete blocked by active bindings; trust changes require `reconfirm:true` (409) |
| Key set/rotate/remove via SecretStore | AVAILABLE | `POST /management/providers/accounts/key` (`secret_bearing`); daemon-memory-only key handling |
| Endpoint trust (private-network / insecure-HTTP grants) | AVAILABLE | durable account-level flags with re-confirmation on scope broadening |
| Model discovery (bounded probe) | AVAILABLE | `POST /management/providers/models/refresh`; failure preserves catalog, marks account `degraded` |
| Manual model add + pricing | AVAILABLE | `models/add`, `models/set-price`; missing price → `cost_unavailable`, never zero |
| Fixed agent binding (one account+model per agent; pi/dsh) | AVAILABLE | CAS revisions; no fallback/override by design |
| dsh runtime apply (republish selected model to running panel) | AVAILABLE | `POST /personal/dsh/runtime op=apply` (+4 s acknowledgement wait) |
| Provider proxy egress (unary + SSE) | AVAILABLE | `POST /provider/v1/chat/completions` (bound-binding path; legacy fallback) |
| Usage events + aggregates | AVAILABLE | `GET /management/usage` (30-day events / 90-day aggregates; **no query filters**) |
| Budgets | PARTIAL | CRUD exists; **observe-only** — no enforcement hook in the proxy path |
| Alerts (80% warning / 100% exceeded, deduped) | AVAILABLE | list + acknowledge; delivery is pull-only |
| Audit (provider-plane events) | AVAILABLE | `GET /management/audit` (no filters) |
| Capability probe (beyond model discovery) | PARTIAL | route inventory lists bounded capability check as design; SPA renders capability `not-run` unconditionally (`App.tsx:721`) |
| OAuth / browser login / refresh tokens / routing / fallback / hard budget blocking | **NOT AVAILABLE** | explicit non-goals (`provider-control-plane.md:41-43,155-160`) |

## 9. Activity

| Capability | Rating | Evidence |
|---|---|---|
| Provider usage/budget/alert/audit projections | AVAILABLE | §8; pull-only, unfiltered, JSON-shaped |
| Task event watch (SSE) | PARTIAL | process-local, empty snapshot, 128-event window (§2) |
| Resource watch | PARTIAL | inert after startup (§4) |
| Observation plane O2 (authorization) / O3 (cache) / O4 (scheduler) / O5 (effects) / O13 (audit replay) | AVAILABLE | per-task, bounded, named-zero honest (`observation.rs`) |
| Terminal evidence + effect history per task | AVAILABLE | §2 |
| Unified cross-object activity feed ("what happened, newest first") | **NOT AVAILABLE** | no route aggregates events across tasks/resources/providers |
| Management-action audit (memory/skill/tool/backup mutations) | **NOT AVAILABLE** | only Provider CP writes audit rows |
| Notifications (push/badge/alert surfacing) | **NOT AVAILABLE** | alerts exist as a pull projection only |
| Process/Agent output stream to UI | PARTIAL | bounded observation facts; raw output is redacted at the daemon boundary and never required as display fact (`web-ui-design.md:77-78`) |

## 10. System

| Capability | Rating | Evidence |
|---|---|---|
| Daemon liveness | AVAILABLE | `GET /personal/health` (unauthenticated loopback) |
| Status / readiness projection (system, database, secret, provider, daemon, pi) | AVAILABLE | `GET /personal/status` ≡ `/personal/readiness`; per-component `ready|degraded|blocked|not_configured`; `first_conversation_ready` |
| Provider readiness honesty (`secret_ref_resolves`, `provider_secret_unresolvable`) | AVAILABLE | P2-T11 semantics (`readiness.rs:608-680`) |
| Doctor detail (facts/guidance per component) | PARTIAL | core components live; **six_resource/headless_vault/operability sub-sections are static placeholders over HTTP** (`readiness.rs:236-313`) |
| dsh runtime inspection | AVAILABLE | `GET /personal/dsh/runtime` |
| Backup (secret-excluding, digest-bound) | AVAILABLE | `POST /management/resource/v1/backup[/preflight]` |
| Restore (verified, live apply) | AVAILABLE | `POST /management/resource/v1/restore` (409 tampered/incompatible/…) |
| Product upgrade/rollback/uninstall | PARTIAL | admin-cli + installer library paths (P7-T01/P7-T02 evidence); no HTTP surface |
| Session logout / revoke | **NOT AVAILABLE** | in-process sessions; restart clears all |
| Daemon start/stop/restart from UI | **NOT AVAILABLE** | host-service concern; CLI/systemd only |
| Windows/macOS product | **NOT AVAILABLE** | Linux x86_64 only; B01-W never ran |
| Multi-user / RBAC / remote access | **NOT AVAILABLE** | single-owner loopback by design (permanent product boundary, not a gap) |

## 11. Configuration

| Capability | Rating | Evidence |
|---|---|---|
| SecretStore backend selection (`cognitive init`) | PARTIAL | CLI/install-time only; no HTTP surface; desktop Secret Service implemented, headless encrypted vault designed-not-selectable |
| Workspace (Standard Workspace selection) | PARTIAL | install/init flow concern; no HTTP management route found |
| Extended Home grants (bounded path/operation grants) | PARTIAL | designed + preview-governed in Task flow; no standalone HTTP CRUD found |
| Provider/model/budget/alert configuration | AVAILABLE | §8 |
| Tool availability policy (enable/disable/quarantine) | AVAILABLE | §7 |
| Context authorization facts / revocations admission | AVAILABLE | `POST /management/context-authorization/{facts,revocations}` (daemon-admin) |
| UI preferences / settings (theme, density, notifications) | **NOT AVAILABLE** | no settings surface, no preference API |
| Fault profiles / pinned HTTP origins | **NOT AVAILABLE (product)** | campaign-gated test hooks (`PERSONAL-PERF-EVAL-*` only) — must never appear as product controls |

## 12. Explicitly-checked lifecycle matrix (summary)

| Lifecycle verb | Agent (HTTP) | Task (HTTP) | Tool (HTTP) | Skill (HTTP) | Memory (HTTP) |
|---|---|---|---|---|---|
| create/install/import | NOT AVAILABLE (CLI/store only) | admit ≡ create (AVAILABLE) | n/a (static registry) | import AVAILABLE | remember AVAILABLE |
| list/inspect | PARTIAL | PARTIAL | AVAILABLE | AVAILABLE | PARTIAL |
| enable/activate | NOT AVAILABLE | n/a | AVAILABLE | PARTIAL (via bind) | n/a |
| disable/deactivate | NOT AVAILABLE | n/a | AVAILABLE | PARTIAL (via revoke) | n/a |
| pause | NOT AVAILABLE | NOT AVAILABLE | n/a | n/a | n/a |
| resume | NOT AVAILABLE | NOT AVAILABLE | n/a | n/a | n/a |
| stop/cancel | NOT AVAILABLE | NOT AVAILABLE | n/a | n/a | n/a |
| quarantine | NOT AVAILABLE | n/a | AVAILABLE (no un-quarantine) | n/a | n/a |
| revoke/delete/forget | binding remove AVAILABLE | NOT AVAILABLE | revoke AVAILABLE (terminal) | revoke AVAILABLE | forget AVAILABLE (tombstone) |
| retry | n/a | NOT AVAILABLE | n/a | n/a | n/a |
| upgrade/rollback | NOT AVAILABLE (CLI only) | n/a | n/a | supersede AVAILABLE | n/a |

## 13. Capability-honesty risks the redesign must actively handle

1. **200-stub fallthrough**: unmatched `POST /management/*` and `/task/*` return HTTP 200 with a stub note — a naive UI would render success for operations that do not exist. Any new UI must whitelist known routes and treat unknown 200-stubs as `not-run`. (`server.rs:1086-1095`, `task_api.rs:346-356`)
2. **Three error envelope shapes** across front-door / task-resource / backup domains — the UI needs one normalizing layer before design assumes consistent error rendering.
3. **Plane inconsistency**: the Resource Manager/projection plane and the authority plane disagree (e.g. projection families `not-backed` while authority reads exist; context/runtime lists empty while runtime inspect works). The redesign should consume the *authority-backed* surfaces and treat projection-plane availability strings with suspicion.
4. **Inert watch**: resource watch never publishes deltas; task watch snapshot is always empty. Any "live" UI pattern must be validated against what the watch actually emits, or explicitly scheduled as backend work.
5. **Budgets are advisory**: never render budget UI as a control that prevents spend.
6. **`secret_ref` is serialized** in account responses (opaque but an identifier) — display policy: presence/absence only, never the raw value.
7. **Campaign-gated hooks** (fault-profile, http-origin) are forbidden product UI territory.

---

*Inventory coverage: Agents, Tasks, Runs, Resources, Memory, Skills, Tools, Providers, Activity, System, Configuration + lifecycle matrix. All ratings cite daemon/UI sources read on 2026-08-24; anything not verifiable from those sources is UNKNOWN, not assumed.*
