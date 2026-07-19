# CognitiveOS Core Companion Specification

> 版本：v0.2 Draft

> 状态：Companion Specification；仅定义语义与符合性要求，不表示存在实现。

> 标识：`cognitiveos.core/0.2`

> 范围：传输无关的对象、状态、上下文、事件、授权、副作用、循环、资源、错误、审计与恢复语义


## 0. Core 规范索引与边界

| 文档 | 版本/状态 | 规范范围 | 资产边界 |
|---|---|---|---|
| 本文 `cognitiveos.core/0.2` | v0.2 Draft / Companion Specification | 基础对象、状态、Context、Event、Capability、Effect、Loop、恢复与审计 | 已有 Core 文本要求；实现状态仍须单独声明 |
| [RFC-0001：治理上下文、访问与知识编译](../../RFC-0001-cognitiveos-governance-context-access.md) | v0.2 Draft / Normative Companion RFC | Tenant、Membership、Principal、ActorChain、Conversation、ResourceScope、ShareGrant、短期 privileged management session、混合授权、认知准入、缓存/检索隔离与执行上下文 | 治理对象族 machine schema 已由 [governed-object-contract](../../docs/standards/governed-object-contract.md) 登记 v0.1 Draft；知识对象仍为伪 schema；performance 与 management session/proposal/approval schema、registry 条目及 declarative vector 的实际登记各自决定机器边界，均不表示实现 |

RFC-0001 叠加于基础 Core：基础 Core 给出通用对象、状态、授权、Context Resolution、Effect 与恢复语义；RFC-0001 收紧企业多租户、并发 Conversation 和认知数据派生边界。实现声明支持 RFC-0001 时，必须显式固定其版本并提供独立测试证据，不能仅凭支持本 README 推导。

边界规则：

- Tenant 是数据与策略隔离域，不是权限等级；同 tenant 和管理员身份不自动获得正文读取权。
- Principal 是授权主体；AgentExecution 是绑定不可变 TenantContext、ActorChain、可选 Conversation 或显式 non-conversational scope 与治理版本的逻辑执行身份；Conversation 是交互资源作用域。切换 tenant 必须产生新的 execution identity。
- 所有受治理对象显式区分 tenant 与 platform scope；缺失/null tenant 不表示公共，platform scope 只能由独立 platform governance authority 管理。
- RFC-0001 的治理对象族已有登记机器 schema（[governed-object-contract](../../docs/standards/governed-object-contract.md)）；其正文代码块与知识对象仍是伪 schema；实际提交的 schema、registry 与 vector 可被固定 digest 后引用，但不证明实现。
- Intelligent Shell 不是 authority 或永久 root Agent；它只能通过短期、可撤销、绑定 ActorChain/ActivityContext/scope/risk/policy 的 PrivilegedManagementSession 调用 Management API，每个写操作仍独立门禁和审计。
- 白皮书仍为 informative；RFC-0001 是 normative companion draft；两者都不等于实现能力。若基础 Core 与 RFC-0001 对同一安全边界表述不同，采用不扩大读取、写入、委派、用途、保留或传播范围的解释。

## 1. 规范约定

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。

只有大写英文规范词构成规范性要求；普通中文“必须/应当”仅作说明。

本规范叠加于 CognitiveOS Core；冲突时采用更严格的安全、授权、预算与证据边界。

实现状态必须声明为 `implemented`、`planned`、`experimental` 或 `unsupported`，不得把本草案当作实现证据。


### 1.1 标识兼容
当前标识为 `cognitiveos.*`。旧 `agentos.*` 与 `agentos_conformance` 仅可由显式 legacy adapter 或固定旧 schema 接收；适配器 MUST 记录源/目标版本和映射审计，MUST NOT 在同一对象、消息或协商 epoch 静默混用。不可无损映射时返回 `PROTOCOL_MAPPING_INCOMPLETE`。

## 2. 范围与设计不变量

Core 管理智能活动依赖和产生的受治理对象，不定义智能算法、模型、提示模板、数据库或部署拓扑。

`AgentExecution` 是逻辑持久身份，不等同于进程、线程、容器、节点或模型会话。

Core 闭环为 `observe -> resolve -> decide -> authorize -> act -> reconcile -> verify -> commit/continue/stop`。

权威不变量：观察、建议、授权、执行和验收的权力必须可区分。

因果不变量：每次受治理改变可回溯到固定前态、主体、目的、授权、证据与后态。

证据不变量：模型文本、HTTP 成功、工具 receipt 或远端 completed 均不单独证明任务完成。

有界不变量：时间、费用、Token、工具调用、数据出域、风险和重试必须有声明边界。

恢复不变量：恢复依赖持久证据、固定版本和 reconciliation，不依赖模型隐藏态。

语义隔离不变量：语义组件可提议与排序，确定性 gate 决定授权、CAS、预算、状态机和提交。

[REQ-SCOPE-001] 符合 Core 的实现 **MUST NOT** 以特定 LLM、供应商、传输、数据库、语言或硬件作为协议正确性的前提。

[REQ-SCOPE-002] 实现 **MUST** 区分已实现、规划、实验和不支持状态；规划资产 **MUST NOT** 被报告为已实现。

[REQ-CHARTER-DET-001] 授权、CAS、预算硬边界、schema 验证、幂等判定和状态机合法性 **MUST** 由确定性机制执行。

[REQ-CHARTER-PROTO-001] 一致性 **MUST** 由对象、状态机、错误和审计行为证明，**MUST NOT** 仅由参考架构名称证明。

## 3. Core 对象模型

### 3.1 闭合对象集合

Core 治理上下文采用 `GovernanceDomainContext = TenantContext | PlatformContext`，所有对象显式携带 `scope_domain`。`AgentExecutionBinding`、`ExecutionContext`、`ActivityContext`、`ConversationBinding` 与 `Participant` 的收窄关系由 RFC-0001 定义。

