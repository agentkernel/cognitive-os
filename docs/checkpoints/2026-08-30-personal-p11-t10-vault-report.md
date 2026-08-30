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

Wait required CI green on CHECK-fix HEAD `04e828bd82ef7b0b90f6408788f7bb6a9fd768f8`, then parent close. Do not merge from this worker. Do not claim T11. Ubuntu `verify` at that SHA already **fail** (job 99282573080; failed-log body pending until the Windows job finishes). Linux focused retest on `04e828bd` is a parallel worker — pending, not pass. Host filesystem E2E / B01 / Windows OPC remain `not-run`. `DEV-WIN-GNU-01` cargo remains `not-run`. Evaluation routing OFF.

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
| N9 | authority SQLite omits secret-shape bytes after import | store `p11_t10_authority_sqlite_omits_secret_shape_bytes_after_import` |

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
| 2026-08-30 | `verify (ubuntu-latest)` Test Rust workspace | **fail** | `CI-UBUNTU-01` | `62fac6e364d74462c4be88f92f900a14039c742d` | [job 99280877208](https://github.com/agentkernel/cognitive-os/actions/runs/33320263553/job/99280877208) run [33320263553](https://github.com/agentkernel/cognitive-os/actions/runs/33320263553). Not clippy/fmt/handbook (those steps skipped). First assertion: `p8_t13_provider_control_plane::create_without_key_preserves_manual_catalog_and_blocks_delete_with_binding` panicked at line 381: `authority sqlite must not contain API key material`. `layout_migrations` was already `1..=32` (not this fail). T10 N1–N5 negatives unchanged. |
| 2026-08-30 | v32 index CHECK omits persist-forbidden inject-order labels | recorded | `DEV-WIN-GNU-01` | this commit | `sqlite_master` stored the Task-contract layer token, whose `sk-` substring is a false-positive for the P8-T13 raw-sqlite scan. CHECK now persists only `sourced-excerpt` / `summary` / `older-narrative`. `CONTEXT_INJECT_ORDER` and HTTP inject_order JSON still include that label. Added store guard `p11_t10_authority_sqlite_omits_secret_shape_bytes_after_import`. Secret-shape / file-as-authority / LWW / cross-project / Memory swallow negatives unchanged. |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t10_vault` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo test -p kernel-server --test p8_t13_provider_control_plane` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo fmt --all` | pass | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `node tools/src/fill-handbook-fingerprints.mjs` | pass | local Node | this commit | `dev.store-migrations` both locales (vault.rs source fingerprint) |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | pass | local Node | this commit | 58×2 locales after fingerprint refresh |
| 2026-08-30 | `node tools/src/docs-sync-gate.mjs --staged` | pass | local Node | this commit | store source-map group; fingerprint-only `dev.store-migrations` |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t10_vault` | **pass** 8/8 | `DEV-LINUX-NATIVE-01` | `62fac6e364d74462c4be88f92f900a14039c742d` | worktree `/home/wuz/cognitiveos-personal-worktrees/p11-t10-62fac6e` (detached); STORE_TEST=0; 0.85s; FAIL_TAIL empty. 8 tests at this SHA (N9 not yet present). Host FS E2E not claimed. |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- vault_import_index_conflict` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `62fac6e364d74462c4be88f92f900a14039c742d` | HTTP_TEST=0; `personal::project_aggregate::tests::vault_import_index_conflict_and_task_channel_is_forbidden`. |
| 2026-08-30 | Host filesystem / index E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | `62fac6e364d74462c4be88f92f900a14039c742d` | Card allows `not-run` until the host route is qualified. Never pass. |
| 2026-08-30 | B01 campaign guest | **not_available** | n/a | `62fac6e364d74462c4be88f92f900a14039c742d` | No B01. Evaluation routing OFF. |
| 2026-08-30 | Windows OPC vault E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | `62fac6e364d74462c4be88f92f900a14039c742d` | Not claimed. |
| 2026-08-30 | CHECK-token fix SHA | recorded | docs-only | `04e828bd82ef7b0b90f6408788f7bb6a9fd768f8` | Sibling landed v32 CHECK persist-only `sourced-excerpt` / `summary` / `older-narrative`. Inject-order label stays Rust/HTTP only. This worker did not rewrite schema. |
| 2026-08-30 | Linux focused vault retest on CHECK-fix HEAD | **pending** | `DEV-LINUX-NATIVE-01` | `04e828bd82ef7b0b90f6408788f7bb6a9fd768f8` | Parallel worker. Do not invent pass. |
| 2026-08-30 | Ubuntu `verify (ubuntu-latest)` Test Rust workspace | **fail** | `CI-UBUNTU-01` | `04e828bd82ef7b0b90f6408788f7bb6a9fd768f8` | run [33320897378](https://github.com/agentkernel/cognitive-os/actions/runs/33320897378) job [99282573080](https://github.com/agentkernel/cognitive-os/actions/runs/33320897378/job/99282573080). Test Rust workspace failed in ~15s. Failed-log body not yet available (`gh run view --log-failed` refused while Windows job still in progress). Do not invent assertion. |
| 2026-08-30 | Windows `verify (windows-latest)` | **pending** | `CI-WINDOWS-MSVC-01` | `04e828bd82ef7b0b90f6408788f7bb6a9fd768f8` | job [99282573045](https://github.com/agentkernel/cognitive-os/actions/runs/33320897378/job/99282573045); Test Rust workspace in progress. `required-ci` not yet reported. |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit. Not T13 `/ui/` IA. Not Obsidian. Not Memory FTS as Vault index. Not Artifact CAS as Vault files. Not T05 conversation as Vault. Not T11 Memory admission product. Not T14/T15. Evaluation routing OFF.
