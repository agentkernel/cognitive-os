---
doc_id: dev.context-artifact
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-kernel/src/context.rs
    symbols: ["resolve", "STAGNATION_BOUND", "effective_control_plane"]
  - path: crates/cognitive-kernel/src/context_cache.rs
    symbols: ["GovernedContextCache", "ContextCacheKey"]
  - path: crates/cognitive-store/src/context_store.rs
  - path: crates/cognitive-store/src/artifact_store.rs
    symbols: ["put_expected", "get_authorized"]
contracts:
  - specs/schemas/context-request.schema.json
  - specs/schemas/context-view.schema.json
tests:
  - crates/cognitive-kernel/tests/context_pipeline.rs
  - crates/cognitive-store/tests/m5_context_store.rs
  - crates/cognitive-store/tests/p3_t03_artifact_store.rs
fingerprint: "sha256:0faeb3aeaee5e36cd150c344ac527eff4fc2fc19520ff85377fffdc420a26ddf"
non_claims:
  - Context 正确性证据是聚焦测试证据；收益/效率观察是他处拥有的 non-claim。
---

# Context 与 Artifact

## 九阶段解析管线

`context::resolve` 依次执行：准入 → 治理预过滤（租户/会话，在任何排序或 body 读取
**之前**）→ 检索记录 → 经 `authorize` 的逐对象重授权（action `read_body`）+ 内容
digest 去重 → 排序（唯一概率槽位；提案只能对已授权幸存者重排或收缩）→ 预算装配
（required 优先；超预算失败 `CONTEXT_BUDGET_EXCEEDED`，除非 `allow_partial` 且
`missing` 显式）→ 损失声明（不可能静默省略）→ 确定性渲染（分区顺序 control →
authoritative_state → evidence → working → untrusted_input；前缀稳定，digest 域
`cognitiveos.impl.context-render/0.1`）→ 携带钉住版本与完整 `GovernanceBinding` 的视
图产出。

提示注入隔离是结构性的：不可信内容作为数据渲染，`admit_control_mutation` 拒绝归因于
它的控制面变更。有界重解析停滞暴露 `CONTEXT_RESOLUTION_STAGNATED`
（`STAGNATION_BOUND` = 2）。

## 持久 Context 行

`ContextStore` 追加式持久化带封存内容 digest 的 ContextRequest/View；视图的强
`request_ref` 对照**已持久化**请求 digest 校验（而非调用方输入）。workspace source
带 role/trust CHECK 约束；发现是元数据先行、body 单独加载；授权/撤销事实集在**当前**
撤销 epoch 重建 `AuthzSnapshot`。真实调度路径上，daemon 在每次 body 加载前即时重载授
权/撤销，并在任何 Pi 传输前封存视图。合格 Memory/Skill 片段只在精确
scope/pin/digest 复核和当前 forget/revoke 重验之后装入；钉会被封进视图，并写入
v24 只追加消费行，供后续会话在不重述事实的情况下复用。

## 无法提供过期权威的缓存

两个缓存都以完整治理绑定为键（租户、actor-chain digest、capability-set 版本、撤销
epoch、purpose、schema digest、encoding profile、会话）；`GovernedContextCache` 另绑
定 request/contract 身份+digest、有序 source digest、渲染器版本与已校验 tool 描述符
digest，且只存 digest 级 prefix/delta 元数据。过期绑定按构造 miss；声明过期的服务请
求以 `CONTEXT_AUTH_DENIED` 拒绝并清除全部派生缓存类型。

## Artifact CAS

有界文件系统 CAS：引用严格为 `sha256:<64hex>`（绝不当路径解释），`put_expected` 在
staging 文件 + 原子 rename 发布前校验大小 + digest，`get` 读取时重哈希（篡改 ⇒
`DigestMismatch`），`get_authorized(_, false)` fail-closed（策略归调用方），只清理被
遗弃的 staging 文件。verifier 经此存储消费证据，因此证据字节不存在或哈希不符时报告
无法持久化。Personal daemon 现在会在 `data_dir()/artifacts` 打开一个进程期唯一实例，
逐 artifact 上限 8 MiB；D01 组合本身不表示生产 verifier 已运行。已对账 Effect 现在
可与其 verification request 一起，在发布 Loop `ACT -> VERIFY` 的同一权威事务中固定；
criteria 现在只从当前 TaskContract Acceptance 条件推导；登记 fixed-Effect verifier
先把不可变 post-state observation 写入该 CAS，passed report 才可进入
`VERIFY -> CONTINUE`。P2-T14 会在 acceptance 前重读每个 report Artifact URI。daemon
生成的 completion claim 与 affirmative `acceptance_decision` 是 canonical CAS bytes，
经既有 transition `StrongReference` 引用；字节缺失或 digest 不符会在 Task transition
前 fail closed。
