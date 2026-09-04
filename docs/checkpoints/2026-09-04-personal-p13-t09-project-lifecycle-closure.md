# P13-T09 Project lifecycle and local recovery — closure

- Task: `P13-T09` **done** / slice `P13-T09/D01` **done** (single Delivery Slice)
- Change class: `implementation-only` (Project copy / archive / delete / restore-point / export on existing authority tables; **no new applied migration** — `project_lifecycle_migration_entry()` only reserves v41 and is not registered; T11 holds v40 / `personal_db.rs`; no `core/specs`, no contract or negative weakened)
- Lease: `lease/personal/P13-T09/project-lifecycle` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P13-T09-project-lifecycle` (worktree `D:\agent-kernel-wt-P13-T09`) → PR [#321](https://github.com/agentkernel/cognitive-os/pull/321)
- Implementation revision: `0e34d3cb` … `3d4001b0` (rebased onto `main@22718d74`, T08 merged); fold HEAD `0898af16` (merge of `origin/main@763b7909`, T07 merged PR #319; conflicts only in plan docs, bilingual `store-and-migrations`, regenerated `ref.http-api`)
- Required CI: [33761616553](https://github.com/agentkernel/cognitive-os/actions/runs/33761616553) **SUCCESS** at `3d4001b0`; fold HEAD `0898af16` [33843008228](https://github.com/agentkernel/cognitive-os/actions/runs/33843008228) **SUCCESS**
- `DEV-LINUX-NATIVE-01` exact revision: pre-fold `8c681757` (store 2/2 + 1/1, kernel-server 4/4, clippy clean); fold HEAD `0898af16` **pass** (store 2/2 + 1/1, kernel-server 2/2 + 1/1 + 1/1, clippy clean; §2)
- Running report: [P13-T09 report](2026-09-03-personal-p13-t09-project-lifecycle-report.md)
- Claim ceiling: `hypothesis` (A7: local MSVC / Dual Track / ordinary CI / Linux native close "the implementation exists" only). Windows FS E2E (Personal Home `data/` on a real Windows host, restore-point files, export on disk) stays **not-run** until `P13-T13`. Not Gate / release / Profile / B01 / Windows support.
- Evaluation routing: **OFF**

## 1. Acceptance mapping (formal plan P13-T09 card + `P13-T09/D01`)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| copy-project produces an **inactive 副本** (no runtime, grants, or seated Members copied) | store `project_lifecycle.rs` `copy_project` → `p11_project` row `inactive`, 0 grants / seats / armed Routines; automatic `data/projects/<id>/`; HTTP `POST /management/project/v1/copy`; Projects list 「Copy as 副本」 | `inherit_grants` / `inherit_seats` / `inherit_runtime` → store refuse + HTTP 422; in-flight source still refused (`p11_t03_copy_excludes_secrets_and_inflight`) | store `p13_t09_copy_refuses_inherit_flags_but_copies_seated_source_as_inactive`; kernel-server `inherit_export_secret_and_backup_claims_are_refused` + `management_copy_archive_delete_restore_export_round_trip`; Dual Track `projectLifecycle` |
| Archive stops Routine / Trigger first | `archive_project` pauses every `armed` P13-T05 arming before the state change; HTTP `archive` | `skip_stop_triggers` → 422 | store `p13_t09_archive_delete_export_restore_negatives_and_round_trip`; HTTP round trip |
| Delete needs an impact preview (affected Routines / Members / outputs) + second confirmation | `delete.preview` returns the impact list and a digest; `delete.confirm` requires digest + `second_confirm`; logical tombstone (`state='deletion-preview'`, `current_plan_revision_id='tombstone'`), the `p11_project` row is never dropped | preview refuses a non-archived Project or live triggers; `physical_delete` → 422; confirm without second confirmation → refuse | store negatives; HTTP round trip (preview → confirm → row remains) |
| Same-disk auto-versioned local restore points, explicitly **not** disaster backup | `restore-point` → `local-restore-vN`, `is_backup=0`; `GET lifecycle` lists them with `is_disaster_backup: false` | `claimed_as_backup` / `is_disaster_backup` → 422 | store + HTTP negatives; Dual Track shows the non-backup label |
| Manual export excludes secrets by default and is not authority | `export` default-excludes secrets; `is_authority: false` | `include_secrets` → 422 | store + HTTP + Dual Track |
| Every Project gets an automatic `data/` directory under Personal Home | `data/projects/<id>/` created on copy / recorded on lifecycle | — (positive path; host-disk cell `not-run` until T13) | store round trip; `GET lifecycle` `data_dir` |
| Task-channel aliases fail closed | task aliases of all seven routes → 403 (forwarded from closed T06 `project_chat.rs`; T08 `server.rs` untouched) | 403 | kernel-server `task_channel_aliases_are_forbidden` |
| Product origin = daemon `/ui/`; chat has no Approve; Settings chrome not taken | Projects list + Project detail `ProjectLifecyclePanel`; no Approve / FAKE_ACTION label | Dual Track forbids Approve labels | web vitest `projectLifecycle` + `projectSubmenus` 13/13 |

Formal-plan 关闭门, sentence by sentence: copy-project = inactive 副本 — **true**; 归档先停 Routine/Trigger — **true**; 删除影响预览 + 二次确认 — **true**; local restore points 明示非灾备 — **true**; 导出默认排除 secret — **true**; 每 Project 自动 `data/` — **true**.

Drift negatives from the card, all refused / never produced: 副本自动激活 / 继承 grant / 就位 (refused, copy is `inactive`); 删除不停触发 (preview refuses live triggers; archive pauses armings first); restore-as-backup 声称 (422); 导出含 secret (422); 跨项目共享 Member (no seat copied; T06/T07 cross-project 403 unchanged).

## 2. Validation summary

| Environment | Result |
|---|---|
| Local MSVC override (`rustc` host `x86_64-pc-windows-msvc`; `CARGO_PROFILE_DEV_DEBUG=0`) | store `p13_t09_project_lifecycle` **2/2**; `p11_t03_copy_excludes_secrets_and_inflight` **1/1**; kernel-server T09 tests **4/4**; fmt applied — development evidence only (2026-09-03, pre-fold) |
| `DEV-WIN-GNU-01` (Node) | web vitest `projectLifecycle` + `projectSubmenus` **13/13**; `check:consistency` / `check-handbook` / `generate-handbook --check` / `docs-sync-gate` **pass** at fold `0898af16` |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33761616553](https://github.com/agentkernel/cognitive-os/actions/runs/33761616553) **SUCCESS** at `3d4001b0`; fold HEAD `0898af16`: [33843008228](https://github.com/agentkernel/cognitive-os/actions/runs/33843008228) **SUCCESS** (resolve, ubuntu, windows, required-ci all green) |
| `DEV-LINUX-NATIVE-01` (`hal9000`, exact pushed revision, dirty=0) | `8c681757`: store 2/2 + 1/1, kernel-server 4/4, clippy `-D warnings` clean; fold HEAD `0898af16` (worktree `~/cognitiveos-personal-worktrees/p13-t09-0898af16`, `git status --porcelain` empty, `CARGO_TARGET_DIR` reused from `p13-t05-ecd35ab0/target`; log `p13-t09-0898af16-validate.log` START `2026-09-04T06:09:07Z` → END `06:10:58Z`): store `p13_t09_project_lifecycle` **2/2**, `p11_t03_copy_excludes_secrets_and_inflight` **1/1**, kernel-server `task_channel_aliases_are_forbidden` **2/2** (T09 + T06), `inherit_export_secret_and_backup_claims_are_refused` **1/1**, `management_copy_archive_delete_restore_export_round_trip` **1/1**, `cargo clippy -p cognitive-store -p kernel-server --all-targets --locked -- -D warnings` **pass** |
| `DEV-WINDOWS-NATIVE-OPC-01` | **not-run** (Windows FS / Personal Home `data/` / restore-point-on-disk E2E waits `P13-T13`) |
| `B01-Desktop-Linux-002` | **not-run** (no guest `/ui/` deploy for this slice) |

## 3. Non-claims

Not T07 Knowledge/Memory, not T08 Settings chrome, not T10 grants, not T11 reflection, not T12/D02 rendered review, not T13 Windows FS. No applied migration (v41 reserved only). Restore points are not backups (P7-T02 boundary unchanged). Export is not authority. No automatic promotion. No Gate / release / Profile / B01 / Windows qualification. `PERS-PR-051` can now leave `not-run` only through its own evidence; this closure adds none.

## 4. Unique next

Ready/merge PR [#321](https://github.com/agentkernel/cognitive-os/pull/321) once required CI is green on the closure HEAD. After merge: fast-forward `main`, delete the task branch, remove the worktree, record `main@<merge>` in PROGRESS / plan / PARALLEL-LANES §3.1, then continue serially with `P13-T11` (Draft PR [#320](https://github.com/agentkernel/cognitive-os/pull/320)). Do not claim T12/T13/T15 in this closure.
