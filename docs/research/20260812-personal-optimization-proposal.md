# CognitiveOS Personal 架构优化决策方案（实现复核重构版）

- Status: informative review proposal（非规范源、非计划源、非任务台账）
- Date: 2026-08-12
- Review baseline: active branch
  `personal/P9-T04-comprehensive-performance-campaign` at
  `4b072c011f551b7bf4e5d67c09099876308e05c9`
- Current fact owner: `docs/plan/PROGRESS.md` `Current snapshot`
- Architecture/axiom owners:
  `docs/governance/AXIOMS.md`、accepted ADRs、`core/specs/` 与适用 standards
- Scope: 结合当前架构与实现，从 OS、Agent 基座、管理、性能、完成任务能力和协作
  六个视角，给出可验证、可裁剪、不过度扩张的优化决策框架

> 本文不创建任务、REQ、Gate、lease 或产品声明。文中的 `INV-*` 是投资主题，不是
> 平行 backlog。正式采纳必须映射到
> `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` 的既有任务，或先由 owner 登记新的正式
> Personal 任务；公共合同变化必须走 Lane-CTR。若本文与规范源冲突，以规范源为准。

---

## 0. 结论

CognitiveOS Personal 的核心架构不需要推倒重来。daemon-only authority、
candidate/authority 分离、Intent/Effect persist-before-dispatch、fencing、独立验证、
SecretStore 和证据分级都已在承重代码中落地；SQLite 单写者和五张机器状态表也仍是
适合 owner-local、single-principal 产品的正确基线。

需要重构的是**优化顺序和若干实现断层**，不是公理层：

1. 当前第一优先级是按 ADR-0051 完成 P9-T04，而不是在 campaign 中夹带 streaming、
   新 Tool 或新 adapter；
2. 完成任务能力的首要缺口不是“没有 Tool 抽象”，而是**六个已登记且投影为 enabled
   的 native operation family 只有两个有真实 executor**；
3. AKP 不能降格为“词汇表”。ADR-0043 与 P8-T02 已决定 AKP 是唯一 adapter protocol；
   应补真实 envelope transport 证据，而不是为适配现状改弱设计；
4. 非流式 Provider proxy 是已接受的安全边界。是否改成 streaming 应由 P9-T04 的
   route 数据和 Pi 交互需求触发，不应预先认定；
5. verification report 跨请求复用会触碰 request、fixed post-state、fencing 和 freshness
   绑定，不是 implementation-only 优化，当前方案不建议做；
6. P6 的进入条件和 B11 的合法 NO-GO 已在正式计划中存在，不应再建立一套更严格的
   “全局硬闸门”，也不得把 acceptance/promotion dependency 变成 implementation mutex。

### 六视角总览

| 视角 | 当前可信结论 | 真正缺口 | 优化方向 |
|---|---|---|---|
| OS | owner-local 单节点资源治理、持久状态、fencing 已成立 | 多 Task 争用、公平性和全局预算尚无 campaign 证据；这不是 Linux 1.0 缺陷 | 先完成既有 T7 并发场景；仅在 P6 需要时设计仲裁 |
| Agent 基座 | Pi 路径、sidecar 生命周期、adapter manifest、harness/context 均已实现 | AKP envelope 未进入 Personal adapter 数据路径；非 Pi 仅 fixture 资格 | 先补 AKP 真实私有 transport，再做第二个活体 adapter |
| 管理 | status/doctor、resource/task watch、backup/restore、release manifest 已有 | RC、Windows、统一只读概览尚未完成 | 优先 P7-T06；界面只聚合既有 read model |
| 性能 | D01–D07 已完成；L0–L2 已执行并只支持 hypothesis | L3–L5 未执行；TTFT/cost/benefit 仍不可声明 | 完成当前 campaign；按结果触发优化 |
| 完成任务 | authority loop 完整；native Tool 合同面宽 | executor 面仅 WorkspaceRead/ProcessCheck | 先修“投影诚实性”，再补已登记 family 的 executor |
| 协作 | 人工 preview/审批、Task watch 和多 Agent 设计已存在 | P6 runtime 未实现；单 Agent benefit 尚未完成 | 使用正式 P6 顺序和 B11，不建立第二套 Gate |

