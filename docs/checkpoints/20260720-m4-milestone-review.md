# 20260720 M4 Milestone Review

## 1. 范围回顾

M4 = Intent/Effect 与恢复 + tracer bullet（`docs/plan/DEVELOPMENT-PLAN.md` M4 节：Intent 持久化、OperationDescriptor/准入拒绝矩阵（F-023）、Effect 状态机协议、幂等记录、reconcile/verify、checkpoint、恢复八步（§16.6）、故障注入框架、全 sink fencing 清单（F-014）、七性质模型（IMP-07）；标准 `intent-effect-idempotency.md`）。交付分两批：

- **Lane-KRN M4 批**（PR #12，merge `bae94b4`）：`cognitive_kernel::{executor, effects, recovery}` + store 的 intents/fencing/checkpoints 表与**公开故障注入框架** `cognitive_store::faults`（CrashHarness drop-and-reopen = kill -9 WAL 语义；ScriptedExecutor 外部世界台账）；effect 表全 guard 的 sanctioned 派生收口；engine 记录组装换生成绑定。workspace 147 Rust 测试（M4 新增 16）。
- **Lane-CFR M4 行为执行批**（本评审所在 PR）：runner 七个 effect/recovery 行为门——7 份向量脱 not-run，全部经真实故障注入驱动；state-store-degradation 增 M4 fencing 子集；effect/recovery 反模式自检（corrupted 语料 20→27，全部翻 fail）。

## 2. 验收判据逐条对照

判据 1–7 = DEVELOPMENT-PLAN M4 节验收判据原文；判据 8 = 范围项 F-014/F-023（KRN 任务书第 8 判据）。

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 三个 crash point 全覆盖（`eff-crash-001..003` 行为执行） | **通过** | KRN：`m4_recovery.rs::{crash_point_1_*, crash_point_2_*, crash_point_3_*}` + 跨 crash 重放 digest 相等。CFR 行为向量：EFF-CRASH-001/002/003 → **全部 pass**（crash-point-1/2/3-behavior：CrashHarness 真实注入三个崩溃位；恢复八步顺序钉扎；executor 台账证明恰一次外部执行、原键复用；002 的 EXECUTING→OUTCOME_UNKNOWN→RECONCILED 通道从已提交事件链读回；003 含固定后态漂移负例孪生——拒绝文本点名 `verification_still_current`）；RECOVERY-CRASH-006 → **pass**（三场景聚合，audit 链闭合实测） |
| 2 | unknown outcome 不成功不换键（`effect-unknown-outcome`） | **通过** | KRN：`m4_effects.rs::criterion_2_*`。CFR 行为向量：EFF-UNK-003 → **pass**（unknown-outcome-behavior：opaque 幂等 sink ExecuteThenTimeout → OUTCOME_UNKNOWN → 原键对账 query=Indeterminate → QUARANTINED + EFFECT_OUTCOME_UNKNOWN 实测；台账恰一次外呼恰一键，事件链无成功假报） |
| 3 | 同键异参 `EFFECT_IDEMPOTENCY_CONFLICT` 拒绝（`effect-idempotency-conflict`） | **通过** | KRN：`m4_effects.rs::criterion_3_*`（+同键同参 = 重放正例）。CFR 行为向量：EFF-IDEM-CONFLICT-001 → **pass**（idempotency-conflict-behavior：既有 effect 驱动至向量声明的 EXECUTING，同键异 canonical 参数重铸 → 拒绝 EFFECT_IDEMPOTENCY_CONFLICT；耐久 intent 行 digest 未被改写、既有 effect 状态/版本零变化——reload 实测） |
| 4 | receipt/远端 completed 不完成 Task（`remote-completed-not-acceptance`） | **通过** | KRN：`m4_effects.rs::criterion_4_*`（缺 completion_claim/verification guard 拒，状态不变）+ tracer 负链。CFR：GW-REMOTE-COMPLETE-001 已于 M2 批行为执行 pass（task-acceptance-behavior），M4 协议层由 KRN 测试扩展覆盖 |
| 5 | 恢复顺序错乱注入必须被测试捕获 | **通过** | KRN：kernel `recovery::tests::out_of_order_steps_are_rejected`（先恢复 Loop/跳过 fence 全拒）+ `checkpoint_facts_are_validated`；全部 run_recovery 断言 `step_order == RECOVERY_ORDER`。CFR：AGENT-RECOVERY-003 → **pass**（recovery-reconciliation-behavior：在途 Effect 对账/隔离先于 checkpoint 校验与 loop 恢复；「loop 未对账即恢复」反模式实现被自检判 fail） |
| 6 | sink fencing 矩阵逐端负例（旧 epoch dispatch 在每个提交端被拒，F-014） | **通过 → F-014 闭合** | KRN：`COMMIT_SINKS` 审查常量（4 端）+ `f014_every_commit_sink_fences_stale_epoch_writers`（逐端负例+正例）+ `sink_inventory_is_complete_and_stable`。CFR：state-store-degradation 的 `m4_behavioral_fencing_subset` 真实执行（陈旧 epoch 写在 store 提交 sink 事务内被拒、当前 epoch 可提交）；全部 M4 行为向量在 fencing 贯通的门上执行。台账 F-014 → closed-by-M4 |
| 7 | tracer bullet 端到端竖切（UserIntent→…→acceptance 单节点全链 + 证据） | **通过（复现确认）** | KRN：`m4_tracer_bullet.rs::tracer_bullet_intent_to_acceptance_end_to_end`（正链全程 + 负链：无 PASSED verification → 完成拒；全链重放 digest 稳定、executor 恰一次执行）。CFR 复现确认（本批）：`cargo test -p cognitive-store --test m4_tracer_bullet` 本地再执行通过，证据链工件再生成 `artifacts/evidence/faults/tracer-bullet-evidence.json`（本地再生成件 sha256:b09431a71c1a4fe2bb4a89b7d85d3d91ec081bd458712c7ee50a63beecec62de；键 = chain/generated_by/tracer_bullet；CI 双 OS 每轮重跑该测试） |
| 8 | F-023 准入拒绝矩阵（范围项） | **通过 → F-023 闭合（含拒绝码确认）** | KRN：`admit_operation` 2×2 全组合（kernel `admission_matrix_*`）+ `m4_effects.rs::f023_*`（双否 governed_external 拒绝先于 Intent 铸造，零持久化）。**拒绝码选型确认（本评审，KRN handoff §4.1 提请项）**：`NO_AUTHORIZED_OPERATION_CANDIDATE`（catalog 域，注册描述"无既可见又被授权的候选"）为正确口径——无安全 unknown-outcome 闭合路径的执行器不构成可准入候选；备选 `CATALOG_MATCH_INCONCLUSIVE` 语义为匹配歧义，不取；无需新码（注册面冻结遵守）。台账 F-023 → closed-by-M4 |

