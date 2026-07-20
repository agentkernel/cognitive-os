# CognitiveOS 研究资料簿（zh-CN）

> 用途：为博客文章与本地图解提供可复核的作者研究账本；不作为公开路由，不是规范资产，也不是实现证据。
>
> 采集快照：2026-07-20T07:41:57+08:00；父仓库 HEAD `b626e88be3b985399051e6e7624223b9cb38e7c6`（merge commit 时间 `2026-07-20T07:30:03+08:00`）；采集时本资料簿所读取的父仓库 tracked source 路径无未提交改动，已授权的 `personal-blog/` 子工程改动不计入父来源状态。
>
> Hash 说明：本文记录的 SHA-256 仅表示**研究快照文件 hash**。它们不是 CognitiveOS 登记的 specification-set digest、requirement-set digest、schema-bundle digest 或符合性声明。

## 1. 来源层级与使用规则

1. **Tier 1 — 登记机器与测试资产**：registry、schema、transition JSON、声明式 conformance vector。机器 schema 只对可表达的形状有权威性；transition/vector 不自动证明行为已执行。
2. **Tier 2 — Normative behavior**：固定版本的 Core、RFC 与 standards。它们定义 authority、顺序、失败、恢复与验收语义；Draft 必须带版本与快照解释。
3. **Tier 3 — Informative architecture**：`CognitiveOS-Architecture.md` 与产品说明。用于解释双内核、三平面、七层和设计动机，不能覆盖 Tier 1/2。
4. **Tier 4 — 计划与状态证据**：PROGRESS、findings ledger、handoff。只说明某一时点的工程状态，不创造规范要求。

冲突处理：采用不扩大权限、数据范围、风险、预算或完成声明的解释。公开文章用“草案要求”“迁移表定义”“研究快照显示”，不使用无证据的“系统保证”。

## 2. 问题、范围与非目标

### FACT-COS-001 — 问题边界

CognitiveOS 处理的是跨时间运行、混合概率推理与确定性程序、可能改变数字或物理世界的 Agent 工作负载。核心问题不是增加模型调用，而是固定状态解释权、执行身份、Context 读取范围、授权、外部 Effect、恢复与验收边界。

- 来源：`CognitiveOS-Architecture.md` §0 “执行摘要”、§1.4 “明确非目标”
- Tier：3 informative
- 快照 hash：`e71b5dd83549ec7b7eb278e609d22fcea503279ef5918e3aceab10a62dd54b0c`
- 边界：不替代宿主 OS、数据库、消息系统、基础设施编排、行业认证或智能算法；不宣称通用智能理论。

### FACT-COS-002 — 保证不能由架构名称推出

模型或 verifier 仍可能误判；一般跨系统副作用不能无条件承诺 exactly-once；安全、性能、SLO、RPO/RTO 和行业认证都需要部署证据。

- 来源：`CognitiveOS-Architecture.md` §1.5 “保证边界与架构假设”；`specs/core/README.md` §7 Event Protocol
- Tier：2 + 3，行为边界以 Core 为先
- Core 快照 hash：`532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`

## 3. Informative 总体架构

### FACT-COS-003 — 双内核、三平面、七层、横切 Context

白皮书提供四个正交解释视图：

- 双内核：认知微内核负责持久执行、状态、Context 门禁、授权、Effect、预算与恢复；实时安全内核负责硬实时包络、watchdog、急停和最终执行器仲裁。
- 三平面：体验平面；控制平面；执行与数据平面。
- 七层：①宿主/网络/设备/物理世界；②资源织构与异构计算；③操作/技能/运行时；④认知微内核与 AKP；⑤Context/状态/知识/记忆/目录；⑥Harness 与认知服务；⑦Agent 与应用。
- Context Engineering：贯穿三平面与七层的横切服务，不是第四 authority 平面。

- 来源：`CognitiveOS-Architecture.md` §4.1–§4.5
- Tier：3 informative
- 快照 hash：`e71b5dd83549ec7b7eb278e609d22fcea503279ef5918e3aceab10a62dd54b0c`
- 公开限制：可称“白皮书的 informative view”；不可称“已部署双内核”或“已证明实时安全”。

## 4. Proposal 与 deterministic authority