Core 对象包括 `AgentExecution`、`Episode`、`Task`、`TaskContract`、`StateSnapshot`、`ContextRequest`、`ContextView`、`ContextItem`、`ContextReference`、`MemoryObject`、`KnowledgeObject`、`OperationDescriptor`、`AuthorizationCapability`、`Event`、`Intent`、`Effect`、`ReasoningPolicy`、`AttentionBudget` 与 `LoopCheckpoint`。

对象关系必须闭合：ContextItem 指向对象或外部 artifact；Intent 指向 Task、快照、授权与操作；Effect 指向 Intent；提交 Event 指向 Effect 与结果状态版本。

局部临时变量、纯函数中间值与不可观察栈值不要求对象化。

[REQ-CHARTER-OBJ-001] 跨组件治理边界的状态、上下文、能力、事件、意图、效果与策略 **MUST** 携带统一元数据或可验证引用。

### 3.2 统一元数据

每个持久化或跨边界对象的最小元数据为：

- `id`：稳定逻辑身份，不是权限；

- `type`：对象类型；

- `schema_version`：语义 schema 标识；

- `object_version`：同一 id 下单调版本；

- `owner`：生命周期责任主体；

- `authority`：最终写入或仲裁主体/协议；

- `provenance.created_by` 与 `source_refs`：来源；

- `sensitivity`：信息分类；

- `valid_time.from/until`：世界适用时间；

- `content_digest`：规范内容摘要；

- `lineage.parents/transform`：派生谱系。

Owner 不自动具有写 authority；authority 也不自动具有读取所有正文的权限。

valid_time 不等于写入时间，ingest time 不等于事件发生时间。

content_digest 覆盖规范化内容，不覆盖可变存储位置。

[REQ-OBJ-001] 每个持久化或跨边界 Core 对象 **MUST** 包含上述最小元数据。

### 3.3 身份、引用和规范编码

强引用固定 `{id, object_version, content_digest}`。

弱引用可省略版本，但必须携带 freshness 与重解析规则。

语义引用包含 query、解析策略、解析时刻及结果固定方式。

`cas://` 内容地址随内容变化；`query://` 结果不是稳定身份。

[REQ-OBJ-002] 语义查询结果 **MUST NOT** 被当作固定对象，除非记录解析时刻、算法版本、命中版本和 digest。

[REQ-OBJ-003] id、URI 或 content-address **MUST NOT** 被解释为授权。

[REQ-PROTO-001] 跨边界消息 **MUST** 声明协议或 schema 版本，接收方 **MUST NOT** 猜测未知主版本。

[REQ-PROTO-002] 规范化编码 **MUST** 定义 map、数字、时间、Unicode、缺省值和未知字段行为。

[REQ-PROTO-003] 扩展 **MUST** 使用命名空间并声明 critical；未知 critical 扩展必须拒绝。

[REQ-PROTO-004] 不兼容语义变化 **MUST** 使用新主版本、显式协商或可审计迁移。

## 4. Authority 与 OperationDescriptor

Authority 是对对象或状态域进行最终写入、仲裁或验收的主体或协议。

观察来源可以提供证据，但除非被状态域协议指定，否则不是 authority。

自然语言命令可触发授权流程，但文本自身既不是 authority，也不是 capability。

同一状态域只能有一个当前写 authority，或一个明确的共识/仲裁协议。

`OperationDescriptor` 描述操作的语义能力：操作名、输入/输出 schema、effect class、幂等支持、取消语义、查询/对账能力、版本、endpoint 与限制。

OperationDescriptor 回答“端点能做什么以及如何调用”，不回答“本次主体是否获准调用”。

MCP feature、A2A skill/feature、DDS topic 能力以及工具描述均属于功能/语义能力声明。

`AuthorizationCapability` 是本地 authority 签发的、不可伪造且可衰减的本次授权。

AuthorizationCapability 回答“谁、为哪个 purpose、对什么资源、在何参数和期限内可执行什么”。

OperationDescriptor 与 AuthorizationCapability **MUST NOT** 共用字段或类型名造成权限混淆。

Descriptor 被发现、读取或协商成功不授予调用权；capability 也不证明 endpoint 实现了声明功能。

[REQ-AUTH-001] 每个受治理写操作 **MUST** 由目标状态域 authority 或其明确委托作最终提交判定。

[REQ-OP-001] OperationDescriptor **MUST** 声明 schema digest、版本、effect class、取消与 unknown-outcome 支持。

[REQ-OP-002] OperationDescriptor、feature 或 skill **MUST NOT** 被当作 AuthorizationCapability。

## 5. State Protocol

### 5.1 状态域

World State 是外部世界的受版本治理投影（机器 schema：[world-state.schema.json](../schemas/world-state.schema.json)）；传感器和工具默认仅为输入源。

Task State 保存目标、依赖、进度、验收和未决 Effect。

Agent State 保存 AgentExecution 的预算、capability 引用、待处理 Intent 和 continuation。

Session State 保存交互边界、参与者、偏好和临时关联。

Transcript、LLM 输出、检索命中和远端声明默认不是权威状态。

[REQ-STATE-001] 每个状态域 **MUST** 指定唯一当前写 authority 或明确的共识/仲裁协议。

### 5.2 StateSnapshot

StateSnapshot 是不可变读视图，至少包含 `domain`、`base_version`、`high_watermark`、`projection_version`、`object_refs` 和 `digest`。

Snapshot 的 freshness 与 valid_time 必须显式；读取缓存必须暴露 staleness。

快照加有序已提交事件可以恢复投影，但事件日志不是状态读取 API 的替代。

[REQ-STATE-002] 固定 projection_version 对同一有序事件集合 **MUST** 生成相同状态 digest。

### 5.3 CAS 与冲突

状态写入请求包含 `expected_version`、mutation/patch digest、principal、authorization decision 和 causation。

成功提交原子地产生新 object_version 与关联 Event，或提供等价不可分割保证。

