# 20260720 Lane-KRN Handoff（M4 Intent/Effect 与恢复 + tracer bullet 批）

## 1. 本次会话完成

按 `docs/prompts/milestone-m4.md` / `docs/prompts/lane-krn.md` 交付 M4（分支 `lane/krn`，基线 `fd63685` = CFR M3 行为批 merge）。零新 Cargo 依赖；提交哈希见 §6。

- **`cognitive-kernel::executor`（执行器端口）**：`EffectExecutor` trait（dispatch/query_outcome/capabilities 自描述）；`ExecutorCall` 按 REQ-EFF-002 携带稳定幂等键 + canonical 参数 digest + 授权摘要 + fencing epoch；`DispatchOutcome`（Executed/NotExecuted/**Unknown 一等结局**/FencedStaleEpoch）。
- **`cognitive-kernel::effects`（协议驱动，IMP-07 七性质模型宿主）**：
  - **Intent 铸造** `mint_intent`（REQ-EFF-001/002）：F-023 准入前置 → canonical 参数 digest（比较基准，非源字节）→ 幂等仲裁对**耐久行**（同键同 digest = 重放既有 Intent 不新铸；同键异 digest = `EFFECT_IDEMPOTENCY_CONFLICT` 不去重不执行不改写）→ intent 行 + 事件一事务落库（`intents` 表 UNIQUE(idempotency_key) 为结构性兜底，append-only 触发器）。
  - **F-023 准入矩阵** `admit_operation`：`OperationDescriptor`（能力自描述，≠ AuthorizationCapability，分型分检、持有 descriptor 零授权）2×2 判定——queryable→query-reconcile、仅 idempotent→idempotent-redispatch、双否且 governed_external/emergency→`NO_AUTHORIZED_OPERATION_CANDIDATE` 拒绝（无安全 unknown 闭合路径），pure/local 无外部承诺。
  - **effect 表全 guard 的 sanctioned 派生收口**（M3 handoff §4.4 防漂移项完成）：`intent_durably_persisted`（耐久重读）、`fencing_epoch_current`（lease vs store）、`idempotency_binding_valid`（调用字段 == 耐久 intent）、`authorization_current`/`capability_and_revocation_current`（M3 revalidation）、`verification_still_current`/`expected_state_version_matches`（subject 权威重读比对固定后态版本）、`commit_authority_matches`、`compensation_is_separate_governed_effect` + `independent_authorization_granted`（原 grant 复用先于门直接拒 `CONTEXT_AUTH_DENIED`）、reconciliation 三值 guard（查询结果驱动）。协议方法：authorize/dispatch（先落 EXECUTING 记录后外呼）/record_outcome/reconcile（原键查询）/close_not_executed/quarantine_still_unknown（表面 `EFFECT_OUTCOME_UNKNOWN`）/verify/commit/begin_compensation。
  - **F-014 sink fencing 矩阵**：`COMMIT_SINKS` 审查常量（4 端）——①外部 executor（sink 自身拒陈旧 epoch + 驱动预检）②authority-store 事务提交 ③准入+outbox ④checkpoint 写入（②③④ = store 事务内 `verify_fencing_in_tx`，陈旧 → Conflict 整体回滚）；`WriterLease`/`acquire_lease`；kernel 端口结构新增 `fencing_epoch: Option<i64>`（None = M2 兼容路径）。
- **`cognitive-kernel::recovery`（恢复八步，白皮书 §16.6 顺序）**：`RecoverySequencer`（乱序步骤运行时拒绝——顺序违反 = 显式错误非静默重排）；`run_recovery`：barrier → epoch 推进+新 lease → 旧 writer 判定 fenced → 重放已提交历史（零重执行）→ 在途 Effect 对账（AUTHORIZED：确认无 dispatch 记录 → 原键单次重发盘位；EXECUTING：→OUTCOME_UNKNOWN→原键查询对账；不可判 → QUARANTINED 携 `EFFECT_RECOVERY_QUARANTINED`）→ 重授权 → 重解析 Context → checkpoint 事实校验（epoch 早于新 epoch、watermark 不超重放历史）后 loop 可恢复。`RecoveryReport` 全程留证。
- **`cognitive-store` M4 扩展**：`intents`/`fencing`/`checkpoints` 表（append-only 触发器 + epoch 单行 CHECK）；`ProtocolStore` 端口实现（insert_intent 一事务、epoch 读/推进、按域按状态枚举、checkpoint 追加含事务内 epoch 校验）；**`faults` 公开模块（故障注入框架，供 CFR 复用）**：`CrashHarness`（drop-and-reopen WAL = kill -9 语义：只有已提交事务存活）+ `ScriptedExecutor`（记录全部 dispatch/query、可编排 Execute/Refuse/ExecuteThenTimeout/VanishWithoutExecution、sink 侧 fencing、幂等吸收）。
- **`engine.rs` 换生成绑定**（M3 handoff §2 登记项完成）：记录组装由手写 `json!` 换 `cognitive_contracts::generated::state_transition_record::CommittedStateTransitionRecord`（+ request 模块 Reason/Causation），成员集与 canonical 字节不变（M2/M3 全部行为测试与重放 digest 复验通过）。
- **文档联动**：matrix 回填 10 REQ；ledger F-006/F-010（M4 内核行为证据）、F-014/F-023（实现+测试已交付，闭合待 M4 出口评审）、IMP-07（七性质模型交付口径）；PROGRESS；本 handoff。

