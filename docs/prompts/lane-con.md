# Lane-CON 接续提示词：Console 产品车道（占位）

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认车道边界后再动手。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合），implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass；⑤ 规范表面冻结：v0.1 前不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md`；⑥ P0 门禁；⑦ 每个提交关联条目；⑧ 红线：禁读 `History/`、禁虚构规范资产、禁改写向量。

**会话结束协议**（上下文吃紧时提前执行）：更新 PROGRESS → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff → 分批提交。**DoD**：CI 全绿 + 文档联动 + PROGRESS 更新 + handoff。

---

## 车道范围（当前 = 仅依赖台账；后端 gate 通过后激活）

- **所有权**：`apps/cognitiveos-console/`（README 与 PRODUCT-DESIGN 的漂移标注节）；DEVELOPMENT-PLAN §2 Console 依赖表。
- **当前任务（激活前）**：
  1. 维护 DEVELOPMENT-PLAN §2 Console 依赖表：九组依赖（源自 PRODUCT-DESIGN §20.3）随 M2/M4/M5/M6/M7 交付逐项打勾并链接证据。
  2. 后端契约语义型变更影响 Console 设计时，在 PRODUCT-DESIGN 文首"漂移登记"节**追加标注行**（日期/变更/受影响章节），不改写正文（docs-sync-contract §2.8）。
  3. 复核 §17.1 MVP 排除项是否仍成立（每个里程碑出口评审后）。
- **激活条件**（全部满足才可规划实现里程碑）：依赖组 1/2/7 交付 + M5 出口评审通过 + PoC（PRODUCT-DESIGN §12.6 POC-01~12）用**真实 API**（禁 mock 冒充）出具可复现实测报告。
- 激活后第一里程碑建议：MVP Desktop 只读监督（Task/Execution 五轨 + watch 投影），继续遵循"客户端非 authority"（`.cursor/rules/11-typescript-clients.mdc`）。

## 禁止越界

- **激活前禁止任何 Console 实现代码**（组件、脚手架、mock server 均不允许入库）。
- 不改写 PRODUCT-DESIGN 正文（planned 产品设计由产品评审流程管理）；不宣称任何 Console 能力"已实现 CognitiveOS 管理能力"。
- 不代替后端车道登记契约（缺口发现 → findings-ledger + 通告对应车道）。

## 相关规范路径

`apps/cognitiveos-console/PRODUCT-DESIGN.md` §17（MVP 与路线图）、§17.1（排除项）、§20.3（后端依赖结论）、§12.6（PoC 清单）；`docs/plan/DEVELOPMENT-PLAN.md` §2 Console 节；`.cursor/rules/11-typescript-clients.mdc`。

## 第一个动作

读 DEVELOPMENT-PLAN §2 Console 依赖表与 PROGRESS 里程碑状态，核对是否有已交付依赖未打勾；无则本车道无事可做，如实记录后结束（不要制造工作）。