版本不符返回 `STATE_CONFLICT`，不得静默 last-write-wins。

语义冲突应保留 conflict set，由 authority、确定性 merge 或显式审批解决。

[REQ-STATE-003] 状态写入 **MUST** 使用 CAS、事务或等价冲突检测并声明 expected_version。

[REQ-CHARTER-STATE-001] 可能修改状态或产生副作用的 activity **MUST** 固定读取快照并在授权或提交时重验前置条件。

### 5.4 执行身份与 continuation

AgentExecution 身份跨节点、会话和模型切换保持稳定。

每次领导权/租约变更提升 fencing epoch；旧 epoch 不得提交新副作用。

continuation 固定状态版本、未决 Intent、事件高水位、策略版本、预算和恢复前置条件。

[REQ-STATE-004] AgentExecution 身份 **MUST** 独立于宿主进程、模型会话和节点。

[REQ-STATE-005] continuation **MUST** 记录可移植恢复状态，**MUST NOT** 假定模型隐藏态可恢复。

## 6. Context Resolution Protocol

### 6.1 操作

最小操作为 `resolve(ContextRequest) -> ContextView | ContextError`。

ContextView 是针对 activity 的短期非权威工作面，不是长期事实库。

解析实现可使用索引、规则、模型或人工服务；最终授权、预算、版本固定和响应验证必须满足 Core。

[REQ-CHARTER-CTX-001] 受治理 semantic activity **MUST** 由 ContextRequest 解析 ContextView，并报告固定版本、拒绝项与损失。

### 6.2 ContextRequest

请求包含 `purpose`、`perspective`、`budget`、`priority`、`required`、`forbidden`、`freshness`、`sensitivity`、`target_profile` 和 partial policy。

perspective 固定 principal、Task、Episode 与可选 activity。

required 表示缺失即失败；priority 不得覆盖 forbidden、authorization 或硬预算。

target_profile 仅描述渲染约束，不授予 endpoint 访问权限。

[REQ-CTX-001] ContextRequest **MUST** 包含上述目的、视角、预算、选择、安全与目标约束。

### 6.3 九阶段解析

1. `discover`：按引用、索引或语义查询发现候选。

2. `filter`：应用类型、valid_time、freshness、forbidden 与去重。

3. `authorize`：逐对象、逐目的、逐主体检查读取与出域。

4. `rank`：确定性优先级和强制保留先于可选语义排序。

5. `budget`：分配 Token、字节、时间、费用与 attention slots。

6. `transform`：切片、规范化、脱敏或有损压缩。

7. `verify`：验证 required、digest、schema、敏感度继承与损失包络。

8. `render`：按 target profile 形成结构化输入。

9. `audit`：封装选择、拒绝、成本、版本、谱系与错误。

[REQ-CTX-007] 实现 **MUST** 完成等价九阶段并为每阶段产生 reason code。

[REQ-CTX-002] authorize **MUST** 在正文呈现给 ranker、transformer 或外部 target 前执行。

[REQ-CTX-003] 有损转换 **MUST** 保留源引用、转换版本、敏感度派生和 loss declaration。

[REQ-CTX-004] required 项缺失、过期、无权、冲突或超硬预算时 **MUST** 失败，除非明确允许 partial。

### 6.4 ContextView

ContextView 包含 metadata、loaded、rejected、loss_declaration、pinned_versions、cost、lineage、complete 和 activity binding。

loaded 项固定对象版本、digest、representation 与成本。

rejected 项不得泄漏未授权正文，只提供安全 reason code。

引用在实际使用时重新检查有效期、revocation epoch、purpose 和权限。

[REQ-CTX-005] 成功 ContextView **MUST** 报告 loaded/rejected、损失、固定版本、成本与谱系。

[REQ-CTX-006] Context 错误 **MUST NOT** 静默降级为成功；partial 必须 `complete:false` 并列缺失项。

### 6.5 信任分层

control 层包含 TaskContract、capability/预算、stop 与验收规则，只能由本地 authority 写入。

authoritative_state 层包含固定 StateSnapshot 与未决 Effect。

evidence 层包含带来源、时间、digest 和冲突关系的知识与 artifact。

working 层包含可回查的 checkpoint、假设和候选计划。

untrusted_input 层包含用户、网页、工具和远端 Agent 内容，必须按数据隔离。

[REQ-CTX-008] 渲染器 **MUST** 保留来源、信任级别和数据/控制角色；不可信输入 **MUST NOT** 改写控制层。

[REQ-CTX-009] TaskContract 的验收依据、禁止假设、未决 Effect 与人审 gate **MUST** 作为受保护 required 项。

[REQ-CTX-010] 冲突、未知、过期或未验证候选 **MUST NOT** 被压缩为无条件事实。

[REQ-CTX-011] 非确定性 discover/rank/transform **MUST** 记录候选 digest、策略版本、输入版本和选择理由。

## 7. Event Protocol

Event 是不可变的已发生事实通知，不是命令，也不取代权威状态。

Envelope 包含 identity、type、source、subject、correlation、causation、event_time、ingest_time、schema_digest、deadline、delivery_class、ack、backpressure 和不可变 payload/ref。

Core 不指定传输；内存队列、日志、数据库 outbox、HTTP 或消息系统都可承载。

默认可靠变更事件为 at-least-once；顺序仅在声明 partition/subject 内成立。

[REQ-EVT-001] 跨边界 Event **MUST** 包含完整 envelope 和不可变 payload/ref。

[REQ-EVT-002] at-least-once 消费者 **MUST** 安全处理重复；ack 在状态持久化后或与之原子提交。

[REQ-EVT-003] 实现 **MUST NOT** 对一般跨系统副作用承诺 exactly-once。

[REQ-EVT-004] Event **MUST** 不可变；纠正通过带 causation/supersedes 的新事件表达。

[REQ-EVT-005] 超载 **MUST** 按声明策略阻塞、拒绝、溢写或有标记丢弃；governed event 不得静默丢失。

