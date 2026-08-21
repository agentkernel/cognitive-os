# CognitiveOS V2 重构架构决策

- Status: informative architecture research and restructuring proposal
- Date: 2026-08-21
- Scope: CognitiveOS Personal 与 CognitiveOS V2 目标架构
- Evidence baseline: CognitiveOS `main@742a9346e1b544a3addffe3156660a92ac857f6f`;
  DeepSeek Harness `528c682e061696f5a160f363f236ecbf53cbd006`;
  Cordis `8cc9e33fab69e2d0476d126baaf2acb24e6a6ab4`
- Non-normative: this document does not create REQ, schema, Gate, Profile, task,
  or release evidence. Any contract change requires the normal Lane-CTR and plan
  process.

> 本文允许 CognitiveOS 重构。重构对象是架构边界和实现组织，不是为了接入
> DeepSeek Harness 或 Cordis 而削弱 A1-A8、公理合同和现有安全负例。

## Executive Summary

### Fact / 事实

CognitiveOS Personal 已经实现了一个相当完整的 owner-local、single-principal
持久化 Agent control plane：Rust daemon 是唯一 authority writer；SQLite WAL 保存权威
状态；Intent/Effect 使用 persist-before-dispatch、idempotency、fencing、unknown-outcome
reconciliation 和 quarantine；Context 在排序前进行授权和预算过滤；独立 verifier
决定 Task 是否可接受；replay 从 committed append-only event 产生确定性 projection。

主要证据包括：

- `crates/cognitive-kernel/src/effects.rs`
- `crates/cognitive-kernel/src/replay.rs`
- `crates/cognitive-kernel/src/recovery.rs`
- `crates/cognitive-kernel/src/context.rs`
- `apps/kernel-server/src/personal/verification_executor.rs`
- `crates/cognitive-runtime/src/dynamic_tool_ecosystem.rs`
- `crates/cognitive-store/tests/m4_effects.rs`
- `crates/cognitive-store/tests/m4_recovery.rs`
- `crates/cognitive-store/tests/m4_tracer_bullet.rs`

DeepSeek Harness 不是空壳 Harness。它拥有 Cordis 组合的模型、工具、Session、Loop、
Sandbox、Persistence、UI 和 Subagent。Session log 具备事件序列、tool call/result 配对、
compaction surface、合法 fork boundary、parentSession/seedLength lineage、resume 和
崩溃尾部修复。

Cordis 不是普通静态 Plugin loader。它以 Context、Fiber、Service、Dependency injection
和 reversible effect 实现 spatial composability 与 temporal composability。

### Inference / 推断

上一版“尽量不动 CognitiveOS，只在外侧加 Adapter”的结论过于保守。原因是：

1. CognitiveOS 当前的治理能力已经足够成为新架构的核心，但其 `Session`、`Execution`、
   `Task`、`Loop`、`Context`、`Tool`、`Plugin`、`Capability` 之间仍有跨层耦合；
2. DeepSeek Harness 的 conversation ledger 与 CognitiveOS 的 execution ledger 目前没有
   正式 ABI，简单 Adapter 会形成双重状态和重复恢复逻辑；
3. Cordis 的生命周期/依赖图能力适合下沉为 Composition Runtime，而不是继续在
   CognitiveOS Runtime 中复制一套不完整的 Plugin graph；
4. 未来实体 Agent 需要一个比“Tool result”更严格的 Effect/Receipt/Verification 模型。

因此 CognitiveOS V2 应进行**部分核心重构**，但不是推翻治理内核。

### Recommendation / 建议

将 CognitiveOS 重新定位为：

> **Durable Agent Governance Runtime / Agent Control Plane**

V2 的边界如下：

