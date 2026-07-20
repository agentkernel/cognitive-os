# 20260720 Lane-KRN Handoff（M5 kernel 侧批：意图链 / epoch fencing / Loop 端口 / 恢复 6-7 / D-018 协作项）

## 1. 本次会话完成

按协调者 M5 kernel 侧批任务书交付（分支 `lane/krn`，基线 `92dea7f` = clients 集成批 merge）。零新 Cargo 依赖；提交哈希见 §6。全程测试先行（行为测试与实现同批，红→绿）。

- **意图链持久化 + 确定性准入**（REQ-INTENT-RECORD-001 / REQ-INTENT-ADMISSION-001；`cognitive-kernel::intent_chain` 新模块 + store 三张 append-only 表）：
  - `record_user_intent`：UserIntentRecord 在**任何语义解释前**耐久固定（`user_intent_records` 表，append-only 触发器 + 事件同事务；对未固定 record 的解释调用直接拒绝——顺序是结构性的不是约定）。canonical_json = 已登记 `user-intent-record.schema.json` 形状，经 **contracts 生成绑定**组装（`generated::user_intent_record::UserIntentRecord`），header content digest 按 REQ-GOBJ-REF-004 默认投影（排除 `/header/content_digest`）密封，digest 域沿用 `governed-object-content/0.1`。摘要/模型输出/后续修正永不覆盖原记录（重复 record_id = Conflict；raw SQL UPDATE/DELETE 被触发器 ABORT）。
  - `InterpretationCandidate` = **概率组件输出类型**（类型名含 candidate 语义），`record_interpretation_candidate` 持久化为不可变候选行（`intent_interpretations` 表）；行内 `recorded_status` 由 material-ambiguity 事实**确定性派生**（已登记 schema 条件：任一 material 歧义 ⇒ `clarification_required`），模型不选状态。
  - `admit_interpretation` = **确定性 admission 门**：material 歧义 → `INTENT_CLARIFICATION_REQUIRED`（绝不 top-1）；接受者 ≠ record 登记的 intent authority → `CONTEXT_AUTH_DENIED`（agent 自述"用户已同意"过不了比较）；acceptance digest ≠ 持久化候选 digest → `STATE_CONFLICT`（authority 接受的是它审阅过的字节）。通过才产 `AdmittedInterpretation` 令牌——**没有从裸候选到 TaskContract 的 API 路径**。
  - `mint_task_contract`：消费令牌铸造 TaskContract（`task_contracts` 表；canonical_json = 生成绑定 `generated::task_contract::TaskContract` 形状，绑定 user_intent_ref / intent_interpretation_ref / intent_acceptance_ref 三个 strong ref）；无 acceptance 条件的合同拒绝（REQ-RUN-004 可判定出口）；**contract epoch 按 task_ref 单调，CAS 在 store 事务内**（expected N → 插入 N+1，竞争者整体回滚）。
- **用户修正 epoch 推进 + 旧 dispatch fencing**（REQ-INTENT-SUPERSEDE-001；RFC-0001 REQ-SHELL-CORRECTION-001 / REQ-AKP-INTENT-001 的 kernel 承载面）：
  - `supersede_task_contract`：修正 = 新 record + 新候选（必须 `supersedes` 指向被取代解释）+ **同一 admission 门** + epoch N+1 合同（CAS）；旧行零改写。报告分类旧 epoch 未决工作（按权威 effect 状态确定性映射 RFC 五类：PROPOSED/AUTHORIZED→safely-cancelled、EXECUTING/OUTCOME_UNKNOWN→**must-reconcile**、EXECUTED/RECONCILED/VERIFIED→must-complete、VERIFY_FAILED/COMPENSATING→compensate、QUARANTINED→quarantine）。
  - `IntentCommand`/`IntentRow` 新增 `task_binding: Option<TaskBinding{task_ref, contract_epoch}>`（None = pre-M5 未绑定路径）；**mint_intent 与 dispatch_effect 双点 fencing**：绑定 epoch < 当前 → `INTENT_VERSION_SUPERSEDED`，在任何 transition commit 与外呼**之前**拒绝——intent-supersede-002 期望语义（old_epoch_new_dispatch_rejected、零执行、pending reconcile_before_continue）全部有行为测试。
