# PROGRESS — 单页进度仪表

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-20（Lane-TSC 换绑批：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定，删除全部临时手工形状与临时 pin）

## 里程碑状态

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **impl 批已交付**（出口评审待 CFR 行为向量执行） | — | Lane-KRN 已交付三 crate 实现 + 六验收判据 Rust 行为测试（[20260720-lane-krn-m2-handoff.md](../checkpoints/20260720-lane-krn-m2-handoff.md)）；向量保持 not-run 诚实口径，行为执行批归 Lane-CFR，之后做 M2 出口评审 |
| M3 治理链与 Context | not-started | — | 依赖 M2 |
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
| 测试已执行（行为层，runner 真实执行并留证据） | **0（行为）**；静态合同层：**30/81** 向量 pass（traceability/schema/CAS/迁移表/性能合同/信任面静态门 + AKP 信封/流帧/控制 payload schema 负例，逐条 grounding+evidence；**不构成行为覆盖**，见 conformance-evidence §2）；TS 客户端 79 项包内单元测试为实现测试，不计向量执行 |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-20 Lane-CTR 缺口批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **81** |
| **pass（静态合同执行）** | **30**（wire-schema 9：GOBJ 负例 ×2 + spec-coverage + perf 合同 + AKP 信封/流帧/控制 payload 负例 ×5（D-013/D-014/D-015）；contract-traceability 18；state-machine 1（CAS）；shell-intent 1（effect-state-closure-008 表驱动）；security-negative 1（prompt-injection 信任面静态合同）） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run（需内核/运行时行为，逐条理由入报告）** | **51**（含 state-store-degradation：静态合同侧断言已随报告落档，F-008） |
| 错误实现自检 | **11/11 corrupted 向量全部翻 fail**（`--self-check`，conformance-evidence §3；CI 步骤断言） |

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
| Lane-CTR 契约与生成 | **缺口批已交付**（本页所在 PR） | `lane/ctr` | TSC 7 项缺口收敛：①④⑥ 登记 AKP 信封 ×2/流帧/shell-control 机器 schema + 5 份 schema-gate 负例向量（D-013/D-014/D-015 闭合）；②③⑤ codegen 0.2.0（errors.yaml 错误注册表双语言绑定、SCHEMA_ID/SCHEMA_DIGEST 运行时常量 + SCHEMA_DIGESTS 聚合、Shell/AKP 族入生成集 = 28 schema 模块）；⑦ deferred-to-v0.2（D-016）。**触碰通告**：钉扎计数同批调整（ci.yml 81/30/0/0/0/51 + self-check ≥11、`crates/cognitive-conformance/tests/runner_execution.rs`、`tools/static_check.py` 60/81）；`specs/akp/README.md` §3/§7/§8 增机器 schema 指针；sdk-ts 可换绑定（views.ts/envelope.ts/errors.ts/watch.ts 替换点就绪，归 Lane-TSC） |
| Lane-CFR 符合性与工具 | **M1 runner 批已交付**（本页所在 PR） | `lane/cfr` | 已完成：runner 静态合同执行（25 pass/51 not-run 逐条理由）、错误实现自检（6/6 fail + CI 步骤）、F-003 关闭 gate、M1 复验 8 项逐条落档、D-004/D-012 闭合、validate-manifest `$id` 残留兼容层移除、CI 断言演进（诚实性门 + 钉扎计数）、M1 出口评审。**触碰通告**：`crates/cognitive-conformance/**`（执行引擎重构）、`.github/workflows/ci.yml`（runner 断言 + self-check 步骤）、`tools/src/validate-manifest.mjs`、`conformance/README.md`（Running/层映射）、4 份向量 `input.owner_spec`（D-012） |
| Lane-KRN 内核主线 | **M2 内核批已交付**（本页所在 PR） | `lane/krn` | 已完成：`cognitive-domain`（五迁移表嵌入消费 + digest 钉扎 + newtype）、`cognitive-kernel`（集中 transition 入口/CAS/guard/evidence/硬预算/重放投影/注册错误码单点映射）、`cognitive-store`（SQLite WAL 五绑定规则、append-only 触发器、原子提交、outbox、UUIDv7/时钟适配器）；M2 六判据各有行为测试（真并发 CAS、五表非法边穷举、重放 digest、UPDATE/DELETE 负例、预算 fail-closed、事务中断注入）。**触碰通告**：根 `Cargo.toml`（workspace 依赖 rusqlite/uuid/tempfile）、`Cargo.lock`、`cognitive-kernel::KERNEL_PORTS` 常量语义更新（端口能力面，runtime/management 占位断言兼容未动）、matrix 5 REQ impl 回填。M3 待 M2 出口评审 |
| Lane-TSC TS 客户端 | **换绑批已交付**（本页所在 PR）：sdk-ts/agent-shell 全量换用 codegen 0.2.0 生成绑定——errors.ts 消费 `errorRegistry`（删手写 55 码表 + 测试时 YAML 对读）、envelope.ts 消费 `akpRequestEnvelope`/`akpResultEnvelope`（删手工信封接口；新增 payload⊕payload_ref 与 partial⇒continuation 门）、views.ts 消费 shell 族 6 生成模块 + `SCHEMA_DIGESTS`（删 5 手工接口/`SHELL_SCHEMA_DIGESTS`/`CancelControl`/`SHELL_CONTROL_PROVISIONAL_PIN` 及 digest 重derive 漂移门）、watch.ts 消费 `akpStreamFrame` 且流错误码收口 `error.code`（D-015 行为适配 + 旧形状负例）；语义负例全部保持通过；**79 项 TS 客户端单元测试**（sdk-ts 67 / agent-shell 12），仍为实现测试、不计向量执行；剩余临时机制清单见 handoff §2/§4 | `lane/tsc` | M5 集成（真 kernel-server HTTP+SSE 对接）待 Lane-RUN gate |
| Lane-RUN 运行时与管理面 | 阻塞于 M4 | `lane/run` | — |
| Lane-DOC 文档维护 | 持续 | 随各车道 PR | — |
| Lane-CON Console | tracking-only 文档例外 | — | 维护依赖表与 informative 平台产品设计；iOS/Android 提示词待执行；实现 gate 未通过 |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260720-lane-tsc-bindings-handoff.md](../checkpoints/20260720-lane-tsc-bindings-handoff.md)（Lane-TSC 换绑批：四模块换生成绑定、删除临时机制清单、D-015 行为适配、剩余 M5 前待办）
2. [20260720-lane-ctr-gaps-handoff.md](../checkpoints/20260720-lane-ctr-gaps-handoff.md)（Lane-CTR 缺口批：TSC 7 项逐项终态、AKP wire schema ×4、codegen 0.2.0、D-013~D-016）
3. [20260720-lane-krn-m2-handoff.md](../checkpoints/20260720-lane-krn-m2-handoff.md)（Lane-KRN M2 内核批：domain/kernel/store 实现、六判据行为测试、契约缺口清单交 Lane-CTR、CFR 行为执行批入口）