---

## 1. 本版对原方案的关键纠正

| 原方案观点 | 复核结果 | 本版处理 |
|---|---|---|
| “P9-T04 是一切扩张的唯一硬闸门” | 与 Operating Model 的 typed dependency 规则冲突；acceptance/promotion 不能成为 implementation mutex | 改为**promotion/default-enable 决策点**；实现依赖满足的隔离工作仍可并行 |
| 推荐把 AKP 降格为“合同词汇 + profile enum” | 与 accepted ADR-0043、P8-T02 和 A6 冲突 | 删除该路线；只保留“实现补齐到合同” |
| native Tool 只有 WorkspaceRead/ProcessCheck | 只对 executor 成立。Registry 已登记 WorkspaceRead/Search/Write/Patch、ProcessCheck、HttpFetchReadOnly 六类，并全部标 `Enabled` | 将问题改写为 registry/projection 与 executor 的能力断层 |
| 新建 10–20 个 dogfood task corpus | P9-T04 已预注册 T1–T8，L5 已冻结 W1/W2 和 A/B/C/D 统计合同 | 不创建平行 corpus；复用既有 denominator |
| 同 fixed-post-state digest 可复用 verification report | 当前报告唯一性与 continuation 校验绑定 request、verifier/version、fencing epoch 和 current subject version；跨请求复用会改变 A4 语义 | 删除；只允许同一请求的幂等去重，任何更改先做语义 ADR/合同评审 |
| 立即建立新的阶段 SLO 表 | 执行计划 §8.4 已有 W1/W2 governance overhead 门槛；Operating Model 要求阈值先校准、owner 预注册后才能阻塞 | 使用既有阈值；L0–L2 只做诊断，不再造第二套 SLO |
| streaming 是当前高优先级实现 | 当前 proxy 明确 non-streaming；ADR-0051 要求无 streaming timestamp 时 TTFT=`not_available`，并不要求本任务实现 streaming | 改为 P9 结束后的 evidence-triggered 决策 |
| 白皮书历史化、合并既有 Gate ADR、减少 handoff | P8-T01 刚完成文档收敛；Operating Model 已要求整任务只保留最终 handoff。retroactive 合并 accepted ADR 会破坏决策历史 | 删除重写/合并建议；仅允许导航和陈旧标记的 corrective 修正 |
| 公理适用面可收窄 | 公理不能由实现方案收窄；只有产品 scope 可收窄。A3 的窄路径自由也必须由 formal acceptance 与 threat model 允许 | 改为“收窄产品声明，不收窄公理” |
| 当前前沿是 P9-T04/D02 | 已过期：D01–D07 均完成，L0–L2 已执行，D08 是当前 Slice | 全文以 Current snapshot D08 为准 |

---

## 2. 不变边界与优化原则

### 2.1 保持不变

1. A1–A8 不因性能、易用性或生态接入而放松；
2. Rust daemon 仍是唯一 authority writer；
3. 外部或不可逆 mutation 仍须 Intent/Effect persist-before-dispatch、幂等和 fencing
   对账；
4. Task completion 仍须独立验证、closed/reconciled Effects 和 non-stale epoch；
5. Secret 仍只进批准的 SecretStore / non-logging input path；
6. SQLite WAL 单写者与同步 authority 路径保持，除非新的阶段数据推翻 P9-T01 的
   `conservative-no-migration` 结论；
7. adapter 不开默认公网 listener，多 Agent 默认关闭；
8. `not-run` 永不变成 pass；local/fixture/ordinary CI 永不升级为 Gate/release/Profile。

### 2.2 优化原则

1. **先完成当前 vertical path，再抽象。** 不先建通用 framework；
2. **先修声明诚实性，再扩能力。** Registry、projection、executor 三者必须可区分；
3. **测量触发结构变更。** streaming、async、verification 并行化、公平调度均需证据；
4. **复用正式 denominator。** 不在 P9-T04 之外建立平行性能语料；
5. **不向下改合同。** 实现不完整时修实现或明确 non-claim，不改弱 AKP/negative；
6. **最小权限不是最少能力。** 新能力可增加，但必须经过现有 authority path；
7. **既有任务优先。** P7-T05/T06/T07、P6-T01..T04 已有正式落点，不另立重复任务。