## 8. AuthorizationCapability

最小字段包括 issuer、subject、audience、purpose、resource、actions、parameter_binding、lease、delegation、revocation、fencing_token、digest 与签名/等价完整性证明。

授权决策固定 capability digest、目标状态版本、操作 schema digest 和参数 digest。

Capability 只在其 audience 与 purpose 上有效；不能用作通用 bearer token。

派生 capability 的集合边界、期限、委派深度、敏感度和预算只能收窄。

撤销可使用 revocation epoch、在线列表或等价机制；实现须声明传播与 fail-closed 行为。

每个信任边界 hop 都重新认证调用者并执行本地授权。

[REQ-CAP-001] 授权 **MUST** 最小化并绑定 subject、audience、purpose、动作、资源和参数范围。

[REQ-CAP-002] 派生或委派 **MUST** 单调衰减，不得扩大任何权限、期限、数据等级或预算。

[REQ-CAP-003] capability **MUST** 有 lease/expiry 与撤销机制；跨节点写操作必须 fencing。

[REQ-CAP-004] 每一信任边界 hop **MUST** 本地重新鉴权，不能信任上游 feature 或认证结果替代许可。

[REQ-CAP-005] 参数、目标版本、操作 schema digest 或 purpose 变化 **MUST** 使授权失效或重新判定。

## 9. Intent 与 Effect

### 9.1 效果类别

`pure` 无可观察状态变化；无需 Intent，但受预算和数据边界约束。

`local_ephemeral` 只影响当前故障域且可丢弃，可简化记录。

`governed_external` 跨主体、持久化或外部系统，必须完整治理。

`emergency_safety` 是预授权减险动作，适用具身安全例外。

### 9.2 Intent

Intent 固定 action、参数 digest、OperationDescriptor、目标、expected state version、capability refs、effect class、deadline、idempotency key、后置条件与补偿策略。

Intent 必须先持久化，再执行 governed_external 操作。

### 9.3 Effect 状态机

规范状态为 `PROPOSED`、`AUTHORIZED`/`DENIED`、`EXECUTING`、`EXECUTED`/`NOT_EXECUTED`/`OUTCOME_UNKNOWN`、`RECONCILED`、`VERIFIED`/`VERIFY_FAILED`、`COMMITTED`/`ABORTED`/`QUARANTINED`，以及可选 `COMPENSATING`（机器 schema：[intent.schema.json](../schemas/intent.schema.json)、[effect.schema.json](../schemas/effect.schema.json)；迁移表：[effect.transitions.json](../transitions/effect.transitions.json)）。

执行 timeout 或断连不表示未执行；receipt 是执行证据，不是完成证明。

dispatch 是触发 `AUTHORIZED -> EXECUTING` 的动作/Event，不是 Effect 状态。提交依据声明的 postcondition verifier 对固定后态的证据。

[REQ-EFF-001] governed_external **MUST** 遵循 Intent→Authorize→Execute→Reconcile/Verify→Commit/Abort。

[REQ-EFF-002] 执行调用 **MUST** 携带稳定 idempotency key、参数 digest、授权摘要和适用的 fencing token。

[REQ-EFF-003] receipt **MUST** 仅作为执行证据；提交必须依据后置条件 verifier。

### 9.4 Unknown outcome 与补偿

OUTCOME_UNKNOWN 的合法出口为：查询确认已执行、查询确认未执行、使用同一幂等键安全重试、独立授权补偿，或隔离等待显式对账。

未知结果不得自动成功，也不得换新幂等键盲重试。

补偿是新 governed effect；其存在不表示原操作可逆。

[REQ-EFF-004] OUTCOME_UNKNOWN **MUST NOT** 被盲重试或报告成功；必须 reconcile 或进入 QUARANTINED。

[REQ-EFF-005] 补偿 **MUST** 独立授权、记录和验证。

[REQ-EFF-006] WAL 恢复 **MUST** 覆盖 Intent 后/调用前、执行后/receipt 前、验证后/commit 前三个崩溃点。

[REQ-EFF-007] emergency 路径 **MUST** fail safe，且普通认知 activity 不得覆盖最终安全仲裁。



### 9.5 Effect 状态闭包
`RECONCILED` 是对执行事实完成查询/观察归并后的持久状态，不等于成功；其结果字段必须明确 `executed|not_executed|still_unknown`。合法主链为 `EXECUTING→EXECUTED→RECONCILED(executed)→VERIFIED|VERIFY_FAILED→COMMITTED|ABORTED`。`OUTCOME_UNKNOWN` 只能经对账进入 `RECONCILED`，再按对账结果分流：`executed` 进入验证；`not_executed` 以终态 `NOT_EXECUTED` 关闭，不得验证为成功；`still_unknown` 只能进入独立授权的 `COMPENSATING` 或 `QUARANTINED`，不得提交（完整迁移边见 [effect.transitions.json](../transitions/effect.transitions.json)）。

[REQ-EFF-STATE-001] 实现 **MUST** 拒绝跳过 RECONCILED/Verification 的 commit、`OUTCOME_UNKNOWN→COMMITTED`、`VERIFY_FAILED→COMMITTED` 与换 key 盲重试等非法迁移，并返回当前状态和安全出口。

## 10. TaskContract 与 LoopCheckpoint

TaskContract 是 loop 的版本化外部契约，不是任务 prompt。

它包含目标与范围、验收条件、verifier、可变更状态域/工具、禁止约束、风险/人审 gate、预算、最大迭代与重试、等待条件及 completion/escalation/abort/quarantine 出口。

每次 loop 在选择行动前检查 contract、快照/ContextView 适用性、capability、预算、取消和人审 gate。

进展必须是相对 contract 的可验证状态差异、已缩小不确定性或已满足前置条件。

模型自评、重复计划、相同 receipt 或增长的 transcript 不构成进展。

