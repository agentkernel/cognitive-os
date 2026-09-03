---
doc_id: dev.execution-chain-status
locale: zh-CN
kind: concept
audience: [developer, ai]
status: partial
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
    symbols: ["PeriodicSchedulerWorker", "serve_personal_loopback"]
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick_with_store"]
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/worker.rs
  - path: personal/apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: personal/apps/kernel-server/src/personal/registered_check/mod.rs
  - path: personal/apps/kernel-server/src/personal/verification_executor.rs
  - path: personal/apps/kernel-server/src/personal/campaign_observation.rs
    symbols: ["CampaignMutationObservationService", "CampaignExternalStateFixture"]
  - path: personal/apps/kernel-server/src/personal/fault_profile.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/pinned_https.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/observation.rs
    symbols: ["handle"]
  - path: personal/crates/cognitive-store/src/sqlite/protocol.rs
    symbols: ["insert_intent"]
  - path: personal/crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: personal/crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
tests:
  - personal/apps/kernel-server/src/personal/p2_t17_a7_failure_first.rs
  - personal/apps/kernel-server/src/personal/scheduler_authority/tests.rs
  - personal/apps/kernel-server/src/personal/tool_executor/tests.rs
  - personal/apps/kernel-server/tests/p2_t16_registered_check.rs
  - personal/apps/kernel-server/tests/p2_t24_effect_fault.rs
  - personal/apps/kernel-server/tests/p2_t25_tool_lifecycle.rs
  - personal/apps/kernel-server/tests/p2_t26_observation_plane.rs
  - personal/apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - personal/apps/kernel-server/src/personal/fault_profile.rs
  - personal/crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
fingerprint: "sha256:616ca543e4d78a990c5a08fae7584ae42c16c9c8eabc1364cafa23f64dfd9035"
non_claims:
  - 本页把缺口记录为记录基线上的事实；既不预测排期，也不贬低已测组件。
  - A7 评测 fixture 与本地/CI 观察证据不得升格为 Gate、release、Profile、B01 或 EVAL-003 结果。
---

# 执行链状态

P11-T08 Routine occurrence 复用本 daemon 调度器（`scheduler_entries`，`task://personal/routine/{occurrence_id}`）。没有第二套 Temporal 调度器。

全手册最怕漂移的一页。设计链路：

调度 lease → 封存 Context → Pi candidate → candidate 准入（Intent + Effect + 一次
性 WIA）→ 受治理工具执行 → 独立验证 → verified continuation 或 ceiling STOP。

## 各环节今天的状态