### FACT-COS-004 — 概率组件只产候选

LLM、检索、embedding、ranker、matcher、summary 等组件可以发现、排序、缩小或变换候选。授权、CAS、schema 检查、状态机合法性、硬预算、幂等判定、fencing 与最终提交由确定性机制执行。

- 来源：`specs/core/README.md` §2（`REQ-CHARTER-DET-001`）、§6.5、§15.5（`REQ-CORE-CANDIDATE-001`）
- Tier：2 normative behavior
- 快照 hash：`532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`

### FACT-COS-005 — authority 分工

目标状态域 authority 对写入/仲裁负责；task-acceptance-authority 对 Task 完成负责；effect-authority 与 verification-authority 管理各自生命周期。观察、远端报告、工具 receipt、模型叙述和 ContextView 默认不获得 authority。

- 来源：`specs/registry/state-domains.yaml` `domains[*].authority_role`；`specs/core/README.md` §4、§5；`specs/transitions/task.transitions.json`
- Tier：1 + 2
- registry hash：`17ee88cabe13b0c539559e507e36c417455eeb6204a1907848f9c17757296d40`

## 5. OperationDescriptor 与 AuthorizationCapability

### FACT-COS-006 — 描述与权限严格分开

`OperationDescriptor` 回答 endpoint 能做什么以及如何调用：输入/输出 schema、effect class、幂等、取消、查询/对账、版本、endpoint 与限制。`AuthorizationCapability` 回答谁为了哪个 purpose，可在什么资源、动作、参数和期限内执行。本地 authority 签发 capability，派生只能收窄。

- 来源：`specs/core/README.md` §4、§8（`REQ-OP-001/002`、`REQ-CAP-001..005`）
- Tier：2 normative behavior
- 快照 hash：`532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`
- 机器资产限制：当前只有 `operation-summary.schema.json`、`operation-catalog-snapshot.schema.json`、`operation-match-report.schema.json` 等相关 schema，没有完整 `OperationDescriptor` schema。
- 开放项：F-023 要求的不可查询/不可幂等执行器准入矩阵仍开放。

## 6. 五个独立执行生命周期

### FACT-COS-007 — 域、初态与状态

五个域必须正交持久化，不能合并成单一“进度”。

1. **agent-execution**，authority `execution-authority`，初态 `CREATED`：`CREATED, ADMITTED, RUNNABLE, WAITING, CHECKPOINTED, RECOVERING, SUSPENDED, QUARANTINED, TERMINATED`。
2. **task**，authority `task-acceptance-authority`，初态 `DRAFT`：`DRAFT, READY, ACTIVE, BLOCKED, CANDIDATE_COMPLETE, COMPLETED, FAILED, CANCELLED, ESCALATED`。
3. **loop**，authority `execution-authority`，初态 `START`：`START, OBSERVE, RESOLVE, ORIENT, DECIDE, ACT, VERIFY, CONTINUE, DIAGNOSE, WAIT, QUARANTINE, RECONCILE, ESCALATE, STOP, END`。
4. **effect**，authority `effect-authority`，初态 `PROPOSED`：`PROPOSED, AUTHORIZED, DENIED, EXECUTING, EXECUTED, OUTCOME_UNKNOWN, RECONCILED, VERIFIED, VERIFY_FAILED, COMPENSATING, NOT_EXECUTED, COMMITTED, ABORTED, QUARANTINED`。
5. **verification**，authority `verification-authority`，初态 `NOT_REQUESTED`：`NOT_REQUESTED, PENDING, EVIDENCE_READY, PASSED, FAILED, INCONCLUSIVE, EXPIRED`。

- 来源：`specs/registry/state-domains.yaml`；五份 `specs/transitions/*.transitions.json`
- Tier：1 registered machine/test asset
- 文件 hashes：
  - agent-execution：`d29c2926ee9a2ceae945201df25e712ddfbc65000323b24af70466e81652f616`
  - task：`770484aeb2b04f2afd76e07512fb01b5b9760517ce5b879bb58714ed1b375f17`
  - loop：`db836e190920cc008804b8a195f3c01cf2cf696039003914464badd0539509ef`
  - effect：`e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e`
  - verification：`4975c774094ae4f48cd4217497bde27cb71005be41bfd2a1df5ea21aa65e0ade`

