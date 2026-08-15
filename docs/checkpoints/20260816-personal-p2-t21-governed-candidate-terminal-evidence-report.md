# P2-T21 Governed candidate parameters and terminal evidence — running validation report

- Task: `P2-T21`
- Branch: `personal/P2-T21-candidate-parameters-terminal-evidence`
- PR: [#220](https://github.com/agentkernel/cognitive-os/pull/220)
- Head at report open: `0404f0be90413525c90e28cf940326a750fb3df7`
- Base: `origin/main` after P2-T20 merge
- Lease: `lease/personal/P2-T21/governed-candidate-parameters-terminal-evidence`
- Change class: `implementation-only` (test fixture / internal
  classifier only in the D02/D03 closure pass; `DOCS_IMPACT_NONE`
  recorded on every commit)
- Claim ceiling: implementation evidence only; no Gate, release,
  Profile, B01, EVAL, or Agent-benefit claim

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
  worktree at `~/agent-kernel-worktrees/p2-t21-<sha7>`；只做 native build/test/clippy/fmt
  验证；不触碰 `B01-Desktop-Linux-002` guest / EVAL-004 campaign roots。
- `B01-Desktop-Linux-002` guest 属于 owner-directed evaluation campaign，与本 task
  验证无关，本任务不使用。

## D01 (done in prior commits)

D01 candidate parameters contract、canonical digest 与 sealed candidate
persistence、governed Intent binding 已在 head `1223f9e2` 前的 commit
系列完成，Ubuntu + Windows required CI 通过（run `31881772980`）。

## D02 (fixture + integration closure at 0404f0be)

产品实现（`1aaebb27` "Complete P2-T21 production evidence path"）建立
public Task admission → Pi candidate → daemon canonicalization → governed
Intent → scheduler → production Search/Write/Patch router 的端到端可达
路径。该 commit 引入的三个新集成/单元测试在推送时于 Ubuntu CI 失败；本
报告追踪 fixture-only 修复到达 green 的过程。

### D02-CI-01 — Ubuntu CI regression triage (superseded)

- Instrument: GitHub Actions `verify (ubuntu-latest)` on head
  `1aaebb27ef9dc8869fdabdc9c05bf9fa3cf2b8cd`
- Environment: Ubuntu supporting CI
- Outcome: `fail` — 3 failing tests:
  `personal::scheduler_authority::tests::admitted_search_write_and_patch_reach_production_sinks_on_later_ticks`,
  `personal::scheduler_authority::tests::patch_preimage_drift_fails_closed_before_workspace_publication`,
  `personal::task_api::evidence_tests::reconcile_class_uses_only_durable_effect_states`,
  and a compile error on `NativeToolDescriptor.effect_class`.
- Disposition: root-caused as test-fixture drift; classifier lint drift;
  and a resource-scope mismatch. Fixes recorded incrementally below.

### D02-FIX-01 — remove non-existent `effect_class` field usage

- Commit: `6a95457d`
- Instrument: `cargo fmt --check` on the touched file
  (`apps/kernel-server/src/personal/scheduler_authority/tests.rs`);
  local static.
- Outcome: `pass`.
- Note: `NativeToolDescriptor` has no `effect_class` field; two
  scheduler_authority test constructions now hardcode `EffectClass::Pure`
  to match the surrounding pattern.

### D02-FIX-02 — durable-only `classify_reconcile_state` + `workspace:` capability prefix

- Commit: `50492db4`
- Instrument: local `cargo fmt --check` + Ubuntu CI `verify (ubuntu-latest)`
- Ubuntu head after fix: same commit; failure surface now shows the
  three integration tests failing on tick-2 authorization,
  `classify_reconcile_state` case has flipped to pass.
- Product change: `classify_reconcile_state` now uses only durable
  effect states (`RECONCILED | VERIFIED | VERIFY_FAILED`): all durable →
  `"closed"`; any durable + any non-durable → `"pending_reconciliation"`;
  else `"must_reconcile"`. Internal classifier — no HTTP/CLI surface
  change, `DOCS_IMPACT_NONE` recorded and mapped handbook fingerprints
  refreshed.

### D02-FIX-03 — production-chain fixture deadline

- Commit: `7bfa26e8`
- Instrument: Ubuntu CI `verify (ubuntu-latest)`
- Outcome: still `fail` on tick 2 ceiling STOP (progressing past
  "v0.2 contract has no deadline").
- Note: the shared scheduler_authority fixture set `deadline: None`;
  `load_scheduler_authority_snapshot` fails closed on a missing
  deadline. Fixture now uses a fixed future UTC deadline.

### D02-FIX-04 / D02-FIX-05 — ceiling override support

- Commits: `f53493c2`, `ea55a7a6`
- Instrument: Ubuntu CI `verify (ubuntu-latest)`
- Outcome: still `fail` on ContractUnavailable (progressing past
  step- and retry-ceiling STOP).
- Note: `evaluate_authority_ceilings` fires `StepCeilingReached` when
  `completed_steps >= step_ceiling` and `RetryCeilingReached` when
  `retry_count >= retry_ceiling`. Fixture now exposes
  `ContextFixtureExecutionOptions.max_iterations` and `.max_retries`
  overrides (default 1 and 0 for every existing caller); the two
  production-chain tests request 4 for both.

### D02-FIX-06 — independent verifier_ref + `expect_used` lint allow

- Commit: `0404f0be`
- Instrument: Ubuntu CI `verify (ubuntu-latest)`
- Outcome: `pass` (all three regressions cleared; Clippy clean).
- Note: `activate_task_for_worker_authorization` requires every
  `Acceptance` condition to carry a non-empty `verifier_ref`; fixture
  now uses `verifier://personal/fixed-effect` (same as the other
  private-tick tests). `evidence_tests` mod gets the same
  `#[allow(clippy::expect_used, clippy::panic)]` as its sibling
  `#[cfg(test)]` modules.

### D02-CI-02 — Ubuntu required CI at `0404f0be`

- Instrument: GitHub Actions `verify (ubuntu-latest)` +
  `resolve validation route` + `required-ci`
- Environment: Ubuntu supporting CI
- Revision: `0404f0be90413525c90e28cf940326a750fb3df7`
- Outcome: `pass` — `verify` 2m47s, `required-ci` and
  `resolve validation route` both green.
- Disposition: closes the D02 Ubuntu supporting evidence.

### D02-LINUX-01 — DEV-LINUX-NATIVE-01 kernel-server binary tests

- Instrument: `cargo test -p kernel-server --bin kernel-server --locked`
- Environment: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, hal9000,
  Ubuntu 22.04, Rust 1.97.1)
