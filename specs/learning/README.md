# CognitiveOS Controlled Learning Companion Specification

> 版本：v0.2 Draft

> 状态：Companion Specification；仅定义语义与符合性要求，不表示存在实现。

> 标识：`cognitiveos.learning/0.2`

> 范围：学习候选、记忆治理与离线/shadow/canary/release/rollback 发布控制


## 1. 规范约定

本文中的 **MUST**、**MUST NOT**、**SHOULD**、**SHOULD NOT**、**MAY** 按 RFC 2119 与 RFC 8174 解释。

只有大写英文规范词构成规范性要求；普通中文“必须/应当”仅作说明。

本规范叠加于 CognitiveOS Core；冲突时采用更严格的安全、授权、预算与证据边界。

实现状态必须声明为 `implemented`、`planned`、`experimental` 或 `unsupported`，不得把本草案当作实现证据。

## 2. 范围与原则

在线运行产生候选和证据，不直接自改生产策略、模型、verifier 或权威状态。

学习管线是受治理发布系统，不保证持续改进，也不将单个成功 Episode 视为泛化证据。

生产 effect 与学习评估 effect 使用分离 authority、预算和数据政策。

## 3. 四类候选

`MemoryCandidate`：拟保留、更新、合并或遗忘的情节/语义记忆。

`PolicyCandidate`：Context、loop、tool routing、retry 或其他 ReasoningPolicy 变更。

`ModelCandidate`：基础模型、adapter、prompt program、embedding/ranker 或其组合版本。

`VerifierCandidate`：验收、grader、测试、阈值或证据组合规则变更。

候选携带 source episodes、hypothesis、scope、owner、authority、training/eval data refs、sensitivity、risk、expected benefit 与 rollback plan。

[REQ-LEARN-CAND-001] 四类候选 **MUST** 是版本化对象，并与当前生产版本和证据谱系关联。

[REQ-LEARN-CAND-002] VerifierCandidate **MUST** 由独立于被评估候选的 authority 审批。

## 4. 记忆治理

MemoryObject 区分 observation、episode summary、decision、procedure、preference 与 knowledge claim。

相关性不等于真实性；模型摘要不自动成为权威事实。

写入前执行 provenance、tenant/purpose、敏感度、保留、冲突、去重和投毒检查。

更新保留旧版本与 supersedes/contradicts 关系，不静默覆盖冲突。

遗忘可删除正文但保留合规所需 tombstone、digest 与审计；法律保留优先于学习便利。

召回时重新检查权限、purpose、freshness 与当前 policy。

[REQ-LEARN-MEM-001] 记忆晋升 **MUST** 有来源、验证状态、适用范围、retention 和 authority。

[REQ-LEARN-MEM-002] 不可信输入、模型输出或单一 Episode **MUST NOT** 自动晋升为全局事实或 control 项。

[REQ-LEARN-MEM-003] 删除/遗忘 **MUST** 遵循保留、legal hold、派生追踪和可审计 tombstone 政策。


## 5. 受治理知识编译

知识发布遵循 `Evidence -> ClaimCandidate -> KnowledgeCandidate -> AdmissionDecision -> Published KnowledgeObject -> ContextView`。`KnowledgeCompilationProfile` 固定抽取、去重、冲突、citation、deterministic/policy/semantic/security lint、模型/provider/sampling 与渲染版本。

[REQ-KNOW-001] 发布 KnowledgeClaim **MUST** 保存 claim-level provenance、evidence refs、`supports|contradicts|supersedes|derived_from` 依赖、scope 与 valid time。

[REQ-KNOW-002] source/claim 更新、撤销、删除或许可变化 **MUST** 沿依赖图传播 invalidation，并使派生 object/index/cache 失效或标 stale。

[REQ-KNOW-003] `Ingest` **MUST** 隔离不可信 Evidence；`Query` **MUST** 在候选泄露前过滤并逐对象授权；`Lint` **MUST** 覆盖 deterministic、policy、semantic、security 四层。

[REQ-KNOW-004] 模型输出、自生成改写、循环引用和同源摘要 **MUST NOT** 形成独立 corroboration 或直接发布。

[REQ-KNOW-005] 维护 loop **MUST** 有时间、费用、对象数、递归和重试上界；超界 **MUST** stale/quarantine，而非无限自修。

[REQ-KNOW-006] source removal **MUST** 传播到允许删除的派生物并保留最小 tombstone；legal hold **MUST NOT** 被解释为 Query 授权。

## 6. 发布阶段

### 6.1 Offline

固定数据集/任务分布、环境、基线、候选、预算、grader/verifier 和随机性；检查泄漏与代表性。

报告 outcome、安全拒绝、成本、延迟、停滞、人工介入和分层失败归因，不只报告平均分。

### 6.2 Shadow

候选读取镜像输入但不拥有生产写 capability；其输出只进入隔离评估存储。

shadow 必须控制敏感数据复制、额外费用和出域。

### 6.3 Canary

canary 获得最小生产流量、作用域 capability、预算、时间窗与自动停止条件。

与 control 同时监测任务、群体、租户、风险和尾部指标。

### 6.4 Release