- **有界 Loop 驱动 kernel 端口**（REQ-RUN-004/005/007/008 kernel 分量；`cognitive-kernel::harness` 新模块 + `loop_progress_facts` 表）：`LoopDriver`（start_loop / begin_iteration / record_progress / stagnation / retry_count / admit_retry / end_iteration / contract_facts），端口面冻结详单见 **§7**。OODA 相位编排本体留给 Lane-RUN。
- **恢复步骤 6/7 跨 activity 编排的 kernel 承载**（M4 handoff §2 遗留；REQ-REC-001 延伸）：`RecoveryReport` 新增 **步骤 6 事实** `reauthorization_obligations: Vec<ReauthorizationObligation{effect_object_id, idempotency_key, grant_epoch, capability_set_version}>`（非终态 continuation 的耐久授权绑定清单）与 **步骤 7 事实** `context_rebinding: ContextRebinding{fenced_epoch, new_epoch}`；新增 `reauthorization_satisfied()` 确定性判定（RUN 调用）。行为测试补强：旧 grant 在推进后的治理事实下过不了 dispatch 的 `capability_and_revocation_current` guard（零外呼），新 grant 以**原幂等键**恰一次重发；pre-crash 缓存绑定被 `serve_declared` 拒绝并按键清除（含派生缓存），`lookup_current` 只能 miss。
- **D-018 KRN 协作项评估（结论已回写台账）**：outbox 行**无需**新增列；最小补充 = `ProtocolStore::load_event_by_id`（outbox event_id → 已提交事件值，M5 RUN envelope 组装器免扫日志），行为测试在案。
- **文档联动**：matrix 回填 7 REQ + 4 行 note 补强；ledger D-018 行 ⑥ 协作项交付；PROGRESS；本 handoff。

## 2. 未完成 / 进行中

- **intent-interpretation 生成绑定缺失**：codegen CORE_SET 无 `intent-interpretation` 模块，本批候选 canonical 值按 M4 前例手工组装（schema 形状对齐、有 round-trip 测试意图但非 deny_unknown_fields 强制）。**给 Lane-CTR 的请求见 §4.1**；绑定交付后按 M4 engine.rs 换绑先例做机械换装。
- Loop 的 OODA 运行时编排（观察/定向/决策相位驱动、stagnation 事实驱动 STAGNATION_DETECTED/ESCALATE 边）= Lane-RUN M5 批 2；kernel 端口面已冻结（§7）。
- 恢复 6/7 的**跨 activity 完整重授权编排**（对 obligations 逐条重发起 M3 六步授权、重建 AuthzSnapshot）= Lane-RUN M5；kernel 侧已交付事实面 + 判定函数 + guard 强制。
- 向量保持 not-run（84/46/38 未动；执行归 Lane-CFR，候选清单见 §5）。

## 3. 测试与证据状态

