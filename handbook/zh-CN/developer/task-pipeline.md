---
doc_id: dev.task-pipeline
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi", "persist_owner_local_context_authorization"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService", "contract_preview_digest"]
  - path: crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["mint_schedulable_task_contract", "validate_context_request_binding"]
contracts:
  - specs/schemas/task-preview-request.schema.json
  - specs/schemas/task-admit-request.schema.json
  - specs/schemas/task-contract.schema.json
tests:
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - apps/kernel-server/tests/p2_t24_effect_fault.rs
  - apps/kernel-server/tests/p2_t26_observation_plane.rs
  - apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
fingerprint: "sha256:46a8c8944033b536dc39b4d03db095f7304f3267d919fdbd0f09e57c528c1395"
non_claims:
  - 准入仍不消费 worker iteration authorization，也不获取调度 lease；那是后续 tick 的事。
---

# Task 流水线

HTTP（`TaskApi`，task 通道）→ `KernelTaskApplicationService` → kernel intent
chain → SQLite，线上使用生成的 request/result DTO。

| 操作 | 路由 | kernel 组合 |
|---|---|---|
| `propose` | `POST /task/intent.record` | `record_user_intent` —— 原文先持久固定 |
| `clarify` | `POST /task/intent.interpret` | `record_interpretation_candidate` —— 状态推导，绝不挑选 |
| `preview` | `POST /task/preview` | 对类型化草稿做本地 canonical digest（域 `cognitiveos.personal.task-contract-preview`）；不持久化 |
| `admit` | `POST /task/admit` | 重算 preview digest（漂移 → `PreviewDigestMismatch`）→ 持久化 daemon 自有的 owner-local Context 授权事实与租户 `personal` 撤销 epoch → `admit_interpretation` → 一个 fenced 合同 epoch-CAS 事务发布 TaskContract + `START` Loop + 硬 Budget + runnable 调度行 |
| evidence | `GET /task/evidence?task_ref=...` | 从 SQLite authority 与 Artifact CAS 重建有界脱敏的生命周期、Effect 对账类别、current verification/Artifact 可用性、acceptance transition 与持久事件游标 |
| effects | `GET /task/effects?task_ref=...` | 重建有界 Effect 历史（不透明 original-key digest、stage、outcome/reconcile class、mutation count 仅 0/1 或缺省、report refs），不含 receipt 或原始参数 |
| observation | `GET /task/observation?family=o2\|o3\|o4\|o5\|o13&task_ref=...` | 有界 O2/O3/O4/O5/O13 只读平面；空 collector 返回 `observed_zero` 与具名 negative control；O5 复用脱敏 Effect 历史；O13 是持久审计游标回放；不泄漏 body/capability |
| tool exposure | `GET /task/resource/v1/tool/exposure?task_ref=...` | 当前最窄 Agent 暴露集合与 `exposure_digest`；额外 prompt/body/receipt 查询键失败闭合 |
| tool selection | `POST /task/resource/v1/tool/selection` | 有界收据：candidate_set_digest 必须等于当前暴露，所选 operation_id 必须已被暴露，禁止 prompt/body 重述 |
| watch | `GET /task/watch` | 快照先行的有界流（进程本地 128 事件重放；过期 `resume_from` → `TASK_WATCH_RESUME_STALE`） |

合同版本：携带 `context_request_ref` 的铸造在 `validate_context_request_binding`
校验持久 ContextRequest 行（task/digest/type/perspective 一致性）后产出 schema
`cognitiveos.task-contract/0.4`；否则 v0.3。合同钉住 loop/budget ID、允许的状态域与
工具、截止与上限，其自身 ID 成为 WIA 命名空间根。

准入发布在 authority SQLite 文件内全有或全无。靠后的 Loop/Budget/调度冲突会回滚合同
与事件；成功响应后崩溃重开会看到全部成员。Owner-local Context 授权事实是租户
`personal` 的 daemon 策略，不是调用方能力通道；首个调度 tick 在 Pi 之前用封存
ContextView 把 Loop 从 `START` 走到 `DECIDE`。live daemon 上 HTTP 适配器克隆进程持有的
authority-store 句柄，使该 tick 能看到这些事实；另开连接的 in-process
`TaskApi::handle` fixture 不是这条路径。它不创建 candidate Intent/Effect，也不运行
Tool——周期 worker 路径仍是独立接线。
daemon 启动时可从当前不可变合同重构同一引导，并幂等恢复所缺 Loop、Budget 或调度行；
既有权威绝不重置。
该已准入 Task 之后解析 Context 时，daemon 会把当前合格 Memory 与精确 Skill pin
装入封存视图并写入只追加消费记录；后续会话无需聊天重述即可复用这些钉，遗忘、撤销
或 digest 漂移一律失败闭合。这不完成 Task。后续 lease tick 把 retry count 0 视为尚未
达到重试天花板，即使合同 `max_retries` 为 0。
调度 pass 首次看到零 Intent 行时，会选择 pre-admission candidate 路径，而非把 Effect
绑定缺失当作损坏。candidate 准入可签发一份 WIA，但同一 pass 会返回且不消费它；后续
pass 必须在调度 lease 下重新加载。

已实现未暴露：`control`（经 `supersede_task_contract` 的更正/取消）与
`query_intent` 存在于服务 trait 且测试完整，但 Personal HTTP 尚无路由调用。因此经
HTTP 的更正不可用；fencing 机制（`INTENT_VERSION_SUPERSEDED`）在 kernel/store 层已
充分测试。

同样诚实：未知 `POST /task/*` 路径返回 200 加 "no Task API operation matched" 注记
（非 404）；watch 事件源仍是进程本地的。独立 evidence 查询跨重启持久且只读；它不返回
candidate 参数、workspace 字节、receipt、Provider/Pi 内容或 secret。

下游原生工具 staging 只接受与 daemon 不可变目录条目完全相等的 descriptor。执行 attempt
仍归 Effect 所有：不确定 HTTP attempt 跨重启后保持 indeterminate（持久状态缺失同样
fail indeterminate），workspace 变更完成则要求批准 workspace 外的状态存储中存在绑定
原始幂等键的持久 receipt。RegisteredCheckRun 新增一个生产载体，其载荷只有
`check_id`；独立的不可变目录固定可执行文件、argv、cwd、空环境以及全部
进程/输出/写入/网络边界。结果只形成 CAS Evidence，仍须登记的独立 verifier。
HttpFetchReadOnly 只对 task/campaign 范围的钉住 HTTPS origin 登记表 staging
（默认空；GET/HEAD；无凭据、不跟随重定向、不继承代理）。显式端口是精确 origin
钉的一部分。
中间 WorkspaceWrite 在 RegisteredCheck 收口的 Task 上经登记边把 Loop 送回 `DECIDE`
后，后续 tick 可准入 RegisteredCheckRun；只有该 check 的独立 verification 与
acceptance 能完成 Task。公共 C1 WorkspaceRead 配 fixed-Effect verifier 仍走
`ACT -> VERIFY`。
这些执行器保证都不会把准入、Tool receipt 或相同 workspace 字节提升为 Task 完成。
