# CognitiveOS 治理上下文、访问与受治理知识编译规范

- **版本**：v0.2 Draft
- **状态**：Normative Companion RFC
- **标识**：`cognitiveos.governance-context-access/0.2`
- **发布日期**：2026-07-18
- **依赖**：[CognitiveOS Core Companion Specification](./specs/core/README.md)
- **对应白皮书**：[CognitiveOS 总体架构白皮书](./CognitiveOS-Architecture.md)

> 本 RFC 是可独立评审的规范性 companion 草案，不表示实现。仓库已提供 performance report machine schema、registry 条目与 declarative conformance vector；本文 Governance/Conversation/Knowledge 对象代码块仍为伪 schema，尚不能冒充已登记机器对象 schema。

## 1. 规范约定、状态与范围

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。只有大写英文规范词构成规范性要求。

本 RFC 规范：Tenant、Membership、Principal、EffectiveSubject、ActorChain、Delegation、ShareGrant、Conversation、ResourceScope、AgentExecution 绑定；PrivilegedManagementSession、ManagementActionProposal、ManagementApprovalDecision 及其 ActivityContext 绑定；混合授权；认知数据准入；执行上下文；Context Resolution 扩展；受治理知识编译；AKP 封装；错误、审计、缓存、检索、迁移与安全测试语义。

本 RFC 不定义身份提供商、数据库、向量库、模型、提示模板、UI、组织计费、法定合规结论或跨组织信任协议。它不把租户当成权限等级，不承诺管理员可读取正文，也不声称任何现有实现已满足本文。

冲突时，部署 **MUST** 采用不扩大读取、写入、委派、用途、保留期或跨域传播的解释；与基础 Core 无法兼容时 **MUST** 显式拒绝并记录迁移或版本冲突。

## 2. 术语

- **Tenant**：数据、策略、密钥、配额、审计与管理责任的隔离域；不是权限等级。
- **TenantContext**：一次判定固定的 tenant、策略版本、成员版本、撤销版本、地域及信任域上下文。
- **Membership**：Principal 与 Tenant 之间有期限、状态和属性的关系；成员身份本身不授予对象正文读取权。
- **Principal**：可认证、授权和审计的主体，分为 Human、Workload、Device。
- **EffectiveSubject**：授权判定中承担当前权限边界的主体，不等同于发起者或执行工作负载。
- **ActorChain**：从 initiating principal 经委派到 effective subject，再到 workload/device 的不可变有序链。
- **Delegation**：有范围、有期限、可撤销、单调衰减的权力传递。
- **Conversation**：参与者围绕连续交互形成的持久资源作用域；不是认证会话或运行时连接。
- **Turn**：Conversation 内有序的输入、输出或系统事件单元。
- **Participant**：以明确角色和有效期参与 Conversation 的 Principal 或受控代理。
- **ResourceScope**：对象可见、可派生、可保留和可晋升的资源边界。
- **ShareGrant**：资源所有域向另一主体或 tenant 明确授予有限访问的对象。
- **AgentExecution**：逻辑执行身份，绑定发起、有效、工作负载主体、ActorChain、版本和 Conversation。
- **AuthenticationSession**：认证协议产生的短期登录状态。
- **RuntimeSession**：模型、工具、连接或进程的短期运行状态。
- **Capability**：短期、可衰减、可撤销且受 audience/purpose/resource/action/parameter 约束的授权凭证。
- **AdmissionDecision**：认知对象进入特定 scope、用途或生命周期阶段的准入决定。
- **KnowledgeClaim**：带 claim-level provenance、适用 scope/时间与依赖关系的原子主张。
- **KnowledgeCompilationProfile**：固定抽取、规范化、去重、lint、冲突、发布、模型与渲染依赖的版本化配置。
- **PrivilegedManagementSession**：由 session authority 签名、绑定 Human/ActorChain/ActivityContext 且受 domain/action/risk/time/policy/revocation 限制的短期管理授权。
- **ManagementActionProposal**：自然语言意图经结构化后形成、固定目标/参数/版本/幂等键/风险的管理动作提案。
- **ManagementApprovalDecision**：对固定 proposal challenge 作出的、可验证 authority 签名决定。

## 3. 系统不变量

1. Tenant **MUST** 仅表示隔离与治理域；实现 **MUST NOT** 用 tenant 层级推导权限高低。
2. 同一 Tenant 的两个 Principal **MUST NOT** 因共同成员身份互相读取资源。
3. tenant 管理员、平台管理员和基础设施运维者 **MUST NOT** 自动获得正文读取权；break-glass 必须是独立、限时、可审计授权。
4. 每次读取、派生、执行和提交 **MUST** 绑定请求治理上下文、ActorChain、purpose、ResourceScope 和版本：tenant 执行使用 TenantContext；platform 管理操作使用显式 PlatformContext。
5. 授权 **MUST** 默认拒绝，显式 deny **MUST** 优先于 allow。
6. 最终允许集合 **MUST NOT** 超过用户委派、工作负载权限、TaskContract、资源策略和适用 capability 的交集。
7. Conversation、working memory、KV cache、ContextView 和临时索引 **MUST** 按授权绑定隔离，不能因模型、进程或 tenant 相同而串用。
8. 跨 tenant 访问 **MUST** 通过 ShareGrant 或联邦域内重新授权；远端 allow、角色名或 bearer URI 不能直接复用。
9. 派生数据 **MUST** 继承或收紧源对象的 scope、purpose、sensitivity、compartment、retention 和 lineage。
10. 缓存命中、ANN 命中、全文命中、摘要或 embedding **MUST NOT** 绕过对象级重验与撤销。
11. 每个受治理对象 **MUST** 显式声明 `scope_domain` 为 `tenant` 或 `platform`；缺失或 nullable tenant **MUST NOT** 被解释为公共。tenant 对象的 `tenant_id` 必填且在对象身份生命周期内不可变；platform 对象只能由显式 platform authority 治理。
12. AgentExecution 的 tenant **MUST** 在执行生命周期内不可变。切换 tenant **MUST** 新建 AgentExecution，或终止当前执行并从经授权 Checkpoint 恢复为新 AgentExecution；**MUST NOT** 只替换 binding。

