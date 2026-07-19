# 接续提示词公共前缀（唯一维护点）

> 本文件是全部 `docs/prompts/lane-*.md` 与 `milestone-*.md` 内联"公共前缀"节的源头。
> 修改本文件时必须同步全部接续提示词的内联副本（它们必须自包含、可直接粘贴到新 Cursor 窗口）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`（命令速查、目录地图、DoD、红线）。
2. 读 `docs/plan/PROGRESS.md`（当前里程碑/车道状态与开放 P0）。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，并对照 `docs/plan/PARALLEL-LANES.md` 确认本车道边界与所有权后再动手。

## 硬纪律（全程有效）

1. **确定性边界**：概率组件（LLM/检索/排序）只产 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing 与最终提交必须由确定性代码执行。
2. **规范优先级**：digest 固定的机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile 文本 > 白皮书 > 实现建议；冲突时采用不扩大权限、数据范围、风险、预算或完成声明的解释。
3. **四类状态用语**：规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合，严格区分；`implemented` 仅指全部适用 MUST 有通过证据。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；完成证明只来自 authority 状态、Effect、Verification 与 Event，不接受 mock receipt 或模型自述。
5. **规范表面冻结**：v0.1 前不新增对象族、Profile、REQ 域；只允许实现反馈驱动的修正型规范变更；发现漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正闭合，提交说明注明。
6. **P0 门禁**：findings-ledger 中开放的 P0 未闭合前，对应子系统不得进入实现。
7. **可追溯提交**：每个提交/PR 关联 REQ-ID、F/IMP 条目或文档条目；确无关联时写明原因。
8. **红线**：禁止读取、引用或参考 `History/`；禁止虚构 REQ-ID/错误码/schema/向量；禁止改写向量或删除负例迎合实现。

## 会话结束协议（上下文接近极限时提前执行）

更新 `docs/plan/PROGRESS.md` → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff（已完成/未完成 REQ、提交哈希、测试与证据状态、未决风险与漂移、下一步入口与建议提示词路径）→ 分批提交。交接文档是跨会话唯一记忆载体，禁止依赖对话历史承载工程状态。

## 完成定义（DoD）

CI 两 OS 全绿（Rust 构建测试 clippy fmt、TS 构建测试、静态一致性检查、golden digest 对比、runner 报告诚实）+ 相关向量 pass 或 not-applicable 有据（未到执行阶段保持 not-run，不虚报）+ 文档联动完成（`docs/standards/docs-sync-contract.md`）+ PROGRESS 已更新 + handoff 已写。
