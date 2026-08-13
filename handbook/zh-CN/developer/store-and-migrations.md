---
doc_id: dev.store-migrations
locale: zh-CN
kind: reference
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-store/src/personal_db.rs
    symbols: ["authority_migration_plan", "prepare_personal_databases"]
  - path: crates/cognitive-store/src/migration.rs
    symbols: ["execute_sqlite_migration_plan"]
  - path: crates/cognitive-store/src/sqlite/store.rs
    symbols: ["SqliteAuthorityStore"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: crates/cognitive-store/src/scheduler.rs
    symbols: ["SchedulerRepository", "acquire_eligible_lease"]
tests:
  - crates/cognitive-store/tests/p1_t01_layout_migrations.rs
  - crates/cognitive-store/tests/m2_acceptance.rs
  - crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:cecb8ff4b439b173c3d6c16bdd09f04ae364407b4632774335e5547c4f98fdc8"
non_claims:
  - 明确不声明 authority 与 installation 两个 SQLite 文件之间的跨库原子性。
---

# 存储与迁移

`cognitive-store` 是 kernel 端口背后的单写者 SQLite WAL 适配器。XDG state 下两个数
据库：**authority**（迁移 v1–v23）与 **installation**（v1–v4）。不声明跨库原子性；
准备流程先 authority 后 installation，第二阶段失败时报错并指明备份路径。

## 权威库迁移图（v1–v23）

| 版本 | 新增 |
|---|---|
| v1 | 受治理对象（CAS 行）、追加式事件/记录、预算、outbox、intent（幂等键唯一）、fencing 单行、checkpoint、用户意图、解释、任务合同、loop 进度事实 |
| v2–v3 | 调度条目；v3 重建为 PK `(task_ref, contract_epoch)` 并保留 lease |
| v4–v9 | 操作候选提案、daemon 操作描述符 + 授权快照、worker iteration authorization（WIA）及一次性消费与调度 lease 绑定 |
| v10–v11 | fixed post-state、verification request/report、continuation authorization 及 lease 绑定消费 |
| v12–v15 | context request/view、workspace context source（role/trust CHECK）、授权/撤销事实集、调度执行策略 |
| v16–v20 | Memory candidate/decision/object、FTS5 派生索引、tombstone（forget → +expire → +supersede）、版本谱系 |
| v21–v23 | Skill package/revision/binding、binding 撤销、revision 谱系 |

几乎所有持久表都带 BEFORE UPDATE/DELETE 触发器（"append-only" abort）；唯一派生表是
`memory_search_fts`（可重建；检索先跑权威过滤 CTE 再 `MATCH`）。

**承重细节**：`SqliteAuthorityStore::open` 只引导 v1–v17 的 schema 常量；v18–v23 的
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
入/事件、硬 Budget，以及 `(task_ref, contract_epoch)` runnable 调度行。任何靠后的成
员冲突都会回滚先前插入；成功提交后崩溃重开则四项前置全部存在。
启动恢复还可在一个 fenced 事务内幂等修复旧的当前合同所缺 Loop、Budget 或调度工作。
既有行只校验，绝不替换或重置；过期合同 epoch 不可修复。

验证启动复用既有 fixed-post-state/request 表，不新增迁移。一个 immediate 事务校验写
者、当前合同、闭合 Effect 版本、共享行绑定与 Loop CAS，再插入两个追加式行并提交
`ACT -> VERIFY`；任何靠后冲突都会整体回滚。