## 4. 公共对象头与伪 schema

以下伪 schema 表示规范语义，不声称仓库中已有机器 schema。

```text
GovernedObjectHeader {
  id: StableId
  type: String
  schema_version: String
  object_version: UInt64
  scope_domain: tenant | platform
  tenant_id: TenantId?  // scope_domain=tenant 时必填；platform 时禁止
  resource_scope: ResourceScopeRef
  owner: PrincipalRef
  authority: AuthorityRef
  policy_refs: [PolicyRef]
  purpose_constraints: [Purpose]
  sensitivity: Sensitivity
  compartments: [Compartment]
  retention: RetentionPolicy
  provenance: Provenance
  lineage: Lineage
  content_digest: Digest
  created_at: Timestamp
  valid_time: TimeRange?
}
```

`scope_domain` **MUST** 显式存在。`scope_domain=tenant` 时 `tenant_id` **MUST** 存在、与 ResourceScope 一致，并在对象身份生命周期内不可变；`scope_domain=platform` 时 `tenant_id` **MUST NOT** 出现，且不能用 null、缺失 tenant 或特殊 tenant 值暗示公共。platform 对象 **MUST** 由独立 platform authority 和 platform policy 治理，tenant administrator **MUST NOT** 创建、改写或将 tenant 正文降格为 platform 对象。`tenant_id` 不能作为 allow 条件的充分证据。`owner` 是生命周期责任主体，不自动拥有所有 action；`authority` 是写入/仲裁边界，不自动拥有正文读取权。复制、摘要、embedding、索引条目、缓存条目和 View **MUST** 携带等价公共头或可验证引用。

```text
ResourceScope {
  scope_domain: tenant | platform
  tenant_id: TenantId?  // tenant domain 必填；platform domain 禁止
  kind: private | conversation | task | project | tenant_shared
        | federated_share | non_conversational | platform
  scope_id: StableId
  audience_constraints: [SubjectSelector]
  purpose_constraints: [Purpose]
  compartments: [Compartment]
  parent_scope: ResourceScopeRef?
  policy_refs: [PolicyRef]
}
```

`kind=platform` **MUST** 与 `scope_domain=platform` 同时出现，其 authority **MUST** 是显式 platform governance authority；tenant authority、tenant administrator 或普通 workload **MUST NOT** 签发 platform scope。其他 kind 默认为 tenant domain；`non_conversational` 表示无 Conversation 的显式 Activity 作用域，不表示公共。scope 继承 **MUST** 单调收紧。跨 scope 或 scope_domain 晋升不是原地改标签，见 §9。

## 5. Tenant、Membership、Principal 与 ActorChain

### 5.1 TenantContext 与 Membership

```text
GovernanceDomainContext = TenantContext | PlatformContext

PlatformContext {
  scope_domain: platform
  platform_authority: AuthorityRef
  policy_version: UInt64
  revocation_version: UInt64
  trust_domain: String
  evaluated_at: Timestamp
}

TenantContext {
  scope_domain: tenant
  tenant_id: TenantId
  membership_id: MembershipId?
  membership_version: UInt64
  policy_version: UInt64
  revocation_version: UInt64
  region: String?
  trust_domain: String
  evaluated_at: Timestamp
}

Membership {
  id: MembershipId
  tenant_id: TenantId
  principal_id: PrincipalId
  status: invited | active | suspended | revoked | expired
  role_bindings: [RoleBinding]
  attributes: Map<String, Value>
  valid_from: Timestamp
  valid_until: Timestamp?
  version: UInt64
}
```

`PlatformContext` **MUST** 仅由 platform governance authority 建立，并且不授予任何 tenant 正文读取权。Membership 状态非 `active` 时，依赖该 Membership 的新授权 **MUST** 拒绝。角色只提供候选 permission；资源策略、关系、属性、purpose、capability 和 deny 仍须计算。

### 5.2 Principal 类型

```text
Principal {
  id: PrincipalId
  kind: human | workload | device
  authentication_authority: AuthorityRef
  attributes: Map<String, Value>
  status: active | suspended | revoked
}
```

Human Principal 表示人；Workload Principal 表示服务、Agent runtime、模型代理或自动化；Device Principal 表示受认证设备或执行器。共享账号 **SHOULD NOT** 作为 Principal；不可避免时必须保留个体 initiating principal。

### 5.3 EffectiveSubject、ActorChain 与 Delegation

