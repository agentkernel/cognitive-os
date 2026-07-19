# Lane-TSC 接续提示词：TypeScript 客户端

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认车道边界后再动手。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合），implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 规范表面冻结：v0.1 前不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正；⑥ P0 门禁：开放 P0 未闭合前对应子系统不得实现；⑦ 每个提交关联 REQ-ID/F/IMP/文档条目；⑧ 红线：禁读 `History/`、禁虚构规范资产、禁改写向量迎合实现。

**会话结束协议**（上下文吃紧时提前执行）：更新 PROGRESS → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 相关向量 pass/not-applicable 有据（未执行保持 not-run）+ 文档联动（docs-sync-contract）+ PROGRESS 更新 + handoff。

---

## 车道范围

- **所有权**：`packages/sdk-ts`、`apps/agent-shell`（admin-cli 是 Rust，归 Lane-RUN；本车道可为其贡献交互设计评审）。
- **工作分支**：`lane/tsc`。
- 任务（golden 对齐后与 KRN 并行，M5 集成）：
  1. sdk-ts：AKP envelope 客户端（消费 Lane-CTR 生成的 TS 合同）、任务/管理双通道客户端骨架（凭据/缓存隔离，一实例一通道）、snapshot+cursor watch 消费（`WATCH_CURSOR_STALE` → 重新快照）、重试策略遵循 `docs/standards/error-contract.md` §3（retryable 语义）。
  2. agent-shell：proposal/preview/attach/detach/cancel/watch 交互（`docs/standards/task-loop-verification.md` §6：detach 不取消、cancel 经 Effect 闭合、状态一律 authority 投影）。
  3. 与 Lane-RUN 的 M5 集成测试（真实 HTTP+SSE 对接 kernel-server）。

## 代码纪律（`.cursor/rules/11-typescript-clients.mdc` 全文有效）

TS 只消费生成合同（生成物禁手改，ADR-0006）；canonical/digest 只用 `@cognitiveos/contracts-ts`（禁止 `JSON.stringify` 直接哈希）；客户端不做授权判定/完成判定/状态聚合冒充权威；tsconfig strict 全开沿用 `packages/contracts-ts/tsconfig.json` 基线。

## 禁止越界

- 不动 Rust crate；不动 schema/registry（找 Lane-CTR）；不实现服务端语义。
- 不在客户端缓存跨通道复用 token/缓存键；不实现"客户端聚合完成判定"。

## 相关规范路径

`docs/standards/akp-envelope-and-http-profile.md`、`event-audit-watch.md`、`task-loop-verification.md`、`error-contract.md`；`specs/schemas/shell-*.json`、`watch-subscription.schema.json`、`user-intent-record.schema.json`；向量组 `shell-*`、`intent-*`、`remote-completed-not-acceptance`；`docs/adr/0001、0003、0006`。

## 入口 gate 与验收

入口：Lane-CTR 的 TS 生成合同首批冻结（PROGRESS 通告）。验收并入 M5 出口：通道隔离负例、detach/cancel/watch 语义向量行为侧通过、无模型管理路径不受影响。

## 第一个动作

`git checkout -b lane/tsc`，读 `packages/sdk-ts/src/index.ts` 与 `docs/standards/akp-envelope-and-http-profile.md`，起草 sdk-ts 客户端接口（channel 绑定、envelope 封装、watch 迭代器）为失败测试。
