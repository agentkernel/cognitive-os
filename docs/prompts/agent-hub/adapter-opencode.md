# 接续提示词 — Agent Hub OpenCode Adapter（AD-OPENCODE）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界。2. 规范优先级；冲突取保守解释。3. 四类状态用语严格区分。4. 测试先行；安全负例先行。5. 规范表面冻结；漂移登记后修正。6. P0 门禁。7. 可追溯提交。8. 红线：禁 `History/`；禁虚构规范资产；不写 provider 配置；不写 native DB（L6）。

## 本 Adapter 任务

- dossier：[adapters/tier1/opencode.md](../../../apps/cognitiveos-console/docs/agent-hub/adapters/tier1/opencode.md)
- 计划：[docs/plan/agent-hub/adapter-opencode.md](../../plan/agent-hub/adapter-opencode.md)
- 目标：接口一手核验（server/session API/ACP、native 存储、许可 SPDX）、L1/L2 与连接已运行 server、L5 SQLite 只读、账号与凭据。

## gate 与允许范围（当前 blocked）

独立 gate：OpenCode 接口一手核验（AH-B4）、平台 PoC（AH-B2）、许可核验（AH-B5）。未过 gate 前只做接口核验文档，不写实现、不 mock 解阻。安全负例（不可豁免）：连接已运行 server 无并发保证时不写、SQLite 写压力 snapshot 一致（禁 checkpoint/主库单文件复制）、不写 provider 配置、零 secret 落盘。oracle：POC-SESS-002、POC-FILE-002。任务 AH-AD-OPENCODE-01..04 见计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