```text
ActorChain {
  initiating_principal: PrincipalRef
  effective_subject: PrincipalRef
  workload_principal: PrincipalRef
  device_principal: PrincipalRef?
  delegations: [DelegationRef]
  chain_digest: Digest
}

Delegation {
  id: StableId
  issuer: PrincipalRef
  subject: PrincipalRef
  tenant_context: TenantContext
  audience: [Audience]
  purpose: [Purpose]
  resources: [ResourceSelector]
  actions: [Action]
  parameter_constraints: Map<String, Constraint>
  valid_until: Timestamp
  max_depth: UInt8
  revocation_ref: RevocationRef
  parent: DelegationRef?
}
```

ActorChain **MUST** 保持顺序、完整性和可审计性。每级 Delegation **MUST** 单调衰减；任一链节点撤销、过期、暂停或版本失配都 **MUST** 使依赖链失效。Workload Principal **MUST NOT** 冒充 initiating principal；审计必须同时保留二者。

### 5.4 AgentExecution 绑定

```text
AgentExecutionBinding {
  execution_id: StableId
  tenant_context: TenantContext
  actor_chain: ActorChain
  initiating_subject: PrincipalRef
  effective_subject: PrincipalRef
  workload_subject: PrincipalRef
  conversation_binding: ConversationBinding?
  task_ref: TaskRef?
  membership_version: UInt64
  policy_version: UInt64
  revocation_version: UInt64
  capability_refs: [CapabilityRef]
  fencing_epoch: UInt64
}
```

AgentExecution **MUST** 固定这些绑定，其中 tenant 在该 execution 生命周期内 **MUST** 不可变。后台执行 **MAY** 不含 `conversation_binding`；这不授予任何默认对话。恢复、迁移、主体切换、Conversation 切换、版本变化或 capability 变化后 **MUST** 重新授权；不得仅恢复进程内 session。tenant 切换 **MUST** 创建新的 AgentExecution，或终止当前执行并以新 execution identity 从经授权 Checkpoint 恢复；**MUST NOT** 原地替换 TenantContext、tenant_id 或 binding。

## 6. 混合授权模型

授权决策 **MUST** 组合：

- RBAC：Membership 中角色产生的候选 action；
- ABAC：主体、资源、环境、时间、地域、设备健康、sensitivity 与 purpose 属性；
- ReBAC：owner、participant、delegator、reviewer、share recipient 等关系；
- Capability：针对具体 audience、purpose、resource、action、参数和期限的短期权力。

```text
CandidateAllow = RBAC ∪ ABAC ∪ ReBAC
UpperBound = UserDelegation ∩ WorkloadPermissions ∩ TaskContract
             ∩ ResourcePolicy ∩ ApplicableCapabilities
Decision = (CandidateAllow ∩ UpperBound) - ExplicitDeny
```

公式仅表达集合上界。实现 **MUST** 默认拒绝；任一适用显式 deny **MUST** 胜过 allow。Capability **MUST NOT** 扩大基础策略允许集合；没有 capability 的纯读取是否允许由资源策略声明，但敏感读取 **SHOULD** 使用短期 capability。

授权请求至少包含 TenantContext、ActorChain、action、resource、purpose、parameters、TaskContract、环境属性和版本，并包含 ConversationBinding 或显式 `non_conversational` ResourceScope。授权响应必须固定输入 digest、decision、理由类别、有效期和重验条件。

```text
AuthorizeRequest {
  tenant_context, actor_chain, conversation_binding?, non_conversational_scope?,
  action, resource_ref, purpose, parameters_digest,
  task_contract_ref?, capability_refs, environment
}
AuthorizeResponse {
  decision: allow | deny | challenge
  decision_digest: Digest
  effective_actions: [Action]
  expires_at: Timestamp
  recheck_on: [Condition]
  safe_reason_codes: [String]
}
```

`challenge` 表示需要额外认证、审批或 break-glass，不是临时 allow。

## 7. Conversation 模型

### 7.1 Conversation、Turn 与 Participant

```text
Conversation {
  header: GovernedObjectHeader
  state: open | suspended | closed | archived | revoked
  participants: [Participant]
  task_refs: [TaskRef]
  current_turn: UInt64
  retention: RetentionPolicy
}

Participant {
  principal_ref: PrincipalRef
  relation: owner | contributor | observer | agent | reviewer
  joined_at: Timestamp
  valid_until: Timestamp?
  policy_refs: [PolicyRef]
}

Turn {
  header: GovernedObjectHeader
  conversation_id: StableId
  sequence: UInt64
  actor_chain_digest: Digest
  input_refs: [ObjectRef]
  output_refs: [ObjectRef]
  created_task_refs: [TaskRef]
}
```

Participant 关系 **MUST NOT** 自动越过对象 scope 或 sensitivity；离开 Conversation 后，历史可见性由当时和当前策略、保留及撤销共同决定。

### 7.2 三类 session 的区分

AuthenticationSession 只证明某时点完成认证；RuntimeSession 只承载连接、模型 KV 或工具状态；Conversation 是持久交互资源。三者 **MUST** 使用不同 ID 和生命周期。认证或 runtime 复用 **MUST NOT** 推导 Conversation 访问权。

### 7.3 ConversationBinding 与层级关系