---

## 3. 六视角复审

### 3.1 OS 视角

#### 当前事实

- 六族资源、统一 read/action projection、五个独立状态机、SQLite WAL、Event、
  Checkpoint/Resume、Intent/Effect 和 recovery 顺序均已实现；
- scheduler lease 是按明确 `task_ref + contract_epoch` 获取的原子 CAS。当前 repository
  没有一个跨所有 eligible Task 的 global picker，也没有 priority/fairness/starvation
  policy；`list_recoverable ORDER BY task_ref` 只是恢复枚举，不是调度政策；
- budget 已实现 deterministic hard admission 和 debit，但不是跨 Task 的份额仲裁器；
- 这些能力满足当前 owner-local/single-principal/Linux 1.0 边界。不能因为没有
  enterprise fairness 就判定 OS 不成立。

#### 缺口与处理

1. P9-T04 的 T7 已预注册 1/4/8/16 并发、17 in-flight 与 33 connection overload，
   应作为当前 OS 争用证据，不再新增另一套并发 campaign；
2. 若 T7 发现 authority corruption、无界尾延迟或 starvation，再形成 scheduler
   优化决策；
3. global budget pool、Task priority、fair-share 属于 P6-T01 的 product-semantic
   policy，不应在 P6 之前先造通用调度框架。

#### 成功判据

- T7 denominator 完整、超限有界、无 authority corruption；
- 无证据时保持现有简单 scheduler，而不是为“更像 OS”增加策略。

### 3.2 Agent 基座视角

#### 当前事实

- Pi 是 Linux 1.0 唯一活体合格 adapter；
- P8-T03 完成的是 Codex fixture identity/lifecycle/non-claim matrix，不是外部 Codex
  进程的真实 I/O 资格；
- ADR-0043 和 `agent-adapter-contract.md` 明确规定 AKP envelope 是唯一 adaptation
  protocol；
- 当前 `agent_adapter_manifest.rs` 只验证 `AkpHttpJsonSse` profile、声明 digest、
  candidate-only 和 lifecycle；Personal sidecar 路径没有调用
  `cognitive_akp::parse_request/result_ok/WatchLog`。真实 envelope 消费只在 legacy
  `personal/apps/kernel-server/src/main.rs` 和 conformance 测试中可见。

#### 缺口与处理

1. **先补 AKP transport conformance vertical slice**：一个 daemon-created private
   stdio/socketpair fixture 发送真实 request/result/stream envelope，覆盖 version、
   schema digest、payload digest、unknown critical extension、cursor stale 和
   authority-shaped payload negative；不得开 public listener；
2. **再做第二个活体 adapter**：复用 P8-T03 的身份与 lifecycle，但实际拉起一个固定
   版本 CLI agent，最小只声明完成一个 Task 所需能力，不要求一次消费六族资源；
3. 第二个活体 adapter 成功后才抽取 reusable qualification kit。先造 kit 再找第二个
   消费者会重复 framework-first 问题；
4. streaming 是 Provider transport 决策，不与 AKP contract 补齐混为一项。

#### 成功判据

- Personal product path 中存在可追踪的真实 AKP envelope exchange；
- 非 Pi agent 只能产 candidate/observation，且不能继承 Pi/B09 证据；
- 第二个 adapter 不扩大 Linux 1.0、Gate 或 Profile 声明。

### 3.3 管理视角

#### 当前事实

- `cognitive status/doctor/resource get|watch/task watch` 已存在；
- `/personal/doctor` 已聚合 six-resource、vault、sidecar/process/effect/migration
  诊断；P7-T02 已实现 backup/restore 与 transactional update/rollback/uninstall；
- P7-T01 已有 release manifest、SBOM 与 attestation；
- 尚未完成 P7-T06 RC、P7-T07 Windows/B01-W；P7-T05 Web UI 是非阻塞项，且客户端
  实现受外部 `cognitiveos-clients` 仓及其 readiness gate 管理，本仓 Console 仍是 stub。

