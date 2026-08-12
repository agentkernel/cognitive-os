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
fingerprint: "sha256:ec5c301e205f58771e15120f1e27f05810ee2f761de19c98c21722697be6fb9c"
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
| Autonomous scheduler loop | partial | one startup tick; no bootstrap row from admission |
| Governed tool execution (workspace read / process check) | partial | executors test-called only; projection reports other families as `registered_only` |
| Workspace write/patch executors | designed | validators exist; no executor |
| Independent verification loop | partial | verifier seam test-called only |
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