```text
ConversationBinding {
  conversation_id: StableId
  conversation_version: UInt64
  participant_relation_version: UInt64
  history_scope: ResourceScopeRef
  working_scope: ResourceScopeRef
}
```

关系为：`Conversation -> Task -> Episode -> Activity`。一个 Conversation 可产生多个 Task；Task 可跨多个 Episode；Episode 包含有界 Activity。一个 Task 可被显式关联到多个 Conversation。每次 Activity **MUST** 恰好选择以下之一：一个主 ConversationBinding，或一个 `kind=non_conversational` 的 ResourceScope；选择前者时，跨 Conversation 输入作为显式、经授权的对象引用；选择后者时，不得隐式读取任一 Conversation。

### 7.4 并发、切换、恢复与历史可见性

同一 tenant、同一用户可并发多个 Conversation。默认每个并发 Conversation **MUST** 使用独立 AgentExecution。上下文、KV cache、working memory、临时文件、工具会话、检索候选和未决 Effect **MUST** 隔离。

若实现复用 AgentExecution，复用 **MUST** 发生在 Activity 边界，并依次完成：

1. 创建通用 Checkpoint，保存 LoopCheckpoint payload；
2. 对账全部未决 Effect；
3. 清空或按新绑定隔离 working set、KV cache 和临时凭证；
4. 重验 TenantContext、ActorChain、Membership、policy 与 revocation；
5. 为目标 Conversation 重新执行 Context Resolution；
6. 安装新 ConversationBinding 与 fencing epoch 后再运行。

AgentExecution 未绑定 Conversation 时，Activity **MUST NOT** 从 AuthenticationSession、RuntimeSession、用户偏好或先前 Activity 推导“最近对话”。切换 Conversation **MUST NOT** 把上一 Conversation 的隐藏态作为唯一输入。恢复历史时 **MUST** 逐对象按当前权限重验；“曾经看过”不构成永久读取权。被撤销对象可保留不可逆审计 digest，但正文不得继续呈现。


## 7.5 PrivilegedManagementSession 与管理授权

`PrivilegedManagementSession` 是强认证 Human Principal 与 management session authority 建立的短期管理授权对象，不是永久 root Agent、普通 AuthenticationSession、Conversation 或可转让 bearer token。其机器合同见 `specs/schemas/privileged-management-session.schema.json`。

[REQ-MGMT-SESSION-001] Session authority **MUST** 对 session 的 human principal、ActorChain digest、AuthenticationContext、ActivityContext、management domain、action/resource scope、risk ceiling、policy version、revocation epoch、idle timeout、absolute expiry、state 与 digest 进行完整性保护并签名；Shell workload **MUST** 作为用户 ActorChain 中的 workload principal 行动。

[REQ-MGMT-SESSION-002] 实现 **MUST** 同时执行 idle timeout 和 absolute timeout，并在过期、关闭、撤销、ActorChain/ActivityContext 不匹配、policy/revocation version 失配时立即拒绝新操作；撤销不得等待缓存或 Conversation 结束。需要更高风险、敏感目标、远程/sidecar 恢复或策略指定动作时 **MUST** step-up reauthentication。

Session 可授予固定边界内广泛管理 action，但 **MUST NOT** 向模型暴露 root keys、签名密钥或可派生等价秘密；**MUST NOT** 允许 Shell/模型自签、自授权、扩大 session scope、修改或删除权威审计、绕过 R2/R3 独立审批、覆盖实时安全仲裁，或把 unknown outcome 转为 success。

[REQ-MGMT-SESSION-003] Session 续期、重新连接、remote/sidecar 使用、continuation 恢复、trust-domain hop、协商 epoch 或任一安全绑定变化 **MUST** 创建新的完整性受保护 session 版本或新 session，并重新认证、本地重验和审计；实现 **MUST NOT** 仅凭旧 session ID、Conversation credential、缓存或模型隐藏态恢复管理权。Session 过期、撤销或关闭后，依赖它的新 proposal/capability **MUST** 拒绝；已持久化 Intent/Effect 保留并进入 §7.6 的对账流程，不能因 session 终止而丢弃或自动提交。

## 7.6 ManagementActionProposal、challenge 与 approval

自然语言确认、按钮文本、日志、Shell 输出或其他 Agent 文本只可触发结构化 challenge/proposal 流程，**MUST NOT** 自身构成批准、capability 或 authority。`ManagementActionProposal` 固定 session、domain/action/targets、参数 digest、expected versions、idempotency key、risk、ActorChain、ActivityContext、policy version 与期限；`ManagementApprovalDecision` 固定 proposal/session digest、decision、deciding authority、approver ActorChain、challenge、risk、policy、期限与签名。机器合同分别见 `management-action-proposal.schema.json` 与 `management-approval-decision.schema.json`。

[REQ-MGMT-GATE-001] 每个管理写操作 **MUST** 依次通过 session validity、ActorChain/ActivityContext binding、scope、risk、policy/revocation version、step-up、required independent approval、expected-version 与 idempotency gate；任一 gate 失败必须 fail closed。

[REQ-MGMT-APPROVAL-001] 当 policy 要求或操作为 R2/R3 时，批准者 **MUST** 独立于提议 Shell/workload；提议者、执行者或其模型 **MUST NOT** 自批。approval 过期、challenge/proposal digest 不匹配或批准者不独立时不得执行。