#### 优化方向

1. **P7-T06 优先于新增管理框架**：把 clean VM、升级/回滚、支持矩阵和诚实 release
   claim 做完整；
2. 可选增加 `cognitive overview`，但第一版只能组合既有 status/doctor/resource/task
   read model。若需要新增 budget/Effect/Verification public projection，则先做
   product-semantic / Lane-CTR 评审；
3. P7-T05 只消费 daemon read model，不在本仓重建客户端树；
4. 不预建通用 OTLP/JSONL export。P9-T04 已有 redacted measurement envelope；只有
   出现真实产品 consumer 后再设计长期 telemetry contract；
5. 文档减重只做导航/index 和陈旧事实修正，不历史化现行白皮书、不改写 accepted ADR
   历史、不另建公理摘要。

#### 成功判据

- RC claim 与实际支持矩阵一致；
- 管理界面不新增 authority writer；
- 新读模型不得泄露 secret、prompt、response 或 authority-store 原文。

### 3.4 性能视角

#### 当前事实

- P9-T04 D01–D07 已完成；
- D06 已在 exact native Linux 执行一次 L0–L2：L0/L1/L2 retained sample 数为
  1/200/52，七个 safety counter 为 0，cleanup 完整；
- 该报告的 claim ceiling 仍是 `hypothesis`，`benefit_claimed=false`，
  `verifier=not_reviewed`；
- L3–L5 均为 `not_run`；D08 正在构建 T1–T8 L4 governed-Task scenario harness；
- execution plan §8.4 已登记 W1/W2 latency/cost/cache-preservation threshold。

#### 优化方向

1. 当前只完成 P9-T04，不在任务内另做 streaming、async runtime 或 verification
   semantics 重构；
2. D08 先做 T1 read-only admission scenario，再按预注册顺序构造 T3、T2、T5、
   T6/T7/soak；能力缺失必须记 `not_run/not_available`，不得在 measurement-only
   runner 内偷偷补产品能力；
3. L3/L4 execution 仍遵守 B01 start gate 与 graphical hidden-input Provider import；
   L5 仍需要 owner 对 A-arm baseline 作决定；
4. 使用既有 P7-T04 floor 和 §8.4 threshold，不新增第二套 SLO。

#### 数据触发的后续决策

| 观测 | 后续动作 | 当前禁止项 |
|---|---|---|
| Provider complete-response dominates，且用户旅程确需低 TTFT | 新 ADR 评估 bounded streaming | 不把非流式时延伪装成 TTFT |
| verification stage 占治理 p95/p99 主导 | 先 profile verifier executor 与调度等待，再评估并行执行 | 不复用跨请求 report，不跳过 freshness |
| store/open/lock 主导且超过既有门槛 | 用新数据复审 P9-T01/P9-T03 决策 | 不凭直觉迁移 async |
| Context token/cost 主导 | 用 T4 比较 full/stable/changed，调整既有 compaction/budget policy | 不跳过 body reauthorization |
| T7 出现公平性或 starvation 问题 | 在 P6-T01 定义最小 scheduler policy | 不先建通用 priority language |

### 3.5 完成任务能力视角

#### 当前事实

- Task admit → Context → candidate/WIA → Intent/Effect → reconciliation →
  independent verification → continuation 的 authority loop 已实现；
- `BUILTIN_TOOL_CATALOG` 已登记六类 operation，并在 Resource projection 中逐项返回
  `availability: Enabled`；
- 实际 Tool executor 只实现：
  - `NativeWorkspaceReadExecutor`；
  - `NativeProcessCheckExecutor`；
- WorkspaceSearch/Write/Patch、HttpFetchReadOnly 只有 descriptor 与 pre-validator；
  staging 到 executor 会返回 unsupported family；
- `ProcessRun`、Git mutation 并不存在于合同或 registry，不能当作简单 implementation
  补丁。

这形成一个比“工具少”更具体的问题：**用户可见 registry availability 与 daemon
execution readiness 没有被清楚区分。**

#### 投资顺序

