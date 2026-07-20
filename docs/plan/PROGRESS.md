# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-CFR M2 行为执行批：runner 行为模式 + M2 出口评审）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | not-started | — | 入口 gate：M2 出口（已达成）+ F-007 行为侧测试计划评审（= M3 启动会话第一动作，KRN handoff §5 已定义入口） |
| M4 Intent/Effect 与恢复 + tracer bullet | not-started | — | 入口 gate：M1 出口（已达成）+ M2/M3 行为验收 + F-014/F-023 排入 |
| M5 意图链/Harness/Shell/管理面 | not-started | — | 入口另需 F-011 R1 合同登记 |
| M6 安装与适配、v0.1 发布 | not-started | — | F-017 平台矩阵为出口阻断 |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | macOS/Linux 产品切片已记录；iOS/Android 设计提示词已提供但尚未执行；implementation 未启动 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **60**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **8**（matrix impl/impl_tests 已回填：REQ-CONF-001/002/003（M1 CFR）+ REQ-STATE-002/003、REQ-EVT-004、REQ-REC-003、REQ-GOBJ-ID-001（M2 KRN，附 Rust 行为测试证据）；其余待各车道证据口径回填） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 3 向量**（STATE-CAS-002 / EFFECT-STATE-CLOSURE-008 / GW-REMOTE-COMPLETE-001，被测 = 真实 `cognitive-kernel`+`cognitive-store` 权威路径；另 state-store-degradation M2 只读子集真实执行落档）+ 内核 Rust 行为测试 51 项（KRN）；静态合同层：28/81 向量 pass；**均不构成 Profile 覆盖声明**（conformance-evidence §2）；TS 客户端 75 项包内单元测试为实现测试，不计向量执行 |
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
| 漂移 | 0 开放（+1 deferred） | D-013/D-014/D-015（AKP 信封/控制 payload/流帧机器 schema 缺口，Lane-CTR 缺口批登记并闭合）；**D-016 deferred-to-v0.2**（management 操作名登记需新增规范面，M5 实现反馈驱动）；D-004、D-012（M1 Lane-CFR）、D-001/D-006/D-011（M1 Lane-CTR）与 D-002/D-003/D-005/D-007~D-010 均已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | 缺口批已合并（PR #5） | `lane/ctr` | TSC 7 项缺口收敛：AKP 信封 ×2/流帧/shell-control 机器 schema + 5 份负例向量（D-013/D-014/D-015 闭合）、codegen 0.2.0（28 模块、错误注册表绑定、digest 常量）、⑦ deferred-to-v0.2（D-016）；sdk-ts 换绑定就绪（归 Lane-TSC） |
| Lane-CFR 符合性与工具 | **M2 行为执行批已交付**（本页所在 PR） | `lane/cfr` | 已完成：runner 行为执行模式（被测 = `cognitive-kernel`+`cognitive-store` 真实权威路径；3 向量脱静态/not-run → 行为 pass；state-store-degradation M2 只读子集真实执行落档）、行为侧错误实现自检（gate-bypassing 直写 store → 3 向量翻 fail，合计 12/12）、钉扎同批调整（ci.yml 81/31/0/0/0/50 + self-check ≥12、runner_execution.rs 7 测试）、M2 出口评审。**触碰通告**：`crates/cognitive-conformance/**`（+behavior 模块，Cargo 依赖 +kernel/store/domain/tempfile）、`.github/workflows/ci.yml`（钉扎计数）、`conformance/README.md`（Running 节行为模式）、`Cargo.lock` |
| Lane-KRN 内核主线 | M2 内核批已合并（PR #4）；**M3 入口 gate 的 M2 分量已达成** | `lane/krn` | M3 治理链与 Context（`docs/prompts/milestone-m3.md` / `lane-krn.md`；第一动作 = F-007 行为侧测试计划：先写「capability 交集只缩不扩」与「撤销后缓存复用被拒」失败测试） |
| Lane-TSC TS 客户端 | 客户端骨架已合并（PR #2）：sdk-ts AKP envelope 编解码/双通道隔离客户端/registry 驱动重试/snapshot+cursor watch 消费器（`WATCH_CURSOR_STALE`→重新快照）/注入式传输层 + agent-shell 会话层（preview/submit/attach/detach/cancel，detach≠cancel、展示只读投影）；75 项 TS 单元测试通过（sdk-ts 63 / agent-shell 12）；向量状态无虚报（当时全 not-run，现行 25/51 见上表）；发现 7 项契约缺口待 Lane-CTR（清单见 handoff §4）；只触碰 `packages/sdk-ts`、`apps/agent-shell`、`pnpm-lock.yaml` 与本页/handoff | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | 维护依赖表与 informative 平台产品设计；iOS/Android 提示词待执行；实现 gate 未通过 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-lane-cfr-m2-handoff.md](../checkpoints/20260720-lane-cfr-m2-handoff.md)（Lane-CFR M2 行为执行批：runner 行为模式、3 向量行为 pass、降级子集落档、行为自检、M2 出口评审）
2. [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md)（M2 出口评审：5 判据 + 预算范围项逐条证据，GO → M3）
3. [20260720-lane-ctr-gaps-handoff.md](../checkpoints/20260720-lane-ctr-gaps-handoff.md)（Lane-CTR 缺口批：TSC 7 项逐项终态、AKP wire schema ×4、codegen 0.2.0、D-013~D-016）
