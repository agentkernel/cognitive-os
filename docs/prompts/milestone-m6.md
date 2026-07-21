# M6 接续提示词：安装与适配、v0.1 发布

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = **干净 worktree / 仓库根**，基于 `origin/main`）。自包含，不依赖历史对话。主车道 Lane-RUN（`lane/run`），Lane-CFR 协作平台矩阵证据。公共前缀内联自 `docs/prompts/common-prefix.md`。
>
> **权威计划**：[docs/plan/M6-PLAN.md](../plan/M6-PLAN.md)。本提示词是执行入口，不以本文件覆盖计划验收矩阵。

---

你是 CognitiveOS 参考实现的工程代理。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**基线硬要求**：

1. `git fetch origin main`；确认 tip 至少含 M5 GO + pins **pass 52 / not-run 32 / self-check ≥33**（PR #30 / `3c7115c` 一带）。
2. 在干净 worktree 建车道分支；**禁止**从含 `personal-blog/**` 的本地 dirty `main` 推送。
3. 排除：`personal-blog/**`、`History/`、clients 产品实现、无关打开的 clients PR。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` 与 `docs/plan/M6-PLAN.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（含 [20260721-m5-milestone-review.md](../checkpoints/20260721-m5-milestone-review.md)），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界；② 规范优先级（机器资产 > companion/RFC > 白皮书 > 实现建议）；③ 四类状态用语，implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass；⑤ 冻结 + 漂移走台账；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构、禁改写向量。

**会话结束协议**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（见 M6-PLAN；DEVELOPMENT-PLAN M6；`specs/agent-compatibility/README.md`）

- AgentPackageManifest 验证（签名/digest；篡改拒装）。
- 安装事务与回滚（中断后无半安装可见状态）。**注意**：仓库**无** installation transition table；按 companion 状态序列实现，不得声称“机器表已消费”（WP0 裁决见 Batch-0A）。
- **sandbox 拦截**：Linux 为参考平台；Windows 开发经 WSL2 或 Linux CI 覆盖负例；**按平台矩阵分别声明（F-017 出口阻断）**，禁止跨平台合并声明。
- C0/C1 adapter（六族接口映射）；带外修改对账（IMP-11，`agent-out-of-band-reconciliation`）；批量 tool proxy 合法形态（IMP-12）。
- readiness case：MANAGEMENT_READY → USER_READY → OPERATIONAL（故障注入验证顺序）。**注意**：无登记 readiness carrier；证据归 milestone e2e/fault，不得虚报 Profile conformance。
- **profile manifest 首次真实声明**（runner 生成，test_runs 挂真实证据 digest；未达项诚实 planned/experimental）。
- **治理开销指标基线（IMP-04/REQ-PERF-004）**：全指标族 + **声明 ungoverned 基线**。
- §20.5 R0 降级映射验证（IMP-06）。

## 禁止越界

不做 R2/R3 审批、distributed、具身、学习、Console 实现；**不做** REQ-PERF-005 agent 收益声明；不新增对象族/Profile/REQ 域；不改写负例。

## 入口 gate

M5 出口评审 **GO M6**（附带条件：D-018/剩余向量持续消化；clients blocked；F-017 阻断出口）。计划文档已批准：见 [M6-PLAN.md](../plan/M6-PLAN.md)。

## 验收判据（摘要；完整矩阵见 M6-PLAN §D）

1. 篡改包安装被拒（`AGENT-INSTALL-001`，`AGENT_PACKAGE_VERIFICATION_FAILED`）。
2. adapter 绕过被拦截，按平台矩阵分别声明（`AGENT-BYPASS-002`，`AGENT_ADAPTER_BYPASS_DETECTED`）。
3. 安装事务中断回滚干净（故障注入；无半安装）。
4. 带外修改被对账发现（`AGENT-OOB-001`）。
5. readiness 顺序验证（管理面先可用；乱序必须 fail）。
6. REQ-PERF-004 全指标族首次报告 + ungoverned 基线（`PERF-REPORT-CONTRACT-001`）。
7. profile manifest 真实声明经 runner 生成并 schema 校验。
8. **v0.1 发布评审** + **F-017** 平台矩阵闭合。

## 分批入口（勿一次做完）

| 批次 | 提示词 | 车道 |
|---|---|---|
| Batch-0A 合同绑定 + 缺口裁决 | [m6-batch0-contracts.md](m6-batch0-contracts.md) | Lane-CTR |
| Batch-0B/1 篡改拒装 tracer | [m6-batch1-installer.md](m6-batch1-installer.md) | Lane-RUN |
| 后续 WP2–WP10 | 按 [M6-PLAN.md](../plan/M6-PLAN.md) §B 另开会话 | 各 owner |

## 工作分支

按批次：`lane/ctr-m6-bindings` → `lane/run`（+ Lane-CFR 平台矩阵证据管道）。

## 第一个动作（若尚未完成 Batch-0A）

读 `docs/plan/M6-PLAN.md` 与 findings-ledger F-017；若 bindings 未合入，先执行 [m6-batch0-contracts.md](m6-batch0-contracts.md)。若 bindings 已合入，再执行 [m6-batch1-installer.md](m6-batch1-installer.md)：先写失败测试（篡改 manifest 拒装）。
