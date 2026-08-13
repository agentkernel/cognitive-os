---
doc_id: dev.task-pipeline
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
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
fingerprint: "sha256:aedb5d338acec8c1a8c0ed83ebab68a3b63e2705b04a62adf5071ebade227d26"
non_claims:
  - 准入不会启动自主执行；该缺口记录于执行链状态页。
---

# Task 流水线

HTTP（`TaskApi`，task 通道）→ `KernelTaskApplicationService` → kernel intent
chain → SQLite，线上使用生成的 request/result DTO。

| 操作 | 路由 | kernel 组合 |
|---|---|---|
| `propose` | `POST /task/intent.record` | `record_user_intent` —— 原文先持久固定 |
| `clarify` | `POST /task/intent.interpret` | `record_interpretation_candidate` —— 状态推导，绝不挑选 |
| `preview` | `POST /task/preview` | 对类型化草稿做本地 canonical digest（域 `cognitiveos.personal.task-contract-preview`）；不持久化 |
| `admit` | `POST /task/admit` | 重算 preview digest（漂移 → `PreviewDigestMismatch`）→ `admit_interpretation` → 一个 fenced 合同 epoch-CAS 事务发布 TaskContract + `START` Loop + 硬 Budget + runnable 调度行 |
| watch | `GET /task/watch` | 快照先行的有界流（进程本地 128 事件重放；过期 `resume_from` → `TASK_WATCH_RESUME_STALE`） |

合同版本：携带 `context_request_ref` 的铸造在 `validate_context_request_binding`
校验持久 ContextRequest 行（task/digest/type/perspective 一致性）后产出 schema
`cognitiveos.task-contract/0.4`；否则 v0.3。合同钉住 loop/budget ID、允许的状态域与
工具、截止与上限，其自身 ID 成为 WIA 命名空间根。

准入发布在 authority SQLite 文件内全有或全无。靠后的 Loop/Budget/调度冲突会回滚合同
与事件；成功响应后崩溃重开会看到全部成员。它不创建 candidate Intent/Effect，也不运行
Tool——周期 worker 路径仍是独立接线。
daemon 启动时可从当前不可变合同重构同一引导，并幂等恢复所缺 Loop、Budget 或调度行；
既有权威绝不重置。

已实现未暴露：`control`（经 `supersede_task_contract` 的更正/取消）与
`query_intent` 存在于服务 trait 且测试完整，但 Personal HTTP 尚无路由调用。因此经
HTTP 的更正不可用；fencing 机制（`INTENT_VERSION_SUPERSEDED`）在 kernel/store 层已
充分测试。

同样诚实：未知 `POST /task/*` 路径返回 200 加 "no Task API operation matched" 注记
（非 404）；watch 事件源是进程本地的——该表面尚未消费持久事件 outbox。

下游原生工具 staging 只接受与 daemon 不可变目录条目完全相等的 descriptor。执行 attempt
仍归 Effect 所有：不确定 HTTP attempt 跨重启后保持 indeterminate（持久状态缺失同样
fail indeterminate），workspace 变更完成则要求批准 workspace 外的状态存储中存在绑定
原始幂等键的持久 receipt。这些执行器保证都不会把准入、Tool receipt 或相同 workspace
字节提升为 Task 完成。