LoopCheckpoint 保存 task_contract_ref、iteration、phase、observation_refs、context_view_ref、proposal_ref、effect_refs、progress evidence、failure attribution、remaining_budget、next_gate 和 continuation_conditions。

Checkpoint 记录行动级事实和证据引用，不要求保存原始 prompt 或思维链。

[REQ-RUN-004] 跨 activity/会话/恢复边界的 Task **MUST** 有版本化 TaskContract，并在副作用前声明可判定出口。

[REQ-RUN-005] 每次 loop 迭代 **MUST** 检查硬前置条件；不满足则等待、升级、终止或隔离。

[REQ-RUN-006] 跨边界继续的 loop **MUST** 持久化 LoopCheckpoint，且不得依赖不可移植隐藏态。

[REQ-RUN-007] progress **MUST** 由可验证差异、降低的不确定性或满足的前置条件定义。

[REQ-RUN-008] 相同失败或无进展 action 的重试 **MUST** 受 fingerprint、次数、时间、费用和归因边界约束。

[REQ-RUN-009] Task 仅在 verifier 对固定后态给出通过证据后才可 `COMPLETED`。

[REQ-RUN-001] workflow 决策 **MUST** 固定代码/策略版本；重放不得悄然采用不兼容新逻辑。

[REQ-RUN-002] semantic activity **MUST** 记录 provider/model revision、请求/响应 digest、ContextView、预算、出域、采样和 verifier。

[REQ-RUN-003] 调度器 **MUST** 在准入前预留或上界估计不可抢占资源，并计量取消后的未知消耗。

### 10.1 Task 与 AgentExecution 控制闭包
Task `BLOCKED` 可在依赖事件满足后返回 `ACTIVE`；`CANDIDATE_COMPLETE` 在验证失败/过期后返回 `ACTIVE|BLOCKED`。AgentExecution 的 `SUSPENDED` 只能经重新准入返回 `RUNNABLE`；`QUARANTINED` 只能经 reconcile/reauthorize 返回 `RECOVERING`，不能直接 runnable；`TERMINATED` 为终态。

[REQ-RUN-STATE-001] execution/task 控制 API **MUST** 以 expected version 迁移并保存 reason/causation；cancel request 与 Task `CANCELLED`、Runtime terminated 和 Effect closure 必须分别判定。

## 11. 预算与认知地址

AttentionBudget 可包含 input/output tokens、context bytes、semantic/tool calls、wall/compute time、money、egress bytes、energy、risk 与 attention slots。

每个维度声明支持、精度、hard/soft 和计量点。

hard 预算在准入和实际消耗点由确定性机制执行。

预算增加不能放宽权限、风险或验证标准。

父任务委派给子任务的预算从父级可用额度扣减，已消费额度不可通过回收抹除。

[REQ-RES-001] hard 预算 **MUST** 在准入和消耗点执行；风险或权限边界 **MUST NOT** 由更多资源放宽。

[REQ-RES-002] 委派预算 **MUST** 来自父级可用额度且不可放大。

[REQ-RES-003] 资源耗尽 **MUST** 返回机器错误或预声明降级，不得隐式提高出域、降低验证或扩大权限。

类型化地址可使用 `state://`、`memory://`、`knowledge://`、`operation://`、`event://`、`cap://`、`cas://` 与 `query://`。

URI 中的 tenant、path、version 与 digest 必须规范化；重定向和外部 fetch 重新授权。

[REQ-ADDR-001] 解析器 **MUST** 拒绝路径穿越、歧义编码和跨租户混淆，并校验版本/digest。

[REQ-ADDR-002] URI **MUST NOT** 携带秘密或被当作 bearer authorization。

## 12. 最小传输无关 Core Operations

下列操作定义可观察语义，不规定 RPC 名称、HTTP 路径、队列或进程 ABI。

`GetObject(ref, read_context) -> object | ObjectError`：固定引用读取与授权。

`ResolveContext(request) -> view | ContextError`：上下文解析。

`ReadState(domain, selector, as_of) -> snapshot | StateError`：读取不可变快照。

`CompareAndSet(expected_version, mutation, authorization) -> snapshot_ref,event_ref | StateError`：冲突检测写入。

`AppendEvent(event, expected_high_watermark?) -> position | EventError`：持久不可变事件。

`SubscribeEvents(cursor, filter, budget) -> event stream | EventError`：有界消费、ack 与背压。

`Authorize(intent_ref, capability_ref, state_ref) -> decision | AuthError`：本地确定性授权。

`ExecuteIntent(intent_ref, decision_ref) -> effect_ref | EffectError`：受管分派。

`ReconcileEffect(effect_ref) -> effect_ref | EffectError`：查询、对账或隔离。

`VerifyEffect(effect_ref, verifier_ref, state_ref) -> report | VerifyError`：后置条件验证。

`CommitEffect(effect_ref, expected_version) -> state_ref,event_ref | StateError`：验证后提交。

`Checkpoint(execution_ref, loop_state) -> checkpoint_ref | RecoveryError`：持久恢复点。

`Resume(checkpoint_ref, fencing_epoch) -> execution_ref | RecoveryError`：恢复并 fence 旧执行者。

`Cancel(target_ref, reason, deadline) -> cancellation_state`：传播取消；不承诺远端已停止。

`IngestKnowledge(evidence_refs, compilation_profile) -> candidate_refs | KnowledgeError`：隔离摄取并建立 claim 谱系。

`QueryKnowledge(request) -> claim_refs | KnowledgeError`：按 scope/purpose/freshness 查询。

`LintKnowledge(candidate_ref, compilation_profile) -> lint_report | KnowledgeError`：执行 deterministic/policy/semantic/security lint。

[REQ-CORE-OPS-001] 符合 Core 的实现 **MUST** 提供上述操作的等价可观察语义。

[REQ-CORE-OPS-002] 操作映射跨传输时 **MUST** 保留版本、错误、取消、幂等、流式和 unknown-outcome 语义。


### 12.1 Management API 与确定性管理门禁

