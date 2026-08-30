# P11-T10 Markdown Vault — running report

- Task: `P11-T10` / slice `P11-T10/D01`
- Change class: `implementation-only` (v32 Vault documents + rebuildable index + conflict; no `core/specs`, no Lane-CTR, no `/ui/` IA, no Obsidian)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T10/vault`
- Branch: `personal/P11-T10-vault`
- Worktree: `D:\agent-kernel-wt-P11-T05` (original `d:\agent-kernel` left dirty; A8 protected)
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile; host filesystem E2E is `not-run`)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Unique next action

Vault import → rights/provenance → parse/index → conflict on management HTTP. Index is rebuildable and is not Memory FTS. Files are not Project authority. Required CI / Linux store+HTTP still pending. Do not merge until task acceptance.

## Closed predecessor

`P11-T07` **done**: merged PR [#287](https://github.com/agentkernel/cognitive-os/pull/287) at `main@00889df942ba1753211b3f909e1237efe2c9fec2`. Lease `lease/personal/P11-T07/dsh` closed into PARALLEL-LANES §3.1. Required CI [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618) **SUCCESS** at `21c03171`. Hosted DSH is not a T10 rewrite. `P11-T11`/`T13`/`T14`/`T15` stay unclaimed/parked.

## Identifier

Personal-private envelope: `cognitiveos.personal.markdown-vault/0.1`.

Reused as pattern only: Memory/Skill admission fences (secret-shape, fail-closed). Artifact CAS may hold optional blobs; Vault metadata lives in authority SQLite. T05 conversation archive (`cognitiveos.personal.conversation-archive/0.1`) is not Vault.

Not Project authority. Not Memory FTS. Not Obsidian. Not T13 `/ui/` IA.

## Failure-first (this slice)

| ID | Test | Surface |
|---|---|---|
| N1 | secret-shape rejected on import | store `p11_t10_secret_shape_is_rejected_on_import`; HTTP import 422 |
| N2 | file cannot confirm/apply Project authority | store `p11_t10_file_cannot_confirm_or_apply_project_authority`; HTTP `vault.apply-authority` 422 |
| N3 | last-write-wins without a conflict record rejected | store `p11_t10_last_write_wins_without_conflict_record_is_rejected`; HTTP `conflict_policy=last-write-wins` 422 |
| N4 | retrieval overreach / cross-project vault read rejected | store `p11_t10_cross_project_vault_read_is_rejected`; HTTP index `caller_project_id` mismatch 403 |
| N5 | Memory admission cannot swallow Vault files as authority | store `p11_t10_memory_admission_cannot_swallow_vault_files` |
| N6 | conversation archive and Artifact CAS are not Vault files | store `p11_t10_conversation_and_cas_are_not_vault_files` |
| N7 | path traversal rejected | store `p11_t10_path_traversal_is_rejected` |
| N8 | task channel cannot import Vault | HTTP `POST /task/project/v1/vault.import` 403 |

## Vertical slice

Store tables `p11_vault_document` (`is_authority CHECK = 0`), rebuildable `p11_vault_index_entry`, `p11_vault_conflict`. Management HTTP `vault.import` / `vault.index.rebuild` / `vault.index` / `vault.conflicts` is the real caller. Context inject order (task-contract → fixed-decision → sourced-excerpt → summary → older-narrative) is a documented store helper; Vault fills sourced excerpts only. `conflict_policy=record` keeps both documents and an open conflict row.

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-08-30 | T07 lease close + T10 claim | recorded | docs-only | this commit | `lease/personal/P11-T07/dsh` closed after PR [#287](https://github.com/agentkernel/cognitive-os/pull/287) merge `00889df9`; `lease/personal/P11-T10/vault` active |
| 2026-08-30 | D01 store/HTTP implementation + failure-first tests written | recorded | `DEV-WIN-GNU-01` | this commit | v32 Vault; secret-shape / file-as-authority / LWW / overreach / Memory swallow / conversation-CAS / traversal / task-channel |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t10_vault` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- vault_import_index_conflict` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo build` / Clippy | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-30 | Host filesystem / index E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | this commit | Card allows `not-run` until the host route is qualified |
| 2026-08-30 | Obsidian companion / T13 IA / Memory FTS-as-Vault | **not-run** / out of scope | n/a | this commit | Do not claim |
| 2026-08-30 | `cargo fmt --all` | pass | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `node tools/src/generate-handbook.mjs` | pass | local Node | this commit | regenerated `http-api` both locales for `vault.import` / `vault.index.rebuild` / `vault.index` / `vault.conflicts` / `vault.apply-authority` |
| 2026-08-30 | `node tools/src/fill-handbook-fingerprints.mjs` | pass | local Node | this commit | store-migrations + daemon-http + mapped fingerprint-only pages |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | pass | local Node | this commit | Personal plan/leases OK including `P11-T10/D01` |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | pass | local Node | this commit | 58×2 locales; coverage/fingerprint OK |
| 2026-08-30 | `node tools/src/generate-handbook.mjs --check` | pass | local Node | this commit | 18 pages byte-identical |
| 2026-08-30 | `node tools/src/docs-sync-gate.mjs --staged` | pass | local Node | this commit | daemon-http + store source-map groups |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit. Not T13 `/ui/` IA. Not Obsidian. Not Memory FTS as Vault index. Not Artifact CAS as Vault files. Not T05 conversation as Vault. Not T11 Memory admission product. Not T14/T15. Evaluation routing OFF.
