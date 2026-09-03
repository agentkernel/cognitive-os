# P13-T07 Knowledge + Memory authority — running report

- Task: `P13-T07` / slice `P13-T07/D01`
- Change class: `implementation-only` (labeled Vault read + Memory auto-admit / promote on existing Memory tables; Knowledge `/ui/` caller; no new numbered migration; T06 owns `personal_db.rs` v39)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T07/knowledge-memory`
- Branch: `personal/P13-T07-knowledge-memory` (worktree `d:\agent-kernel-wt-P13-T07`; `d:\agent-kernel` not used)
- Base: folding `origin/main@2217722d` (T10 merged PR #318; T06 merged at `main@23355afb`)
- Claim ceiling: `hypothesis` (A7: local MSVC / Dual Track is not Gate / release / Profile / Windows qualification)
- Evaluation routing: **OFF**
- Host FS/privacy E2E: `not-run` until `P13-T13`

## Unique next action

Required CI green on the folded HEAD of Draft PR [#319](https://github.com/agentkernel/cognitive-os/pull/319) after `origin/main@2217722d` (T10 merged), then `P13-T07/D01` formal acceptance and ready/merge. Slice status is `in-progress`. Auto-admit UI stays honest empty / Requires-backend (0 fake Admit buttons). Do not merge while a sibling ready/merge is in flight.

## Failure-first (D01)

| ID | Negative | Surface |
|---|---|---|
| N1 | Vault file cannot become Project authority (`apply_as_project_authority` Invalid) | store |
| N2 | Cross-project labeled read is Forbidden | store + HTTP 403 |
| N3 | Agent / task-channel cannot self-admit chat into Memory | store Forbidden + HTTP 403 aliases |
| N4 | Secret-shaped chat is not admitted (archive already rejects `sk-`) | store |
| N5 | Tombstoned Memory cannot be promoted or resurrected | store |
| N6 | Unconfirmed promote does not copy Memory; confirm is digest-bound | store + HTTP |
| N7 | Last-write-wins without a conflict record is rejected; import failure keeps original fields | existing Vault + Knowledge ingest |
| N8 | No fake Admit button; auto-admit UI stays honest empty / Requires-backend (T06 group-chat is on `main`; this surface still does not list turns as admit candidates) | Dual Track UI |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-03 | `cargo test -p cognitive-store --test p13_t07_knowledge_memory --locked` | **pass** 8/8 | local MSVC override `1.97.1-x86_64-pc-windows-msvc`; `CARGO_PROFILE_DEV_DEBUG=0` | worktree, uncommitted | development evidence only |
| 2026-09-03 | `cargo test -p kernel-server p13_t07 --locked` | **pass** 2/2 | same local MSVC | worktree, uncommitted | labeled/documents/task aliases + promote preview/confirm + auto-admit 404 then 201 |
| 2026-09-03 | Dual Track vitest `knowledgeMemory` + `vault` + `knowledgeIngest` | **pass** 22/22 | `DEV-WIN-GNU-01` / Node | worktree, uncommitted | no new KNOWN_ROUTES; no Admit button |
| 2026-09-03 | Host FS/privacy E2E | **not-run** | `P13-T13` | — | recorded, not inferred pass |
| 2026-09-03 | required CI run [33746065086](https://github.com/agentkernel/cognitive-os/actions/runs/33746065086) at `e1e276df` | **fail** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | PR [#319](https://github.com/agentkernel/cognitive-os/pull/319) | `check-consistency`: Phase 13 counts 13/5/1/0/7 vs task rows 13/5/2/0/6; lease owned `PARALLEL-LANES.md` |
| 2026-09-03 | governance fix (lease ledger + plan/PROGRESS counts) | **pending** | worktree | after push | unique next: required CI green |
| 2026-09-03 | required CI run [33748054839](https://github.com/agentkernel/cognitive-os/actions/runs/33748054839) at `5e2c5efc` | **fail** | `CI-UBUNTU-01` | PR #319 | `check-consistency`: `P13-T07/D01` status `in-progress (owner-paused)` is invalid; lease/slice mismatch |
| 2026-09-03 | fold `origin/main@ef9baab2` (T06 done) | **pass** (local gates) | worktree | `ad633b64` | `check:consistency` OK; handbook OK; generate-handbook --check OK; 0 fake Admit |
| 2026-09-03 | required CI run [33751771572](https://github.com/agentkernel/cognitive-os/actions/runs/33751771572) at `ad633b64` | **fail** | `CI-UBUNTU-01` | PR #319 | clippy `-D warnings`: `too_many_arguments` on `admit_text` / `prepare_admission` |
| 2026-09-03 | clippy argument grouping (`AdmissionDraft`) | **pass** | local MSVC override | worktree | `cargo clippy -p cognitive-store --all-targets --locked -- -D warnings`; focused store 8/8 |
| 2026-09-03 | required CI run [33752950398](https://github.com/agentkernel/cognitive-os/actions/runs/33752950398) at `070fd243` | **pass** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | PR #319 | resolve 4s, ubuntu 4m9s, windows 20m13s, required-ci 4s. HEAD became DIRTY after T10 merged at `main@2217722d`. |
| 2026-09-03 | fold `origin/main@2217722d` (T10 done) | **pass** (local gates) | worktree | folding | `check:consistency` OK; `check-handbook` OK; `generate-handbook --check` OK; store 8/8; kernel-server 2/2; Dual Track 22/22; 0 fake Admit |