发布固定 artifact digest、配置、schema、migration、compatible contract、审批与生效 epoch。

逐步扩大作用域；不得把 planned/shadow 状态标记为 production。

### 6.5 Rollback

rollback 是预先测试的版本化迁移，恢复 artifact、policy、state/schema 与 routing。

不可逆数据/外部效果必须有 forward-fix、补偿或隔离计划，不能假设回滚代码即回滚世界。

[REQ-LEARN-OFF-001] 离线评估 **MUST** 固定任务/环境、harness、模型/候选、verifier、预算与数据版本。

[REQ-LEARN-SHD-001] shadow 候选 **MUST NOT** 获得生产 governed-effect 写 authority。

[REQ-LEARN-CAN-001] canary **MUST** 有有界流量、capability、预算、监控、kill criteria 和回滚目标。

[REQ-LEARN-REL-001] release **MUST** 固定 artifact/config/schema digest、审批、迁移、epoch 与证据。

[REQ-LEARN-RBK-001] rollback **MUST** 被测试并区分代码回退、状态回退和外部效果补偿。

## 7. 数据与评估安全

训练/评估数据按 tenant、purpose、consent、retention、sensitivity 与 residency 管理。

EpisodePackage 默认不要求保存 prompt、秘密或思维链；只保留最小可归因证据。

测试污染、反馈回路、选择偏差、reward hacking 和 grader drift 必须作为风险记录。

LLM-as-judge 不得作为资金、权限、安全、合规或生产完成的唯一 gate。

候选不得通过修改 verifier 来制造自身提升；联合变更必须拆分或做因果控制。

[REQ-CONF-005] 比较结果 **MUST** 绑定任务/环境、harness、模型/采样、grader/verifier、预算和干预。

[REQ-CONF-006] 高风险完成声明中 LLM-as-judge **MUST NOT** 是唯一验收者。

[REQ-LEARN-DATA-001] 数据使用 **MUST** 可追溯到允许的 purpose、consent/authority、驻留和保留政策。

## 8. 失败与恢复

候选状态至少为 PROPOSED、QUARANTINED、EVALUATING、SHADOW、CANARY、APPROVED、RELEASED、REJECTED、ROLLED_BACK。

失败包括 EVAL_INCONCLUSIVE、DATA_POLICY_DENIED、SHADOW_DIVERGENCE、CANARY_REGRESSION、RELEASE_CONFLICT、ROLLBACK_FAILED、VERIFIER_DRIFT。

评估不可判定时不得发布；canary 安全或 hard 指标越界时停止扩量并执行回滚/隔离。

发布中断使用 CAS 和 epoch 恢复，防止两个版本同时宣称当前。

[REQ-PROFILE-LEARN-001] 生产状态、策略、模型或 verifier 变更 **MUST** 经过版本发布、独立评估、审批、可回滚迁移与审计；Episode 不得直接修改生产行为。

[REQ-LEARN-FAIL-001] 评估证据缺失、冲突或 verifier 漂移时 **MUST** 停止晋升并返回机器错误。

## 9. 安全与符合性场景

记忆投毒：工具文本要求写入 control memory，被隔离/拒绝。

数据泄漏：跨租户 Episode 进入训练集，被 purpose/tenant gate 拒绝。

指标投机：PolicyCandidate 同时弱化 verifier，被独立审批和对照阻止。

shadow 越权：候选尝试生产写入，capability 拒绝。

canary 回归：安全拒绝率或尾部风险超门限，自动停止并回滚。

回滚失败：状态 schema 不可逆，进入隔离和 forward-fix，不伪报恢复。

记忆冲突：新摘要与权威事实冲突，保留 conflict set 而非覆盖。

[REQ-LEARN-SEC-001] 实现 **MUST** 测试记忆投毒、数据泄漏、评估污染、reward hacking、verifier 篡改与回滚竞态。

[REQ-LEARN-CONF-001] 声明 **MUST** 列出四类候选支持、各阶段 gate、数据治理、独立 authority、回滚覆盖和证据引用。

## 10. 与在线 Governed Memory 的边界
本规范的 `MemoryCandidate` 既可来自学习流水线，也可来自在线 Episode，但准入统一交给 Governed Memory 的目标 scope authority。在线 admission 只发布 MemoryObject，不等于发布 Policy/Model/Verifier；后者继续经过 Offline→Shadow→Canary→Release。

[REQ-LEARN-MEM-004] MemoryCandidate 的在线准入 **MUST NOT** 顺带改变模型、策略、verifier、TaskContract 或 authority；联合候选必须拆分并独立评估。

[REQ-LEARN-MEM-005] memory consolidation/forgetting 算法可替换，但其输出仍是 candidate；跨 Conversation/subject/knowledge 晋升必须产生新 admission、scope、retention 与 lineage。


## Shell 与用户旅程映射
Shell 可提出 candidate、启动评估或回滚，但“从这次成功中学习”不能直接修改生产。

[REQ-LEARN-SHELL-001] Shell 学习/发布动作 **MUST** 显示候选、阶段、数据 purpose、独立 verifier、canary/rollback 边界并走发布 authority。