```text
External / Physical World
          ^
          | Effect adapters, receipts, verification
┌─────────┴──────────────────────────────┐
│ CognitiveOS Governance Kernel           │
│ Identity · Scope · Grant · Intent       │
│ Effect · Receipt · Verification          │
│ Event log · Projection · Recovery       │
└────────────────┬───────────────────────┘
                 │ CognitiveOS ABI
┌────────────────┴───────────────────────┐
│ Composition ABI                          │
│ Cordis adapter · native backend · WASM   │
└────────────────┬───────────────────────┘
                 │ Harness ABI
┌────────────────┴───────────────────────┐
│ Harnesses                                │
│ DeepSeek Harness · Codex · Claude · ...  │
└────────────────┬───────────────────────┘
                 │
              Agent / LLM
```

最终推荐不是 `CognitiveOS + Cordis + DeepSeek Harness` 的静态拼接，而是：

```text
CognitiveOS Governance Kernel
        + language-neutral ABI
        + optional Cordis Composition Backend
        + DeepSeek Reference Harness Adapter
        + other replaceable Harnesses and Robot Adapters
```

## 1. Research Method and Evidence Boundary

### Fact / 事实

本研究严格区分：

| 类型 | 含义 |
|---|---|
| Specification | schema、REQ、公理和目标合同 |
| Reference architecture | 文档中的目标模块和设计叙述 |
| Implemented runtime | 当前源码中可执行的实现 |
| Tests | 当前测试实际覆盖的行为 |
| Roadmap | 尚未实现的目标 |

CognitiveOS 当前状态以 `docs/plan/PROGRESS.md` 为准；架构合同以
`docs/governance/AXIOMS.md`、`specs/` 和 accepted ADR 为准。DeepSeek Harness 和 Cordis
均使用完整本地 clone、固定 commit，并通过 Git object connectivity 检查。

### Non-claim / 非声明

本机 TLS 的 Schannel 错误没有被伪称为“已修复”。远程采集采用 `git -c
http.sslBackend=openssl` 的显式路径；当前自动审批/网络限制导致的远程探针失败不影响
固定本地源码证据，但也不产生新的在线来源声明。

## 2. Current CognitiveOS Reality

### Fact / 事实

CognitiveOS Personal 当前是一个 durable control plane，而不是实现为空的白皮书：

```text
Client / Pi / Agent Adapter
          |
          v
Rust daemon (sole authority writer)
          |
  admission + scheduler + CAS/fencing
          |
      SQLite WAL authority store
          |
  events -> projections -> read models
          |
 Intent -> Effect -> Receipt -> Verification -> acceptance
```

已实现的强项：

- 唯一 authority writer 和 candidate/authority 分离；
- Context authorization-before-ranking、预算和显式 loss；
- Intent persist-before-dispatch；
- idempotency key 和 canonical parameter digest；
- fencing epoch；
- `OUTCOME_UNKNOWN` 只能 reconcile 或 quarantine；
- recovery 固定顺序、checkpoint epoch 校验；
- 独立 Verification，不接受 Agent 或 executor 自报完成；
- Tool descriptor、availability、quarantine 和动态 lifecycle；
- sidecar/process/adapter identity 分层；
- SQLite projection replay 的 digest stability。

### Fact / 事实：当前不足

当前缺口不是“没有 Runtime”，而是 Runtime 的边界尚未压缩为一组稳定 primitive：

1. Authority event log 是生命周期投影账本，不是完整 Harness conversation ledger；
2. Session、Execution、Task、Loop 已被正确区分，但跨模块关联字段和恢复 ABI 不统一；
3. Tool、Capability、Plugin、Skill 的语义已有局部实现，正式层次仍不够清晰；
4. Dynamic Tool lifecycle 存在，但通用 composition/dependency graph 不如 Cordis；
5. Multi-Agent 仍为 post-1.0 设计，不应被当前实现暗示为已交付；
6. 未来实体执行的实时、安全、不可逆物理 Effect 尚无完整协议。

### Recommendation / 重构态度

保留已有安全不变量，允许重构以下内部边界：

