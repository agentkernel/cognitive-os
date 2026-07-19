# CognitiveOS AKP Companion Specification

> 版本：v0.2 Draft

> 状态：Companion Specification；仅定义语义与符合性要求，不表示存在实现。

> 标识：`cognitiveos.akp/0.2`

> 范围：Agent Kernel Protocol：跨进程/节点调用 Core Operations 的传输无关信封与结果语义


## 1. 规范约定

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。

只有大写英文规范词构成规范性要求；普通中文“必须/应当”仅作说明。

本规范叠加于 CognitiveOS Core；冲突时采用更严格的安全、授权、预算与证据边界。

实现状态必须声明为 `implemented`、`planned`、`experimental` 或 `unsupported`，不得把本草案当作实现证据。

## 2. 范围与边界

AKP 定义消息信封、结果、协商、幂等、取消、流式、continuation 与短期 privileged management session 的跨边界绑定，不定义本地 authority 或业务操作本身。

AKP 可映射到请求/响应、消息队列、双向流或共享内存；传输不能改变 Core 状态机。

MCP、A2A 与 DDS 是外部/下层适配边界，不是 AKP 的同义词。

## 3. Envelope

请求信封字段：`message_id`、`operation`、`protocol_version`、`schema_digest`、`sender`、`audience`、`correlation_id`、`causation_id`、`deadline`、`idempotency_key`、`authorization_ref`、`budget`、`payload|payload_ref`、`extensions`。

结果信封字段：原请求关联、`status`、`result|result_ref`、`error`、`observed_versions`、`cost`、`continuation` 与审计引用。

payload_ref 读取需再次授权；URI 或网络可达性不是权限。

[REQ-AKP-ENV-001] 每个跨边界 AKP 消息 **MUST** 带版本、schema digest、发送者、受众、关联、deadline 和不可变 payload/ref。

[REQ-AKP-ENV-002] 接收方 **MUST** 在解析业务 payload 前完成版本选择、大小限制、digest 和 critical extension 检查。

## 4. 版本与协商

主版本不兼容时返回 VERSION_UNSUPPORTED，不猜测语义。

同主版本新增字段不得改变既有状态与错误含义。

协商结果固定 peer identity、版本、编码、schema bundle 与 operation descriptors digest。

运行中 descriptor/schema 漂移触发重新协商和重新授权。

[REQ-AKP-VER-001] 连接或会话 **MUST** 固定协商结果；漂移 **MUST** 失败或建立新协商 epoch。

## 5. 结果与状态

`status` 至少为 `ok`、`accepted`、`partial`、`cancel_pending`、`error`、`outcome_unknown`。

accepted 只表示已受理，不表示执行或 Task 完成。

partial 必须携带序号、完整性声明和 continuation。

outcome_unknown 必须映射 Core Effect 对账，不得包装成普通超时。

[REQ-AKP-RES-001] receipt/accepted/remote completed **MUST NOT** 被 AKP 映射器提升为本地 verifier 通过。

## 6. 幂等与重试

消息 ID 用于投递去重；业务 idempotency key 用于效果等价去重，两者不可混用。

重试必须复用稳定 idempotency key、参数 digest 和授权绑定。

同一 key 不同参数必须拒绝，返回已登记机器码 `EFFECT_IDEMPOTENCY_CONFLICT`（见 [errors.yaml](../registry/errors.yaml) 与 Core REQ-EFF-002）。

[REQ-AKP-IDEM-001] governed operation **MUST** 声明幂等支持和 key 作用域；未知 outcome 时不得创建新 key 盲重试。

## 7. 取消

取消是有因果关系的请求，至少包含 target、reason、deadline 与 principal。

`cancel_pending` 表示传播中；`cancelled` 仅表示本地确定取消条件成立。

取消不保证远端计算、计费、数据处理或外部效果已经停止。

[REQ-AKP-CAN-001] 取消 **MUST** 保留 pending/confirmed/too-late/unknown 区分，并继续计量未知资源。

## 8. 流式与背压

每个流片段带 stream_id、sequence、kind、digest、final 标志和累计成本。

流式 partial 默认为候选数据，不能在 final verification 前作为 committed state。