### FACT-COS-008 — CANDIDATE_COMPLETE 不是完成

Task 只有从 `CANDIDATE_COMPLETE` 经 `ACCEPTANCE_GRANTED` 才能进入 `COMPLETED`；guard 要求 acceptance authority 匹配、Verification 通过且当前、固定后态未变化。失败、过期、证据不足或争议走返回、阻塞、失败或升级路径。

- 来源：`specs/transitions/task.transitions.json` transition `CANDIDATE_COMPLETE -> COMPLETED`；`docs/standards/task-loop-verification.md` §5
- Tier：1 + 2
- standard hash：`5075759a0ce6940707bf47b65d7b930819b841978996307620345f8c93996a42`

## 7. ContextView 与两套九阶段词汇

### FACT-COS-009 — ContextView 非 authority

ContextView 是绑定 Activity 的短期工作投影，报告 loaded/rejected、损失、固定版本、成本、谱系和 complete。授权、CAS、迁移与提交必须重验 authority 状态；不可信内容不能通过 ContextView 提升成 control。

- 来源：`specs/core/README.md` §6.4–§6.5；`docs/standards/context-resolution-and-cache.md` §6
- Tier：2
- standard hash：`e973a36801a5e66393a1942186a5b1a82f80e200393e331d24f23fb2f95683a8`

### FACT-COS-010 — 九阶段命名差异

- Core vocabulary：`discover → filter → authorize → rank → budget → transform → verify → render → audit`。
- Context standard vocabulary：`ContextRequest admission → governance pre-filter → candidate retrieval → per-object authorization re-validation → semantic ranking/selection → budget fitting → loss declaration → deterministic rendering → ContextView emission with provenance`。

后者强调实现合同顺序；前者是 Core 概念阶段。两者不能静默逐项等同。图 5 并列展示并标记 discrepancy。

- 来源：`specs/core/README.md` §6.3；`docs/standards/context-resolution-and-cache.md` §2
- Tier：2
- hashes：Core `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11`；standard `e973a36801a5e66393a1942186a5b1a82f80e200393e331d24f23fb2f95683a8`

### FACT-COS-011 — ContextRequest prose/shape mismatch

Core/RFC/standard prose要求治理绑定覆盖 Tenant、ActorChain、Conversation/Task、ActivityContext、ResourceScope 等维度。当前 `context-request.schema.json` 通过 `GovernedObjectHeader` 提供治理 header，但 `perspective` 只显式要求 `principal`、`task`、`episode`，未把 prose 中全部绑定作为该对象的直接结构字段。不要在博客里自行发明缺失字段。

- 来源：`specs/schemas/context-request.schema.json` `/properties/header` 与 `/properties/perspective`；`specs/core/README.md` §0、§6.2；`docs/standards/context-resolution-and-cache.md` §2、§4
- Tier：shape 以 Tier 1 schema 为准，行为边界以 Tier 2 为准
- schema hash：`080747d0f1510cab9d94c1cca30dae43bda9f51f2eb2a2a75ac16cad9062b104`

## 8. Intent、Effect、幂等、未知与对账

### FACT-COS-012 — No Intent, no dispatch

governed external side effect 之前必须持久化 Intent，固定稳定幂等键、参数 digest、expected state version 与授权绑定。Intent 持久化和关联事件必须形成原子提交；权威存储不可写时 fail-before-effect。

- 来源：`docs/standards/intent-effect-idempotency.md` §2、§3、§6
- Tier：2
- 快照 hash：`9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c`

### FACT-COS-013 — OUTCOME_UNKNOWN 先对账

dispatch 后超时、断连或缺少 receipt 表示“可能已执行”，进入 `OUTCOME_UNKNOWN`。不允许直接到 `VERIFIED/COMMITTED`，也不能换新幂等键盲目重试。Reconcile 绑定原幂等键，结果为 executed、not_executed 或 still_unknown。

