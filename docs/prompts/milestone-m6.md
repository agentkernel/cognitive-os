# M6 接续提示词：安装与适配、v0.1 发布

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。主车道 Lane-RUN（`lane/run`），Lane-CFR 协作平台矩阵证据。公共前缀内联自 `docs/prompts/common-prefix.md`。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（含 M5 milestone-review），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界；② 规范优先级（机器资产 > companion/RFC > 白皮书 > 实现建议）；③ 四类状态用语，implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass；⑤ 冻结 + 漂移走台账；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构、禁改写向量。

**会话结束协议**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（DEVELOPMENT-PLAN M6 节；`specs/agent-compatibility/README.md`）

- AgentPackageManifest 验证（签名/digest；篡改拒装）。
- 安装事务与回滚（中断后无半安装可见状态）；安装迁移表消费。
- **sandbox 拦截**：Linux 为参考平台；Windows 开发经 WSL2 或 Linux CI 覆盖负例；**按平台矩阵分别声明（F-017 出口阻断）**，禁止跨平台合并声明。
- C0/C1 adapter（六族接口映射）；带外修改对账（IMP-11，`agent-out-of-band-reconciliation`）；批量 tool proxy 合法形态（IMP-12）。
- readiness case：MANAGEMENT_READY → USER_READY → OPERATIONAL 分级（管理面先于用户面，故障注入验证顺序）。
- **profile manifest 首次真实声明**（runner 生成，test_runs 挂真实证据 digest；未达项诚实 planned/experimental）。
- **治理开销指标基线（IMP-04/REQ-PERF-004）**：授权/Context/Effect 各阶段延迟 p50/p95/p99、cache-hit preservation、每受治理调用额外持久化写、审批延迟与橡皮图章率、开销占端到端比例；**声明 ungoverned 基线**。
- §20.5 R0 降级映射验证（IMP-06：薄路径合法性 + 不可降级边界测试）。

## 禁止越界

不做 R2/R3 审批、distributed、具身、学习、Console 实现（各归其里程碑/车道）；**性能收益宣称**：本里程碑只报告开销（REQ-PERF-004），不做任何 agent 收益声明（REQ-PERF-005 四臂对照是 M7+ 的事，此前只可 hypothesis）。

## 入口 gate

M5 出口评审通过。

## 验收判据

1. 篡改包安装被拒（`agent-installation-verification`，`AGENT_PACKAGE_VERIFICATION_FAILED`）。
2. adapter 绕过被拦截，按平台矩阵分别声明（`agent-adapter-bypass`，`AGENT_ADAPTER_BYPASS_DETECTED`）。
3. 安装事务中断回滚干净（故障注入）。
4. 带外修改被对账发现（`agent-out-of-band-reconciliation`）。
5. readiness 顺序验证（管理面先可用；故障注入乱序必须 fail）。
6. REQ-PERF-004 全指标族首次报告 + ungoverned 基线声明（`performance-report-contract` 合同校验）。
7. profile manifest 真实声明经 runner 生成并 schema 校验；`spec-contract-coverage` 等 generic 追溯项状态如实。
8. **v0.1 发布评审**：DEVELOPMENT-PLAN §1 首版定义逐项核对 + F-017/F-011(R1) 闭合核验 + 全里程碑 milestone-review 链完整。

## 工作分支

`lane/run`（+ Lane-CFR 的平台矩阵证据管道）。

## 第一个动作

读 `specs/agent-compatibility/README.md` 与 findings-ledger F-017/IMP-11 条目，先写失败测试：篡改 manifest 拒装 + 安装中断回滚。
