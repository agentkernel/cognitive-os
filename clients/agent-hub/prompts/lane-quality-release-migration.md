# 接续提示词 — Agent Hub Quality/Release/Migration 车道（QRM）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界；完成证明只来自 authority 状态/Effect/Verification/Event。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分；测试已执行 ≠ Profile 已符合。
4. 测试先行；安全负例不可豁免。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；禁把测试/检查写成实现证据；禁把 Host ledger 改写为 authority Event。

## 本车道任务

- canonical：[traceability/evidence-index.md](../docs/traceability/evidence-index.md)、[architecture/relay-pairing-and-migration.md §6](../docs/architecture/relay-pairing-and-migration.md#6-direct--governed-迁移evidence-only)
- 计划：[clients/agent-hub/plan/lane-quality-release-migration.md](../plan/lane-quality-release-migration.md)
- 目标：Open PoC 执行与留证、发布 gate 校验、Direct→Governed evidence-only 迁移、无障碍与恢复回归。

## gate 与允许范围（当前 blocked）

依赖全部车道 + Governed 契约（外部 M6）；未满足全局 gate 前不得声明发布或迁移完成、不得 mock 冒充。安全负例（不可豁免）：任一负例失败即阻断里程碑；迁移不追认历史 authority/Verification、不改写 ledger 为 Event。oracle：evidence-index 27+ PoC 由 not-run 转 pass/documented。任务 AH-QRM-01..04 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
