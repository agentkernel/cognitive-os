# Lane-RUN 接续提示词：运行时与管理面

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认车道边界后再动手。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合），implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 规范表面冻结：v0.1 前不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正；⑥ P0 门禁：开放 P0 未闭合前对应子系统不得实现；⑦ 每个提交关联 REQ-ID/F/IMP/文档条目；⑧ 红线：禁读 `History/`、禁虚构规范资产、禁改写向量迎合实现。

**会话结束协议**（上下文吃紧时提前执行）：更新 PROGRESS → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 相关向量 pass/not-applicable 有据（未执行保持 not-run）+ 文档联动（docs-sync-contract）+ PROGRESS 更新 + handoff。

---

## 车道范围（M4 后启动；M5–M6 主线）

- **所有权**：`crates/cognitive-runtime`、`crates/cognitive-management`、`crates/cognitive-akp`、`apps/kernel-server`、`apps/admin-cli`；`tests/e2e/`。
- **工作分支**：`lane/run`。
- 任务：
  1. **M5**：Operation 执行器（经 kernel Effect 协议 dispatch）、有界 Harness Loop（进展/停滞判定）、Management API + PrivilegedManagementSession + 确定性 admin CLI（无模型 inspect/stop/revoke/reconcile）、AKP envelope + HTTP JSON + SSE watch（ADR-0003 五条绑定规则）、任务 Shell 服务端语义、R1 聊天内结构化确认（IMP-05 最低集）。
  2. **M6**：AgentPackageManifest 验证、安装事务与回滚、sandbox 拦截（Linux 参考平台，平台矩阵分别声明——F-017）、C0/C1 adapter、带外对账（IMP-11）、readiness 分级（MANAGEMENT_READY→USER_READY→OPERATIONAL）、REQ-PERF-004 治理开销基线。
- Intelligent Management Shell 保持 **experimental**：确定性管理/恢复/停止路径永不依赖模型（REQ-MGMT-FALLBACK-001，向量 `management-deterministic-fallback`）。

## 禁止越界

- 不动 kernel/store/domain 内部（需要新端口找 Lane-KRN 契约协商）；不动 schema/registry（Lane-CTR）；不动 runner/tools（Lane-CFR）。
- 不启动 Console 实现（Lane-CON 台账管辖）；不做 distributed/R2R3/SMS/CRB/具身/学习（v0.1 排除项）。

## 相关规范路径

`docs/standards/akp-envelope-and-http-profile.md`、`task-loop-verification.md`、`event-audit-watch.md`、`authn-authz-capability.md` §5；`specs/akp/README.md`、`specs/agent-compatibility/README.md`、`specs/core/README.md` §12.1；`specs/schemas/privileged-management-session/management-action-proposal/management-approval-decision/agent-package-manifest/agent-installation`；向量组 `management-*`、`shell-*`、`agent-*`、`harness-*`；`docs/adr/0003`；DEVELOPMENT-PLAN M5/M6 验收清单。

## 入口 gate 与验收

入口：M4 出口通过（tracer bullet 证据在案）+ F-011 R1 最低集合同登记完成（M5 前置）。验收 = DEVELOPMENT-PLAN M5（8 条，全含负例）与 M6（7 条）判据。

## 第一个动作

`git checkout -b lane/run`，读 `docs/prompts/milestone-m5.md` 与最近 KRN handoff，确认 kernel 端口冻结面，先为"无模型 inspect/stop/revoke/reconcile"写失败 e2e 测试。
