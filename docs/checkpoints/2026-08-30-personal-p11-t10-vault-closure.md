# P11-T10 Markdown Vault closure

- Task: `P11-T10` / slice `P11-T10/D01` (full Phase 11 T10 acceptance)
- Change class: `implementation-only` (v32 Vault documents + rebuildable index + conflict; no `core/specs`, no Lane-CTR, no `/ui/` IA, no Obsidian)
- Branch: `personal/P11-T10-vault`
- Linux native focused HEAD: `2cfb7ae53fdd8939ed8d3cd1f8991698893affbc`
- Required-CI / PR head: `64734a135d094dde12237c9ecd2b3586935f7766`
- Merge revision: `main@e51b616e41c7481f209d081525623f7c556fa5d1`
- Pull request: [#288](https://github.com/agentkernel/cognitive-os/pull/288) (merged 2026-08-30)
- Lease: `lease/personal/P11-T10/vault` (closed into PARALLEL-LANES §3.1 by this ledger)
- Required CI on `64734a13`: **SUCCESS** — run [33321852981](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981): `resolve validation route` **SUCCESS** [99285136143](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99285136143), `verify (ubuntu-latest)` **SUCCESS** [99285144577](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99285144577) 3m53s, `verify (windows-latest)` **SUCCESS** [99285144546](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99285144546) 13m51s, `required-ci` **SUCCESS** [99286886862](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99286886862). Incremental log: [report](2026-08-30-personal-p11-t10-vault-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers full Phase 11 T10 close gate. Host filesystem / index E2E, B01, Windows OPC vault E2E, and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux store **9/9**, named HTTP conflict, and P8-T13 sqlite-scan at `2cfb7ae5` are **pass**. Workspace `required-ci` on `64734a13` is **SUCCESS**. Files are not Project authority. Memory FTS is not the Vault index.

| Acceptance item | Evidence |
|---|---|
| Knowledge / Markdown Vault import with rights and provenance | v32 `p11_vault_document` (`is_authority CHECK = 0`). Management `POST /management/project/v1/vault.import`. Linux store **9/9** at `2cfb7ae5` |
| Rebuildable index (not Memory FTS) | `p11_vault_index_entry`; HTTP `vault.index.rebuild` / `vault.index`. Cross-project read **403** (`p11_t10_cross_project_vault_read_is_rejected`) |
| Conflict record required; no silent last-write-wins | `p11_vault_conflict`; store `p11_t10_last_write_wins_without_conflict_record_is_rejected`; HTTP `conflict_policy=last-write-wins` **422** |
| File cannot confirm/apply Project authority | store `p11_t10_file_cannot_confirm_or_apply_project_authority`; HTTP `vault.apply-authority` **422** |
| Secret-shape rejected on import; authority SQLite omits secret-shape bytes | store N1 + N9; HTTP import **422**. N9 bind at `2cfb7ae5` |
| Path traversal rejected; task channel cannot import | store N7; HTTP `POST /task/project/v1/vault.import` **403** (N8) |
| Memory admission cannot swallow Vault files; conversation/CAS are not Vault | store N5 / N6 |
| Linux store T10 focused negatives | **pass** **9/9** at `2cfb7ae5` (`DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t10-2cfb7ae5`) |
| Linux HTTP import / index / conflict + task channel forbidden | **pass** 1/1 at `2cfb7ae5` (`vault_import_index_conflict_and_task_channel_is_forbidden`) |
| P8-T13 authority sqlite scan (Ubuntu `sk-` false-positive at `62fac6e3`) | **pass** 1/1 at `2cfb7ae5` (`create_without_key_preserves_manual_catalog_and_blocks_delete_with_binding`) |
| Host filesystem / index E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01`; card allows until qualified) |
| B01 campaign guest | **not_available** / **not-run** (evaluation routing OFF) |
| Windows OPC vault E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01`) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `64734a13` | **SUCCESS** run [33321852981](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store N1–N9 (secret-shape, file-as-authority, LWW, overreach, Memory swallow, conversation/CAS, traversal, sqlite omit) | **pass** 9/9 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t10-2cfb7ae5` | `2cfb7ae53fdd8939ed8d3cd1f8991698893affbc` |
| kernel-server `vault_import_index_conflict_and_task_channel_is_forbidden` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `2cfb7ae53fdd8939ed8d3cd1f8991698893affbc` |
| `p8_t13_provider_control_plane` sqlite-scan retest | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `2cfb7ae53fdd8939ed8d3cd1f8991698893affbc` |
| Host filesystem / index E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | `2cfb7ae5` |
| B01 guest | **not-run** | evaluation routing OFF | `2cfb7ae5` |
| Windows OPC vault E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | `2cfb7ae5` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `64734a13` |
| `verify (ubuntu-latest)` on `64734a13` | **SUCCESS** [99285144577](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99285144577) | `CI-UBUNTU-01` | `64734a135d094dde12237c9ecd2b3586935f7766` |
| `verify (windows-latest)` on `64734a13` | **SUCCESS** [99285144546](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99285144546) | `CI-WINDOWS-MSVC-01` | `64734a135d094dde12237c9ecd2b3586935f7766` |
| `required-ci` on PR head `64734a13` | **SUCCESS** [99286886862](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981/job/99286886862) | GitHub Actions run [33321852981](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981) | `64734a135d094dde12237c9ecd2b3586935f7766` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T13 `/ui/` IA. Not Obsidian. Not Memory FTS as Vault index. Not Artifact CAS as Vault files. Not T05 conversation as Vault. Not T11 Memory admission product. Not T14/T15. Evaluation routing OFF. Live `/ui/` remains Linux 1.0 six-family.

## Deterministic closure

1. Linux native focused **pass** at `2cfb7ae5` (store 9/9, HTTP conflict 1/1, P8-T13 sqlite-scan 1/1);
2. required CI [33321852981](https://github.com/agentkernel/cognitive-os/actions/runs/33321852981) **SUCCESS** on `64734a13`;
3. PR [#288](https://github.com/agentkernel/cognitive-os/pull/288) merged as `main@e51b616e` on 2026-08-30;
4. lease `lease/personal/P11-T10/vault` moved to §3.1;
5. remote `personal/P11-T10-vault` already gone; local task branch deleted when safe; local `main` fast-forwarded to the merge plus this status/closure commit.

Unique next: parent claims `P11-T11/D01`. This file does **not** claim `lease/personal/P11-T11/memory`. Do not auto-claim `P11-T02`/`T08`/`T13`. Do not unpark `P11-T14`/`T15`.
