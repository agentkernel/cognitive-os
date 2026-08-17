# P2-T30 Public-admit scheduler lease acquisition — running validation report

- Task: `P2-T30`
- Branch: `personal/P2-T30-scheduler-lease-acquisition`
- Lease: `lease/personal/P2-T30/scheduler-lease-acquisition`
- Change class: `implementation-only`
- Claim ceiling: hypothesis/non-claim. No Gate, release, Profile, B01, EVAL, or
  Agent-benefit promotion.

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt --check`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy（`RUST-LINK-DEV-WIN-GNU-01`）。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu supporting CI
  (`verify (ubuntu-latest)` workspace test + Clippy `-D warnings` + handbook)。
  Windows 是 `not-run by owner-directed Linux-only route`。
- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2` / `hal9000`）是 D03 所需的 exact
  pushed-revision 验收环境。
- `B01-Desktop-Linux-002` 属于后续 EVAL freeze guest；本任务不使用。

## Root cause (EVAL-005 skip class)

Public `POST /task/admit` published TaskContract + `START` Loop + runnable
scheduler row, but omitted Context authorization facts and the tenant
`personal` revocation epoch. The first scheduler tick then skipped with
`ContextAuthorizationUnavailable` (`scheduler_row_skip_before_lease`):
Tasks stayed `DRAFT`, `lease_acquired` 0/0, no Pi child. Fixture production
tests hid this by injecting facts and leaving Loop at `DECIDE`.

## D01 — persist Context authorization at public admit

### D01-IMPL-01 — daemon-owned facts + Loop START→DECIDE

- Instrument: `persist_owner_local_context_authorization` on public admit;
  `LoopDriver::advance_start_to_decide_after_context_view`; candidate admission
  walks `START -> DECIDE` before Pi when Loop is still `START`.
- Outcome: authored. No client capability channel. Admission still does not
  consume WIA or acquire a lease. Conflicting existing facts fail closed 409.

### D01-VAL-01 — local static gates (`DEV-WIN-GNU-01`)

- `cargo fmt --all -- --check`: **pass**.
- `node tools/src/check-handbook.mjs`: **pass** (54 documents × 2 locales).
- `node tools/src/generate-handbook.mjs --check`: **pass** (18 pages
  byte-identical).
- `git diff --check`: **pass**.
- `pnpm run check:consistency`: **pass**.
- `node tools/src/docs-sync-gate.mjs --staged`: **pass** (daemon-http,
  scheduler-execution, kernel-authority mappings; no `DOCS_IMPACT_NONE`).
- Ubuntu supporting CI run `32019929603` `verify (ubuntu-latest)`: **fail** —
  Clippy `-D warnings` `too_many_arguments` on
  `LoopDriver::advance_start_to_decide_after_context_view` (8/7). Fix: same
  `#[allow(clippy::too_many_arguments)]` used by sibling LoopDriver methods
  at `85d6a2bb89136c6907f3c260740af9032ddc88b7`.
- Ubuntu supporting CI run `32020479890` at `85d6a2bb`: `verify (ubuntu-latest)`
  **pass**; `verify (windows-latest)` **pass**; `required-ci` **pass**. Windows
  GitHub MSVC is the workflow job; extra native GNU remains `not-run by
  owner-directed Linux-only route`.




### D01-VAL-02 — failure-first public-admit test

- Instrument: `public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`
  through the real `TaskApi` record → interpret → preview → admit chain.
- Expected: tick 1 spawns Pi (`calls >= 1`), Task stays `DRAFT`, `lease_owner`
  None, `attempt_count` 0; tick 2 acquires a lease, Task leaves `DRAFT`, Effect
  `RECONCILED`.
- Outcome: **pass** on exact `85d6a2bb89136c6907f3c260740af9032ddc88b7`
  `DEV-LINUX-NATIVE-01`:
  `personal::scheduler_authority::tests::public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`
  **ok** (1 passed / 336 filtered on the focused filter; full kernel-server bin
  337/337). Tick 1 spawns Pi and leaves DRAFT; tick 2 acquires the lease.

## D03 — exact-revision `DEV-LINUX-NATIVE-01` (`85d6a2bb`)

- Host: `wuz@192.168.1.2` (`hal9000`), rustc 1.97.1.
- Worktree: `/home/wuz/agent-kernel-worktrees/p2-t30-590e41bb` at exact
  `85d6a2bb89136c6907f3c260740af9032ddc88b7`.
- Focused `public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`:
  **pass**.
- `cargo test --locked -p kernel-server --bins`: **pass** 337/337.
- `cargo test --workspace --locked`: **pass**, 0 failed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: **pass**.
- `cargo fmt --all -- --check`: **pass**.
- Ubuntu/Windows/`required-ci` run `32020479890`: **pass**.
- `B01-Desktop-Linux-002` untouched.