- 来源：`specs/transitions/effect.transitions.json` `EXECUTING -> OUTCOME_UNKNOWN` 及 `OUTCOME_UNKNOWN -> RECONCILED`；`docs/standards/intent-effect-idempotency.md` §4
- Tier：1 + 2
- hashes：effect table `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e`；standard `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c`；negative vector `effect-unknown-outcome.json` `6364e3eadce30c7918b83087ae8bf6a7779ce7d945508ab5e1d8f5e3da69c512`

### FACT-COS-014 — quarantine 与 compensation

still unknown 不能回到普通成功路径；只能进入 quarantine，或启动 separately authorized compensation。补偿是新的 governed Effect，必须拥有 `compensation_intent` 和 independent authorization；不能继承原 capability。

- 来源：`specs/transitions/effect.transitions.json` `RECONCILED -> COMPENSATING|QUARANTINED`；`docs/standards/intent-effect-idempotency.md` §4
- Tier：1 + 2
- hashes：effect table `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e`；standard `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c`

## 9. Verification 与 Acceptance

### FACT-COS-015 — Verification 独立且可过期

Verification 绑定 subject、criteria、verifier version 与 fixed post-state；状态可为 `PASSED/FAILED/INCONCLUSIVE`，且 `PASSED` 后仍可因后态变化、证据失效或 verifier 撤销转为 `EXPIRED`。

- 来源：`specs/transitions/verification.transitions.json`
- Tier：1
- hash：`4975c774094ae4f48cd4217497bde27cb71005be41bfd2a1df5ea21aa65e0ade`

### FACT-COS-016 — Acceptance 单独推进完成

远端 completed、receipt、工具 exit code 或模型自述都不是 acceptance。Task acceptance authority 消费当前 Verification 证据后决定完成。

- 来源：`docs/standards/task-loop-verification.md` §5；`conformance/vectors/remote-completed-not-acceptance.json`；`conformance/vectors/intent-acceptance-007.json`
- Tier：2 + Tier 1 normative-test（向量仍 not-run）
- 限制：当前 schema 目录没有完整 `AcceptanceDecision` schema；不要把行为规则写成已闭合机器形状。
- hashes：standard `5075759a0ce6940707bf47b65d7b930819b841978996307620345f8c93996a42`；remote-completed vector `b7d8df6d38111452d868520972306578e57c051de3e8e6f885fb23f8edb320b1`；intent-acceptance vector `ad859a72d8aa1e2670be9e75290c0e7a33edaed09cca79cf10c400bcc79c3c6c`

## 10. 当前四类状态与计数

### FACT-COS-017 — b626e88 的 M1 工程状态

| 状态类别 | 快照事实 | 不代表 |
|---|---:|---|
| 规范已登记 specified | 273 REQ；55 error codes；56 schema；5 执行生命周期表 | 实现存在 |
| 实现已提供 | 0 REQ；Lane-CTR contract code 已存在，但 matrix 尚无 REQ 级实现声明 | 行为通过或 REQ 已实现 |
| 测试已执行 | 0 | Profile 符合 |
| Profile 已符合 | 0 | — |
| 向量报告状态 | 76 / 76 `not-run` | 失败或通过 |

M1 为 **in-progress**。Lane-CTR 契约批已经交付：双语言 schema contract tests 证明 56 份 schema 可编译、两份 F-003 legacy 负例实例被拒，codegen、registered bundle digest 与 projection/golden 合同代码已落地。这里的“合同测试已运行”不是“conformance vector 行为已执行”：两份新向量 `GOBJ-LEGACY-METADATA-001` 与 `GOBJ-LEGACY-STRONGREF-001` 和其余向量一样仍为 `not-run`；F-003 唯一剩余 gate 是 Lane-CFR runner 的真实执行。

本次只读复核还确认：56 份 schema 全部具有 `$id` 且值严格等于各自文件名；绝对 URL `$id` 为 0。

