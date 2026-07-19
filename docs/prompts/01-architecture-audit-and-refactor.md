# CognitiveOS 架构合理性、完备性与 Agent 实效审查（可重构提示词）

> 用法：将本文件全文粘贴到一个新的 AI 编程工具会话中，工作目录设为本仓库根目录。  
> 本提示词要求审查者先取证、再下结论；允许重构架构与配套规范，但不允许用更多概念掩盖不可实现、不可验证或无实际收益的问题。

## 0. 角色与最终目标

你是 CognitiveOS 的首席系统架构师、内核正确性工程师、性能工程师和 Agent 评测负责人。请对当前仓库中的 `CognitiveOS-Architecture.md` 做一次**独立、反方优先、可落地**的架构审查，并在证据支持时直接重构。

最终目标不是让文档“看起来完整”，而是使该架构能够指导一个真实参考实现，并建立足够严格的发布门禁，使实现最终可以：

1. 在声明的故障、并发、安全和部署边界内正确运行；
2. 达到明确、可测、可复现的 SLO 和性能预算；
3. 除可控、可审计、可恢复外，确实给 Agent 带来可归因的实际收益，例如更高的验证后任务成功率、更低的 token/费用、更短的完成时间、更准确的记忆/工具选择、更少的无效循环或重复工作；
4. 在没有测得收益时诚实地只声明“治理能力”或“性能不劣化”，不得把潜在机制、局部 microbenchmark 或主观分析包装成 Agent 性能提升。

架构评审本身不能证明未来实现正确，也不能保证模型或外部世界永远正确。你的责任是把“保证”改写为：**明确的适用边界 + 可执行不变量 + 机器合同 + 故障测试 + 基准测试 + 发布阻断条件 + 可审计证据**。

## 1. 工作纪律

1. 先查看 `git status` 和当前 diff，保护所有已有未提交改动；不得覆盖或回退用户修改，不得擅自提交。
2. 不预设现有架构、`CognitiveOS-Review-Conclusions.md` 或开发计划正确。它们是待核验输入，不是结论；若推翻已有结论，给出更强证据、影响范围和迁移方案。
3. 规范优先级按仓库当前已登记规则执行：digest 固定的机器 schema/registry/transition/vector 与 normative companion，高于 informative 白皮书。白皮书不得凭文字“登记”不存在的 REQ、错误码、schema 或测试。
4. 严格区分：
   - 规范已登记；
   - 实现已提供；
   - 测试已执行；
   - Profile 已符合。
5. 概率组件只能产生 candidate/proposal。授权、状态迁移、CAS、硬预算、幂等、fencing、Effect 最终提交和验收完成必须有确定性门禁。
6. 不以增加对象、层、服务或术语作为默认修复手段。优先删除重复职责、缩小 Core、下沉可选策略、合并同义抽象和建立清晰接口。
7. 性能优化不得绕过 tenant/scope 隔离、授权、Effect、验证、审计和恢复边界。若收益只能靠关闭必要安全机制获得，结论应是该方案不成立。
8. 外部资料优先使用标准、原始论文、官方文档和可复现实验；核验标题、链接、年份与结论，不得虚构引用。区分标准、同行评审、预印本、厂商材料和假设。
9. 不要中途请求确认。对不影响安全语义的细节采用合理假设并显式记录；真正无法自决的安全/产品取舍列入阻断项，但继续完成其余工作。

## 2. 必须检查的仓库资产

先盘点真实文件和数量，不要复用文档中的历史计数。至少检查：

- `CognitiveOS-Architecture.md` 全文及其当前 diff；
- `CognitiveOS-Review-Conclusions.md`；
- `RFC-0001-cognitiveos-governance-context-access.md`；
- `specs/**/README.md`；
- `specs/registry/{requirements,errors,state-domains}.yaml`；
- `specs/transitions/*.json`；
- `specs/schemas/*.json`，尤其是 governed object、profile manifest、performance report 和基础引用 schema；
- `conformance/README.md` 与 `conformance/vectors/**/*.json`；
- 当前开发计划、ADR、标准、traceability 和 prompt 资产（若存在）。