- Worktree: `/home/wuz/agent-kernel-worktrees/p2-t21-0404f0be`
- Revision: `0404f0be90413525c90e28cf940326a750fb3df7`
- Outcome: `pass` — 283/283 tests passed in 3.51s (contains
  `admitted_search_write_and_patch_reach_production_sinks_on_later_ticks`,
  `patch_preimage_drift_fails_closed_before_workspace_publication`,
  `candidate_digest_mismatch_fails_before_effect_or_workspace_io`, and
  the durable evidence classifier tests).
- Disposition: closes the D02 production-chain exact-revision evidence
  on the registered native Linux host.

### D02-LINUX-02 — DEV-LINUX-NATIVE-01 full workspace tests

- Instrument: `cargo test --workspace --locked`
- Environment: same as D02-LINUX-01
- Outcome: `pass` — every workspace target reports
  `test result: ok. N passed; 0 failed; ...`. No `FAILED`, `error[`,
  or panic in the log.
- Disposition: closes the D02 workspace-wide regression evidence.

### D02-LINUX-03 — DEV-LINUX-NATIVE-01 Clippy (-D warnings)

- Instrument: `cargo clippy --workspace --all-targets --locked -- -D warnings`
- Environment: same as D02-LINUX-01
- Outcome: `pass` — `Finished dev profile ... in 32.83s`, no warning
  emitted.
- Disposition: closes the D02 lint evidence on Linux; matches the
  Ubuntu CI Clippy result.

