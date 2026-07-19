# M5 接续提示词：意图链与 Harness/Shell/管理面

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。主车道 Lane-RUN（`lane/run`）+ Lane-TSC（`lane/tsc`）。公共前缀内联自 `docs/prompts/common-prefix.md`。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（含 M4 milestone-review 与 tracer bullet 证据），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界；② 规范优先级（机器资产 > companion/RFC > 白皮书 > 实现建议）；③ 四类状态用语；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 冻结 + 漂移走台账；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构、禁改写向量。

**会话结束协议**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（DEVELOPMENT-PLAN M5 节；标准 `task-loop-verification.md`、`akp-envelope-and-http-profile.md`、`event-audit-watch.md`、`authn-authz-capability.md` §5）

- 意图链：UserIntentRecord → IntentInterpretation（概率候选）→ 确定性准入 → TaskContract；实质歧义澄清（`INTENT_CLARIFICATION_REQUIRED`；R0 可放宽为预览义务，IMP-14）。
- 有界 Harness Loop：硬预算 + 进展/停滞判定（stagnation 停或升级，不空转）。
- Management API + PrivilegedManagementSession + **确定性 admin CLI**（无模型 inspect/stop/revoke/reconcile；Intelligent Management Shell 保持 experimental，不得成为依赖）。
- 任务 Shell 服务端语义 + `apps/agent-shell`/`sdk-ts` 客户端（proposal/preview/attach/detach/cancel/watch）。
- AKP envelope + HTTP JSON + SSE watch（ADR-0003 五条绑定规则；snapshot+cursor；`WATCH_CURSOR_STALE`）。
- **R1 聊天内结构化确认（IMP-05 最低集，闭合 F-011 的 R1 部分）**：高风险动作需结构化确认对象（canonical digest 绑定），无确认不执行。

## 禁止越界

不做安装/sandbox/C0C1（M6）；不做 R2/R3 完整审批矩阵、SMS/CRB、distributed（v0.1 排除）；kernel 内部改动找 Lane-KRN。

## 入口 gate

M4 出口通过（tracer bullet 证据在案）**+ F-011 R1 最低集机器合同已登记**（approval 确认对象 schema 硬化 + 负例向量；此登记本身属 Lane-CTR 修正型/语义型变更，先行完成）。

## 验收判据（8 条全含负例）

1. 实质歧义必须澄清（`shell-target-ambiguity-001`）。
2. 用户修正推进 epoch 并 fence 旧 dispatch（`intent-supersede-002`，`INTENT_VERSION_SUPERSEDED`）。
3. Shell 退出/断连不取消（`shell-detach-attach-004`）。
4. cancel 经 Effect 闭合：`CANCEL_PENDING`→reconcile；晚了 `CANCEL_TOO_LATE`（`shell-cancel-semantics-005`）。
5. 无模型仍可 inspect/stop/revoke/reconcile（`management-deterministic-fallback`：断模型注入下四动词可用）。
6. 通道隔离：跨通道凭据/缓存复用被拒（`shell-channel-isolation-003`，`SHELL_CHANNEL_BINDING_MISMATCH`）。
7. 管理门禁负例组全过（`management-gate-denials`/`management-untrusted-self-authorization`/`management-independent-approval`/`management-session-denials`，全部 dispatches=0）。
8. watch 断线重连 + 陈旧 cursor 强制重新快照（`shell-watch-resume-006`）；R1 确认负例（无结构化确认的高风险动作不执行）。

## 工作分支

`lane/run`（服务端）+ `lane/tsc`（客户端）；合并顺序：kernel 端口冻结 → RUN → TSC 集成。

## 第一个动作

读 M4 handoff 确认 kernel 端口面 → 先写失败 e2e：断模型环境下 admin-cli 四动词可用。