**INV-TASK-1：投影诚实性（最小 corrective slice）**

- 在不改变 immutable descriptor digest 的前提下，让 private Tool projection 区分
  `registered/enabled` 与 `execution_ready`；
- composition root 只为已装配 executor 的 family 报 `execution_ready=true`；
- 若这要求改变 public DTO，则走 Lane-CTR；不能直接把未实现 family 改成 enabled
  的另一种含义。

**INV-TASK-2：完成已登记 family 的 executor parity**

| 顺序 | Vertical slice | 复用面 | 必须覆盖 |
|---|---|---|---|
| 1 | WorkspaceSearch | Workspace path validator、bounded/redacted output、现有 read executor pattern | path escape、query bounds、duplicate key、stale fence |
| 2 | WorkspaceWrite/Patch | Intent/Effect、Artifact CAS、workspace validator | expected preimage、atomic publish、partial write、symlink/path race、duplicate dispatch、OUTCOME_UNKNOWN/reconcile |
| 3 | HttpFetchReadOnly | HTTPS validator、bounded transport policy | credential-in-URL、redirect、body/output bound、timeout、DNS/target policy |

每个 executor 都必须沿用 daemon authority、descriptor digest、idempotency、fencing 和
recovery；不得因为是“本地工具”绕过现有 Effect 语义。A3 允许只读窄路径，但必须由
正式 acceptance 与 threat model 明确批准，不能由性能提案自行推导。

WorkspaceSearch 直接服务 T1；WorkspaceWrite/Patch 与下节的 deterministic test
执行决策共同服务 T2。HttpFetchReadOnly 不应先于这条最小有用任务闭环。

**INV-TASK-3：是否需要进程执行**

- T2 软件修复需要 deterministic test oracle，但当前只有 ProcessCheck；
- owner 需在“新增 bounded ProcessRun native family”与“复用已资格化 MCP Tool”
  之间做 product-semantic 决策；
- 新 native family 需要合同/registry/风险/命令 allowlist、cwd/env/timeout/output、
  child process cleanup 和 fault/reconcile 设计；不能用 `ProcessCheck` 偷换语义；
- 在该决策前，不建议新增 Git 专用 operation；Git 可先由未来受控 process/MCP
  能力承载，再以真实需求决定是否提升为 typed native Tool。

#### 成功判据与 kill criteria

- T2 若因 capability 缺失而 `not_run`，这是注册 INV-TASK-2/3 的直接证据；
- 若产品定位明确只做只读分析，则停止 mutation/process 投资，保持诚实 projection；
- Tool success 仍不等于 Task completion。

### 3.6 协作视角

#### 当前事实

- 人工 admission preview、management proposal/approval、自授权拒绝、Task watch 已有；
- P6-T01..T04 已定义 policy/scheduler → mailbox/findings → orchestration → B11；
- 正式 Phase 6 的 implementation start 已写为“单 Agent benchmark 与明确并行假设”；
- B11 允许 GO 或合法 NO-GO/default-off。

#### 优化方向

1. P9-T04 完成后，用实际单 Agent bottleneck 写出“为什么需要并行 agent”的一条明确
   hypothesis；若 bottleneck 是 Provider latency、工具缺失或 verifier 等待，多 Agent
   不会自动解决它；
2. P6-T01 内定义最小 budget/lease/isolation policy，不在其前建立循环依赖式的
   “先完成全局预算 ADR 才能开始 P6”；
3. P6-T02 的 mailbox/findings 保持 append-only candidate，消息永不成为 authority；
4. P6-T03 可先实验 Reviewer，但 Reviewer agent 只是 candidate producer，绝不等同于
   A4 的 independent verifier；最终 completion 仍走现有 verifier；
5. P6-T04 按 B11 评估质量/速度收益，NO-GO 不算失败，也不阻塞 RC。

---

## 4. 收敛后的投资主题

原方案的 24 个 `OPT-*` 被收敛为六个主题，避免形成平行 backlog：

