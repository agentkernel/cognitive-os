# CognitiveOS 架构评审结论与优化建议

- **版本**：1.2
- **日期**：2026-07-19
- **变更记录**：v1.2 记录各改进项在白皮书 1.0.0 定稿中的应用状态（§3 总表新增"应用状态"列），并将开发计划引用改指 History 归档路径；v1.1 新增 IMP-18（Agent 性能提升路径与时机声明，见 §1 第 4 项研判）；v1.0 初版。
- **应用说明**：[CognitiveOS-Architecture.md](./CognitiveOS-Architecture.md) v1.0.0 已按本文档完成修订闭环；本文档转为评审与应用追踪记录。原开发计划已归档至 [History/CognitiveOS-Development-Plan.md](./History/CognitiveOS-Development-Plan.md)，其 M0—M11 里程碑衔接内容随实现启动时另行恢复，白皮书 §21 路线图为当前有效规划。
- **文档性质**：Informative 评审结论。本文不构成规范义务，不新增任何已登记 REQ-ID、错误码、schema 或测试向量；文中所有"候选 REQ / 候选向量"均须按规范变更流程分类登记后才具有规范地位（与 [History/CognitiveOS-Development-Plan.md](./History/CognitiveOS-Development-Plan.md) §2 的分类要求一致；IMP-02/03/04/11 的候选项已在白皮书 1.0.0 定稿中按该流程登记）。
- **评审对象**：[CognitiveOS-Architecture.md](./CognitiveOS-Architecture.md) v0.8 Draft 及仓库配套资产。
- **用途**：作为后续修订架构白皮书（及关联 companion / registry / 开发计划）的改动依据与追踪清单。每个改进项有稳定 ID（IMP-xx），后续 PR / 编辑应引用对应 ID。

---

## 0. 使用方式

1. 改动前先读 §2"经验证的设计基线"，避免在修订中削弱已被证据支持的设计。
2. 按 §3 总表选择改进项，按 §4 详述执行；每项含目标章节、改动类型、验收判据。
3. §5 是事实核查与资产漂移记录，属于修正性改动，可独立执行。
4. §6 说明与开发计划的衔接关系，涉及两份文档联动的项在此标注。

---

## 1. 评审方法与证据基础

本轮评审由四部分组成：

1. **整体架构评审**：从第一性原理检验 §2 推导链与七不变量，并将各机制与 2024–2026 年学术研究、业界实践逐项对照。
2. **兼容负载分析**：以两个真实成熟 Agent（OpenClaw：Gateway 守护进程 + Markdown 文件记忆 + ClawHub 技能 + cron/heartbeat；Hermes Agent：自改进技能循环 + MEMORY.md/USER.md + 8 种外部记忆插件 + 40+ 工具 + MCP + Kanban 多智能体）为样本，推演其按 §5 安装路径运行在 CognitiveOS 上的性能损耗与统一管理映射。
3. **审批协议评价**：评估"Agent Shell 作为审批终端 + 对话流内插入审批 + 自然语言/密码完成审批"提案，对照通道安全、审批疲劳攻击与业界审批产品形态。
4. **性能基建研判**：针对"安装成熟 Agent 后，除治理能力外能否显著提升其运行性能（记忆/工具/技能调取准确度、任务完成准确率、token 经济性、多 Agent 协作），是否需要统一规范 OS 基建、置于当前架构阶段还是后期"的问题，逐项核对现有架构承载机制与缺口。结论收录为 IMP-18。

外部证据核验说明：白皮书附录 C 被抽查的引用（AIOS arXiv:2403.16971、MAST arXiv:2503.13657、Nature 639 CIM 处理器、AI Harness Engineering arXiv:2605.13357、Anthropic harness/context engineering 博客、Karpathy LLM Wiki gist 等）**全部真实存在**。评审新引入的外部证据见附录。

仓库资产实测（2026-07-19）：白皮书 1776 行；RFC-0001 507 行；companion 规范 12 份；`specs/schemas/` JSON Schema **40 份**；`conformance/vectors/` 测试向量 **65 个**；`specs/registry/requirements.yaml` REQ 条目 **244 条**；`errors.yaml` 错误码 **49 个**；实现代码 0 行。

---

## 2. 经验证的设计基线（不建议改动）

以下设计在评审中获得学术与业界证据支持，后续修订**不应削弱**其语义；如需变更须给出强于本表证据的理由。