| 环节 | 状态 | 证据 |
|---|---|---|
| 调度持久化、CAS lease、fencing、上限 | implemented | store 调度测试；`SchedulerService` 上限测试 |
| Task 准入调度引导 | implemented | 单个 fenced SQLite 事务发布 TaskContract + `START` Loop + 硬 Budget + 当前 epoch runnable 行；含崩溃/重复/回滚负例 |
| daemon 周期调度 worker | implemented | 仅在绑定/发布 endpoint 后启动；唯一固定延迟串行 worker 拒绝重入、在 pass 错误后继续，顺序退出时取消并 join |
| Pi 之前封存 ContextRequest/View、逐 body 重授权 | implemented | kernel-server scheduler_authority 真 SQLite 测试；生产路径还会在当前 forget/revoke 与 digest 重验后装入合格 Memory/Skill 钉，并用这些钉替换相同正文的普通 workspace 源，使受治理身份到达 Pi |
| 一次性私有 socket 上的受限 Pi candidate 进程 | implemented | pi-agent-adapter 协议/启动测试 |
| candidate 准入捆绑（Intent + Effect@PROPOSED + WIA + loop DECIDE→ACT，全或无） | implemented | `p2_t03_worker_authorization.rs` |
| WorkspaceRead 执行器（persist-before-dispatch、原键对账） | implemented，生产调用 | 周期 worker 重载 WIA/candidate/Intent/持久 descriptor，重查精确调度 lease 与当前授权，在 daemon 数据 workspace 下 staging，并进入既有 Effect 协议；中断后的 leased 行只查询原键且绝不重复派发 |
| WorkspaceSearch 执行器 | implemented，生产调用 | 生产 router 从持久 Intent 携带受治理 query 并 staging 进 search sink；句柄相对 no-follow 打开、打开后类型/reparse 校验，并在枚举时执行访问上限 |
| ProcessCheck 执行器 | implemented，生产调用 | 生产 router 会 staging 有界 process check；在 daemon 受监督进程 registry 接线前 dispatch 仍 fail closed（无环境进程观测） |
| RegisteredCheckRun 执行器 | implemented，生产调用 | 调用载荷严格只有 `check_id`；daemon 不可变目录固定当前二进制 helper、argv、workspace-root cwd、空环境、超时、输出/进程/写入/网络边界与 descriptor digest。冻结目录绑定 `c2a.repair.typescript`（descriptor_version 2，含公开与 hidden 测试）与 `c2a.repair.rust`；oracle 是文件 digest 相等，因此削弱 hidden 测试即使源文件与公开测试完好也会失败。Intent/Effect 在 spawn 前进入持久 `EXECUTING`，原键状态跨重启保留，有界输出进入 CAS Evidence 并由登记的独立 verifier 校验 |
| WorkspaceWrite / WorkspacePatch 变更执行器 | implemented，生产调用 | 生产 router 从持久 Intent 携带受治理 payload + 期望 preimage 并 staging 进 mutation sink；句柄锚定的 no-follow parent/target/staging 操作；逐目标 OS 锁闭合最终 CAS 窗口；write 流式 preimage、patch 显式 preimage 上限、批准 workspace 外的持久原键 attempt/receipt 与 orphan 清理。期望 preimage `digest:sha256:<64 hex>` 可以是带域前缀的 workspace-image digest，也可以是文件原始字节的 SHA-256（sha256sum / P-arm 形式）。独立 verification 只在 Effect 已 RECONCILED 后开始 |
| HttpFetchReadOnly 执行器，走仓库唯一受审计的 Rustls 边界（dispatch 仅 GET；无调用方 header、不跟随重定向、不继承代理、仅已登记 origin） | implemented，生产调用 | 生产 router 用 task/campaign origin 登记表 staging 钉住的 HTTPS target；白名单默认为空，因此 management 钉住精确 HTTPS origin（`host` 或 `host:port`）之前 staging fail closed；attempted/completed 状态跨重启保留；回环 TLS 证明仍见 `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs` |
| 公开钉住 HTTPS origin 登记表 | implemented，HTTP 调用；生产咨询 | management `GET/POST /management/resource/v1/http-origin` 需 campaign 授权（`P2-T25` 或 `PERSONAL-PERF-EVAL-*`）；task 调用方被拒绝。钉从不携带凭据、header 或 body。生产 HttpFetchReadOnly 按 Intent `task_ref` 咨询该表 |
| 固定 post-state + verification request + Loop `ACT -> VERIFY` 发布 | implemented，生产调用 | WorkspaceRead 对账后，一个 fenced SQLite 事务校验当前闭合 Effect，并把两个追加式行与登记 Loop 转移一起提交 |
| 独立 verifier + continuation loop | implemented，生产调用 | criteria 只从当前 Acceptance 条件推导；fixed-Effect 与 RegisteredCheck verifier 只接受各自登记身份。RegisteredCheck 从 CAS Evidence 重校验精确 descriptor/file digest 和全部安全观察；通过的报告进入 `VERIFY -> CONTINUE`，随后 checkpoint 绑定的一次性权威经 `CONTINUE -> OBSERVE` 消费，不完成 Task。WorkspaceRead 配 fixed-Effect verifier 仍发布 `ACT -> VERIFY`。在 RegisteredCheck 收口的 Task 上，闭合的中间 WorkspaceWrite/Patch/Search Effect 则走登记边 `ACT -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE`，以便后续 tick 准入 RegisteredCheckRun；只有该 check 的独立 verification 可以完成 Task |
| Personal 2.0 Attempt 产物 verifier（`verifier://personal/attempt-artifact`）+ 末环验收 | implemented，生产调用（P13-T04） | 写托管 Attempt 终态的 broker 线程把每个 `DeliverableDraft` 候选交给同一 daemon CAS，并以 `principal://personal/independent-verifier` 运行这一登记 verifier 身份：只做确定性重读（CAS digest、来源帧绑定、Attempt 终态、UTF-8 / 非空 / 无 secret 形状）；child 的 `response done`、exit code 与文本记为 `not-used`；evidence 只追加，报告放在 CAS。P11-T03 StageTestPassed 由该 evidence + 真实就位 + CAS 重读推导（无调用方 `passed`）；run 验收是 `run-acceptance` ApprovalPreview，不在末环即拒绝；这里不触碰上表的 core Task/Effect verifier 路径，也不完成任何 core Task |
| A7 评测回环外部变更观察 | implemented，仅测试调用 | 评测自有幂等 fixture（有界 mutate/query/reset/cleanup 与持久请求/查询计数）；persist-before-dispatch Effect；默认关闭的授权故障点；持久变更后丢失应答时，重启只查询原键，以一次实际变更、零第二次 POST 完成对账；绑定独立验证且 `acceptance_ref` 保持为空。本地/fixture 证据不是 Gate、release、Profile、B01 或 EVAL-003 结果 |
| 公开 Effect 历史与默认关闭 fault profile | implemented，HTTP 调用；生产咨询 | task 通道 `GET /task/effects` 返回不透明 original-key digest、stage、outcome/reconcile class、mutation count 仅 0/1 或缺省，以及 report refs，不含 receipt/参数；management `POST/GET /management/resource/v1/fault-profile` 默认关闭且需 campaign 授权；task 调用方被拒绝。生产 native dispatch 在四个固定点咨询已持久化 profile；缺失、默认关闭与未授权文件内容永不注入。重启只查询原 idempotency key；replacement key 不能绑定第二条 Intent；Indeterminate/open Effect 永不完成 Task |
| 公开 Tool lifecycle、Agent 暴露与选择收据 | implemented，HTTP 调用 | management `GET/POST /management/resource/v1/tool*` overlay `enabled`/`disabled`/`quarantined`/`revoked`，不改 descriptor digest；`agent_exposed` 跟随 overlay 与已装配 executor 就绪。task 调用方不能变更 lifecycle。`GET /task/resource/v1/tool/exposure` 返回最窄暴露集合与 digest；`POST /task/resource/v1/tool/selection` 仅对该 digest 且已暴露的 operation_id 记录收据。prompt/body/receipt 重述与过期/扩权 candidate digest 失败闭合 |
| Task candidate + acceptance authority | implemented；公共 C1 native-proven | scheduler materialize/activate governed Task；随后只有最新当前 independent passed report、可重读 CAS evidence、未变 fixed state、闭合 Effect 集合与独立 daemon acceptance principal 才可提交两条登记 Task transition；缺报告、重复 acceptance、open Effect、被取代 report、缺失 CAS evidence 与 stale fixed post-state 均 fail closed |
| 启动恢复 | implemented | 对账已消费交接；当前已准入合同只幂等修复缺失的 Loop/Budget/调度前置，不替换既有权威 |

