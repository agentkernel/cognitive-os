# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（M0 收尾会话）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | 本页所在提交 |
| M1 合同收敛与 Runner | not-started | — | 入口 gate 已满足（M0 出口）；用 `docs/prompts/milestone-m1.md` 启动 |
| M2 对象/状态/事件内核 | not-started | — | 依赖 M1 |
| M3 治理链与 Context | not-started | — | 依赖 M2 |
| M4 Intent/Effect 与恢复 + tracer bullet | not-started | — | 入口 gate：F-002~F-010 类全闭合（现余 F-003→M1） |
| M5 意图链/Harness/Shell/管理面 | not-started | — | 入口另需 F-011 R1 合同登记 |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | tracking-only | — | 依赖台账见 DEVELOPMENT-PLAN §2；九组依赖全未交付 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema 56；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | 0（M0 仅骨架 + canonical 编码层） |
| 测试已执行（runner 真实执行并留证据） | 0 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner）

| 状态 | 计数 |
|---|---|
| 向量总数 | 74 |
| pass / fail / not-applicable / documented-degradation | 0 / 0 / 0 / 0 |
| **not-run** | **74**（M0 骨架仅枚举；执行能力 M1 交付） |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`）。层 7/8 无专属 slug（漂移 D-004，M1 处置）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 1（+1 证据性质） | **F-003**（阻断 M1 出口）；F-001（证据缺口，随 M1~M6 消解） |
| P1 | 4 | F-011（M5）、F-014（M4）、F-023（M4）、F-017（M6）；另 F-015 持续收敛 |
| 漂移 | 3 开放 | D-001/D-004/D-006（均排 M1）；D-002/D-003/D-005 已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | 待启动（**下一个**） | `lane/ctr` | M1：F-003 迁移 + codegen（`docs/prompts/lane-ctr.md`） |
| Lane-CFR 符合性与工具 | 待启动（可与 CTR 并行） | `lane/cfr` | M1：runner 执行能力（`docs/prompts/lane-cfr.md`） |
| Lane-KRN 内核主线 | 阻塞于 M1 | `lane/krn` | — |
| Lane-TSC TS 客户端 | 阻塞于 CTR golden 对齐 | `lane/tsc` | — |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | 仅台账 | — | 维护依赖表 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md)（M0 出口评审 + 交接）
2. —
3. —
