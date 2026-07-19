# CognitiveOS Cognitive Discovery Companion Specification

> 版本：v0.1 Draft  
> 标识：`cognitiveos.cognitive-discovery/0.1`

## 1. Resource Manifest
`CognitiveResourceManifest` 是绑定 ActivityContext 的逻辑命名空间，列出可发现 context domains、memory kinds、knowledge domains、state authorities、operation/verifier categories、expandable refs、query capabilities、fault protocol、budget summary、freshness 与 policy version。它不要求暴露全部标题、对象、工具或隐藏资源存在性。

[REQ-DISC-MANIFEST-001] Manifest **MUST** 绑定主体、Conversation/ResourceScope、purpose、policy/revocation、budget 与 expiry；discover 权限 **MUST NOT** 被解释为 read/call capability。

## 2. Candidate Admission
`ContextRequestCandidate → Normalize → Bind ActivityContext → Narrow Scope → Apply TaskContract → Apply required/forbidden → Budget/Egress/Freshness → ContextRequestAdmission`。Admission 保存原 candidate、最终 request、修改理由与 policy version。

[REQ-DISC-ADMIT-001] 概率组件产生的 candidate **MUST NOT** 被直接执行；确定性 admission 只能收窄，强制约束变化必须可见且可审计。

## 3. 增量解析
`InformationGap` 固定 decision/claim、missing type、why required、authority/evidence、freshness/deadline、risk 与 prior attempts。expand kind 仅为 `content_expand|relation_expand|authority_refresh`。

`ContextViewDelta` 绑定 base view，列新增、删除/失效、替换、冲突/loss、累计成本、complete 变化和下一 expandable refs。

[REQ-DISC-DELTA-001] Delta **MUST NOT** 扩大 base ActivityContext、purpose、scope、audience、egress 或原 hard budget；每个新增正文重新授权。

## 4. 停滞与错误隐私
ResolutionAttempt 保存 query/candidate fingerprint、信息增益、缺口/冲突变化、成本、延迟与拒绝。连续无增益、重复摘要、source 不可达、预算/deadline 不足进入 `WAIT|ESCALATE|ACCEPT_AUTHORIZED_PARTIAL|STOP`。Partial 必须由 TaskContract 预授权。

[REQ-DISC-STAGNATION-001] 实现 **MUST** 有 bounded attempts 和机器可观察停滞出口，**MUST NOT** 把服务不可用解释为无结果。

[REQ-DISC-PRIVACY-001] not-found、not-discoverable 与 denied 的外部安全表示 **MUST NOT** 泄露未授权对象存在性；审计可在独立授权下保留真实内部原因。

## 5. 符合性
测试 discover/read 分离、delta scope expansion、累计预算、authority refresh、重复查询、撤销后 expandable ref 失效、跨 Conversation reuse 与存在性侧信道。
