---
doc_id: dev.store-migrations
locale: zh-CN
kind: reference
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-store/src/personal_backup.rs
    symbols: ["write_personal_backup_archive", "restore_personal_backup_archive"]
  - path: personal/crates/cognitive-store/src/personal_db.rs
    symbols: ["authority_migration_plan", "prepare_personal_databases"]
  - path: personal/crates/cognitive-store/src/project_aggregate.rs
    symbols: ["PROJECT_AGGREGATE_SCHEMA_V26", "APPROVAL_PREVIEW_NARROW_SCHEMA_V29", "STANDING_APPROVAL_POLICY_SCHEMA_V30", "ProjectAggregateStore"]
  - path: personal/crates/cognitive-store/src/employee.rs
    symbols: ["EMPLOYEE_SCHEMA_V27", "EmployeeStore", "HandoffSpec"]
  - path: personal/crates/cognitive-store/src/conversation.rs
    symbols: ["CONVERSATION_ARCHIVE_SCHEMA_V28", "ConversationStore", "CONVERSATION_ARCHIVE_PROJECTION_ID", "ArchiveReadSpec", "ArchiveAppendSpec"]
  - path: personal/crates/cognitive-store/src/assistant.rs
    symbols: ["AssistantPlane", "AssistantTurnSpec", "ASSISTANT_ENGINE_ID", "ASSISTANT_PI_PIN"]
  - path: personal/crates/cognitive-store/src/hosted_dsh.rs
    symbols: ["HOSTED_DSH_SCHEMA_V31", "HostedDshPlane", "HostedDshStartSpec", "HOSTED_DSH_ENGINE_ID"]
  - path: personal/crates/cognitive-store/src/hosted_dsh_attempt.rs
    symbols: ["HOSTED_DSH_ATTEMPT_SCHEMA_V36", "HostedDshAttemptStore", "HOSTED_ATTEMPT_PROJECTION_ID", "HostedAttemptIntentSpec", "HostedAttemptTerminalSpec", "HostedArtifactObservation"]
  - path: personal/crates/cognitive-store/src/vault.rs
    symbols: ["VAULT_SCHEMA_V32", "VaultStore", "VaultImportSpec", "CONTEXT_INJECT_ORDER", "VAULT_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/routine.rs
    symbols: ["ROUTINE_SCHEMA_V33", "RoutineStore", "ROUTINE_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/windows_host.rs
    symbols: ["WINDOWS_HOST_SCHEMA_V34", "WindowsHostStore", "WINDOWS_HOST_PROJECTION_ID", "WAKE_RECOVERY_STEPS"]
  - path: personal/crates/cognitive-store/src/x_connector.rs
    symbols: ["X_CONNECTOR_SCHEMA_V35", "XConnectorStore", "X_CONNECTOR_PROJECTION_ID"]
  - path: personal/crates/cognitive-store/src/migration.rs
    symbols: ["execute_sqlite_migration_plan"]
  - path: personal/crates/cognitive-store/src/provider_control_plane.rs
    symbols: ["honest_usage_read_model", "labelled_cost_source", "honest_unknown_cost", "replace_binding"]
  - path: personal/crates/cognitive-store/src/sqlite/store.rs
    symbols: ["SqliteAuthorityStore"]
  - path: personal/crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: personal/crates/cognitive-store/src/scheduler.rs
    symbols: ["SchedulerRepository", "acquire_eligible_lease"]
tests:
  - personal/crates/cognitive-store/tests/p1_t01_layout_migrations.rs
  - personal/crates/cognitive-store/tests/p11_t03_project_aggregate.rs
  - personal/crates/cognitive-store/tests/p11_t04_employee.rs
  - personal/crates/cognitive-store/tests/p11_t05_conversation.rs
  - personal/crates/cognitive-store/tests/p11_t06_assistant.rs
  - personal/crates/cognitive-store/tests/p11_t07_hosted_dsh.rs
  - personal/crates/cognitive-store/tests/p13_t02_hosted_dsh_attempt.rs
  - personal/crates/cognitive-store/tests/p11_t10_vault.rs
  - personal/crates/cognitive-store/tests/p11_t08_routine.rs
  - personal/crates/cognitive-store/tests/p11_t02_windows_host.rs
  - personal/crates/cognitive-store/tests/p11_t14_x_connector.rs
  - personal/crates/cognitive-store/tests/p11_t09_hitl_canvas.rs
  - personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/m2_acceptance.rs
  - personal/crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:8f710ca7e373cb3b745b607486f4138cddb5602d7972fbcc076e12f9d835f1f8"
non_claims:
  - 明确不声明 authority 与 installation 两个 SQLite 文件之间的跨库原子性。
---

# 存储与迁移

`cognitive-store` 是 kernel 端口背后的单写者 SQLite WAL 适配器。`SqliteAuthorityStore`
可克隆：克隆共享同一连接互斥，使 Personal daemon 能把同一个 writer 交给 HTTP Task
准入与周期调度 tick。XDG state 下两个数
据库：**authority**（迁移 v1–v37）与 **installation**（v1–v4）。不声明跨库原子性；
准备流程先 authority 后 installation，第二阶段失败时报错并指明备份路径。

## 权威库迁移图（v1–v37）

| 版本 | 新增 |
|---|---|
| v1 | 受治理对象（CAS 行）、追加式事件/记录、预算、outbox、intent（幂等键唯一）、fencing 单行、checkpoint、用户意图、解释、任务合同、loop 进度事实 |
| v2–v3 | 调度条目；v3 重建为 PK `(task_ref, contract_epoch)` 并保留 lease |
| v4–v9 | 操作候选提案、daemon 操作描述符 + 授权快照、worker iteration authorization（WIA）及一次性消费与调度 lease 绑定 |
| v10–v11 | fixed post-state、verification request/report、continuation authorization 及 lease 绑定消费 |
| v12–v15 | context request/view、workspace context source（role/trust CHECK）、授权/撤销事实集、调度执行策略 |
| v16–v20 | Memory candidate/decision/object、FTS5 派生索引、tombstone（forget → +expire → +supersede）、版本谱系 |
| v21–v23 | Skill package/revision/binding、binding 撤销、revision 谱系 |
| v24 | 按 Task/epoch/request/session 绑定的只追加 Memory/Skill 消费记录 |
| v25 | Provider Control Plane 账户、模型、binding、用量事件/聚合、预算、告警、审计 |
| v26 | Personal-private Project 聚合（`p11_draft`、`p11_candidate`、`p11_charter_revision`、`p11_project`、`p11_plan_revision`、`p11_stage`、`p11_gap`、`p11_stage_test_fact`、`p11_acceptance_fact`、`p11_approval_preview`）。新表，不是 `family=task`。 |
| v27 | Role Blueprint / Assignment / Employee / Grant（`p11_role_blueprint`、`p11_role_blueprint_revision`、`p11_employee`、`p11_employee_revision`、`p11_assignment`、`p11_install_fact`、`p11_grant`、`p11_speech_audit`、`p11_handoff`）。Blueprint 无 Provider binding。权威 id 是 Employee；`runtime_binding_ref` 可替换。Handoff 行保持 `authority_stays=1`；写入走 `HandoffSpec`，聊天不能转移权威。 |
| v28 | Personal-private 对话档案（`p11_conversation_archive`），新标识 `cognitiveos.personal.conversation-archive/0.1`。白名单投递发言落档案行；owner `append` 接受 `note`/`deliverable`/`handoff`/`blocked`/`decision-request`。chatter 只留 `p11_speech_audit`。索引用 `limit` 1..=32，返回引用（record_id + digest）而不是正文。ADR-0058 `conversation-projection/0.1` 不被 coerce。档案行只是观察；record_id 不能当作 stage-test 完成。 |
| v29 | ApprovalPreview `superseded_by`（P11-T09 HITL）。改窄签发**新** pending preview，旧行冻结为 `superseded`。拒绝留下 `receipt_ref`。stale 只按机械 `base_state_digest` 不等判定，不是墙钟新鲜度。聊天/task 不能 confirm/reject/narrow。 |
| v30 | `grant-expansion` subject_kind 与 StandingApprovalPolicy 时间盒（`p11_standing_approval_policy`）。`expires_at` 必填且 ≤7 天。Settings 列表/撤销是 management HTTP。聊天不能签发。重建 `p11_approval_preview` CHECK。 |
| v31 | 隐藏托管 DSH 子进程（`p11_hosted_dsh_child`）。`runtime_binding_ref` 绑到 `hosted-dsh:<artifact>:<child_id>`（pid/digest/artifact）。进程退出清除 pid 并标 `exited`；不删除 Employee、对话档案或 Memory。Windows GNU 上 isolated spawn 失败闭合。Windows OPC E2E 为 `not-run`。 |
| v32 | Markdown Vault（`p11_vault_document`、可重建 `p11_vault_index_entry`、`p11_vault_conflict`），标识 `cognitiveos.personal.markdown-vault/0.1`。导入必须带 rights/provenance。文件不是 Project 权威（`is_authority=0`）。索引不是 Memory FTS。无冲突记录的 last-write-wins 被拒绝。宿主文件系统 E2E 为 `not-run`。 |
| v33 | Routine revision / Trigger occurrence 台账（`p11_routine`、`p11_routine_revision`、`p11_routine_occurrence`），标识 `cognitiveos.personal.routine/0.1`。重叠策略为 `no-overlap-queue-latest`。missed/coalesced 行可见。active occurrence 复用 `scheduler_entries`（`task://personal/routine/{occurrence_id}`）。checkpoint 不是完成。无 Temporal / 第二套调度表。clock/sleep/restart E2E 为 `not-run`。 |
| v34 | Windows host Personal Home / 生命周期 / missed / 有序恢复（`p11_windows_host_home`、`p11_windows_host_daemon`、`p11_windows_host_dsh_child`、`p11_windows_host_offline_segment`、`p11_windows_host_recovery`、`p11_windows_host_restore_point`），标识 `cognitiveos.personal.windows-host/0.1`。布局为 `Personal Home/app/` + `Personal Home/data/`；升级替换 app、保留 data。托盘只观察与请求，不写权威。daemon 不能兑现时拒绝 close background-or-pause。同盘版本是本地 restore point，不是备份。原生 tray/ACL/sleep/SecretStore E2E 为 `not-run`。 |
| v35 | X/Twitter connector 账户 / preview / 发布台账（`p11_x_connector_account`、`p11_x_connector_preview`、`p11_x_connector_publish`），标识 `cognitiveos.personal.x-connector/0.1`。仅 SecretStore `secret_ref`。`is_p0_hero` 与 `platform_qualified` CHECK=0。impressions 保持字面量 `unknown`。receipt 不是完成。live X API E2E 为 `not-run`。 |
| v36 | 托管 DSH 真实 Attempt 循环（`p13_hosted_dsh_artifact_fact`、`p13_hosted_dsh_attempt`、`p13_hosted_dsh_attempt_frame`），标识 `cognitiveos.personal.hosted-dsh-attempt/0.1`（P13-T02）。artifact 事实只追加，kind 由 daemon 推导为 `health-check` / `update` / `rollback`，health 为 `pinned` / `absent` / `corrupt` / `mismatch` / `script-missing`；只有 `pinned` 允许 spawn。Attempt 行就是 persist-before-dispatch 的 Intent（`intent_persisted` CHECK=1）：`persisted` → `dispatched`（pid，Effect 标记）→ `terminal`（`exited` / `signaled` / `timed-out` / `spawn-failed`——没有 `success`），daemon 崩溃后为 `unknown-outcome`。`completion_claimed` CHECK=0，`verification_status` CHECK=`not-run`，`context_bytes` ≤ 65536。帧是只追加的观察（`authority_written` CHECK=0）。Attempt 永不删除。Windows sandbox / ACL / supply-chain E2E 为 `not-run`。 |
| v37 | Attempt 产物 → CAS → 独立 verifier → 末环验收 → external send（`p13_attempt_artifact`、`p13_artifact_evidence`、`p13_run_acceptance`、`p13_external_send`），标识 `cognitiveos.personal.attempt-artifact/0.1`（P13-T04）。产物行是指向唯一 P3-T03 `ArtifactStore` CAS（`<data_dir>/artifacts`）的 `sha256:` 引用，另带格式、来源帧（`hosted-dsh-child:candidate:DeliverableDraft`、帧 seq、payload digest）与产出时间；新鲜度（`current` / `superseded`）按 Project + task 推导。evidence 只追加，由 CHECK 钉住 verifier `verifier://personal/attempt-artifact` 与 principal `principal://personal/independent-verifier`；其报告字节放在同一 CAS。`p13_run_acceptance` 用 CHECK 钉住末环（`stage_position = stage_count - 1`），并绑定一个 StageTestPassed 事实、产物与 evidence。`p13_external_send` 是 persist-before-dispatch Intent，`published` CHECK=0、`connector` CHECK=`none-qualified`（planned ≠ published）。重建 `p11_approval_preview`，使 `subject_kind` 也接受 `run-acceptance` 与 `external-send`（v30 先例）。宿主打开文件 E2E 为 `not-run`。 |

P11-T07 隐藏托管 DSH 新增 v31 `p11_hosted_dsh_child`。Attempt-runner `start` 的真实调用者是 management HTTP `dsh.hosted.start`；task 通道别名 403。digest/protocol 不匹配、env/argv 含 secret、Pi 作 Member 引擎、Installed Agent chrome、未知子进程输出（`success`/`ok`/`agent_end`）一律失败闭合。daemon Provider 代理 `POST /provider/v1/dsh/chat/completions` 仍是唯一持 secret 路径。Linux Path B 不等于 Windows 托管资格。

P11-T06 隐藏 Pi Assistant **不新增迁移**。它复用 v26 `p11_candidate` / `p11_approval_preview` 与 T05 只读档案上下文。助手登记必须带 typed 出处（`sources[]` | `owner-stated` | `assistant-assumption`）；非空 blob 不够。封闭候选 JSON 禁止 `grant` / `secret` / `trigger-arm`。`draft.apply` 指向 Project/Employee/Grant/已确认 charter 会被拒。助手平面不能写 archive、SecretStore、Memory，也不能 confirm/apply 权威。工具 default-deny；research 只能点名既有 `HttpFetchReadOnly`。exact Pi `0.81.1` 与 `cognitiveos.private-candidate/1` 是身份钉，不是第二套调度器或 Installed Agent。

P13-T03 真实推理同样**不新增迁移**。`AssistantPlane::run_turn` 现在必须带 daemon 观测到的 `AssistantInferenceRecord`（协议 `cognitiveos.personal.assistant-inference/0.1`、已绑 `model_id`、`provider_round_trips ≥ 1`、有界 reply、推理出的对象链、daemon 推导的可引用 URI）；登记进 v26 的候选 ops 携带推理链、标记为 owner 输入的 owner payload、reply digest 与注入顺序引用——绝不把回显 payload 当候选。`validate_inferred_object_chain` 是唯一的对象链校验器（封闭 kind 按链序、每 kind 一个对象、每个字段 `{value, provenance}` 带 typed 出处、`sources[]` uri 只能来自已抓取或 owner 提供的 URI、封闭 schema）；runtime 解析与 HTTP 都调用它。`admit_turn_request` 在任何 Pi 进程启动前拒绝 ambient tool。`provider_unbound_guidance()` 是固定的 Settings 指路（`chat_input: false`、`silent_bind: false`、`candidate_registered: false`）。`candidate_count` 是只读计数辅助。登记时的 secret-shape 守卫只在 token 起始处把 `sk-` 当作 key 前缀（`risk-based`、`task-contract`、`desk-side` 是普通文本）；`bearer `、`api_key`、`x-api-key`、`ssv1:` 与 token 起始的 `sk-…` 仍被拒绝。

P11-T09 HITL 画布复用 v26 `request_preview` / `confirm_preview` / `p11_approval_preview` 与 v29 `superseded_by`、v30 grant-expansion / StandingApprovalPolicy。真实调用者是 management HTTP `preview.reject` / `preview.narrow` / `confirm` / `standing-policy.*`；T05 只宣布+深链；T06 `draft.apply` 不是 authority-approve。宿主 UI E2E 为 `not-run`。Settings chrome 是 T13。无第二套调度器、无聊天 Approve、无 Inbox 一级。

P11-T12 诚实 usage **不新增迁移**。它是对 v25 `llm_usage_events` / `agent_provider_bindings` / `provider_accounts` 的带标签读取：`cost_label` 为 `actual`（`provider_reported`+`priced`）、`estimated`（仅当确实记录了 `locally_estimated`+`priced`）或 `unknown`（序列化绝不为 JSON `0`）。`GET /management/usage` 同时返回四层 binding 说明；Project/employee/Task 层今日显式 `unbound`。账户身份与配额是分开的对象。静默改账户/模型会被拒绝。成员级预算硬停属 2.1 / Deferred。

P11-T08 Routine/Trigger 新增 v33。真实调用者是 management HTTP `routine.revision` / `routine.trigger` / `routine.ledger` / `routine.checkpoint` / `routine.resume`。task 通道别名 403。重叠被拒绝或按 queue-latest 排队；host-unavailable 记入可见 missed；过期 revision 失败闭合；用 checkpoint 当完成被拒绝；consequential 自动恢复失败闭合。HITL 仍是 T09 画布（不是 Inbox 一级）。clock/sleep/restart 宿主 E2E 为 `not-run`。

P11-T10 Markdown Vault 新增 v32。真实调用者是 management HTTP `vault.import` / `vault.index.rebuild` / `vault.index` / `vault.conflicts`。Context 注入顺序是已文档化的 store helper（当前 Task 合同 → 已固定决定 → 带出处摘录 → 摘要 → 旧叙述；超限先砍旧叙述）。Vault 文件不能确认/应用 Project 权威。Memory 准入不能把 Vault 文件吞成权威。对话档案与 Artifact CAS blob 不是 Vault 文件。不捆绑 Obsidian。宿主文件系统 E2E 在 `DEV-WINDOWS-NATIVE-OPC-01` 资格化前为 `not-run`。

P11-T02 Windows host / tray / background 新增 v34。真实调用者是 management HTTP `host.home.admit` / `host.daemon.bind` / `host.close.request` / `host.offline.record` / `host.dsh.bind` / `host.recovery.run` / `host.recovery.advance` / `host.restore-point.record` / `GET host.status`。task 通道别名 403。错误安装根、ACL 逃逸、raw secret env/argv、重复 daemon、孤儿 DSH、假 background、把 restore 当 backup、跳步恢复一律失败闭合。wake/restart 跑完七步有序恢复且只恢复合格工作。不是第二套凭据平面。不是把 DSH web 当宿主壳。原生 Windows install/tray/sleep/SecretStore E2E 在 `DEV-WINDOWS-NATIVE-OPC-01` 资格化前为 `not-run`。

P11-T14 X/Twitter connector 新增 v35。真实调用者是 management HTTP `connector/x/v1/{account.bind,preview.request,preview.confirm,publish.dispatch}` 与 `GET connector/x/v1/status`。task 通道别名 403。raw secret env/argv/body、evasion、抓取内容、聊天 Approve、未确认就发布、把 receipt 当完成、unknown 指标写成 `0`、把 X 当 P0 hero 一律失败闭合。先持久化 Intent 再标记 dispatched。status 不返回 `secret_ref`。live X/CAPTCHA/platform qualification 为 `not-run`。不是 chrome。不是业务结果。

P13-T02 托管 DSH 真实 Attempt 循环新增 v36。真实调用者是 management HTTP `dsh.hosted.attempt.run`：先记 artifact 事实，再在 `cognitive-runtime` broker spawn exact-artifact 子进程**之前**调用 `HostedDshAttemptStore::persist_intent`（已就位 Member、精确 revision、`pinned` artifact、有界且无 secret 形状的 Context），OS pid 出现时标 `dispatched`，每一帧作为观察追加，终态行由 daemon 自己写入。`reconcile_unknown_outcomes` 在 daemon 启动时运行，把崩溃形状的 `persisted`/`dispatched` 行变为 `unknown-outcome`，永不变成 success。`dsh.hosted.attempt.list` / `detail` 是 `runs` 的读取；`dsh.hosted.artifact.check` / `facts` 暴露 health / update / rollback。task 通道别名 403。心跳、`response: done`、exit 0、`agent_end` 形状的文本只改变观察台账；完成权属于独立 verifier（P13-T04）。Linux 真实 spawn 只是实现证据。

P13-T04 新增 v37。写 Attempt 终态的 broker 线程把该次运行交给 `AttemptArtifactStore::ingest_candidate`：只有 `terminal` Attempt 的 `DeliverableDraft` 候选帧、且其 canonical payload 的 digest 等于帧上已记录的 digest，才能成为产物；payload 与交付物文本都放入 CAS（文件与 `file://` 引用永不被接受——`resolve_openable_ref` 只接受 `sha256:`）。`verify_artifact` 是独立 verifier：确定性重读（CAS digest、来源帧绑定、Attempt 终态、UTF-8 / 非空 / 无 secret 形状）产生 `passed` / `failed` / `indeterminate` evidence；child 的 `response done`、exit code 与文本记为 `not-used`。`derive_stage_test` 只从 durable 事实（真实就位、成员持有该环节 slot、`current` 新鲜度、checked digest 等于产物 digest 的 passed evidence、CAS 重读、Attempt 终态）构造 P11-T03 `StageTestOracle` 并调用 `derive_stage_test_passed`；不存在调用方布尔值。`request_run_acceptance` 只为带 passed evidence 支撑的当前 StageTestPassed 的末环铸造 `run-acceptance` ApprovalPreview；`confirm_preview` 写入只追加的 `p13_run_acceptance` 事实。`publication_packet` 是只读 AUTONOMY 发布包（`planned: true`、`published: false`、`chat_can_confirm: false`）；`request_external_send` 铸造 `external-send` ApprovalPreview 与 `previewed` Intent，确认后变为 `planned`——永不 `published`。真实调用者是 management HTTP `outputs` / `outputs.detail` / `outputs.open` / `outputs.export` / `attempt.artifact.verify` / `attempt.artifact.stage-test` / `run.acceptance.request` / `run.acceptance` / `publication.packet` / `publication.external-send.request` / `publication.sends`；task 通道别名 403。

几乎所有持久表都带 BEFORE UPDATE/DELETE 触发器（"append-only" abort）；派生表是
`memory_search_fts` 与 `p11_vault_index_entry`（可重建；Vault 检索不走 Memory FTS）。

**承重细节**：`SqliteAuthorityStore::open` 只引导 v1–v17 的 schema 常量；v18–v37 的
表只有在 `prepare_personal_databases` 执行版本化计划后才存在（生产路径与 P4 测试都
会执行）。

## 迁移引擎

计划先校验（版本严格递增、digest 自洽）再产生任何副作用。`DryRun` 在
`VACUUM INTO` 的临时副本上执行；`Apply` 先写带时间戳备份，然后在**单个** immediate
事务内跑完全部待办迁移，含已记录行 digest 校验、重放跳过安全与提交前
`PRAGMA quick_check` 门。准备流程持有排他 `migration.lock`（崩溃后残留锁需人工移
除）。

## 并发模型

每个 store 实例一个 `Mutex<Connection>`；open 时断言 WAL + `synchronous=FULL`；只读
打开建模降级卷（写 fail-closed 为 `STATE_STORE_UNAVAILABLE`，读与重放保持可用）。调
度 lease 是事务化 CAS：可得性要求 `runnable` 且过 `next_eligible`，或以严格更高
epoch 回收过期 lease；释放要求精确 `(owner, epoch)`；WIA/continuation 授权的消费在
同一事务内绑定到精确的活动 leased 行。

Task 准入复用既有 v1–v3 表，不新增迁移或平行调度器。
`insert_task_contract_with_execution_bootstrap` 在单个 immediate 权威事务内重查写
者 fence 与合同 epoch CAS，再插入 TaskContract 事件、注册初态 `START` 的 Loop 准
入/事件、注册初态 `DRAFT` 的 governed Task 投影（不写第二条 `(object_id, INITIAL)` 事件）、硬 Budget，以及
`(task_ref, contract_epoch)` runnable 调度行。任何靠后的成员冲突都会回滚先前插入；
成功提交后崩溃重开则五项前置全部存在。启动恢复还可在一个 fenced 事务内幂等修复旧的
当前合同所缺 Task、Loop、Budget 或调度工作。
既有行只校验，绝不替换或重置；过期合同 epoch 不可修复。

验证启动复用既有 fixed-post-state/request 表，不新增迁移。一个 immediate 事务校验写
者、当前合同、闭合 Effect 版本、共享行绑定与 Loop CAS，再插入两个追加式行并提交
`ACT -> VERIFY`；任何靠后冲突都会整体回滚。

verified Task completion 不新增 migration 或专用 acceptance 表。canonical decision
bytes 位于 Artifact CAS；两个 immediate 事务复用既有 governed-object/event/transition
record 表，并在 candidate 与最终 acceptance CAS 更新前重查当前合同、精确 fixed state、
最新 report、完整闭合 Effect 集合与 fencing。

Resource Manager 的 list/inspect 辅助方法（`list_non_tombstoned_memory_objects`、
`load_non_tombstoned_memory_object`、`list_skill_bindings`）是对这些 v16–v23 行的
inherent store 读取。它们不新增 migration，也不发明第七族表。

## 用户备份归档

`write_personal_backup_archive` 把 config/data/state/artifact 文件复制进 digest
绑定的目录归档，并写入 Memory/Skill 导出 sidecar。它跳过 `authority.sqlite`、
secret 命名路径和 `provider-config.json`。恢复预检查 schema、完整性与 part
digest，再从 staging 覆盖 live 文件并在失败时回滚快照。聚焦测试把恢复后字节
相等和有限墙钟记为 hypothesis-only 事实。这不是 SQLite dump，也不声明
Gate/RTO/RPO。