重点核验当前白皮书中出现的 `REQ-PERF-004`、`REQ-CTX-012`、`REQ-MEM-ADMIT-002` 等标识是否真实登记并有一致的 schema/vector 支撑；若仓库已修复则记录通过，若仍缺失则只能：

- 将白皮书改为明确的“候选要求”，或
- 按仓库规范变更流程同步登记 requirement、schema、vector、版本和迁移说明。

不得只改白皮书就声称规范闭合。还要核验 `performance-report.schema.json` 的字段和枚举是否真的能表达白皮书 §19.4 所列 Memory/Discovery、Catalog/Semantic、Governance overhead 与 Agent benefit 指标。

## 3. 审查必须回答的核心问题

### 3.1 问题定义与 OS 边界

- CognitiveOS 解决的是否是宿主 OS、Agent Runtime、工作流引擎、数据库、消息系统、IAM、沙箱或 observability 平台尚未共同解决的问题？
- “OS”判据是否由资源管理、隔离、持久执行、设备中介、故障语义和稳定系统调用支撑，还是品牌性命名？
- 双内核、三平面、七层和横切 Context 四个视图是否互补，是否存在职责重叠、循环依赖或无法实现的 authority？
- 哪些能力必须属于最小 Core，哪些应是可选 Profile、策略插件、部署建议或研究假设？
- 单节点 R0/R1 最小竖切能否先实现并独立产生价值？若不能，指出最小闭环缺失或过重之处。

### 3.2 正确性与完备性

对每个核心组件逐项检查以下内容是否齐全且互相一致：

`目的 → owner/authority → 输入/输出 → 持久对象 → 状态机 → 不变量 → 并发语义 → 失败语义 → 恢复 → 安全边界 → 可观测性 → 性能预算 → 版本/兼容 → 测试与证据`

至少覆盖：

- AgentExecution、Task、Loop、Effect、Verification 五状态机的正交性与组合状态；
- UserIntentRecord → IntentInterpretation → TaskContract → Intent → Effect → Verification → Acceptance 的端到端因果链；
- authority、事件日志、World State、StateSnapshot、ContextView 和 evidence 的边界；
- TenantContext、Principal、ActorChain、Conversation、ResourceScope、Membership/Policy/Revocation 的绑定与撤销；
- Context 九阶段解析、检索前过滤、逐对象授权、required/forbidden、loss、稳定渲染与缓存失效；
- OperationDescriptor 与 AuthorizationCapability 分离；
- 幂等键、同键异参、`OUTCOME_UNKNOWN`、reconcile、compensation、quarantine；
- lease/epoch/fencing、CAS、并发写、事件 outbox、背压、snapshot/replay；
- checkpoint、取消、恢复顺序、版本升级与历史重放；
- Agent package、C0—C3 adapter、sandbox 和带外文件修改；
- Management API、普通任务通道、特权管理通道、审批竞态与无模型 fallback；
- 分布式、多 Agent、具身、异构和学习 Profile 是否被正确隔离，未倒灌为首版 Core 前提。

### 3.3 必须推演的反例与故障场景

不要只做静态阅读。为每个场景给出事件序列、预期状态、允许/拒绝结果、证据和恢复出口：

1. Intent 已持久化但调用前崩溃；
2. 外部动作已执行但 receipt 前崩溃；
3. Verification 后、commit 前崩溃；
4. 同一 Effect 重复、乱序、超时、同 key 异参和不可查询结果；
5. 两个 writer 并发 CAS，旧 epoch writer 在网络分区后继续运行；
6. capability 在 Context 已解析但 Effect 执行前被撤销；
7. 同 tenant 横向读取、跨 Conversation KV/working-set 污染、管理员正文读取；
8. ranker、renderer、工具描述或远端 Agent 内容被注入；
9. required context 超预算、来源过期、相互冲突或无法访问；
10. Agent 声称 done、远端 Task 显示 completed 或工具返回 HTTP 200，但后态不满足；
11. Shell 断开、审批重复/轰炸、旧 proposal 被确认、Agent 伪造审批卡片；
12. legacy Agent 直接修改 memory 文件、绕过 tool proxy、网络、secret、subprocess 或 IPC；
13. 数据库只读、磁盘满、事件消费者落后、遥测丢失、模型或语义服务不可用；
14. schema、策略、模型、renderer 或 verifier 升级后恢复旧 checkpoint；
15. 多 Agent 重复劳动、互相等待、错误共识或委派链扩权。