- **Rust（本地全绿 + 待 CI 复核）**：`cargo fmt --check` / `check --workspace` / `test --workspace` / `clippy --workspace --all-targets -D warnings` 全绿；workspace **166** 测试（基线 150 + M5 新增 **16**：kernel 单元 3 = intent_chain 2 + harness 1；store 集成 13 = m5_intent_chain 5 + m5_harness 5 + m5_recovery_governance 3）。
- 判据对照（任务书原子任务 ↔ 测试）：
  1. UserIntentRecord 固定 → `m5_intent_chain.rs::user_intent_record_is_fixed_first_and_never_overwritten`（先于解释、生成绑定 round-trip、重复拒、raw SQL 触发器 ABORT、scope 查询）；
  2. candidate 准入隔离 → `material_ambiguity_forces_clarification_not_top1`（确定性状态派生 + `INTENT_CLARIFICATION_REQUIRED` + 零合同零 intent）+ `acceptance_is_authority_and_digest_bound_then_mints_the_contract`（非 authority 拒、错 digest 拒、无 acceptance 条件拒、epoch CAS 单调、生成绑定 round-trip）；
  3. 修正 fencing（intent-supersede-002 语义）→ `user_correction_advances_epoch_and_fences_old_dispatch`（epoch 1→2；EXECUTING→must-reconcile、AUTHORIZED→safely-cancelled；旧 epoch 新 mint 拒 `INTENT_VERSION_SUPERSEDED` 零持久化；已铸旧 intent 的 dispatch 拒同码**零外呼**（executor 台账断言）、effect 停 AUTHORIZED；原记录/旧解释/epoch-1 合同逐字节未动；stale epoch 二次 supersede 输 CAS）；
  4. Loop 端口 → `m5_harness.rs` 5 测试（无合同拒、预算耗尽 guard 拒、无 checkpoint 拒、迭代单调、max_iterations 上限 = `RESOURCE_BUDGET_EXHAUSTED` 且零 debit 零迁移、超预算 charge 整体回滚、advanced 无证据不可记录、停滞/重试算术、fenced writer 无法投毒计数、COMPLETED task 拒继续迭代）；
  5. 恢复 6/7 → `m5_recovery_governance.rs::recovery_reports_reauthorization_obligations_and_old_grants_cannot_continue`（报告事实 + 旧 grant guard 拒零外呼 + 新 grant 原键恰一次）+ `stale_context_bindings_are_refused_and_purged_after_recovery`；
  6. D-018 → `outbox_rows_resolve_to_committed_events_for_envelope_assembly`（含未知 id → None fail-closed）。
  7. replay 完整性 → `chain_events_fold_as_provenance_in_replay`（三种链事件 provenance 折叠、对象投影不动、digest 字节稳定）。
- **向量**：0 变化（**84 枚举 / 46 pass / 0 fail / 38 not-run** 本地复跑确认；self-check **27/27** corrupted 全翻 fail）；conformance 仅 behavior_m4.rs 1 处 `task_binding: None` 机械补丁（提交信息注明，行为零变化）。
- TS/工具：`pnpm -r build`/`test` 绿（contracts-ts 35 / sdk-ts 67 / agent-shell 12 / tools 2）；`check:consistency` OK（273/55/61/84）；`gen-matrix --check` 无 drift；`git diff --check` 干净。
- CI：push + PR 后 Windows+Linux 全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移（含交 Lane-CTR/CFR 事项）

无新漂移登记。口径与请求：

