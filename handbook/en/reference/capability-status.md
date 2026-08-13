---
doc_id: ref.capability-status
locale: en
kind: reference
audience: [user, developer, ai]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
  - path: apps/admin-cli/src/personal_cli/mod.rs
  - path: crates/cognitive-store/src/personal_backup.rs
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
  - path: crates/cognitive-secret/src/backend_select.rs
  - path: apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: crates/cognitive-management/src/task_application.rs
fingerprint: "sha256:2d8db6e79276263c7bb164845497531ebe8afbadd575a77e5480c7ca241c6502"
non_claims:
  - Statuses are code+contract+test judgments at the recorded baseline, not Gate/release/Profile results and not the formal plan's task states.
---

# Capability status matrix

Legend: `implemented` (real path + tests), `partial` (works with named gaps),
`designed` (contract/design only), `unavailable` (no usable path).

| Capability | Status | The gap, if any |
|---|---|---|
| Linux bundle install/upgrade/rollback/uninstall | implemented | production signing/publication pending |
| systemd user service + health-gated activation | implemented | — |
| `cognitive init` (layout, secrets, discovery, selected model) | implemented | — |
| Daemon loopback HTTP + channel auth + bounds | implemented | bearer RNG non-cryptographic |
| Provider proxy (non-streaming chat) | implemented | streaming unsupported |
| SecretStore | implemented (Linux Secret Service; Windows Credential Manager) | headless vault designed; macOS unavailable |
| Pi shell conversation via daemon | implemented | one-shot, text-only |
| Pi tool use inside shell | unavailable | all built-ins denied by policy |
| Task record/interpret/preview/admit | implemented | — |
| Task watch | implemented | process-local event source |
| Task control/query over HTTP | unavailable | service methods exist, no route |
| Autonomous scheduler loop | partial | admission atomically publishes the current-epoch runnable row, `START` Loop, and hard Budget; startup repairs missing members; one post-bind non-reentrant periodic worker reaches candidate admission and production-dispatches WorkspaceRead, but the other families and verification remain unwired |
| Governed tool execution (all six registered families) | partial | every family has an assembled executor, so the projection reports `execution_ready`; WorkspaceRead now has the periodic production caller, while the other five still lack a production request carrier and remain test-called only |
| Workspace search/write/patch executors | partial | handle-relative no-follow traversal/publication, bounded enumeration/preimages, target-locked CAS, workspace-external durable key-bound receipts and restart orphan recovery are tested on Linux/Windows; no production caller |
| Independent verification and Task acceptance | partial | production WorkspaceRead reaches a CAS-backed independent passed report, checkpoint, one-time continuation authority, and Loop `OBSERVE`; P2-T14 has fixed the narrow existing evidence/acceptance-principal decision and registered an expected-red public C1 completion proof, but production does not yet move the governed Task to `COMPLETED` |
| Memory remember/forget/search/versions | implemented | no automatic harvesting |
| Skill import/bind/revoke/explain | implemented | scripts never execute |
| Context request/view + caches | implemented | — |
| Artifact CAS | implemented | GC deferred (abandoned staging only) |
| Six-family resource projection/watch | implemented | management+task channels only |
| Agent lifecycle (Pi acquisition→sidecar) | implemented | — |
| Non-Pi agents | designed | Codex fixture qualification only |
| MCP tools | designed | post-1.0 fixture adapter |
| Management fallback verbs | implemented | R0/R2/R3 approval flows partial |
| Backup/restore command | unavailable | planning APIs only |
| Web UI / Console | unavailable | external repository, design-only |
| Windows/macOS product | unavailable | Linux x86_64 only; Windows installer templates + credential backend are authored and CI-validated but the B01-W install campaign has not run |
| Performance campaign tooling | implemented | results are non-claim records in the plan |

Per-row detail and sources: the user and developer pages listed in
[`_meta/source-map.json`](../../_meta/source-map.json).
