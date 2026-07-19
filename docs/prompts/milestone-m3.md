# M3 接续提示词：治理链与 Context

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。主车道 Lane-KRN（`lane/krn`）。公共前缀内联自 `docs/prompts/common-prefix.md`。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（含 M2 milestone-review），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器资产 > companion/RFC > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语；④ 测试先行，schema-valid ≠ behavior-pass；⑤ 冻结 + 漂移走台账；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构、禁改写向量。

**会话结束协议**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（DEVELOPMENT-PLAN M3 节；标准 `authn-authz-capability.md`、`context-resolution-and-cache.md`；RFC-0001 §5-6/10-11/15）

- 治理链：TenantContext/Principal/Membership/ActorChain、Conversation/ResourceScope（消费 Lane-CTR 生成对象）。
- 授权：六步判定序（认证→租户→交集→显式 deny→lease→scope/purpose）、capability 交集与单调衰减、撤销 epoch。
- 九阶段确定性 Context Resolution：治理预过滤→检索→逐对象正文重验→排序→预算→loss declaration→确定性渲染→ContextView；两条不可交换次序（检索前过滤、授权先于 ranker/renderer）。
- 缓存键治理绑定（tenant/principal/capability 版本/revocation epoch/purpose/schema digest/encoding profile 七维）。
- 确定性渲染与前缀稳定（IMP-02、REQ-CTX-012）。

## 禁止越界

不做 Intent/Effect dispatch（M4）；不做 HTTP/Shell（M5）；不做记忆准入（M7）；schema 变更找 Lane-CTR。

## 入口 gate

M2 出口通过 + F-007 行为侧测试计划（撤销竞态：解析后-dispatch 前撤销、dispatch 后-commit 前撤销两点）经评审列入本里程碑测试清单。

## 验收判据（全部为安全负例或含负例，行为执行 + 证据）

1. 同租户横向越权拒 + 响应与 not-found 同形（`tenant-lateral-read-denial`）。
2. 管理员无正文授权读取正文被拒（管理权 ≠ 读内容权）。
3. 撤销后缓存复用被拒（`context-revocation-cache-reuse`：epoch 键失配路径验证）。
4. 检索前过滤：ranker 输入集合无未过滤对象（`context-rank-before-auth`）。
5. 跨 Conversation 污染被拒；注入内容不得提升为控制（`prompt-injection-isolation`）。
6. required 超预算 fail-closed（`context-required-over-budget`，`CONTEXT_BUDGET_EXCEEDED`/`CONTEXT_INCOMPLETE`）。
7. 渲染字节稳定 + 前缀稳定断言（`context-render-stability`，REQ-CTX-012）。
8. 出口 milestone-review + matrix/台账（F-007/F-021 行为侧状态推进）。

## 工作分支

`lane/krn`。

## 第一个动作

读 `docs/standards/context-resolution-and-cache.md` 全文与 RFC-0001 §11，先写失败测试：越权同形响应 + ranker 输入集断言。
