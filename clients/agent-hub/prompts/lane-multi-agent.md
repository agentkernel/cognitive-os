# 接续提示词 — Agent Hub Multi-Agent 车道（MULTI）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界：Lead 只产 proposal；DAG/预算/并发/lease/提交由确定性调度器执行。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；安全负例先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；禁多层递归/跨 Host/自治 A2A。

## 本车道任务

- canonical：[collaboration/lead-workers.md](../docs/collaboration/lead-workers.md)
- 计划：[clients/agent-hub/plan/lane-multi-agent.md](../plan/lane-multi-agent.md)
- 目标：单 Host、一层 Lead+Workers 确定性调度器、Worker 隔离与冲突、handoff 与群组完成。

## gate 与允许范围（当前 blocked）

依赖 HOST + DESK + CRED（worktree）；未满足后端 gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：Lead 不直接派发/停止/提交、停止一个 Worker 不误杀其他进程、child done 不完成 parent、handoff 无越权。oracle：POC-COLLAB-001/002/003。任务 AH-MULTI-01..03 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
