# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-KRN M3 治理链与 Context 批会话：六步授权门、capability 交集/衰减/撤销、九阶段确定性 Context Resolution、治理绑定缓存键、确定性渲染；并行合并 Lane-CTR KRN 缺口批 PR #8——transition request/record 入 codegen（30 模块）、D-017 deferred、D-018 决策落档——后合并方 KRN 解决本页冲突）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **impl 批已交付**（出口评审待 CFR 行为向量执行） | — | Lane-KRN 已交付六步授权门 / capability 交集·单调衰减·撤销 epoch / 九阶段确定性 Context Resolution / 治理绑定缓存键 / 确定性渲染 + 8 验收判据行为测试与 F-007 双竞态点证据（[20260720-lane-krn-m3-handoff.md](../checkpoints/20260720-lane-krn-m3-handoff.md)）；security-negative/context 向量保持 not-run 诚实口径，行为执行批归 Lane-CFR，之后做 M3 出口评审 |
| M4 Intent/Effect 与恢复 + tracer bullet | not-started | — | 入口 gate：M1 出口（已达成）+ M2/M3 行为验收 + F-014/F-023 排入 |
| M5 意图链/Harness/Shell/管理面 | not-started | — | 入口另需 F-011 R1 合同登记 |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | macOS/Linux 产品切片已记录；iOS/Android 设计提示词已提供但尚未执行；implementation 未启动 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **60**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **24**（matrix impl/impl_tests 已回填：REQ-CONF-001/002/003（M1 CFR）+ REQ-STATE-002/003、REQ-EVT-004、REQ-REC-003、REQ-GOBJ-ID-001（M2 KRN）+ REQ-CAP-001/002/003/005、REQ-CTX-002/004/005/006/007/008/011/012、REQ-SEC-001/002、REQ-DISC-STAGNATION-001、REQ-PROFILE-CVM-001（M3 KRN，附 Rust 行为测试证据）；其余待各车道证据口径回填） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 3 向量**（STATE-CAS-002 / EFFECT-STATE-CLOSURE-008 / GW-REMOTE-COMPLETE-001，被测 = 真实 `cognitive-kernel`+`cognitive-store` 权威路径；另 state-store-degradation M2 只读子集真实执行落档）+ 内核 Rust 行为测试 51 项（KRN）；静态合同层：28/81 向量 pass；**均不构成 Profile 覆盖声明**（conformance-evidence §2）；TS 客户端 79 项包内单元测试为实现测试，不计向量执行 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-20 Lane-CFR M2 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **81** |
| **pass** | **31** = 静态合同执行 28（wire-schema 9：GOBJ 负例 ×2 + spec-coverage + perf 合同 + AKP/控制 payload 负例 ×5；contract-traceability 18；security-negative 1（prompt-injection 信任面静态合同））+ **行为执行 3**（state-machine 1：STATE-CAS-002 陈旧写拒绝；shell-intent 2：EFFECT-STATE-CLOSURE-008 非法出口拒绝、GW-REMOTE-COMPLETE-001 强推 COMPLETED 拒绝——被测 = 真实 kernel/store 权威路径，报告 execution.implementation 标注） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run（需 M3+ 运行时行为，逐条理由入报告）** | **50**（含 state-store-degradation：M1 静态断言 + **M2 只读降级子集真实执行**落 `partial_contract_assertions`，disk-full/dispatch/stop-revoke 归 M4/M5，F-008） |
| 错误实现自检 | **12/12 corrupted 向量全部翻 fail**（静态 9 + 行为 3：gate-bypassing 直写 store 的错误实现被判 fail；`--self-check`，conformance-evidence §3；CI 步骤断言） |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`；报告 sha256 由 runner 打印）。层 7/8 无专属 slug = D-004 已按文档化跨切片映射闭合（conformance/README + runner `CROSS_SLICE_HOSTED`）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 0（+1 证据性质） | **F-003 已闭合**（2026-07-20 Lane-CFR：runner 真实执行两份双轨拒绝负例 → pass）；F-001（证据缺口，随 M1~M6 消解，不阻断） |
| P1 | 4 | F-011（M5）、F-014（M4）、F-023（M4）、F-017（M6）；另 F-015 持续收敛 |
| 漂移 | 0 开放（+2 deferred，+1 决策落档） | **D-017 deferred-to-v0.2**（内核重放投影合同/digest 域：REQ-STATE-002 属性级、投影 derived 非登记面，注册 = 新增对象形合同；触发条件 = 投影 digest 跨信任边界成为机器合同）；**D-018 决策落档**（事件 envelope 升格路径：M3 治理链后 CTR 修正型评估，实施随 M3/M5 gate）；**D-016 deferred-to-v0.2**（management 操作名，M5 实现反馈驱动）；D-013/D-014/D-015（Lane-CTR 缺口批闭合）、D-004、D-012（M1 Lane-CFR）、D-001/D-006/D-011（M1 Lane-CTR）与 D-002/D-003/D-005/D-007~D-010 均已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **KRN 缺口批已交付**（本页所在 PR） | `lane/ctr` | KRN M2 handoff §4 四项收敛：① state-transition-request/record 入 codegen 生成集（30 schema 模块 ×2 语言，`engine.rs` 替换点归 Lane-KRN）；② 投影合同/digest 域 **D-017 deferred-to-v0.2**（判定依据入台账；内核 `cognitiveos.impl.` 域维持）；③ 事件 envelope 升格路径 **D-018 决策落档**（M3 后 CTR 评估，M5 边界组装为预期路线）；④ 错误码映射口径核对无误、无动作。schema/向量/runner 钉扎零变化（60/81、81/31/50、self-check 12）。**触碰通告**：仅 `crates/cognitive-contracts`、`packages/contracts-ts` 生成物与计数测试（28→30 模块钉扎） |
| Lane-CFR 符合性与工具 | **M2 行为执行批已交付**（本页所在 PR） | `lane/cfr` | 已完成：runner 行为执行模式（被测 = `cognitive-kernel`+`cognitive-store` 真实权威路径；3 向量脱静态/not-run → 行为 pass；state-store-degradation M2 只读子集真实执行落档）、行为侧错误实现自检（gate-bypassing 直写 store → 3 向量翻 fail，合计 12/12）、钉扎同批调整（ci.yml 81/31/0/0/0/50 + self-check ≥12、runner_execution.rs 7 测试）、M2 出口评审。**触碰通告**：`crates/cognitive-conformance/**`（+behavior 模块，Cargo 依赖 +kernel/store/domain/tempfile）、`.github/workflows/ci.yml`（钉扎计数）、`conformance/README.md`（Running 节行为模式）、`Cargo.lock` |
| Lane-KRN 内核主线 | **M3 治理链与 Context 批已交付**（本页所在 PR） | `lane/krn` | 已完成：`cognitive-domain::capability`（单调衰减违规清单/链交集只缩不扩/lease 窗口/instant 比较）、`cognitive-kernel::authz`（六步判定序、拒绝与 not-found 同形、撤销 epoch 复验 + `capability_and_revocation_current` 唯一 attestation 派生）、`cognitive-kernel::context`（九阶段管线：治理预过滤→逐对象正文重验→ranker 提案隔离→预算 fail-closed→loss declaration→确定性渲染前缀稳定）、`cognitive-kernel::context_cache`（7+1 维治理键、epoch 失配即失效、派生缓存随条目死亡）；8 验收判据 + 注入隔离 + 停滞 + F-007 双竞态点各有行为测试（真 SQLite 跑竞态）。**触碰通告**：仅三 crate 源码 + matrix 16 REQ impl 回填（无 Cargo 依赖/schema/runner 变化）。M4 待 M3 出口评审 |
| Lane-TSC TS 客户端 | 换绑批已合并（PR #6）：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定——errors.ts 消费 `errorRegistry`（删手写 55 码表 + 测试时 YAML 对读）、envelope.ts 消费 `akpRequestEnvelope`/`akpResultEnvelope`（删手工信封接口；新增 payload⊕payload_ref 与 partial⇒continuation 门）、views.ts 消费 shell 族 6 生成模块 + `SCHEMA_DIGESTS`（删 5 手工接口/`SHELL_SCHEMA_DIGESTS`/`CancelControl`/`SHELL_CONTROL_PROVISIONAL_PIN` 及 digest 重derive 漂移门）、watch.ts 消费 `akpStreamFrame` 且流错误码收口 `error.code`（D-015 行为适配 + 旧形状负例）；语义负例全部保持通过；**79 项 TS 客户端单元测试**（sdk-ts 67 / agent-shell 12），仍为实现测试、不计向量执行；剩余临时机制清单见 handoff §2/§4 | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | 维护依赖表与 informative 平台产品设计；iOS/Android 提示词待执行；实现 gate 未通过 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-lane-krn-m3-handoff.md](../checkpoints/20260720-lane-krn-m3-handoff.md)（Lane-KRN M3 治理链与 Context 批：六步授权门、capability 交集/衰减/撤销、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态点证据、CFR 扩展批候选向量清单）
2. [20260720-lane-ctr-krn-gaps-handoff.md](../checkpoints/20260720-lane-ctr-krn-gaps-handoff.md)（Lane-CTR KRN 缺口批：transition 对入 codegen、D-017 deferred、D-018 决策落档、错误码口径核对）
3. [20260720-lane-cfr-m2-handoff.md](../checkpoints/20260720-lane-cfr-m2-handoff.md)（Lane-CFR M2 行为执行批：runner 行为模式、3 向量行为 pass、降级子集落档、行为自检、M2 出口评审）
