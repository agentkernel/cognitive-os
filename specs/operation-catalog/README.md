# CognitiveOS Operation Catalog Companion Specification

> 版本：v0.1 Draft  
> 标识：`cognitiveos.operation-catalog/0.1`

## 1. Catalog 生命周期
Operation 注册、更新、健康变化和下线形成版本化 `OperationCatalogSnapshot`。Summary 与 full Descriptor digest 绑定；endpoint/schema/health 漂移触发新 snapshot 或 stale。

[REQ-CAT-LIFECYCLE-001] 注册/下线 **MUST** 由 catalog authority 提交并保留版本、descriptor digest、endpoint attestation、availability 与审计。

## 2. 两级目录
`OperationSummary` 提供 semantic name、short description、I/O type summary、effect/risk/data/realtime class、approval hint、descriptor version/digest、availability。完整 Descriptor 仅在候选后加载 schema、pre/postcondition、idempotency、query/reconcile、compensation、endpoint、verifier、resource/egress/attestation。

## 3. Discover、Match 与 Bind
顺序为 scope/discover visibility → effect/risk → type → precondition → residency/deadline/resource → rules/ontology → authorized-candidate semantic ranking → disambiguation → full descriptor bind。LLM 不得恢复被前序门禁移除的操作。

[REQ-CAT-DISCOVER-001] discover 输出 **MUST** 是候选而非 capability，并不得暴露当前 scope 不可发现 Operation。

`OperationMatchReport` 列 top-k、匹配依据、schema/effect/risk 差异、缺失 precondition、approval、不确定性和 dry-run/read-only 替代。

[REQ-CAT-MATCH-001] effect class 或高风险差异未消除时，matcher **MUST NOT** 自动执行 top-1；必须比较、dry-run、升级或停止。

[REQ-CAT-BIND-001] InvocationBinding **MUST** 固定 catalog snapshot、descriptor/endpoint/schema digest、health epoch 与参数类型；漂移必须重绑定并重新授权。

## 4. 安全探索
Descriptor 声明 validate-only、dry-run、plan、cost estimate、explain-precondition、query/reconcile。高风险路径优先建立低风险证据，并独立处理 semantic intent、candidate、planner selection 与 authorized Intent。

## 5. 指标与符合性
报告 correct-operation top-1/top-k、schema-compatible rate、prohibited exposure、effect confusion、false no-tool、unnecessary high-risk、dry-run disagreement、post-selection auth rejection、verifier failure、repeat discovery 与 latency/cost。测试相似 read/refund、catalog stale、descriptor rug pull 与 capability confusion。
