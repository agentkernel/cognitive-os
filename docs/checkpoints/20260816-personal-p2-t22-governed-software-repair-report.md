# P2-T22 Governed software-repair journey — running validation report

- Task: `P2-T22`
- Branch: `personal/P2-T22-governed-software-repair`
- Lease: `lease/personal/P2-T22/governed-software-repair`
- Base: `origin/main` after P2-T21 merge `5d1f5c2643d82807eb96f2c615d460dbdca749c3` / PR #220
- Change class: `implementation-only` (catalog freeze, sequential journey
  wiring, and failure-first tests; no public contract). Mapped handbook
  pages updated bilingually for the RegisteredCheck-terminated Loop return
  to `DECIDE`; fingerprints refreshed in the same change set.
- Claim ceiling: implementation evidence only; hypothesis/non-claim. No Gate,
  release, Profile, B01, EVAL, or Agent-benefit promotion.

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录；已发布
结果只通过追加的 superseding entry 更正。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt --check`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy（`RUST-LINK-DEV-WIN-GNU-01`
  已登记 exit 121 linker failure）。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu required CI（`verify (ubuntu-latest)`
  workspace test + Clippy + handbook）。Windows 是
  `not-run by owner-directed Linux-only route`。
- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2` / `hal9000`）：exact pushed-revision
  worktree；只做 native build/test/clippy/fmt 验证；不触碰 `B01-Desktop-Linux-002`
  guest / EVAL-004 campaign roots。
- `B01-Desktop-Linux-002` guest 属于 owner-directed evaluation campaign，与本 task
  验证无关，本任务不使用。

## D01 — freeze TypeScript/Rust corpora + failure-first tests

### D01-DOC-01 — lease, plan, and BR-02 registration

- Instrument: `docs/plan/PARALLEL-LANES.md` active table,
  `docs/plan/PROGRESS.md` Current snapshot,
  `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`,
  `docs/evaluation/personal-performance-benchmark-readiness-closure-plan.md`
- Outcome: `pass` (authored). Lease
  `lease/personal/P2-T22/governed-software-repair` claimed with
  `P2-T22/D01`. P2-T21 flipped to `done` at merge
  `5d1f5c2643d82807eb96f2c615d460dbdca749c3` / PR #220. Layer 1
  `88 | 74 | 1 | 1 | 12 | 14`. BR-01 `done`, BR-02 `in-progress`.
- Disposition: opens D01; does not execute Rust tests.

### D01-IMPL-01 — catalog freeze

- Instrument: `apps/kernel-server/src/personal/registered_check/mod.rs`
  plus on-disk fixtures under `tests/fixtures/p2_t16_registered_check/`
- Outcome: authored. `c2a.repair.typescript` descriptor_version 2 pins
  hidden `tests/hidden.repair.test.ts` (`add(4,1)!==5`). New
  `c2a.repair.rust` pins repaired `src/repair.rs` plus public/hidden
  tests. Broken starting sources are not in `expected_file_digests`.
  Shared `frozen_registered_check_descriptor` helper owns argv/env/
  timeout/network=deny policy. Corpus helpers:
  `RepairCorpusFamily`, `reset_broken_repair_corpus`,
  `write_repaired_oracle_files`, `corpus_snapshot_digest`,
  `repaired_source_bytes`.
- Disposition: failure-first tests below must prove freeze + D02 gap.

### D01-LOCAL-01 — cargo fmt (Windows GNU allowlist)

- Instrument: `cargo fmt --all -- --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass`
- Note: Rust build/test/Clippy are `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`).

### D01-LOCAL-02 — check:consistency

- Instrument: `pnpm run check:consistency`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass` (275 requirements, 55 error codes, 74 schemas, 89 vectors,
  Personal plan/Gates, leases verified)

### D01-LOCAL-03 — handbook + docs-sync-gate

- Instrument: bilingual `execution-chain-status` update +
  `node tools/src/fill-handbook-fingerprints.mjs` +
  `node tools/src/docs-sync-gate.mjs --staged`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass` — `check-handbook` OK (54×2 locales); generator `--check` OK
  (18 pages byte-identical); docs-sync-gate OK without `DOCS_IMPACT_NONE`

### D01-LINUX-01 — exact-revision native tests (`48fa7d0465cd8499f41c46aba41522711e3c8583`)

- Instrument: `cargo test -p kernel-server --test p2_t16_registered_check --locked`
  then `cargo test -p kernel-server --bin kernel-server --locked`
