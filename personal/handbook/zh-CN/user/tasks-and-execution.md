---
doc_id: user.tasks-and-execution
locale: zh-CN
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
  - path: personal/crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
  - path: core/crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["record_user_intent", "mint_schedulable_task_contract"]
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick"]
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/architecture/multi-agent-orchestration.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/project-role-employee.md
  - path: personal/docs/architecture/routine-trigger-missed-run.md
tests:
  - personal/crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - personal/crates/cognitive-store/tests/m5_intent_chain.rs
fingerprint: "sha256:13aa2540b523f9b6f1aaed23cfd586d252296d32a613019414acb2012916a06c"
non_claims:
  - 准入同一趟仍不消费 worker 授权、也不获取调度 lease；那是后续 tick 的事。不作 Gate、release、Profile 或 EVAL 升格。
---

# Task 与执行

## 这里的 Task 是什么

Task 不是"agent 说它做了什么"，而是带持久证据链的受治理对象：

1. **Record** —— 你的原话在任何 AI 解释之前先持久化（`POST /task/intent.record`）。
2. **Interpret** —— 模型可提议目标/假设；实质歧义强制 `clarification_required`。提议
   持久化为 candidate，绝非真相。
3. **Preview** —— daemon 签发 canonical、digest 绑定的合同预览（目标、范围、预算、
   截止、允许工具、验收条件）。
4. **Admit** —— 你接受的正是那个 digest；daemon 在一个 fenced epoch-CAS 事务内铸
   造 TaskContract，并发布其命名的 `START` Loop、硬 Budget 与当前 epoch 的 runnable
   调度行，以及租户 `personal` 的 owner-local Context 授权。之后改主意会 supersede 到新 epoch，并 fence 一切绑定旧 epoch 的事物。

该准入流水线为 `implemented`，也是默认路径上唯一的人工确认点。`GET /task/watch`
提供有界、快照先行的事件流。已认证的 task 调用方还可以读取有界 O2/O3/O4/O5/O13 观测
（`GET /task/observation?family=…&task_ref=…`）和 Effect 历史
（`GET /task/effects?task_ref=…`）；空观测窗口返回具名 `observed_zero`，而不是沉默计数。
O13 审计回放在过期游标或 digest 断裂时失败闭合。

## 执行按设计如何运转——以及今天真正在跑什么

设计链路：调度 lease → 封存 Context → Pi 产出 **candidate** → daemon 将其接纳为
Intent + Effect + 一次性 Worker Iteration Authorization → 受治理工具执行
（persist-before-dispatch）→ 独立验证 → 循环继续或 STOP。

今天准入会持久入列完整的调度引导，包括 owner-local Context 授权，使后续第一趟 pass
能解析 Context。零 Intent 工作现在会在把 Loop 从 `START` 走到 `DECIDE` 后到达
candidate 准入，并把新 worker 授权留给后续获取调度 lease 的 pass。唯一非重入周期 worker 会在 daemon 开始监听后启动，因此后续 pass 可看
到本进程接纳的 Task；pass 错误不终止监听，顺序退出会取消并 join worker。**daemon
的公共 C1 completion 实现已 native 证明**：生产会派发无参数 WorkspaceRead、
独立验证其固定的已对账 Effect，再只从当前 CAS-backed authority facts 推导 candidate
与最终 acceptance。exact native `22c3f502` 到达 `COMPLETED`。open Effect、被取代
report 与缺失 CAS 负例已写入；stale fixed post-state 仍开放。RegisteredCheck 收口的
软件修复 Task 可在闭合的 WorkspaceWrite 后回到 Loop `DECIDE`，并只在 RegisteredCheckRun
加独立 verification 之后完成。因此已接纳
Task 在权威状态中持久、可观察且 runnable；自主执行仍为 `partial`。开发者细节见
[执行链状态](../developer/execution-chain-status.md)。

## Personal 2.0 Project 工作（`Requires-backend`）

当前持久工作对象仍是 **Task**。OPC 目标在其上增加：

`Project -> Charter/Goal/Plan revision -> Routine -> Task -> Attempt`。

Project setup 在 Owner 确认 daemon 签发的 charter/team/permission/budget/trigger
preview 前保持 draft。每个 active Project 有一个 current manager。管理员可在批准
envelope 内调整 subgoal、Task、顺序、频率和 responsibility；primary goal、team、
budget、Provider、Tool、permission 或 external rule 变化必须形成新的 Owner-confirmed
revision。

每次 retry/fork 创建新的 Attempt，并保留旧 failure/evidence。Routine Trigger 可为
manual、scheduled 或 qualified event；同一 Routine 不 overlap，只保留 latest pending
occurrence，记录 coalesced/missed work，并在 offline 后对 consequential catch-up 再次
询问。

Digital Employee 通过 daemon-owned Task、artifact 与 handoff 协作。DSH 是目标默认
runtime，但 process output 与 engine checkpoint 仍只是 observation。这些
Project/Routine/Employee capability 当前不存在，也不重命名现有 Task row。

## 构造上绝不可能发生的事

- Provider 回复、Pi `agent_end`、工具退出 0、进程退出、worker self-report 或 stale
  verifier report 永远不是 Task 完成。必须同时具备当前独立验证、未变 fixed state、
  闭合 Effects、可重读 evidence 与独立 daemon acceptance authority。
- 未知的外部结果绝不换新身份盲重试——对账复用原幂等键。
- 预算与截止是派发前检查的**含端点**硬轨。