| ID | 设计 | 主要证据锚点 |
|---|---|---|
| V1 | 第一性推导与七不变量（§2）：推导链严密，不变量集合无明显冗余或遗漏 | 全部下列证据的交集 |
| V2 | 三事实分离：事件日志 / World authority / 观察证据（§8.1），ContextView 非权威（§9.1） | 事件溯源实践；控制论 observer/plant 区分；业界"HTTP 200 当成功"事故史 |
| V3 | 五个执行生命周期状态机正交分离；`CANDIDATE_COMPLETE→COMPLETED` 由验收 authority 推进；Verification 可独立过期（§6.3） | Temporal workflow/activity 分离；MAST 第三类失败（验证缺失）；Anthropic harness"外部评估者" |
| V4 | Effect 协议：`OUTCOME_UNKNOWN` 一等状态、稳定幂等键、timeout 不换键、补偿独立授权、无条件 exactly-once 不承诺（§6.3、§12.5、§8.6） | Temporal/saga 行业共识；Stripe 幂等键实践 |
| V5 | 恢复顺序 fence→重放→对账→重授权→重解析→恢复 Loop（§16.6）；lease/epoch/fencing（§16.3） | Kleppmann fencing token；Gray & Cheriton lease |
| V6 | Context 九阶段解析：检索前 tenant/scope 过滤、授权先于 ranker、loss declaration、required fail-closed（§9.3、§9.6） | Anthropic context engineering（注意力预算/压缩有损/JIT）；Lost in the Middle；MemGPT 虚拟上下文；prompt cache 跨租户侧信道研究 |
| V7 | 语义隔离不变量与确定性参考监视器：控制/数据分区、注入内容不得提升为控制、模型受注入时确定性门禁仍阻止越权副作用（§9.2、§17.3） | CaMeL（DeepMind 2025，控制/数据流分离 + capability 的可证明防御）；Dual-LLM 模式；MCP 工具描述投毒攻击 |
| V8 | 双内核：实时安全内核独立于认知路径、硬实时回路禁用动态 Context Resolution、不规定统一频率（§4.2、§13） | Simplex/Run-Time Assurance 架构；ISO 10218:2025；机器人三层架构三十年共识 |
| V9 | 机制/策略分离与最小可信计算基方向（§7、§23 决策九） | L4/seL4 二十年经验（最小性原则） |
| V10 | 多 Agent 审慎立场：独立性不可由角色名假定、多副本一致≠事实正确、委派为显式衰减契约（§15） | MAST：14 种失败模式，多 Agent 收益常为边际 |
| V11 | BenchmarkManifest 与禁止综合分掩盖安全失败/尾延迟（§19.4） | MLPerf/HELM 多指标规范；评估复现性危机 |
| V12 | C0—C3 与 R0—R3 正交（§5.2、决策二十） | 避免"集成度冒充安全等级"的谬误；无直接反例 |
| V13 | 身份模型：ActorChain（initiating/effective/workload）、委派单调衰减、Tenant≠权限（§6.2、§12.3） | IETF OAuth on-behalf-of AI agents 草案（act 链）；SPIFFE/WIMSE；RFC 8693；Zanzibar/Cedar 实践 |
| V14 | 附录 C 证据分级纪律（[STD]/[PEER]/[PRE]/[IND]/[HYP]）；抽查引用全部真实 | 本次核验 |
| V15 | 任务通道/管理通道隔离："普通 Conversation 不能通过措辞切换为管理通道"（§12.9）；"自然语言本身不是 capability"（§12.3） | 审批协议评审中该边界恰好挡住全部三条攻击路径（见 IMP-05） |
| V16 | "认知经济学"降级为可观测核算而非统一市场（§2.2） | 对朴素 agent 市场设计的普遍批评 |
| V17 | §5.3 六族 Adapter 接口（Identity/Memory/Tool/Completion/Checkpoint/Sandbox）映射模型 | 对 OpenClaw/Hermes 的实测形态推演：两者的记忆工具、技能描述文件、cron/heartbeat 均有清晰拦截点，映射自然（见 §1 第 2 项） |

---

## 3. 改进项总表

优先级定义：**P0** = 影响架构可信度或采用可行性的关键项；**P1** = 显著提升质量/落地性；**P2** = 清理与优化。

| ID | 主题 | 优先级 | 目标位置 | 改动类型 | 应用状态（白皮书 1.0.0） |
|---|---|---|---|---|---|
| IMP-01 | 实现优先原则写入路线图，冻结规范表面扩张 | P0 | §21 路线图原则；§1.2 | 原则性修订 | 已应用（§21 冻结条款、§1.2 实现验证状态注记） |
| IMP-02 | 渲染前缀稳定性成为显式要求 | P0 | §9.3 render、§9.4；候选 REQ + 候选向量 | 新增要求 | 已应用并登记（`REQ-CTX-012`、向量 CTX-RENDER-001、core companion §6.6） |
| IMP-03 | 记忆/知识准入异步化与 read-your-write 工作集 | P0 | §9.5、§18.3；governed-memory 与 agent-compatibility companion | 新增要求 | 已应用并登记（`REQ-MEM-ADMIT-002`、向量 MEM-RYW-001、两份 companion 同步） |
| IMP-04 | 治理开销自身的性能契约 | P0 | §19.4 新增指标族 | 新增指标 | 已应用并登记（Governance overhead 指标族、`REQ-PERF-004`、performance-report schema 扩展） |
| IMP-05 | 审批子系统：审批即服务、可协商审批、分级确认、防疲劳 | P0 | §12.10 扩展（或新 §12.12）；§17.1；§19.4；§23 | 新增小节 + 威胁模型扩充 | 文本已应用（新 §12.12、§17.1/§17.2、§19.4 审批指标、决策二十五）；REQ/向量登记按冻结原则递延为独立草案 |
| IMP-06 | R0"薄路径"：低风险档位的合法降级实现形态 | P1 | §20.4、§20.5、§4.7 | 新增说明 | 已应用（§20.5 降级映射表 + 不可降级边界、§4.7 注记） |
| IMP-07 | 关键状态机形式化验证承诺（TLA+/Alloy） | P1 | §21、§19.5、§22 | 路线图项 | 已应用（挂 §21 Phase 4，§19.5 注明证据地位；形式化本身待实现阶段） |
| IMP-08 | 对象本体分层：最小核心对象集 vs 扩展集 | P1 | §4.7、附录 A | 结构重组 | 已应用（附录 A.1 最小集 14 项 / A.2 按 Profile 归组 / A.3 判定表） |
| IMP-09 | 验证带宽 = 采用预测器；人审机制设计 | P1 | §2.2、§19.7 | 新增说明 | 已应用（§2.2 一级约束说明、§19.7 新增第 12 条验收容量） |
| IMP-10 | 参考文献补全（CaMeL、Simplex、MemGPT、seL4 等） | P1 | 附录 C | 引用补全 | 已应用（附录 C.5 增补 19 条，保持分级纪律） |
| IMP-11 | 带外修改（file-as-truth）对账路径 | P1 | §5.3；agent-compatibility companion §4 | 新增语义 | 已应用并登记（§5.3、companion §4、`REQ-AGENT-OOB-001`、向量 AGENT-OOB-001） |
| IMP-12 | 已登记 tool proxy 的批量调用合法形态 | P1 | §5.3、§12.7 | 澄清说明 | 已应用（§5.3、agent-compatibility 与 operation-catalog companion 澄清） |
| IMP-13 | CIM 实现细节下沉 Heterogeneous companion | P2 | §14 | 精简迁移 | 已应用（§14.4—14.6 边界声明化，companion §6/§8a 承接） |
| IMP-14 | 意图澄清义务的 profile 化裁剪（消费级场景） | P2 | §6.9、§20.5 | 新增说明 | 已应用（§6.9 低风险档放宽注记） |
| IMP-15 | C2 生态差距的路线图承认与采纳激励 | P2 | §5.2、§21 | 新增说明 | 已应用（§5.2 现状承认与激励建议、§21 Phase 4 注记） |
| IMP-16 | 主文档瘦身与可读性 | P2 | 全文 | 长期编辑策略 | 持续执行（本轮 §14 下沉部分抵消新增；净增行数已在附录 D 交代） |
| IMP-17 | 开发计划资产漂移修正 | P2 | [History/CognitiveOS-Development-Plan.md](./History/CognitiveOS-Development-Plan.md) §1 | 事实修正 | 已过时——仓库资产已追平原表述（56 份 schema、`docs/standards/governed-object-contract.md` 已存在），且开发计划已归档，无需修正 |
| IMP-18 | Agent 性能提升路径与时机声明（informative） | P1 | §4 IMP-18 映射表；开发计划 §5 M6 验收 | 研判声明 + 验收补充 | 调整后应用：性能前提（IMP-02/03/04）已登记；M6 性能对齐验收因开发计划归档改挂白皮书 §21 Phase 1（A/B 对齐报告） |

