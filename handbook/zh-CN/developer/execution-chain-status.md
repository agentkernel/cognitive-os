---
doc_id: dev.execution-chain-status
locale: zh-CN
kind: concept
audience: [developer, ai]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick_with_store"]
  - path: apps/kernel-server/src/personal/scheduler_authority/worker.rs
  - path: apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: apps/kernel-server/src/personal/verification_executor.rs
  - path: crates/cognitive-store/src/sqlite/protocol.rs
    symbols: ["insert_intent"]
tests:
  - apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - apps/kernel-server/src/personal/tool_executor/tests.rs
fingerprint: "sha256:17defef26d28ff8e88cfb19f00b2ea27b139966ed4bd51a8038792da6bc09bd6"
non_claims:
  - 本页把缺口记录为记录基线上的事实；既不预测排期，也不贬低已测组件。
---

# 执行链状态

全手册最怕漂移的一页。设计链路：

调度 lease → 封存 Context → Pi candidate → candidate 准入（Intent + Effect + 一次
性 WIA）→ 受治理工具执行 → 独立验证 → verified continuation 或 ceiling STOP。

## 各环节今天的状态

| 环节 | 状态 | 证据 |
|---|---|---|
| 调度持久化、CAS lease、fencing、上限 | implemented | store 调度测试；`SchedulerService` 上限测试 |
| Pi 之前封存 ContextRequest/View、逐 body 重授权 | implemented | kernel-server scheduler_authority 真 SQLite 测试 |
| 一次性私有 socket 上的受限 Pi candidate 进程 | implemented | pi-agent-adapter 协议/启动测试 |
| candidate 准入捆绑（Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT，全或无） | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead / WorkspaceSearch / ProcessCheck 执行器（persist-before-dispatch、原键对账） | implemented，仅测试调用 | `tool_executor/tests.rs` |
| WorkspaceWrite / WorkspacePatch 变更执行器：expected-preimage 比较交换、staging 文件加 rename 发布、通过重读目标对账 | implemented，仅测试调用 | `tool_executor/tests.rs` |
| 独立 verifier 接缝（fixed post-state、追加式报告、CAS 背书证据） | implemented，仅测试调用 | verifier 模块测试 |
| 启动时恢复已消费交接 | implemented | daemon 启动路径 |

## 四个接线缺口（基线上已核验）

1. **无引导行**：Task 准入持久化合同 + context + 策略，但不插入调度行。行由
   `ProtocolStore::insert_intent`（与 task 绑定 Intent 同事务）创建——而到达该处只能
   经 candidate 准入，后者又要求已存在的 leased 行。`SchedulerRepository::upsert`
   的生产调用者：无（仅测试与 benchmark）。
2. **单 tick、无循环**：daemon 仅在启动时执行一次
   `run_private_scheduler_tick_with_store`；不存在周期调度线程。
3. **执行器未接线**：`dispatch_staged_workspace_read_effect`、
   `dispatch_staged_process_check_effect` 无生产调用者。daemon 现在对此诚实呈报：资
   源投影从 `ASSEMBLED_EXECUTOR_FAMILIES`（WorkspaceRead、ProcessCheck）派生逐工具
   `execution_readiness`，其余族显示为 `registered_only` 而非看似可执行。
4. **verifier 未接线**：`record_independent_verification` 与 loop continuation 入口
   仅测试演练；没有生产路由推进验证或 Task 验收。

跨模块细节：调度闭合把 `RECONCILED/VERIFIED/VERIFY_FAILED` 视为已闭合，而管理面
stop 把它们计为 pending——接线时须记住这一有意的保守不对称。

当上述任何一项变化时，须在同一 PR 内更新本页（以及
[`user/tasks-and-execution`](../user/tasks-and-execution.md) 与
[`ref.capability-status`](../reference/capability-status.md)）——本页指纹会强制这次
复核。
