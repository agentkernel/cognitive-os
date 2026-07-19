# Lane-CFR 接续提示词：符合性与工具

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认车道边界后再动手。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合），implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 规范表面冻结：v0.1 前不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正；⑥ P0 门禁：开放 P0 未闭合前对应子系统不得实现；⑦ 每个提交关联 REQ-ID/F/IMP/文档条目；⑧ 红线：禁读 `History/`、禁虚构规范资产、禁改写向量迎合实现。

**会话结束协议**（上下文吃紧时提前执行）：更新 PROGRESS → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 相关向量 pass/not-applicable 有据（未执行保持 not-run）+ 文档联动（docs-sync-contract）+ PROGRESS 更新 + handoff。

---

## 车道范围（M1 起持续）

- **所有权**：`crates/cognitive-conformance`、`tools/`、`.github/workflows/`；向量增补（负例向量新增可经本车道，走 docs-sync-contract）。
- **工作分支**：`lane/cfr`。
- M1 任务（按序）：
  1. **runner 执行能力**：分层执行 74 向量（先从 layer 1 wire-schema、layer 2 state-machine 的可静态判定子集起步），比对 `input`→`expected`，输出五态（pass/fail/not-applicable/documented-degradation/not-run，口径 `docs/standards/conformance-evidence.md` §2）；机器 JSON 报告 + 人读摘要保持现有格式演进。
  2. **runner 自检**：加入一个**故意错误实现**（schema-valid、行为错误），证明 runner 将其判 fail——"仅 schema-valid 不能 pass"（DEVELOPMENT-PLAN M1 验收 2）。
  3. 未实现层诚实保持 not-run；CI 断言演进（`.github/workflows/ci.yml` 的 not-run 断言随执行能力逐步放开，但 pass 必须有真实执行支撑）。
  4. **D-004 处置**：与 Lane-CTR 商定 layer 7/8 slug 方案（修正型变更），或维持跨切片映射并文档化。
  5. tools 演进：把 M1 迁移后的 `$id` 策略回写 `check-consistency.mjs`（移除剥离兼容层）；注入演练重跑（docs-sync-contract §5）。
- M2 起：为各里程碑行为验收提供执行与证据管道（fault 注入框架协作在 M4 归 KRN，证据格式归本车道）。

## 禁止越界

- 不改业务 crate；不实现内核行为。**绝对禁止**：为让向量通过而改写向量/删除负例/放宽 expected（发现向量真错 → findings-ledger 登记 + Lane-CTR 契约流程）。
- 不把 not-run/planned 呈现为通过；样例 manifest 之外不生成任何"声明性"manifest。

## 相关规范路径

`conformance/README.md`（15 层、状态语言、Running）；`docs/standards/conformance-evidence.md`、`canonical-encoding-and-digest.md` §14；`specs/schemas/profile-manifest.schema.json`；`.cursor/rules/15-conformance-evidence.mdc`；`docs/traceability/findings-ledger.md`（F-001/F-015/F-016、D-002/D-004）。

## 入口 gate 与验收

- 入口：M0 出口通过；与 Lane-CTR 并行（依赖其 schema 迁移的部分排后）。
- 验收（并入 M1 出口评审）：全部向量五态输出；错误实现自检 fail；未实现层 not-run 无虚报；D-004 处置完成；CI 绿。

## 第一个动作

`git checkout -b lane/cfr`，读 `crates/cognitive-conformance/src/lib.rs` 与 `conformance/README.md` Running 节，列出 74 向量中可静态判定（wire-schema/contract-traceability 类）与需实现行为（effect-recovery 等）的执行分批清单，写入 PROGRESS 车道栏。