### D02-LINUX-04 — DEV-LINUX-NATIVE-01 formatting

- Instrument: `cargo fmt --all -- --check`
- Environment: same as D02-LINUX-01
- Outcome: `pass` — no diff output; exit 0.
- Disposition: closes the D02 formatting evidence on Linux.

## D03 (delivered in `1aaebb27`, verified by the same runs)

D03 kernel-server task-channel terminal-evidence endpoint
(`GET /task/evidence?task_ref=…`) and the `admin-cli evidence` caller
are covered by the same `evidence_tests` unit module and by the
D02-LINUX-01/02 kernel-server + workspace tests. `classify_reconcile_state`
is now tested against durable-only inputs (D02-FIX-02).

## Focused negatives coverage

The `admitted_search_write_and_patch_reach_production_sinks_on_later_ticks`
suite iterates search/write/patch. Sibling negatives at head `0404f0be`:

- `candidate_digest_mismatch_fails_before_effect_or_workspace_io` —
  digest drift blocks candidate persistence and workspace I/O
  (D02-LINUX-01 pass).
- `patch_preimage_drift_fails_closed_before_workspace_publication` —
  preimage drift produces `NOT_EXECUTED`, not a mutation receipt
  (D02-LINUX-01 pass).
- `candidate_mutation_rejects_router_incompatible_raw_digest_preimage`
  and family/epoch/action drift tests already present in
  `scheduler_authority::tests` — see D02-LINUX-01.
- `runtime_spine_outcome_unknown_reconciles_original_key_and_rejects_blind_retry`,
  `unknown_native_workspace_read_reconciles_original_key_without_second_read`,
  `unknown_native_workspace_search_reconciles_original_key_without_second_scan`,
  `unknown_process_check_dispatch_reconciles_original_key_without_advancing_task`,
  `http_fetch_effect_protocol_restart_keeps_unresolved_attempt_indeterminate`,
  `http_fetch_effect_protocol_restart_recovers_completed_key_bound_receipt` —
  all pass in D02-LINUX-01 (unknown/restart/CAS negatives).

Missing / follow-up: cross-task and wrong-channel negatives against
the new terminal-evidence endpoint are covered by
`evidence_query_rejects_missing_duplicate_and_malformed_values` and the
authenticated task-channel isolation in `task_api::evidence_tests`;
larger public HTTP restart coverage remains a supporting-CI-only
concern and is not gating this task's supported validation.

## Docs-sync

Every commit in the D02/D03 closure pass ran the local
`docs-sync-gate`:

- `50492db4` regenerated the mapped handbook http-api pages and
  refreshed `task_api.rs` fingerprints (source-map only; no semantic
  change).
- `0404f0be` did the same for the `expect_used` allow.
- All others acknowledged `DOCS_IMPACT_NONE` for test-fixture-only
  changes.

`check-handbook`: OK (54 documents × 2 locales, 9 generated,
coverage/link/fingerprint/status/secret checks verified) at every
commit.

## Ready-for-merge criteria

- [x] D01 acceptance evidence (prior CI)
- [x] D02 production-chain reachability (D02-LINUX-01/02/03/04 +
      D02-CI-02)
- [x] D03 durable redacted terminal evidence
      (`GET /task/evidence` + `admin-cli evidence`; unit + integration
      evidence in the same runs above)
- [x] Focused negatives (digest / preimage / family / epoch / unknown /
      restart / missing-CAS / task-channel isolation)
- [x] Docs-sync every commit
- [x] Ubuntu required CI green at head `0404f0be`
- [x] `DEV-LINUX-NATIVE-01` exact-revision validation at head
      `0404f0be`

Next actions: flip PR #220 Draft → ready, merge, close lease
`lease/personal/P2-T21/governed-candidate-parameters-terminal-evidence`,
delete task branch locally and remote, fast-forward local `main` to the
merge commit, and update the `PROGRESS.md` `Current snapshot` and
`Benchmark readiness product train` rows to reflect P2-T21 done and
P2-T22 as the next ready task.
