# P13-T09 Project lifecycle running report

- Task: `P13-T09` (`P13-T09/D01`)
- Lease: `lease/personal/P13-T09/project-lifecycle`
- Branch: `personal/P13-T09-project-lifecycle`
- Worktree: `D:\agent-kernel-wt-P13-T09`
- Base: `main@ef9baab2`
- Claim ceiling: `hypothesis`
- Handbook this slice: T09-only annotations + generated `http-api` + bilingual `store-and-migrations` (reserved v41, not applied). `source-map.json` is not edited (T08). Sibling T07/T08/T10 may also touch `http-routes.json` — fold at merge.

## Campaign / claim

- Owner-directed campaign `PERSONAL-PERF-EVAL-015` is **closed**. Evaluation routing OFF.
- Claim allowed: T06 lease is closed (PR #316); T09 `implementation_requires` (`P11-T03`, `P11-T08`, `P11-T02`, `P7-T02`) are done. T06 is not a mutex.
- Path overlap vs active sibling leases:
  - T07: vault / knowledge / memory / KnowledgePage / handbook pages — missed.
  - T08: Settings / `mod.rs` / `server.rs` / `normalize.ts` / `source-map.json` / handbook — missed.
  - T10: `employee.rs` / store `lib.rs` / kernel-server `project_aggregate.rs` / MemberConfig / `plan.md` — missed.
  - T11: `personal_db.rs` / `reflection.rs` / v40 — missed. T09 exports reserved **v41** `project_lifecycle_migration_entry()` but does not register or apply it this slice. Runtime uses existing `p11_project` / `p13_routine_arming` / `p11_grant` / `p11_employee` / `p11_windows_host_*`.
  - DOC-REFRAME: `personal/docs/product/*` + `clients/docs/design/opc-2.0/` — missed.
- HTTP is forwarded from closed T06 `project_chat.rs` so T08 `server.rs` is untouched.

## D01 behaviour

- `POST /management/project/v1/copy` — inactive 副本; `inherit_grants` / `inherit_seats` / `inherit_runtime` 422; copy has 0 grants / seated Members / armed Routines; automatic `data/projects/<id>/`.
- `POST /management/project/v1/archive` — pause armed Routines first; `skip_stop_triggers` 422.
- `POST /management/project/v1/delete.preview` — impact list; refuses live triggers / non-archived Project.
- `POST /management/project/v1/delete.confirm` — digest + `second_confirm`; logical only; `physical_delete` 422; `p11_project` row remains.
- `POST /management/project/v1/restore-point` — `local-restore-vN`; `claimed_as_backup` / `is_disaster_backup` 422.
- `POST /management/project/v1/export` — default exclude secrets; `include_secrets` 422; `is_authority: false`.
- `GET /management/project/v1/lifecycle` — events, data dir, restore points with `is_disaster_backup: false`.
- Task-channel aliases 403.
- UI: Copy as 副本 on Projects list; archive / delete preview / logical delete / restore / export on Project detail. Chat has no Approve. Settings chrome not taken.

## Tests

Local MSVC override (`rustc` host `x86_64-pc-windows-msvc`; `CARGO_PROFILE_DEV_DEBUG=0`). Development evidence only — not supported CI / Gate / Windows qualification.

| Unit | Result | Notes |
|---|---|---|
| `cargo test -p cognitive-store --test p13_t09_project_lifecycle` | **pass 2/2** | inherit flags refuse; seated source copies as `inactive` with `data/`; archive `skip_stop_triggers` refuse; delete while live refuse; second-confirm / physical-delete refuse; logical tombstone keeps the row; export `include_secrets` refuse; restore-as-backup refuse; secret-free export + same-disk restore point |
| `cargo test -p cognitive-store --test p11_t03_project_aggregate p11_t03_copy_excludes_secrets_and_inflight` | **pass 1/1** | inflight copy still refused; clean copy still `inactive` |
| `cargo test -p kernel-server --bin kernel-server` `task_channel_aliases_are_forbidden` | **pass 2/2** | T09 task aliases 403 + existing T06 chat aliases 403 |
| `inherit_export_secret_and_backup_claims_are_refused` | **pass 1/1** | inherit grants 422; `include_secrets` 422; `claimed_as_backup` 422; live delete.preview 422 |
| `management_copy_archive_delete_restore_export_round_trip` | **pass 1/1** | live request-line trim; copy inactive; archive; preview+confirm; restore; export `is_authority:false` |
| `pnpm test` focused web (`projectLifecycle` + `projectSubmenus`) | **pass 13/13** | 2 UI + 2 projection + 9 submenu regression; no Approve / FAKE_ACTION labels |
| `cargo fmt --all` | **pass** | applied in this worktree |
| `cargo clippy --workspace` | **not-run** | reserved for CI / exact-rev Linux |
| required CI `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | **not-run** | needs this checkpoint push |
| `DEV-LINUX-NATIVE-01` | **not-run** | after push, exact revision |
| Windows FS E2E | **not-run** | until P13-T13 |

## Next

- Coordinator expanded the T09 lease for additive T09 handbook files. `check-handbook` OK (58×2); `generate-handbook --check` OK.
- Checkpoint commit + one Draft PR; then required CI + exact-rev `DEV-LINUX-NATIVE-01`.
- Windows FS E2E stays `not-run` until P13-T13.
