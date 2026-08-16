# P2-T26 Bounded OS observation plane — running validation report

- Task: `P2-T26`
- Branch: `personal/P2-T26-bounded-os-observation`
- PR: [Draft PR #225](https://github.com/agentkernel/cognitive-os/pull/225)
- HEAD: `268cb4dcb9d5c95f5129dc4121b9625e0cd26b81`
- Lease: `lease/personal/P2-T26/bounded-os-observation`
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
- `B01-Desktop-Linux-002` 属于 owner-directed EVAL-004-only guest；本任务不使用。

## D01 — authenticated bounded O2/O3/O4 collectors

### D01-TEST-01 — empty collectors are controlled zeros

- Instrument: `apps/kernel-server/src/personal/observation.rs` unit tests
  (`missing_samples_are_controlled_zeros_not_silent_counts`) and
  `apps/kernel-server/tests/p2_t26_observation_plane.rs`.
- Oracle: `GET /task/observation?family=o2|o3|o4&task_ref=…` returns HTTP 200 with
  `observed_zero=true`, `denominator=0`, and a named `negative_control`
  (`no_authorization_sample` / `no_cache_or_compaction_sample` /
  `no_scheduler_sample`). This is not a silent default-zero count and not a
  missing-route 404.
- Windows: `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
- `DEV-LINUX-NATIVE-01` at `36fcad84` and `268cb4dc`: **pass** (D01-LINUX-01 / D01-LINUX-02).

### D01-TEST-02 — channel, restatement, and leakage negatives

- Instrument: same HTTP test plus
  `prompt_restatement_and_unknown_family_fail_closed`,
  `management_and_writes_are_channel_forbidden`.
- Oracle: management `GET /management/resource/v1/observation` returns 403
  `RESOURCE_OBSERVATION_CHANNEL_FORBIDDEN`. `prompt`/`body`/`receipt`/
  `capability` query keys return 400 `TASK_OBSERVATION_QUERY_FORBIDDEN`.
  POST write is 403 `RESOURCE_OBSERVATION_WRITE_FORBIDDEN`. Projection JSON
  has no Context body or capability material.
- Windows: `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
- `DEV-LINUX-NATIVE-01` at `36fcad84` and `268cb4dc`: **pass** (D01-LINUX-01 / D01-LINUX-02).

### D01-TEST-03 — deny, cache/compaction, and scheduler probes

- Instrument: `authorization_deny_is_an_active_negative_control`,
  `cache_revalidation_and_compaction_zeros_are_controlled`,
  `scheduler_zero_runnable_records_a_probe_and_names_missing_counters`.
- Oracle: a recorded deny is `deny_recorded` with `input_digest` and
  `reason_code`; cache miss/revalidated counts are non-default; missing
  compaction is `compaction_not_invoked`; a runnable probe with count 0 still
  has denominator 1 and names missing O4 counters (`no_budget_stop_sample`).
- Windows: `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
- `DEV-LINUX-NATIVE-01` at `36fcad84` and `268cb4dc`: **pass** (D01-LINUX-01 / D01-LINUX-02).

### D01-IMPL-01 — public surfaces and production hooks

- Instrument: `apps/kernel-server/src/personal/observation.rs`,
  `apps/kernel-server/src/personal/server.rs`,
  `apps/kernel-server/src/personal/task_api.rs`,
  `apps/kernel-server/src/personal/scheduler_authority/{candidate,context,dispatch,effect}.rs`
- Outcome: authored. Overlay file `$data_dir/personal-observation-plane.json`
  schema `cognitiveos.personal.observation-plane/0.1`. Families: `o2`
  (authorization grant/deny receipts), `o3` (cache miss/revalidated/evicted plus
  compaction loss-manifest digest), `o4` (runnable_count, lease_acquired,
  budget_stop, stale_fence_denial, and a `queue_wait` zero probe). `fairness`
  stays a named missing counter until a real starvation signal exists. Daemon
  binds the data dir at startup. Task GET `/task/observation` and
  `/task/resource/v1/observation`. Management is forbidden.

### D01-CI-01 — Ubuntu supporting CI

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31933795702`](https://github.com/agentkernel/cognitive-os/actions/runs/31933795702)
  on Draft PR [#225](https://github.com/agentkernel/cognitive-os/pull/225) at
  `36fcad84`.
- Outcome: `fail` — Clippy `-D warnings` rejected duplicated
  `#[allow(clippy::too_many_arguments)]` attributes and collapsible `if`
  let-chains in `observation.rs`. Tests were not the failure. Superseded by
  the Clippy-fix head.

### D01-LINUX-01 — focused observation tests at `36fcad84`

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`, rustc 1.97.1)
  worktree `/home/wuz/agent-kernel-worktrees/p2-t25-4dfd4f89` checked out
  `36fcad84`. `kernel-server` is binary-only; the filter used
  `cargo test -p kernel-server --bin kernel-server --locked observation`.
- Outcome: **pass** — 26/26 filtered bin tests (includes 9/9
  `personal::observation::tests`); HTTP `p2_t26_observation_plane` 1/1;
  `cargo fmt --all -- --check` pass. Clippy at this SHA **fail** (see D01-CI-01).

### D01-LINUX-02 — Clippy-fix revalidation at `268cb4dc`

- Instrument: same worktree, exact `268cb4dcb9d5c95f5129dc4121b9625e0cd26b81`.
- Outcome: **pass** — `--bin kernel-server` filter `observation` 26/26; HTTP
  `p2_t26_observation_plane` 1/1; `cargo fmt --all -- --check` pass;
  `cargo clippy --workspace --all-targets --locked -- -D warnings` pass.

### D01-CI-02 — Ubuntu supporting CI after Clippy fix

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31934303018`](https://github.com/agentkernel/cognitive-os/actions/runs/31934303018)
  on Draft PR [#225](https://github.com/agentkernel/cognitive-os/pull/225) at
  `268cb4dc`.
- Outcome: **pass** — `verify (ubuntu-latest)` and `required-ci` green on
  run [`31934303018`](https://github.com/agentkernel/cognitive-os/actions/runs/31934303018)
  at `268cb4dc`.

## D02 — O5 Effect history and O13 audit cursor/replay

### D02-TEST-01 — empty O5/O13 collectors are controlled zeros

- Instrument: `effect_and_audit_empty_windows_are_controlled_zeros` and
  `p2_t26_observation_plane` HTTP coverage.
- Oracle: `family=o5` returns HTTP 200 `observed_zero` / `no_effect_sample`
  without receipts or parameters. `family=o13` returns HTTP 200
  `no_audit_sample` with `chain_head_digest` and `high_watermark`. This is
  not a silent 0 and not `GET /task/effects` 404.
- Windows: `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
- `DEV-LINUX-NATIVE-01` at `36fcad84` and `268cb4dc`: **pass** (D01-LINUX-01 / D01-LINUX-02).

### D02-TEST-02 — stale cursor, digest break, and query isolation

- Instrument: `audit_cursor_digest_and_family_negatives_fail_closed`.
- Oracle: `cursor` beyond the high watermark is 409
  `TASK_OBSERVATION_CURSOR_STALE`. `expect_digest` mismatch is 409
  `TASK_OBSERVATION_DIGEST_BREAK`. `cursor` on `family=o2` is 400
  `TASK_OBSERVATION_QUERY_FORBIDDEN`. Invalid cursor is 400
  `TASK_OBSERVATION_CURSOR_INVALID`.
- Windows: `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
- `DEV-LINUX-NATIVE-01` at `36fcad84` and `268cb4dc`: **pass** (D01-LINUX-01 / D01-LINUX-02).

### D02-TEST-03 — restart-stable replay digest

- Instrument: `audit_replay_is_stable_across_store_reopen`.
- Oracle: two successive `family=o13` reads of the same empty window return
  the same `chain_head_digest` and `high_watermark`.
- Windows: `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
- `DEV-LINUX-NATIVE-01` at `36fcad84` and `268cb4dc`: **pass** (D01-LINUX-01 / D01-LINUX-02).

### D02-IMPL-01 — reuse Effect history; durable audit export

- Instrument: `apps/kernel-server/src/personal/observation.rs` `project_o5` /
  `project_o13`.
- Outcome: authored. O5 reconstructs the same redacted Effect fields as
  `GET /task/effects` (opaque original-key digest, stage, outcome/reconcile
  class, mutation count 0/1). O13 reads the append-only authority event log
  after `cursor`, binds event digests into a chain, filters samples to the
  task's contract/intent/effect identities, and fails closed on stale cursor,
  missing event, digest break, and sequence gap. Oversize windows set
  `samples_truncated`. `GET /task/effects` is unchanged.

### D02-CI-01 — Ubuntu supporting CI

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31933795702`](https://github.com/agentkernel/cognitive-os/actions/runs/31933795702)
  on Draft PR [#225](https://github.com/agentkernel/cognitive-os/pull/225) at
  `36fcad84`.
- Outcome: `fail` — same Clippy duplicated-attribute / collapsible-if
  defects as D01-CI-01. Superseded by the Clippy-fix head.

### D02-CI-02 — Ubuntu supporting CI after Clippy fix

- Same run as D01-CI-02 (`31934303018` at `268cb4dc`): **pass**.

## D03 — exact-revision linux-002 concurrency/restart/redaction/cleanup matrix

### D03-LINUX-01 — focused observation, fmt, Clippy at `268cb4dc`

- Instrument: same as D01-LINUX-02.
- Outcome: **pass** — already recorded. Ubuntu supporting CI `31934303018` **pass**.

### D03-LINUX-02 — kernel-server bins

- Instrument: `cargo test -p kernel-server --locked --bins` at `268cb4dc` on
  `DEV-LINUX-NATIVE-01`.
- Outcome: **pass** — 328/328.

### D03-IMPL-01 — overlay serialization and cross-task isolation

- Instrument: `OVERLAY_LOCK` around overlay load/append/project, plus
  `cross_task_samples_stay_isolated_under_concurrent_records`.
- Outcome: authored. Concurrent O2 records for two task refs are retained;
  each query returns only its own samples and no capability/receipt/parameter
  /prompt keys. Linux revalidation of this head is pending.