| ID | 投资主题 | 优先级 | 正式落点/状态 |
|---|---|---:|---|
| INV-1 | 完成 ADR-0051 P9-T04（D08、L3–L5、报告、cleanup、verifier disposition） | 最高 | **现有 P9-T04；当前唯一 in-progress** |
| INV-2 | Tool projection 诚实性与已登记 family executor parity | 高（由 T1/T2 能力缺口直接触发） | 需 owner 登记新任务和非重叠 lease；可并行解除能力缺口，但不得夹带到 P9 branch/PR |
| INV-3 | AKP envelope 进入 Personal 私有 adapter transport；第二活体 adapter | 中 | 后续 adapter qualification；先 transport 后 live adapter |
| INV-4 | RC/升级/支持矩阵与最小只读管理概览 | 高/中 | P7-T06 优先；P7-T05/P7-T07 按正式计划 |
| INV-5 | streaming/verification/store/context 的数据触发优化 | 条件式 | P9-T04 final report 后分别决策，不预注册为必做 |
| INV-6 | Multi-Agent 实验与 B11 | 条件式 | P6-T01..T04；单 Agent benchmark + parallel hypothesis 后进入 |

### 4.1 不建立全局硬闸门

INV-1 是当前任务的完成优先级，不是所有未来实现的 mutex。满足
`implementation_requires` 的隔离工作可按 Operating Model 并行；只有默认启用、
release claim、收益 claim 和结构迁移必须等待对应 evidence/owner decision。

### 4.2 P9-T04 结束后的决策树

1. **若 governance non-inferiority 未满足：**
   - 保留 A1–A8；
   - 先定位 stage tax，优化实现；
   - 收窄产品支持 workload/claim，而不是收窄公理；
   - 暂不扩大 Multi-Agent 或新 adapter 默认面。
2. **若 T2/T5 因 Tool capability 缺失：**
   - 优先 INV-2；
   - campaign 如实保留 `not_run`，不得回填虚假样本。
3. **若性能满足但用户交互受 complete-response latency 限制：**
   - 启动 bounded streaming ADR；
   - 增加 cancellation、frame/total bound、redaction、single-choice 和 usage trailer
     负例。
4. **若单 Agent 表现良好且存在可归因 parallel bottleneck：**
   - 启动 P6-T01；
   - 否则记录 B11 NO-GO/default-off。
5. **P7-T06 独立推进：**
   - P7-T06 不依赖 P9 benefit 或 P6，可在其 `implementation_requires` 满足时推进；
   - P9 report 只约束可写入 RC 的 performance/benefit claim，不成为 RC implementation
     mutex；
   - P7-T05 UI 与 P7-T07 Windows 保持独立，不继承 Linux evidence。

### 4.3 统一北极星与投资上限

所有主题以 `verified task completion rate` 为北极星，同时报告：

- time-to-verified-completion p50/p95；
- 每个 verified Task 的人工干预次数/分钟、token、费用和 Tool calls；
- governance stage 占总时长比例；
- duplicate Effect、false completion、stale commit、secret exposure（目标均为 0）；
- recovery-to-reconciled；
- install/upgrade/rollback 成功率；
- adapter 资格耗时与核心特例数；
- Multi-Agent 等预算收益与协调/重复工作开销。

| 主题 | 初始投资上限 | Kill criterion |
|---|---|---|
| INV-2 Tool | 先闭合 T1/T2 所需 search → patch/write → deterministic test，不扩 Git 专用面 | 产品明确只读定位，或 mutation 无法可靠 query/reconcile |
| INV-3 Adapter | 一个 private AKP vertical slice + 一个固定版本活体 adapter spike | 需要公开 listener、放松 authority 或大量核心特例 |
| INV-5 性能 | 每次只改一个已测得 dominant stage，并做 before/after | 未达到预注册阈值或破坏任何 safety counter |
| INV-6 Multi-Agent | 一个 default-off、等预算、双 agent experiment | 无质量/速度收益，或协调/重复工作上升 |

---

## 5. 建议交付与验证矩阵