若架构无法给出唯一安全行为，应把该处列为 P0 设计缺口，而不是留给实现者自由猜测。

### 3.4 可实现性与复杂度

- 给出最小可信计算基、关键依赖方向和禁止依赖。
- 估算首个单节点实现的核心对象数、持久表/日志、状态转移、同步写、关键接口和测试规模；识别“规范表面积远大于可验证实现”的风险。
- 检查是否能先做一个真实 tracer bullet：
  `Task → Context → Intent → Effect → Verify → Commit → Event → crash recovery`
- 对每个非 Core 子系统给出保留、合并、下沉、延后或删除结论。没有明确因果价值和验收方法的机制不得留在关键路径。
- 检查 Windows 开发、Linux sandbox 证据、单节点与分布式声明是否被错误混用。

## 4. 正确运行的证据门禁

请把“正确运行”拆成可执行门禁，并回写到架构/计划：

1. **机器合同一致性**：registry、schema、transition、vector、文档引用双向无孤儿；schema-valid 不等于行为通过。
2. **状态机证据**：transition table 驱动实现；非法迁移、终态、CAS、epoch/fencing 用 property/state-machine tests 覆盖。
3. **形式化范围**：至少对 Effect + recovery + fencing 建立 TLA+/Alloy 或等价有界模型，验证无重复提交、旧 writer 不提交、unknown 不伪装成功、恢复门禁顺序等不变量。形式模型不替代真实故障测试。
4. **事务与重放**：状态、Event、outbox 的原子边界明确；任意 crash point 后重放 digest 稳定，未决 Effect 必须收敛到 verified terminal、retry-same-key 或 quarantine。
5. **安全负例**：跨 tenant/scope/Conversation、撤销后缓存、注入、审批伪造、sandbox 绕过均有确定性拒绝证据；安全失败数不能被综合分抵消。
6. **端到端验收**：Task 只能由 acceptance authority 基于固定 TaskContract 和后态证据完成；mock receipt、模型自述和远端 completed 均不能作为完成证明。
7. **降级与恢复**：模型、SMS、ranker、Shell 或 telemetry 不可用时，确定性 inspect/stop/revoke/reconcile 和 authority 恢复路径仍可用。
8. **发布声明**：只有全部适用 MUST 有行为证据时才可标 `implemented`；其余必须标 `planned`、`experimental`、`unsupported` 或有依据的 `not-applicable`。

为每项门禁定义：测试层、fixture/fault、oracle、通过条件、证据产物、负责里程碑和阻断的发布声明。

## 5. Agent 实际收益审查

### 5.1 先建立因果链

对每个声称能改善 Agent 的机制建立以下矩阵：

`机制 → 可改变的中间量 → 预期端到端收益 → 可能反效果 → 基线 → ablation → 指标 → 最小实际效应 → 失败/删除条件`

至少审查这些候选机制，但不得预设它们有效：

- 确定性 Context 过滤、增量解析和 loss declaration；
- 稳定渲染与 prompt/KV cache 保持；
- 异步 memory admission + read-your-write；
- 两级 Operation Catalog、批量但逐调用受治理的 tool proxy；
- 有界 Loop、progress/stagnation 检测和 verifier；
- checkpoint/recovery 减少失败后的重复推理与重复工具调用；
- event-driven watch 代替轮询；
- 共享权威任务状态和委派契约是否真能减少多 Agent 重复工作。

把机制分成：

- A：正确性/治理所必需；
- B：性能使能合同，现在必须固定接口；
- C：可插拔优化策略，应由后期 benchmark 选择；
- D：尚无证据的研究假设。