消费者通过 credit/window 或等价机制施加背压；越界发送可拒绝或截断并审计。

断流后 continuation 指定最后确认序号与恢复条件。

[REQ-AKP-STR-001] 流 **MUST** 可检测重复、缺口、乱序和截断；安全关键缺口 fail closed。

[REQ-AKP-STR-002] 背压策略 **MUST** 有界且不得静默丢弃 governed 片段。

## 9. Continuation

continuation 是可移植令牌或对象引用，固定 operation、协议 epoch、状态版本、流高水位、剩余预算、expiry 和恢复前置条件。

令牌不得编码秘密或不可恢复模型隐藏态。

恢复时重新认证、授权、校验 freshness 与 fencing。

[REQ-AKP-CONT-001] continuation **MUST** 具备完整性保护、期限和单调高水位，并在恢复时重验权限。

## 10. MCP、A2A、DDS 边界

MCP tool/resource/prompt 映射为 OperationDescriptor 或数据对象；MCP feature/auth negotiation 不等于 AuthorizationCapability。

A2A Agent Card/skill 是发现信息；Task/Message/Artifact 是远端互操作对象，远端 completed 不是本地验收。

DDS 提供 topic、QoS 与实时数据分发；topic 可写性、QoS 或发现成功不授予 CognitiveOS authority。

网关必须逐对象执行 schema、大小、媒体类型、恶意内容、出域和 purpose 检查。

无法无损映射 deadline、取消、partial、错误、重试或 unknown outcome 时返回 PROTOCOL_MAPPING_INCOMPLETE。

[REQ-GW-001] 网关 **MUST** 固定 peer identity、协议版本和 schema/artifact digest。

[REQ-GW-002] 外部输入输出 **MUST** 在本地验证；远端 completed 不等于本地 Task 验收。

[REQ-GW-003] 出域前 **MUST** 逐对象执行数据政策并记录接收方、purpose 和 digest。

[REQ-GW-004] 无损映射失败 **MUST** 返回 PROTOCOL_MAPPING_INCOMPLETE。


## 10.1 Management session 与请求/结果信封

管理请求信封在 §3 字段之外 **MUST** 携带 `management_session_ref`、`actor_chain_digest`、`activity_context_ref`、proposal/approval refs（适用时）、policy/revocation versions 与 expected target versions。只传 session ID、自然语言批准或远端 bearer token 不足以授权。

建立、续期、关闭或恢复 management session 的消息 **MUST** 把 Human Principal、AuthenticationContext、ActorChain、ActivityContext、management domain、scope、risk ceiling、idle/absolute expiry、policy/revocation version 和 session authority 决定作为不可变 payload 或受完整性保护引用；普通 AKP connection/negotiation epoch、Conversation、continuation 或 transport credential 不能替代这些绑定。

[REQ-AKP-MGMT-001] 接收方 **MUST** 在解析/分派管理 payload 前验证 session state/expiry/revocation、ActorChain/ActivityContext、scope、risk、policy version、step-up 与 approval digest；结果 envelope **MUST** 区分 `accepted`、`cancel_pending`、`outcome_unknown`、`verified` 和 `committed`，且 Shell finished/remote completed 不得映射 committed。

[REQ-AKP-MGMT-002] challenge/step-up **MUST** 是绑定 proposal digest、session、human principal、method、expiry 与单次/有限使用策略的结构化交换。自然语言确认只触发 challenge；remote Shell、sidecar、continuation 恢复、trust-boundary hop 或协商 epoch 变化 **MUST** 重新认证并本地重验，不能转发普通 Conversation credential。

[REQ-AKP-MGMT-003] Session expiry/revocation/close **MUST** 使后续 management request fail closed；reconnect、renew、remote/sidecar 或 continuation resume **MUST NOT** 复用旧 session authority，除非 session authority 签发新的完整性受保护版本并由接收方本地重验。已有 `outcome_unknown` Effect 必须保留原 operation、proposal、approval、idempotency 与 session audit refs 供对账，但旧 session 不因此继续允许新 dispatch。