- 将 `Event` 从“多个模块都能各自解释的记录”重构为明确的治理 Event taxonomy；
- 将 `State` 改为 Event Log 的 projection，而不是与 Event 并列的第二事实来源；
- 将 `Capability` 从 Tool/Plugin descriptor 中抽离为 daemon-issued attenuated grant；
- 将 Harness Session 设为外部 ledger，通过 ABI 关联到 CognitiveOS Execution；
- 将 Plugin dependency/lifecycle 下沉给 Composition Runtime；
- 将 Task/Loop/Execution 的关系改成明确的 containment，而非互相替代。

## 3. DeepSeek Harness Findings

### Fact / 事实

源码基线：`DeepSeek-Harness@528c682e061696f5a160f363f236ecbf53cbd006`。

关键位置：

- `packages/core/session/src/types.ts`
- `packages/core/session/src/index.ts`
- `packages/core/session/src/repair.ts`
- `packages/session/session-persistence/src/coordinator.ts`
- `packages/session/session-persistence-sqlite/src/schema.ts`
- `packages/subagent/subagent/src/continuation.ts`
- `packages/sandbox/sandbox/src/escalation.ts`
- `packages/extensions/tool-cordis/README.md`

Harness Session Log 的 source of truth 是 Session 的追加事件及其 surface projection：

```text
turn/step/message/tool events
          |
          v
append-only session log
          |
          +--> model surface projection
          +--> stats / telemetry
          +--> resume / fork / compaction
```

它能记录：

- raw model messages；
- `tool/call` 和 `tool/result`；
- turn/step boundary；
- request headers、usage、errors；
- surface replacement 和 compaction；
- `parentSession`、`seedLength`；
- crash repair synthetic closers。

### Fact / 事实：Harness 的边界

DeepSeek Harness 的 Cordis dynamic package：

- 是进程内动态实验；
- 不持久安装；
- 不跨重启保留；
- 不自动 promote；
- 其 VM 明确不是 security boundary；
- 应按 bash authority 对待。

DSH approval 是一次性、turn-scoped、fail-closed 的人类审批，不等于 Durable Capability
Grant，也不提供 CognitiveOS 所需的 revocation currency、fencing 和 Effect reconciliation。

### Recommendation / 重构吸收方式

吸收 DSH 的三项能力：

1. Session log 的事件模型和合法 fork/resume；
2. Harness-level crash repair 和 compaction surface；
3. Subagent lineage、continuation 和 sandbox adapter。

不吸收为 CognitiveOS Kernel 的部分：

- model loop；
- conversational surface；
- tool schema registry；
- Cordis dynamic VM；
- one-shot approval；
- Harness 的 process-local plugin authority。

## 4. Cordis Findings

### Fact / 事实

源码基线：`cordis@8cc9e33fab69e2d0476d126baaf2acb24e6a6ab4`。

关键位置：

- `packages/core/src/context.ts`
- `packages/core/src/fiber.ts`
- `packages/core/tests/dispose.spec.ts`
- `packages/core/tests/fiber.spec.ts`
- `packages/loader/src/config/entry.ts`
- `packages/hmr/src/index.ts`
- `packages/hmr/tests/index.spec.ts`

Cordis 的 Context 是：

- service environment；
- dependency injection scope；
- event bus；
- reflection/registry；
- lifecycle effect host。

Temporal composability：

```text
Context enters scope
    -> effect/disposer registered
    -> nested effects collected
    -> scope exits
    -> reverse/LIFO disposal
```

Spatial composability：

```text
provider appears/replaced/disappears
              |
              v
dependency fiber epoch changes
              |
              v
consumer load / active / unload / reload
```

### Inference / 推断

Cordis 的核心价值是“可组合运行时”，不是“安全插件市场”。它适合成为
Composition Backend，但不能成为 CognitiveOS 的 authority store、authorization kernel
或 external effect system。

### Recommendation / 建议

新架构不应在 Rust daemon 中复制一套 Cordis-like Plugin graph。应定义语言无关的
Composition ABI，然后提供：

1. 一个最小 native backend（可服务 daemon/sidecar）；
2. 一个 Cordis adapter（Node/TypeScript Harness）；
3. 后续可选 WASM Component backend；
4. 后续可选 robot capability backend。