- Environment: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`), worktree
  `/home/wuz/agent-kernel-worktrees/p2-t22-48fa7d0` at exact
  `48fa7d0465cd8499f41c46aba41522711e3c8583`
- Outcome: `pass` — `p2_t16_registered_check` 3/3 (including
  `fixed_rust_worker_runs_without_shell_or_ambient_environment`); kernel-server
  bin tests 290/290 including D01 freeze, hidden-test, and write-alone gap
  proofs. `B01-Desktop-Linux-002` untouched.
- Note: Windows PowerShell `|` in SSH `-- --test production_write` filters is
  forbidden (pipe / zero matches); use a single substring or no filter.

### D01-LINUX-02 — Clippy deny-warnings at `48fa7d04` (superseded)

- Instrument: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- Environment: `DEV-LINUX-NATIVE-01` at exact `48fa7d04`
- Outcome: `fail` — `clippy::collapsible_if` in
  `scheduler_authority/tests.rs` `artifact_dir_contains_schema` (nested
  `if let Ok(bytes)` + inner `if`). Same defect as Ubuntu required CI.

### D01-CI-01 — Ubuntu required CI at `48fa7d04` (superseded)

- Instrument: GitHub Actions run
  [31914724268](https://github.com/agentkernel/cognitive-os/actions/runs/31914724268)
  / job `verify (ubuntu-latest)`
- Environment: `ubuntu-latest` supporting CI
- Outcome: `fail` — Clippy `-D warnings` on the same `collapsible_if` at
  `scheduler_authority/tests.rs:3730`. Tests did not run. `required-ci`
  failed because `VERIFY_RESULT=failure`. Windows is
  `not-run by owner-directed Linux-only route`.

### D01-IMPL-02 — collapse artifact-schema scan (Clippy)

- Instrument: `apps/kernel-server/src/personal/scheduler_authority/tests.rs`
  `artifact_dir_contains_schema`
- Outcome: authored. Nested `if let` + inner `if` replaced with
  `is_some_and` so `-D warnings` Clippy accepts the helper. Assertion
  unchanged: write-alone must still prove no
  `personal-registered-check-evidence/0.1` artifact.
- Disposition: test-only; no product or handbook semantic change.

### D01-LOCAL-04 — cargo fmt after Clippy flatten

- Instrument: `cargo fmt --all -- --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass`

### D01-CI-02 — Ubuntu required CI at `4a737884`

- Instrument: GitHub Actions run
  [31915159130](https://github.com/agentkernel/cognitive-os/actions/runs/31915159130)
  (`resolve validation route`, `verify (ubuntu-latest)`, `required-ci`)
- Environment: `ubuntu-latest` supporting CI
- Outcome: `pass` at exact `4a737884d80a2a25b350f965ea9b92229e6ea356`
- Note: Windows is `not-run by owner-directed Linux-only route`.
  `B01-Desktop-Linux-002` untouched.

### D01-LINUX-03 — Clippy deny-warnings + fmt at `4a737884`

- Instrument: `cargo clippy --workspace --all-targets --locked -- -D warnings`
  and `cargo fmt --all -- --check`
- Environment: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`) at exact
  `4a737884d80a2a25b350f965ea9b92229e6ea356`
- Outcome: `pass` (rustc 1.97.1). Kernel-server bin tests at the prior
  freeze revision `48fa7d04` were 290/290; D01-CI-02 is the supporting CI
  for the Clippy flatten head.
- Disposition: D01 freeze + failure-first tests accepted for this task.
  D02 starts below.

## D02 — write → RegisteredCheckRun → verifier → acceptance

### D02-DOC-01 — lease expansion and snapshot

- Instrument: `docs/plan/PARALLEL-LANES.md` Task/slice `P2-T22/D02` with
  scheduler_authority glob, `crates/cognitive-kernel/src/harness.rs`, and
  mapped handbook pages; `PROGRESS.md` Current snapshot; formal plan
  evidence note
- Outcome: `pass` (authored). Lease remains
  `lease/personal/P2-T22/governed-software-repair`. D01 marked `done`.
  BR-02 stays `in-progress` until D03 close.
- Disposition: opens D02 production wiring.

### D02-IMPL-01 — sequential governed journey

