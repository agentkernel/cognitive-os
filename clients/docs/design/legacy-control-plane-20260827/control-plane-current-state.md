# Control Plane Current State Map

- Status: `P10-T01` adopted Personal 2.0 current/target alignment
- Document revision: 2026-08-27
- Change class: `product-semantic` documentation; no implementation, public
  contract, Gate, release, Profile, or task-status change
- Current implementation:
  [P7-T05 closure](../../../docs/checkpoints/20260826-personal-p7-t05-control-plane-redesign-closure.md),
  merged PR [#274](https://github.com/agentkernel/cognitive-os/pull/274) at
  `main@5996afbb` with product SPA head `872074bf`
- Adopted target:
  [ADR-0056](../../../docs/adr/0056-personal-2-0-desktop-control-plane.md) and
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)
- Frozen audit body: read-only 2026-08-24 audits of
  `D:\cognitiveos-clients\pc\web\` (branch
  `personal/P7-T05-dsh-binding-cas` @ `0320c1a`, `main` @ `db56374`),
  `D:\agent-kernel` daemon/management/contract sources, and then-canonical
  product/architecture/plan documents. Historical claims below retain their
  original citations; unverifiable items remain `UNKNOWN`.
- Authority note: this document aligns current evidence and adopted target
  semantics. It does not modify product code, contracts, or status ledgers.

## Current implementation (frozen P7-T05 evidence)

The accepted current Control Plane uses seven routes:
Home / Work / Agents / Providers / Resources / Activity / System. This claim is
pinned to the
[P7-T05 closure](../../../docs/checkpoints/20260826-personal-p7-t05-control-plane-redesign-closure.md):
merged PR [#274](https://github.com/agentkernel/cognitive-os/pull/274) at
`main@5996afbb`, product SPA head `872074bf`, with required CI
[32942980183](https://github.com/agentkernel/cognitive-os/actions/runs/32942980183)
successful at `b147711a`.

The entire body is retained as an as-of 2026-08-24 pre-closure audit at the
exact revisions above; its older Bindings/Tasks/Session route observations, API
limitations, strengths, problems, and citations remain historical truth. Later
P7-T05 integration or Personal 2.0 target choices must not be rewritten into
those observations.

## Personal 2.0 target delta

The adopted target is a distinct redesign:

- IA: Home / Agents / Work / Library / Activity / Settings;
- Providers and System under Settings;
- desktop-primary three-region shell with global Agent Shell;
- Adapter-backed embedded native conversation/history and capability matrix;
- explicit Manage with Personal into daemon Goal/Plan
  revisions/Tasks/attempts and multi-Agent orchestration;
- seven-family task-oriented placement: Library has Memory/Skills/Tools/MCP,
  Work has Context/Task, and Agents has Runtime/Process;
- Account Hub acquisition tiers with ADR-0055 consent and daemon
  SecretStore/proxy custody;
- federated observation and governed writeback;
- one Native/Observed/Governed/Verified timeline.

These are target semantics, not corrections to the current-state evidence.
Where no verified backend support exists they are `Requires-backend`, with no
active fake control or fake progress. Controlling target specs:
[Product Model](01-control-plane-product-model.md),
[Recommended IA](06-control-plane-recommended-ia.md), and
[Target Traceability](35-design-to-code-traceability.md).

---

## 1. Current Architecture

### 1.1 System shape

CognitiveOS Personal is a single-owner, local "operating system for cognitive resources": a Rust daemon is the sole authority writer over six resource families (Memory, Skill, Tool, Context, Task, Runtime/Process); Pi-hosted Agent Shell, `cognitive` CLI, SDK, and the Web UI are clients (`docs/product/personal/product-design.md:17-25`, `docs/architecture/personal/README.md:17-46`).

The Control Plane Web UI is a **static React SPA served same-origin by the daemon** under `GET /ui/` from `<data_dir>/ui` on numeric loopback (product port `48181` when started via `cognitive daemon start`), per ADR-0053 (`docs/adr/0053-personal-web-ui-stack.md:43-64`; serving implementation `apps/kernel-server/src/personal/server.rs:2943-2989`).

```text
Browser (static SPA, loopback, HashRouter)
   |  Authorization: Bearer <channel-token>  (memory-only; no cookies)
   |  Origin/Referer must equal daemon loopback origin (LOCAL_ORIGIN_HEADER_REJECTED)
   v
