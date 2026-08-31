---
doc_id: ref.capability-status
locale: en
kind: reference
audience: [user, developer, ai]
status: implemented
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
  - path: personal/crates/cognitive-store/src/personal_backup.rs
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
  - path: personal/crates/cognitive-secret/src/backend_select.rs
  - path: personal/apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: personal/crates/cognitive-management/src/task_application.rs
  - path: personal/apps/kernel-server/src/personal/capability_truth.rs
    symbols: ["FROZEN_UJ_CAPABILITY_TRUTH", "validate_capability_truth_matrix"]
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/windows-host-background.md
  - path: personal/crates/cognitive-store/src/windows_host.rs
    symbols: ["WINDOWS_HOST_SCHEMA_V34", "WindowsHostStore", "WAKE_RECOVERY_STEPS"]
  - path: personal/docs/architecture/x-twitter-connector.md
  - path: personal/crates/cognitive-store/src/x_connector.rs
    symbols: ["X_CONNECTOR_SCHEMA_V35", "XConnectorStore"]
fingerprint: "sha256:45992e27d8b59d1a5b22645ea71b794c90898fc036e638011314d04086122fac"
non_claims:
  - Statuses are code+contract+test judgments at the recorded baseline, not Gate/release/Profile results and not the formal plan's task states.
---

# Capability status matrix

Legend: `implemented` (real path + tests), `partial` (works with named gaps),
`designed` (contract/design only), `unavailable` (no usable path),
`Requires-backend` (adopted target but no required daemon/API implementation),
`Requires-environment` (qualified native/campaign environment absent), and
`Requires-core` (adopted target also needing approved core contract/authority).

Rows without a Personal 2.0 qualifier describe the current Linux/current-API
baseline: six resource families and Pi as the only qualified Agent. The
same-origin `/ui/` exists at `clients/pc/web/`; its adopted desktop-first
redesign is a separate, unimplemented target. Personal 2.0 is a full-version
commitment, but every missing row remains `Requires-backend` and each platform
and Agent requires independent qualification.