## 剩余生产接线缺口

原先的引导缺口已在准入路径闭合，且未新增平行调度器：成功的
`TaskApplicationService::admit` 会把合同命名的 Loop、Budget 与 runnable 调度行原子发
布。公开 `POST /task/admit` 还会持久化 daemon 自有的 Context 授权事实与租户
`personal` 撤销 epoch。零 Intent 行现在进入 pre-admission candidate 分支，而非抛出
`MissingEffectBinding`；该首个 tick 用封存 ContextView 把 Loop 从 `START` 走到
`DECIDE`，准入一条私有 Pi candidate，签发 WIA 后立即返回，不能消费自己刚产生的
worker 权威，也不获取调度 lease。后续 tick 在 lease 下重载持久 WIA 并激活 Task。
逐行失败彼此隔离，不会中止有界 pass 中的后续行。daemon 现在只在绑定并发布 endpoint
后启动唯一、非重入且可取消的周期 worker；pass 级失败会重试，不能阻止监听。live HTTP
`TaskApi` 克隆 daemon 持有的 `SqliteAuthorityStore` 句柄，使 tick 能看到 admit 持久化的
Context 事实；每次请求另开 writer 即 EVAL-006 skip。合同 `max_retries` 为 0 仍允许
首次调度派发：retry count 0 不是已达天花板，后续 WIA tick 可以获取 lease，而不是在
没有 checkpoint 时调用 `stop_for_ceiling`。剩余缺口
为：