1. **给 Lane-CTR 的契约请求（精确）**：将 `intent-interpretation.schema.json` 纳入 codegen CORE_SET（双语言模块 + SCHEMA_ID/SCHEMA_DIGEST 常量 + 聚合表）。确定具名消费者 = `cognitive-kernel::intent_chain::record_interpretation_candidate`（现手工 `json!` 组装候选 canonical 值，替换点单一、换装机械）。同请求顺带确认：`user-intent-record`/`task-contract` 生成绑定已被本批消费（消费者事实供 ADR-0006 delivery record 更新）。
2. **REQ-SHELL-CORRECTION-001 / REQ-AKP-INTENT-001 matrix 归属**：kernel 承载面已交付（supersede 编排 + 双点 fencing + 五类分类），但 shell 命令面与 AKP 信封携带（parent/supersedes digest 字段）归 Lane-RUN——本批**不回填**这两行 matrix，防止把 kernel 分量虚报成端到端实现；RUN 交付后按其证据回填。
3. **`GovernanceSeed` 是过渡输入面**：header 治理引用（owner/authority/resource_scope strong ref、tenant、retention）由调用方以确定性数据供给；M5 RUN 落地持久化治理对象后应从治理链解析（D-018 ②同源）。若 RUN 需要 kernel 提供治理对象解析端口，走 §4.1 同通道提请求。
4. **进展事实与 F-014 sink 口径**：`loop_progress_facts` 写入沿用 checkpoint 的事务内 fencing 校验（store-transaction sink 类），`COMMIT_SINKS` 审查常量保持 4 项不扩（progress fact 归 CheckpointWrite 同类：loop 事实持久化）；`sink_inventory_is_complete_and_stable` 钉扎不变。
5. 本机工具链：M4 handoff §4.5 基础上**新增** `RUSTFLAGS="-C link-self-contained=yes"`（llvm-mingw 无 libgcc/libgcc_eh，链接用 rustc 自带 self-contained mingw 库；CC/AR 仍为 llvm-mingw 绝对路径不进 PATH）。
6. **已接收 Lane-RUN 批 1 handoff §4.2 的两项 KRN 请求**（批 2 前协商，不阻塞）：① governance currency（revocation epoch / capability set version）收编为 store 表 + ProtocolStore 端口——认领为 KRN 下一批候选（与 `GovernanceSeed` 过渡面同源，见 §4.3）；② execution↔effect 关联暴露——本批的 `IntentRow.task_binding` + `IntentChainStore::list_intents_for_task` 已给出 **task↔effect** 关联（每 intent 行携 `effect_object_id`），stop 的 pending-effects 检查可先收窄到 per-task；per-execution 粒度需 AgentExecutionBinding 持久化，同列 KRN 下一批候选。

## 5. 下一步入口

- **Lane-RUN M5**（可立即消费）：§7 冻结端口面 + M4 handoff §5 效果协议面；第一动作建议按 `docs/prompts/lane-run.md`。
- **Lane-CFR 向量执行候选**（均已有内核行为孪生可复用）：
  - `intent-supersede-002`（INTENT-SUPERSEDE-002）：`m5_intent_chain.rs::user_correction_advances_epoch_and_fences_old_dispatch` 即模板——expected 字段逐项可断言（old_epoch_new_dispatch_rejected / pending_effect_action=reconcile_before_continue / error.code+category）；
  - `shell-target-ambiguity-001`：clarification 判定的 intent 侧分量（`INTENT_CLARIFICATION_REQUIRED` 路径）已可执行；`SHELL_TARGET_AMBIGUOUS` 的 TargetSelector 解析分量等 RUN shell 面；
  - `loop-contract-001` / `loop-gate-001`（contract-traceability 层）：kernel 行为证据可引 `m5_harness.rs`；
  - `intent-acceptance-007` 复核：chain 绑定事实新增可断言面。
- 工作分支：`lane/krn`；建议提示词：`docs/prompts/lane-run.md`（RUN）/ `docs/prompts/lane-cfr.md`（CFR）。

## 6. 快照

- PROGRESS 已更新：是（Lane-KRN 行、实现计数 34→41、行为测试 93→109、workspace 166、M5 行、handoff 列表）。
- 本次提交（按序）：`0fbe9ac` intent 链 + supersede fencing 批 → `769606a` Loop 端口批 → `0196ba3` 恢复 6/7 + D-018 批 → `98f6ca5` lint 批（行为零变化）→ docs 批（本文件所在提交）。

## 7. M5 kernel 端口面（冻结，给 Lane-RUN）

> 语义不变量总则：以下全部入口是确定性代码；概率组件只能出现在 `InterpretationCandidate` 的生产侧。所有 guard 从**耐久重读**派生（不是内存缓存）；所有拒绝码是 registry 注册码。`S` 统一指 `SqliteAuthorityStore`（同时实现 `AuthorityStore + ProtocolStore + IntentChainStore + HarnessStore`）。

### 7.1 意图链（`cognitive_kernel::intent_chain`）

