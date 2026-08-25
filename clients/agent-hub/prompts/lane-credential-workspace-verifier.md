# 接续提示词 — Agent Hub Credential+Workspace+Verifier 车道（CRED）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；安全负例先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；禁导出/复制/回写 provider secret；禁 secret 落 ledger/URL/argv/env 明文/云同步。

## 本车道任务

- canonical：[security/security-and-credentials.md](../docs/security/security-and-credentials.md)、[collaboration/lead-workers.md](../docs/collaboration/lead-workers.md)
- 计划：[clients/agent-hub/plan/lane-credential-workspace-verifier.md](../plan/lane-credential-workspace-verifier.md)
- 目标：opaque credential handle broker、多账号 profile 与切换、worktree/workspace 管理、固定 checks verifier。

## gate 与允许范围（当前 blocked）

依赖 HOST + CTR；未满足接口/后端 gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：ledger/日志/push 零 secret、不抽取 token/cookie/keychain、session 内热切换（无官方支持）拒绝、checks 单独不等于 user acceptance（完成双轴）。oracle：POC-SEC-003/004。任务 AH-CRED-01..04 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