1. **执行器接线已完成，覆盖全部七个已登记族**：P2-T10 的原六族加 P2-T16 的
   RegisteredCheckRun 都有生产请求载体。周期 worker 会把无参数 WorkspaceRead、带
   query 的 WorkspaceSearch、带 preimage 的 WorkspaceWrite/Patch、有界 ProcessCheck、
   origin 门控的 HttpFetchReadOnly 与仅含 `check_id` 的 RegisteredCheckRun 经持久
   Effect 协议从生产派发。ProcessCheck 在 daemon 受监督进程 registry 接线前 dispatch
   fail closed，HttpFetchReadOnly 在 campaign 授权 origin 被钉住前 staging fail closed——两者都不臆造
   输入、也不绕过 Effect 协议。
2. **Task 完成已实现且公共 C1 已经 native 证明**：P2-T14 代码沿用已登记的
   `completion_claim` / `fixed_post_state` / `verification_report` /
   `acceptance_decision` 槽位；canonical decision bytes 位于 Artifact CAS，
   daemon-private acceptance principal 与 worker/verifier 身份分离；SQLite 在
   两条 transition 事务内都重查 currentness 与完整 Effect 集合。exact native
   `95f402d3`（已合并 `main@b30386be`）通过 scheduler authority 57/57、
   verification executor 12/12 与 Clippy。全部 D02 负例通过：缺报告/非权威、
   重复 acceptance、open Effect、被取代 report、缺失 CAS 与 stale fixed
   authority。报告、checkpoint、continuation 或 A7 评测观察都不完成 Task；验收仍属
   P2-T14。A7 fixture/本地证据不得升格为 Gate、release、Profile、B01 或 EVAL-003
   post-state。其他 Tool 请求载体仍未接线。
3. **受治理软件修复 journey 已是一条 Task（P2-T22/D02）**：在 RegisteredCheck
   收口的 Task 上，中间 mutation Effect 闭合后 Loop 经登记边回到 `DECIDE`；后续
   tick 在 workspace capability 下准入仅含 `check_id` 的 RegisteredCheckRun，只有
   该 check 的独立 verifier 与 acceptance 可将 Task 标为 `COMPLETED`。公共 C1
   WorkspaceRead 配 fixed-Effect verifier 的路径不变。同一合同 epoch 上有多条
   Intent 时，未消费 WIA 选出当前 Intent，而不是把集合判为 ambiguous。Journey
   测试从合同钉住的 Loop 对象读取 `DECIDE`。hidden-test 被掏空、公开
   测试被削弱、越界写全部 fail closed。D03 仍拥有 exact-revision linux-002
   restart/unknown-outcome/resource/secret/cleanup 矩阵。

跨模块细节：调度闭合把 `RECONCILED/VERIFIED/VERIFY_FAILED` 视为已闭合，而管理面
stop 把它们计为 pending——接线时须记住这一有意的保守不对称。
O2/O3/O4/O5/O13 观测是 task 通道只读平面
（`GET /task/observation?family=o2|o3|o4|o5|o13&task_ref=…`）。O2–O4 样本由 daemon
写入 `personal-observation-plane.json`。O5 复用有界 `GET /task/effects` 历史。O13
是持久审计游标回放，过期游标、缺失事件、digest 断裂与序列缺口失败闭合。空
collector 返回带具名 negative control 的 `observed_zero`。这不是第二套 authority
API，也不升格 Gate、release、Profile、B01 或 EVAL 结果。

当上述任何一项变化时，须在同一 PR 内更新本页（以及
[`user/tasks-and-execution`](../user/tasks-and-execution.md) 与
[`ref.capability-status`](../reference/capability-status.md)）——本页指纹会强制这次
复核。