- 来源：`docs/plan/PROGRESS.md` “里程碑状态”“REQ 覆盖计数”“向量分层计数”；`docs/checkpoints/20260720-lane-ctr-handoff.md` §1–§3；本次只读复核（56 schema、76 vector、0 absolute `$id`、0 non-filename `$id`）
- Tier：4 status evidence
- registry hashes：requirements `26d514db49b37df09312f7faec5367048d4af1ec3f179320d54cd10f61cb82d9`；errors `b0499ef3f14e4f3d071bbd5b3f445e1b7cab17894a7e815c21135d1c5d22716a`
- status hashes：PROGRESS `29386c6e5ad4301fcfe5e0f05ef24b6072dba40c405b64ec7354d865b476cb00`；Lane-CTR handoff `d4b61f8c5fb3725c10f9772bb576e0ca822f078b7a49ffca51a39244138d0957`
- new vector hashes：legacy metadata `7588782abd50a1a3f7e51026326ed810cef9cf589c9eff32b2646ba0c4d9fa79`；legacy strongRef `de04ad8b9f6d983bc5f97f7da30447c9b6d61a9c3b2fb1b1954153abcf9f94f7`

## 11. 必须保留的差异与开放项

### FACT-COS-018 — 来源不一致，不在博客内修复

1. 白皮书文件头为 v1.0.2；根 `README.md` 仍称 v1.0.1。
2. `conformance/README.md` §Running 字面写“仓库当前没有 conformance runner”；当前代码与 PROGRESS 更精确的状态是 enumerate-only skeleton，能枚举并报告 76 个 `not-run`，没有行为执行能力。根 README 仍列 74 向量与 M0 状态，findings ledger 的 IMP-17 摘要行也仍写 74（同一台账的 F-003 与 PROGRESS 已写 76）；这些旧行不能作为当前计数来源。
3. F-003 仍标记 partially-closed，但合同层义务已经完成；唯一剩余 gate 是 Lane-CFR runner 真实执行两份负例。F-001 是证据缺口；F-011/F-014/F-023/F-017 open；F-015 partially-closed。
4. D-001、D-006、D-011 已由 M1 Lane-CTR 闭合；D-004 仍 open。当前不存在绝对 URL `$id` 或缺失/非文件名 `$id`。
5. ContextRequest prose 与 shape 存在上述绑定差异。
6. 没有完整 OperationDescriptor schema；没有 AcceptanceDecision schema。
7. Core 与 Context standard 有两套九阶段命名。

- 来源：`README.md`；`CognitiveOS-Architecture.md` 文件头；`conformance/README.md` §Running；`docs/traceability/findings-ledger.md` F/D 表；schema 文件枚举
- Tier：1/3/4 discrepancy record
- hashes：
  - root README `22a4b6d9c1da1d4ea4308faafe82c6615fd69ccb476fba1ad24c2387d06133f1`
  - conformance README `a36b8e4e2d47384fa2f28da50f432e53f1f3b87e6ee72924eab1d1254ba17d19`
  - findings ledger `419b23c6b4c855a7683a81106ee6fd5f42e2b623ba1c7feec78b1a50d9b38066`
  - Lane-CTR handoff `d4b61f8c5fb3725c10f9772bb576e0ca822f078b7a49ffca51a39244138d0957`

## 12. 公开表述护栏

### 可用

- “当前 Draft 机器合同登记了 273 条 requirement。”
- “五个执行生命周期由独立 transition table 定义。”
- “Core 要求概率组件只产生 candidate/proposal；authority 决定授权、迁移与提交。”
- “ContextView 是非 authority 投影。”
- “Effect transition table 不允许从 `OUTCOME_UNKNOWN` 直接提交。”
- “Task 完成需要当前 Verification 和 acceptance authority 决定。”
- “Lane-CTR 已交付合同层代码与测试；当前仍有 0 个 REQ 级实现声明、0 个行为执行向量和 0 个符合 Profile。”

每句都应带快照 commit/date 或可定位来源。

### 禁用

- “CognitiveOS 已实现/生产就绪/已证明安全。”
- “76 项测试已通过”或“全部向量已验证。”
- “任一 Profile 已符合。”
- “所有 sink 都具备 fencing。”
- “Console 已实现。”
- “Agent 性能、成功率、收入、用户量或时延已经提升。”
- 把研究 snapshot hash 称为 registered specification-set digest。

## 13. 双语术语表

