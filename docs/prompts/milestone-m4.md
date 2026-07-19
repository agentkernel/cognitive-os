# M4 接续提示词：Intent/Effect 与恢复 + tracer bullet

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。主车道 Lane-KRN（`lane/krn`），Lane-CFR 协作故障注入证据管道。公共前缀内联自 `docs/prompts/common-prefix.md`。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（含 M3 milestone-review），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界；② 规范优先级（机器资产 > companion/RFC > 白皮书 > 实现建议，冲突不扩大权限/范围/风险/预算/完成声明）；③ 四类状态用语；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 冻结 + 漂移走台账；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构、禁改写向量。

**会话结束协议**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（DEVELOPMENT-PLAN M4 节；标准 `intent-effect-idempotency.md`；白皮书 §16.6）

- Intent 持久化（幂等键 + 参数 digest + expected_state_version + 授权绑定，一事务）。
- OperationDescriptor/绑定 + **准入拒绝矩阵（F-023）**：执行器按查询性/幂等性能力自描述，不可查询且不可幂等的组合走拒绝路径。
- Effect 状态机（`effect.transitions.json` v0.2）、幂等记录、reconcile/verify、checkpoint（恢复稳定事实）。
- **恢复八步（§16.6 顺序固定）**：fence 新 epoch → 重放已提交历史 → 对账在途 Effect → 重授权 → 重解析 Context → 恢复 Loop（+ 前置的存储完好性确认与后置的 readiness 上报，按白皮书全序执行）。
- 故障注入框架（进程级 kill + 存储故障模拟）；**全 sink fencing 清单（F-014）**：枚举每个外部提交端并逐端负例。
- 七性质形式化模型（IMP-07：幂等、fencing、无双执行、unknown 分流、恢复顺序、补偿独立授权、fail-before-effect）。
- **末尾 tracer bullet 端到端竖切**：一条真实任务 UserIntent→interpretation→TaskContract→Context→Intent→Effect→Verification→acceptance 在单节点全链跑通留证据。

## 禁止越界

不做 HTTP/Shell/管理面（M5）；不做安装/sandbox（M6）；schema/向量变更走 Lane-CTR/CFR。

## 入口 gate（硬条件，不得绕过）

**F-002~F-010 类 P0 全闭合**：findings-ledger 确认 F-003 closed（M1）+ F-004/005/006/007/008/010 复验通过（M1）+ F-007/F-008 行为侧（M2/M3 验收）在案。另 F-014/F-023 工作项已在本里程碑排期。

## 验收判据

1. 三个 crash point 行为覆盖（`eff-crash-001..003`）：崩溃-恢复后同键单次 dispatch、无重复 effect、审计链闭合。
2. unknown outcome 不成功不换键（`effect-unknown-outcome`）；quarantine 路径可达。
3. 同键异参 `EFFECT_IDEMPOTENCY_CONFLICT`（`effect-idempotency-conflict`）。
4. receipt/远端 completed 不完成 Task（`remote-completed-not-acceptance`）。
5. 恢复顺序错乱注入被捕获（如先恢复 Loop 后 fence 必须 fail）。
6. sink fencing 矩阵：每个提交端旧 epoch dispatch 被拒（逐端负例）。
7. tracer bullet 证据在案（`artifacts/evidence/` + milestone-review 引用）。
8. 出口 milestone-review；F-010/F-014/F-023 台账状态推进。

## 工作分支

`lane/krn`（故障注入证据管道与 Lane-CFR 协作，合并顺序 KRN 先）。

## 第一个动作

读 `docs/standards/intent-effect-idempotency.md` 与白皮书 §16.6，先写失败测试：crash point 1（Intent 已持久化未 dispatch，kill 后恢复必须复用原幂等键单次 dispatch）。
