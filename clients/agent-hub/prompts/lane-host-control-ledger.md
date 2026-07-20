# 接续提示词 — Agent Hub Host/Control/Ledger 车道（HOST）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界与所有权。

## 硬纪律（全程有效）

1. 确定性边界：授权、CAS、状态迁移、硬预算、幂等、fencing、最终提交必须由确定性代码执行；概率组件只产 candidate/proposal。
2. 规范优先级：机器合同 > RFC/Core/Profile > 白皮书 > 实现建议；冲突取不扩大权限/范围/风险/预算/完成声明。
3. 四类状态用语严格区分。
4. 测试先行；先写失败测试与安全负例再实现。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；禁任意 PID 注入/劫持。

## 本车道任务

- canonical：[architecture/takeover-architecture.md](../docs/architecture/takeover-architecture.md)、[security/security-and-credentials.md](../docs/security/security-and-credentials.md)
- 计划：[docs/plan/agent-hub/lane-host-control-ledger.md](../plan/lane-host-control-ledger.md)
- 目标：per-user 非提权 Host、认证控制面、ownership generation + single controller lease、Local Event Ledger、崩溃恢复。

## gate 与允许范围（当前 blocked）

未满足 Console 后端 gate（组 1/2/7 + M5）、技术栈 ADR、（若复用 Paseo）AGPL 法务 gate 前，不得写实现代码、不得用 mock 解阻。新建 Host crate/package 前先在 PARALLEL-LANES 登记所有权，不改他人车道。安全负例（必做，不可豁免）：pipe/socket squatting 拒绝、未授权 client 拒绝、旧 generation 输入拒绝、崩溃窗口未确认动作标 unknown 不重放。oracle：POC-SEC-001/002、POC-PROC-002。任务 AH-HOST-01..05 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff（含未过 gate 与证据 not-run 状态）→ 逐路径分批提交。