[REQ-MGMT-AUTHZ-001] Session authority **MUST** 是批准签名与 session authority 的受信根；Shell、模型、普通 Agent 与被管理 runtime **MUST NOT** 签发、扩张或替换该 authority。

[REQ-MGMT-TRUST-001] Shell/log/tool/other-agent text **MUST** 按 untrusted input 处理；控制字段只能来自结构化、schema-valid、完整性受保护对象。每个 proposal、challenge、approval、denial、dispatch、reconcile、verify 和 commit **MUST** 产生逐操作审计。

一次允许 decision **MUST** 只适用于其固定 proposal/session/ActorChain/ActivityContext/target version/idempotency key；它不能把 session 提升为无界 capability。Shell 输出“完成”、本地进程退出或远端 `completed` 只形成候选证据；只有 Effect 对账、verifier 与 authority commit 可以建立管理改变和 Task 完成事实。

## 7.7 隔离、正文访问与秘密

Privileged management context、challenge、credential、step-up material、签名输入、root key handle 与 management cache **MUST** 与普通 Conversation、AgentExecution、working memory、KV/prompt cache、Memory/Knowledge admission 和日志正文隔离；普通 Agent/Conversation **MUST NOT** 接收或恢复这些内容。审计只保存必要引用/digest，不能记录可复用秘密。

管理权不等于 tenant 正文读取权。即使 Human 是管理员且 session 为 active，读取 tenant body 仍 **MUST** 通过目标资源的独立正文授权或显式 break-glass 流程；management scope 不提供自动读取。

## 8. ShareGrant 与跨 tenant

```text
ShareGrant {
  header: GovernedObjectHeader
  source_tenant: TenantId
  target_tenant: TenantId?
  grantee: SubjectSelector
  resource_refs: [ObjectRef]
  actions: [Action]
  purposes: [Purpose]
  transform_policy: PolicyRef?
  retention: RetentionPolicy
  valid_until: Timestamp
  revocation_ref: RevocationRef
  onward_sharing: none | constrained
}
```

ShareGrant **MUST** 由源资源 authority 签发，目标域 **MUST** 本地重新授权。跨 tenant capability **MUST NOT** 被直接接受为本地 capability。联邦身份只建立认证映射，不建立正文读取权。目标域保存副本或派生物时 **MUST** 记录 source tenant、grant、lineage、用途和撤销处理；ShareGrant 撤销后的保留行为必须由 grant 与法律/审计策略显式声明。

<a id="knowledge-compilation"></a>
## 9. Knowledge、Memory 与受治理知识编译

### 9.1 编译管线与统一治理字段

规范管线为：`Evidence -> ClaimCandidate -> KnowledgeCandidate -> AdmissionDecision -> Published KnowledgeObject -> ContextView`。raw source、检索命中、模型输出和发布对象均不自动等于 world truth；本管线 **MUST NOT** 被简化为无治理 RAG，也 **MUST NOT** 允许模型直接发布。

KnowledgeObject、KnowledgeClaim、MemoryObject、ContextItem/View、缓存、embedding、索引、摘要和其他派生数据 **MUST** 携带或继承 `scope_domain`、tenant/platform context、ResourceScope、owner、policy、purpose、sensitivity、compartment、retention、provenance、valid time 和 lineage。缺失必要绑定的对象 **MUST NOT** 进入生产 ContextView。

### 9.2 KnowledgeClaim 与依赖图

```text
KnowledgeClaim { id, text_digest, evidence_refs, scope, valid_time,
                 status, confidence_method, compilation_profile_ref,
                 supports, contradicts, supersedes, derived_from }
KnowledgeCompilationProfile { id, version, chunking, extraction, normalization,
                              deduplication, conflict_policy, citation_policy,
                              deterministic_lint, policy_lint, semantic_lint,
                              security_lint, model_provider_sampling, render_profile }
```

每个 claim **MUST** 保留 claim-level provenance 和上述依赖边。上游 Evidence/claim 更新、删除、撤销、许可变化、可信度下降或 supersede 时，实现 **MUST** 传播 invalidation，固定受影响集合并触发有界 recompile；旧 embedding、摘要、缓存和索引 **MUST** 同步失效或标 stale。

[REQ-KNOW-001] 发布 claim **MUST** 具有 claim-level provenance、evidence refs、scope、valid time、profile 与依赖边。

[REQ-KNOW-002] 上游失效 **MUST** 传播到全部可达派生物并产生可审计 invalidation/recompile 结果。

### 9.3 Ingest、Query、Lint 与 Admission

- `Ingest(evidence, profile)`：在隔离区解析，建立 Evidence/ClaimCandidate 与谱系；
- `Query(request)`：在 tenant/scope/compartment/purpose 过滤、对象级授权、freshness 和 conflict policy 后返回 claim；
- `Lint(candidate, profile)`：执行 deterministic（schema/digest/引用闭合）、policy（scope/purpose/license/retention）、semantic（claim-evidence/矛盾/时效）和 security（注入、恶意链接、秘密、跨域、持久投毒）四层 lint。

语义组件 **MAY** 提出 claim 或 lint finding，但 AdmissionDecision/publish **MUST** 由目标 scope authority 与确定性门禁控制。生成内容、同一模型改写、循环引用或下游再次引用 **MUST NOT** 作为独立 corroboration。

