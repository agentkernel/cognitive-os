# 31 — Agent Reality Map

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Question (brief §10): which Agent concepts are persistent domain entities / runtime entities / observations / projections / CLI concepts / documentation concepts — and which lifecycle operations are reachable where. Sources: `crates/cognitive-store/src/installation.rs`, `crates/cognitive-runtime/`, `apps/admin-cli/src/main.rs`, `apps/kernel-server/src/personal/`.

## Current implementation (frozen audit baseline)

The identity and lifecycle tables below remain the P7-T05 as-of audit. They
prove limited runtime/binding/dsh projections and CLI/library lifecycle—not a
browser Agent workspace.
The accepted current SPA exposes the bounded dossier under Agents and does not
embed native conversation/history or typed lifecycle controls.

## Personal 2.0 target delta

Agents now targets Adapter-backed embedded conversation/history, a common
capability matrix with display/artifact-only native slots, signed catalog
install/connect in at most three steps, first-real-chat success, Runtime
placement, managed-Work links, and distinct disconnect/uninstall. None of those
target additions is established by the facts below; absent projections/actions
remain `Requires-backend`. The current identity discipline is preserved in
[Agent Spec](16-control-plane-agent-spec.md).

---

## 1. The eight identities, classified

| Identity | What it is (verified) | Class |
|---|---|---|
| Package | immutable distribution + provenance; `agent_package_manifest` contract binding exists (`contracts-ts` generated) | **persistent** (installation DB: `installation_staging`, `installations`, root bindings — `installation.rs:18-78`) |
| Installation | verified private bytes + acquisition lock; append-only installation tables + quarantine | **persistent** |
| Registration | policy + installation/sidecar binding; `agent_registrations` (immutable), `current_agent_registrations` | **persistent** (`installation.rs:85-121`) |
| Instance | supervised logical runtime; `agent_instances(instance_id, registration_id, lifecycle_state, fencing_epoch)` | **persistent + runtime** |
| Sidecar | `sidecar_sessions(session_id, instance_id, protocol_digest, fencing_epoch, lifecycle_state)`, `current_sidecar_sessions` | **persistent + runtime** |
| Execution | AgentExecution — registered governed domain (9 states `CREATED, ADMITTED, RUNNABLE, WAITING, CHECKPOINTED, RECOVERING, SUSPENDED, QUARANTINED, TERMINATED`; `agent-execution.transitions.json`) | **persistent domain** — but instantiated by the management-plane library/conformance; **daemon production-path usage UNKNOWN** |
| Process | `sidecar_process_attempts` + `current_sidecar_process_bindings`; **no OS PID is ever stored** (`installation.rs:146-151`) | **persistent observation binding** (attempt records; liveness is observation) |
| Shell session | Pi-hosted conversational session | **runtime entity** (client-side; separate channel credential) |

Lifecycle vocabularies (plain TEXT columns — deliberately no registered transition table, per D-020 note `installation.rs:1-8`):
- Instance: `registered → active → paused / stopped`; recovery `paused|stopped → active` (`installation.rs:1208, 1385, 1651-1659, 1735, 1785`).
- Sidecar session: `active → paused/stopped/recovered` via fencing (`:2160-2196`).
- Process attempt: `bound → cleared` (`:2223-2278`).
- Library adapter handles (in-memory): `Registered|Active|Paused|Stopped` (`agent_adapter_manifest.rs:49-55`); dsh adapter: `Registered|Active|Stopped` (`deepseek_harness.rs:66-71`).

## 2. Lifecycle operations — where each is reachable

| Operation | API (HTTP) | CLI (`admin-cli`) | Backend library | UI today |
|---|---|---|---|---|
| install | **none** | `install` | `installer.rs:1104` `install_package_durable` | none |
| register | **none** | `register` | `agent_registration.rs:37-390` | none |
| activate | **none** | `activate` / `activate-root` | `installation.rs:1309`, `installer.rs:866` | none |
| pause | **none** | `agent-pause` | `installation.rs:1731` | not-run label |
| resume | **none** | `agent-resume` | `installation.rs:1805` | not-run label |
| stop | **none** | `agent-stop` | `installation.rs:1739` | not-run label |
| restart | **none** | — (stop+activate composition) | — | not-run label |
| recover | **none** | `agent-recover` | `installation.rs:1581` | not-run label |
| quarantine | **none** (agent-level) | — | `installation.rs:979` (installation-root quarantine) | not-run label |
| health observation | **none** | `agent-health` | `installation.rs:1521` `observe_agent_health` | none |
| upgrade/rollback/uninstall | **none** | `rollback`, `uninstall` | `installer.rs:947` etc. | none |
| configure (Pi/dsh) | **none** | `cognitive pi configure` / `dsh configure` | runtime config writers | none |
| bind (provider) | **API** (`/management/agent-bindings`, CAS) | `agent binding …` | provider store | yes (Bindings page) |

**Zero `/agent/*` or `/installation/*` routes exist in the daemon dispatch** (`server.rs:656-943` — confirmed by full-route audit). The two HTTP-visible agent-adjacent surfaces are: provider `agent-bindings` and the **dsh runtime snapshot** (`GET /personal/dsh/runtime`: state ACTIVE/INACTIVE/CRASHED, per-session `{session_id,state,fencing_epoch,last_sequence,task_ref}`, process liveness — `task_api.rs:766-803`).

## 3. Consequences for the Agent dossier (design `16`)

| Dossier section | Reality backing | Honesty treatment |
|---|---|---|
| Overview (7 identity cards) | installation DB facts via `inspect?family=runtime` envelope — **depth limited to what the envelope carries** | source-labeled cards; S7 where facets are not projected |
| Current work | dsh: runtime snapshot (real). Pi: **no HTTP projection** (sidecar session is store-internal) | Pi renders S7 "not observable over HTTP (BD-2)" — never inferred from process |
| Binding | fully real (CAS) | actionable via Providers flow |
| Capabilities | tool exposure is task-scoped (`tool/exposure?task_ref=`) | per-task honest; standing annotation content≠permission |
| Activity/Evidence | per-task projections only; no per-agent feed | actor slice is best-effort from session-observed tasks (labeled) |
| Controls | CLI-only (table above) | class-C text + exact CLI verbs (DD-08) |

## 4. P6 headroom

Multi-agent policy/scheduler/orchestration is P6-T01..T04, all `not-started`, default-off ("NO-GO 是合法结果"). The AgentExecution domain exists in the management library as the registration point for future execution supervision. No UI is designed for it in this phase; the IA reserves the grammar (agent identity on every work row) per `08` §6.

---

*Cross-checks: capability matrix `29` §3; traceability `35` §5; backend dependency BD-2 in `37`.*
