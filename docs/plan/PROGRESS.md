# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-DOC/Lane-CON Agent Hub 安全直连接管提示词；隔离 `personal-blog` CognitiveOS Research 重构）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | 以该 milestone review 为稳定证据入口 |
| M1 合同收敛与 Runner | **in-progress** | — | Lane-CTR 契约批已交付（F-003 收尾、$id 统一、codegen、注册式 bundle digest、golden §14 全覆盖）；Runner 执行能力待 Lane-CFR |
| M2 对象/状态/事件内核 | not-started | — | 依赖 M1 |
| M3 治理链与 Context | not-started | — | 依赖 M2 |
| M4 Intent/Effect 与恢复 + tracer bullet | not-started | — | 入口 gate：F-002~F-010 类全闭合（现余 F-003→M1） |
| M5 意图链/Harness/Shell/管理面 | not-started | — | 入口另需 F-011 R1 合同登记 |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | macOS/Linux/iPhone/Android phone 产品切片已记录；Agent Hub 两模式提示词已加入 Paseo 类接管、独立文档治理、分计划/进度表和 gate 后多代理开发协议；客户端 implementation 均未启动，平台测试未执行，Profile 未符合 |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行** | Next.js 38 静态/SSG 页面；Vitest 14/14；Playwright Chromium 22/22；全模板 axe WCAG 2.0/2.1/2.2 A/AA 通过 | 仅研究发布与展示层；不改变 REQ 实现、向量执行或 Profile 符合状态 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema 56；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | 0（合同层已有 canonical/bundle/projection/生成绑定实现，未作 REQ 级实现声明——待 runner 证据与 matrix impl 字段回填） |
| 测试已执行（runner 真实执行并留证据） | 0 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner）

| 状态 | 计数 |
|---|---|
| 向量总数 | **76**（M1 Lane-CTR 增补 2 份 F-003 双轨拒绝负例：GOBJ-LEGACY-METADATA-001 / GOBJ-LEGACY-STRONGREF-001） |
| pass / fail / not-applicable / documented-degradation | 0 / 0 / 0 / 0 |
| **not-run** | **76**（runner 仍为枚举骨架；执行能力待 Lane-CFR；合同层 schema 校验测试已在双语言证明两份负例被拒，但不计为向量执行） |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`）。层 7/8 无专属 slug（漂移 D-004，M1 处置）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 1（+1 证据性质） | **F-003**（合同层复验已完成：负例向量 + 双语言 schema 校验测试 + codegen 对齐 + legacy `$defs` 保留决策；唯一剩余 gate = Lane-CFR runner 真实执行负例向量）；F-001（证据缺口，随 M1~M6 消解） |
| P1 | 4 | F-011（M5）、F-014（M4）、F-023（M4）、F-017（M6）；另 F-015 持续收敛 |
| 漂移 | 1 开放 | D-004（排 M1，Lane-CFR）；D-001/D-006/D-011 已闭合（M1 Lane-CTR）；D-002/D-003/D-005/D-007~D-010 已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **M1 契约批已交付**（merge `b626e88`） | `lane/ctr` | 已完成 F-003 收尾 / D-001·D-006 `$id` 统一 / ADR-0006 codegen / §13 bundle digest / golden §14；**触碰通告（CFR 合并前须 rebase）**：`tools/src/check-consistency.mjs`（移除剥离 `$id` 兼容层 + 新增 `$id`==文件名红灯）、`tools/static_check.py`（向量计数 76、删除绝对 URL 别名注册）、`.github/workflows/ci.yml`（新增 codegen regenerate-and-diff 步骤）、`crates/cognitive-conformance/src/lib.rs`（provisional_digests → registered_digests，非 runner 执行逻辑） |
| Lane-CFR 符合性与工具 | 待启动（可与 CTR 并行） | `lane/cfr` | M1：runner 执行能力（`docs/prompts/lane-cfr.md`）；开工前先 rebase 上述 CTR 触碰点 |
| Lane-KRN 内核主线 | 阻塞于 M1 | `lane/krn` | — |
| Lane-TSC TS 客户端 | 阻塞于 CTR golden 对齐 | `lane/tsc` | — |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | iPhone/Android phone 独立产品设计已记录；Agent Hub 安全直连接管/完整治理及 gate 后多代理开发提示词已提供、产品决策尚未执行；实现 gate 未通过 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-personal-blog-research-refactor-handoff.md](../checkpoints/20260720-personal-blog-research-refactor-handoff.md)（CognitiveOS Research 品牌、信息架构、发布合同、来源账本、视觉/a11y/SEO 与验证）
2. [20260720-lane-con-agent-takeover-prompt-handoff.md](../checkpoints/20260720-lane-con-agent-takeover-prompt-handoff.md)（Agent Hub 两模式、Paseo 参考与安全进程/session/文件接管提示词）
3. [20260720-lane-con-agent-hub-prompt-handoff.md](../checkpoints/20260720-lane-con-agent-hub-prompt-handoff.md)（Agent Hub 初版产品设计、第三方 Agent 适配与开发任务编排提示词）
