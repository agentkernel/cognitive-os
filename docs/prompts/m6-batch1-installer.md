# M6 Batch-1：篡改拒装 tracer（Lane-RUN）

> 用法：粘贴到干净 worktree 的新 Cursor Agent 会话。**前置**：Batch-0A（[m6-batch0-contracts.md](m6-batch0-contracts.md)）已合入 main。本批只做到 VERIFIED/拒绝，不做完整 sandbox/adapter/OOB。

---

你是 CognitiveOS Lane-RUN 工程代理。开工前：`git fetch origin main`；确认 tip 含 M6 bindings；`git status`；逐路径 `git add`，禁 `git add -A`。干净 worktree 建/更新 `lane/run`。

**接入**：`AGENTS.md` → `PROGRESS` → `M6-PLAN.md` WP1 → Batch-0A handoff → `PARALLEL-LANES.md`。只动 `crates/cognitive-runtime`、`crates/cognitive-management`（若需管理入口）、`apps/kernel-server`、`apps/admin-cli`（可选）、本车道测试与 docs。

## 目标

实现最小 deterministic package verifier + installer orchestration：

1. 输入：`AgentPackageManifest`（生成绑定）、artifact bytes、signature/provenance verifier **端口**、adapter/sandbox/compatibility digests、management authority commit token。
2. 验证失败 → `AGENT_PACKAGE_VERIFICATION_FAILED`；authority 不变；capability 不扩张；**零** installation commit。
3. 成功路径最多推进到 companion 的 `VERIFIED`（或等价内部状态）；**不要**在本批实现完整 ADAPTED/TESTED/COMMITTED 或 sandbox。
4. 证明 LLM/workload 输出不能绕过 verifier（确定性门在 runtime）。

## 测试先行（必须先红后绿）

1. artifact digest mismatch → 拒装。
2. invalid signature / provenance → 拒装。
3. adapter/sandbox/compatibility evidence digest mismatch → 拒装。
4. （可选 e2e）kernel-server 管理入口调用同一门；非 authority 客户端不能 commit。

相关：REQ-AGENT-INSTALL-001/002；向量 `AGENT-INSTALL-001`（本批可只做单元/e2e；CFR 行为执行另批）；错误码 `AGENT_PACKAGE_VERIFICATION_FAILED`。

## 禁止

- 不碰 `cognitive-contracts` 生成逻辑（除非发现绑定消费 bug，回交 CTR）。
- 不新增错误码/REQ/schema/transition。
- 不做 F-017 平台矩阵、六族 adapter、OOB、PERF、真实 manifest。
- 不改向量 expected；不虚报 Profile 符合；不执行/改写无关 not-run。
- 不启动 Console/clients 实现。

## 第一个动作

1. 读 `specs/agent-compatibility/README.md` §2、package/installation schema、Batch-0A 裁决。
2. 先写上述失败测试（应失败）。
3. 实现 verifier ports + orchestration 到绿。
4. `cargo test`（相关包）+ consistency；更新 matrix/PROGRESS/handoff。
5. 逐路径 commit；PR 关联 REQ-AGENT-INSTALL-001 与 M6-A1。

## DoD

- 三类篡改路径全拒；零半安装可见（本批范围内）。
- 状态用语 = 实现已提供 + 车道测试已执行；向量仍可 not-run 直至 CFR 批。
- handoff 写明给 CFR 的公开行为面与给 KRN 的事务端口需求（WP2）。