---

## 4. 改进项详述

### IMP-01 实现优先原则写入路线图（P0）

**结论**：仓库现状为"规范体量大、实现证据零"（244 REQ / 40 schema / 65 向量 / 0 实现）。版本史 0.2→0.8 每版新增整个子系统，规范表面积增速远超验证速度；v0.4 版本说明自认曾纠正过四项过度设计，证明该风险已实际发生。这与 IETF"rough consensus and running code"及成功系统（Unix、seL4、Kubernetes）"最小核心先验证再扩张"的路径相反。

**建议改动**：
- §21"路线图原则"新增一条：**在单节点 R0/R1 参考实现（对应开发计划 M0—M6）交付并跑通现有 65 个向量之前，冻结新对象族、新 Profile、新 REQ 域的引入**；仅允许实现反馈驱动的修正型规范变更。
- §1.2 规范地图中注明各 companion 的"实现验证状态"维度（当前全部未经实现验证）。

**验收判据**：路线图原则含冻结条款；后续版本说明中新增规范对象须引用实现反馈或安全事故证据。

### IMP-02 渲染前缀稳定性（P0）

**结论**：兼容负载分析显示，对 OpenClaw/Hermes 类会话式负载，治理机制的机械开销（门禁 µs–ms 级、事件落盘 ms 级）相对模型推理（秒级）可忽略；**唯一能造成 2–10 倍成本/TTFT 退化的因素是供应商 prompt/KV cache 失效**。§9.4 的绑定语义（"绑定均未变化时可复用"）允许同 Conversation 内保持缓存命中，但前提是 Context 渲染器做到**前缀稳定**（分区顺序固定、对象版本未变时渲染结果字节级一致）。该前提目前仅是隐含实现自由，规范未作要求——C1 适配器在此翻车的概率极高。

**建议改动**：
- §9.3 第 8 阶段（render）补充："渲染必须是确定性的：相同输入对象版本、相同 target profile、相同 renderer 版本下，输出字节级一致；分区与条目排序必须稳定，避免破坏下游推理缓存的前缀复用。"
- §9.4 补充一句：缓存绑定语义的设计意图包含"同 Conversation 未变绑定下保持供应商缓存命中"，实现不得以逐 Activity 重排渲染的方式使其失效。
- 登记候选 REQ（如 `REQ-CTX-RENDER-STABILITY`，名称待规范流程确定）与候选向量：同一 ContextView 两次渲染 digest 一致；新增无关对象不改变已有条目相对顺序。
- 开发计划 M3 验收补充对应测试。

**验收判据**：§9.3/§9.4 含确定性渲染与前缀稳定表述；registry 中出现已分类登记的候选 REQ 与向量。

### IMP-03 准入异步化与 read-your-write（P0）

**结论**：OpenClaw 的记忆写是文件追加（即时），Hermes 的外部记忆同步是后台非阻塞。若 Memory adapter 将 `memory.add/update` 同步映射为"候选→（含模型调用的语义 lint）→准入"，写路径从毫秒级劣化到秒级，且压缩前的记忆 flush 会被拖慢。同时若准入异步化而无 read-your-write 保障，Agent 会出现"刚写的记忆读不到"的行为回归。