## 5. What Should Be Reconstructed

### 5.1 保留：Governance Kernel

保留并收敛为核心边界：

- Identity；
- Scope；
- Capability Grant；
- Intent；
- Effect；
- Receipt/Observation；
- Verification；
- Event append；
- Projection；
- Lease/Fencing/Recovery。

这些能力是 CognitiveOS 与普通 Harness 的真正差异。

### 5.2 重构：State/Event

当前应重构为：

```text
Committed Event Log (source of truth)
          |
          +--> authority projection
          +--> audit projection
          +--> recovery projection
          +--> read model
```

不再将可变 `State` 视为和 Event 平行的事实来源。缓存、索引和 view 都必须可删除、
可重建，并携带 source watermark/digest。

### 5.3 重构：Session/Execution/Task/Loop

不合并为单一对象，而改成明确的层级：

```text
AgentIdentity
  └── HarnessSession (conversation ledger)
        └── Execution (governed run)
              └── TaskContract (goal/acceptance)
                    └── LoopEpoch (bounded iteration)
                          └── Intent -> Effect -> Verification
```

解释：

- Session 可以持续而没有 Task；
- 一个 Session 可以 fork 多个 Execution；
- 一个 Execution 可以执行一个或多个 Task epoch；
- Loop 是 Task 的受预算和 checkpoint 约束的迭代；
- Process 只是 Execution 的宿主观察，不是完成权威。

### 5.4 重构：Plugin/Capability/Tool/Skill

正式分层：

```text
Plugin Package
     |
     +-- provides --> Capability Descriptor
                              |
                              +-- daemon attenuates --> Grant
                                                        |
                                                        +--> Intent
```

规则：

- Plugin 描述实现、版本、依赖、来源和生命周期；
- Capability 描述“能做什么”，不表示“谁可以做”；
- Grant 绑定 principal、scope、purpose、budget、expiry、revocation epoch；
- Tool 是 Capability Provider 的一个实现形态；
- Skill 是可进入 Context 的内容包，不自动授予执行权限；
- Agent 只能提出 Capability Request，不能自授权。

### 5.5 重构：Context/Memory

Context 与 Memory 必须保持不同：

```text
Memory = durable, admitted knowledge
Context = one execution's authorized, bounded view
```

建议三层 Context：

1. **Infrastructure Context**：Cordis service/dependency/lifecycle context；
2. **Governed Context**：daemon 解析的授权、scope、purpose、freshness、budget view；
3. **Cognitive Context**：Harness 渲染给模型的 messages、fragments 和 tool summaries。

## 6. New CognitiveOS V2 Kernel

### Recommendation / 最小内核

如果只保留 8 个 primitive：

```text
1. Identity
2. Scope
3. Event
4. Projection
5. CapabilityGrant
6. Intent
7. EffectReceipt
8. VerificationLease
```

Recovery 由 Event + Lease + Projection 形成；Context 是 Scope 上的 governed projection；
Task、Session、Plugin、Memory、Tool 是上层资源，不进入最小内核。

### Kernel invariants

以下是对既有 A1-A8 的架构重述，不是新增公理：

```text
probabilistic components -> candidates only
authority writer         -> daemon only
external mutation        -> persist Intent/Effect before dispatch
completion               -> independent verification
secret                   -> approved SecretStore only
stale writer             -> fenced
unknown effect           -> reconcile or quarantine
```

### Non-goals of the kernel

Kernel 不包含：

- LLM inference；
- prompt rendering；
- Plugin JavaScript VM；
- UI；
- MCP server implementation；
- ROS 2 executor；
- Kubernetes controller；
- generic distributed workflow engine；
- automatic package marketplace；
- root shell。

## 7. Dual Ledger and Event Model

### Recommendation / 双账本

V2 使用两个相互关联但不合并的 append-only ledger：

```text
Harness Conversation Ledger
  messages, tool calls, chunks, compaction, fork lineage

CognitiveOS Governance Ledger
  grants, intents, effects, receipts, verification, recovery, lifecycle
```