**反虚报证据（effect/recovery 自检）**：五种反模式错误实现（crash 后换新键重铸并双发、unknown 盲重发、commit 恢复期重执行外部动作、冲突当去重、未对账即恢复 loop——可行处真实驱动）→ 对应行为向量全部翻 **fail**；合计 **27/27** corrupted 向量翻 fail（CI 断言 ≥27）。

## 3. 安全负例清单（M4 执行）

- 三 crash point 恢复不重复副作用（EFF-CRASH-001..003 + RECOVERY-CRASH-006 行为执行，CrashHarness 注入）；
- unknown 不换键不盲试，不可判 → 隔离（EFF-UNK-003 行为执行）；
- 同键异参拒绝、耐久行不可改写（EFF-IDEM-CONFLICT-001 行为执行）；
- 固定后态漂移 → commit 拒（EFF-CRASH-003 负例孪生）；
- 恢复顺序错乱全拒（KRN 测试 + AGENT-RECOVERY-003 行为执行）；
- 全 sink 陈旧 epoch 拒（KRN f014 + fencing 子集实测）；
- 双否执行器准入拒（KRN f023）；
- tracer bullet 负链（无验证不完成）；
- 五种反模式实现被 runner 判 fail（27/27 总计）。

## 4. 五态分布变化（本批）

81 向量：**pass 39 → 46**（7 份 effect/recovery 向量脱 not-run，全部故障注入驱动）、**not-run 35**；STATE-STORE-DEGRADE-001 保持 not-run 但新增真实执行的 M4 fencing 子集（M1 静态 + M2 只读 + M4 fencing 三层落档；disk-full 分量维持 deferred——无可移植注入手段，F-008 台账口径不变）。fail / not-applicable / documented-degradation 均 0。行为执行向量累计 **19**（M2 3 + M3 9 + M4 7）。

## 5. 漂移与规范变更

无新漂移登记；schema/向量/REQ/错误码表面零变化（60/81 钉扎不动），向量文件零改写。findings-ledger：F-006/F-010 → verified-by-vector（M4 行为侧）；F-014/F-023 → closed-by-M4（本评审判据 6/8 确认，F-023 含拒绝码选型确认）；F-008 补 M4 fencing 子集（disk-full deferred 维持）；F-001 证据推进（行为执行 19 向量 + 内核行为测试 93 项 + tracer bullet 证据链）。

## 6. 指标快照

- 向量五态（实测）：**pass 46 / fail 0 / not-applicable 0 / documented-degradation 0 / not-run 35**；行为执行 19；自检 27/27。
- Rust 测试：workspace 全绿（KRN 147 基线 + CFR 本批集成 9 项）；clippy -D warnings / fmt 绿。
- 开放 P0：0（合同缺口类）；F-001 持续消解。开放 P1：**F-011（M5）、F-017（M6）、F-015（持续）**——F-014/F-023 本评审闭合。开放漂移：0。
- tracer bullet 证据链：本地复现通过（判据 7）；CI 双 OS 每轮重跑。

## 7. 结论与 M5 入口 gate 判定

**GO 判定：M4 done**（八判据全过，F-014/F-023 闭合，tracer bullet 复现确认）。

**M5 入口 gate 如实判定**：DEVELOPMENT-PLAN M5 入口 = M4 出口（tracer bullet 证据在案）+ **F-011 R1 最低集机器合同登记完成**。本评审达成 **M4 分量**；**F-011 R1 审批合同登记（approval schema 硬化 + 向量）为 M5 入口剩余项**，归 Lane-CTR 后续批（建议范围：R1 聊天内结构化确认最低集 = approval-request/approval-decision 机器 schema + 负例向量 + registry 映射，见 F-011 台账与 IMP-05）。遗留条件（不阻断该 CTR 批）：

1. F-008 disk-full 分量 deferred（可移植注入手段出现或 M6 平台矩阵时复议）；management stop/revoke 期望挂 M5。
2. DISC-DELTA-SCOPE-003 与 shell/management/harness 类向量（35 份 not-run 主体）挂 M5+；F-004 运行时 admission 决策挂 M5。
3. 恢复步骤 6/7 的跨 activity 完整重授权编排 = M5 运行时（KRN handoff §2）；M5 后建议 CFR 在 crash-recovery 向量复跑时覆盖该分支。
4. CTR 侧待办同批候选：D-016（management 操作名，v0.2 defer 维持）、membership 生成绑定（M5 组装器需要时）、D-018 事件 envelope 升格实施（M5 Lane-RUN 治理发布边界，PR #11 已裁决路径）。
