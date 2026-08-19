---
doc_id: ai.source-of-truth
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: docs/governance/PROJECT-IDENTITY.md
  - path: docs/governance/DEVELOPMENT-OPERATING-MODEL.md
    symbols: ["TASK-ATOMIC-DELIVERY-01", "Sources of truth"]
  - path: docs/standards/normative-source-and-versioning.md
fingerprint: "sha256:3c72c48da7f30ce90677fbe42ae1d2581f378ee2f107ed474fac3651d9ed28fc"
non_claims:
  - 本页只做 canonical 来源路由，绝不替代或复述其当前内容。
---

# 事实源优先级

来源冲突时按以下顺序裁决（依据
[`PROJECT-IDENTITY.md`](../../../docs/governance/PROJECT-IDENTITY.md) §4）：

1. [`docs/governance/PROJECT-IDENTITY.md`](../../../docs/governance/PROJECT-IDENTITY.md) —— 仓库身份；`cognitiveos-personal` 是唯一活动项目。
2. [`docs/governance/DEVELOPMENT-OPERATING-MODEL.md`](../../../docs/governance/DEVELOPMENT-OPERATING-MODEL.md) —— 工作流、证据、lease 与收口语义。
3. [`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`](../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md) —— 正式任务、验收、Delivery Slice 与 Gate。
4. [`docs/plan/PROGRESS.md`](../../../docs/plan/PROGRESS.md) 的 `Current snapshot` —— 唯一的当前任务/Slice/Gate/claim 事实。每个会话现读，绝不凭记忆或本手册转述。其 `Owner-directed campaign` 行处于 active 时，continuation 一律路由到该评测 campaign 并暂停开发任务领取（Operating Model §2.5）。
5. [`docs/plan/PARALLEL-LANES.md`](../../../docs/plan/PARALLEL-LANES.md) 活动 lease 表 —— 当前可写路径。
6. [`docs/product/personal/`](../../../docs/product/personal/README.md) 与 [`docs/architecture/personal/`](../../../docs/architecture/personal/README.md) —— 稳定设计意图（绝非当前状态）。
7. [`docs/checkpoints/`](../../../docs/checkpoints/TEMPLATE.md) 下最新匹配 handoff —— 仅提供操作连续性。
8. 根 [`plan.md`](../../../plan.md) —— 研究细节，绝非状态源。

合同语义方面：精确机器资产（`specs/registry/`、`specs/schemas/`、`specs/transitions/`、
`conformance/vectors/`）优先于 normative 伴随文档（`specs/*/README.md`、
[`RFC-0001`](../../../RFC-0001-cognitiveos-governance-context-access.md)、
[`docs/standards/`](../../../docs/standards/normative-source-and-versioning.md)），后者又优先于 informative 白皮书
（[`CognitiveOS-Architecture.md`](../../../CognitiveOS-Architecture.md)）。不可变公理只存在于
[`docs/governance/AXIOMS.md`](../../../docs/governance/AXIOMS.md)。

本手册位于上述全部来源之下：它是关于实现的派生文档。任何手册页面与 canonical 来源冲突
时，以 canonical 来源为准，并在同一交付中修正手册。

绝不读取或引用 `History/`（冻结归档）。绝不把旧提示词（`docs/prompts/`）、带日期的
handoff 或聊天上下文当作当前事实。