## 2. 未完成 / 进行中

- **M4 出口评审未做（有意）**：等 CFR effect-recovery 向量执行批（§5 候选清单）+ tracer bullet 判据核验后由协调者安排；本批口径 =「实现已提供 + Rust 行为测试已执行」，向量计数保持 81/39/42 未动（本地复跑 runner + self-check 20/20 确认）。
- 进程级 kill 注入：CrashHarness 用 drop-and-reopen 模拟（WAL 语义等价——未提交事务丢失、已提交存活）；真 `kill -9` 子进程编排可在 CFR 向量批按需加壳（框架接口不变）。disk-full 注入仍缺可移植手段（M2 只读降级 + 事件冲突注入在案；state-store-degradation 完整行为侧的 disk-full 分量维持 deferred，F-008 台账口径不变）。
- 恢复第 6/7 步（重授权/重解析 Context）在单节点参考实现中为步骤事实记录 + M3 组件的结构性保证（陈旧 grant/缓存键在新 epoch 不可用）；跨 activity 的完整重授权编排 = M5 运行时。
- Loop 状态机的运行时驱动（OODA 迭代器）= M5；本批只交付 checkpoint 持久化 + 恢复侧校验（REQ-RUN-006 部分，matrix note 已限定）。

## 3. 测试与证据状态（判据 ↔ 测试对照）

- **Rust（本地全绿 + 待 CI 复核）**：build/test/clippy -D warnings/fmt --check 全绿；workspace **147** 测试（M3 基线 131 + M4 新增 **16**：kernel 单元 5 = effects 3 + recovery 2；store 集成 11 = m4_effects 6 + m4_recovery 4 + m4_tracer_bullet 1）。
- 判据对照（编号按协调者任务书）：
  1. 三 crash point → `m4_recovery.rs::{crash_point_1_recovers_to_single_dispatch_with_the_original_key, crash_point_2_reconciles_never_blind_retries, crash_point_3_commits_from_evidence_without_reexecution}`（+ `recovery_replays_committed_history_and_validates_checkpoints`：跨 crash 重放 digest 相等；executor 台账断言无重复副作用；crash 3 负例 = 固定后态漂移 → commit 拒）；
  2. unknown 不成功不换键 → `m4_effects.rs::criterion_2_unknown_outcome_reuses_the_key_and_quarantines_when_unresolvable`（对账绑定原键、单次外呼、不可判 → QUARANTINED + `EFFECT_OUTCOME_UNKNOWN`）；
  3. 同键异参 → `m4_effects.rs::criterion_3_same_key_different_parameters_is_rejected`（`EFFECT_IDEMPOTENCY_CONFLICT`、既有 intent 不变、零新建；同键同 canonical 参数 = 重放）；
  4. receipt/远端 completed/模型自述不完成 Task → `m4_effects.rs::criterion_4_receipt_and_remote_completed_do_not_complete_the_task`（缺 completion_claim 证据拒 + 缺 verification guard 拒，状态不变）+ tracer 负链；
  5. 补偿独立授权 → `m4_effects.rs::criterion_5_compensation_requires_independent_authorization`（原 grant 拒 `CONTEXT_AUTH_DENIED`、新 grant + 新 intent/键 → COMPENSATING）；
  6. 恢复八步顺序 → kernel `recovery::tests::out_of_order_steps_are_rejected`（先恢复 Loop/跳过 fence 等全拒）+ `checkpoint_facts_are_validated` + 全部 run_recovery 调用断言 `step_order == RECOVERY_ORDER`；
  7. tracer bullet → `m4_tracer_bullet.rs::tracer_bullet_intent_to_acceptance_end_to_end`（正链：UserIntent 钉扎 → M3 授权 → task 至 ACTIVE → intent 铸造 → effect authorize/dispatch/EXECUTED/RECONCILED → verification 对象自身状态机 NOT_REQUESTED→…→PASSED → effect VERIFIED→COMMITTED → task CANDIDATE_COMPLETE→COMPLETED 由 acceptance authority guard 从权威状态重读派生；负链：无 PASSED verification → 完成拒、状态停 CANDIDATE_COMPLETE；全链重放 digest 稳定、executor 恰一次执行；evidence 落 `artifacts/evidence/faults/tracer-bullet-evidence.json`，再生成：`cargo test -p cognitive-store --test m4_tracer_bullet`）；
  8. F-014/F-023 行为侧 → `m4_effects.rs::{f014_every_commit_sink_fences_stale_epoch_writers, f023_unqueryable_nonidempotent_operation_cannot_mint_an_intent}` + kernel `effects::tests::admission_matrix_*`（2×2 全组合）+ `sink_inventory_is_complete_and_stable`（矩阵常量钉扎）。
