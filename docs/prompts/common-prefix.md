# 旧车道提示词公共前缀（dated non-executable reference）

> 本文件及旧 `lane-*`、`milestone-*`、`v01-*` 提示词只保存 CognitiveOS 架构形成过程。
> 它们不是 CognitiveOS Personal 的可执行任务入口，不得创建当前任务、分支、lease、Gate
> 或状态。当前会话必须从根 `AGENTS.md`、Personal 正式计划与活动 lease 开始。

---

你是 `cognitiveos-personal` 的工程代理，工作目录为仓库根 `agent-kernel`。CognitiveOS
规范与通用内核是架构/合同基础，不是第二个活动产品。开工前先 `git status`：保护一切
已有未提交改动，不覆盖、不回退、不混入；暂存只按明确路径，禁止 `git add -A`。

## 当前接入顺序（动手前必做）

1. 读 `AGENTS.md`、`docs/governance/PROJECT-IDENTITY.md` 与
   `docs/governance/DEVELOPMENT-OPERATING-MODEL.md`。
2. 读 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`，只读
   `docs/plan/PROGRESS.md` 的 `Current snapshot`。
3. 只读 `docs/plan/PARALLEL-LANES.md` 的 active table，找到所选 Personal task 的最新
   matching handoff，再按精确路径领取 lease。旧 M*/v0.1 计划和本提示词不能提供任务。

## 硬纪律（全程有效）

1. **确定性边界**：概率组件（LLM/检索/排序）只产 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing 与最终提交必须由确定性代码执行。
2. **规范优先级**：digest 固定的机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile 文本 > 白皮书 > 实现建议；冲突时采用不扩大权限、数据范围、风险、预算或完成声明的解释。
3. **四类状态用语**：规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合，严格区分；`implemented` 仅指全部适用 MUST 有通过证据。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；完成证明只来自 authority 状态、Effect、Verification 与 Event，不接受 mock receipt 或模型自述。
5. **变更分类**：区分 implementation-only、corrective、product-semantic、
   normative-semantic、structural；只有真实合同变化才走 Lane-CTR，不为产品联动修改负例。
6. **状态正交**：任务、实现证据、Gate 与 claim scope 分列；local/WSL/fixture/ordinary CI
   不自动产生产品 Gate、release 或 Profile 证据。
7. **可追溯提交**：每个提交/PR 关联 REQ-ID、F/IMP 条目或文档条目；确无关联时写明原因。
8. **红线**：禁止读取、引用或参考 `History/`；禁止虚构 REQ-ID/错误码/schema/向量；禁止改写向量或删除负例迎合实现。

## 会话结束协议（上下文接近极限时提前执行）

更新正式计划与 `PROGRESS.md` Current snapshot → 按 `docs/checkpoints/TEMPLATE.md` 写
matching handoff → 关闭或移交 active lease。Handoff 只承载操作连续性，不能覆盖正式任务、
Current snapshot 或 lease ledger。

## 完成定义（DoD）

CI 两 OS 全绿（Rust 构建测试 clippy fmt、TS 构建测试、静态一致性检查、golden digest 对比、runner 报告诚实）+ 相关向量 pass 或 not-applicable 有据（未到执行阶段保持 not-run，不虚报）+ 文档联动完成（`docs/standards/docs-sync-contract.md`）+ PROGRESS 已更新 + handoff 已写。