| 中文 | English | 使用说明 |
|---|---|---|
| 权威主体/权威协议 | authority | 对特定状态域写入、仲裁或验收负责 |
| 候选 | candidate | 概率组件可产出；不是决定 |
| 提议 | proposal | 待确定性门禁检查 |
| 操作描述 | OperationDescriptor | 能做什么；不授予权限 |
| 授权能力 | AuthorizationCapability | 谁在何边界内可做什么 |
| 上下文视图 | ContextView | Activity 绑定的非 authority 工作投影 |
| 意图记录 | Intent | dispatch 前持久化的受治理动作意图 |
| 效果 | Effect | 外部或受治理改变的生命周期记录 |
| 未知结果 | OUTCOME_UNKNOWN | 可能已经执行，必须先对账 |
| 对账 | Reconcile / reconciliation | 查询并闭合外部执行事实 |
| 隔离 | quarantine | 禁止正常提交/调度的安全处置 |
| 补偿 | compensation | 新的、独立授权的 governed Effect |
| 验证 | Verification | 对固定后态与 criteria 的证据判断 |
| 验收 | Acceptance | authority 推进 Task 完成的决定 |
| 栅栏令牌/代际 | fencing token / epoch | 拒绝旧 writer |
| 幂等键 | idempotency key | 同一逻辑尝试链稳定复用 |

## 14. 五张图的脚本、图注与文本替代

### 图 1 — 总体架构（informative）

- 脚本：纵向放七层；第 7–6 层归体验平面，第 5–4 层归控制平面，第 3–1 层归执行与数据平面；认知微内核位于第 4 层；实时安全内核旁路连接第 3/1 层；Context 作为横向虚线穿过三平面。
- 图注：双内核、三平面、七层与横切 Context 是责任解释视图，不替代机器合同。
- alt/long text：七层从宿主世界向上连接 Agent；Context 横穿三平面；认知微内核管理确定性门禁，实时安全内核独立仲裁执行器。
- 来源：`CognitiveOS-Architecture.md` §4.1–§4.5。

### 图 2 — 概率/确定性边界

- 脚本：左侧 LLM/retriever/ranker 只输出 candidate/proposal；中央粗线标 authority boundary；右侧 schema/auth/CAS/budget/idempotency/fencing/transition/commit；只允许候选向右进入 gate，不允许 decision 反向伪造。
- 图注：开放式语义搜索可以提议；共享事实只能由确定性 authority 路径改变。
- alt/long text：概率组件的输出在边界前停止，确定性 gate 检查后才可能授权、执行状态迁移或提交。
- 来源：Core §2、§15.5。

### 图 3 — Governed Flow Thread

- 脚本：Context → Proposal → Persisted Intent → Authorization → Effect → Reconcile → Verification → Acceptance；Effect 后分支 `OUTCOME_UNKNOWN`，只能回到 Reconcile；still unknown 进入 independently authorized compensation 或 quarantine。
- 图注：这是一条静态语义链，不是实时任务进度。
- alt/long text：提议必须先持久 Intent 并获得授权；未知 Effect 先对账；验证不等于验收；补偿需单独授权。
- 来源：Intent/Effect standard §2–§5；task/verification transitions。

### 图 4 — 五个正交生命周期

- 脚本：五条独立泳道展示完整 state name；只用细线指示 evidence 引用，不画成一个串行总状态机。
- 图注：AgentExecution、Task、Loop、Effect、Verification 可以同时处在不同状态。
- alt/long text：五个独立域分别有自己的 authority、初态、终态和恢复路径；Task 的完成不会自动终止其他域。
- 来源：state-domains registry 与五份 transition JSON。

### 图 5 — Context 九阶段与词汇差异

- 脚本：主轴使用 Context standard 的九阶段；下方对齐显示 Core 的九个词；在 transform/verify 与 loss declaration 等非一一对应处使用断开的连线和“vocabulary differs”注记。
- 图注：标准顺序是实现合同；Core 提供另一组概念词汇，二者不静默合并。
- alt/long text：治理预过滤先于候选检索，逐对象授权重验先于语义排序；两套九阶段名称并列展示。
- 来源：Context standard §2；Core §6.3。

## 15. 研究快照 hash 索引