| 端口 | 签名（摘要） | 语义 | 拒绝码 |
|---|---|---|---|
| `record_user_intent` | `(store, clock, ids, &WriterLease, &UserIntentCommand) -> Result<UserIntentRecordRow, EffectError>` | 语义解释前耐久固定原始表达（append-only + 事件同事务；schema 形状 = 生成绑定；REQ-GOBJ-REF-004 content digest） | fenced/重复/空表达 → `STATE_CONFLICT` |
| `record_interpretation_candidate` | `(store, clock, ids, lease, &record_id, &InterpretationCandidate, &GovernanceSeed, &correlation) -> Result<InterpretationRow, EffectError>` | 候选持久化；`recorded_status` 确定性派生（material ⇒ clarification_required）；record 未固定即拒 | `STATE_CONFLICT` |
| `admit_interpretation` | `(store, &AcceptanceCommand{interpretation_id, accepted_by, accepted_digest}) -> Result<AdmittedInterpretation, EffectError>` | 确定性准入门；令牌是铸合同的唯一入场券 | material 歧义 → `INTENT_CLARIFICATION_REQUIRED`；非 authority → `CONTEXT_AUTH_DENIED`；digest 不符 → `STATE_CONFLICT` |
| `mint_task_contract` | `(store, clock, ids, lease, &AdmittedInterpretation, &TaskContractCommand, expected_current_epoch) -> Result<TaskContractRow, EffectError>` | epoch = expected+1（store 事务内 CAS）；≥1 acceptance 条件强制；合同绑定三 strong ref | 竞争/无 acceptance 条件 → `STATE_CONFLICT` |
| `supersede_task_contract` | `(store, clock, ids, lease, &SupersedeCommand{acceptance, contract, expected_current_epoch}) -> Result<SupersedeReport, EffectError>` | 修正 cutover：同一准入门 + `supersedes` 强制 + epoch CAS + 旧 epoch 未决工作五类分类（`PendingWork{effect, key, state, disposition}`） | 同上 + 分类为事实非动作 |
| `verify_task_binding_current` | `(store, &TaskBinding) -> Result<(), ProtocolDenial>` | epoch 现势判定（effects 已内嵌调用；RUN 可单独预检） | 旧 epoch → `INTENT_VERSION_SUPERSEDED`；超前 epoch → `STATE_CONFLICT` |
| `derive_candidate_status` | `(&InterpretationCandidate) -> &'static str` | 纯函数：schema 条件的确定性派生 | — |

**Effects 面变化**：`IntentCommand.task_binding: Option<TaskBinding>`（Some 时 mint 校验 epoch 现势并落库）；`dispatch_effect` 对耐久 IntentRow 的绑定二次 fencing（在 transition commit 与外呼之前）。`current_contract_epoch` 归入 **`ProtocolStore`**（与 writer fencing epoch 同类的协议侧 fencing 状态读取），因此 `EffectProtocol`/`mint_intent`/`run_recovery` 的 `S` bound 保持 M4 冻结面 `AuthorityStore + ProtocolStore` **不变**（RUN 批 1 的 management plane 无需改动即过编译，已实测）。

### 7.2 有界 Loop（`cognitive_kernel::harness::LoopDriver`）

