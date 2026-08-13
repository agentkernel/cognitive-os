---
doc_id: dev.execution-chain-status
locale: zh-CN
kind: concept
audience: [developer, ai]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
    symbols: ["PeriodicSchedulerWorker", "serve_personal_loopback"]
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick_with_store"]
  - path: apps/kernel-server/src/personal/scheduler_authority/worker.rs
  - path: apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: apps/kernel-server/src/personal/verification_executor.rs
  - path: apps/kernel-server/src/personal/campaign_observation.rs
    symbols: ["CampaignMutationObservationService", "CampaignExternalStateFixture"]
  - path: crates/cognitive-store/src/sqlite/protocol.rs
    symbols: ["insert_intent"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
tests:
  - apps/kernel-server/src/personal/p2_t17_a7_failure_first.rs
  - apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - apps/kernel-server/src/personal/tool_executor/tests.rs
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:325865c0bb7c551cb018bca16546a234b91cf8818ee286d195832794c4e7fbbe"
non_claims:
  - 本页把缺口记录为记录基线上的事实；既不预测排期，也不贬低已测组件。
  - A7 评测 fixture 与本地/CI 观察证据不得升格为 Gate、release、Profile、B01 或 EVAL-003 结果。
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
| daemon 周期调度 worker | implemented | 仅在绑定/发布 endpoint 后启动；唯一固定延迟串行 worker 拒绝重入、在 pass 错误后继续，顺序退出时取消并 join |
| Pi 之前封存 ContextRequest/View、逐 body 重授权 | implemented | kernel-server scheduler_authority 真 SQLite 测试 |
| 一次性私有 socket 上的受限 Pi candidate 进程 | implemented | pi-agent-adapter 协议/启动测试 |
| candidate 准入捆绑（Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT，全或无） | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead 执行器（persist-before-dispatch、原键对账） | implemented，生产调用 | 周期 worker 重载 WIA/candidate/Intent/持久 descriptor，重查精确调度 lease 与当前授权，在 daemon 数据 workspace 下 staging，并进入既有 Effect 协议；中断后的 leased 行只查询原键且绝不重复派发 |
| WorkspaceSearch / ProcessCheck 执行器 | implemented，仅测试调用 | 每个 sink 都重查不可变目录完全相等；search 使用句柄相对 no-follow 打开、打开后类型/reparse 校验，并在枚举时执行访问上限 |
| WorkspaceWrite / WorkspacePatch 变更执行器 | implemented，仅测试调用 | 句柄锚定的 no-follow parent/target/staging 操作；逐目标 OS 锁闭合最终 CAS 窗口；write 流式 preimage、patch 显式 preimage 上限、批准 workspace 外的持久原键 attempt/receipt 与 orphan 清理 |
| HttpFetchReadOnly 执行器，走仓库唯一受审计的 Rustls 边界（仅 GET；无调用方 header、不跟随重定向、不继承代理、仅已登记 origin） | implemented，仅测试调用 | attempted/completed 状态跨重启保留；timeout/network attempt 与持久状态缺失均对账为 `Indeterminate`，完整原键 receipt 对账为已执行；回环 TLS 证明仍见 `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| 固定 post-state + verification request + Loop `ACT -> VERIFY` 发布 | implemented，生产调用 | WorkspaceRead 对账后，一个 fenced SQLite 事务校验当前闭合 Effect，并把两个追加式行与登记 Loop 转移一起提交 |
| 独立 verifier + continuation loop | implemented，生产调用 | criteria 只从当前 Acceptance 条件推导；登记 fixed-Effect verifier 生成 CAS 背书证据、持久报告并进入 `VERIFY -> CONTINUE`，随后 checkpoint 绑定的一次性权威经 `CONTINUE -> OBSERVE` 消费，不完成 Task |
| A7 评测回环外部变更观察 | implemented，仅测试调用 | 评测自有幂等 fixture（有界 mutate/query/reset/cleanup）；persist-before-dispatch Effect；默认关闭的授权故障点；重启只查询原键并恰好对账一次；绑定独立验证且 `acceptance_ref` 保持为空。本地/fixture 证据不是 Gate、release、Profile、B01 或 EVAL-003 结果 |
| 启动恢复 | implemented | 对账已消费交接；当前已准入合同只幂等修复缺失的 Loop/Budget/调度前置，不替换既有权威 |

## 剩余生产接线缺口

原先的引导缺口已在准入路径闭合，且未新增平行调度器：成功的
`TaskApplicationService::admit` 会把合同命名的 Loop、Budget 与 runnable 调度行原子发
布。零 Intent 行现在进入 pre-admission candidate 分支，而非抛出
`MissingEffectBinding`；该趟签发 WIA 后立即返回，不能消费自己刚产生的 worker 权威。
逐行失败彼此隔离，不会中止有界 pass 中的后续行。daemon 现在只在绑定并发布 endpoint
后启动唯一、非重入且可取消的周期 worker；pass 级失败会重试，不能阻止监听。剩余缺口
为：

1. **执行器接线仍为 partial**：六个已登记族都有已装配 sink（P2-T10），所以
   `execution_ready` 仍只表示本二进制包含它。周期 worker 现在会把无参数
   WorkspaceRead 经持久 Effect 协议从生产派发。WorkspaceSearch、
   WorkspaceWrite/Patch、ProcessCheck 与 HttpFetchReadOnly 在生产尚无独立治理的
   payload/preimage、受监督进程或已登记 origin 载体，因此在 Effect 授权前失败；这些
   sink 仍仅测试调用。
2. **Task 完成仍是独立范围**：生产现在闭合
   `ACT -> VERIFY -> CONTINUE -> OBSERVE`，包含 checkpoint 与一次性 continuation
   authority。报告、checkpoint、continuation 或 A7 评测观察都不完成 Task；验收仍属
   P2-T14。A7 fixture/本地证据不得升格为 Gate、release、Profile、B01 或 EVAL-003
   评测结果。

跨模块细节：调度闭合把 `RECONCILED/VERIFIED/VERIFY_FAILED` 视为已闭合，而管理面
stop 把它们计为 pending——接线时须记住这一有意的保守不对称。

当上述任何一项变化时，须在同一 PR 内更新本页（以及
[`user/tasks-and-execution`](../user/tasks-and-execution.md) 与
[`ref.capability-status`](../reference/capability-status.md)）——本页指纹会强制这次
复核。
