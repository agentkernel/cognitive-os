# CognitiveOS Semantic Mediation Companion Specification

> 版本：v0.1 Draft  
> 标识：`cognitiveos.semantic-mediation/0.1`

## 1. SMS 边界
Semantic Mediation Service 可执行 need compilation、query planning、authorized-candidate reranking、operation matching、context transform 与 information-gap detection。输出仅为 proposal/candidate/ranking/estimate，不是 authority、capability、事实、Effect commit 或 Task acceptance。

[REQ-SEM-SOFT-001] SMS 输出 **MUST** 带 candidate 语义，**MUST NOT** 扩大 ResourceScope、解除 forbidden、签发 capability、改变 TaskContract 或推进权威状态。

## 2. 确定性外壳
`SemanticServiceManifest` 声明 `unsupported|basic|vector|model_assisted|llm_mediated`、backend/provider/model/version、residency/retention/training、用途/risk ceiling、fallback。调用固定 template/planner/version/sampling、输入 digest、ActivityContext、egress、预算、schema、timeout；输出经 schema、reference existence/authorization、sensitivity 和 budget 验证。

[REQ-SEM-ENVELOPE-001] 每次非确定语义调用 **MUST** 固定可归因 envelope 并审计；unsupported reference 与授权失败必须拒绝。

## 3. 降级
可用链为 primary model → local model → vector/reranker → BM25/type → static catalog → wait/escalate。不可用与无候选必须区分。

[REQ-SEM-FALLBACK-001] 降级 **MUST NOT** 扩大权限、数据、egress、预算或 risk ceiling，也不得静默删除 required 或默认选择第一个 Operation。

## 4. Cognitive Resource Broker
CRB 输入 TaskContract、ActivityContext、AttentionBudget、ContextView、InformationGap、ResourceGraph、risk/deadline/policy/health 与 SMS soft signal；输出 `CognitiveAllocationDecision`：`admitted|denied|wait|degrade|escalate`、resolver/provider、reservation、target profile、egress、verifier、fallback 与 reasons。

[REQ-CRB-BOUND-001] CRB 只能在已授权 hard ceiling 内选择和预留；capability、residency、deadline、risk、concurrency、egress、fencing、cancel/reclaim **MUST** 由确定性机制执行。

[REQ-CRB-ACCOUNT-001] 多轮解析的 reservation/consumption **MUST** 计入同一父预算，验证和安全退出预留不得被语义优化挪用。

## 5. Shell 与实时边界
Shell、Agent adapter、native Agent 和 background job 均可作为 SMS 客户端；Shell 不是 SMS、memory、catalog 或 authority 宿主。硬实时周期禁止动态 SMS/CRB；技能回路只用固定 Descriptor 和预加载参数。

## 6. 指标与符合性
报告 gap closure、marginal information gain、unsupported references、operation top-k/effect confusion、fallback correctness、egress denial、stagnation、latency/cost 与按风险分层结果；模型不可用场景必须保持确定性管理与停止路径。


## Shell 与用户旅程映射
SMS 可解释 Shell 文本与生成 candidate，但不得读取 privileged secret、确认用户意图或选择有歧义高风险目标。

[REQ-SEM-SHELL-001] Shell semantic fallback **MUST** 保持原 intent digest、channel isolation、歧义和 hard bounds；模型不可用不影响确定性 inspect/stop/reconcile。