| 端口 | 签名（摘要） | 语义 | 拒绝码 |
|---|---|---|---|
| `LoopDriver::new` | `(store, clock, ids, actor_ref, authority_ref, correlation_id)` | 驱动器构造（同 EffectProtocol 惯例） | — |
| `contract_facts` | `(task_ref) -> Result<ContractFacts{task_ref, contract_epoch, max_iterations, max_retries, contract_digest}, EffectError>` | 当前合同耐久重读 + 生成绑定解析（sanctioned derivation） | 无合同 → `STATE_CONFLICT` |
| `start_loop` | `(loop_id, expected_version, task_ref, &BudgetId, lease) -> CommittedTransition` | START→OBSERVE（LOOP_STARTED）；guard `task_contract_pinned`（耐久合同）+ `loop_budget_available`（台账重读、无维度耗尽）；evidence = 合同 strong ref | 无合同 → `STATE_CONFLICT`；预算耗尽 → gate `GuardUnsatisfied`(`STATE_CONFLICT`) |
| `begin_iteration` | `(loop_id, expected_version, task_ref, iteration, &BudgetId, &BudgetCharge, lease) -> CommittedTransition` | CONTINUE→OBSERVE（NEXT_ITERATION）；迭代单调（= 上次进展事实 + 1）；`iteration > max_iterations` → 注册硬限码（先于任何迁移/debit）；checkpoint 必须存在（evidence）；charge 与迁移**同事务** debit | 上限/超预算 → `RESOURCE_BUDGET_EXHAUSTED`；单调/无 checkpoint → `STATE_CONFLICT` |
| `record_progress` | `(loop_id, iteration, ProgressStatus, action_fingerprint, &[evidence_refs], lease) -> ProgressFactRow` | 类型化进展事实 append-only；`Advanced` 必须携证据；事务内 fencing 校验 | `STATE_CONFLICT`（含 stale writer Conflict） |
| `stagnation` | `(loop_id) -> StagnationFacts{consecutive_without_progress, last_advanced_iteration, recorded_iterations}` | 耐久事实纯折叠；喂 STAGNATION_DETECTED/升级边（编排归 RUN） | — |
| `retry_count` | `(loop_id, action_fingerprint) -> u64` | 同指纹非 advanced 计数（REQ-RUN-008 键） | — |
| `admit_retry` | `(&ContractFacts, prior_failed_attempts) -> Result<(), ProtocolDenial>` | `count >= max_retries` 拒 | `RESOURCE_BUDGET_EXHAUSTED` |
| `end_iteration` | `(loop_id, expected_version, &task_object_id, &report_id, report_content, &BudgetId, lease) -> CommittedTransition` | VERIFY→CONTINUE（PROGRESS_VERIFIED）；guard `loop_budget_remaining` + `task_not_accepted`（task 耐久重读 ≠ COMPLETED）；evidence = verification_report | gate `GuardUnsatisfied` |

Checkpoint 写入沿用 M4 `ProtocolStore::append_checkpoint`（F-014 事务内 fencing）；恢复侧校验 `validate_checkpoint` 不变。

### 7.3 恢复 6/7 与 D-018（`cognitive_kernel::recovery` / `ports`）

| 端口/事实 | 形状 | 语义 |
|---|---|---|
| `RecoveryReport.reauthorization_obligations` | `Vec<ReauthorizationObligation{effect_object_id, idempotency_key, grant_epoch, capability_set_version}>` | 步骤 6 事实：非终态 continuation 的耐久授权绑定；RUN 必须逐条换新 grant |
| `reauthorization_satisfied` | `(&ReauthorizationObligation, &AuthorizationGrant, &GovernanceCurrency, &WallTimestamp) -> bool` | 步骤 6 算术；teeth = dispatch/commit 的 `capability_and_revocation_current` guard |
| `RecoveryReport.context_rebinding` | `ContextRebinding{fenced_epoch, new_epoch}` | 步骤 7 事实；消费点 = `ContextViewCache::serve_declared`（声明旧绑定 → `CONTEXT_AUTH_DENIED` + 按键清除）与 `lookup_current`（只 miss） |
| `ProtocolStore::load_event_by_id` | `(&EventId) -> Result<Option<CommittedEvent>, StorePortError>` | D-018 组装器输入：outbox 行 → 已提交事件值（event_type/subject/causation/event_time/payload 字节）；未知 id → None fail-closed |

### 7.4 新持久化面（store，供参考不直接消费）

`user_intent_records` / `intent_interpretations` / `task_contracts`（UNIQUE(task_ref, contract_epoch) + 事务内 epoch CAS）/ `loop_progress_facts`（UNIQUE(loop, iteration) + 事务内 fencing）——全部 append-only 触发器钉扎；`intents` 表新增可空 `task_ref`/`contract_epoch` 列（CHECK 成对出现，append-only 兼容）。
