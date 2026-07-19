# Lane-KRN 接续提示词：内核主线（domain → store → kernel）

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认车道边界后再动手。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合），implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 规范表面冻结：v0.1 前不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正；⑥ P0 门禁：开放 P0 未闭合前对应子系统不得实现；⑦ 每个提交关联 REQ-ID/F/IMP/文档条目；⑧ 红线：禁读 `History/`、禁虚构规范资产、禁改写向量迎合实现。

**会话结束协议**（上下文吃紧时提前执行）：更新 PROGRESS → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 相关向量 pass/not-applicable 有据（未执行保持 not-run）+ 文档联动（docs-sync-contract）+ PROGRESS 更新 + handoff。

---

## 车道范围（M2–M4 主线）

- **所有权**：`crates/cognitive-domain`、`crates/cognitive-store`、`crates/cognitive-kernel`；`tests/faults/`（M4）。
- **工作分支**：`lane/krn`。
- 阶段任务（详细验收 = `docs/plan/DEVELOPMENT-PLAN.md` M2/M3/M4 节 + 对应里程碑提示词）：
  - **M2**：五状态机执行器（消费 `specs/transitions/*.json`，集中 transition 入口）、GovernedObject SQLite(WAL) 仓储（ADR-0002 五条绑定规则）、CAS、append-only 事件日志 + outbox、预算计量、状态+事件原子提交。
  - **M3**：治理链（Principal/Membership/ActorChain/Conversation/ResourceScope）、capability 交集/单调衰减/撤销、九阶段 Context Resolution、缓存键治理绑定、确定性渲染（标准：`authn-authz-capability.md`、`context-resolution-and-cache.md`）。
  - **M4**：Intent/Effect/幂等/reconcile/checkpoint/恢复八步 + 故障注入框架 + 全 sink fencing 清单（F-014）+ OperationDescriptor 准入矩阵（F-023）+ tracer bullet 竖切（标准：`intent-effect-idempotency.md`）。

## 依赖方向与代码纪律（`.cursor/rules/10-rust-kernel.mdc` 全文有效）

contracts → domain → 端口 trait（kernel 定义）→ 适配器（store）→ 应用。`domain`/`kernel` 禁依赖 HTTP、SQLite 具体类型、模型 SDK；库代码禁 panic/unwrap（workspace lints 已 deny）；ID 用生成的 newtype；状态迁移仅经集中表驱动入口，非法迁移返回 registry 错误码且状态不变。

## 禁止越界

- 不动 `specs/**`（契约变更找 Lane-CTR）；不动 runner/tools（Lane-CFR）；不动 runtime/management/akp/apps（Lane-RUN）。
- 入口 gate 未开不越级：M2 需 M1 出口（生成合同冻结）；M4 tracer bullet 需 F-002~F-010 类全闭合（findings-ledger 为准）。

## 相关规范路径

`specs/transitions/`（5 表）、`specs/registry/state-domains.yaml`、`docs/standards/state-and-transition-contract.md`、`intent-effect-idempotency.md`、`authn-authz-capability.md`、`context-resolution-and-cache.md`、`event-audit-watch.md`、`docs/adr/0002、0005`；向量组：`state-conflict`、`eff-crash-001..003`、`effect-*`、`context-*`、`tenant-lateral-read-denial`、`capability-attenuation`、`crash-recovery`、`state-store-degradation`。

## 入口 gate 与验收

入口：PROGRESS 显示 M1 出口通过。验收以 DEVELOPMENT-PLAN 对应里程碑判据为准（每条含安全负例）；里程碑结束写 milestone-review。

## 第一个动作

`git checkout -b lane/krn`，读 `docs/prompts/milestone-m2.md` 领取 M2 范围，先为"并发 CAS 仅一个成功"与"非法迁移全拒"写失败测试。
