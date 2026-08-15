---
doc_id: dev.contributing-workflow
locale: zh-CN
kind: guide
audience: [developer]
status: implemented
generated: false
sources:
  - path: docs/governance/DEVELOPMENT-OPERATING-MODEL.md
    symbols: ["TASK-ATOMIC-DELIVERY-01", "CHECKPOINT-DELIVERY-01"]
  - path: docs/standards/docs-sync-contract.md
  - path: .github/workflows/ci.yml
fingerprint: "sha256:d88282228407ba45a33d1a4e081b23bd25e55ee380d5c880ed3e565d7e3f9a8d"
non_claims:
  - 具约束力的工作流由 Operating Model 拥有；本页是面向贡献者的导向性摘要。
---

# 贡献工作流

约束性规则在
[Operating Model](../../../docs/governance/DEVELOPMENT-OPERATING-MODEL.md)；实操形
状如下：

1. **选取**：从
   [`PERSONAL-DEVELOPMENT-PLAN.md`](../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
   选依赖已满足的正式任务。现读
   [`PROGRESS.md`](../../../docs/plan/PROGRESS.md) Current snapshot 与活动 lease
   表。例外：snapshot 的 `Owner-directed campaign` 行处于 active 时暂停任务选取，
   会话改为执行该评测 campaign（Operating Model §2.5）。
2. **整体领取**：一个 task branch、一个 Draft PR、一条精确路径 lease（登记于
   [`PARALLEL-LANES.md`](../../../docs/plan/PARALLEL-LANES.md)）；在计划中登记
   Delivery Slice（`<task>/DNN`）。Slice 是执行检查点，不是独立分支/PR。
3. **垂直交付**：先做最小真实切片（优先真实调用者或持久权威结果，而非堆叠
   helper）；聚焦 failure-first 测试；验证按[开发环境](./development-environments.md)
   路由。
4. **checkpoint**：把 coherent、secret-free 的进展 commit/push 到同一 Draft PR——
   checkpoint 是后台持久化，不是汇报边界或 merge 触发器。每次 commit 与 push 必须先
   通过 docs-sync 门（`node tools/src/docs-sync-gate.mjs --staged|--push`；每克隆运
   行一次 `pnpm run hooks:install` 启用仓库 hooks）。
5. **同一变更集内、commit/push/merge 之前联动文档**：声明变更类别，旧文档走
   [`docs-sync-contract.md`](../../../docs/standards/docs-sync-contract.md) §2，手册走
   [`handbook/_meta/sync-policy.md`](../../_meta/sync-policy.md)（source-map 查询、
   重生成生成页、刷新指纹）。确无文档影响的变更只能以
   `DOCS_IMPACT_NONE="<具体理由>"` 过门，并把理由记入 commit/PR。
6. **确定性收口**：把每条 acceptance 映射到实现 + 负例 + 已执行证据；在精确 merge
   候选 head 上跑 required CI；仅此后才把 PR 从 Draft 转 ready；合并；关闭 lease；
   删除任务分支；本地 `main` fast-forward；`git status` 干净。

绝不：带失败/未完成检查合并、force-push 共享历史、使用 `git add -A`、混入未知工作树
改动、把未执行验证记成 `not-run` 以外的任何东西。
