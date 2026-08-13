---
doc_id: user.tasks-and-execution
locale: zh-CN
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
  - path: crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["record_user_intent", "mint_schedulable_task_contract"]
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick"]
tests:
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - crates/cognitive-store/tests/m5_intent_chain.rs
fingerprint: "sha256:fac528b42b5e40976dc4f56892e1ab4c3755d34545e530620712ed5df8e0cb68"
non_claims:
  - 不声明已接纳的 Task 今天能自主执行；执行流水线的组件证据存在于聚焦测试中，而非端到端产品路径。
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
   调度行。之后改主意会 supersede 到新 epoch，并 fence 一切绑定旧 epoch 的事物。

该准入流水线为 `implemented`，也是默认路径上唯一的人工确认点。`GET /task/watch`
提供有界、快照先行的事件流。

## 执行按设计如何运转——以及今天真正在跑什么

设计链路：调度 lease → 封存 Context → Pi 产出 **candidate** → daemon 将其接纳为
Intent + Effect + 一次性 Worker Iteration Authorization → 受治理工具执行
（persist-before-dispatch）→ 独立验证 → 循环继续或 STOP。

今天准入会持久入列完整的调度引导，后续每个环节也存在且有聚焦测试（lease CAS 与
fencing、封存 ContextView、candidate 准入捆绑、带未知结果对账的六族已装配 Tool 执行
器、独立 verifier 接缝）。**但 daemon 尚未自主驱动整条链**：它只在接受 Task 准入前
跑一次调度，生产代码尚未调用 Tool 执行器与 verifier。因此已接纳 Task 在权威状态中
持久、可观察且 runnable；自主执行仍为 `partial`。开发者细节见
[执行链状态](../developer/execution-chain-status.md)。

## 构造上绝不可能发生的事

- Provider 回复、Pi `agent_end`、工具退出 0 或进程退出永远不是 Task 完成（要求独立
  验证）。
- 未知的外部结果绝不换新身份盲重试——对账复用原幂等键。
- 预算与截止是派发前检查的**含端点**硬轨。