关联字段：

```text
session_id
execution_id
task_id
causation_id
correlation_id
fencing_epoch
```

### Governance Event taxonomy

建议的最小事件族：

```text
identity/*
scope/*
grant/issued | grant/revoked | grant/consumed
intent/recorded | intent/replayed | intent/rejected
effect/authorized | effect/dispatch-started | effect/receipt
effect/outcome-unknown | effect/reconciled | effect/quarantined
verification/requested | verification/reported
task/accepted | task/rejected
execution/checkpointed | execution/resumed | execution/forked
plugin/installed | plugin/activated | plugin/replaced | plugin/quarantined
```

### Event-sourcing decision

**Recommendation / 建议：采用有边界的 Event-Sourced Runtime。**

Event Log 是 governance facts 的 source of truth；projection 是当前状态；索引和缓存是
derived data。不要将每个 model chunk、KV cache 或高频 sensor sample 写入同一治理账本。

## 8. Effect and Verification Refactor

### Recommendation / Effect Model

```text
Candidate
  -> Policy admission
  -> Capability grant check
  -> Persist Intent
  -> Persist Effect
  -> Dispatch with idempotency key
  -> Receipt / Unknown
  -> Reconcile or Quarantine
  -> Fixed post-state
  -> Independent Verification
  -> Commit / Compensate
```

Tool Call 的新定义：

```text
Tool Call = model-originated Effect Candidate
```

只有经过 CognitiveOS admission 并完成 Intent/Effect persistence 后，才成为 governed
Effect。纯读取操作可以使用 `NoExternalCommitment` fast path，但仍需 descriptor、scope
和 audit；文件写入、网络 mutation、凭据使用、机器人动作必须走完整 Effect protocol。

### Physical Effect extension

实体世界 Effect 还需：

- deadline；
- safety envelope；
- sensor timestamp；
- uncertainty/confidence；
- reversibility class；
- emergency stop；
- actuator receipt；
- physical post-state verifier。

LLM/Harness 不得成为 hard real-time controller。

## 9. Composition ABI

### Recommendation / ABI

语言无关 Composition ABI 至少提供：

```text
mount(plugin, scope, dependencies)
unmount(plugin, revision)
replace(plugin, new_revision)
inspect(scope)
dependency_status(plugin)
health(plugin)
dispose(plugin)
quarantine(plugin, reason)
```

每次生命周期变化返回：

```json
{
  "plugin_id": "plugin://...",
  "revision": "sha256:...",
  "scope": "scope://...",
  "dependencies": [],
  "lifecycle": "active",
  "disposal": "complete",
  "event_ref": "event://..."
}
```

### Cordis strategy

| 方案 | 结论 |
|---|---|
| 直接依赖 Cordis | 不推荐，Kernel 被 Node/TS 绑定 |
| Fork Cordis | 不推荐，持续同步和行为分叉成本高 |
| 自己完整重写 | 不推荐，重复实现且失去成熟生命周期语义 |
| Composition ABI + Cordis Adapter | 推荐 |

Cordis adapter 负责将 Context/Fiber/Service/dispose 映射为 ABI；CognitiveOS 负责 grant、
event、audit、effect、verification 和 recovery。

## 10. Harness ABI and DeepSeek Strategy

### Recommendation / Harness ABI

Harness 必须实现：

```text
start_session
resume_session
fork_session
request_context
submit_candidate
request_capability
observe_effect
checkpoint
cancel
report_failure
```

Harness 不得直接调用：

```text
mint_task
grant_capability
commit_effect
complete_task
write_authority_state
```

### DeepSeek Harness strategy

DeepSeek Harness 作为 Reference Harness Adapter：

- 使用它的 Session/Fork/Resume/Compaction；
- 将 tool call 映射为 candidate；
- 将 CognitiveOS ContextView 渲染为 Harness context；
- 将 DSH subagent lineage 映射为 Execution lineage；
- 将 DSH sandbox 视为 containment，不视为 security boundary；
- 不 Fork DSH；
- 不让 DSH 成为 CognitiveOS Runtime；
- 不让 DSH session log 取代治理 ledger。

