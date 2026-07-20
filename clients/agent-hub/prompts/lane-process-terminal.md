# 接续提示词 — Agent Hub Process+Terminal 车道（PROC）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界：停止/信号/身份判定由确定性代码执行。
2. 规范优先级：机器合同 > RFC/Core/Profile > 白皮书 > 实现建议；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；安全负例先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；任意 stdin 抢占/TIOCSTI/ptrace/内存注入/二进制 patch 永久禁止。

## 本车道任务

- canonical：[architecture/process-and-terminal.md](../docs/architecture/process-and-terminal.md)、平台事实 [sources/platform-security-ledger.md](../docs/sources/platform-security-ledger.md)
- 计划：[docs/plan/agent-hub/lane-process-terminal.md](../plan/lane-process-terminal.md)
- 目标：进程身份锚（防 PID reuse）、受控 spawn+containment、停止分层、Host-owned ConPTY、reaping。

## gate 与允许范围（当前 blocked）

依赖 HOST；未满足 Console 后端/平台 PoC/ADR gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：PID 复用零误杀、Job breakaway/WMI 逃逸阻止或检测、普通既有 console 不进入 terminal-attached、外部 PID send 被拒、信号后仍存活标 unknown。oracle：POC-PROC-001/003/004、POC-TERM-001/002。逐平台分别声明（Job vs 进程组 vs cgroup），不跨平台外推。任务 AH-PROC-01..05 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
