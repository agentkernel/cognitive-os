# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-CTR F-011 R1 审批合同登记完成，M5 入口 gate 达成；Lane-CON `clients/` 客户端项目根迁移完成并对齐远端 61 schema / 84 vectors，structure-ready yes / implementation-ready no(blocked)）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | **入口 gate 已达成，可启动** | — | M4 出口（已达成）+ **F-011 R1 审批合同登记完成（2026-07-20 Lane-CTR：approval-request schema + decision R1 硬化 + 3 负例向量 + registry 映射；行为验证挂 CFR M5 批）**；建议提示词 docs/prompts/milestone-m5.md / lane-run.md |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | 客户端项目根迁移完成：全部产品/平台/Agent Hub 文档归于 `clients/`（[项目地图](../../clients/README.md)，ADR-0007）；macOS/Linux/iPhone/Android phone 产品切片、Agent Hub canonical 文档（两部署模式、L1–L8、6 dossier、21 威胁项实测）与开发计划/提示词全部迁入；客户端 implementation 均未启动，平台测试未执行，Open PoC 全 not-run，Profile 未符合 |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行** | Next.js 38 静态/SSG 页面；Vitest 14/14；Playwright Chromium 22/22；全模板 axe WCAG 2.0/2.1/2.2 A/AA 通过 | 仅研究发布与展示层；不改变 REQ 实现、向量执行或 Profile 符合状态 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **61**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **34**（matrix impl/impl_tests 已回填：REQ-CONF-001/002/003（M1 CFR）+ REQ-STATE-002/003、REQ-EVT-004、REQ-REC-003、REQ-GOBJ-ID-001（M2 KRN）+ REQ-CAP-001/002/003/005、REQ-CTX-002/004/005/006/007/008/011/012、REQ-SEC-001/002、REQ-DISC-STAGNATION-001、REQ-PROFILE-CVM-001（M3 KRN）+ REQ-EFF-001/002/004/005/006、REQ-EFF-STATE-001、REQ-REC-001/002、REQ-RUN-006、REQ-INTENT-ACCEPT-001（M4 KRN，附 Rust 行为测试证据）；其余待各车道证据口径回填） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 19 向量**（M2 3 + M3 9 + **M4 7**：三 crash point、三场景聚合、unknown 隔离、幂等冲突、对账先于 loop 恢复——全经 `cognitive_store::faults` 故障注入驱动；另 state-store-degradation 三层子集落档：M1 静态 + M2 只读 + M4 fencing）+ 内核 Rust 行为测试 93 项（KRN M2 51 + M3 26 + M4 16）+ tracer bullet 端到端证据链（复现确认）；静态合同层：27/81 向量 pass；**均不构成 Profile 覆盖声明**（conformance-evidence §2）；TS 客户端 79 项包内单元测试为实现测试，不计向量执行 |
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
| 漂移 | 0 开放（+2 deferred，+1 decided） | **D-017 deferred-to-v0.2**；**D-018 decided**（M5 Lane-RUN+KRN 实施）；**D-016 deferred-to-v0.2**；D-019 客户端长期分支编号/计数漂移已在本集成批闭合；其余 D-001~D-015 均已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **F-011 批已交付** | `lane/ctr` | R1 审批合同登记完成，M5 入口 gate 达成；遗留 D-016、membership 绑定与 D-018 实施随 M5 反馈收口 |
| Lane-CFR 符合性与工具 | **M4 故障注入向量执行批已交付** | `lane/cfr` | M5 行为向量持续；优先领取 `clients/` 扫描根、链接/anchor、必填字段与唯一 canonical 自动防漂移任务并附注入演练 |
| Lane-KRN 内核主线 | M4 批已合并（PR #12）；**M5 kernel 侧可启动** | `lane/krn` | UserIntentRecord/interpretation 绑定、运行时 Loop 驱动、admission 编排、恢复步骤 6/7 跨 activity 编排 |
| Lane-TSC TS 客户端 | 换绑批已合并（PR #6）：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定——errors.ts 消费 `errorRegistry`（删手写 55 码表 + 测试时 YAML 对读）、envelope.ts 消费 `akpRequestEnvelope`/`akpResultEnvelope`（删手工信封接口；新增 payload⊕payload_ref 与 partial⇒continuation 门）、views.ts 消费 shell 族 6 生成模块 + `SCHEMA_DIGESTS`（删 5 手工接口/`SHELL_SCHEMA_DIGESTS`/`CancelControl`/`SHELL_CONTROL_PROVISIONAL_PIN` 及 digest 重derive 漂移门）、watch.ts 消费 `akpStreamFrame` 且流错误码收口 `error.code`（D-015 行为适配 + 旧形状负例）；语义负例全部保持通过；**79 项 TS 客户端单元测试**（sdk-ts 67 / agent-shell 12），仍为实现测试、不计向量执行；剩余临时机制清单见 handoff §2/§4 | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | **M5 入口 gate 已开，可启动** | `lane/run`（待建） | 领取 M5 runtime/management/AKP/kernel-server 主线；先按 `docs/prompts/lane-run.md` 建车道 |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | `clients/` 客户端项目根已建立（ADR-0007、CLIENTS-DEC-001）：PC/mobile/shared/Agent Hub 文档全部迁入（Agent Hub 树在 `clients/agent-hub/{docs,plan,prompts}/`），全部实现车道 `blocked`；实现 gate（canonical：`clients/governance/readiness-gates.md`）未通过（Agent Hub 另加 Paseo/AGPL 法务 gate） |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-lane-con-clients-root-integration-handoff.md](../checkpoints/20260720-lane-con-clients-root-integration-handoff.md)（clients 迁移 squash 集成远端 M5 gate 基线、D-019 闭合、独立博客历史排除）
2. [20260720-lane-con-clients-root-migration-handoff.md](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md)（`clients/` 项目根建立与四域文档迁移、4 stub、rules 16/17、ADR-0007、readiness 双结论）
3. [20260720-lane-ctr-f011-handoff.md](../checkpoints/20260720-lane-ctr-f011-handoff.md)（Lane-CTR F-011 批：R1 审批合同登记、3 负例向量、codegen 32 模块、M5 gate 达成）

## 客户端目录治理交付

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](../../clients/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](../../clients/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](../../clients/READINESS.md)：M5 入口已开，但 M5 出口、依赖组 1/2/7 完整交付、五平台 PoC、技术栈 ADR、AGPL 法务与 provider 接口核验仍未解 |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](../../clients/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