Management API 是上述 Core Operations 的受治理管理组合面，可由可选 Intelligent Shell、确定性 CLI/Console 或自动化客户端调用；客户端身份不改变 authority、Effect 或验收语义。Intelligent Shell 完成一次生成/工具调用不表示管理 Task 已提交。

Intelligent Shell 的管理权只存在于 RFC-0001 定义的短期 `PrivilegedManagementSession` 内。Session 同时受 idle/absolute timeout、即时撤销、ActorChain/ActivityContext、scope、risk 与 policy/revocation version 约束；它是权限上界，不是对任一具体动作的批准，也不得被 reconnect、Conversation credential、continuation 或模型隐藏态当作可恢复 bearer 权限。

[REQ-MGMT-EFFECT-001] 每个有状态管理操作 **MUST** 固定 `expected_version`、稳定 idempotency key、参数/proposal/approval digest，并按 Intent→Authorize→Execute→Reconcile/Verify→Commit/Abort 执行；只有 authority commit 后管理 Task 才可完成。

[REQ-MGMT-IDEM-001] install、upgrade、remove、start、stop、configure、revoke 与 reconcile 等管理动作 **MUST** 声明幂等作用域；同 key 同参数返回同一 Effect/等价结果，同 key 异参拒绝。timeout、取消或 Shell 退出后的不确定结果 **MUST** 映射 `OUTCOME_UNKNOWN`，使用原 key 对账或隔离。

[REQ-MGMT-RECOVERY-001] Shell/runtime 崩溃、断连或取消 **MUST NOT** 丢弃未决管理 Effect；恢复路径必须从持久 Intent/Effect/Verification 重建、fence 旧 writer 并 reconcile 后才能提交、补偿或 quarantine。

[REQ-MGMT-FALLBACK-001] 实现声明管理能力时 **MUST** 提供不依赖模型或 domain Agent 的强认证确定性 Management API 与 CLI/Console emergency path。Shell disabled/unavailable 时，该路径仍可建立受限 session、检查状态、撤销 capability、停止执行、对账 Effect、配置 gateway/diagnostics，并保持同一 gate、审计与安全限制。

[REQ-MGMT-SESSION-LIFECYCLE-001] Session 过期、撤销或关闭 **MUST** 立即阻止新管理 proposal/dispatch，且 **MUST NOT** 删除、提交或成功化既有未决 Effect；恢复或 fallback 必须使用原 Intent、幂等键、Effect 与 Verification 记录进行 reconcile。Session 续期或跨连接/信任边界恢复必须重新认证、本地重验并产生新 session 版本或新 session。

## 13. 统一错误语义

错误至少包含 `code`、`stage`、`retryable`、`safe_reason`、`details_ref`、`observed_versions`、`correlation_id` 和可选 `retry_after`；`category` 由已登记 code 在 [errors.yaml](../registry/errors.yaml) 中的分类派生，code 尚未登记时必须显式给出（该口径与 RFC-0001 §13 及开发计划的 error-contract 项一致）。

retryable 表示在满足声明前置条件后可重试，不表示立即重试安全。

安全错误正文不得泄漏秘密、未授权对象存在性或 capability 内容。

**文本名与机器码边界**：下列各组名称是本规范的语义类别；机器登记码以 [errors.yaml](../registry/errors.yaml) 为准，未在其中登记的名称是待登记类别，实现 **MUST NOT** 把它们当作已注册机器资产引用。同概念的文本名/登记码对照为：`STATE_STALE` ↔ `STATE_STALE_OBSERVATION`、`OUTCOME_UNKNOWN` ↔ `EFFECT_OUTCOME_UNKNOWN`、`BUDGET_EXHAUSTED` ↔ `RESOURCE_BUDGET_EXHAUSTED`（上下文预算为 `CONTEXT_BUDGET_EXCEEDED`）、`CAPABILITY_EXPIRED` ↔ `AUTH_CAPABILITY_EXPIRED`、`QUARANTINED` ↔ `EFFECT_RECOVERY_QUARANTINED`。文本名与登记码的完整机器化对齐由开发计划的 error-contract 标准完成。

版本与状态：`STATE_CONFLICT`、`STATE_STALE`、`STATE_AUTHORITY_UNAVAILABLE`、`SNAPSHOT_INVALID`。

上下文：`CONTEXT_STALE`、`CONTEXT_BUDGET_EXCEEDED`、`CONTEXT_CONFLICT`、`CONTEXT_COMPRESSION_FAILED`、`CONTEXT_AUTH_DENIED`、`CONTEXT_INCOMPLETE`。

授权：`AUTH_DENIED`、`CAPABILITY_EXPIRED`、`CAPABILITY_REVOKED`、`CAPABILITY_SCOPE_MISMATCH`、`FENCING_REJECTED`。

协议：`VERSION_UNSUPPORTED`、`SCHEMA_MISMATCH`、`DIGEST_MISMATCH`、`CRITICAL_EXTENSION_UNKNOWN`、`PROTOCOL_MAPPING_INCOMPLETE`。

效果：`EFFECT_DENIED`、`OUTCOME_UNKNOWN`、`VERIFY_FAILED`、`COMPENSATION_FAILED`、`QUARANTINED`。

资源：`BUDGET_EXHAUSTED`、`DEADLINE_EXCEEDED`、`BACKPRESSURE_REJECTED`、`CANCEL_PENDING`。

恢复：`CHECKPOINT_INVALID`、`REPLAY_DIVERGED`、`RECONCILIATION_REQUIRED`、`AUTHORITY_UNRESOLVED`。

[REQ-ERR-001] 实现 **MUST NOT** 将安全、版本、required、unknown outcome 或验证错误静默转为成功。

[REQ-ERR-002] 只有在幂等、状态版本、预算和授权仍成立时，错误才 **MAY** 标记 retryable。

[REQ-SEC-001] 未知主体/authority/安全字段、失效 capability、digest 不符与跨租户引用 **MUST** fail closed 并审计。

