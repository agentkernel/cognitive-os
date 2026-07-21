# 20260721 M6 Planning Handoff（Lane-DOC）

## 1. 本次会话完成

- 落地 canonical 计划：[docs/plan/M6-PLAN.md](../plan/M6-PLAN.md)（A–G 完整：目标/WP/合入序/验收矩阵/风险/Batch-0/不做项）。
- 修订 [docs/prompts/milestone-m6.md](../prompts/milestone-m6.md)：干净 worktree、pins 52/32、安装表与 readiness 缺口、分批入口。
- 新增执行提示词：
  - [docs/prompts/m6-batch0-contracts.md](../prompts/m6-batch0-contracts.md)（Lane-CTR WP0）
  - [docs/prompts/m6-batch1-installer.md](../prompts/m6-batch1-installer.md)（Lane-RUN WP1）
- 更新 [docs/plan/PROGRESS.md](../plan/PROGRESS.md)：M6 = 计划已批准 / Batch-0 可启动；不虚报实现或测试。
- 修正型文档漂移：`DEVELOPMENT-PLAN` Console 依赖组 9 的陈旧 `46 pass / 38 not-run` → 实测 **52 / 32**；PROGRESS Console 行去掉“缺 M5 出口评审”（M5 review 已存在）。

## 2. 入口证据（核实，非对话记忆）

| 项 | 值 |
|---|---|
| `origin/main` tip（规划基线） | `3c7115c3eaa50de468505d2e125e5ad81abbf673`（PR #30） |
| runner pins | pass **52** / not-run **32** / self-check ≥**33** |
| M5 出口 | [20260721-m5-milestone-review.md](20260721-m5-milestone-review.md) **GO M6** |
| 附带条件 | D-018 持续；clients blocked；F-017 出口阻断 |
| 本地 dirty `main` | 含 personal-blog 恢复提交且 ahead/behind；**不得**用作推送基线 |

## 3. 未完成 / 进行中

- Batch-0A：M6 schema codegen + installation/readiness 缺口裁决（Lane-CTR）。
- Batch-0B/1：篡改拒装 tracer（Lane-RUN，待 0A）。
- WP2–WP10：见 M6-PLAN；产品实现/向量行为执行/F-017 闭合均未启动。
- Profile 已符合 = 0（样例 manifest 仍 planned）。

## 4. 合同缺口（实现前必须遵守）

1. **无** installation transition 机器表（`specs/transitions/` 仅五表）→ 实现按 companion 状态序列；不得声称“迁移表已消费”。
2. **无** readiness REQ/schema/vector/carrier → readiness 证据归 milestone e2e/fault；不得虚报 registry conformance。
3. F-017 仍 open，阻断 M6 出口。

## 5. 测试与证据状态

- 本批：文档落地；**无**产品实现；**无**向量状态变更；**不构成** Profile 符合。
- 静态：交付前跑 `pnpm run check:consistency` 与 `git diff --check`（本 PR）。

## 6. 下一步入口

1. 提示词：[m6-batch0-contracts.md](../prompts/m6-batch0-contracts.md)
2. 分支建议：干净 worktree `lane/ctr-m6-bindings` ← `origin/main`
3. 合入后再开 [m6-batch1-installer.md](../prompts/m6-batch1-installer.md)

## 7. 快照

- PROGRESS 已更新：是（M6 计划批准入口句）。
- 本 handoff 路径：`docs/checkpoints/20260721-m6-planning-handoff.md`
- 状态用语：规范计划已批准；实现未提供；测试未执行；Profile 未符合。
