# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-21（Lane-TSC M5：HttpSseTransport 真对接 kernel-server；Lane-RUN 批 2b 已合入；向量执行归 CFR；Lane-CON 仍 blocked 等待 M5 出口）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | **in-progress（RUN+TSC 已交付；待 CFR 向量 + 出口评审）** | — | **RUN 批 1–2b**（PR #16/#21/#27）+ **KRN/CTR** 已合入。**TSC M5**：`HttpSseTransport` 对接 `/management/*`、`/shell/*`、`GET /task/watch`（sdk-ts 72 / agent-shell 13；含可选 live）。**待**：CFR M5 行为向量（F-011 三负例、shell cancel/detach/watch、intent-supersede 等）+ milestone review；F-011/D-018 闭合挂 CFR |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | 客户端项目根迁移完成（ADR-0007）；Phase 0 文档收口；解阻复核后仍 blocked：RUN 批 2b 已交付但缺 M5 出口评审 / PoC / ADR；依赖组 1/2/7 仍未完整；implementation-ready 仍 **no**；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行** | Next.js 38 静态/SSG 页面；Vitest 14/14；Playwright Chromium 22/22；全模板 axe WCAG 2.0/2.1/2.2 A/AA 通过 | 仅研究发布与展示层；不改变 REQ 实现、向量执行或 Profile 符合状态 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **61**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **64**（matrix 实测非空 impl；相对批 2a 的 59：+RUN 批 2b 回填 REQ-SHELL-ATTACH/CONTROL/CORRECTION/DETACH/PREVIEW；RUN-004/005/007/008 与 REC-001 增补 runtime 路径） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 19 向量**（不变；CFR M5 批前）+ workspace Rust **196** 项 + tracer bullet；静态 27/81；**均不构成 Profile 覆盖声明**；TS **85** 项包内单元测试（sdk-ts 72 / agent-shell 13；含 3 live HTTP/SSE）不计向量执行 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-20 Lane-CFR M4 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **84**（F-011 批 +3：MGMT-APPROVAL-R1-009/SELF-010/FATIGUE-011，行为负例登记 not-run） |
| **pass** | **46** = 静态合同执行 27（wire-schema 9；contract-traceability 18）+ **行为执行 19**——M2 3（CAS 陈旧写拒、effect 非法出口拒、强推 COMPLETED 拒）+ M3 9（越权/衰减/撤销缓存/rank 前授权/超预算/渲染稳定/停滞/候选收窄/注入隔离）+ **M4 7**（EFF-CRASH-001/002/003 三 crash point、RECOVERY-CRASH-006 聚合、EFF-UNK-003 unknown 隔离、EFF-IDEM-CONFLICT-001 同键异参拒、AGENT-RECOVERY-003 对账先于 loop 恢复；被测 = effects/recovery + faults 故障注入，报告 execution.implementation 标注） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run（需 M5+ 运行时行为，逐条理由入报告）** | **38**（+3 F-011 审批负例，行为执行归 CFR M5 批；含 state-store-degradation：M1 静态 + M2 只读 + **M4 fencing 三层子集落档**，disk-full deferred/管理面挂 M5；DISC-DELTA-SCOPE-003：delta 消费 = M5） |
| 错误实现自检 | **27/27 corrupted 向量全部翻 fail**（静态 8 + M2 行为 3 + M3 行为 9 + **M4 行为 7**（effect/recovery 反模式：换键重铸双发/unknown 盲重发/commit 恢复期重执行/冲突当去重/未对账即恢复 loop）；`--self-check`，conformance-evidence §3；CI 步骤断言） |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`；报告 sha256 由 runner 打印）。层 7/8 无专属 slug = D-004 已按文档化跨切片映射闭合（conformance/README + runner `CROSS_SLICE_HOSTED`）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 0（+1 证据性质） | F-001（证据缺口，随 M1~M6 消解，不阻断；行为执行 19 向量 + 内核行为测试 93 项 + tracer bullet 证据链在案） |
| P1 | **2** | F-011（R1 合同已登记、M5 入口已开；行为证据随 M5 收口）、F-017（M6）；另 F-015 持续收敛。**F-014/F-023 已于 M4 出口评审闭合** |
| 漂移 | 0 开放（+2 deferred，+1 decided/partial） | **D-017 deferred-to-v0.2**；**D-018 partially-implemented**（RUN 批 2a 组装器已交付；闭合仍待 CFR watch/shell 向量证据 + 治理对象端口）；**D-016 deferred-to-v0.2**；D-019 已闭合；其余 D-001~D-015 均已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **M5 修正型绑定批已交付** | `lane/ctr` | `intent-interpretation` / `privileged-management-session` / `management-action-proposal` 纳入 codegen CORE_SET；Rust/TS 各 35 schema 模块，KRN/RUN 精确换装点见 CTR M5 handoff；无规范资产或行为语义变化 |
| Lane-CFR 符合性与工具 | **M4 故障注入向量执行批已交付** | `lane/cfr` | M5 行为向量持续；优先领取 `clients/` 扫描根、链接/anchor、必填字段与唯一 canonical 自动防漂移任务并附注入演练 |
| Lane-KRN 内核主线 | **M5 kernel 侧批已交付**（意图链 + 修正 fencing + Loop 端口 + 恢复 6/7 事实 + D-018 协作项；端口面冻结于 [KRN M5 handoff §7](../checkpoints/20260720-lane-krn-m5-handoff.md)；intent-interpretation 生成绑定已由 CTR 交付并完成机械换装） | `lane/krn` | 下一批候选：governance currency 收编 store 表 + execution↔effect 关联（RUN 批 1 handoff §4.2 请求，已认领评估） |
| Lane-TSC TS 客户端 | **M5 HTTP/SSE 集成已交付**（`HttpSseTransport` 换绑 kernel-server；live 3 + 单元；agent-shell `createLiveShellSession`）；客户端永不做 authority | `lane/tsc` | M5 TSC 面完成；后续增量 = proposal/preview/submit 完整 HTTP 面（随 RUN 扩展） |
| Lane-RUN 运行时与管理面 | **M5 批 2b 已交付**（PR #27） | `lane/run` | D-018/F-011 闭合挂 CFR；治理对象端口待 KRN |
| Lane-CFR 符合性与工具 | **M4 已交付；M5 向量执行待启动** | `lane/cfr` | 优先：F-011 三负例、MGMT-FALLBACK、shell-cancel/detach/watch-resume、intent-supersede |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | `work/clients-m5-unblock-review` | Phase 0 本地已尽；PR #21 合入后解阻复核完成——仍 blocked（缺批 2b/M5 出口/PoC/ADR；依赖组 1/2/7 未完整）；**进入等待上游**；gate：`clients/governance/readiness-gates.md`；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260721-lane-tsc-m5-http-sse-handoff.md](../checkpoints/20260721-lane-tsc-m5-http-sse-handoff.md)（Lane-TSC M5：HttpSseTransport ↔ kernel-server）
2. [20260721-lane-run-m5-batch2b-handoff.md](../checkpoints/20260721-lane-run-m5-batch2b-handoff.md)（Lane-RUN M5 批 2b：Harness Loop / Shell / intent / recovery 6–7）
3. [20260721-lane-con-m5-unblock-review-handoff.md](../checkpoints/20260721-lane-con-m5-unblock-review-handoff.md)（Lane-CON：仍缺 M5 出口/PoC/ADR；gate 仍 blocked）

## 客户端目录治理交付

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](../../clients/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](../../clients/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](../../clients/READINESS.md)：PoC runbook/模板与技术栈比较草案已提供（非执行/非 ADR）；仍 blocked 于 M5 出口、依赖组 1/2/7、五平台 PoC 执行、技术栈 ADR、AGPL 法务评估（POC-LIC not-run）、Tier 1 runtime PoC |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](../../clients/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
