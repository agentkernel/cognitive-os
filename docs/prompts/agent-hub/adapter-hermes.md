# 接续提示词 — Agent Hub Hermes Adapter（AD-HERMES）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界。2. 规范优先级；冲突取保守解释。3. 四类状态用语严格区分。4. 测试先行；安全负例先行。5. 规范表面冻结；漂移登记后修正。6. P0 门禁。7. 可追溯提交。8. 红线：禁 `History/`；禁虚构规范资产；**禁臆造 Hermes 接口/session 格式/条款**；任意 PID 注入永久禁止。

## 本 Adapter 任务

- dossier：[adapters/tier1/hermes.md](../../../apps/cognitiveos-console/docs/agent-hub/adapters/tier1/hermes.md)
- 计划：[docs/plan/agent-hub/adapter-hermes.md](../../plan/agent-hub/adapter-hermes.md)
- 目标：**首要且硬前置**——用官方仓库/文档 + 查询日 + version/commit 完成接口存在性与一手核验；据此定级并映射控制/账号。

## gate 与允许范围（当前 blocked，接口未核验）

本轮无 Hermes 官方接口一手事实。硬前置 gate：AH-AD-HERMES-01 接口一手核验完成前，其余任务全部 `blocked`，不得声明任何 `目标` 之外能力、不得写实现、不得 mock 解阻。无稳定接口则定级 launch-only/observe-only。安全负例（不可豁免）：普通既有进程不可 send。任务 AH-AD-HERMES-01..03 见计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