**建议改动**：
- §9.5 或 governed-memory companion 补充：**准入可以异步**；候选在准入完成前以"本 Activity/Conversation 私有工作集"形式对写入者立即可见（read-your-write），但不得跨 scope 泄露；准入失败时工作集内候选被标记并隔离。
- §18.3 知识编译同理：ingest 隔离区候选对提交者可见。
- agent-compatibility companion §4 Memory Adapter 条目注明该语义为 C1 适配的默认要求。

**验收判据**：规范明确"异步准入 + 写者即时可见 + 跨 scope 不可见"三要素；候选向量覆盖"未准入候选跨 Conversation 读取被拒"。

### IMP-04 治理开销性能契约（P0）

**结论**：§19.4 的指标族全面覆盖任务/安全质量，但**治理机制自身的开销没有指标**——这正是"规范太贵所以绕开"风险的量化盲区。

**建议改动**：§19.4 新增 **Governance overhead** 指标族（延续现有统计约定）：
- 门禁延迟：授权判定、Context 解析（不含模型/检索 I/O）、Effect 协议各阶段的 p50/p95/p99；
- 缓存保持率：治理绑定下供应商缓存命中率相对无治理基线的比值（cache-hit preservation ratio）；
- 每受治理调用的额外持久化写次数与字节；
- 审批延迟与审批吞吐（与 IMP-05 共用）；
- 治理开销占端到端延迟/费用的百分比，按 R0—R3 分层报告。

**验收判据**：performance-report schema 增补对应字段（按规范变更流程）；BenchmarkManifest 要求报告治理开销基线对比。

### IMP-05 审批子系统（P0）

**结论**（审批协议评审）：
1. **审批集中化为 OS 级服务是正确方向**，且填补白皮书留白：白皮书把"验证带宽/审批拥塞"列为稀缺资源（§2.2）但未给机制设计。集中队列才能实现批处理、去重、限流、风险路由与审批指标。业界已验证该形态（HumanLayer 全通道审批 API、Teleport 聊天审批插件）。
2. **审批请求路由进用户所在对话通道**（含插入 OpenClaw/Hermes 会话流）作为**通知面与低风险确认面**成立；作为高风险审批**完成面**不成立，会同时踩中三个已实证攻击面：
   - **主持人伪造**：同一聊天窗口内"OS 主持"只是文本声明，被注入的 Agent 可渲染像素级相同的假审批对话（Windows UAC secure desktop / secure attention sequence 所防御的正是此类）；
   - **审批疲劳轰炸**：MITRE ATT&CK T1621（Uber 2022 事故：高频推送 + 冒充 IT 劝批准）；
   - **凭证与回复截获**：聊天流中的密码进入 Agent 进程、会话存储与通道服务商；静态密码可重放、无法绑定 proposal（NIST SP 800-63B 将记忆型秘密列为最弱验证器）。
3. **自然语言定位**：NL 承担"协商与解释层"（询问、要求解释、修改提案范围→生成 `supersedes` 的收窄新 proposal），**批准行为本身**必须是对固定 proposal digest 的结构化动作——与 §12.3"自然语言批准可以触发 authority 签发 capability，自然语言本身不是 capability"完全一致。
4. **分级确认矩阵**（建议规范化）：

| 风险档 | 通知面 | 确认面与凭证 |
|---|---|---|
| R0 | 不打扰，事后可查 | 策略自动批准 |
| R1 | 插入 Agent 对话流（**OS 独立通道身份**发出） | 聊天内结构化确认：回显 proposal digest 短码（number matching 的对话版），一次性、限时、精确匹配 |
| R2 | 对话流仅通知 + 深链 | 离开对话通道：Shell 可信表面 / 系统级弹窗，passkey/FIDO2 对 proposal digest 签名（PSD2 动态链接原则：凭证密码学绑定审批内容） |
| R3 | 同上 | 独立 principal 双人审批 + step-up |

5. 关键机制约束：OS 必须持有**独立于被审 Agent 的通道身份**（独立 bot/号码）；聊天内审批不建立常驻 `PrivilegedManagementSession`，每次为一次性 `ManagementApprovalDecision`；防疲劳机制（同类聚合、频控、连续拒绝升级告警）。

**建议改动**：
- §12.10 扩展或新增小节"审批分发与确认强度"，纳入上述矩阵与机制约束；明确"可协商审批"语义（NL 修改产生新版本 proposal）。
- §17.1 威胁模型增补：**approval bombing（审批轰炸）**与**审批界面伪造/主持人冒充**两类威胁；§17.2 对应防线注明"审批确认面必须渲染在被审对象不可写入的表面，或使用被审对象无法预知的一次性挑战"。
- §19.4 增补审批健康指标：审批延迟分布、橡皮图章率（批准延迟 < 阈值的比例）、拒绝后重试率。
- §23 新增决策条目（候选表述）："审批是 OS 服务：请求可全通道分发，确认强度按风险分级，自然语言可协商提案但不构成批准行为本身。"
- 候选向量：假主持人卡片无有效短码被拒；同一 proposal 高频重发触发限流；R2 审批在对话通道内回复"同意"不生效。

**验收判据**：上述四处修订落地；候选向量登记；与 §12.8/§12.9 现有隔离语义无冲突（本评审已确认方向一致）。

### IMP-06 R0"薄路径"（P1）

**结论**：§4.7 最小闭环含 10 个组件，对 R0 个人助理/检索场景显著重于业界可用基线（文件 + git + 进度日志式 harness）。若低风险档位实现成本远高于朴素方案，实践者将绕开规范——安全规范被绕开的主因从来是成本。

