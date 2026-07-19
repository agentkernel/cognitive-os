# CognitiveOS Distributed Companion Specification

> 版本：v0.2 Draft

> 状态：Companion Specification；仅定义语义与符合性要求，不表示存在实现。

> 标识：`cognitiveos.distributed/0.2`

> 范围：多节点/多主体委派、租约、mailbox、分区、冲突与恢复


## 1. 规范约定

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。

只有大写英文规范词构成规范性要求；普通中文“必须/应当”仅作说明。

本规范叠加于 CognitiveOS Core；冲突时采用更严格的安全、授权、预算与证据边界。

实现状态必须声明为 `implemented`、`planned`、`experimental` 或 `unsupported`，不得把本草案当作实现证据。

## 2. 范围与假设

本 Profile 叠加于 Core，不要求多 Agent，也不承诺全局强一致或一般 exactly-once。

manifest 声明节点身份、时钟误差、分区模型、quorum/authority、一致性等级、数据驻留和故障域。

## 3. 对象与状态

`Delegation` 固定 parent/child principal、TaskContract 子集、capability/预算衰减、deadline、lease、数据可见性和回收规则。

`Lease` 固定 holder、resource/role、epoch、not_before、expiry 与 fencing token。

`Mailbox` 是持久有界 inbox/outbox，记录 partition key、sequence/high-watermark、dedupe scope、ack 和 dead-letter policy。

`ConflictSet` 保留竞争版本、authority、向量/逻辑时钟、证据和解决状态。

[REQ-DIST-OBJ-001] Delegation、Lease、Mailbox cursor 与 ConflictSet **MUST** 是版本化、可审计对象。

## 4. Delegation 协议

父方先固定子 TaskContract，再衰减 capability、预算、purpose、数据范围和期限。

子方明确 accept/reject；accept 不表示任务完成，也不转移父方未明确移交的 authority。

结果以 evidence/artifact 引用返回，并由父方或目标 authority 独立验收。

撤回只影响未来行动；未决 Effect 仍需 reconcile。

[REQ-CHARTER-HIER-001] 子主体能力、预算、数据可见性和期限 **MUST NOT** 超过父级明确授予范围。

[REQ-DIST-DEL-001] 委派 **MUST** 固定父契约版本与派生关系；子契约不得放宽验收、安全或出域条件。

## 5. Lease、epoch 与 fencing

lease 到期、撤销、失去 quorum/authority 或观测到更高 epoch 后，holder 停止新的 governed effect。

续租是新决策，不能仅靠本地时钟自延长。

执行器以 fencing token 拒绝旧 epoch，即使旧节点仍可联网。

[REQ-CAP-003] 跨节点写操作 **MUST** 使用 lease/expiry、撤销 epoch 与 fencing。

[REQ-PROFILE-DIST-001] 实现 **MUST** 证明过期 lease、旧 epoch 和失去 authority 的执行者不能提交新 governed effect。

## 6. Mailbox 与投递

outbox 与状态提交原子或可恢复关联；inbox 在业务状态持久化后 ack。

投递默认 at-least-once，以 message id 去重；业务效果以 idempotency key 去重。

顺序只在声明 mailbox partition 内成立；跨 partition 不推断顺序。

容量耗尽按 bounded block/reject/spill/dead-letter 明示处理。

[REQ-DIST-MBX-001] mailbox **MUST** 持久化 high-watermark、去重窗口和 ack 因果关系。

[REQ-DIST-MBX-002] governed message **MUST NOT** 因超载静默丢弃。

## 7. 能力与预算衰减

远程 hop 重新认证并执行本地授权；不能传递本地根 capability。

衰减维度包括 audience、purpose、action、resource、parameter、sensitivity、expiry、depth、money、calls、time、egress 和 risk。

预算 escrow 从父级可用余额预留；回收仅返回未消费额度。

[REQ-CAP-002] 派生 capability **MUST** 单调衰减。

[REQ-RES-002] 子预算 **MUST** 来自父级可用额度且不可放大。

[REQ-DIST-BUD-001] 并发子任务预算预留总和 **MUST NOT** 超过父级可委派余额。

## 8. 冲突与网络分区

authority 可达且 quorum 成立时按正常 CAS 提交。

失去 authority 的分区可继续纯读或明确允许的本地临时活动，但不得提交新的 governed external effect。

多主输入形成 ConflictSet，不静默 LWW；解决由权威 merge、业务仲裁或人工审批。

合并后以新事件 supersede 竞争版本，保留完整谱系。

[REQ-DIST-PART-001] 分区策略 **MUST** 对每类操作声明 continue、degrade、wait 或 fail-closed。

[REQ-DIST-CONF-001] 未经仲裁的竞争写 **MUST NOT** 被呈现为单一权威事实。

## 9. 失败、安全与恢复

节点崩溃后提升 epoch、恢复 mailbox 高水位、replay committed event、reconcile 未决 Effect，再接受新写。

时钟回拨不得延长 lease；安全判定采用受界时钟假设或 authority 时间。

重复委派、迟到结果和孤儿子任务按 contract/epoch 判定，不自动提交。

符合性场景：旧 leader 写入被 executor fencing；双分区仅 authority 侧可提交；重复 mailbox 不重复效果；子能力扩权被拒绝；预算并发超分被拒绝；冲突保留至显式仲裁。

[REQ-DIST-SEC-001] 实现 **MUST** 测试 split brain、时钟偏移、撤销延迟、消息重复/乱序、孤儿 Effect 与恢复竞态。

[REQ-DIST-REC-001] 恢复 **MUST** 在新写前完成 fencing、mailbox cursor 恢复和未决 Effect 对账。