[REQ-KNOW-003] `Ingest` **MUST** 隔离不可信内容并记录 profile、来源、许可与内容 digest。

[REQ-KNOW-004] `Query` **MUST** 在候选泄露前过滤，并在返回正文前逐对象重新授权。

[REQ-KNOW-005] `Lint` **MUST** 覆盖 deterministic、policy、semantic、security 四层并保存版本化结果。

[REQ-KNOW-006] 模型或生成内容 **MUST NOT** 绕过独立 admission/publish authority，且自生成链 **MUST NOT** 自我佐证。

### 9.4 有界维护、删除与 legal hold

维护 loop 为 `discover changes -> invalidate -> recompile -> lint -> review/admit -> atomic publish -> refresh indexes/caches -> audit`。它 **MUST** 声明时间、费用、对象数、递归深度与重试上界；超界时保留最后已知安全版本并标 stale，或 quarantine，不能假定接近零维护成本。

source removal **MUST** 沿 lineage 删除或失效所有允许删除的正文和派生索引；必要 tombstone 只保存 digest、删除原因、authority、时间与依赖闭合状态。legal hold **MAY** 阻止物理删除，但 **MUST NOT** 自动允许继续 Query 或 Context 呈现。

[REQ-KNOW-007] 维护 loop **MUST** 有界并对超界、部分重编译与发布原子性给出机器状态。

[REQ-KNOW-008] source deletion/legal hold **MUST** 区分保留义务与读取授权，并证明派生 closure 或列出无法删除的边界。

### 9.5 持久投毒防御与符合性

实现 **MUST** 测试 ingestion sandbox、压缩/递归上限、MIME/签名、内容/控制隔离、跨源独立性、scope/sensitivity 继承、发布职责分离、删除传播和发布后异常监控。性能和质量按 architecture §18.4 的 BenchmarkManifest 报告 claim precision/recall、citation coverage、unsupported-claim、contradiction detection、stale exposure、invalidation latency、recompile、deletion closure 与 poison admission/escape，并给出分母、窗口、p50/p95/p99 和置信区间。

[REQ-KNOW-009] 知识编译符合性 **MUST** 覆盖持久投毒、删除/invalidation、冲突、未授权查询与 generated self-corroboration 负例。

## 10. ExecutionContext 与 ActivityContext

```text
ExecutionContext {
  execution_binding: AgentExecutionBinding
  authentication_context_ref: ObjectRef
  task_contract_ref: TaskContractRef?
  checkpoint_ref: CheckpointRef?
  policy_bundle_digest: Digest
  environment_digest: Digest
}

ActivityContext {
  activity_id: StableId
  execution_id: StableId
  conversation_binding: ConversationBinding?
  non_conversational_scope: ResourceScopeRef?
  task_ref: TaskRef
  episode_ref: EpisodeRef
  actor_chain_digest: Digest
  purpose: Purpose
  resource_scope: ResourceScopeRef
  context_view_ref: ContextViewRef
  capability_refs: [CapabilityRef]
  budget_ref: BudgetRef
  started_at: Timestamp
}
```

ExecutionContext 是 AgentExecution 级固定治理包；ActivityContext 是一次 Activity 的更窄绑定。`conversation_binding` 与 `non_conversational_scope` **MUST** 恰有一个存在；后者 **MUST** 指向 `kind=non_conversational` 且与 execution tenant 一致的 ResourceScope。ActivityContext **MUST** 不扩大 ExecutionContext，也 **MUST NOT** 隐式继承“最近对话”。每个 Effect、ContextView、审计事件和派生对象 **MUST** 可追溯到 ActivityContext。

`Checkpoint` 是跨实现的通用恢复封装；`LoopCheckpoint` 是其中承载 Loop 控制事实的 payload；`Continuation` 是恢复后可执行的后续体及前置条件引用。三者 **MUST NOT** 混作同一对象或把模型隐藏态作为唯一恢复依据。

## 11. Context Resolution 扩展

ContextRequest 除 Core 字段外 **MUST** 包含 TenantContext、ActorChain digest、ResourceScope、purpose、policy/membership/revocation versions 和 ActivityContext 引用，并与 ActivityContext 一致地包含 ConversationBinding 或显式 non-conversational scope，二者恰有一个。

解析流水线 **MUST** 满足：

1. 在 ANN、全文检索、图查询或外部 ranker 之前执行 tenant、scope、compartment、purpose 可见性过滤；
2. 索引若暴露敏感术语、计数或存在性，候选发现本身也必须授权；
3. 在正文解密、读取或呈现前逐对象重验 owner/policy/relation/capability/撤销；
4. 派生项按最严格源策略继承，除非有明确脱敏决策；
5. 返回 rejected 项时不得泄露未授权对象存在性或正文；
6. ContextView 绑定 ActivityContext，不能跨 Conversation 或 purpose 复用。

```text
ResolveContextRequest {
  core_request,
  tenant_context, actor_chain_digest, conversation_binding?,
  non_conversational_scope?, resource_scope, activity_context_ref,
  membership_version, policy_version, revocation_version
}
ResolveContextResponse {
  context_view_ref?, complete, loaded_refs, safe_rejections,
  admission_decision_refs, authorization_decision_refs,
  pinned_versions, cache_binding_digest
}
```

## 12. AKP envelope 与操作