| 主题 | 最小垂直交付 | Supported validation | Evidence ceiling |
|---|---|---|---|
| INV-1 | D08 T1 authority-path harness，随后按预注册 T1–T8/L3–L5 | exact pushed revision、native campaign guest、required CI、independent review | final report 实际支持的 claim；不足即 non-claim |
| INV-2 | execution readiness projection + 一个真实 executor family | focused negatives、exact native Linux、required CI；mutation 加 crash/reconcile | implementation evidence；不自动更新 Gate |
| INV-3 | 一个真实 private AKP request/result/stream exchange | version/digest/critical extension/channel/authority-shape negatives | adapter implementation/qualification；不转移 Pi/B09 |
| INV-4 | clean VM upgrade/rollback + honest support matrix | P7-T06 campaign、artifact/SBOM/attestation 校验 | RC 范围内 claim |
| INV-5 | 一项由 stage data 触发的 before/after change | Operating Model performance ladder | calibration 或 preregistered threshold 允许的结论 |
| INV-6 | default-off 两 agent candidate experiment | isolation/budget/lease/mailbox negatives + B11 denominator | GO 或合法 NO-GO；无第二 authority |

---

## 6. 明确删除或延期的原提案

以下内容不再作为建议：

1. 把 AKP 降级为仅 profile enum；
2. 在 P9-T04 外新建 dogfood benchmark denominator；
3. 跨 verification request 复用 report；
4. 在没有数据时迁移 async runtime；
5. 在现有 registered Tool family 未完成前新增 Git 专用工具；
6. 立即建立通用 OTLP/export 子系统；
7. 历史化现行白皮书、retroactive 合并 accepted ADR；
8. 用新的治理清单把所有 P6 或其它实现完全锁死；
9. 以“缩小公理适用面”作为性能退路；
10. 在当前 P9-T04 branch/lease 中夹带本方案中的产品功能。

---

## 7. 风险与终止条件

| 风险 | 控制 | 终止/降级条件 |
|---|---|---|
| P9-T04 无法证明收益 | 保留完整 denominator 和 non-claim report | 停止 generalized benefit claim；收窄产品 workload，不改弱公理 |
| Tool projection 继续把非 executable family 显示为 enabled | 增加 execution readiness 事实 | 若无法兼容修正，至少在 UI/CLI 明示 registry-only |
| Workspace mutation 引入不可恢复副作用 | preimage/expected state、atomic publish、Effect/reconcile、focused crash negatives | 无法可靠 query/reconcile 时不启用 |
| streaming 泄露 secret/内容或无界缓存 | daemon-only credential、bounded frames、no logging、cancellation | 威胁模型/负例未闭合则保持 non-streaming |
| 第二 adapter 依赖外部 CLI 漂移 | 固定 package/version/digest、独立资格 | 只能 fixture 时不作生态/通用性 claim |
| Multi-Agent 只增加成本 | B11 paired benefit 与 default-off | 无质量/速度收益即 NO-GO |
| 管理面扩大 authority | 只读 projection、channel isolation | 需要新 writer 时必须另做 product-semantic/contract 决策 |

---

## 8. 正式采纳路径

1. 本文保持 informative，不更新 `PROGRESS.md` 或 task status；
2. INV-1 只由当前 P9-T04 task/lease 推进，本文不改变其 denominator；
3. INV-2/INV-3 需要 owner 先登记新的 Personal formal task 和 exact writable lease；
4. INV-4 使用既有 P7-T05/T06/T07，不建立重复任务；
5. INV-5 每项按 before/after evidence 单独决策；product-semantic 变化需 ADR；
6. INV-6 使用既有 P6-T01..T04 与 B11；
7. schema/public DTO/registered error/vector 变化一律走 Lane-CTR/CFR；
8. 未正式采纳的 `INV-*` 不能写入 current status，也不能成为其它任务的隐藏
   acceptance dependency。

该路线保持 CognitiveOS Personal 最有价值的部分——authority correctness、
recoverability 与 evidence honesty——同时把下一阶段投资收敛到三个可判定问题：

1. 当前 governed path 的真实成本与收益是什么；
2. 已登记能力中哪些必须补成真实 executor 才能完成目标任务；
3. 哪些扩展（streaming、第二 adapter、Multi-Agent）有数据支持，而不是仅因架构上
   “可以做”就进入实现。
