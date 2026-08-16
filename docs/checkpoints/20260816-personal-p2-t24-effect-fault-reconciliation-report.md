# P2-T24 Effect fault and reconciliation observation — running validation report

- Task: `P2-T24`
- Branch: `personal/P2-T24-effect-fault-reconciliation`
- PR: pending
- Lease: `lease/personal/P2-T24/effect-fault-reconciliation-observation`
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

## D01 — default-off authorized fault profiles and bounded Effect history

### D01-TEST-01 — task channel cannot enable fault profiles

- Instrument: `apps/kernel-server/tests/p2_t24_effect_fault.rs`
  (`public_fault_profile_denies_task_channel_and_unauthorized_campaign`)
- Oracle: task bearer `POST /task/resource/v1/fault-profile` returns 403
  `RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN`; management bearer with campaign
  `owner-local` returns 403 `RESOURCE_FAULT_PROFILE_UNAUTHORIZED`.
- Initial status: `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`). Ubuntu
  supporting CI is the first execution.

### D01-TEST-02 — GET /task/effects forbids receipts and missing-task 404

- Instrument: same HTTP test plus
  `personal::task_api::evidence_tests::effect_history_query_rejects_receipt_and_parameter_restatement`
  and `effect_history_projection_hashes_original_key_and_drops_receipts`
- Oracle: extra `receipt`/`parameters` query keys return 400
  `TASK_EFFECT_HISTORY_QUERY_FORBIDDEN`; unknown task_ref returns 404; a
  synthetic Effect body containing `receipt_ref` and raw parameters serializes
  without those fields and hashes the original key.
- Initial status: `not-run` locally.

### D01-IMPL-01 — public surfaces

- Instrument: `apps/kernel-server/src/personal/fault_profile.rs`,
  `apps/kernel-server/src/personal/task_api.rs`,
  `apps/kernel-server/src/personal/server.rs`
- Outcome: authored. Four fixed authorized fault points match the P2-T17
  campaign enum. Profiles are default-off and persist only after a management
  campaign/case grant. `GET /task/effects` reconstructs bounded history from
  durable Intent/Effect rows. `select_single_effect_intent` is unchanged.