AKP envelope 扩展如下；它是伪 schema，不是已登记 wire schema。

```text
AKPGovernanceEnvelope {
  protocol_version: String
  message_id: StableId
  tenant_context: TenantContext
  actor_chain: ActorChain
  conversation_binding: ConversationBinding?
  execution_id: StableId?
  activity_context_ref: ObjectRef?
  purpose: Purpose
  resource_scope: ResourceScopeRef?
  policy_version: UInt64
  membership_version: UInt64
  revocation_version: UInt64
  correlation_id: StableId
  causation_id: StableId?
  deadline: Timestamp
  payload_digest: Digest
  payload: Any | ObjectRef
}
```

接收方 **MUST** 在解释敏感 payload 前验证 envelope、版本和本地授权。推荐的传输无关操作为：

- `ResolveTenantContext(principal, tenant) -> TenantContext | Error`
- `AuthorizeAccess(request) -> AuthorizeResponse | Error`
- `CreateConversation(request) -> ConversationBinding | Error`
- `AppendTurn(request) -> TurnRef | Error`
- `SwitchConversation(request) -> AgentExecutionBinding | Error`
- `IssueShareGrant(request) -> ShareGrantRef | Error`
- `RevokeShareGrant(ref) -> RevocationRef | Error`
- `IngestKnowledge(request) -> KnowledgeCandidate | Error`
- `QueryKnowledge(request) -> KnowledgeClaimSet | Error`
- `LintKnowledge(request) -> KnowledgeLintReport | Error`
- `InvalidateKnowledge(request) -> InvalidationReport | Error`
- `RecompileKnowledge(request) -> KnowledgeCandidate | Error`
- `AdmitKnowledge(request) -> AdmitKnowledgeResponse | Error`
- `PromoteKnowledge(request) -> AdmitKnowledgeResponse | Error`
- `ResolveContext(request) -> ResolveContextResponse | Error`

切换、晋升和共享操作 **MUST** 使用幂等键、expected version 和审计关联。远端成功只证明远端响应，不能替代本地提交。

## 13. 错误语义

错误对象至少包含 `category`、`stage`、`retryable`、`safe_reason`、`observed_versions`、`correlation_id` 和可选 `retry_after`。本 RFC 使用已登记的通用错误，并定义下列知识/治理语义类别；只有 `specs/registry/errors.yaml` 中实际存在的 code 才是机器注册资产：

- tenant/membership 不可用、暂停、过期或版本失配；
- ActorChain 无效、委派越界、撤销或深度耗尽；
- Conversation 不可见、关系失效、绑定陈旧或并发冲突；
- ResourceScope 不匹配、跨 tenant 未共享、purpose 不匹配；
- capability 缺失、过期、撤销或参数不符；
- admission 被拒绝、要求 review、对象被 quarantine/revoked/expired；
- cache binding 失配、索引过滤不足、对象级重验失败；
- execution reuse 不安全、未决 Effect 未对账或 working set 未隔离；
- knowledge lint 失败、依赖失效、recompile 超界、source deletion 未闭合或 poisoning quarantine。

安全拒绝 **MUST NOT** 泄露资源存在性、标题、相似度、参与者、tenant 名称或 capability 内容。`retryable` 仅表示前置条件变化后可重试，不表示立即重试安全。

## 14. 审计

审计 **MUST** 区分 initiating、effective、workload 和 device principal，并记录：tenant、Membership、ActorChain/Delegation digest、Conversation/Turn、Task/Episode/Activity、purpose、resource/scope、policy/membership/revocation versions、capability、decision、拒绝类别、ShareGrant、AdmissionDecision、cache/index 路径、Effect 与结果引用。

管理员读取审计元数据不自动允许读取正文。正文审计导出 **MUST** 独立授权。跨 tenant 事件 **MUST** 在双方域产生可关联但各自最小披露的记录。普通 Agent 和 tenant 管理员 **MUST NOT** 修改权威审计。

## 15. 缓存、embedding 与检索

缓存键至少绑定：

```text
CacheKey = digest(scope_domain, tenant_id_or_platform, ActorChain,
                  ConversationBinding_or_non_conversational_scope,
                  resource_scope, purpose, policy_version,
                  membership_version, revocation_version,
                  object_versions, transform_version, target_profile)
```

缺少任一安全相关维度的缓存 **MUST NOT** 用于敏感数据。KV cache、prompt cache、工具缓存、embedding cache、检索结果缓存、摘要缓存与 ContextView cache 均适用。撤销版本推进、Membership/Participant 变化、Conversation 切换、purpose 变化或对象策略变化 **MUST** 失效或触发重验。

ANN/全文索引 **MUST** 使用 tenant/scope/compartment 分区或等价的检索前强制过滤。检索后过滤不能弥补已向共享 ranker 泄露向量、词项、标题或存在性的架构。每个候选正文在返回前 **MUST** 逐对象重验。embedding 与索引 **MUST** 被视为源数据的敏感派生物，并受删除、保留和驻留策略约束。

## 16. 迁移与兼容

当前标识使用 `cognitiveos.*` 与 `cognitiveos_conformance`。旧 `agentos.*` ID/property 仅可经显式 legacy adapter 或固定旧 schema 接收；适配器 **MUST** 记录源/目标版本、映射 digest 与审计，**MUST NOT** 在同一对象、消息、manifest 或协议 epoch 内静默混用。无法无损映射时 **MUST** 返回 `PROTOCOL_MAPPING_INCOMPLETE`。