| Capability | Status | The gap, if any |
|---|---|---|
| Linux bundle install/upgrade/rollback/uninstall | implemented | production signing/publication pending |
| systemd user service + health-gated activation | implemented | — |
| `cognitive init` (layout, secrets, discovery, selected model) | implemented | — |
| Daemon loopback HTTP + channel auth + bounds | implemented | OS-CSPRNG tokens; sessions are process-local, daemon restart invalidates them, and logout/introspection routes are absent |
| Provider proxy (unary chat + public SSE) | implemented | Pi/private-candidate stay unary; no disconnect-to-cancel |
| Provider Control Plane (named accounts, bindings, usage) | partial | daemon API + `cognitive` CLI + current same-origin `/ui/`; usage/audit query has no filters; target Account Hub redesign is not implemented |
| SecretStore | implemented (Linux Secret Service; Windows Credential Manager) | headless vault designed; macOS unavailable |
| Account Hub user-directed credential import | Requires-backend | ADR-0055 defines exact-source consent, daemon-only read/write, retention-by-default, and explicit deletion; concrete browser/Agent/subscription/OAuth import mechanisms do not exist |
| dsh runtime inspect | implemented | `/proc` liveness is Linux-only; Windows reports unknown rather than CRASHED |
| Pi shell conversation via daemon | implemented | one-shot, text-only |
| Pi tool use inside shell | unavailable | all built-ins denied by policy |
| Task record/interpret/preview/admit | implemented | — |
| Task watch | implemented | process-local event source |
| Task control/query over HTTP | unavailable | service methods exist, no route |
| Autonomous scheduler loop | partial | public admit persists owner-local Context authorization facts and the tenant `personal` revocation epoch with the runnable row, `START` Loop, and hard Budget; the first scheduler tick walks Loop `START -> DECIDE` from the sealed ContextView then admits one private Pi candidate; a later tick acquires the lease and activates the Task; startup repairs missing members; one post-bind non-reentrant periodic worker reaches candidate admission and production-dispatches WorkspaceRead, WorkspaceSearch, WorkspaceWrite/Patch, ProcessCheck, HttpFetchReadOnly, and `check_id`-only RegisteredCheckRun through the durable Effect protocol; a closed intermediate mutation on a RegisteredCheck-terminated Task returns the Loop to `DECIDE` so a later tick can admit RegisteredCheckRun |
| Governed tool execution (all seven current native Tool-operation families) | partial | all seven Tool-operation families have a production request carrier; WorkspaceRead, WorkspaceSearch, and WorkspaceWrite/Patch dispatch through the periodic caller; ProcessCheck stages through a fail-closed carrier until the supervised-process registry is wired; HttpFetchReadOnly stages through the campaign-authorized pinned-HTTPS registry (empty by default); RegisteredCheckRun dispatches `check_id`-only through the immutable registry and drops Agent exposure when disabled |
| Workspace write/patch executors | implemented, production-called | handle-relative no-follow traversal/publication, bounded preimages, target-locked CAS, workspace-external durable key-bound receipts and restart orphan recovery; payload + expected preimage carried from the persisted Intent; `digest:sha256:<raw file SHA-256>` is an equivalent CAS token to the domain-tagged workspace-image digest; verification is not requested while the Effect is still pending reconciliation |
| Independent verification and Task acceptance | implemented; public C1 native-proven | production WorkspaceRead and RegisteredCheckRun reach registered independent verifiers; RegisteredCheck requires exact CAS Evidence, descriptor/file digests and clean safety observations before a passed report, checkpoint, one-time continuation authority and Loop `OBSERVE`; WorkspaceRead reaches a CAS-backed passed report and evidence-bound `COMPLETED` through the distinct daemon acceptance authority |
| Memory remember/forget/search/versions | implemented | no automatic harvesting |
| Skill import/bind/revoke/explain | implemented | scripts never execute |
| Governed Memory/Skill Context consumption | implemented | exact scope/pin/digest load, durable v24 records, session-2 reuse, and forget/revoke fail-closed; public HTTP lifecycle cycles remain separate |
| Context request/view + caches | implemented | O2/O3/O4/O5/O13 bounded observation plane is task-channel read-only; empty collectors return named negative controls rather than silent zeros; O13 audit replay fails closed on stale cursor or digest break |
| Artifact CAS | implemented | GC deferred (abandoned staging only) |
| Current six-family resource projection/watch | implemented | management+task channels only; this is not the adopted MCP seventh family |
| Agent lifecycle (Pi acquisition→sidecar) | implemented | — |
| Non-Pi agents | designed | Codex fixture qualification only |
| Personal 2.0 Windows OPC product | Requires-backend + Requires-environment | Today/Projects/Knowledge (Team and Inbox are not first-level); Dual Track L1 empty chrome is on daemon `/ui/`; frozen-prototype default scenes are Phase 12 formal-plan cards (not canvas pixel-replica, not 2.1); current interaction spec remains `personal-20-opc-e2e-optimized-v9`; one-module catalog [`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md); Linux/WSL/CI/Canvas evidence does not transfer |
| Project/Charter/Goal/Plan/Routine/Task/Attempt | Requires-backend | current Task authority is reusable, but Project activation, manager envelope, Routine/missed ledger and complete hierarchy do not exist |
| Role Blueprint/Assignment/Digital Employee | Requires-backend | no complete authority/projection; employee identity must remain separate from runtime/process |
| Pi-backed Personal Assistant | Requires-backend | Pi is the hidden candidate-only target engine; current Pi Shell/Linux qualification does not establish the OPC Assistant |
| Preinstalled managed DSH Installed Agent | Requires-backend + Requires-environment | existing dsh Path B is not the exact Windows artifact/isolated child/sandbox/update/rollback qualification; no native DSH UI/conversation target |
| Personal Conversation archive/index/retrieval | Requires-backend | Personal-owned scoped archive and single composer are absent; ADR-0058 `conversation-projection/0.1` must not be reinterpreted |
| Knowledge/Markdown Vault/episodic retrieval | Requires-backend | no OPC Personal Home/import/OCR/index/Vault/conflict/Obsidian companion product path |
| Semantic Memory privacy/correct/forget integration | Requires-backend | current Memory admission/forget exists, but Conversation/Vault extraction/retrieval integration and privacy matrix do not |
| Routine/Trigger/Inbox/offline-missed recovery | partial (walking skeleton) + Requires-backend | v33 + management `routine.*` prove no-overlap/queue-latest and a visible missed ledger; daemon `scheduler_entries` is the only schedule authority. Dual Track L1 chrome is on `main` (`P11-T13`); clock/sleep/restart host E2E and Inbox L1 remain absent (HITL is T09 canvas). |
| Windows host/tray/background (hidden) | partial (walking skeleton) + Requires-environment | v34 + management `host.*` prove Personal Home `app/`/`data/`, close honesty, missed segments, and ordered seven-step recovery. Not chrome. Not a second credential plane. Native install/tray/ACL/sleep/SecretStore E2E is `not-run` until `DEV-WINDOWS-NATIVE-OPC-01`. |
| Provider global→Project→employee→Task binding and hard budgets | Requires-backend | current fixed Agent binding and advisory budgets remain partial; DSH/Pi raw-secret-free daemon proxy is required |
| X/Twitter connector (hidden) | partial (walking skeleton) + Requires-environment | v35 + management `connector/x.*` prove SecretStore-only bind, original digest-bound preview, HITL confirm, persist-before-dispatch, and honest unknown readback. Not P0 hero. Not a business result. Evasion forbidden. Live X API / CAPTCHA / platform qualification is `not-run`. |
| Existing MCP Tool transport + bounded dynamic-Tool MVP | implemented for its accepted P5-T03/P5-T04 scope | interop produces Tool candidates; it has no Personal 2.0 server/package/connection/binding/health/quarantine family lifecycle |
| Personal MCP seventh family | deferred / Requires-backend | ADR-0057/0058 retained advanced private target; not OPC P0, no current family API, and DSH native MCP/base tools remain disabled |
| Windows OPC fixed-denominator acceptance | Requires-environment / not-run | unparked N=15 at one qualified Windows revision; not the Phase 12 prototype-completeness mutex; required CI/Canvas does not execute it; signing/B01-W/release remain separate |
| Management fallback verbs | implemented | R0/R2/R3 approval flows partial |
| Backup/restore command | partial | secrets/bearer/provider-config/authority SQLite excluded; Memory/Skill as digest-bound sidecar; public `admin-cli` covers Pi install→recover |
| Current Web UI / Console | partial | same-origin daemon-served `/ui/` Dual Track L1 (Today/Projects/Knowledge + Settings + rail) exists at `clients/pc/web/`; empty home is only-create (`#/projects/new` five-step wizard) on `main` (`P12-T02`); Project four submenus (`#/projects/:id` plus members/runs/outputs) are on `main` (`P12-T03`); select-then-configure + add member is on `main` (`P12-T04`); Today decision packets (`today-incomplete` continue-create vs live pending-previews) are on `main` (`P12-T05`); HITL canvas Confirm (`P12-T06`) is on `main`; Knowledge ingest (`P12-T07`) is in progress on daemon `/ui/`; connections/rail-write remain later Phase 12 cards, not complete IA; Linux 1.0 six-family remains Advanced/secondary; not Windows OPC |
| Current Windows product | unavailable | installer/credential fragments and ordinary CI are not Windows OPC host/DSH/UI support; qualified native environment and B01-W do not exist |
| Personal 2.1 native mobile/E2E relay remote | deferred | host-online only; device-bound keys/revocation/short sessions/preview/audit/no secret downlink remain future controls |
| Performance campaign tooling | implemented | results are non-claim records in the plan |
| UJ1–UJ6 capability-truth register | implemented | frozen public-caller/oracle/cleanup/evidence rows; Web UI/Multi-Agent scope-excluded and cannot block the required arm; linux-002 named oracles are product evidence, not EVAL/Gate |

Per-row detail and sources: the user and developer pages listed in
[`_meta/source-map.json`](../../_meta/source-map.json).