**建议改动**：§20.4/§20.5 增补"R0 最小合法实现形态"说明：明确 R0 下事件日志可为本地追加文件、Context 门禁可退化为确定性选择、TaskContract 可为静态模板、验收可按策略自动化等合法降级映射，并给出"哪些边界即使在 R0 也不可降级"（tenant/scope 隔离、检索前过滤、注入内容不得提升为控制）。§4.7 注记指向该说明。

**验收判据**：R0 实现者无需阅读全部 Core 即可从该节得到合法最小组件表。

### IMP-07 形式化验证承诺（P1）

**结论**：架构可信度建立在"确定性门禁"上，而五状态机 × 开放 authority 域 × Loop 门禁 × Shell 状态的组合状态空间只有 JSON 向量抽样覆盖。Effect + 恢复 + fencing 是最值得形式化的子集（状态空间小、安全收益大）；seL4 已证明小内核形式化的可行性。

**建议改动**：§21 路线图（建议挂 M4 附近）新增"Effect/恢复/fencing 状态机的 TLA+ 或 Alloy 模型与不变量证明"交付项；§19.5 符合性层级注明形式模型可作为第 3、5 层的补充证据；§22 开放问题可相应收敛。

**验收判据**：路线图含形式化交付项及其范围（明确不承诺全系统形式化）。

### IMP-08 对象本体分层（P1）

**结论**：全文约 80 个命名对象，其中 Context/Binding/Scope 族（TenantContext、GovernanceDomainContext、ExecutionContext、ActivityContext、ConversationBinding、AgentExecutionBinding、ResourceScope、ShareGrant）语义区分细微，实现者混用几乎必然。

**建议改动**：附录 A 重组为两层：**最小核心对象集**（建议 ≤ 15 个：AgentExecution、Task、TaskContract、Intent、Effect、Event、StateSnapshot、ContextRequest/View、AuthorizationCapability、OperationDescriptor、Checkpoint、Principal、TenantContext、ResourceScope 量级）与**扩展对象集**（其余按 Profile 归组）。§4.7 责任矩阵引用最小集。对 Context/Binding/Scope 族增加一张"何时用哪个"的判定表。

**验收判据**：新读者可在附录 A 首屏得到最小对象宪法；每个扩展对象标注所属 Profile。

### IMP-09 验证带宽与人审机制（P1）

**结论**：完成语义依赖"独立 verifier 对固定后态给出证据"。可确定验证的域（代码+测试、交易+对账）收益极高；开放式知识工作缺确定性 verifier，完成判定退化为人工验收，而人审吞吐与疲劳未获机制设计。**架构的采用价值分布由目标域的可验证性决定**——这应成为明示的部署指引而非隐含事实。

**建议改动**：
- §2.2 或 §19.7 增补一段"验证带宽是部署的一级约束"：建议部署方在 Readiness Case 中声明验收吞吐预算（人审容量、verifier 容量）并据此选择任务域。
- §19.7 增补人审机制建议：风险分层抽样复核、批处理审批、审批 SLA 与超时升级、橡皮图章率监控（与 IMP-05 指标共用）。

**验收判据**：Readiness Case 模板含验收容量条目。

### IMP-10 参考文献补全（P1）

**结论**：白皮书多项设计的学术支撑强于其引用表所示。建议按附录 C 现有分级补充：

| 建议新增引用 | 支撑章节 | 建议等级 |
|---|---|---|
| Debenedetti et al., *Defeating Prompt Injections by Design*（CaMeL），arXiv:2503.18813 | §9.2、§17.3 | [PRE]/[PEER]（以发表状态为准） |
| Sha, *Using Simplicity to Control Complexity*（Simplex 架构）, IEEE Software 2001；及 Run-Time Assurance 综述 | §4.2、§13 | [PEER] |
| Packer et al., *MemGPT: Towards LLMs as Operating Systems*, arXiv:2310.08560 | §9（CVM） | [PRE] |
| Klein et al., *seL4: Formal Verification of an OS Kernel*, CACM；Heiser & Elphinstone, *L4 Microkernels: The Lessons from 20 Years* | §7 | [PEER] |
| Sumers et al., *Cognitive Architectures for Language Agents*（CoALA），arXiv:2309.02427 | §6、§10 | [PRE] |
| Pang et al.（Google），*Zanzibar: Google's Consistent, Global Authorization System*, USENIX ATC 2019；AWS Cedar 语言 | §12.3 | [PEER]/[IND] |
| Gray & Cheriton, *Leases: An Efficient Fault-Tolerant Mechanism…*, SOSP 1989；Kleppmann fencing token 论述 | §16.3 | [PEER]/[IND] |
| IETF draft-oauth-ai-agents-on-behalf-of-user；SPIFFE/WIMSE 文档；RFC 8693 | §6.2、§12.3 | [STD]（草案标注 Draft） |
| NIST SP 800-63B；CISA phishing-resistant MFA / number matching 指南；MITRE ATT&CK T1621；PSD2 SCA 动态链接（EBA RTS） | §12.8—§12.10、§17（配合 IMP-05） | [STD] |
| Temporal saga/幂等实践文档 | §12.5、§16 | [IND] |

**验收判据**：附录 C 收录以上条目并保持分级纪律。

### IMP-11 带外修改对账路径（P1）

**结论**：OpenClaw 类 Agent 的哲学是"文件即真相"（用户可直接手改 MEMORY.md）。C1 适配后文件成为受治理对象的投影，带外编辑会造成双源分歧。白皮书有 reconcile 原语但未描述该场景。

