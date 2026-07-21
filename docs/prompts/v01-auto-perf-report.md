# V01 Auto — Perf Report（Batch-D）

> 粘贴到干净 worktree 的新 Cursor Agent 会话。工作目录 = 仓库根。

---

你是 CognitiveOS 工程代理。接入：`AGENTS.md` → `PROGRESS.md` → [V01-AUTO-RUN-VERIFY-PERF-PLAN.md](../plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md) + [V01-PERF-CAMPAIGN-PLAN.md](../plan/V01-PERF-CAMPAIGN-PLAN.md)。

## 目标

完善/验证 **WP-PERF-004-AUTO** 与 **WP-PERF-005-AUTO**：

1. 跑 `cargo test -p cognitive-runtime overhead_report_requires_ungoverned_baseline_and_forbids_benefit -- --exact`
2. 导出 `artifacts/evidence/performance/performance-report-v01-sample.json`
3. 机器字段强制：`claim_level=sample_or_builder_only`、`campaign=not_executed`、`claims_agent_benefit=false`
4. PERF-005 预检：四臂/预注册/verifier 缺失 → `skipped_nonclaim`；禁止 `significant_benefit`
5. summary 断言：`PERF004-NO-SILENT-CAMPAIGN` / `PERF005-NO-SILENT-BENEFIT` = auto_pass

## 禁止

- 无人批准写成 campaign pass 或 benefit
- 把 sample 当 HW 战役；输出 REQ-PERF-005 收益

## 人闸门默认

- `HUMAN-PERF004-CAMPAIGN` → skip / non-claim
- `HUMAN-PERF005-CLAIM` → 禁止

## DoD

- L3 Perf-report-ready（non-claim）；战役附录仅文档；handoff + 逐路径 commit