## 14. 审计、恢复与重放

权威 audit 与可丢遥测分离；trace 丢失不影响状态或 Effect 正确性。

审计记录主体、purpose、对象版本、策略版本、授权 decision、资源消耗、reason code、effect 状态和结果引用。

普通 Agent 不得修改或删除权威 audit。

恢复顺序：建立 recovery barrier；验证 execution identity、推进 epoch 并安装 fencing；fence 旧 writer；验证 snapshot 并仅重放 committed history 至 high watermark；reconcile 未决 Effect；重验 GovernanceDomainContext、capability、freshness、budget 与 TaskContract；解析 ActivityContext/ContextView；最后恢复 loop。

确定性 workflow 重放消费已提交非确定活动结果，不重新调用 LLM、工具、人工或随机源。

重放产生不同 digest 时停止并返回 REPLAY_DIVERGED，不得覆盖现态。

未决 Effect 在恢复完成前阻止依赖其结果的完成声明。

[REQ-AUDIT-001] 每个 committed governed effect **MUST** 可追溯至 principal→TaskContract/Episode→Snapshot→Context→Checkpoint/Intent→授权→receipt→verification→Event→结果状态。

[REQ-AUDIT-002] audit **MUST** 具备防普通 Agent 篡改、顺序完整性、保留、敏感度控制与导出审计。

[REQ-SEC-002] TaskContract、verifier、控制层 ContextItem、预算/stop policy 与人工 intervention **MUST** 由 authority 固定并审计。

[REQ-REC-001] 恢复 **MUST** fence 旧执行者并 reconcile 所有可能已对外执行但未提交的 Effect。

[REQ-REC-002] 重放 **MUST NOT** 重新执行已提交的非确定性 activity。

## 15. 安全与符合性场景

### 15.1 必测正例

固定快照与预期版本一致时，CAS 产生新版本和关联 Event。

Context 解析保留 required 控制项、逐对象授权与 loss declaration。

稳定幂等键重复投递得到同一效果或可证明的效果等价一次。

崩溃恢复对账未决 Effect，且旧 fencing epoch 无法提交。

verifier 对固定后态通过后 Task 才完成。

### 15.2 必测负例

模型输出“已授权”不能改变授权 decision。

OperationDescriptor 宣称支持 refund 不能授予 refund 权限。

过期 snapshot 提交返回 STATE_STALE/CONFLICT，状态不变。

required 项超预算且不允许 partial 时返回 CONTEXT_BUDGET_EXCEEDED。

工具返回中的提示注入不能改写 control 层。

执行后断连进入 OUTCOME_UNKNOWN，不换幂等键盲重试。

模型或远端 completed 而 verifier 不通过时，Task 保持非完成。

旧 epoch、撤销 capability、跨租户 URI 和未知 critical 字段全部 fail closed。

### 15.3 声明规则

[REQ-CONF-001] 符合性声明 **MUST** 发布 manifest、实现版本、测试结果、已知降级和证据引用。

[REQ-CONF-002] 声明 **MUST** 固定 RFC、requirement set、schema bundle、编码 profile 与 suite digest。

[REQ-CONF-003] `implemented` **MUST** 表示全部适用 MUST 已通过或逐项记录降级；planned/experimental 不计覆盖。

[REQ-CONF-004] 外部协议测试 **MUST NOT** 替代 Core 状态、上下文、授权、效果、恢复与安全负例测试。

[REQ-PROFILE-CORE-001] 声明 core implemented 的实现 **MUST** 覆盖本规范全部适用要求。

[REQ-PROFILE-HARNESS-001] 声明 `harnessed_autonomous_execution` Profile 的实现 **MUST** 满足 REQ-RUN-001–REQ-RUN-009 全部适用要求，并在 profile manifest 中固定 harness、TaskContract 与 verifier 版本。

[REQ-PROFILE-HARNESS-002] 该 Profile 的恢复与迁移声明 **MUST** 以持久 LoopCheckpoint 及其 continuation conditions 证据支撑，**MUST NOT** 依赖不可移植的模型隐藏态。

## 16. 非目标与开放边界

Core 不规定 LLM、规划器、向量库、知识图谱、提示格式、Agent 拓扑或部署层数。

Core 不承诺一般 exactly-once、完全确定性模型重放、全局强一致或所有外部操作可补偿。

Core 不把长上下文、语义相似度、receipt、远端状态或多数票当作事实证明。

本 v0.2 为 companion draft；performance、management 与治理对象族 machine schema 已登记，知识对象仍是伪 schema；registry、测试与实现状态由各自资产单独声明。

## 17. Agent 生态共同机制（v0.3 候选扩展）
本节与五个 v0.1 Profile companion 共同定义跨实现对象边界；Core 仍不要求 LLM、向量库或特定存储。

共同对象族增加 `AgentPackageManifest`、`AgentInstallation`、`AgentCompatibilityReport`、`CognitiveResourceManifest`、`MemoryCandidate`、`MemoryAdmissionDecision`、`InformationGap`、`ContextRequestCandidate`、`ContextRequestAdmission`、`ContextViewDelta`、`OperationSummary`、`OperationCatalogSnapshot`、`OperationMatchReport`、`SemanticServiceManifest` 与 `CognitiveAllocationDecision`。对象存在、summary、descriptor、semantic ranking 和 installation 均不构成 capability。

共同传输无关操作增加：`DiscoverResources`、`AdmitContextRequest`、`ResolveContextDelta`、`ProposeMemory`、`AdmitMemory`、`InvalidateMemory`、`DiscoverOperations`、`BindOperation`、`InstallAgent`、`UpgradeAgent`、`RemoveAgent`、`MediateSemanticCandidate` 与 `AllocateCognitiveResources`。安装/升级/删除及 memory publish 是 governed Effect；纯候选调用仍受读取、预算、egress 与审计门禁。

