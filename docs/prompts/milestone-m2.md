# M2 接续提示词：对象/状态/事件内核

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。主车道 Lane-KRN（`lane/krn` 分支）。公共前缀内联自 `docs/prompts/common-prefix.md`。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（含 M1 milestone-review），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议；③ 四类状态用语；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 冻结：不新增对象族/Profile/REQ 域，漂移先登记台账再最小修正；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构规范资产、禁改写向量。

**会话结束协议**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（DEVELOPMENT-PLAN M2 节为准；标准 `docs/standards/state-and-transition-contract.md`、ADR-0002/0005）

- GovernedObject SQLite(WAL) 仓储（`crates/cognitive-store`；rusqlite 依赖此时引入；ADR-0002 五条绑定规则：一事务一提交、WAL+synchronous=FULL、CAS=WHERE version、失败 `STATE_STORE_UNAVAILABLE` fail-closed、单写者）。
- 五状态机执行器（`crates/cognitive-domain` 表驱动 + `cognitive-kernel` 集中 transition 入口，消费 `specs/transitions/*.json` 五表；非法迁移返回 registry 错误码且状态不变）。
- CAS、append-only 事件日志 + outbox、预算计量原语、状态+事件原子提交；ID=UUIDv7 newtype、三时钟域分型（ADR-0005）。

## 依赖方向红线（`.cursor/rules/10-rust-kernel.mdc`）

`domain`/`kernel` 禁依赖 HTTP、SQLite 具体类型、模型 SDK；SQLite 只出现在 `cognitive-store`；库代码禁 panic/unwrap；迁移表单一来源 `specs/transitions/`，禁止复制常量。

## 禁止越界

不动 schema/registry/vectors（Lane-CTR）；不动 runner/tools（Lane-CFR）；不做 Context/capability（M3）、Intent/Effect 行为（M4）、HTTP（M5）。

## 入口 gate

M1 出口评审通过（生成合同冻结 + runner 可执行 + F-003 关闭）。PROGRESS 确认后才动工。

## 验收判据（全部需可执行测试 + 证据）

1. 并发 CAS：N 并发写同对象仅 1 成功，其余 `STATE_CONFLICT` 无副作用（向量 `state-conflict` 行为侧）。
2. 非法迁移全拒：对五表逐一穷举非法 from→to，状态不变、错误码与 registry 一致。
3. 投影重放 digest 稳定：事件历史重放两次 → canonical digest 相同（`event-audit-watch.md` §3）。
4. 事件不可原地修改（负例：任何 UPDATE 已提交事件被拒）。
5. 提交路径故障注入：只读库/磁盘满 → `STATE_STORE_UNAVAILABLE` fail-closed、无内存缓冲（`state-store-degradation` 行为侧，REQ-REC-003）。
6. 出口产出 milestone-review；PROGRESS/matrix（impl 字段）/台账更新。

## 工作分支

`lane/krn`。

## 第一个动作

读 `specs/transitions/effect.transitions.json` 与 `state-and-transition-contract.md`，先写失败测试：并发 CAS 单胜者 + task 表一条非法迁移拒绝。
