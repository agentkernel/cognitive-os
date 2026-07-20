# 接续提示词 — Agent Hub Relay/Pairing 车道（RELAY）

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
8. 红线：禁 `History/`；禁虚构规范资产；禁公网明文控制面；禁 Relay 侧解密；禁把 Relay 当 authority/ledger。

## 本车道任务

- canonical：[architecture/relay-pairing-and-migration.md](../docs/architecture/relay-pairing-and-migration.md)
- 计划：[clients/agent-hub/plan/lane-relay-pairing.md](../plan/lane-relay-pairing.md)
- 目标：E2EE 通道、配对与设备身份、扩权与本机确认边界、撤销/轮换/恢复。

## gate 与允许范围（当前 blocked）

依赖 HOST + 桌面本机能力；未满足后端/ADR/（若复用 Paseo Relay）AGPL 法务 gate 前不得写实现、不得 mock 解阻。Paseo Relay 反例（不可复制）：等权 trusted client、pairing offer 永久复用、live-session 无 replay 防护、无逐设备 revoke。安全负例（不可豁免）：MITM（matching code 绕过）拒绝、replay/乱序不重复交付、丢失设备 revoke 后全拒、push 仅 opaque hint。oracle：POC-RELAY-001/002/003/004。任务 AH-RELAY-01..04 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