**建议改动**：agent-compatibility companion §4（Memory/Sandbox Adapter）增补：workspace 文件的带外修改按"外部观察"处理——检测（digest 漂移）→ ingest 为候选 → 按策略准入或标记冲突；不得静默覆盖任何一侧。§5.3 提及该要求。

**验收判据**：候选向量覆盖"带外编辑后首次读取触发对账而非静默采用"。

### IMP-12 已登记 tool proxy 模式（P1）

**结论**：Hermes 的"脚本经 RPC 批量调工具"模式高效且应保留，但 §5.3 仅以"tool proxy 绕过被拒"一笔带过，未说明**合法形态**。

**建议改动**：§5.3 或 operation-catalog companion 增补："批量/脚本式工具调用必须经**已登记的 proxy 端点**（即 Tool Adapter 本身暴露的批量接口），逐调用保留授权与审计，批量性只体现在传输与上下文成本上，不体现在门禁豁免上。"

**验收判据**：实现者能区分"非法 tool proxy 绕过"与"已登记批量代理"。

### IMP-13 CIM 细节下沉（P2）

**结论**：§14.4—§14.6（CIM 误差分解、校准、编译栈细节）学理正确（与 2025–26 CIM 可靠性研究一致）但与治理主线耦合最弱，是主文档中最可整体下沉的内容。

**建议改动**：§14 保留 ResourceGraph、数据驻留优先、误差预算作为调度输入的**边界声明**（各约一段），实现细节移入 Heterogeneous companion。

**验收判据**：§14 篇幅约减半，无语义丢失（companion 承接）。

### IMP-14 意图澄清的 profile 化裁剪（P2）

**结论**：§6.9"存在实质歧义必须澄清"与消费级 UX 存在张力；个人场景下验收 authority 塌缩回用户本人，部分机制成为仪式。

**建议改动**：§6.9 或 §20.5 R0 行注明：低风险档位允许"默认解释 + 事后可纠正"策略（澄清义务收敛为预览义务），高风险/不可逆边界不受此放宽。

**验收判据**：R0 实现无需逐次澄清亦可合规。

### IMP-15 C2 生态差距（P2）

**结论**：兼容负载分析显示成熟 Agent（OpenClaw、Hermes）现实落点为 C0/C1；C2 所需的 checkpoint/cancel/pending-effect 暴露在业界普遍缺失。这是路线图风险：C2 采纳无外部动力。

**建议改动**：§5.2 或 §21 增补现状承认与采纳激励设计（如：C2 换取更高自动化配额/更低审批频度的策略建议，作为 informative 部署建议）。

**验收判据**：路线图对 C2 的预期不再隐含"生态自然到位"假设。

### IMP-16 主文档瘦身（P2）

**结论**：主文档 1776 行/约 173KB，体量与其"注意力稀缺"论点构成自反。IMP-08（本体分层）与 IMP-13（CIM 下沉）是主要抓手；此外 §12.8—§12.11（Shell 相关四节）可考虑收敛合并。

**建议改动**：作为长期编辑策略执行，不单独立项；每次修订净增行数应有理由。

### IMP-17 开发计划资产漂移修正（P2）

**结论**（事实核查）：[History/CognitiveOS-Development-Plan.md](./History/CognitiveOS-Development-Plan.md) §1 声称"已有 56 份 JSON Schema"并称治理对象族"已由 docs/standards/governed-object-contract.md 登记 v0.1 machine schema"。评审当日实测：仓库 schema 为 **40 份**；**`docs/` 目录不存在**，该文件亦不存在（属 M0 计划产物被误写为现状）。**v1.2 复核**：仓库其后已追平——schema 达 56 份、`docs/standards/` 四份标准与 `docs/adr/` 均已建立，本项失去事实基础且开发计划已归档，判定为已过时、不执行。

**建议改动**：修正 §1 表述——schema 数量改为实测值；`docs/standards/governed-object-contract.md` 改为"计划于 M0 建立"；同时核对白皮书 v0.8 版本说明中"governed-object-contract v0.1 Draft 已登记"的表述与仓库实际状态是否一致，不一致则同步修正。

**验收判据**：两份文档中关于资产存量的陈述与 `Glob`/计数实测一致。

### IMP-18 Agent 性能提升路径与时机声明（P1，informative）

**问题**：在 OS 上安装 OpenClaw、Hermes、Codex 类成熟 Agent 后，除治理能力外，能否显著提升其运行性能（记忆/工具/技能调取准确度、任务完成准确率、token 经济性、多 Agent 协作）？是否需要为此对 OS 基建进行统一规范？该规划属于当前架构设计阶段还是后期？

**研判结论**：