| Source path | SHA-256 research snapshot hash |
|---|---|
| `CognitiveOS-Architecture.md` | `e71b5dd83549ec7b7eb278e609d22fcea503279ef5918e3aceab10a62dd54b0c` |
| `specs/core/README.md` | `532645e076f27905efab65776fc573daf5cc85bf38bee25bf87b993465a1ce11` |
| `docs/standards/context-resolution-and-cache.md` | `e973a36801a5e66393a1942186a5b1a82f80e200393e331d24f23fb2f95683a8` |
| `docs/standards/intent-effect-idempotency.md` | `9172948c9bcc77f798cb90d2b9284312aeebb8b773ff446575a19d546c751f5c` |
| `docs/standards/task-loop-verification.md` | `5075759a0ce6940707bf47b65d7b930819b841978996307620345f8c93996a42` |
| `docs/standards/normative-source-and-versioning.md` | `c0d0dd4b5ef6b97da1c0ce82d2947905774cce9e9814b7d057b7f008a44a7e6b` |
| `specs/registry/requirements.yaml` | `26d514db49b37df09312f7faec5367048d4af1ec3f179320d54cd10f61cb82d9` |
| `specs/registry/errors.yaml` | `b0499ef3f14e4f3d071bbd5b3f445e1b7cab17894a7e815c21135d1c5d22716a` |
| `specs/registry/state-domains.yaml` | `17ee88cabe13b0c539559e507e36c417455eeb6204a1907848f9c17757296d40` |
| `specs/transitions/agent-execution.transitions.json` | `d29c2926ee9a2ceae945201df25e712ddfbc65000323b24af70466e81652f616` |
| `specs/transitions/task.transitions.json` | `770484aeb2b04f2afd76e07512fb01b5b9760517ce5b879bb58714ed1b375f17` |
| `specs/transitions/loop.transitions.json` | `db836e190920cc008804b8a195f3c01cf2cf696039003914464badd0539509ef` |
| `specs/transitions/effect.transitions.json` | `e8aa2dfebac6dbf179814e40067034270e1b9fde9964bb631aab79af2f3b960e` |
| `specs/transitions/verification.transitions.json` | `4975c774094ae4f48cd4217497bde27cb71005be41bfd2a1df5ea21aa65e0ade` |
| `specs/schemas/context-request.schema.json` | `080747d0f1510cab9d94c1cca30dae43bda9f51f2eb2a2a75ac16cad9062b104` |
| `conformance/vectors/remote-completed-not-acceptance.json` | `b7d8df6d38111452d868520972306578e57c051de3e8e6f885fb23f8edb320b1` |
| `conformance/vectors/intent-acceptance-007.json` | `ad859a72d8aa1e2670be9e75290c0e7a33edaed09cca79cf10c400bcc79c3c6c` |
| `conformance/vectors/effect-state-closure-008.json` | `e74aa5bf26ddc8b900a0c0b213b522bb40f4fbe8090d1e295cec4d7d18d82b52` |
| `conformance/vectors/effect-unknown-outcome.json` | `6364e3eadce30c7918b83087ae8bf6a7779ce7d945508ab5e1d8f5e3da69c512` |
| `conformance/vectors/governed-object-legacy-metadata-001.json` | `7588782abd50a1a3f7e51026326ed810cef9cf589c9eff32b2646ba0c4d9fa79` |
| `conformance/vectors/governed-object-legacy-strongref-001.json` | `de04ad8b9f6d983bc5f97f7da30447c9b6d61a9c3b2fb1b1954153abcf9f94f7` |
| `conformance/README.md` | `a36b8e4e2d47384fa2f28da50f432e53f1f3b87e6ee72924eab1d1254ba17d19` |
| `docs/plan/PROGRESS.md` | `29386c6e5ad4301fcfe5e0f05ef24b6072dba40c405b64ec7354d865b476cb00` |
| `docs/checkpoints/20260720-lane-ctr-handoff.md` | `d4b61f8c5fb3725c10f9772bb576e0ca822f078b7a49ffca51a39244138d0957` |
| `docs/traceability/findings-ledger.md` | `419b23c6b4c855a7683a81106ee6fd5f42e2b623ba1c7feec78b1a50d9b38066` |
| `README.md` | `22a4b6d9c1da1d4ea4308faafe82c6615fd69ccb476fba1ad24c2387d06133f1` |

末次作者检查：每个公开事实必须能回到一个 `FACT-COS-*` 条目；后续 commit 变化时，不得沿用本快照数字或 hash。