- Instrument: `LoopDriver::return_to_decide_after_closed_effect`;
  scheduler dispatch returns Loop `ACT -> OBSERVE -> RESOLVE -> ORIENT ->
  DECIDE` after a closed intermediate mutation on a RegisteredCheck-terminated
  Task; RegisteredCheckRun still verifies and may complete; WorkspaceRead
  with the fixed-Effect verifier keeps `ACT -> VERIFY`. Admission mints a
  fresh candidate id when a receipt already exists for the policy pin.
  RegisteredCheckRun authorizes against `resource_scope_prefix` while the
  executor target stays `check://<id>`.
- Focused tests (must pass, not expected-fail): TypeScript and Rust
  journeys complete after RegisteredCheck; write-alone leaves Loop at
  `DECIDE` without check evidence; hidden-test gutting, public-test
  weakening, and out-of-scope write fail closed.
- Disposition: Ubuntu supporting CI + exact-revision Linux must confirm.
  No generic ProcessRun. No CandidateParameters variant for the check.

### D02-LOCAL-01 — cargo fmt (Windows GNU allowlist)

- Instrument: `cargo fmt --all -- --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass`
- Note: Rust build/test/Clippy are `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`).

### D02-LOCAL-02 — check:consistency

- Instrument: `pnpm run check:consistency`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass` (275 requirements, 55 error codes, 74 schemas, 89 vectors,
  Personal plan/Gates, leases verified)

### D02-LOCAL-03 — handbook + generator

- Instrument: bilingual mapped handbook updates +
  `node tools/src/fill-handbook-fingerprints.mjs` +
  `pnpm run check:handbook` +
  `node tools/src/generate-handbook.mjs --check`
- Environment: local `DEV-WIN-GNU-01`
- Outcome: `pass` — `check-handbook` OK (54×2 locales); generator `--check` OK
  (18 pages byte-identical)

### D02-CI-01 — Ubuntu required CI at `40d96665` (superseded)

- Instrument: GitHub Actions run
  [31917596065](https://github.com/agentkernel/cognitive-os/actions/runs/31917596065)
  / job `verify (ubuntu-latest)`
- Environment: `ubuntu-latest` supporting CI
- Outcome: `fail` — `crates/cognitive-kernel` did not compile:
  `return_to_decide_after_closed_effect` returned
  `Result<_, TransitionRejection>` instead of `Result<_, EffectError>`.
  Tests did not run. `required-ci` failed because `VERIFY_RESULT=failure`.
  Windows is `not-run by owner-directed Linux-only route`.

### D02-LINUX-01 — exact-revision compile at `40d96665` (superseded)

- Instrument: `cargo test -p kernel-server --test p2_t16_registered_check --locked`
- Environment: `DEV-LINUX-NATIVE-01` at exact `40d96665`
- Outcome: `fail` — same `E0308` mismatched types in
  `crates/cognitive-kernel/src/harness.rs` final `ORIENT -> DECIDE` commit.
  Assertions were not weakened.

### D02-IMPL-02 — wrap LoopDriver DECIDE commit in EffectError

- Instrument: `crates/cognitive-kernel/src/harness.rs`
  `return_to_decide_after_closed_effect`
- Outcome: authored. Final `commit_transition` is now
  `Ok(self.engine().commit_transition(&cmd)?)` matching `start_loop`.
  Journey semantics unchanged.

### D02-LINUX-03 — journey tests at `de97b4d2` (superseded)

- Instrument: `cargo test -p kernel-server --bin kernel-server --locked -- personal::scheduler_authority::tests::production_`
- Environment: `DEV-LINUX-NATIVE-01` at exact `de97b4d2`
- Outcome: `fail` — 8/10 production_ tests pass, including hidden/public
  gutting, out-of-scope write, and write-alone. The two complete-journey
  tests stayed `ACTIVE` with
  `ambiguous durable Effect bindings` after the second candidate was
  admitted. `resolve_scheduler_work_for_task` still required exactly one
  Intent per epoch. Assertions were not weakened.

### D02-IMPL-04 — unconsumed WIA selects the current Intent

- Instrument: `apps/kernel-server/src/personal/scheduler_authority/effect.rs`
  `resolve_scheduler_work_for_task`
- Outcome: authored. An unconsumed WIA binds the current Intent when several
  share the epoch; multiple Intents without a current WIA leave the binding
  unset so the next pass can admit another candidate. `select_single_effect_intent`
  remains the fail-closed helper for the single-Intent and Effect-resolution
  paths.

### D02-LINUX-02 — kernel-server tests at `0fce61df` (superseded)

- Instrument: `cargo test -p kernel-server --test p2_t16_registered_check --locked`
  then `cargo test -p kernel-server --bin kernel-server --locked`
- Environment: `DEV-LINUX-NATIVE-01` at exact `0fce61df`
- Outcome: `fail` — `p2_t16_registered_check` 3/3; kernel-server bin 293/295.
  `production_typescript_repair_journey_completes_after_registered_check` and
  the Rust twin stayed `ACTIVE`. Tick log:
  `candidate admission authorization already exists` then
  `no further governed repair candidate remains`. Root cause: WIA table is
  `UNIQUE (loop_object_id, iteration)`; the second admission reused iteration 1
  because no progress fact was recorded after the intermediate Write.
  Assertions were not weakened.

### D02-CI-02 — Ubuntu required CI at `0fce61df` (superseded)

- Instrument: GitHub Actions run
  [31917808207](https://github.com/agentkernel/cognitive-os/actions/runs/31917808207)
- Environment: `ubuntu-latest` supporting CI
- Outcome: `fail` — workspace tests hit the same two journey failures.
  Clippy did not run.

### D02-IMPL-03 — record Advanced progress after intermediate mutation

- Instrument: `apps/kernel-server/src/personal/scheduler_authority/dispatch.rs`
- Outcome: authored. After returning the Loop to `DECIDE`, the daemon records
  a monotonic `advanced` progress fact bound to the closed Effect so the next
  candidate WIA receives `iteration = last + 1`.

### D02-CI-03 — Ubuntu required CI at `de97b4d2` (superseded)

- Instrument: GitHub Actions run
  [31918058549](https://github.com/agentkernel/cognitive-os/actions/runs/31918058549)
- Environment: `ubuntu-latest` supporting CI
- Outcome: `fail` — same two journey failures as D02-LINUX-03
  (`ambiguous durable Effect bindings`). Assertions were not weakened.

### D02-LINUX-04 — journey tests at `12a91554` (superseded)

- Instrument: `cargo test -p kernel-server --bin kernel-server --locked -- personal::scheduler_authority::tests::production_`
- Environment: `DEV-LINUX-NATIVE-01` at exact `12a91554`
- Outcome: `fail` — 8/10 production_ tests pass. Ambiguous Effect bindings
  are gone. The two complete-journey tests reached RegisteredCheckRun then
  stayed `ACTIVE` with `production verifier did not pass`. `cargo test --bin
  kernel-server` uses the libtest harness as `current_exe`, so the spawned
  `--personal-registered-check-worker` child exits non-zero even when file
  digests match. Assertions were not weakened.

### D02-CI-04 — Ubuntu required CI at `12a91554` (superseded)

- Instrument: GitHub Actions run
  [31918385857](https://github.com/agentkernel/cognitive-os/actions/runs/31918385857)
- Environment: `ubuntu-latest` supporting CI
- Outcome: `fail` — same two journey failures as D02-LINUX-04. Assertions
  were not weakened.

### D02-IMPL-05 — in-process digest oracle under bin unit tests

- Instrument: `apps/kernel-server/src/personal/registered_check/mod.rs`
  `SystemRegisteredCheckRunner`
- Outcome: authored. Production still spawns `current_exe` with env_clear.
  `cfg(test)` invokes the same `run_registered_check_worker` digest oracle
  in-process because the bin test harness is not kernel-server `main`.
  Isolation spawn remains covered by `tests/p2_t16_registered_check.rs`.

### D02-LINUX-05 — kernel-server tests at `f7172342`

- Instrument: `cargo test -p kernel-server --test p2_t16_registered_check --locked`
  then `cargo test -p kernel-server --bin kernel-server --locked`
- Environment: `DEV-LINUX-NATIVE-01` at exact `f7172342`
- Outcome: `pass` — p2_t16 3/3; kernel-server bin 295/295 including both
  complete-journey tests and hidden/public/out-of-scope negatives. First
  full-bin attempt had 4 `p2_t17_a7_failure_first` failures that passed on
  isolation re-run and on the immediate full-bin re-run (pre-existing
  parallel flake, not journey assertions).

### D02-CLIPPY-01 — workspace clippy at `f7172342` (superseded)

- Instrument: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- Environment: `DEV-LINUX-NATIVE-01` at exact `f7172342`
- Outcome: `fail` — `clippy::expect_used` in `load_loop_state`. Assertions
  were not weakened.

### D02-IMPL-07 — replace expect with unwrap in the Loop helper

- Instrument: `apps/kernel-server/src/personal/scheduler_authority/tests.rs`
  `load_loop_state`
- Outcome: authored. The module already allows `clippy::unwrap_used`.

### D02-IMPL-06 — keep timeout and output-ceiling fail-closed in the test oracle

- Instrument: `apps/kernel-server/src/personal/registered_check/mod.rs`
  `observe_registered_check_digest_oracle`
- Outcome: authored. The in-process path still applies the descriptor
  timeout and `output_limit_bytes` so `cargo test --bin kernel-server`
  does not warn unused `timeout` / `OutputTooLarge` under `-D warnings`.

### D02-LINUX-06 — clippy and fmt at `4a803070`

- Instrument: `cargo test -p kernel-server --bin kernel-server --locked -- personal::scheduler_authority::tests::production_`
  then `cargo clippy --workspace --all-targets --locked -- -D warnings`
  then `cargo fmt --all -- --check`
- Environment: `DEV-LINUX-NATIVE-01` at exact `4a803070`
- Outcome: `pass` — production_ 10/10; workspace Clippy `-D warnings` clean;
  rustfmt check clean.

### D02-CI-05 — Ubuntu required CI at `4a803070`

- Instrument: GitHub Actions run
  [31919267639](https://github.com/agentkernel/cognitive-os/actions/runs/31919267639)
- Environment: `ubuntu-latest` supporting CI
- Outcome: `pass` — `verify (ubuntu-latest)` including workspace Rust tests,
  Clippy deny-warnings, rustfmt, handbook, consistency, and `required-ci`.
  Windows is `not-run by owner-directed Linux-only route`.

## D03 — exact-revision linux-002 matrix and task closure

### D03-LINUX-01 — restart / unknown-outcome / resource / secret / cleanup

- Instrument:
  `cargo test -p kernel-server --bin kernel-server --locked -- personal::registered_check::`
  (21/21); `personal::p2_t17_a7_failure_first` (15/15);
  `cargo test -p kernel-server --test p2_t16_registered_check --locked` (3/3);
  `personal::scheduler_authority::tests::production_` (10/10);
  `private_tick_dispatches_admitted_workspace_read_through_production_router`
  (1/1); then `cargo test --workspace --locked` (0 failed, kernel-server bin
  295/295).
- Environment: `DEV-LINUX-NATIVE-01` at exact `4a803070`
- Outcome: `pass`. Matrix coverage:
  - restart / unknown-outcome: `crash_mid_dispatch_is_indeterminate_after_restart_and_never_redispatched`,
    `crash_after_dispatch_reconciles_artifact_under_original_key`,
    A7 original-key replay / lost-response / success-before-receipt;
  - resource / timeout / orphan / oversize: `timeout_output_orphan_write_and_network_boundaries_fail_closed`;
  - descriptor drift: `descriptor_version_or_field_drift_is_rejected`;
  - exit-0-without-verification: write-alone journey stays `ACTIVE` with no
    check evidence; `nonzero_exit_is_evidence_but_cannot_pass_independent_verification`;
  - secret: `frozen_repair_corpora_contain_no_secret_shaped_bytes`; fixture
    tree scan found 0 secret-shaped hits (needles exist only in that negative);
  - cleanup: passing tests call `fixture.cleanup()` / `remove_dir_all`; four
    leftover `/tmp/cognitiveos-p2-t17-*-267145-*` dirs from the earlier A7
    parallel flake were removed; older 2026-08-14 A7 leftovers were left
    untouched as out-of-task residue.
- C1 WorkspaceRead with the fixed-Effect verifier still completes (`ACT -> VERIFY`).
- `B01-Desktop-Linux-002` was not used.

### D03-LOCAL-01 — Windows GNU route

- Instrument: owner-directed Linux-only product-validation route
- Environment: `DEV-WIN-GNU-01`
- Outcome: `not-run by owner-directed Linux-only route`

### D03-ACCEPT-01 — whole-task acceptance mapping

- TypeScript and Rust repair journeys complete after RegisteredCheckRun +
  independent verifier + acceptance on one Task.
- Hidden-test gutting, public-test weakening, and out-of-scope write fail
  closed on the production journey.
- Descriptor drift, timeout/orphan/oversize/network, restart unknown-outcome,
  and exit-without-independent-verification fail closed on the registered-check
  and A7 suites run at the same revision.
- Public C1 WorkspaceRead is unchanged.
- Ubuntu required CI green at `4a803070` (run `31919267639`).
- Claim ceiling remains `hypothesis`; no Gate/release/Profile/B01/EVAL
  promotion.
