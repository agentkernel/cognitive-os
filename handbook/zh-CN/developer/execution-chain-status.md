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
fingerprint: "sha256:a24fbe111ac9ef29f335d79e1bb81044d831205bad2f6666fa04777ba099d940"
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
| daemon 周期调度 worker | implemented | 仅在绑定/发布 endpoint 后启动；唯一固定延迟串行 worker 拒绝重入、在 pass 错误后继续，顺序退出时取消并 join |
| Pi 之前封存 ContextRequest/View、逐 body 重授权 | implemented | kernel-server scheduler_authority 真 SQLite 测试 |
| 一次性私有 socket 上的受限 Pi candidate 进程 | implemented | pi-agent-adapter 协议/启动测试 |
| candidate 准入捆绑（Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT，全或无） | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead 执行器（persist-before-dispatch、原键对账） | implemented，生产调用 | 周期 worker 重载 WIA/candidate/Intent/持久 descriptor，重查精确调度 lease 与当前授权，在 daemon 数据 workspace 下 staging，并进入既有 Effect 协议；中断后的 leased 行只查询原键且绝不重复派发 |
| WorkspaceSearch 执行器 | implemented，生产调用 | 生产 router 从持久 Intent 携带受治理 query 并 staging 进 search sink；句柄相对 no-follow 打开、打开后类型/reparse 校验，并在枚举时执行访问上限 |
| ProcessCheck 执行器 | implemented，生产调用 | 生产 router 会 staging 有界 process check；在 daemon 受监督进程 registry 接线前 dispatch 仍 fail closed（无环境进程观测） |
| WorkspaceWrite / WorkspacePatch 变更执行器 | implemented，生产调用 | 生产 router 从持久 Intent 携带受治理 payload + 期望 preimage 并 staging 进 mutation sink；句柄锚定的 no-follow parent/target/staging 操作；逐目标 OS 锁闭合最终 CAS 窗口；write 流式 preimage、patch 显式 preimage 上限、批准 workspace 外的持久原键 attempt/receipt 与 orphan 清理 |
| HttpFetchReadOnly 执行器，走仓库唯一受审计的 Rustls 边界（仅 GET；无调用方 header、不跟随重定向、不继承代理、仅已登记 origin） | implemented，生产调用 | 生产 router 会 staging 钉住的 HTTPS target；已登记 origin 白名单默认为空，因此 origin 登记前 staging fail closed；attempted/completed 状态跨重启保留；回环 TLS 证明仍见 `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| 固定 post-state + verification request + Loop `ACT -> VERIFY` 发布 | implemented，生产调用 | WorkspaceRead 对账后，一个 fenced SQLite 事务校验当前闭合 Effect，并把两个追加式行与登记 Loop 转移一起提交 |
| 独立 verifier + continuation loop | implemented，生产调用 | criteria 只从当前 Acceptance 条件推导；登记 fixed-Effect verifier 生成 CAS 背书证据、持久报告并进入 `VERIFY -> CONTINUE`，随后 checkpoint 绑定的一次性权威经 `CONTINUE -> OBSERVE` 消费，不完成 Task |
| Task candidate + acceptance authority | implemented；公共 C1 native-proven | scheduler materialize/activate governed Task；随后只有最新当前 independent passed report、可重读 CAS evidence、未变 fixed state、闭合 Effect 集合与独立 daemon acceptance principal 才可提交两条登记 Task transition；缺报告、重复 acceptance、open Effect、被取代 report、缺失 CAS evidence 与 stale fixed post-state 均 fail closed |
| 启动恢复 | implemented | 对账已消费交接；当前已准入合同只幂等修复缺失的 Task/Loop/Budget/调度前置，不替换既有权威 |

## 剩余生产接线缺口

原先的引导缺口已在准入路径闭合，且未新增平行调度器：成功的
`TaskApplicationService::admit` 会把合同命名的 Loop、Budget 与 runnable 调度行原子发
布。零 Intent 行现在进入 pre-admission candidate 分支，而非抛出
`MissingEffectBinding`；该趟签发 WIA 后立即返回，不能消费自己刚产生的 worker 权威。
逐行失败彼此隔离，不会中止有界 pass 中的后续行。daemon 现在只在绑定并发布 endpoint
后启动唯一、非重入且可取消的周期 worker；pass 级失败会重试，不能阻止监听。剩余缺口
为：

1. **执行器接线已完成，仅剩两个 fail-closed 执行边界**：六个已登记族现在都有生产请求
   载体。周期 worker 会把无参数 WorkspaceRead、带 query 的 WorkspaceSearch、带 preimage
   的 WorkspaceWrite/Patch（query、payload 与期望 preimage 均携带于持久 Intent）、有界
   ProcessCheck 与 origin 门控的 HttpFetchReadOnly 经持久 Effect 协议从生产派发。
   ProcessCheck 在 daemon 受监督进程 registry 接线前 dispatch fail closed，HttpFetchReadOnly
   在 origin 登记前 staging fail closed——两者都不臆造输入、也不绕过 Effect 协议。
2. **Task 完成已实现且公共 C1 已经 native 证明**：P2-T14 代码沿用已登记的
   `completion_claim` / `fixed_post_state` / `verification_report` /
   `acceptance_decision` 槽位；canonical decision bytes 位于 Artifact CAS，
   daemon-private acceptance principal 与 worker/verifier 身份分离；SQLite 在
   两条 transition 事务内都重查 currentness 与完整 Effect 集合。exact native
   `95f402d3`（已合并 `main@b30386be`）通过 scheduler authority 57/57、
   verification executor 12/12 与 Clippy。全部 D02 负例通过：缺报告/非权威、
   重复 acceptance、open Effect、被取代 report、缺失 CAS 与 stale fixed
   post-state。其他 Tool 请求载体仍未接线。

跨模块细节：调度闭合把 `RECONCILED/VERIFIED/VERIFY_FAILED` 视为已闭合，而管理面
stop 把它们计为 pending——接线时须记住这一有意的保守不对称。

当上述任何一项变化时，须在同一 PR 内更新本页（以及
[`user/tasks-and-execution`](../user/tasks-and-execution.md) 与
[`ref.capability-status`](../reference/capability-status.md)）——本页指纹会强制这次
复核。