- IMP-07 七性质对照 = `cognitive_kernel::effects` 模块文档（性质↔强制点↔测试逐条）；本 §3 即 handoff 侧映射。
- **向量**：0 变化（81 / 39 pass / 42 not-run 保持；self-check 20/20；conformance 仅 behavior.rs 3 处 `fencing_epoch: None` 机械补丁，runner 行为与计数不变）。
- TS/工具：pnpm -r build/test 绿；check:consistency OK（273/55/60/81）；gen-matrix --check 无 drift。
- CI：push + PR 后 Windows+Linux 全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移（含交 Lane-CTR/CFR 事项）

无新漂移登记。口径与缺口：

1. **F-023 拒绝码选型**：准入矩阵拒绝用注册码 `NO_AUTHORIZED_OPERATION_CANDIDATE`（catalog 域，"无既可见又被授权的候选"——双否执行器无安全闭合 ⇒ 非可准入候选）。备选曾考虑 `CATALOG_MATCH_INCONCLUSIVE`（语义偏匹配歧义，不取）。提请 CTR/CFR 在 M4 出口评审确认口径；若改判为需新码则属注册面冻结外议题。
2. **kernel 端口结构演进**：`TransitionCommand`/`AdmitCommand`/`TransitionCommit`/`ObjectAdmission` 新增 `fencing_epoch: Option<i64>`（None = M2 兼容）；conformance `behavior.rs` 3 处构造点由本批机械补齐（跨车道触碰通告，行为零变化）——CFR 下批接手时知悉即可。
3. **恢复步骤 6/7 的深度**（§2 已述）：单节点下为结构性保证 + 步骤事实；M5 运行时补全编排后建议 CFR 在 crash-recovery 向量执行时覆盖跨 activity 重授权分支。
4. **eff-crash-002 的「executor_query_result: executed_with_original_key」** 语义由 `ScriptedExecutor` 台账实现（真实外部系统的 query 端点 = M5/M6 适配器职责）；CFR 向量执行可直接复用 `cognitive_store::faults`。
5. 本机工具链照旧（llvm-mingw CC/AR 绝对路径 + dlltool-shim 前置；`TMP=D:\tmp`；磁盘 ≥5GB）。

## 5. 下一步入口

- **CFR M4 行为执行批（候选清单，均已有内核行为孪生 + faults 框架可复用）**：
  - `eff-crash-001/002/003`（EFF-CRASH-001..003）：CrashHarness + ScriptedExecutor 直接驱动，期望字段逐项可断言（recovered_effect_state/idempotency_key_reused/duplicate_effect/blind_retry/audit_chain_closed——audit 链 = intent 事件 + dispatch/outcome/reconcile 记录序列，reload 实测）；
  - `crash-recovery`（RECOVERY-CRASH-006）：三场景聚合 = 三个 crash point 测试的向量化重跑；
  - `effect-unknown-outcome`（EFF-UNK-003）：quarantine 路径 + `EFFECT_OUTCOME_UNKNOWN` 已实测；
  - `effect-idempotency-conflict`（EFF-IDEM-CONFLICT-001）：mint_intent 拒绝路径逐字段可断言；
  - `state-store-degradation`：M2 只读子集 + M4 fencing 分量可扩展；disk-full 分量维持 deferred（如实 partial）；
  - `agent-recovery-reconciliation` / `effect-state-closure-008` 复核：run_recovery 报告面可对 expected 断言。
  - 注意（CFR M3 handoff §4 提醒的对应落实）：effect-recovery 向量的结构事实字段须 reload 实测——`m4_*` 套件的断言模式即模板。
- **Lane-KRN M5**（待 M4 出口评审 + F-011 R1 合同登记）：意图链/Harness/Shell/管理面（`lane-krn.md` 范围表将由协调者派发；kernel 侧剩余 = UserIntentRecord/interpretation 绑定、运行时 Loop 驱动、admission 编排）。
- **Lane-CTR**：M3 handoff §4 两项（渲染域并入 D-017、membership 绑定）+ 本批 §4.1（F-023 码选口径确认）。
- 工作分支：`lane/krn`。

## 6. 快照

- PROGRESS 已更新：是（M4 行、实现计数 24→34、行为测试 93、车道表、handoff 列表）。
- 本次提交（按序，哈希见 git log）：kernel M4 协议批（executor/effects/recovery/error/ports/replay + engine：fencing 字段贯通 **及** 记录组装换生成绑定——换装与字段同文件故并批，字节不变性由 runner 报告 digest 换装前后一致复验）→ store 协议批（sqlite 扩展 + faults 框架）→ M4 测试批（m4_common/m4_effects/m4_recovery/m4_tracer_bullet + M2/M3 构造点补丁 + conformance behavior.rs 机械补丁）→ docs 联动批（matrix/ledger/PROGRESS/handoff）。