## 11. Governed Self-Extension

### Recommendation / 生命周期

```text
Capability discovery
  -> request
  -> policy and grant check
  -> acquire immutable artifact
  -> verify digest and dependencies
  -> static validation
  -> sandbox test
  -> independent verification
  -> staged install
  -> activation under lease
  -> health observation
  -> rollback or revoke
```

必须持久化 package identity、source、digest、dependency lock、requested/granted
capabilities、sandbox evidence、verifier、activation epoch、rollback target 和 revoke event。

DSH `cordis_define/run` 只能是 `ephemeral-experiment` 类型，不能直接生成 durable Plugin。

## 12. Multi-Agent and Embodied Boundary

### Multi-Agent

```text
Agent A ─candidate─┐
Agent B ─candidate─┼─> daemon arbitration -> Intent/Effect/Verification
Agent C ─candidate─┘
```

每个 Agent 保持独立 identity、session、budget、grant、mailbox 和 execution lease。Context
共享必须重新授权；delegation 必须由 daemon 签发；child success 不能绕过 verifier；默认关闭。

Cordis Plugin Graph 可以借鉴为 dependency graph，但不能直接充当 Agent trust/delegation graph。

### Embodied

```text
Agent / LLM
    -> CognitiveOS Intent + safety policy
    -> Robot adapter
    -> ROS 2 / real-time controller
    -> actuator
```

DeepSeek Harness 适合高层计划和工具交互，不适合 hard real-time。Cordis 适合机器人
service/capability composition，但不替代实时控制器、安全 watchdog 或 emergency stop。

## 13. Layer Model

| Layer | V2 owner | 责任 | 明确不负责 |
|---:|---|---|---|
| 0 | external adapters | web、filesystem、robot、physical world | governance truth |
| 1 | infrastructure | process、SQLite、network、sandbox host | Agent intent |
| 2 | CognitiveOS runtime | event、projection、lease、recovery | model loop |
| 3 | Composition runtime | dependency、mount、replace、dispose | authorization |
| 4 | Harness ABI/adapters | session、loop、tool schema、subagent | authority write |
| 5 | Agent execution | planning、proposal、observation | grant/commit |
| 6 | Context | authorized bounded model view | raw memory authority |
| 7 | governance | identity、policy、grant、revocation | prompt generation |
| 8 | effect/verification | receipt、reconcile、verify、commit | model reasoning |
| 9 | application | user/product/robot workflow | kernel invariants |

## 14. Migration Plan

### Phase 0: preserve and measure

- 保持现有 Personal runtime 可运行；
- 建立 `session_id/execution_id/causation_id` 关联；
- 给现有事件分类并标注 source/projection；
- 不迁移 authority 数据，先生成 read-only compatibility projection。

### Phase 1: ledger bridge

- 引入 Harness Conversation Ledger adapter；
- 为每次 Tool Candidate 生成 candidate event；
- 保持 authority ledger 唯一治理 source of truth；
- 增加 session fork/resume 的 lineage mapping。

### Phase 2: object boundary refactor

- 将 Plugin、Capability Descriptor、Grant、Tool、Skill 分成不同类型；
- 以 adapter 包装旧 API，先保持数据库兼容；
- 让旧 Tool registry 逐步转为 Capability Provider registry；
- 为 deprecated cross-layer fields 增加迁移 projection，而不是立刻删除存量行。

### Phase 3: event/projection refactor

- 明确 governance Event taxonomy；
- 把 mutable state 改为 projection output；
- 为 projection version、watermark、digest 建立稳定合同；
- 为 compaction、retention、PII redaction 建立独立策略，不修改 raw authority facts。

### Phase 4: Composition ABI

- 先实现 native backend 的最小 mount/unmount/replace/dispose；
- 再实现 Cordis adapter；
- 将 dynamic Tool lifecycle 映射到 Plugin lifecycle events；
- Cordis service disappearance 不得绕过 CognitiveOS revocation。

