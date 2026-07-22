# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-23（AUDIT PR #54 已 merge：`54929f1ed8fef1e09ffbb5593633f5d94d5e281e`，merge-triggered main CI `29937238562` 双平台成功；GitHub reviews/reviewDecision/requested reviewers 为空；owner-authorized security/audit/compliance 复审未发现阻断性设计缺陷，但不是外部人类、第三方或 GitHub review；**SIG independent security/cryptography review 仍未完成**。[V02-CA-OPS-REG-READINESS-01](V02-CA-OPS-REGISTRATION-ELIGIBILITY-AUDIT.md) 逐项复核八个 candidate 后裁决全部 `blocked`，且 foundation 的 exact ID/SemVer/digest domain/empty-set/freeze-order/error/digest-cycle 选择未闭合，因此本批为 docs-only registration NO-GO，不登记 machine asset/member；OPS/TARGET/SIG/AUDIT machine contracts 仍未登记，D-016 open，D-022 blocking，CA-1～CA-8 blocked；pins **84/59/25**、self-check **40/40**、matrix impl **70**；Profile **implemented = 0**）

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
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **61**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **70**（matrix 实测非空 impl；shell channel + target resolution 两批各回填 2 条后的当前值） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 32 向量**（M2 3 + M3 9 + M4 7 + M5 6 + M5-intent 2 + M5-shell-channel 1 + **SHELL-TARGET-AMBIGUITY-001** + M6 3）+ workspace Rust 项 + tracer bullet；静态 27/81；**均不构成 Profile 覆盖声明**；TS **85** 项（sdk-ts 72 / agent-shell 13） |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`；RC manifest ≤ `experimental`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-22 Lane-CFR shell-target-ambiguity 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **84** |
| **pass** | **59** = 静态 27 + **行为 32**（M2 3 + M3 9 + M4 7 + M5 6 + M5-intent 2 + SHELL-CHANNEL-ISOLATION-003 + **SHELL-TARGET-AMBIGUITY-001** + M6 3） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run** | **25**（含 MGMT-FALLBACK 全动词、shell migration、delta-scope、store-degradation disk-full 等） |
| 错误实现自检 | **40/40 corrupted 向量全部翻 fail**（+1 shell-target）；CI 地板 ≥40 |

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
| Lane-CTR 契约与生成 | **OPS registration eligibility audited / machine registration NO-GO；PR #55 review pending** | `lane/ctr-v02-ca-ops-registration` | PR #54 merge/main CI gate 已闭合；AUDIT owner-authorized security/audit/compliance review completed（非外部/GitHub review）；`V02-CA-OPS-REG-READINESS-01` 证明八个 candidates 全部 blocked，foundation choices 亦未唯一闭合；PR #55 只交付 docs-only NO-GO audit，不登记任何 member/asset；SIG independent review、四类 registration、CA-0 GO 仍 pending |
| Lane-CFR 符合性与工具 | **shell-target-ambiguity 已合入 main（PR #46）** | `main` @ `0ab3ab4` | pins **59/25**；self-check 40；`SHELL-TARGET-AMBIGUITY-001` pass；handoff：`20260722-lane-cfr-shell-target-ambiguity-handoff.md` |
| Lane-KRN 内核主线 | **M5 kernel 侧批已交付** | `lane/krn` | D-018 端口残留（v0.1 non-claim）；InstallationStore 未做（durable non-claim）；Post-v0.1 计划标 P2 |
| Lane-TSC TS 客户端 | **M5 HTTP/SSE 已交付**（PR #28） | `lane/tsc` | proposal/preview/submit 完整 HTTP 面增量（计划标 P2）；channel isolation 已由 RUN+CFR 补 authority 证据 |
| Lane-RUN 运行时与管理面 | **shell target ambiguity authority 已合入**（PR #45） | `main` @ `eef258d` | `target_resolution::admit_target_selector`；handoff：`20260722-lane-run-shell-target-ambiguity-handoff.md` |
| Lane-DOC 文档维护 | **Post-v0.1 下一阶段计划落盘** | `lane/doc-post-v01-next-phase` | 计划+执行提示词+handoff；V01 L3 non-claim 继承；见 [20260721-post-v01-next-phase-planning-handoff.md](../checkpoints/20260721-post-v01-next-phase-planning-handoff.md) |
| Lane-CON Console | tracking-only | — | M5 GO 后可复评 gate；仍缺 PoC/ADR；implementation-ready blocked；计划明确 tracking-only |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260723-lane-ctr-v02-ca-ops-registration-readiness-handoff.md](../checkpoints/20260723-lane-ctr-v02-ca-ops-registration-readiness-handoff.md)（CTR：八项 eligibility audit；machine registration NO-GO；无 asset/member 登记）
2. [20260722-lane-ctr-v02-ca-audit-design-handoff.md](../checkpoints/20260722-lane-ctr-v02-ca-audit-design-handoff.md)（CTR：AUDIT design merged；owner-authorized review provenance 由后续 handoff 更新）
3. [20260722-lane-ctr-v02-ca-sig-design-handoff.md](../checkpoints/20260722-lane-ctr-v02-ca-sig-design-handoff.md)（CTR：SIG design merged；independent security review still unclaimed；profiles 未登记/不可用）

## 客户端目录治理交付

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](../../clients/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](../../clients/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](../../clients/READINESS.md)：PoC runbook/模板与技术栈比较草案已提供（非执行/非 ADR）；M5 出口已 GO，仍 blocked 于依赖组 1/2/7 完整交付、五平台 PoC 执行、技术栈 ADR、AGPL 法务评估（POC-LIC not-run）、Tier 1 runtime PoC |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](../../clients/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