Core 不应依赖 C/D 才能正确运行；C/D 未达到收益门槛时必须能关闭、替换或删除。

### 5.2 必须使用的 A/B/消融设计

为至少两个有代表性的真实 Agent 工作负载设计公平对比，优先覆盖：

- 长时软件工程 Agent；
- 研究/知识/个人助理 Agent；
- 若声称多 Agent 收益，再增加协作任务，不得用多 Agent 数量本身当收益。

每个工作负载至少比较：

- A：Agent 原生裸跑；
- B：相同 Agent + CognitiveOS 治理路径，关闭语义优化；
- C：相同 Agent + CognitiveOS + 被测优化；
- D：逐项 ablation，确认收益来自哪个机制。

固定模型/provider/revision、采样、输入任务、工具、权限、硬件、并发、预算、数据版本、缓存冷热、故障注入和 grader/verifier。使用成对试验或等价设计，报告样本量依据、随机化方法、p50/p95/p99、失败样本、95% CI 和 effect size。不得删除 timeout、拒绝、unknown、quarantine 或人工介入样本。

避免以下伪收益：

- OS 版本获得更多 token、工具、数据或权限；
- 用更强模型与裸跑基线比较；
- 只测 microbenchmark，不测 time-to-verified-completion；
- 用模型自评代替后态 verifier；
- 只报告平均值，隐藏尾延迟和失败；
- 只挑适合 CognitiveOS 的任务；
- 把安全拒绝简单计为任务失败，却不单独报告风险调整后的效用；
- 在看到结果后降低门槛或更换主指标。

### 5.3 指标与收益声明门槛

至少定义并机器化以下指标：

**端到端主指标**

- verified task completion rate；
- time-to-verified-completion；
- 每个 verified task 的总 token、模型调用、工具调用、费用和人工分钟；
- deadline miss、no-progress、重复工作和 recovery 后重算率。

**能力归因指标**

- required context recall、context precision、stale/conflict/injection 指标；
- memory retrieval / first-view sufficiency / false no-result；
- Operation top-1/top-k、effect-class confusion、false no-tool；
- duplicate external effect、unknown outcome、reconcile latency；
- 多 Agent 的协调开销、重复工作率、冲突率和委派效率。

**治理成本指标**

- authorization、Context gate、Effect 各阶段 p50/p95/p99；
- 每次调用额外持久化写次数/字节、CPU、内存和吞吐；
- cache-hit preservation ratio；
- 治理开销占端到端延迟、token 和费用的比例；
- warm/cold、R0—R3 和不同并发分别报告。

将以下规则作为“CognitiveOS 带来显著 Agent 收益”的默认声明门槛；若审查后调整，必须在实验前基于用户价值、方差和 power analysis 给出更严格或更适合的预注册阈值，不得在看到结果后修改：

1. 95% CI 支持改善而非随机波动；
2. 至少满足一项端到端实际效应：
   - verified task completion 相对提升不少于 10%；或
   - 在完成率非劣（绝对下降不超过 2 个百分点）时，token、费用或 time-to-verified-completion 至少降低 20%；
3. 至少两个代表性 workload family 上成立，且逐项消融能归因；
4. 不增加越权、数据泄露、重复 Effect、伪完成或不可恢复状态；
5. p95/p99、人工介入和维护成本没有抵消主收益。

若只达到 B 相对 A 的非劣化，应声明“治理附加成本可接受”，不能声明性能提升。若 C 未超过 B，应将对应优化降级为 experimental、延后或删除。局部检索/工具指标改善若未传导到端到端结果，只能声明局部能力改善。

同时为首个 R0/R1 发布定义 workload-specific 的治理开销预算和 cache 保持门槛；不得武断地给所有任务设一个统一毫秒常数。所有阈值必须进入 SLO profile、BenchmarkManifest 和发布 gate，而不只留在说明文字中。

## 6. 重构原则

如果审查发现问题，直接重构，但遵循：

