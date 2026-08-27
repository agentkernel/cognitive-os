# 29 — Real Control Plane Capability Matrix

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Method: every cell rated against **implementation-verified** surfaces (`28` route map + `32` security map + store-level audit where cited). Ratings: **API** = reachable over daemon HTTP (the SPA's only backend) · **CLI** = `cognitive`/admin-cli only · **LIB** = runtime library only · **NONE** = does not exist · **FORBIDDEN** = deliberately refused by design.
- This matrix supersedes nothing in Phase 1 (`control-plane-capability-inventory.md`); it re-keys the same reality to the Phase-3 question: *where can each capability actually be reached?*

## Current implementation (frozen audit baseline)

The matrix below remains the P7-T05 capability evidence at 2026-08-24. Its
API/CLI/LIB/NONE/FORBIDDEN ratings must not be upgraded from target design.
The accepted current SPA presents its implemented subset under seven routes:
Home / Work / Agents / Providers / Resources / Activity / System.

## Personal 2.0 target delta

The target adds product placement and target-only capabilities without changing
these ratings: Providers/System move under Settings; Library holds
Memory/Skills/Tools/MCP; conversation/history and Runtime/Process live under
Agents; Goal/Plan/attempt/Task/Context live under Work; Activity adds four
provenance classes. Unsupported Agent, MCP, Account Hub, orchestration and
writeback features remain `Requires-backend`. The current target gap map is
[Backend Dependency Matrix](37-backend-dependency-matrix.md).

---

## 1. System / readiness

| Capability | API | CLI | Notes |
|---|---|---|---|
| Daemon liveness | API (`/personal/health`, unauth) | yes | |
| Status/readiness (6 components) | API (`/personal/status` ≡ `/readiness`) | yes | doctor sub-sections placeholder over HTTP |
| Doctor detail | API (core) / PARTIAL (sub-sections static) | yes (`doctor --bundle` richer) | |
| dsh runtime snapshot + apply | API (`/personal/dsh/runtime`) | yes | |
| Backup / restore / preflight | API (mgmt) | yes | secrets/SQLite excluded |
| Product upgrade/rollback/uninstall | **NONE over HTTP** | CLI+LIB | |
| Daemon start/stop | **NONE over HTTP** | CLI (`cognitive daemon start|stop`) | host service concern |
| Session logout/revoke | **NONE** | none | in-process sessions; restart clears |

## 2. Work (tasks / runs)

| Capability | Reachability | Notes |
|---|---|---|
| Record intent / interpret / preview / admit | API (task channel) | full governed chain implemented |
| Candidate admission (Pi public candidate) | API | scheduler path |
| Task list | API PARTIAL (`list?family=task`, envelope-only, limit 64, no objective/state) | **BD-3** for a real inventory projection |
| Task detail (state/transitions) | API PARTIAL via `GET /task/evidence` (per task_ref; terminal-oriented) | no general task GET |
| Task watch | API PARTIAL (SSE; process-local 128 ring; snapshot `tasks:[]` empty) | BD-4 |
| Effects history | API (`/task/effects`) | bounded, per task_ref |
| Observation O2/O3/O4/O5/O13 | API (`/task/observation`) | bounded, named zeros |
| Terminal evidence | API (`/task/evidence`) | digest-bound |
| Task consumption pins (memory/skill) | API (`/task/resource/v1/consumption`) | |
| **Task cancel / pause / resume / retry / delete** | **NONE** (FORBIDDEN: `POST /task/cancel`, `/task/complete`) | service method exists without route; BD-1 |
| Run as first-class entity | **NONE** — composed projection (see `30`) | Run timeline composable from evidence+observation+watch |

## 3. Agents

| Capability | Reachability | Notes |
|---|---|---|
| Runtime inventory / inspect | API (`list/inspect?family=runtime`) | envelope depth |
| Binding view/set/remove | API (CAS) | pi/dsh only |
| dsh runtime observe/apply | API | |
| Pi sidecar session state | **NONE over HTTP** (store-internal) | BD-2 |
| Agent lifecycle: install/register/activate/pause/resume/stop/recover/upgrade/rollback/uninstall | **CLI + LIB only** (`admin-cli` store-direct with `PrivilegedManagementSession`) | BD-2; UI renders not-available |
| Agent quarantine | **NONE** (tool quarantine exists; agent-level does not) | |
| Multi-agent | **NONE** (P6 not-started) | |

## 4. Providers

| Capability | Reachability | Notes |
|---|---|---|
| Accounts CRUD + trust reconfirm | API | |
| Key set/rotate/remove | API (secret-bearing body → SecretStore) | one-way; never read back |
| Models refresh/add/set-price/list | API | failed refresh preserves catalog |
| Bindings set/remove/list (CAS) | API | |
| Usage / budgets / alerts / audit | API | budgets observe-only (BD-8); no query filters |
| Provider proxy (unary + SSE) | API | agent-facing; SPA does not call |
| Capability probe beyond discovery | PARTIAL (designed; UI renders not-run) | |

## 5. Resources (Memory / Skill / Tool / Context)

| Capability | Reachability | Notes |
|---|---|---|
| Memory remember/forget/explain | API (mgmt) | retention cap; tombstones |
| Memory list | API PARTIAL (envelope, limit 64) | |
| Memory content search | **NONE over HTTP** (FTS is daemon-internal) | BD-6 |
| Memory proposal review queue | UNKNOWN→**no route found** | BD-6 |
| Skill import/supersede/bind/revoke/explain | API | route-order fix landed (P2-T11) |
| Tool catalog/exposure/selection | API | selection is task-channel, digest-gated |
| Tool enable/disable/quarantine/revoke | API (mgmt) | quarantine one-way; revoke terminal |
| Tool production dispatch chain | PARTIAL (executors assembled; "production call chain does not exist yet" — `tool_executor/mod.rs:48-52`) | |
| Context view browse | **NONE standalone**; per-task via consumption/evidence facets | |
| Resource projection/watch | API PARTIAL (projection plane `not-backed` for memory/skill/context; watch inert) | plane inconsistency flagged |
| Common envelope mutations | API (skill bind/unbind/revoke; tool enable/disable/revoke) | generic verbs FORBIDDEN by design |

## 6. Activity / events / evidence

| Capability | Reachability | Notes |
|---|---|---|
| Provider-plane audit | API (`/management/audit`) | no filters |
| Per-task event/transition history | API PARTIAL (`/task/evidence` transitions; O13 replay per task) | |
| Cross-domain unified activity feed | **NONE** | BD-5 |
| Management-action audit (non-provider) | **NONE** | |
| Live push to UI | **NONE** (SSE exists but process-local + empty snapshot; SPA polls manually) | BD-4 |
| Alerts delivery | pull-only (`/management/alerts`) | |

## 7. Configuration

| Capability | Reachability | Notes |
|---|---|---|
| SecretStore backend selection | CLI (`init`) | no HTTP |
| Workspace / Extended Home grants | PARTIAL (preview-governed in task flow; no standalone CRUD found) | |
| Context-authorization facts/revocations | API (mgmt) | |
| UI preferences | **NONE** | client-local only (acceptable) |
| Campaign hooks (fault-profile/http-origin) | API but **campaign-gated** — never product UI | FORBIDDEN in product surface |

## 8. Matrix summary for Phase-3 planning

- **Fully API-backed spaces:** Providers, System (readiness core), Work creation chain, per-task Work detail (evidence/effects/observation), Resources (memory/skill/tool cores).
- **API-partial spaces:** Work inventory (BD-3), Agents (BD-2), Activity (BD-5), live updates everywhere (BD-4).
- **CLI-only domains (UI must link, not fake):** agent lifecycle, product upgrade, daemon service control, SecretStore backend selection.
- **Forbidden/absent by design:** generic resource verbs, browser task completion, campaign hooks, multi-user anything.

---

*Dependencies register verification: `37-backend-dependency-matrix.md`. Design conflicts arising from this matrix: `38-phase2-design-challenges.md`.*
