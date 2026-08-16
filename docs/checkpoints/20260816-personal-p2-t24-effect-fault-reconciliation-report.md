# P2-T24 Effect fault and reconciliation observation — running validation report

- Task: `P2-T24`
- Branch: `personal/P2-T24-effect-fault-reconciliation`
- PR: https://github.com/agentkernel/cognitive-os/pull/223
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

### D01-CI-01 — Ubuntu supporting CI at `1ef365a7` (superseded)

- Instrument: GitHub Actions run
  [31924285207](https://github.com/agentkernel/cognitive-os/actions/runs/31924285207)
- Outcome: `fail` — `kernel-server` did not compile under `-D warnings`: unused
  `IntentRow` / `StoredObject` imports because the history projector used
  fully-qualified paths.
- Disposition: use the imported types in `project_effect_history_entry`.
  Compile-fix commit `eaab6e3ec7a8587a5ea20f08e13f25be23172e09`.

### D01-CI-02 — Ubuntu supporting CI at `eaab6e3e` (superseded)

- Instrument: GitHub Actions run
  [31924662017](https://github.com/agentkernel/cognitive-os/actions/runs/31924662017)
- Outcome: `fail` — Clippy `-D warnings` on `AuthorizedFaultPoint` sharing the
  `Before` postfix (`clippy::enum_variant_names`). Same allow already used on
  `CampaignFaultPoint`.
- Disposition: `#[allow(clippy::enum_variant_names)]` on the D01 enum, included
  in the D02 head.

## D02 — original-key reconcile under persisted profiles

### D02-TEST-01 — persisted profile original-key restart, mutation count 1

- Instrument: `campaign_observation::p2_t24_d02_tests::persisted_profile_reconciles_original_key_once_after_restart`
- Oracle: a D01 persisted `mutation_after_receipt_before` profile authorizes
  injection; restart queries only the original key; mutation count is 1;
  replacement-key Intent is `DuplicateEffect`; a mismatched fault point is
  `FaultUnauthorized`; `acceptance_ref` stays absent.
- Status: `pass` on Ubuntu supporting CI run
  [31925407318](https://github.com/agentkernel/cognitive-os/actions/runs/31925407318)
  (workspace tests green; Clippy failed separately).

### D02-TEST-02 — dispatch-before keeps mutation 0 and Indeterminate

- Instrument: `campaign_observation::p2_t24_d02_tests::dispatch_before_profile_keeps_mutation_zero_and_indeterminate`
- Oracle: mutation count stays 0; restart outcome is Indeterminate; dispatch
  count stays 0; `acceptance_ref` stays absent.
- Status: `pass` on Ubuntu supporting CI run
  [31925407318](https://github.com/agentkernel/cognitive-os/actions/runs/31925407318)
  (workspace tests green; Clippy failed separately).

### D02-TEST-03 — missing/default-off/unauthorized files never inject

- Instrument: `fault_profile::tests::{missing_and_default_off_profiles_never_inject,unauthorized_campaign_file_never_injects,authorized_enabled_profile_exposes_the_pinned_point}`
  and `campaign_observation::p2_t24_d02_tests::missing_profile_cannot_authorize_injection`
- Oracle: production consult returns `None` unless an authorized enabled
  profile names one of the four fixed points.
- Status: `pass` on Ubuntu supporting CI run
  [31925407318](https://github.com/agentkernel/cognitive-os/actions/runs/31925407318)
  (workspace tests green; Clippy failed separately).

### D02-IMPL-01 — production native dispatch consults persisted profiles

- Instrument: `tool_executor/router.rs` `bind_fault_profiles` /
  `authorized_fault_point`; `scheduler_authority/effect.rs` and `dispatch.rs`
  inject only at the four fixed points; `select_single_effect_intent` is
  unchanged.
- Outcome: authored. Missing, default-off, and unauthorized file content never
  inject. P2-T17 test-only `CampaignAuthorization::authorized` remains
  `cfg!(test)`.

### D02-CI-01 — Ubuntu supporting CI at `a2a92a02` (superseded)

- Instrument: GitHub Actions run
  [31925407318](https://github.com/agentkernel/cognitive-os/actions/runs/31925407318)
- Outcome: workspace tests **pass**; Clippy `-D warnings` **fail** on
  `clippy::question_mark` in `load_enabled_authorized_profile`.
- Disposition: use `profile.fault_point?;` instead of an explicit `is_none` return.
