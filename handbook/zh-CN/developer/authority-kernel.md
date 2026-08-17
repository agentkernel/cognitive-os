---
doc_id: dev.authority-kernel
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-kernel/src/engine.rs
    symbols: ["TransitionEngine", "prepare_object_admission", "prepare_transition", "validate_registered_transition"]
  - path: crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["record_user_intent", "admit_interpretation", "mint_schedulable_task_contract", "prepare_task_execution_bootstrap", "supersede_task_contract", "verify_task_binding_current"]
  - path: crates/cognitive-kernel/src/effects.rs
    symbols: ["EffectProtocol", "mint_intent", "COMMIT_SINKS"]
  - path: crates/cognitive-kernel/src/authz.rs
    symbols: ["authorize", "revalidate_grant", "capability_and_revocation_current"]
  - path: crates/cognitive-kernel/src/budget.rs
    symbols: ["check_and_debit"]
  - path: crates/cognitive-kernel/src/recovery.rs
    symbols: ["RECOVERY_ORDER", "run_recovery"]
  - path: crates/cognitive-kernel/src/harness.rs
    symbols: ["return_to_decide_after_closed_effect", "advance_start_to_decide_after_context_view"]
contracts:
  - specs/transitions/effect.transitions.json
  - specs/transitions/task.transitions.json
  - specs/registry/errors.yaml
tests:
  - crates/cognitive-kernel/tests/engine_gate.rs
  - crates/cognitive-kernel/tests/governance_gate.rs
  - crates/cognitive-store/tests/m4_effects.rs
  - crates/cognitive-store/tests/m4_recovery.rs
fingerprint: "sha256:250dff539a72f06829c1e1f5b90b62eb592eebbe4814281c4a056fb64aa56368"
non_claims:
  - 内核正确性证据是聚焦测试证据，不构成 Gate、release 或 Profile 结论。
---

# 权威内核

`cognitive-kernel` 是确定性内核：无 HTTP、无 SQLite、无模型 SDK。本 crate 内的原生
Tool 预执行校验器是纯函数（HTTPS origin 形态允许把可选显式端口作为精确 origin 的
一部分；userinfo、query 与 fragment 仍禁止）。适配器实现其 port
trait；参考适配器是 `cognitive-store`。守护进程私有的
`MemorySkillConsumptionStore` 端口只记录用于 Context 装载的精确 Memory/Skill 钉，
不授予客户端写权限，也不完成 Task。

## 十步转移门

`TransitionEngine::prepare_transition` 按固定顺序校验：(1) 表 pin（注册转移表的版本
+ canonical digest）；(2) 权威行加载；(3) from 状态现时性；(4) `expected_version`
CAS；(5) 边查找 `(from, to, reason)`；(6) 每个守卫必须在调用方声明集合中——缺失即
fail-closed；(7) 必需证据以强引用提供；(8) 可选硬预算扣减（纯函数
`check_and_debit`，并入同一提交）；(9) schema 形状的提交记录 + canonical 事件；
(10) 单原子 `TransitionCommit`（对象 CAS + 事件 + 记录 + 预算 CAS + outbox +
fencing epoch）。拒绝携带权威状态/版本与排序后的合法出口，并确定性映射到注册错误码
（`STATE_CONFLICT`、`DIGEST_MISMATCH`、`STATE_STORE_UNAVAILABLE`、
`RESOURCE_BUDGET_EXHAUSTED`，以及钉住的 `EFFECT_OUTCOME_UNKNOWN` 特例）。

复合原子事务仅有三个受认可的准备接缝：纯校验器
`validate_registered_transition`（供 candidate 准入）、`PreparedTransition`
（在 verified-continuation 消费内原样提交），以及
`TransitionEngine::prepare_object_admission`（把注册初态的对象准入原样放进不可
分割的复合权威事务）。三者都保持精确的已校验提交。

## Intent chain

`record_user_intent` 在解释前固定原文；解释候选以提案持久化，其状态**推导**得出
（实质歧义 ⇒ `clarification_required`）；`admit_interpretation` 是 admitted 解释的唯
一构造器（权威身份 + 精确 digest）；`mint_task_contract` 要求可判定的验收条件并在合
同 epoch CAS 下铸造。生产路径 `mint_schedulable_task_contract` 还会在同一个 fenced
存储事务内发布合同事件及其 `DRAFT` governed Task、合同命名且处于 `START` 的 Loop、
合同命名的硬 Budget，以及当前 epoch 的 runnable 调度行；成功准入不可能只暴露其中一部分。
`supersede_task_contract` 使用同一可调度发布，fence 旧 epoch 工作（在 mint 与
dispatch 两个 sink 上 `INTENT_VERSION_SUPERSEDED`）并对在途 Effect 分类以待对账。
启动修复对当前不可变合同调用同一纯组合 `prepare_task_execution_bootstrap`；它可恢复
缺失前置，但不能替换既有 Task/Loop/Budget/调度权威。当前 daemon-issued WIA 会在 Tool
I/O 前真实推导登记的 `DRAFT -> READY -> ACTIVE` Task guards。

## Effect：七性质、四 sink

`mint_intent` 执行持久幂等算术：同键同 canonical 参数 digest 即重放；同键不同
digest 为 `EFFECT_IDEMPOTENCY_CONFLICT`。`EffectProtocol` 驱动
PROPOSED→AUTHORIZED→EXECUTING→…→COMMITTED，守卫只从持久重载推导
（`intent_durably_persisted`、`capability_and_revocation_current`、
`verification_still_current`）；dispatch 在外部调用**之前**先提交 EXECUTING；未知结
果用原键对账或隔离。四个提交 sink（executor、权威提交、准入+outbox、checkpoint）都
在存储事务内复核写者 fencing epoch。验证入口同样是复合权威提交：当前闭合 Effect
pin、其 verification request 与 Loop `ACT -> VERIFY` 要么一起持久化，要么一起回滚。
非 RegisteredCheckRun 的中间闭合 Effect 在 RegisteredCheck 收口的 Task 上则从持久
Effect/合同事实走 `ACT -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE`，以便准入下一
candidate 而不完成 Task。公开 admit 的 Task 把 Loop 留在 `START`；封存 ContextView
存在后，`LoopDriver::advance_start_to_decide_after_context_view` 在 Pi 之前走
`START -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE`。该行走不是 Task 验收。
Task 验收属于独立权威：candidate 与最终 acceptance transition 都先经同一 deterministic
engine 准备，再由 SQLite 在事务内重查当前合同 epoch、完整闭合 Effect 集合、fixed
post-state、最新 passed report 与 fencing；最终 principal 是 daemon-private acceptance
authority，绝不是 worker 或 verifier。

## 授权与预算

`authorize` 走六步 fail-closed（authn/链 → 租户/成员 → 能力交集 + 撤销现时性 → 显
式拒绝优先 → lease 窗口 → scope/purpose/action）。拒绝对存在性安全（denied 与
not-found 字节相同）。`revalidate_grant` 在 dispatch 与 commit 时点复核 F-007 竞态。
预算是九个注册维度上的纯整数台账。

## 恢复

`RECOVERY_ORDER` 固定八步（barrier → 身份/epoch → fence → 重放 → 对账 → 重授权 →
重解析 context → 恢复 loop）；`run_recovery` 对 AUTHORIZED 工作用原键恰好一次重派
发，把 EXECUTING 压入 OUTCOME_UNKNOWN 再对账，不确定者隔离，且只恢复 checkpoint 校
验通过（epoch 更旧、水位在重放历史内）的 loop。
