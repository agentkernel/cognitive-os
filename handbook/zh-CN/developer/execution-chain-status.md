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
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
tests:
  - apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - apps/kernel-server/src/personal/tool_executor/tests.rs
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:5ab033d2a72d760e8927064c7cfd9083711fd4406f9b79203bf4210ea9d9ccee"
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
| Task 准入调度引导 | implemented | 单个 fenced SQLite 事务发布 TaskContract + `START` Loop + 硬 Budget + 当前 epoch runnable 行；含崩溃/重复/回滚负例 |
| Pi 之前封存 ContextRequest/View、逐 body 重授权 | implemented | kernel-server scheduler_authority 真 SQLite 测试 |
| 一次性私有 socket 上的受限 Pi candidate 进程 | implemented | pi-agent-adapter 协议/启动测试 |
| candidate 准入捆绑（Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT，全或无） | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead / WorkspaceSearch / ProcessCheck 执行器（persist-before-dispatch、原键对账） | implemented，仅测试调用 | 每个 sink 都重查不可变目录完全相等；search 使用句柄相对 no-follow 打开、打开后类型/reparse 校验，并在枚举时执行访问上限 |
| WorkspaceWrite / WorkspacePatch 变更执行器 | implemented，仅测试调用 | 句柄锚定的 no-follow parent/target/staging 操作；逐目标 OS 锁闭合最终 CAS 窗口；write 流式 preimage、patch 显式 preimage 上限、批准 workspace 外的持久原键 attempt/receipt 与 orphan 清理 |
| HttpFetchReadOnly 执行器，走仓库唯一受审计的 Rustls 边界（仅 GET；无调用方 header、不跟随重定向、不继承代理、仅已登记 origin） | implemented，仅测试调用 | attempted/completed 状态跨重启保留；timeout/network attempt 与持久状态缺失均对账为 `Indeterminate`，完整原键 receipt 对账为已执行；回环 TLS 证明仍见 `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| 独立 verifier 接缝（fixed post-state、追加式报告、CAS 背书证据） | implemented，仅测试调用 | verifier 模块测试 |
| 启动恢复 | implemented | 对账已消费交接；当前已准入合同只幂等修复缺失的 Loop/Budget/调度前置，不替换既有权威 |

## 剩余生产接线缺口

原先的引导缺口已在准入路径闭合，且未新增平行调度器：成功的
`TaskApplicationService::admit` 会把合同命名的 Loop、Budget 与 runnable 调度行原子发
布。剩余缺口为：

1. **单 tick、无循环**：daemon 仅在启动时执行一次
   `run_private_scheduler_tick_with_store`；不存在周期调度线程。
2. **执行器未接线**：六个已登记族现在都有已装配 sink（P2-T10），因此
   `ASSEMBLED_EXECUTOR_FAMILIES` 列出全部六族，资源投影把每一族报告为
   `execution_ready`。这一事实必须窄读：它表示*本二进制含有该族的执行器*，不表示
   Agent 能到达它。没有任何 `dispatch_staged_*_effect` 存在生产调用者——sink 目前只能
   从测试到达；周期 daemon worker 派发接线后才可从生产路径到达。
3. **verifier 未接线**：`record_independent_verification` 与 loop continuation 入口
   仅测试演练；没有生产路由推进验证或 Task 验收。

跨模块细节：调度闭合把 `RECONCILED/VERIFIED/VERIFY_FAILED` 视为已闭合，而管理面
stop 把它们计为 pending——接线时须记住这一有意的保守不对称。

当上述任何一项变化时，须在同一 PR 内更新本页（以及
[`user/tasks-and-execution`](../user/tasks-and-execution.md) 与
[`ref.capability-status`](../reference/capability-status.md)）——本页指纹会强制这次
复核。
