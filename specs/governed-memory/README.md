# CognitiveOS Governed Memory Companion Specification

> 版本：v0.1 Draft  
> 标识：`cognitiveos.governed-memory/0.1`

## 1. 范围与边界
本规范统一记忆对象、准入和生命周期，不规定数据库、向量引擎或 consolidation 算法。Memory 是历史认知记录，不自动等于当前 World State、发布 Knowledge 或 Evidence authority。

## 2. Memory kinds 与对象
标准 kinds：`working`、`episode`、`conversation`、`task`、`subject`、`procedural`、`semantic`、`world_observation`、`system`。对象固定 governance scope、owner/authority、principal/conversation/task/episode refs、purpose/sensitivity/compartment、provenance/evidence、valid/observed/ingest time、freshness、confidence method/assessor/claim、conflict refs、retention/expiry、representations、admission status 与 supersedes/invalidates。

[REQ-MEM-OBJECT-001] 持久或跨边界 MemoryObject **MUST** 固定上述治理、时间、来源、retention 与 lineage；无来源 confidence **MUST NOT** 作为高风险 gate。

## 3. 写入与准入
规范链为 `Observation → MemoryCandidate → Validate → Policy → Conflict → MemoryAdmissionDecision → MemoryObject`。模型摘要、Agent 自述和单一 Episode 只能成为 candidate。

[REQ-MEM-ADMIT-001] 非 working 的持久 memory **MUST** 由目标 scope authority 准入；Agent **MUST NOT** 直接发布长期事实。

Authority 按 kind 分离：Activity owner 管 working；Conversation authority 管 conversation；subject/data authority 管主体长期记忆；knowledge authority 管发布知识；world authority 管当前状态；audit authority 管系统审计。

## 4. 晋升、冲突与修改
`working→episode→conversation→subject` 或 `candidate→knowledge` 每次跨 scope 都创建新对象/版本、AdmissionDecision、retention/purpose/sensitivity 与 lineage，禁止原地改 URI/标签。冲突以 `contradicts|supersedes|invalidates` 共存，除非 authority 明确裁决。

[REQ-MEM-PROMOTE-001] 跨 scope/kind 晋升 **MUST** 产生新对象和新 admission，且 **MUST NOT** 扩大源 purpose 或删除冲突。

[REQ-MEM-MUTATE-001] 创建者、owner 与 authority 权限必须区分；一个 Agent **MUST NOT** 修改另一主体创建的 MemoryObject，除非目标 authority 明确授权并保留旧版本。

## 5. 失效、遗忘与删除
撤销、过期、source deletion、policy/license 变化沿 lineage 传播到摘要、embedding、index、cache 和 ContextView。遗忘可删正文但保留最小 tombstone；legal hold 不产生 read 权限。

[REQ-MEM-DELETE-001] 删除/遗忘 **MUST** 区分物理删除、读取撤销、legal hold 与派生 closure，并给出可审计结果。

## 6. 读取和符合性
召回重新检查 ActorChain、Conversation/ResourceScope、purpose、freshness、policy/revocation 与当前 authority。高风险 claim 应 authority refresh，不得只依赖 memory recall。测试覆盖直接长期写、跨 scope 原地晋升、冲突覆盖、跨 Conversation 污染、撤销缓存绕过和派生删除。

机器 schema：[memory-object.schema.json](../schemas/memory-object.schema.json)、[memory-candidate.schema.json](../schemas/memory-candidate.schema.json)、[memory-admission-decision.schema.json](../schemas/memory-admission-decision.schema.json)。


## Shell 与用户旅程映射
Shell 可 inspect/propose/invalidate memory，但自然语言“记住”只产生 MemoryCandidate；history/undo 不能绕过 retention、authority 或派生 closure。

[REQ-MEM-SHELL-001] Shell memory 操作 **MUST** 使用目标 scope admission/authorization 并显示来源、有效期、冲突和持久化范围。