管理取消遵循 §7：取消 Shell 或 request stream 不证明 Effect 未执行。断连/timeout/无法无损映射的结果必须为 `outcome_unknown` 并关联 `ReconcileEffect`；不得返回普通 `ok`。Management API 的 deterministic CLI/Console 与 Intelligent Shell 使用相同 envelope/result 语义。

## 11. 失败、安全与符合性

错误遵循 Core 错误 envelope；retryable 不表示立即重试安全。

未知版本、digest、主体、authority、critical 字段或跨租户引用 fail closed。

网关不得把外部 token、feature、skill、topic ACL 自动转换为本地 capability。

符合性场景：重复请求同 key 不重复效果；同 key 异参被拒绝；断流可从确认序号恢复；取消过晚返回 too-late；schema 漂移终止协商 epoch；MCP/A2A/DDS feature 不能越权；过期/撤销/关闭的 management session 拒绝新请求；Shell reconnect 或 continuation 不恢复旧特权；session 终止前的 unknown management Effect 仍可用原 key 对账。

[REQ-AKP-SEC-001] AKP 实现 **MUST** 对消息重放、confused deputy、schema rug pull、流截断和 continuation 伪造执行负例测试。

[REQ-AKP-CONF-001] 声明 AKP implemented **MUST** 固定协议/编码版本、operation set digest、映射 profile 和测试证据。

## 12. Agent 生态对象与操作映射
AKP 可传递或引用 package/installation/compatibility、memory candidate/admission/object、resource manifest/gap/request admission/delta、operation summary/catalog/match/binding、semantic manifest/allocation decision。未知 critical 字段继续 fail closed。

操作族包括：`agent.install|upgrade|remove|inspect`、`memory.propose|admit|invalidate|tombstone`、`resource.discover`、`context.admit|expand|resolve_delta`、`catalog.discover|describe|bind`、`semantic.mediate`、`cognitive.allocate`。安装与持久 memory mutation 采用 command/result + Intent/Effect；discover/mediate 是 request/response candidate；delta/stream partial 不能提交事实。

[REQ-AKP-ECO-001] 映射 **MUST** 保留 candidate/admitted/authorized/executed/verified/committed 的区分，且 descriptor、summary、remote installation 或 Agent completed **MUST NOT** 转换为本地 capability/acceptance。

[REQ-AKP-ECO-002] `resolve_delta` **MUST** 携带 base view、ActivityContext、父预算、attempt high-watermark 与 expansion kind；跨传输恢复不得扩大原绑定。

[REQ-AKP-ECO-003] semantic unavailable、no authorized candidate、not discoverable、catalog stale 与 outcome unknown **MUST** 保持不同错误语义；网关不得用空成功替代。


## 13. Shell、意图与观察协议映射
AKP 可传递 `UserIntentRecord`、`IntentInterpretation`、`IntentAdmissionDecision`、`ShellActionProposal`、`ShellCommandPreview`、`TargetResolution`、`WatchSubscription` 与 `ShellStatusView`。操作族为 `intent.record|interpret|admit|supersede`、`shell.preview|submit|attach|control` 和 `watch.open|ack|resume|close`。

[REQ-AKP-SHELL-001] `shell.submit` **MUST** 固定 channel、proposal/preview digest、resolved target strong refs、expected versions、contract epoch、idempotency key 与 authorization/approval refs；接收方不得重新解释原始自然语言来改变已确认动作。

[REQ-AKP-SHELL-002] `watch.resume` **MUST** 携带 subscription、snapshot version、last acknowledged cursor 与 dedupe window；cursor 过期或不可连续恢复时返回 `WATCH_CURSOR_STALE` 并要求新的授权 snapshot，不得静默丢失 governed event。

[REQ-AKP-SHELL-003] Shell control 结果 **MUST** 区分 accepted、cancel_pending、cancelled、too_late、runtime_terminated、writer_fenced、outcome_unknown、quarantined、verified 与 committed；transport close 或 client exit 不映射为 cancel/commit。

[REQ-AKP-INTENT-001] 意图修正 **MUST** 携带 parent/supersedes digest 与新 contract epoch；旧 epoch 的 proposal/dispatch 由接收端 fencing 拒绝，未决 Effect 保留原 key 对账。
