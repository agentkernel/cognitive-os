# 前沿对照评审：CognitiveOS 设计全面性 与 "运行环境感知" 创新点论证

> 日期：2026-07-26
> 分类：研究评审文档（documentation-only）。不是 REQ、schema、Gate、Profile 或任务状态变更；
> 若采纳第 5 节建议，须按台账 §6 结构变更清单走同批修订（docs/plan/plan.md / personal-trace.yaml 同步，无孤儿任务）。

## 0. 结论摘要

1. 从第一性原理、harness、context、loop 四个工程维度对照 2025–2026 学术与工业前沿，CognitiveOS 的规范层设计**全面且领先**：业界 2026 年才命名的实践（loop engineering、progressive disclosure、harness 与模型解耦）在本仓库均已是带 REQ 的一等公民。
2. 发现一个真实缺口：**运行环境感知（Environment Perception）目前是碎片化的**——能力快照（P1-T03）、Tool health（DEC-P-14/P5）、doctor（P7-T03）、`environment_digest`/`pinned_environment_versions`/`environment_fingerprint` 各自存在，但没有统一的、可查询、带新鲜度与出处的环境快照服务供 Agent 消费。
3. **建议采纳该创新点**，但定位为"对既有原语的统一与扩展"，不是新的平行子系统。学术证据（agent 缺乏环境好奇心、probing 修复世界模型）与工业证据（MCP gateway 按环境呈现工具、schema drift CI 固定、Nix/E2B 声明式环境）都支持：环境感知应由平台提供，而非每个 Agent 每个 loop 用 bash 重新探测。

---

## 1. 第一性原理维度

**前沿基线。** Karpathy 的 LLM OS 类比（模型=CPU、context window=内存、工具=外设）已成为主流心智模型；AIOS（COLM 2025）将 scheduling、context、memory、storage、tool、access 六类服务内核化，实测 2.1× 提速；Quine 进一步把 agent 做成原生 POSIX 进程；Model-Native Computing Architecture 从计算机体系结构角度重构系统栈。

**CognitiveOS 对照。** AIOS 的六类内核服务本仓库全部覆盖，且多出 AIOS 缺失的三层：(a) **治理与证据学科**——概率组件只产 candidate、确定性 admission 只收窄（REQ-DISC-ADMIT-001、REQ-INTENT-ADMISSION-001）；(b) **权威唯一性**——单 writer daemon、Shell 非权威（PERS-PR-004/005）；(c) **非声明纪律**——done ≠ Gate ≠ Profile。这三层正是学术原型（AIOS/AutoGen 类）到可信产品之间的差距，属于本项目的差异化优势。

**评审结论：** 第一性原理层无结构缺陷。"LLM 是新 CPU"的推论——**OS 必须替 CPU 管理它看不见的外部世界**——恰好指向第 5 节的缺口。

## 2. Harness Engineering 维度

**前沿基线。** Anthropic 2026 年公开的长程 harness 实践：initializer agent 建立结构化环境 + coding agent 跨 context window 增量推进；关键教训是"**harness 随模型升级重新定价**——每次升级要重新测试每个组件是否还值得存在"。探索性研究（arXiv 2602.14690）将 harness 要素归纳为工具面、权限、记忆、可观测与编排。

**CognitiveOS 对照。** 强匹配：Pi 仅为可替换 Shell、Provider 是配置而非硬编码（DEC-P-06）、能力用主动探针而非静态声明（PERS-PR-003）——模型/Shell 升级只触碰 adapter，harness 主体（authority、Intent/Effect、verifier）不重定价，这正是业界教训的结构化解法。B01–B12 基准即"组件是否还值得存在"的常态化回答。

**缺口：** 无结构性缺口。次要建议：P7-T04 性能 campaign 的 baseline delta 已隐含"模型升级重定价"语义，可在该卡执行时显式加一行"model/harness 组件贡献分解"，不需要新任务。

## 3. Context Engineering 维度

**前沿基线。** Anthropic 将 context engineering 定义为选择/压缩/裁剪/动态加载；Claude Code 用 progressive disclosure（frontmatter ~100 token → 触发才载全文）支撑大规模技能库，并在 Claude 5 世代删掉 80% 系统提示词。学术侧：context rot 的诊断与缓解（arXiv 2606.29718）、LOCA-bench 极端上下文增长基准、Self-GC 自治理上下文、VISTA 指出前沿模型对自身上下文状态"本体感知盲"（proprioceptively blind），需要运行时仪表盘。