1. **不需要新建"性能基建层"**。五类性能目标的承载基建已在现有架构系统性存在（Governed Memory、Cognitive Discovery、Operation Catalog、SMS/CRB、§19.4 性能契约与 `performance-report.schema.json`），只是以"治理优先"叙事写成，性能是第二叙事。性能提升与治理不是两套基建：五个目标全部落在治理层已定义的同一批对象上（MemoryObject、OperationDescriptor、ContextView、TaskContract、Delegation）。
2. **当前阶段（M0 契约冻结期）要定的不是"怎么优化"，而是"性能前提的契约形状"**。IMP-02（渲染前缀稳定）、IMP-03（异步准入 + read-your-write）、IMP-04（治理开销指标）正是这一类——属于"现在不定、实现后再改就是破坏性变更"的接口语义。兼容负载分析（§1 第 2 项）表明：适配负载的首要性能风险是**劣化**而非"缺少提升"——同步准入使毫秒级记忆写劣化到秒级并引发"刚写的记忆读不到"回归；逐 Activity 重排渲染使供应商缓存失效、成本/TTFT 退化 2–10 倍。若这三项不落定，"装上 OS 的 Agent 比裸跑更贵更慢更健忘"将成为首次接触体感，性能提升叙事在源头破产。
3. **优化的主体应后置且永不进 Core**：检索排序、embedding/reranker 选型、consolidation、压缩、模型路由、协作拓扑属于策略层，按既有路线图在 M7/M8（Phase 2–3）凭实现证据交付，以 §19.4 指标验收。现在写入规范：无基准数据（244 REQ / 0 实现，违反自身证据纪律）、违反决策九机制/策略分离、会被季度级演化的业界实践淘汰、并与 IMP-01 冻结条款直接冲突。
4. **判定准绳**（供后续同类问题复用）：凡影响对象/适配器契约形状与确定性语义的性能前提 → 当前阶段定型；凡可插拔、靠指标衡量、随实践快速演化的优化 → 后期策略层；性能的**度量语义**本身是契约 → 当前阶段定型。

**五目标 → 承载机制 → 提升来源 → 缺口与时机映射**：

| 性能目标 | 已有承载基建 | 提升的机制来源 | 缺口与时机 |
|---|---|---|---|
| 记忆调取准确度 | Governed Memory 九类 kind + valid time/冲突/置信语义；§9.9 InformationGap/delta/停滞检测 | 结构化召回（按 kind、时效、冲突过滤）替代平文件 grep；多轮 delta 解析带边际增益核算 | 检索算法与 consolidation 是 SMS 策略（M7/M8）；当前只需 IMP-03 保证写路径不劣化 |
| 工具/技能调取准确度 | §12.7 两级目录（Summary/Descriptor）+ 类型/效果过滤 + OperationMatchReport；§19.4 已有 top-1/top-k、effect-class confusion 指标 | 确定性预过滤 + 授权候选内语义排序，替代"全部工具描述塞进上下文" | 匹配后端在 M8；Descriptor 撰写质量指南属生态建设，待 M8 匹配器就绪后以 informative 形式补充 |
| 任务完成准确率 | TaskContract + 五状态机 + 独立 Verification + 有界 Loop/停滞检测（§10） | 把"自称完成"变成"验证完成"，靠拒绝伪完成和有界诊断重试提高真实完成率 | 已在 Core（M4/M5 实现）；开放域缺确定性 verifier 是部署约束，见 IMP-09 |
| Token 经济性 | §9.4 缓存绑定语义、§9.7 注意力预算、§9.6 有损声明、两级目录省 token | **最大杠杆是供应商 KV/prompt cache 保持**：治理机械开销 µs–ms 级可忽略，缓存失效才是 2–10 倍成本/TTFT 退化源 | IMP-02 是唯一缺失的规范性前提（P0，当前批次）；压缩/路由策略后置 |
| 多 Agent 协作 | §15 委派契约（capability 衰减、子预算、verifier）+ 共享权威 Task 状态 + AKP/A2A 网关 + ConflictSet（M9） | 共享**权威**状态替代文件式 Kanban；独立验证与委派衰减压制 MAST 类失败传播 | 协作拓扑是策略（V10：多 Agent 收益常为边际，不应过度承诺）；§19.4 缺"协作效率"指标族，见下文候选占位 |

**建议改动**：

- 现在（并入当前修订批次，均为既有 IMP 项，从性能视角确认其优先级）：IMP-02 → 白皮书 §9.3/§9.4；IMP-03 → §9.5/§18.3 与 governed-memory / agent-compatibility companion；IMP-04 → §19.4 与 performance-report schema（含缓存保持率 cache-hit preservation ratio）。均按规范变更流程登记候选 REQ/向量。
- 现在（本项新增的唯一动作）：为适配交付增加**"适配 Agent（legacy reference agent）vs 原生裸跑基线的 A/B 性能对齐报告"**验收——以 BenchmarkManifest 固定输入，报告任务成功率、延迟、token/费用与缓存命中对比。（v1.2 注：原目标为开发计划 §5 M6 验收；开发计划归档后改挂白皮书 §21 Phase 1，已应用。）
- 后置（按既有路线图，不提前）：检索排序 / embedding / consolidation / 压缩 / 模型路由留在 M7/M8 的 SMS/CRB 可插拔策略层；**多 Agent 协作效率指标族**（协调开销、重复工作率、冲突率、委派深度效率）作为 Phase 5/M9 前的候选登记项，本文仅占位，不登记 REQ；OperationDescriptor 撰写质量指南待 M8 后补充。
- 不做：不新建"性能优化"规范域或新对象族（遵守 IMP-01 冻结）；不为性能开绕过治理面的旁路路径（语义隔离与认知资源中介两条不变量禁止）——性能必须从受治理表面内部长出来。
- 诚实边界（建议写入部署指引）：OS 能提升的是系统层性能（检索准确度、伪完成拦截、token 经济、协作失败抑制），提升不了模型本身的语义质量；缺乏确定性 verifier 的开放域，"任务完成准确率"提升受验证带宽约束（IMP-09），属部署选型问题而非基建问题。

**验收判据**：开发计划 M6 验收含性能对齐 A/B 报告条目；后续任何"性能基建"提案先对照本项判定准绳与映射表分类，避免重复研判；IMP-02/03/04 的登记与实施仍按各自条目验收。

---

## 5. 与开发计划的衔接

