# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-23（经批准的 Lane-CFR + Lane-CTR 原子例外已为 Ordinary Core `status.inspect` 登记 `ORDINARY-CORE-AUDIT-INSPECT-001`，同时映射 `REQ-AUDIT-001` / `REQ-AUDIT-002` 并刷新 matrix。runner 最小行为实现已提供，向量测试已执行：公开 `inspect_with_audit` + durable file audit + 正式 decision/receipt + release gate，真实结果 **85 vectors / 60 pass / 25 not-run / 0 fail**，self-check **41/41**；candidate/schema/generated/golden 与 Lane-RUN 生产路径未改。此向量证据不构成 machine-registration 总体完成、CA-0 GO、High-Assurance、Profile 已符合或 D-022 解除；D-022 仍 blocking，Profile **implemented = 0**。）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | **done** | [20260721-m5-milestone-review.md](../checkpoints/20260721-m5-milestone-review.md) | KRN+CTR+RUN 1–2b+TSC+CFR 已合入。行为向量当时 **52 pass / 32 not-run**；F-011 三负例行为闭合；D-018 仍 partially-implemented。**GO M6**（附带条件见评审 §7） |
| M6 安装与适配、v0.1 发布 | **实现已提供；测试已执行（局部）；出口 GO-with-explicit-non-claim** | [20260721-v01-rereview.md](../checkpoints/20260721-v01-rereview.md)（初评 [NO-GO](../checkpoints/20260721-m6-milestone-review.md)） | RUN/CFR M6 交付 + EXIT 声明集/F-017 digests；pins **55/29**；RC ≤ experimental；**implemented = 0**；durable install / PERF 战役 / D-018 / Win-native / WSL2 = explicit non-claim；计划：[M6-EXIT-PLAN.md](M6-EXIT-PLAN.md) |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | 客户端项目根迁移完成（ADR-0007）；Phase 0 文档收口；M5 出口已 GO，但 implementation-ready 仍 **no (blocked)**：缺五平台 PoC / 技术栈 ADR / 依赖组 1/2/7 完整交付与法务 gate；与 M6 核心可并行 tracking-only，不混入主线 PR；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行**（嵌套独立仓；**不入** Cos `origin/main`） | Next.js 静态/SSG；Vitest / Playwright / axe 证据以 **blog 仓** 为准 | 仅研究发布与展示层；不改变 REQ/向量/Profile。**唯一路径** `personal-blog/`；远程 [`agentkernel/blog`](https://github.com/agentkernel/blog)；纪律见 `.cursor/rules/19-personal-blog-boundary.mdc` |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **63**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **70**（matrix 实测非空 impl；shell channel + target resolution 两批各回填 2 条后的当前值） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 33 向量**（既有 32 + **ORDINARY-CORE-AUDIT-INSPECT-001**）+ workspace Rust 项 + tracer bullet；静态执行 27 向量；**均不构成 Profile 覆盖声明**；TS **85** 项（sdk-ts 72 / agent-shell 13） |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`；RC manifest ≤ `experimental`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-23 Ordinary Core AUDIT 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **85** |
| **pass** | **60** = 静态 27 + **行为 33**（既有 32 + **ORDINARY-CORE-AUDIT-INSPECT-001**） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run** | **25**（含 MGMT-FALLBACK 其余未执行范围、shell migration、delta-scope、store-degradation disk-full 等） |
| 错误实现自检 | **41/41 corrupted 向量全部翻 fail**（新增 audit-before-release / receipt mismatch anti-pattern）；CI 地板 ≥41 |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`；报告 sha256 由 runner 打印）。层 7/8 无专属 slug = D-004 已按文档化跨切片映射闭合（conformance/README + runner `CROSS_SLICE_HOSTED`）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 0（+1 证据性质） | F-001（证据缺口，随里程碑消解，不阻断） |
| P1 | **0**（+持续） | F-017 **closed-for-release-claim-set**；F-015 持续。**F-011 已于 CFR M5 行为批闭合**；F-014/F-023 已于 M4 闭合 |
| 漂移 | **0 open**（+3 deferred/design-materialized，+1 decided/partial） | **D-022 v0.2 design/registration blocker**（AUDIT owner-authorized security/audit/compliance review 分量完成但 provenance 受限；SIG independent review、四类 machine registration、OPS member closure 与 CA-0 GO pending；继续阻断 CA-1～CA-8）；**D-017 deferred-to-v0.2**；**D-018 partially-implemented**（组装器 + watch/shell 行为证据已有；治理对象端口仍缺）；**D-016 registration eligibility NO-GO**（八项 blocked；machine contracts 未登记）；D-019 已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **Ordinary Core AUDIT vector mapping registered in joint batch** | `lane/cfr-ctr-ordinary-core-audit-inspect` | `REQ-AUDIT-001` / `002` both map to `ORDINARY-CORE-AUDIT-INSPECT-001`; matrix is fresh; no schema/candidate semantics changed |
| Lane-CFR 符合性与工具 | **Ordinary Core AUDIT vector test executed** | `lane/cfr-ctr-ordinary-core-audit-inspect` | `ORDINARY-CORE-AUDIT-INSPECT-001` pass via audited public consumer; pins **60/25**; self-check **41/41**; non-Profile claim |
| Lane-KRN 内核主线 | **D-018 durable governance-header resolution port implemented; focused test executed** | `lane/krn-d018-governance-ports` | Append-only M5 governed records now resolve their canonical `GovernedObjectHeader` by identity, fail closed on malformed/ambiguous values, and never synthesize missing governance facts. D-018 remains partially-implemented pending Lane-RUN consumption and CFR watch/shell behavior evidence; InstallationStore remains a durable non-claim. |
| Lane-TSC TS 客户端 | **M5 HTTP/SSE 已交付**（PR #28） | `lane/tsc` | proposal/preview/submit 完整 HTTP 面增量（计划标 P2）；channel isolation 已由 RUN+CFR 补 authority 证据 |
| Lane-RUN 运行时与管理面 | **single-node loopback serve foundation implemented** | `lane/run-single-node-serve` | `kernel-server --serve --bind` accepts multiple loopback requests; non-loopback bind is rejected until authenticated deployment middleware exists. This is process-liveness only, not a deployment/Profile claim. |
| Lane-DOC 文档维护 | **ADR-0015 complexity boundary accepted** | `lane/doc-product-complexity-boundary` | Ordinary Core remains the default product range; strict independent AUDIT/SIG/TARGET work is High-Assurance deferred/tracking. This changes priority only, never factual D-016/D-022 or Profile gates. |
| Lane-CON Console | tracking-only | — | M5 GO 后可复评 gate；仍缺 PoC/ADR；implementation-ready blocked；计划明确 tracking-only |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260723-lane-doc-product-complexity-boundary-handoff.md](../checkpoints/20260723-lane-doc-product-complexity-boundary-handoff.md)（DOC：ADR-0015 将严格 High-Assurance 审计后置，Ordinary Core 保持默认产品范围）
2. [20260723-lane-krn-d018-governance-ports-handoff.md](../checkpoints/20260723-lane-krn-d018-governance-ports-handoff.md)（KRN：D-018 durable governance-header resolution port 已提供；focused test 已执行；仍非 D-018 闭合）
3. [20260723-lane-cfr-ctr-ordinary-core-audit-inspect-handoff.md](../checkpoints/20260723-lane-cfr-ctr-ordinary-core-audit-inspect-handoff.md)（CFR+CTR：双 REQ mapping + Ordinary Core AUDIT 行为向量测试已执行）

## 客户端目录治理交付

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](../../clients/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](../../clients/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](../../clients/READINESS.md)：PoC runbook/模板与技术栈比较草案已提供（非执行/非 ADR）；M5 出口已 GO，仍 blocked 于依赖组 1/2/7 完整交付、五平台 PoC 执行、技术栈 ADR、AGPL 法务评估（POC-LIC not-run）、Tier 1 runtime PoC |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](../../clients/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