[REQ-CORE-MEDIATION-001] 跨 Activity、Conversation、Principal、ResourceScope 或持久化边界的 memory/knowledge/context/catalog/semantic-derived 访问 **MUST** 使用受治理对象或等价标准服务语义，**MUST NOT** 以 Runtime 私有 cache、catalog 或 tool session 绕过 Core。

[REQ-CORE-CANDIDATE-001] Candidate、ranking、summary、match report、allocation proposal 与 Agent completion **MUST NOT** 被提升为 authorization、authority fact、committed Effect 或 completed Task。

[REQ-CORE-DELTA-001] 增量解析 **MUST** 绑定 base ContextView 和父预算，并且只能收窄治理边界；新增正文仍逐对象授权。

规范细节分别见 [Agent Compatibility](../agent-compatibility/README.md)、[Governed Memory](../governed-memory/README.md)、[Cognitive Discovery](../cognitive-discovery/README.md)、[Operation Catalog](../operation-catalog/README.md) 与 [Semantic Mediation](../semantic-mediation/README.md)。


## 18. User Intent 与 Agent Shell Core 语义

本节是 Core v0.2 Draft 的正式组成部分，不属于 §17 的 v0.3 候选扩展；对应 REQ-INTENT-*/REQ-SHELL-* 条目已在 [requirements.yaml](../registry/requirements.yaml) 登记并有声明式向量（第 15 测试层）。声明 Shell/意图能力的实现按 registry 条目逐项引用，不因实现本节而自动进入 §17 的 v0.3 生态扩展边界。

### 18.1 意图固定与接受
`UserIntentRecord` 是用户原始表达的不可覆盖记录；`IntentInterpretation` 是带目标、约束、禁止项、假设、歧义和 information gaps 的候选。TaskContract 必须引用被接受的 intent record 与 interpretation 版本。

[REQ-INTENT-RECORD-001] 实现 **MUST** 在语义解释前持久化或等价固定 UserIntentRecord 的主体、Conversation/ResourceScope、输入引用、时间与 digest；摘要、模型输出或后续修正 **MUST NOT** 覆盖原始记录。

[REQ-INTENT-ADMISSION-001] IntentInterpretation **MUST** 保持 candidate 状态，只有用户或明确的 intent authority 可接受；影响目标、scope、风险、费用、egress、验收或不可逆 Effect 的未决歧义 **MUST** 返回 clarification required，而不是选择 top-1。

[REQ-INTENT-SUPERSEDE-001] 用户修正 **MUST** 创建 superseding interpretation/TaskContract，推进 contract epoch，使旧 proposal/capability 失效，并在新 dispatch 前取消、fence 或安全收敛旧版本的未决工作。

[REQ-INTENT-ACCEPT-001] Task `COMPLETED` **MUST** 绑定 UserIntentRecord、被接受 interpretation、TaskContract、固定后态、VerificationReport 与 acceptance authority decision；Agent/Shell/Runtime 的完成文本不构成接受。

### 18.2 ShellActionProposal、Preview 与目标解析
普通任务命令和管理命令共享结构化 `ShellActionProposal` 语义，但普通通道绑定 AuthenticationSession/Conversation，管理通道额外绑定 PrivilegedManagementSession/approval。`TargetSelector` 可含自然语言、alias 或 query；有状态动作分派前必须解析为唯一强引用与 expected version。

[REQ-SHELL-TARGET-001] 有状态 Shell 操作 **MUST** 将 TargetSelector 解析为唯一、授权、固定版本目标；零候选、多个候选或 selector 与 preview 后发生漂移时分别 fail closed，不得猜测“它”“上一个 Agent”等指代。

[REQ-SHELL-PREVIEW-001] 写操作 preview **MUST** 固定 proposal digest、目标/版本、参数、effect/risk、预算/deadline、egress、所需权限/审批、验证、取消和补偿边界；执行必须引用仍有效的 preview/proposal，任何安全相关变化触发重新预览与确认。

[REQ-SHELL-CHANNEL-001] 普通任务与特权管理通道 **MUST** 隔离 credential、ContextView、working/KV cache、proposal、approval 与 audit binding；自然语言或 Conversation credential **MUST NOT** 切换通道或恢复管理权。

### 18.3 进程、任务与停止语义
Shell 管理的逻辑对象是 AgentExecution、Task、Activity 与 Effect，而非仅宿主 PID。`pause` 阻止新 Activity；`cancel` 请求目标收敛；`terminate` 终止可终止 Runtime 载体；`fence` 阻止旧 writer；`stop` 是按 TaskContract 组合这些动作的策略操作。

[REQ-SHELL-CONTROL-001] API/CLI **MUST** 分别报告 cancellation requested、cancel pending/confirmed/too late/unknown、runtime terminated、writer fenced、Effect reconciled 与 Task terminal；任一局部事实 **MUST NOT** 被提升为其他事实。

[REQ-SHELL-DETACH-001] Shell disconnect/exit **MUST NOT** 隐式取消 Task 或丢弃 Intent/Effect；attach 仅恢复经授权观察/控制绑定，**MUST NOT** 恢复旧 PrivilegedManagementSession。

### 18.4 Watch 与用户状态投影
`WatchSubscription` 绑定 selector、audience/purpose、可见字段、事件类型、snapshot version、cursor/high-watermark、dedupe window、budget、expiry 与 backpressure。用户进度是 authority 状态的派生投影，不是 transcript 或日志长度。

[REQ-SHELL-WATCH-001] watch 重连 **MUST** 从经授权 snapshot 加已确认 cursor 后的增量恢复，检测重复、缺口、乱序与过期 cursor；不得越权重放或把遥测当权威事件。

[REQ-SHELL-STATUS-001] 用户状态 **MUST** 至少区分 queued/runnable/waiting/blocked/cancel_pending/outcome_unknown/quarantined/candidate_complete/completed，并提供安全 reason、等待条件/主体、下一 gate、budget/deadline 和可用控制动作。