Personal daemon front door (hand-rolled HTTP/1.1, std::net::TcpListener,
   one thread per connection, Connection: close; loopback-only bind;
   single-instance lock; body <= 1 MiB; <=32 conns / <=16 in-flight)
   |
   +-- Session:        POST /local/session (bootstrap secret -> channel bearer)
   +-- Personal:       /personal/{health,status,readiness,doctor}, /personal/dsh/runtime
   +-- Provider CP:    /management/providers/*, /management/agent-bindings,
   |                   /management/{usage,budgets,alerts,audit}
   +-- Provider proxy: /provider/v1/chat/completions (+ SSE stream), /provider/v1[/dsh]/selected-model
   +-- Task channel:   /task/{intent.record,intent.interpret,preview,admit,candidate,
   |                   akp/dsh,watch,evidence,effects,observation}
   +-- Resources:      /resource/v1/{projection,watch}; /management/resource/v1/{list,inspect,
   |                   bind,unbind,enable,disable,revoke} + memory/skill/tool/backup/fault/http-origin
   +-- Static:         GET /ui[/...]  (unauthenticated, CSP default-src 'self')
   v
Authority SQLite / Event log / SecretStore / Provider egress (daemon only)
```

### 1.2 Trust and session architecture

- Two disjoint channel classes: `task` and `management`; a bearer presented on the wrong channel fails with 403 `SHELL_CHANNEL_BINDING_MISMATCH` (`apps/kernel-server/src/personal/auth.rs:13-34, 340-342`). Management channel == daemon-administrator boundary (`auth.rs:356-369`).
- Sessions are minted by `POST /local/session` from the bootstrap secret (file `local-bootstrap.secret`), live **in daemon process memory only** (restart invalidates all), expire after 12 h absolute / 30 min idle (`auth.rs:155-163`; `bounds.rs:33-34`). No logout/revoke endpoint exists.
- SPA holds bearers in JS memory only; localStorage/sessionStorage/IndexedDB/URL/history persistence is forbidden and self-checked (`pc/web/src/session.ts:3-30`; ADR-0053 §3).
- The browser never receives Provider keys, resolvable SecretRefs, sidecar bearers, or filesystem credentials; Provider key material travels only in the `POST /management/providers/accounts/key` body into the daemon-approved SecretStore (ADR-0053 §3; `web-ui-architecture.md:173-189`).
- Front door rejects cookies (403 `LOCAL_COOKIE_AUTH_FORBIDDEN`), non-loopback Host (400), non-loopback Origin/Referer (403) (`server.rs:589-598, 2781-2802, 2853-2894`).

### 1.3 Related but separate surfaces (do not conflate)

| Surface | What it is | Status |
|---|---|---|
| Personal Web UI (`/ui/`) | The Control Plane SPA audited here; `cognitiveos-clients/pc/web/` | D01–D09 accepted; D10 Apple-theme refinement **blocked** on client-repo write access (PROGRESS.md Current snapshot) |
| dsh native web panel | DeepSeek Harness's own SPA via `cognitive dsh web`, `http://127.0.0.1:3080`, dsh-owned; **not Personal `/ui/`** (P8-T15 report §170-239) | done (PR #265) |
| Pi-hosted Agent Shell | Primary conversational entry; client of the same daemon services | implemented (one-shot, text-only) |
| `cognitive` CLI | Deterministic operator client; same application services | implemented |
| `apps/cognitiveos-console` | Deprecated compatibility stub; implementation forbidden here (ADR-0053) | stub |

---

## 2. Current WebUI

Source: `D:\cognitiveos-clients\pc\web\` (audited read-only).

| Concern | Reality | Evidence |
|---|---|---|
| Framework | React 18.3.1, `StrictMode` | `pc/web/package.json:17-18` |
| Build | Vite 5.4.11, `base:"/ui/"`, deterministic static bundle, no sourcemap | `pc/web/vite.config.ts:4-11` |
| Language | TypeScript 5.6.3 strict | `pc/web/tsconfig.json:12-17` |
| Styling | One hand-written global CSS file (155 lines), dark-only `color-scheme: dark`, ~10 utility classes; no Tailwind/design system/tokens | `pc/web/src/styles.css:1-14` |
| Routing | `react-router-dom` 6.28 **HashRouter only** (daemon has no SPA fallback; `/ui/providers` 404s, only `/ui/#/providers` works) | `src/App.tsx:2,1402`; P8-T15 report `:170-239` |
| State | No store library; per-page `useState` + module-level session memory + one context tick | `src/App.tsx:108-114`, `src/session.ts:3-7` |
| Tests | Vitest + jsdom, 9 unit/DOM test files; no network-layer tests, no e2e | `pc/web/vite.config.ts:12-15` |
| Code shape | **Single-file app**: all 10 pages + shell in `src/App.tsx` (~1485 lines); logic modules: `api.ts`, `channels.ts`, `session.ts`, `policy.ts`, `probe.ts`, `taskDraft.ts`, `watch.ts`, `watchSse.ts`, `identities.ts` | `src/App.tsx` |
| Serving | `dist/` copied to daemon `data_dir()/ui`; CSP `default-src 'self'` (+`'unsafe-inline'` styles) | `pc/web/README.md:17-18`, `index.html:6-9` |

Visual state: dark theme only (`--bg #10141c`, `--panel #1a2130`, `--ink #e8edf5`, `--accent #7eb6ff`), `"Segoe UI", system-ui` stack, no motion, no elevation, no badges/chips, no light theme. The accepted D10 "Apple-inspired" visual refinement exists **only as an unpublished Git bundle** (SHA-256 `02a0216f…641e`); no trace of it exists in the clients repo branches (UNKNOWN beyond the bundle).

---

## 3. Current Navigation

Flat left sidebar (`nav.side`, `App.tsx:129-144`), no grouping/icons/badges/collapse:

1. Home
2. Agents
3. Providers
4. Bindings
5. Tasks
6. Activity
7. Resources
8. Session (appended after the main map)

Facts:

- Active state = bold font via `aria-current="page"` only (`styles.css:68`).
- No top bar, command palette, global search, notification surface, settings page, user menu, or breadcrumbs.
- One responsive breakpoint (720px) stacks sidebar above content (`styles.css:151-154`).
- Skip-to-content link exists (`App.tsx:119-128`).
- Unknown hash routes render an empty main area (no `*` route).

Deviation from the canonical product IA (`product-design.md:202-224`: Home / Agents / Tasks / Resources / Activity): the shipped nav adds **Providers**, **Bindings**, and **Session** as top-level peers. `web-ui-design.md:41-48` intended Provider management as "a dedicated operator view/shortcut reachable from Home and Agents (and may be grouped under Resources)", not a sixth space; Bindings has no canonical IA position at all.

---

## 4. Current Pages

All routes in `App.tsx:1400-1484`; every page except Session is wrapped in `RequireSession` (inline session gate, per the sidebar-fix branch).

| Route | Page | Channel | What it does today |
|---|---|---|---|
| `#/` | HomePage (`254-304`) | mgmt | Fetches `/personal/health` (unauth) + status/readiness/doctor; renders 4 status lines (`status · ms · message`) + 2 raw JSON panels (readiness, doctor). No actions, no auto-refresh. |
| `#/agents` | AgentsPage (`320-381`) | mgmt | Table (Instance, Package, Status, Inspect link) from `resource/v1/list?family=runtime` + bindings + dsh runtime JSON panels. Lifecycle verbs shown as `not-run` labels (`339-344`). |
| `#/agents/:id` | AgentDetailPage (`383-427`) | mgmt | 9 identity cards (package, installation, registration, instance, sidecar, execution, process, task, shell_session) + "Typed lifecycle" panel listing five `not-run` labels + raw inspect JSON. No actions. |
| `#/providers` | ProvidersPage (`429-586`) | mgmt | Create-account form (name, kind, endpoint, trust checkboxes with conditional confirmation) + table (Id, Name, Kind, Status, Secret present/absent, Open). |
| `#/providers/:id` | ProviderDetailPage (`588-785`) | mgmt | Info panel; key set/rotate/remove via daemon; bounded model/capability probe (`models/refresh`); manual model add; delete account (binding-blocked); catalog table with honest cost display. Capability always rendered `not-run` (`721`). |
| `#/bindings` | BindingsPage (`787-1091`) | mgmt | Heaviest page: set fixed binding (agent select hardcoded to `pi`/`dsh`; account; model; CAS expected revision; required confirm checkbox; two deliberate negative checkboxes "Request fallback"/"Per-request override" that must be rejected); active-bindings table with Dispatch callable/blocked + per-row Remove; "Apply Cos model to running dsh" section (fail-closed gate, `POST /personal/dsh/runtime op=apply`). |
| `#/tasks` | TasksPage (`1093-1341`) | **task** | Objective input (default `"search the workspace for needle"`) → chained `intent.record` → `intent.interpret` → `preview` → "Confirm admit" (`admit`); then task_ref-scoped: load effects/observation/evidence, manual "Watch poll" (SSE fetched as text, parsed offline), "Reconnect snapshot", "Simulate cursor gap", "Detach observation". Cancel = `not-run` (`1259`). |
| `#/activity` | ActivityPage (`1343-1368`) | mgmt | Four raw JSON panels: usage, budgets, alerts, audit. No filters/pagination/actions. |
| `#/resources` | ResourcesPage (`1370-1398`) | mgmt | Family select over six hardcoded families → one raw JSON list panel. No inspect links, no actions. |
| `#/session` | SessionPage (`240-252`) | none | Principal + bootstrap-secret (non-echoing) → two `POST /local/session` (management + task); "Clear memory session". Copy stresses the secret is not a Provider key. |

State model: one shared `LoadState` union `loading|ready|empty|denied|disconnected|unknown|not-run` (`App.tsx:27-32`) rendered as a single muted text line (`StateNote`); no spinners, skeletons, retry buttons, or error banners.

---

## 5. Current Components

There is no component library. Shared elements (all in `App.tsx`):

- `Shell` (layout + sidebar), `RequireSession` (channel gate), `SessionForm` (reused by SessionPage and every gate), `StateNote` (status line), `JsonPanel` (titled `<pre>` of `redactSecrets(value)` — **the dominant display "component"**, used 10+ times), `SessionScope`/`SessionTick` context.
- Inline HTML `<table>`, `<form>`, `<section class="panel">` per page; no reusable table/card/badge/dialog/toast/modal components. **No modal anywhere; confirmations are checkbox-based.**
- Logic modules (no UI): `api.ts` (readJson, session issue, header-injection rejection), `channels.ts` (path→channel classification, bearer injection), `session.ts` (memory store + storage self-check), `policy.ts` (redaction, CAS derivation, dispatch/apply gates, cost display, untrusted-text escaping), `probe.ts` (probe error classification, provider kinds, trust gate), `taskDraft.ts` (uuidV7, fixed workspace-search draft builder), `watch.ts` (watch controller state machine), `watchSse.ts` (SSE frame parser + resume-stale detection), `identities.ts` (9-key agent identity merge).

---

## 6. Current Data Model

### 6.1 As the SPA models it (client-side)

All API data is coerced via untyped `asRecord`/`asList` helpers (`App.tsx:44-58`); list extraction probes keys `items/accounts/bindings/events/alerts/models`. Modeled objects:

- **Provider account**: `id, display_name, provider_kind, status, endpoint, network_scope, catalog_revision, secret_ref (rendered present/absent), last_discovery_error`
- **Model catalog entry**: `model_id, source, price_input_per_million, price_output_per_million` (unknown cost never shown as 0 — `policy.ts:65-73`)
- **Agent binding**: `agent, account_id, model_id, revision, status` + derived CAS revision and dispatchability
- **Runtime resource (agent)**: `id, status/lifecycle` + 9 merged identity fields
- **dsh runtime**: `state (ACTIVE|INACTIVE|CRASHED), process_alive`; selected model + snapshot digest
- **Task draft** (client-built, fixed shape): `WorkspaceSearchDraft` — `allowed_tools:["native.workspace.search"]`, budget 4/4, `max_retries:0`, `expected_current_epoch:0`, hardcoded deadline `2027-12-31` (`taskDraft.ts:39-65`)
- **Task projections**: effects / observation / evidence (opaque JSON), watch frames, `WatchState = live|stale|disconnected|reconciling|unknown`
- **Activity**: usage/budgets/alerts/audit — opaque JSON, no field-level modeling

### 6.2 As the daemon exposes it (API-visible authority shapes)

- **Task**: never returned whole. Visible via `TaskAdmitResult{task_ref, task_contract_ref, contract_digest, contract_epoch}`; Resource Manager task envelope `{id, object_version:contract_epoch, health:"contracted", allowed_actions:["inspect"]}`; evidence `lifecycle.current_state/current_version + transitions[]`. Store lifecycle: `ACTIVE → CANDIDATE_COMPLETE → COMPLETED` (`crates/cognitive-store/src/sqlite/continuation.rs:435,483`). **No DRAFT state is API-visible; no list-tasks API except Resource Manager `list?family=task` (limit 64).**
- **Effect**: stages `NOT_EXECUTED|DENIED|PROPOSED|AUTHORIZED|EXECUTING|EXECUTED|RECONCILED|VERIFIED|VERIFY_FAILED|OUTCOME_UNKNOWN` (+ synthetic `MISSING`), derived `outcome_class` and `reconcile_class` (`observation.rs:761-831`).
- **Agent/SidecarSession**: **not exposed over HTTP.** Store record `{session_id, instance_id, protocol_digest, fencing_epoch, lifecycle_state, process_attempt_id?}`; runtime adapter states `Registered|Active|Paused|Stopped` (`crates/cognitive-runtime/src/agent_adapter_manifest.rs:50`). HTTP-visible "agent" facts are only: provider `agent-bindings` (agent ∈ pi|dsh) and the dsh runtime snapshot.
- **Provider account status**: `active|revoked|degraded`; binding revision is a monotonic CAS counter.
- **Resource Manager envelope**: `{id, family, object_version, projection_version:"personal-resource-manager/1", health, owner, scope, revision_digest, blocked_reason, allowed_actions[], typed_bindings[]}` (`resource_manager.rs:485-738`).
- **Memory**: explain shape `{memory_id, candidate_id, decision_id, canonical_json}`; remember/forget only (no update, no content search API).
- **Skill**: binding explain + revision inspect; binding `status:"active"` + `revocation_reason`.
- **Tool**: descriptor `{operation_id, action, descriptor_version, descriptor_digest, risk, executor, required_capability, family, availability, execution_readiness, input/output limits}` + lifecycle overlay `enabled|disabled|quarantined|revoked` (revoked terminal; quarantined→enabled refused) + `agent_exposed`.

### 6.3 Canonical product concepts (documented, not all API-visible)

Six resource families (Memory, Skill, Tool, Context, Task, Runtime/Process) + cross-cutting objects (Budget, Permission, Model, Artifact, Intent/Effect, Evidence, Event); Agent is "a navigation and actor concept composed from Runtime identities, not a seventh family" (`cognitive-resource-model.md:30-47`). Runtime identity decomposition: Package / Installation / Registration / Instance / Sidecar / Execution / Process — seven distinct identities (`cognitive-resource-model.md:249-272`).

---

## 7. Current API

Full inventory audited at `apps/kernel-server/src/personal/server.rs` (`dispatch_http_route` `:645-953`) and subsystem handlers. Summary by domain (auth classes: none / bootstrap / task / mgmt):

| Domain | Routes (method + path) | State |
|---|---|---|
| Session | `POST /local/session` | implemented |
| Health/readiness | `GET /personal/health` (none); `/personal/status`, `/personal/readiness` (identical), `/personal/doctor` (mgmt) | implemented; doctor sub-sections are static placeholders (`readiness.rs:236-313`) |
| Provider proxy | `POST /provider/v1/chat/completions` (+SSE stream), `POST /provider/v1/dsh/chat/completions`, `GET /provider/v1[/dsh]/selected-model` | implemented |
| Provider CP | accounts list/inspect/create/update/delete/key; models list/refresh/add/set-price; agent-bindings list/set/remove; usage; budgets list/set/remove; alerts list/acknowledge; audit | implemented; budgets **observe-only** (no enforcement hook) |
| dsh runtime | `GET|POST /personal/dsh/runtime` (ops bind/heartbeat/clear/apply) | implemented |
| Task channel | `intent.record`, `intent.interpret`, `preview`, `admit`, `candidate`, `akp/dsh`, `GET watch` (SSE, process-local, snapshot `tasks:[]` always empty), `GET evidence`, `GET effects`, `GET observation` | implemented; **no cancel/pause/retry/stop** |
| Resources (private projection) | `GET /resource/v1/projection`, `GET /resource/v1/watch` (+ `/task/resource/v1/*` twins) | implemented; **watch inert after startup** (only `projection.initialized`); memory/skill/context families self-declare `not-backed` |
| Resources (authority) | memory object explain / remember / forget; skill import / bind / binding revoke / binding explain / revision inspect | implemented (mgmt only) |
| Resource Manager envelope | `list`, `inspect`, `bind`/`unbind`/`revoke` (skill), `enable`/`disable`/`revoke` (tool); generic `create/install/execute/complete` **deliberately refused** (400 `RESOURCE_MANAGER_OPERATION_FORBIDDEN`) | implemented |
| Tool lifecycle | catalog/discover (both channels), exposure, selection receipt (task), enable/disable/quarantine/revoke (mgmt) | implemented; execution-ready ≠ production-wired (`tool_executor/mod.rs:48-52`) |
| Consumption | `POST|GET /task/resource/v1/consumption` | implemented |
| Backup/restore | `backup`, `backup/preflight`, `restore` (mgmt; secrets/SQLite excluded) | implemented |
| Observation plane | `GET /task/observation?family=o2|o3|o4|o5|o13` (task channel; named zeros, negative controls, cursor/digest fail-closed) | implemented |
| Campaign-gated | fault-profile, http-origin pin (mgmt; `PERSONAL-PERF-EVAL-*`/pinned task only) | implemented but not product-available |
| Static UI | `GET /ui`, `GET /ui/*` (unauth; 503 `LOCAL_UI_BUNDLE_UNAVAILABLE` when absent; 1 MiB/asset cap) | implemented; bundle external (PARTIAL) |

Explicitly unavailable over HTTP (must render not-run per `web-ui-route-inventory.json`): **task-cancel; agent pause/resume/stop/restart/quarantine**. Forbidden routes (implemented refusals): `POST /task/cancel`, `POST /task/complete`, generic `/management/agent/transition`, `/management/lifecycle`, Resource Manager generic create/install/execute/complete. Cross-channel twins return 403 (`PROVIDER_CONTROL_CHANNEL_FORBIDDEN`, `RESOURCE_MANAGER_CHANNEL_FORBIDDEN`, etc.).

API hygiene issues (design-relevant):

1. **Unmatched `POST /management/*` returns a 200 stub** ("business routes deferred", `server.rs:1086-1095`) and unmatched `/task/*` returns a 200 stub (`task_api.rs:346-356`) — unknown operations look successful. Capability-honesty risk: HIGH.
2. **Three inconsistent error envelopes**: front-door `{status,error{code,message,category,retryable,stage}}` vs task/resource `{status,code,message}` vs backup `{error{code,detail}}`.
3. No PUT/PATCH/DELETE anywhere; all mutations are POST.
4. No unified activity feed, no cross-task event listing, no audit of management actions other than Provider CP (`/management/audit` covers provider events only).
5. `secret_ref` (opaque identifier) is serialized in provider account responses (`provider_control_plane.rs:1276`).
6. Sessions: in-process only, no logout/revoke endpoint; daemon restart drops all sessions.

### 7.1 Operator CLI surface (the UI's sibling client)

`cognitive` (apps/admin-cli, `personal_cli/mod.rs:32-49`) — the deterministic operator client the owner currently uses to route around UI gaps:

| Verb family | Commands |
|---|---|
| Setup | `init` (layout, provider, SecretStore bind, key via file/hidden input) |
| Health | `status`, `doctor` |
| Daemon | `daemon start` (canonical port 48181) / `status` / `stop` |
| Pi agent | `pi configure` / `pi launch` (hermetic runtime root; print mode) |
| dsh agent | `dsh configure` / `launch` / `web` (native panel :3080) / `apply` / `status` |
| Resources | `resource get` / `watch` / `list` / `inspect` / `mutate` (bind, unbind, enable, disable, revoke) |
| Tasks | `task watch` / `task evidence` (read-only observation) |
| Providers | `provider account create/list/show/update/delete`, `provider key set/rotate/remove`, `provider models refresh/list/add/set-price` |
| Bindings | `agent binding set/show/list/remove` |
| Usage/governance | `usage query`, `budget set/list/remove`, `alerts list/acknowledge`, `audit query` |
| Stewardship | `backup`, `restore` |

Privileged agent-lifecycle verbs (install/register/activate/pause/resume/stop/recover/upgrade/uninstall) exist only in admin-cli operating directly on the SQLite store with a `PrivilegedManagementSession` file (`apps/admin-cli/src/main.rs:111-128`) — **no HTTP route**, which is why the SPA renders them `not-run`.

---

## 8. Current User Flows

End-to-end flows the shipped SPA actually supports:

1. **Bootstrap session**: any page → inline gate → paste `local-bootstrap.secret` + principal → two `POST /local/session` → bearers in memory. Sessions die on every page reload (by design).
2. **Check daemon health**: Home → 4 status lines + readiness/doctor raw JSON. Load-once; no auto-refresh.
3. **Configure a Provider**: Providers → create form (kind/endpoint/trust) → open detail → paste API key → set/rotate via daemon → optional bounded probe (`models/refresh`) → optional manual model add → catalog table. Cleanup: remove key (revokes), delete account (blocked by active bindings).
4. **Bind an Agent**: Bindings → pick pi/dsh → account → model → CAS revision auto-filled → confirm checkbox → submit (fallback/per-request checkboxes exist only to be rejected) → table shows callable/blocked → per-row Remove.
5. **Apply model to running dsh** (current branch): Bindings → fail-closed gate → `POST /personal/dsh/runtime {op:apply, expected_revision}`.
6. **Run a governed task**: Tasks → edit objective → record→interpret→preview (3 chained POSTs) → inspect preview digest → admit → task_ref → load effects/observation/evidence → manual watch poll / reconnect / simulate gap / detach (detach never cancels).
7. **Inspect agents**: Agents → table → Inspect → identity cards + raw JSON. No lifecycle control (all five verbs `not-run`).
8. **Review activity**: Activity → four JSON dumps. Resources → family select → JSON dump.

Flows that **do not exist**: listing/searching existing tasks (must know a `task_ref`), cancelling a task, any agent lifecycle action, budget editing, alert acknowledgement, notifications, settings, memory/skill/context inspection beyond raw list JSON, any live/streaming update.

---

## 9. Current Strengths

1. **Capability honesty is implemented, not aspirational.** Unavailable verbs render as explicit `not-run` labels; fallback/per-request override checkboxes exist only to be rejected; completion is never inferred (`inferCompletionFromObservation` always `"unknown"`); unknown cost is never shown as zero; failed model refresh preserves catalog + binding. This matches the frozen route inventory and ADR-0053 §4.
2. **Security posture is real.** Memory-only tokens with a storage self-check; channel binding enforced client- and daemon-side; secret-bearing body rejected in URLs; all responses redacted before render; untrusted text escaped; CSP `default-src 'self'`; Origin/Referer/Host loopback enforcement; no cookies.
3. **The Provider → Binding → Apply journey is genuinely end-to-end** (account create, key handoff to SecretStore, probe, manual model, CAS binding, dispatchability, dsh apply) — the most complete workflow in the product.
4. **Task governance chain is real**: record → interpret → preview (digest) → admit (CAS + principal binding) → evidence/effects/observation projections with honest unknowns.
5. **Watch plumbing exists**: SSE parser, cursor resume, stale detection, gap simulation, detach semantics — the hardest correctness thinking is done, even though the UI only polls manually.
6. **Accessibility seeds**: skip link, `aria-current`, `role="status"` live regions, table captions, focus-visible outlines.
7. **Zero mock data**: every panel is backed by a real daemon call.

---

## 10. Current Problems

1. **Raw JSON is the primary UI.** Home readiness/doctor, Agents bindings/dsh, Tasks effects/evidence/observation, all four Activity panels, and Resources lists are `<pre>` dumps. The product's core value (making authority state *legible*) is not delivered visually.
2. **No task inventory.** There is no way to see "what tasks exist / are running / blocked / finished" — the single most important operator question. Tasks page requires a known `task_ref`.
3. **No live system.** Every page loads once; watch is manual polling of an SSE endpoint; no streaming, no timers, no invalidation. "What is happening right now" is never answered without manual reload.
4. **Navigation drift from the product IA.** Providers and Bindings are top-level peers without canonical IA sanction; Bindings is a sub-capability of Provider/Agent governance; Session is a utility, not a space. Eight flat peers with no grouping.
5. **Agents page cannot answer "what is my agent doing".** No lifecycle state beyond a status string, no current task/run linkage, no activity, no control (all verbs not-run) — it is an identity card, not an operator surface.
6. **Activity page is not activity.** It is four Provider-CP JSON projections (usage/budgets/alerts/audit). Runs, Effects, Evidence, Events — the canonical Activity content — live on the Tasks page behind a manual task_ref query.
7. **Resources page is a stub**: family picker + raw list JSON; no inspect, no memory content, no skill detail, no tool lifecycle actions despite HTTP verbs existing.
8. **Interaction model is form-plus-checkbox.** No modals, no previews with consequence framing, no toasts/receipts, no undo affordances, no keyboard speed paths, no command surface.
9. **State vocabulary is text-only.** `loading|ready|empty|denied|disconnected|unknown|not-run` all render as the same muted line — no visual hierarchy between "blocked, act now" and "not-run, informational".
10. **Session UX cost is high**: paste bootstrap secret on every reload; no expiry display; no session status in the shell.
11. **No notifications/alerts surface**: budget alerts exist in the API but appear only as raw JSON on Activity.
12. **Debug affordances shipped as UI**: "Simulate cursor gap" is a development tool exposed in the product surface.

## 11. Technical Debt

1. **Single-file application** (~1485-line `App.tsx`): all pages, shell, gates, and shared components in one file; no component library; no design tokens; one 155-line global CSS.
2. **Untyped API layer**: `asRecord`/`asList` coercion everywhere; envelope shapes probed by key lists (`items/accounts/bindings/...`); no generated or hand-mapped typed client despite `web-ui-architecture.md:78-81` calling for one.
3. **No network-layer tests**: fetch never mocked; only 4 DOM-level page tests; api/channel/policy/watch logic partially unit-tested.
4. **HashRouter-only with no `*` route**; daemon has no SPA fallback (documented live-operator consequence: `/ui/providers` 404s).
5. **Hardcoded product facts in the client**: agent choices `pi`/`dsh`; fixed workspace-search task draft with hardcoded deadline `2027-12-31`; default principal; six-family list.
6. **SSE parsed offline**: `watchSse.ts` exists but no `EventSource` is ever opened — streaming infrastructure without streaming.
7. **Stale repo docs**: clients-repo root docs still claim "no runnable code" (predate the SPA).
8. **Daemon-side debt that constrains the UI**: 200-stub fallthroughs on unmatched management/task routes; three error envelope shapes; resource watch publishes no deltas after startup; task watch snapshot always empty; doctor sub-sections unprobed over HTTP; projection plane families self-declare `not-backed` while authority reads exist elsewhere (plane inconsistency).
9. **D10 visual refinement unpublished**: accepted Apple-theme work exists only as a Git bundle (SHA-256 `02a0216f…641e`); the served UI remains the pre-D10 dark theme.
10. **No design-system document** for Personal anywhere in either repo.

## 12. Capability Gaps

Gaps the redesign must design *around* (render unavailable / not-run) or escalate as backend dependencies — never fake:

| Gap | Reality today | UI consequence |
|---|---|---|
| Task cancel / pause / retry / stop over HTTP | No route; completion is internal-only (`continuation.rs:428-485`) | Task control renders not-run |
| Agent install/activate/pause/resume/stop/recover/upgrade/rollback/uninstall over HTTP | admin-cli + in-process runtime only (`main.rs:111-128`) | Agents page is read-only |
| Task list/search | Only Resource Manager `list?family=task` (limit 64, envelope-only) | No real task inventory page possible without projection work |
| Live activity feed | Watch is process-local, 128-event ring, empty snapshot; resource watch inert | "What changed" requires polling; no cross-task stream |
| Unified audit | Only Provider CP appends audit rows; memory/skill/tool/backup mutations are not API-audited | Activity/audit views are Provider-scoped only |
| Memory search/list-with-content | list = id envelopes; content via `memory/object?id=` canonical_json | Memory browsing is possible but primitive |
| Budget enforcement | Budgets/alerts observe-only | Budget UI is informational, never a control |
| Agent lifecycle state over HTTP | Only dsh runtime snapshot + bindings; SidecarSession not exposed | Agent status is partial for Pi |
| Session logout/revoke | None | "Clear memory session" is client-side only |
| Doctor sub-sections live probes | Static placeholders over HTTP | Doctor detail beyond core components is not-run |
| Multi-agent | P6 not-started, default-off | No multi-agent UI |
| Browser bootstrap flow | No browser-specific auth; bootstrap secret is file-based; no CORS | Session gate requires manual secret paste every reload |

---

## Appendix: audit provenance

| Source | Scope | Date |
|---|---|---|
| `D:\cognitiveos-clients` worktree `personal/P7-T05-dsh-binding-cas` @ `0320c1a`, `main` @ `db56374`, branch `personal/P7-T05-web-ui-sidebar-fix` @ `adea0b5` | SPA code, tests, docs | 2026-08-24 |
| `D:\agent-kernel` `apps/kernel-server/src/personal/*`, `crates/cognitive-*`, `apps/admin-cli` | daemon HTTP, auth, resources, tools, observation | 2026-08-24 |
| `docs/product/personal/*`, `docs/architecture/personal/*`, `docs/adr/0053`, `docs/plan/*`, `docs/checkpoints/*p7-t05/p8-t13/p8-t15/p2-t28*`, `handbook/en/*` | canonical product/architecture/status | 2026-08-24 |
