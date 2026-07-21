# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-21（Lane-RUN M5 批 2a：R1 结构化审批、session 生命周期、AKP HTTP/SSE watch 与 D-018 可证明组装分量已交付；向量保持 not-run。Lane-KRN M5 kernel 侧批：意图链 UserIntentRecord→candidate→确定性准入→TaskContract、修正 epoch fencing `INTENT_VERSION_SUPERSEDED`、有界 Loop 驱动端口、恢复 6/7 编排事实、D-018 KRN 协作项交付——实现已提供 + 车道 Rust 行为测试已执行，端口面冻结于 [KRN M5 handoff](../checkpoints/20260720-lane-krn-m5-handoff.md) §7；向量保持 not-run 归 Lane-CFR。同日：Lane-RUN M5 批 1 无模型确定性 admin CLI 四动词已交付；ADR-0008 自动提交/推送治理政策生效）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | **in-progress（RUN 批 2a + KRN kernel 侧批已交付）** | — | 入口 gate 达成（M4 出口 + F-011 R1 登记）。**RUN 批 1（PR #16）**：无模型四动词 + session 门（判据 5）。**RUN 批 2a**：session 签发/续期/撤销生命周期 + R1 结构化审批门（F-011 三负例语义、dispatches=0）+ AKP envelope/HTTP JSON + SSE snapshot/cursor（`WATCH_CURSOR_STALE`）+ D-018 event envelope 组装器（可证明分量）。**KRN M5**：意图链/修正 fencing/Loop 端口/恢复 6/7（判据 1–2 的 kernel 分量；端口冻结 handoff §7）。**待批 2b**：Harness Loop 运行时 + Shell proposal/preview/submit/attach/detach/cancel（判据 1–4/6）；向量执行归 CFR |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | 客户端项目根迁移完成（ADR-0007）；Phase 0 文档收口（PR #18/#19/#20）；**2026-07-21 解阻复核（PR #21 已合）**：akp **已脱离骨架**；runtime/kernel-server **部分脱离**（D-018 / `--once` HTTP）；无 m5-milestone-review；缺批 2b/出口/PoC/ADR；依赖组 1/2/7 仍未完整；implementation-ready 仍 **no**；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行** | Next.js 38 静态/SSG 页面；Vitest 14/14；Playwright Chromium 22/22；全模板 axe WCAG 2.0/2.1/2.2 A/AA 通过 | 仅研究发布与展示层；不改变 REQ 实现、向量执行或 Profile 符合状态 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **61**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **59**（matrix 实测非空 impl；相对 KRN M5 基线 46：+RUN 批 2a 回填含 REQ-MGMT-APPROVAL-001、REQ-AKP-CAN/CONT/ENV/MGMT/SHELL/STR/VER 族、REQ-EVT-001 等；批 1 五 MGMT + KRN 七 INTENT/RUN 仍在列） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 19 向量**（不变）+ workspace Rust **189** 项全绿（含 KRN 109 + RUN 批 1 16 + 批 2a 7 等）+ tracer bullet；静态 27/81；**均不构成 Profile 覆盖声明**；TS 79 项包内单元测试不计向量执行 |
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
| Lane-CTR 契约与生成 | **F-011 批已交付** | `lane/ctr` | R1 审批合同登记完成，M5 入口 gate 达成；遗留 D-016、membership 绑定与 D-018 实施随 M5 反馈收口 |
| Lane-CFR 符合性与工具 | **M4 故障注入向量执行批已交付** | `lane/cfr` | M5 行为向量持续；优先领取 `clients/` 扫描根、链接/anchor、必填字段与唯一 canonical 自动防漂移任务并附注入演练 |
| Lane-KRN 内核主线 | **M5 kernel 侧批已交付**（意图链 + 修正 fencing + Loop 端口 + 恢复 6/7 事实 + D-018 协作项；端口面冻结于 [KRN M5 handoff §7](../checkpoints/20260720-lane-krn-m5-handoff.md)；给 CTR 的 intent-interpretation codegen 请求见其 §4.1） | `lane/krn` | 下一批候选：governance currency 收编 store 表 + execution↔effect 关联（RUN 批 1 handoff §4.2 请求，已认领评估）；intent-interpretation 生成绑定换装（等 CTR） |
| Lane-TSC TS 客户端 | 换绑批已合并（PR #6）：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定——errors.ts 消费 `errorRegistry`（删手写 55 码表 + 测试时 YAML 对读）、envelope.ts 消费 `akpRequestEnvelope`/`akpResultEnvelope`（删手工信封接口；新增 payload⊕payload_ref 与 partial⇒continuation 门）、views.ts 消费 shell 族 6 生成模块 + `SCHEMA_DIGESTS`（删 5 手工接口/`SHELL_SCHEMA_DIGESTS`/`CancelControl`/`SHELL_CONTROL_PROVISIONAL_PIN` 及 digest 重derive 漂移门）、watch.ts 消费 `akpStreamFrame` 且流错误码收口 `error.code`（D-015 行为适配 + 旧形状负例）；语义负例全部保持通过；**79 项 TS 客户端单元测试**（sdk-ts 67 / agent-shell 12），仍为实现测试、不计向量执行；剩余临时机制清单见 handoff §2/§4 | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | **M5 批 2a 已交付实现**（session 生命周期 + R1 审批负例 + AKP HTTP/SSE watch + D-018 组装器；车道测试已执行，向量仍 not-run） | `lane/run` | 批 2b：Harness Loop + Shell proposal/preview/submit/attach/detach/cancel；D-018 治理对象持久化/解析端口待 KRN |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | `work/clients-m5-unblock-review` | Phase 0 本地已尽；PR #21 合入后解阻复核完成——仍 blocked（缺批 2b/M5 出口/PoC/ADR；依赖组 1/2/7 未完整）；**进入等待上游**；gate：`clients/governance/readiness-gates.md`；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260721-lane-con-m5-unblock-review-handoff.md](../checkpoints/20260721-lane-con-m5-unblock-review-handoff.md)（Lane-CON：PR #21 合入后解阻复核；akp 已非骨架；仍缺批 2b/出口/PoC/ADR；gate 仍 blocked；进入等待上游）
2. [20260721-lane-run-m5-batch2a-handoff.md](../checkpoints/20260721-lane-run-m5-batch2a-handoff.md)（Lane-RUN M5 批 2a：session 生命周期 / R1 审批 / AKP HTTP+SSE / watch / D-018 组装器）
3. [20260721-lane-con-m5-monitor-handoff.md](../checkpoints/20260721-lane-con-m5-monitor-handoff.md)（Lane-CON：PR #20 合入后 M5 细监控；批 2a 当时仅 lane/run）

## 客户端目录治理交付

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](../../clients/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](../../clients/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](../../clients/READINESS.md)：PoC runbook/模板与技术栈比较草案已提供（非执行/非 ADR）；仍 blocked 于 M5 出口、依赖组 1/2/7、五平台 PoC 执行、技术栈 ADR、AGPL 法务评估（POC-LIC not-run）、Tier 1 runtime PoC |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](../../clients/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
