# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-CFR M4 故障注入向量执行批：7 向量行为执行 + M4 出口评审 + F-014/F-023 闭合）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | not-started | — | 入口 gate：M4 出口（已达成）+ **F-011 R1 审批合同登记（唯一剩余项，Lane-CTR 批：approval-request/decision schema 硬化 + 负例向量 + registry 映射）** |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | macOS/Linux 产品切片已记录；iOS/Android 设计提示词已提供但尚未执行；implementation 未启动 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **60**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **34**（matrix impl/impl_tests 已回填：REQ-CONF-001/002/003（M1 CFR）+ REQ-STATE-002/003、REQ-EVT-004、REQ-REC-003、REQ-GOBJ-ID-001（M2 KRN）+ REQ-CAP-001/002/003/005、REQ-CTX-002/004/005/006/007/008/011/012、REQ-SEC-001/002、REQ-DISC-STAGNATION-001、REQ-PROFILE-CVM-001（M3 KRN）+ REQ-EFF-001/002/004/005/006、REQ-EFF-STATE-001、REQ-REC-001/002、REQ-RUN-006、REQ-INTENT-ACCEPT-001（M4 KRN，附 Rust 行为测试证据）；其余待各车道证据口径回填） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 19 向量**（M2 3 + M3 9 + **M4 7**：三 crash point、三场景聚合、unknown 隔离、幂等冲突、对账先于 loop 恢复——全经 `cognitive_store::faults` 故障注入驱动；另 state-store-degradation 三层子集落档：M1 静态 + M2 只读 + M4 fencing）+ 内核 Rust 行为测试 93 项（KRN M2 51 + M3 26 + M4 16）+ tracer bullet 端到端证据链（复现确认）；静态合同层：27/81 向量 pass；**均不构成 Profile 覆盖声明**（conformance-evidence §2）；TS 客户端 79 项包内单元测试为实现测试，不计向量执行 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-20 Lane-CFR M4 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **81** |
| **pass** | **46** = 静态合同执行 27（wire-schema 9；contract-traceability 18）+ **行为执行 19**——M2 3（CAS 陈旧写拒、effect 非法出口拒、强推 COMPLETED 拒）+ M3 9（越权/衰减/撤销缓存/rank 前授权/超预算/渲染稳定/停滞/候选收窄/注入隔离）+ **M4 7**（EFF-CRASH-001/002/003 三 crash point、RECOVERY-CRASH-006 聚合、EFF-UNK-003 unknown 隔离、EFF-IDEM-CONFLICT-001 同键异参拒、AGENT-RECOVERY-003 对账先于 loop 恢复；被测 = effects/recovery + faults 故障注入，报告 execution.implementation 标注） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run（需 M5+ 运行时行为，逐条理由入报告）** | **35**（含 state-store-degradation：M1 静态 + M2 只读 + **M4 fencing 三层子集落档**，disk-full deferred/管理面挂 M5；DISC-DELTA-SCOPE-003：delta 消费 = M5） |
| 错误实现自检 | **27/27 corrupted 向量全部翻 fail**（静态 8 + M2 行为 3 + M3 行为 9 + **M4 行为 7**（effect/recovery 反模式：换键重铸双发/unknown 盲重发/commit 恢复期重执行/冲突当去重/未对账即恢复 loop）；`--self-check`，conformance-evidence §3；CI 步骤断言） |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`；报告 sha256 由 runner 打印）。层 7/8 无专属 slug = D-004 已按文档化跨切片映射闭合（conformance/README + runner `CROSS_SLICE_HOSTED`）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 0（+1 证据性质） | F-001（证据缺口，随 M1~M6 消解，不阻断；行为执行 19 向量 + 内核行为测试 93 项 + tracer bullet 证据链在案） |
| P1 | **2** | F-011（M5 入口剩余项，Lane-CTR 登记批）、F-017（M6）；另 F-015 持续收敛。**F-014/F-023 已于 M4 出口评审闭合（F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认）** |
| 漂移 | 0 开放（+2 deferred，+1 decided） | **D-017 deferred-to-v0.2**（范围已扩展覆盖两个 `cognitiveos.impl.` 域：M2 重放投影 + M3 渲染 digest；判定框架同构，触发条件 = 任一域 digest 跨信任边界成为机器合同）；**D-018 decided**（事件 envelope 升格正式裁决：路线 (b)，M5 Lane-RUN 治理发布边界组装、字段来源 = M3 链经 M5 持久化治理对象、content digest 域 = `governed-object-content/0.1`、schema_digest 消费生成 SCHEMA_DIGEST 常量、零 schema 修正需求；M5 交付时闭合）；**D-016 deferred-to-v0.2**（management 操作名，M5 实现反馈驱动）；D-013/D-014/D-015（Lane-CTR 缺口批闭合）、D-004、D-012（M1 Lane-CFR）、D-001/D-006/D-011（M1 Lane-CTR）与 D-002/D-003/D-005/D-007~D-010 均已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | M4 前评估批已合并（PR #11） | `lane/ctr` | 渲染域并入 D-017、membership defer 至 M5、D-018 decided。**下批 = M5 入口 gate 剩余项：F-011 R1 审批合同登记**（approval-request/decision 机器 schema 硬化 + 负例向量 + registry 映射，范围建议见 M4 review §7；D-016 维持 v0.2 defer；membership 绑定与 D-018 实施随 M5 消费方） |
| Lane-CFR 符合性与工具 | **M4 故障注入向量执行批已交付**（本页所在 PR） | `lane/cfr` | 已完成：runner M4 行为门（被测 = `effects`/`recovery` + `cognitive_store::faults` 故障注入；7 向量脱 not-run：三 crash point + 聚合 + unknown 隔离 + 幂等冲突 + 对账先于恢复）、state-store-degradation 增 M4 fencing 子集（三层落档，disk-full deferred 如实）、effect/recovery 反模式自检（五种 → 对应向量翻 fail，合计 27/27）、钉扎同批调整（ci.yml 81/46/0/0/0/35 + self-check ≥27、runner_execution.rs 9 测试）、台账升级（F-006/F-010 → verified-by-vector；F-014/F-023 → closed-by-M4）、M4 出口评审（tracer bullet 复现确认 + F-023 拒绝码确认）。**触碰通告**：`crates/cognitive-conformance/**`（+behavior_m4 模块）、`.github/workflows/ci.yml`（钉扎计数）、`conformance/README.md`（Running 节） |
| Lane-KRN 内核主线 | M4 批已合并（PR #12）；**M5 kernel 侧待 F-011 登记后启动** | `lane/krn` | M5 剩余 kernel 面（KRN M4 handoff §5）：UserIntentRecord/interpretation 绑定、运行时 Loop 驱动（OODA 迭代器）、admission 编排、恢复步骤 6/7 跨 activity 编排 |
| Lane-TSC TS 客户端 | 换绑批已合并（PR #6）：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定——errors.ts 消费 `errorRegistry`（删手写 55 码表 + 测试时 YAML 对读）、envelope.ts 消费 `akpRequestEnvelope`/`akpResultEnvelope`（删手工信封接口；新增 payload⊕payload_ref 与 partial⇒continuation 门）、views.ts 消费 shell 族 6 生成模块 + `SCHEMA_DIGESTS`（删 5 手工接口/`SHELL_SCHEMA_DIGESTS`/`CancelControl`/`SHELL_CONTROL_PROVISIONAL_PIN` 及 digest 重derive 漂移门）、watch.ts 消费 `akpStreamFrame` 且流错误码收口 `error.code`（D-015 行为适配 + 旧形状负例）；语义负例全部保持通过；**79 项 TS 客户端单元测试**（sdk-ts 67 / agent-shell 12），仍为实现测试、不计向量执行；剩余临时机制清单见 handoff §2/§4 | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | 维护依赖表与 informative 平台产品设计；iOS/Android 提示词待执行；实现 gate 未通过 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-lane-cfr-m4-handoff.md](../checkpoints/20260720-lane-cfr-m4-handoff.md)（Lane-CFR M4 故障注入向量执行批：7 向量行为执行、fencing 子集、反模式自检 27/27、F-006/F-010/F-014/F-023 台账、M4 出口评审）
2. [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md)（M4 出口评审：8 判据逐条证据 + tracer bullet 复现 + F-023 拒绝码确认，GO；M5 入口剩余项 = F-011 R1 登记）
3. [20260720-lane-krn-m4-handoff.md](../checkpoints/20260720-lane-krn-m4-handoff.md)（Lane-KRN M4 批：Intent/Effect 协议、幂等、F-023 准入矩阵、F-014 sink fencing 矩阵、恢复八步、三 crash point 故障注入框架、tracer bullet 正负链、CFR effect-recovery 候选清单）