**CognitiveOS 对照。** REQ-CTX-002..007（最小充分、先授权后排序、显式 loss、新鲜度、cache key）+ cognitive-discovery（manifest、增量 delta、停滞出口）+ operation-catalog 两级目录（Summary→Descriptor）——progressive disclosure 与 context rot 防御在规范层已经完备，早于业界普及。

**缺口：** VISTA 所指的**本体感知**（Agent 知道自己还剩多少预算/上下文）在本仓库以 REQ-RES-001/REQ-PERF 可观测指标存在，但主要面向 operator 而非 Agent 本身。低成本改进：P3-T02 context builder 把"剩余 token/成本预算"作为一个 System fragment 回注给 Agent（已有数据，仅是暴露面问题），归入 P3 现有卡即可。

## 4. Loop Engineering 维度

**前沿基线。** 2026 年 6 月业界正式命名 loop engineering：设计"触发→行动→验证→重试→停止"的控制循环而非逐轮人肉提示；核心构件是 verifier、stop rule、retry budget；成本现实（单 loop 50k–200k token）使预算成为一等约束。学术侧：长程连贯性成为核心问题（Awesome-Long-Horizon-Agents 路线图）、VeRO 用 harness 优化 harness、hypothesis-tree refinement 做研究型 loop。

**CognitiveOS 对照。** 这是本仓库最强的维度：独立 verifier（REQ-RUN-009）、有界重试/迭代（REQ-RUN-005/008）、OUTCOME_UNKNOWN 不盲重（PERS-PR-009）、停滞机器可观察出口（REQ-DISC-STAGNATION-001）、loop-checkpoint 带 `pinned_environment_versions` 的恢复重验证、B07 loop 效率基准。业界刚命名的实践在此已是带负例测试的合同。

**缺口：** 无。B07 执行时可顺带报告业界通行的 loop 成本口径（token/loop、verify 占比）以便外部可比，不需要计划变更。

## 5. 创新点论证：OS 级运行环境感知（Environment Perception）

### 5.1 学术与工业证据

- **Agent 缺乏环境好奇心**（arXiv 2604.17609）：agent 不会主动识别并调查环境中意外但相关的观察；影响因素之一就是 scaffold 提供了哪些工具——即**感知能力应由 harness/OS 侧供给**。
- **Ask the World Before Acting**（arXiv 2606.31422）：把 probing 定义为"修复结构化信念表"的专门动作，校准的世界模型显著提升行动质量。
- **Look Before You Leap**（arXiv 2605.16143）：把环境探索形式化为独立能力（无目标信息采集，发现状态/对象/affordance/动作语义）。
- **VISTA**（arXiv 2606.30005）：运行时状态仪表盘使 agent 可对自身状态行动。
- **工业侧**：MCP gateway 只向当前执行环境呈现已注册启用的工具（"Playground 所见即运行时所得"）；生产实践把 `tools/list` 序列化为 CI fixture、health endpoint 暴露 version+tool hash、MATCH/DRIFT/UNVERIFIABLE 三态处理宿主装载状态；AGENTS.md 静态描述环境但研究显示 LLM 生成的 AGENTS.md 在 8 个设置中 5 个降低成功率——**机器验证的事实优于生成的散文**；Nix/E2B/devcontainer 用声明式模板供给环境——但**声明 ≠ 实际**，仍需感知层对账。

### 5.2 CognitiveOS 现状：碎片存在，未统一

| 已有原语 | 位置 | 覆盖 |
|---|---|---|
| Provider 能力快照（主动探针） | P1-T03、`cognitive init` 流程 | 模型 chat/stream/tool/cancel 能力 |
| Tool descriptor + health epoch + 漂移重绑定 | DEC-P-14、P2-T05、P5-T03/T04、REQ-CAT-BIND-001 | 单个已注册 Operation 的 schema/健康 |
| doctor redacted facts/digests | P7-T03 | daemon/DB/Secret/Provider/Pi/Tools/Processes/migrations |
| `environment_digest` | execution-context.schema | 执行绑定的环境指纹（引用，无内容模型） |
| `pinned_environment_versions` | loop-checkpoint.schema | 恢复时按 pin 重验证 |
| `environment_fingerprint` | profile-manifest.schema | 证据环境标识 |
| Cognitive Discovery manifest | core/specs/cognitive-discovery | **受治理资源**（context/memory/operation 域）的可发现性 |

**缺口定义：** 没有一个统一对象回答 Agent 的问题："我现在运行在什么宿主上（OS/arch/资源余量）、有哪些系统工具和运行时及其版本、依赖状态（lockfile digest）、网络可达性等级（offline/proxy/端点存活）、哪些 MCP/Tool server 活着"。后果：每个 Agent 每个 loop 用 bash 临时探测（token/延迟浪费，且学术证据表明 agent 自主探测做得差），或基于过期假设行动（漂移→失败→触发重试预算）。`environment_digest` 有指纹无内容模型，正是这个洞的形状。