### Phase 5: DeepSeek reference adapter

- 接入 session、fork、resume、subagent、sandbox observation；
- Tool call 只能进入 candidate path；
- DSH approval 映射为短期 human decision observation，不直接映射为 durable grant；
- 建立 cross-ledger replay tests。

### Phase 6: governed extension and agents

- 实现 artifact/digest/dependency/verification/activation/rollback；
- 实现 daemon-issued delegation 和 mailbox；
- 通过独立收益评估决定 Multi-Agent 默认是否开启；
- NO-GO 保持合法且不影响单 Agent 主线。

### Phase 7: embodied adapter

- 定义 physical capability/effect schema；
- 引入 real-time controller boundary；
- 引入 sensor uncertainty、physical receipt、safety verifier；
- 在模拟器/非安全关键设备上进行独立 qualification。

## 15. Refactor Risk and Compatibility Rules

### Recommendation / 风险控制

允许重构不等于允许无计划推翻：

1. A1-A8、SecretStore、persist-before-dispatch、independent verification、fencing 和
   unknown-outcome quarantine 不得因重构而变弱；
2. 新 Event taxonomy 先以 compatibility projection 并行验证；
3. 数据库迁移必须可回滚、可重放、可检测 digest drift；
4. 旧 Harness adapter 先保留，直到新 ABI 的 negative tests 覆盖 authority-shaped payload；
5. Cordis adapter 必须有 process crash、dependency removal、dispose failure 和 rollback
   测试；
6. Dynamic self-extension 先使用 staged/quarantine 默认，不允许 auto-enable；
7. 每个重构阶段都必须区分 local test、supported CI、Gate 和 Profile claim。

## 16. Final Decisions

1. **CognitiveOS 可以且应该部分重构。** 重构重点是边界、账本和 ABI，不是削弱治理内核。
2. **CognitiveOS V2 定位为 Durable Agent Governance Runtime / Control Plane。**
3. **Plugin 与 Capability 分离。** Plugin 是实现和生命周期；Capability 是能力描述；Grant
   是受限授权。
4. **Tool Call 重新定义为 Effect Candidate。** 只有通过 Intent/Effect admission 才成为
   governed Effect。
5. **Event-Sourced Runtime 采用双账本。** Harness conversation ledger 与 CognitiveOS
   governance ledger 关联但不合并。
6. **Cordis 通过 Composition ABI Adapter 接入。** 不直接依赖为 Kernel，不 Fork。
7. **DeepSeek Harness 作为 Reference Harness Adapter。** 不 Fork、不取代 CognitiveOS
   Runtime、不拥有 authority。
8. **Session、Execution、Task、Loop 不合并。** 应通过 containment 和 stable references
   重构关联。
9. **Context、Memory 不合并。** Context 是一次执行的 view；Memory 是 durable knowledge。
10. **实体智能必须使用 Robot Adapter + real-time controller。** Harness 不进入 hard
    real-time safety loop。
11. **最小 Kernel 是 Identity、Scope、Event、Projection、CapabilityGrant、Intent、
    EffectReceipt、VerificationLease。**
12. **核心壁垒是可恢复、可验证、跨 Harness 和跨物理适配器的治理执行账本。**

## 17. Explicit Non-Claims

本文不声明：

- 当前新增的 dsh AKP bridge preview 已完成 Linux 真机集成、Provider 往返或性能无损验证；
  这些仍需正式 Personal task、exact revision 和 Linux-002 procedure。

- CognitiveOS V2 已实现；
- DeepSeek Harness 已被集成；
- Cordis 已成为生产依赖；
- Multi-Agent 已通过 B11；
- Robot Adapter 已满足实时或安全认证；
- Profile conformance 已实现；
- 任何 Gate、release 或 benchmark 已因本研究通过。

本文件是重构决策输入，不替代正式计划、合同、Gate 预注册或实现验收。
