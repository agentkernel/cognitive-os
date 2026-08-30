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
    symbols: ["PROJECT_AGGREGATE_SCHEMA_V26", "ProjectAggregateStore"]
  - path: personal/crates/cognitive-store/src/employee.rs
    symbols: ["EMPLOYEE_SCHEMA_V27", "EmployeeStore", "HandoffSpec"]
  - path: personal/crates/cognitive-store/src/migration.rs
    symbols: ["execute_sqlite_migration_plan"]
  - path: personal/crates/cognitive-store/src/provider_control_plane.rs
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
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/m2_acceptance.rs
  - personal/crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:66a6ca92e9f759652c0b76da82bd44c8ccfb09028a76986c660663f1fac48f88"
non_claims:
  - 明确不声明 authority 与 installation 两个 SQLite 文件之间的跨库原子性。
---

# 存储与迁移

`cognitive-store` 是 kernel 端口背后的单写者 SQLite WAL 适配器。`SqliteAuthorityStore`
可克隆：克隆共享同一连接互斥，使 Personal daemon 能把同一个 writer 交给 HTTP Task
准入与周期调度 tick。XDG state 下两个数
据库：**authority**（迁移 v1–v27）与 **installation**（v1–v4）。不声明跨库原子性；
准备流程先 authority 后 installation，第二阶段失败时报错并指明备份路径。

## 权威库迁移图（v1–v27）

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

几乎所有持久表都带 BEFORE UPDATE/DELETE 触发器（"append-only" abort）；唯一派生表是
`memory_search_fts`（可重建；检索先跑权威过滤 CTE 再 `MATCH`）。

**承重细节**：`SqliteAuthorityStore::open` 只引导 v1–v17 的 schema 常量；v18–v27 的
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
