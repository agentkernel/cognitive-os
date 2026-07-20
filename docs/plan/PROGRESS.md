# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-CFR M3 行为执行扩展批：治理/context 向量行为执行 + M3 出口评审 + M4 入口 gate 判定）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | not-started | — | **入口 gate 已开**（M3 review §7 逐条核验：F-002~F-010 无开放项，F-006/F-008(disk-full)/F-010 行为项 = M4 自身验收交付物；F-014/F-023 已排入 M4 范围）；建议提示词 docs/prompts/milestone-m4.md / lane-krn.md |
| M5 意图链/Harness/Shell/管理面 | not-started | — | 入口另需 F-011 R1 合同登记 |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | macOS/Linux 产品切片已记录；iOS/Android 设计提示词已提供但尚未执行；implementation 未启动 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **60**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **24**（matrix impl/impl_tests 已回填：REQ-CONF-001/002/003（M1 CFR）+ REQ-STATE-002/003、REQ-EVT-004、REQ-REC-003、REQ-GOBJ-ID-001（M2 KRN）+ REQ-CAP-001/002/003/005、REQ-CTX-002/004/005/006/007/008/011/012、REQ-SEC-001/002、REQ-DISC-STAGNATION-001、REQ-PROFILE-CVM-001（M3 KRN，附 Rust 行为测试证据）；其余待各车道证据口径回填） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 12 向量**（M2 3：CAS/effect 闭包/task 强推，被测 = kernel+store 权威路径；M3 9：横向越权/衰减/撤销缓存/rank 前授权/超预算/渲染稳定/停滞/候选收窄/注入隔离，被测 = authz/context/context_cache/capability 面；另 state-store-degradation M2 只读子集落档）+ 内核 Rust 行为测试 77 项（KRN M2 51 + M3 26）；静态合同层：27/81 向量 pass；**均不构成 Profile 覆盖声明**（conformance-evidence §2）；TS 客户端 79 项包内单元测试为实现测试，不计向量执行 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-20 Lane-CFR M3 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **81** |
| **pass** | **39** = 静态合同执行 27（wire-schema 9；contract-traceability 18）+ **行为执行 12**——M2 3（STATE-CAS-002 陈旧写拒、EFFECT-STATE-CLOSURE-008 非法出口拒、GW-REMOTE-COMPLETE-001 强推 COMPLETED 拒；被测 = kernel+store 权威路径）+ M3 9（GOBJ-TENANT-LATERAL-001 越权拒+拒绝/不存在同形、CAP-ATTEN-004 放大拒、CTX-REVOKE-CACHE-001 撤销缓存拒+派生失效、CTX-RANK-AUTH-001 rank 前授权、CTX-REQ-007 超预算 fail-closed、CTX-RENDER-001 前缀稳定、DISC-STAGNATION-004 停滞有界、DISC-ADMISSION-002 候选收窄、CTX-TRUST-004 注入隔离（静态→行为升级）；被测 = authz/context/context_cache/capability 面，报告 execution.implementation 标注） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run（需 M4+/M5+ 运行时行为，逐条理由入报告）** | **42**（含 state-store-degradation：M1 静态断言 + M2 只读降级子集落档，disk-full/dispatch 归 M4；DISC-DELTA-SCOPE-003：delta 消费 = M5 路径，理由入报告） |
| 错误实现自检 | **20/20 corrupted 向量全部翻 fail**（静态 8 + M2 行为 3（gate-bypass 直写）+ M3 行为 9（治理反模式：membership 即读/先 rank 后授权/陈旧缓存命中/静默截断/无界重试/重排渲染/内容即控制面/接受放大）；`--self-check`，conformance-evidence §3；CI 步骤断言） |

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
| Lane-CTR 契约与生成 | KRN 缺口批已合并（PR #8） | `lane/ctr` | transition request/record 入 codegen（30 模块）、D-017 deferred、D-018 决策落档。**下批候选**（M3 review §7 遗留 + KRN M3 handoff §4）：渲染 digest 域并入 D-017 或同款判定、membership 生成绑定评估、D-018 事件 envelope 升格评估（M3 治理链已落地） |
| Lane-CFR 符合性与工具 | **M3 行为执行扩展批已交付**（本页所在 PR） | `lane/cfr` | 已完成：runner M3 行为门（被测 = `authz`/`context`/`context_cache`/`capability` 真实面；8 向量脱 not-run + CTX-TRUST-004 静态→行为升级；DISC-DELTA-SCOPE-003 如实 not-run 附 M5 理由）、治理类错误实现自检（八种反模式 → 对应向量翻 fail，合计 20/20）、钉扎同批调整（ci.yml 81/39/0/0/0/42 + self-check ≥20、runner_execution.rs 8 测试、check-consistency REQ 引用正则修正——向量 id `CTX-REQ-007` 尾段误报，负向后视断言修正 + 注入演练复绿）、台账升级（F-007/F-018/F-021 → verified-by-vector）、M3 出口评审 + M4 入口 gate 判定。**触碰通告**：`crates/cognitive-conformance/**`（+behavior_m3 模块）、`.github/workflows/ci.yml`（钉扎计数）、`tools/src/check-consistency.mjs`（正则修正）、`conformance/README.md`（Running 节） |
| Lane-KRN 内核主线 | M3 批已合并（PR #9）；**M4 入口 gate 已开**（M3 review §7） | `lane/krn` | M4 Intent/Effect 与恢复 + tracer bullet（`docs/prompts/milestone-m4.md` / `lane-krn.md`；第一动作 = 读 `intent-effect-idempotency.md` 全文，先写「同键异参 EFFECT_IDEMPOTENCY_CONFLICT 拒绝」与「eff-crash-001 dispatch 前崩溃恢复」失败测试） |
| Lane-TSC TS 客户端 | 换绑批已合并（PR #6）：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定——errors.ts 消费 `errorRegistry`（删手写 55 码表 + 测试时 YAML 对读）、envelope.ts 消费 `akpRequestEnvelope`/`akpResultEnvelope`（删手工信封接口；新增 payload⊕payload_ref 与 partial⇒continuation 门）、views.ts 消费 shell 族 6 生成模块 + `SCHEMA_DIGESTS`（删 5 手工接口/`SHELL_SCHEMA_DIGESTS`/`CancelControl`/`SHELL_CONTROL_PROVISIONAL_PIN` 及 digest 重derive 漂移门）、watch.ts 消费 `akpStreamFrame` 且流错误码收口 `error.code`（D-015 行为适配 + 旧形状负例）；语义负例全部保持通过；**79 项 TS 客户端单元测试**（sdk-ts 67 / agent-shell 12），仍为实现测试、不计向量执行；剩余临时机制清单见 handoff §2/§4 | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | 维护依赖表与 informative 平台产品设计；iOS/Android 提示词待执行；实现 gate 未通过 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-lane-cfr-m3-handoff.md](../checkpoints/20260720-lane-cfr-m3-handoff.md)（Lane-CFR M3 行为执行扩展批：9 向量行为执行、治理自检 20/20、台账升级、M3 出口评审 + M4 gate 判定）
2. [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md)（M3 出口评审：7 判据 + 范围项逐条证据，GO → M4，tracer bullet 入口 gate 开启）
3. [20260720-lane-krn-m3-handoff.md](../checkpoints/20260720-lane-krn-m3-handoff.md)（Lane-KRN M3 治理链与 Context 批：六步授权门、capability 交集/衰减/撤销、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态点证据、CFR 扩展批候选向量清单）