> **v1.2 归档注记**：开发计划已移入 [History/CognitiveOS-Development-Plan.md](./History/CognitiveOS-Development-Plan.md)，按 History 目录约定为冻结历史制品。本节衔接表保留为历史规划参照；实现启动时以白皮书 §21 路线图（1.0.0 已含冻结原则、Phase 1 性能对齐与 Phase 4 形式化交付）为当前有效依据，届时可参照本表恢复里程碑级衔接。

开发计划 v1.0 已确立"实现优先、原地转型单仓库、M0—M6 交付单节点 R0/R1"，与 IMP-01 方向一致。具体衔接点：

| 改进项 | 开发计划衔接 |
|---|---|
| IMP-01 | 已被 M0—M6 隐含采纳；仍需回写进白皮书 §21 使两文档一致 |
| IMP-02 | M3（Context Resolution）验收标准补充确定性渲染/前缀稳定测试 |
| IMP-03 | M7（Governed Memory）交付与验收补充异步准入 + read-your-write |
| IMP-04 | M6 发布的性能基线与 M1 runner 证据格式中纳入治理开销指标 |
| IMP-05 | M5 范围目前含"Management API + 确定性 CLI + 普通任务 Shell"；审批分发子系统建议作为 M5 的显式交付项或 M6 前的独立小里程碑；注意与开发计划"自然语言 Intelligent Shell 首版保持 disabled/experimental"的约束一致——**分级确认矩阵中 R1 的聊天内结构化确认不依赖 NL 解析，可先行交付** |
| IMP-07 | 建议挂 M4（Effect/恢复）交付物：形式模型与不变量证明 |
| IMP-11、IMP-12 | M6（C1/C2 适配）交付与负例范围补充 |
| IMP-17 | M0 之前即可修正 |
| IMP-18 | M6 验收增加"适配 Agent vs 原生基线 A/B 性能对齐报告"；性能优化主体维持 M7（记忆/发现）与 M8（目录匹配/SMS/CRB）既有挂点，不提前；多 Agent 协作效率指标族在 M9 启动前按规范变更流程登记候选 |

---

## 6. 建议实施顺序

> **v1.2 执行状态**：三批修订已在白皮书 1.0.0 定稿中一次性完成（IMP-17 判定过时跳过；IMP-05 采纳文本、REQ/向量登记递延独立草案；IMP-16 转为长期纪律）。§2 基线经核对未被削弱；新增 REQ/向量已同步 registry 与 conformance 资产。

1. **第一批（纯文档修正，低风险）**：IMP-17 → IMP-10 → IMP-01 → IMP-09 → IMP-14 → IMP-15 → IMP-18（研判声明与 M6 验收补充部分）。
2. **第二批（新增规范语义，需按规范变更流程分类登记）**：IMP-02 → IMP-03 → IMP-04 → IMP-06 → IMP-11 → IMP-12（IMP-18 从性能视角确认前三项优先级，不新增规范对象）。
3. **第三批（结构性修订）**：IMP-05（审批子系统，建议先出独立草案再并入）→ IMP-08 → IMP-13 → IMP-07。
4. IMP-16 贯穿执行。

每批完成后核对 §2 基线未被削弱；涉及候选 REQ/向量的项须同步 registry 与 conformance 资产，遵守"不虚构已登记资产"纪律。

---

## 附录：本轮评审新引入的外部证据

按白皮书附录 C 的分级约定：

- `[PEER]` Cemri et al., *Why Do Multi-Agent LLM Systems Fail?*（MAST）, NeurIPS 2025, arXiv:2503.13657 — 白皮书已引用，本轮复核其失败分类作为 V3/V10 证据。
- `[PRE]` Debenedetti et al., *Defeating Prompt Injections by Design*（CaMeL）, arXiv:2503.18813 — V7 直接同构证据。
- `[PRE]` Packer et al., *MemGPT*, arXiv:2310.08560 — CVM 先行工作。
- `[PEER]` Klein et al., *seL4*, CACM 2010；Heiser & Elphinstone, *L4 Lessons*, 2016 — V9 证据。
- `[PEER]` Sha, *Using Simplicity to Control Complexity*, 2001 — V8 证据。
- `[STD]` IETF draft-oauth-ai-agents-on-behalf-of-user（Draft）；SPIFFE/WIMSE；RFC 8693 — V13 证据。
- `[STD]` NIST SP 800-63B；CISA number matching / phishing-resistant MFA；MITRE ATT&CK T1621；PSD2 SCA 动态链接 — IMP-05 证据。
- `[IND]` Temporal 文档（saga、幂等键、durable execution）— V4/V5 证据。
- `[IND]` Anthropic, *Effective context engineering for AI agents*；*Effective harnesses for long-running agents* — V6 与 §10 证据（白皮书已引用后者）。
- `[IND]` HumanLayer（全通道人审 API）；Teleport 聊天审批插件 — IMP-05 业界形态证据。
- `[IND]` OpenClaw 文档（Gateway/workspace/memory/skills/cron/heartbeat/session 存储）；Hermes Agent 仓库与文档（记忆双层、外部记忆插件、技能自改进、RPC 批量工具调用、Kanban 多智能体）— §1 第 2 项分析的事实基础。
- `[PEER]` Khwa et al., *A mixed-precision memristor and SRAM compute-in-memory AI processor*, Nature 639 (2025) — 白皮书已引用，本轮核验真实性；配合 2025–26 CIM 可靠性研究（尾部鲁棒性、自校准）作为 IMP-13 的"学理正确但宜下沉"判断依据。
- `[PRE]` *AI Harness Engineering*, arXiv:2605.13357 — 白皮书已引用，本轮核验真实性。