旧对象缺少 tenant、`scope_domain` 或 scope 时 **MUST** 进入隔离迁移区，不得默认归入“公共”、platform 或当前 tenant。只有经 platform governance authority 明确判定的对象才可迁移为 `scope_domain=platform`。迁移工具 **MUST** 产生可审计映射、owner/authority 决策、策略版本和不确定项；无法确定归属的对象保持 quarantined。

旧 Session ID **MUST NOT** 自动转换为 Conversation ID。可从旧 transcript 创建 Conversation，但必须新建对象、参与关系、scope、retention 与 AdmissionDecision。旧 AgentExecution 恢复时必须补齐 ActorChain 和 ConversationBinding；无法补齐则只读隔离或拒绝恢复。

兼容实现 **MAY** 在单 tenant 部署使用固定 TenantContext，但仍 **MUST** 执行同租户非隐含授权、Conversation 隔离和对象级 scope。版本降级 **MUST NOT** 丢弃 critical governance 字段。

## 17. 安全测试与验收

实现声明支持本 RFC 前，至少应证明以下正例与负例；performance contract 已有 declarative vector；知识编译场景仍是规范清单，除非 conformance 索引明确列出对应 vector。

### 17.1 必测正例

- 同一 Human 在两个 Conversation 并发运行，使用独立 AgentExecution 和隔离 cache；
- 授权交集收窄，RBAC allow 被资源 deny 覆盖；
- ShareGrant 经目标 tenant 本地重授权后只允许声明 purpose；
- MemoryObject 从 proposed 经 review/validated 晋升时产生新决定、新版本和 lineage；
- Conversation 切换完成 checkpoint、Effect 对账、working set 隔离、重授权与重解析；
- 撤销版本推进使缓存失效，正文读取重新判定。
- 强认证建立受 scope/risk/idle/absolute timeout 限制的 PrivilegedManagementSession，管理写操作经 proposal、step-up/独立批准（适用时）、Effect 对账和 authority commit；
- Session 关闭后拒绝新 proposal，但关闭前已持久化且结果未知的管理 Effect 保留原幂等键并可被确定性 fallback 对账。

### 17.2 必测负例

- 同 tenant、同角色或管理员身份不能读取未授权 Conversation 正文；
- RuntimeSession/KV cache 不得把 Conversation A 内容带入 B；
- ANN/全文检索不得在 scope 过滤前向 ranker 暴露敏感候选；
- 旧 Membership、旧 Participant relation、旧 policy 或旧 revocation version fail closed；
- URI、对象 ID、远端 allow、ShareGrant 文本或自然语言批准不能作为 bearer 授权；
- quarantined MemoryObject 不能直接 published；
- 跨 scope 晋升不能原地改标签；
- AgentExecution 有未决 Effect 时不能切换并继续执行；
- AgentExecution 不能通过原地替换 TenantContext、tenant_id 或 binding 切换 tenant；必须创建新 execution，或终止并恢复到新 execution；
- federation authentication 不能自动变成本地读取许可。
- 旧 session ID、普通 Conversation credential、Shell reconnect、remote/sidecar 或 continuation 不能恢复或转移 privileged management authority；
- Shell/模型、被管理 runtime 或提议者不能自签 session、自批 R2/R3 proposal、扩大 scope、读取 root key，或把 timeout/退出映射为成功。

### 17.3 验收标准

符合性声明 **MUST** 固定 RFC 版本、适用范围、实现版本、策略模型、迁移状态和测试证据。所有适用 MUST 必须通过，或逐项声明不支持；`planned`、`experimental` 和人工演示不算通过。声明 **MUST NOT** 把本文伪 schema 当作机器 schema，也只能引用实际登记的 registry 条目和 conformance vector；当前 performance 资产已登记，知识对象 schema/vector 仍未登记。

## 18. 开放问题

1. ShareGrant 撤销与已依法保留派生物之间如何形成可互操作证明？
2. 大规模 ReBAC 图如何在检索前过滤中兼顾低延迟与无存在性泄露？
3. Conversation 多归属 Task 的主绑定和审计聚合是否需要独立对象？
4. 短期 capability 的离线验证、撤销传播和分区行为如何按风险分级？
5. embedding 删除证明与近似索引重建如何标准化？
6. break-glass 正文访问怎样实现双人控制、最小披露和事后通知？
7. 跨域 ActorChain 证明如何避免暴露源 tenant 的内部组织结构？
8. 哪些伪 schema 应优先进入机器 schema，哪些应保留为策略接口？

## 19. 机器规范化边界

本 v0.2 Draft 已登记 performance report schema、PrivilegedManagementSession、ManagementActionProposal、ManagementApprovalDecision machine schema、REQ-PERF/REQ-KNOW registry 条目与 performance declarative vector。GovernanceDomainContext、Conversation、KnowledgeClaim、KnowledgeCompilationProfile 和 Ingest/Query/Lint 对象仍是伪 schema；后续可另行登记 JSON/CBOR schema、错误码、迁移工具与 knowledge vectors。只有实际提交、索引并固定 digest 的资产可用于机器符合性声明；管理 schema 的存在不证明管理 session、gate 或 Shell 已实现。