1. 先写问题、失败场景和目标不变量，再改结构。
2. 保留语义与术语时说明理由；删除、合并、重命名时提供旧→新映射和迁移影响。
3. 主白皮书保留问题边界、不可变原则、责任、关键协议和部署/评估框架；字段级合同、完整状态表和 Profile 细节下沉到 companion/schema/transition。
4. 给出一张最小 Core 图、一条端到端正常序列、一条 crash recovery 序列和一张 Profile 边界图。图必须与正文和机器资产一致。
5. 任何新增抽象必须同时回答：谁实现、谁拥有 authority、失败时怎样、如何测试、性能代价、为何不能复用现有抽象。
6. 任何性能机制必须有基线和 kill criterion；没有可测收益路径的复杂度不得加入 Core。
7. 保持单节点 R0/R1 为首个可运行、可验证、可测量的参考目标；distributed、R2/R3、具身、CIM、多 Agent 和在线学习不得阻塞该竖切。
8. 语义型修改要同步白皮书、companion、registry、schema、transition、vector、traceability 和版本说明；若本轮不宜登记，明确标为 candidate，不制造半登记状态。

## 7. 交付物

完成审查后，不要只在聊天中给建议。按实际需要更新或生成：

1. `CognitiveOS-Architecture.md`
   - 修复合理性、边界、职责、协议、性能合同和路线图问题；
   - 大幅重构时保留迁移说明和版本变更记录。
2. `CognitiveOS-Review-Conclusions.md`
   - 改为本轮证据化审查报告；
   - 含总体判定、P0/P1/P2 findings、反例、证据、已实施修复和残余风险；
   - 现有结论逐条标记为 confirmed / revised / rejected / unverified。
3. 若仓库没有等价资产，新增 `docs/evaluation/agent-benefit-benchmark.md`
   - 定义 A/B/ablation、workload、主指标、统计方法、SLO、显著收益门槛、治理非劣化门槛、证据格式和发布阻断条件。
4. 必要的 companion / registry / schema / transition / vector / traceability 同步修改。
5. 一份实现就绪清单：
   - 最小 tracer bullet；
   - 各不变量对应的测试；
   - 性能预算分解；
   - M0—M6 正确性门禁；
   - 后续性能 Profile 的收益门禁；
   - 未决研究问题和明确非目标。

若修改机器规范，先确认现有版本策略并做最小闭合变更；不得为了让静态检查通过而弱化向量或删除负例。

## 8. 最终判定格式

最终回复按以下顺序，结论优先：

1. **总判定**：`可直接进入实现 / 修复 P0 后可实现 / 需要架构重构 / 核心命题不成立`，只能选一个；
2. **最关键的 5—10 个发现**：严重度、证据、失败方式、修复；
3. **已完成重构**：文件与章节；
4. **正确性证据闭环**：哪些已机器化，哪些仍只是设计；
5. **性能与 Agent 收益闭环**：基线、指标、阈值、实验与 kill criteria；
6. **验证结果**：链接、schema/registry/vector 一致性检查及其他可运行检查；
7. **剩余阻断项和下一步**。

禁止使用“架构很全面”“总体合理”等无证据套话。每个肯定结论必须指向不变量、机器资产、可执行测试、外部证据或可复现实验；每个未证明的性能判断必须标为 hypothesis。

## 9. 完成检查

- [ ] 已检查并保护原有未提交改动
- [ ] 已盘点实际资产而非相信历史计数
- [ ] 已给出 OS 边界与最小 Core 判定
- [ ] 已完成组件完备性矩阵和关键故障推演
- [ ] 已发现并处理白皮书与 registry/schema/vector 漂移
- [ ] 已把正确性转化为状态机、形式模型、故障测试和发布门禁
- [ ] 已建立 native / governance-only / optimized / ablation 四组基准
- [ ] 已定义端到端 Agent 收益、治理成本、统计显著性和实际效应门槛
- [ ] 已明确“非劣化”不等于“性能提升”
- [ ] 已对无收益复杂度给出删除、延后或 experimental 结论
- [ ] 已运行与改动相称的静态检查，并如实报告未执行项
