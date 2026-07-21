# V01 PERF Campaign Plan（可选升格附录）

- 状态：informative appendix **默认不执行**
- 父计划：[V01-AUTO-RUN-VERIFY-PERF-PLAN.md](V01-AUTO-RUN-VERIFY-PERF-PLAN.md)
- 触发闸门：`HUMAN-PERF004-CAMPAIGN`（战役）/ `HUMAN-PERF005-CLAIM`（收益）

## 默认口径（无人值守）

| 项 | 默认 |
|---|---|
| PERF-004 | 自动 sample/builder 报告；`campaign=not_executed` |
| PERF-005 | 预检 `skipped_nonclaim`；禁止 `significant_benefit` |

`pnpm run verify:local` **不得**在无人批准时把结果写成 campaign pass 或 agent benefit。

## HUMAN-PERF004-CAMPAIGN（升格到 L3-campaign-pass）

**前置**：L2 Verify-green；平台标签 = `linux_native`（参考）或书面接受的其他行；硬件拓扑与并发预注册。

**最小交付**：

1. 同条件 HW 战役测量（授权/Context/Effect p50/p95/p99、cache-hit、额外写、审批延迟、开销占比）
2. 声明 ungoverned 基线（REQ-PERF-004 / IMP-04）
3. `artifacts/evidence/performance/` 下战役报告 + digest
4. 更新 summary：`campaign=executed_pass` **仅当**人闸门批准且证据齐全

**禁止**：用 unit sample / builder 字段冒充战役；跨平台合并声明。

## HUMAN-PERF005-CLAIM（升格到 L3-benefit-claim）

**前置**：M7+ 可评测机制；四臂 harness；预注册 BenchmarkManifest；独立 verifier（见 `docs/evaluation/agent-benefit-benchmark.md`、F-026、IMP-18）。

**最小交付**：

1. Arms A/B/C/D + 至少 W1/W2
2. `comparison.claim_level ∈ {hypothesis, non_inferiority, significant_benefit}`
3. `significant_benefit` 须满足合同全部六条门槛

**默认**：继续 non-claim / 最多 hypothesis 草稿；**禁止**静默输出收益。

## 与自动编排的边界

- 自动流水线停在 **L3 Perf-report-ready（non-claim）**
- 本附录仅在人显式批准后另开会话执行；不得与改向量同 PR