### 5.3 建议：采纳，定位为"统一与扩展"，非新子系统

**设计要点（复用既有模式）：**

1. **EnvironmentSnapshot**：版本化快照对象，沿用 operation-catalog snapshot 模式——每个 fact 带 `probe 出处、观测时间、TTL/freshness、digest`；整体 digest 即 execution-context.`environment_digest` 的内容来源，闭合现有悬空引用。
2. **facts-not-beliefs 纪律**：只承载机器探测的事实（含 MATCH/DRIFT/UNVERIFIABLE 三态），不承载生成的描述——与 AGENTS.md 负面证据一致。
3. **消费面 = Context fragment**：作为 P3-T01 context source 集合中的一个 source 进入 REQ-CTX 管道（最小充分、预算、显式 loss 全部自动适用），Agent 不需要新 API 面。
4. **probe-on-demand**：快照过期或 Agent 声明 InformationGap（`authority_refresh` 类）时，daemon 在预算内重探——对齐"Ask the World"的 probing-as-belief-repair。
5. **生产者复用**：init 探针（P1-T03）、Tool health（P5）、doctor（P7-T03）是同一探测层的三个消费者，避免三处各写一套。
6. **安全边界**：环境事实进入 evidence 前走 doctor 同款 redaction；网络可达性探测本身是外部读操作，走只读短路径但仍审计（DEC-P-17）。

**落点建议（若采纳，走同批修订）：**

- 新增 **DEC-P-21 环境感知**（DEC-P-20 已被授权交互模型 / ADR-0026 占用）：候选 = 每 Agent 自探（bash ad hoc）/ 静态声明文件（AGENTS.md 式）/ **daemon 统一快照服务（选此）**；重评条件 = 快照维护成本超过 Agent 自探节省。
- 规范落点：cognitive-discovery companion 新增 environment domain，或独立薄 companion + `environment-snapshot.schema.json`；`execution-context.environment_digest` 语义由其定义。
- Personal 任务落点：**扩展 P3-T01/T02**（environment 作为 context source + fragment，G3/B03/B06 可直接度量收益：探测轮次下降、token/loop 下降），探测器实现挂 P1-T03（已有 probe 骨架）与 P2-T05（tool health 生产者）；**不建议新增独立任务**，避免 critical path 变动。若 owner 倾向独立任务，则为 P3-T07 + PERS-PR-024（022/023 已被低摩擦授权与 backup/restore 占用），且不入 critical path。
- 非声明：采纳本建议不改变任何 Gate/证据/Profile 结论。

### 5.4 反方观点与风险

1. **快照自身会过期**——必须带 TTL 与 drift 事件，镜像 REQ-CAT-BIND-001 的重绑定语义；否则比无快照更危险（过期权威事实 > 无事实）。
2. **范围蠕变为"世界模型"**——严格限定 facts-with-provenance；语义推断留给 Agent。
3. **隐私面扩大**——环境事实（安装软件清单、网络拓扑）本身敏感，须按 REQ-DISC-PRIVACY-001 的不泄露存在性原则限定 scope。
4. **过早优化**——反驳：P3 本来就是 Context/Token/Loop 效率阶段，B03/B06 提供现成的度量口径，收益可证伪。

---

## 6. 参考来源

学术：AIOS（arXiv 2403.16971，COLM 2025）；LiteCUA（2505.18829）；Quine（2603.18030）；Model-Native Computing（2606.00288）；Harness Engineering 探索研究（2602.14690）；LOCA-bench（2602.07962）；Context Rot 诊断（2606.29718）；Self-GC（2607.00692）；VISTA 状态本体感知（2606.30005）；环境好奇心（2604.17609）；Look Before You Leap（2605.16143）；Ask the World Before Acting（2606.31422）；VeRO（2602.22480）。

工业：Anthropic 长程 harness（ZenML LLMOps DB 收录）；Claude 5 世代 context engineering 规则（claude.com blog）；Agent Skills progressive disclosure（platform.claude.com）；AGENTS.md 格式与负面证据（agents.md / augmentcode）；MCP registry/gateway 环境作用域呈现（Kong/TrueFoundry/MintMCP）；MCP 生产化 schema-drift CI 与 health hash 实践（ByteBridge）；E2B/Modal/Nix 声明式环境（northflank/modal/codex.danielvaughan）。
