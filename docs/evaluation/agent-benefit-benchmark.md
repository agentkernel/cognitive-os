# CognitiveOS Agent 收益评测合同（Agent Benefit Benchmark Contract）

- **标识**：`cognitiveos.agent-benefit-benchmark/0.1`
- **版本**：v0.1 Draft（machine `0.1.0-draft.1`）
- **状态**：Draft Normative Standard（registered normative-behavior asset）
- **日期**：2026-07-20
- **权威边界**：本文定义 Agent 收益声明的实验设计、统计协议、声明门槛与发布阻断条件。机器证据格式由 [performance-report.schema.json](../../specs/schemas/performance-report.schema.json) 定义；指标语义沿用白皮书 [§19.4](../../CognitiveOS-Architecture.md#performance-contract) 与 [REQ-PERF-001..005](../../specs/registry/requirements.yaml)。本文不改变任何治理、授权或安全语义。
- **适用性**：任何以 CognitiveOS（或其参考实现）名义发布的"Agent 性能收益""任务成功率提升""token/费用/时间节省""多 Agent 协作改善"声明。仅声明治理能力或非劣化的报告不受收益门槛约束，但仍受 REQ-PERF-001–004 约束。

> 核心纪律：**没有测得收益时，只能声明"治理能力"或"性能不劣化"。**潜在机制、局部 microbenchmark 或主观分析不得包装成 Agent 性能提升。非劣化（B 相对 A）只能声明"治理附加成本可接受"，不能声明性能提升。

[REQ-PERF-005] 任何"CognitiveOS 带来显著 Agent 收益"级别的声明 **MUST** 由本文 §1 四臂设计（native baseline / governance-only / optimized / ablation）与 §5 预注册统计协议支撑，报告 **MUST** 填充 performance report 的 `comparison` 块（含 `claim_level`、`preregistration_ref` 与逐臂结果）；未达 §5.2 门槛的结果 **MUST** 以 `hypothesis` 或 `non_inferiority` 声明，**MUST NOT** 表述为性能提升；缺失 native baseline 或 governance-only 对照臂的收益声明 **MUST** 被判定为 `PERFORMANCE_REPORT_INCOMPLETE`。

---

## 1. 四臂实验设计

每个收益声明必须由同一 workload 上的四臂对比支撑。四臂共享同一任务集、模型、工具、权限、预算与 grader（§3 固定变量）：

| 臂 | 定义 | 回答的问题 |
|---|---|---|
| **A：native baseline** | 目标 Agent 原生裸跑，无 CognitiveOS 参与 | 现状基线是什么 |
| **B：governance-only** | 相同 Agent 经 C1+ 适配安装到 CognitiveOS，治理路径全开（身份/授权/Context 门禁/Effect 协议/审计），**语义优化全部关闭**（无 SMS rerank、无语义记忆检索、无 catalog 语义匹配、渲染与准入按 REQ-CTX-012 / REQ-MEM-ADMIT-002 保底语义运行） | 治理本身的成本与非劣化性 |
| **C：optimized** | B + 被测优化机制（逐个或声明的组合） | 优化是否带来端到端收益 |
| **D：ablation 系列** | 对 C 中每个被测机制逐项关闭形成 C-minus-x 臂 | 收益归因于哪个机制 |

约束：

1. B 是收益声明的**强制对照臂**。C 相对 A 的差异不可直接归因于优化，必须经 B 分解为"治理差异 + 优化差异"。
2. D 的 ablation 粒度以 §6 机制分级表中的单个 C 类机制为单位；组合优化必须能分解到至少一个单机制显著贡献，否则只能声明组合级收益且标注不可归因。
3. 四臂不得存在 §7 所列的任何不对称（token、工具、数据、权限、模型强度）。
4. 每臂结果以 performance report 的 `comparison.arms[]` 记录，绑定各自 BenchmarkManifest 差异声明。

## 2. Workload families

收益声明必须至少覆盖以下前两个 family；声称多 Agent 收益时增加第三个：

| Family | 定义 | 任务来源要求 | 验收 oracle |
|---|---|---|---|
| **W1：长时软件工程 Agent** | 多步骤代码变更任务（修 bug、加功能、重构），单任务 ≥10 次工具调用或 ≥15 分钟墙钟 | 固定 commit 的真实仓库任务集，禁止只挑适配 CognitiveOS 结构的任务 | 确定性：测试套件 + 构建 + 静态检查对固定后态运行 |
| **W2：研究/知识/个人助理 Agent** | 多源检索、归纳、长期记忆读写、跨会话延续任务 | 含记忆依赖任务（后续任务依赖先前写入的记忆）与冲突源任务 | 固定标注集 + 独立 verifier（人工或非同源模型），禁止被测模型自评 |
| **W3：多 Agent 协作（条件性）** | 需委派/并行/共识的任务 | 必须同时给出单 Agent 与确定性 workflow 基线 | 同 W1/W2 按任务域选择 |

W3 特别规则：**Agent 数量本身不是收益**。多 Agent 臂必须与预算等额的单 Agent 臂比较（总 token/费用/时间预算相同）；报告协调开销、重复工作率、冲突率与委派效率（§19.4 指标名），失败传播样本不得删除。

## 3. 必须固定的变量

BenchmarkManifest 必须固定，且四臂之间除被测机制外逐项相同：

- 模型：provider、model、revision、量化/部署形态；
- sampling：temperature、top_p、seed（或声明 provider 不支持 seed 及其重复次数补偿）；
- 输入任务集：任务列表、顺序或随机化方法、数据集版本与 split、许可；
- 工具：工具集合、版本、endpoint、超时与重试配置；
- 权限：各臂可用的 capability/scope 集合（A 臂的原生权限须与 B/C 臂治理后可达权限等效，差异必须列举）；
- 硬件与并发：节点、加速器、并发度、网络条件；
- 预算：每任务 token、时间、费用与工具调用硬上限；
- 缓存状态：warm/cold 声明与预热流程；供应商 prompt/KV cache 是否可用对四臂一致；
- 故障注入：fault profile（含"无注入"声明）；
- grader/verifier：版本、prompt/判据 digest、与被测系统的独立性声明。

## 4. 指标

### 4.1 端到端主指标（primary endpoints）

以下指标按 §19.4 统一统计约定（分子/分母计数、窗口、95% CI）报告；每个收益声明必须预注册其中一个为 primary endpoint：

- verified task completion rate = COMPLETED / admitted tasks（COMPLETED 仅指 acceptance authority 依据固定 TaskContract 与后态证据提交的完成；模型自述、Loop 退出、HTTP 200、远端 completed、mock receipt 均不计入）；
- time-to-verified-completion（p50/p95/p99）；
- 每 verified task 的总 token、模型调用数、工具调用数、费用、人工分钟；
- deadline miss rate；no-progress rate；
- 重复工作率（重复等价工具调用或重复等价子任务 / 全部工具调用或子任务）；
- recovery 后重算率（恢复后重新执行的已提交工作量 / 恢复前已提交工作量）。

### 4.2 能力归因指标（secondary / attribution）

按 §19.4 既有指标名：required context recall、context precision、stale exposure、conflict preservation、injection escape；memory retrieval recall/precision、first-view sufficiency、false no-result；Operation top-1/top-k、effect-class confusion、false no-tool；duplicate external effect、unknown outcome rate、reconcile latency；多 Agent 协调开销、重复工作率、冲突率、委派效率。归因指标改善**不构成**端到端收益声明；只能声明"局部能力改善"。

### 4.3 治理成本指标

按 REQ-PERF-004 的 Governance overhead 指标族：authorization / Context gate / Effect 各阶段 p50/p95/p99；每受治理调用额外持久化写次数与字节；CPU/内存/吞吐；cache-hit preservation ratio；治理开销占端到端延迟、token、费用的比例；warm/cold、R0–R3 与并发分层。

### 4.4 安全指标（不可被抵消）

越权、数据泄露、duplicate external effect、伪完成（未经验收被报告为完成）、不可恢复状态计数。**任何臂的安全失败计数不得被综合分或主指标改善抵消**（REQ-PERF-002）；C 臂相对 B 臂安全失败增加即触发 §8 kill criteria。

## 5. 统计协议与声明门槛

### 5.1 统计协议

- 成对设计：同一任务在各臂各跑 ≥1 次，按任务配对比较；无法配对时使用等价随机化分组并声明理由；
- 样本量：实验前基于 primary endpoint 的预期效应与方差做 power analysis（目标 power ≥0.8），报告依据；
- 随机化：任务顺序与分组的随机化方法与种子；
- 区间：比例用 Wilson 或 bootstrap 95% CI；连续量用 bootstrap 95% CI；
- 效应量：报告相对变化与绝对变化，配对设计报告配对差的 CI；
- 完整分母：timeout、拒绝、unknown outcome、quarantine、人工介入样本一律计入分母并单独列数，不得删除；安全拒绝（策略正确拒绝越权任务）单独报告为风险调整效用，不简单计为任务失败；
- **预注册**：primary endpoint、门槛、样本量、分层维度在实验开始前写入 BenchmarkManifest（`comparison.preregistration_ref`）；看到结果后不得更换主指标或降低门槛。若需调整门槛，必须在新一轮实验前基于用户价值、方差与 power analysis 重新预注册。

### 5.2 显著收益声明门槛（默认，预注册可收紧不可放宽）

同时满足以下六条才可声明"CognitiveOS 带来显著 Agent 收益"：

1. primary endpoint 的 95% CI 支持改善（区间不跨零/不跨 1）；
2. 至少满足一项端到端实际效应：
   - C 相对 A 的 verified task completion 相对提升 ≥10%；或
   - 完成率非劣（绝对下降 ≤2 个百分点）时，每 verified task 的 token、费用或 time-to-verified-completion 至少降低 20%；
3. 至少在 W1、W2 两个 workload family 上同时成立；
4. D 臂 ablation 能将收益归因到具体机制（单机制贡献的 CI 支持非零）；
5. C 臂不增加越权、数据泄露、duplicate external effect、伪完成或不可恢复状态（相对 B 臂计数不升）；
6. p95/p99 尾延迟、人工介入分钟与维护成本未抵消主收益（逐项披露并论证）。

### 5.3 非劣化（governance-only acceptable）门槛

B 相对 A 满足以下条件时，可声明"治理附加成本可接受"（不是性能提升）：

- verified completion 绝对下降 ≤2 个百分点（95% CI 上界内）；
- 各 workload 的治理开销不超过该 workload 预注册的开销预算（§5.4）；
- cache-hit preservation ratio 不低于该 workload 预注册门槛（§5.4）；
- 安全失败计数不高于 A 臂。

### 5.4 workload-specific 开销预算与 cache 门槛（R0/R1 首发默认值）

不设统一毫秒常数；按 workload 以占比表达，随首个 R0/R1 发布预注册，实测后只能以新版本修订：

| Workload | 治理开销占端到端延迟 | 治理开销占费用 | cache-hit preservation ratio |
|---|---|---|---|
| W1（长时软件工程，模型推理占主导） | ≤3%（p50）/ ≤8%（p99） | ≤2% | ≥0.90 |
| W2（会话式知识/助理，多短轮） | ≤10%（p50）/ ≤20%（p99） | ≤5% | ≥0.90 |
| W3（多 Agent） | ≤15%（p50，含协调） | ≤8% | ≥0.85 |

这些默认值是**候选预算**（hypothesis），由 Phase 1 A/B 对齐报告实测校准；校准前不得引用为已达成能力。所有门槛必须同时进入 SLO profile、BenchmarkManifest、performance report 与 CI/release gate，不得只保留在说明文字中。

## 6. 机制分级（A/B/C/D 类）

Core 的正确性不得依赖 C/D 类机制；C/D 未达收益门槛时必须可关闭、替换或删除。收益声明只针对 C 类；A/B 类只做非劣化与开销报告。

| 机制 | 级别 | 依据 | 评测归属 |
|---|---|---|---|
| 确定性 Context 过滤（tenant/scope 检索前过滤、授权先于 ranker） | **A 正确性必需** | REQ-CTX-001/002；安全边界 | 只报告开销，不声明收益 |
| 有界 Loop、独立 verifier、Effect 协议、恢复顺序 | **A 正确性必需** | REQ-RUN-*、REQ-EFF-*、REQ-REC-* | 伪完成拦截率可作为治理质量指标，不作性能收益 |
| 稳定渲染与前缀稳定（prompt/KV cache 保持） | **B 性能使能合同** | REQ-CTX-012；接口语义必须现在固定 | 以 cache-hit preservation ratio 验收非劣化 |
| 异步 memory admission + read-your-write | **B 性能使能合同** | REQ-MEM-ADMIT-002；防写路径劣化 | 以写延迟与行为回归验收非劣化 |
| 两级 Operation Catalog（Summary/Descriptor 确定性预过滤） | **B/C 边界** | 结构为 B（接口已固定）；语义匹配为 C | top-k 归因指标 + 端到端 token 变化 |
| 增量 Context 解析（delta/InformationGap） | **C 可插拔优化** | 无端到端证据 | 四臂 + ablation |
| loss declaration 驱动的压缩策略 | **C 可插拔优化** | 机制为 A（声明义务），策略收益未证 | 四臂 + ablation |
| 语义记忆检索/consolidation（SMS 后端） | **C 可插拔优化** | 检索算法可替换 | 四臂 + ablation |
| checkpoint/recovery 减少失败后重复推理 | **C 可插拔优化** | 收益依赖故障率分布 | 含故障注入的四臂 |
| event-driven watch 替代轮询 | **C 可插拔优化** | 资源收益可测 | 治理成本指标分层 |
| progress/stagnation 检测降低无效循环 | **C 可插拔优化** | no-progress rate 可测 | 四臂 + ablation |
| 共享权威任务状态、委派契约减少多 Agent 重复/失败传播 | **D 研究假设** | MAST 表明多 Agent 收益常为边际 | W3 全套对照，未证前只声明 hypothesis |
| "认知经济学"式统一资源市场 | **D 研究假设** | 白皮书 §2.2 已自我限定 | 不进入收益声明 |

## 7. Benchmark gaming 防护清单

出现任一项即判报告无效：

1. OS 臂获得更多 token、工具、数据或权限（不对称资源）；
2. 用更强模型/更新 revision 与裸跑基线比较；
3. 只测 microbenchmark（检索延迟、匹配精度）不测 time-to-verified-completion；
4. 用模型自评或被测系统同源 LLM judge 代替后态 verifier；
5. 只报告均值，隐藏 p95/p99、失败样本或人工介入；
6. 任务集只挑适合 CognitiveOS 的任务（任务来源与选择标准必须预注册）;
7. 把安全拒绝计为对照臂任务失败而不单独报告风险调整效用；
8. 看到结果后更换 primary endpoint、降低门槛、追加样本至显著（optional stopping 无 alpha 校正）；
9. 删除 timeout、拒绝、unknown、quarantine 或人工介入样本；
10. warm cache 臂对 cold cache 基线（缓存状态不对称）。

## 8. Kill criteria 与发布阻断

**优化机制 kill criteria**：对任一 C 类机制，若在两个 workload family 上 C 相对 B 的 primary endpoint 95% CI 不支持改善，或安全指标恶化，或 p99 恶化超过预注册预算，该机制必须：降级为 `experimental`、延后、替换或删除；不得保留在默认开启路径。

**发布阻断条件**：

1. 无 B 臂（governance-only）数据的收益声明 → 阻断该声明发布；
2. 治理开销超出 §5.4 预注册预算且无缓解计划 → 阻断 `implemented` 声明，只可 `experimental`；
3. 安全失败计数相对基线上升 → 阻断发布（安全失败不可被综合分抵消，REQ-PERF-002）；
4. 收益声明缺 comparison 块、预注册引用或 ablation → 报告判定为 `PERFORMANCE_REPORT_INCOMPLETE`（REQ-PERF-005）；
5. 首个 R0/R1 发布必须附 Phase 1 适配前后 A/B 对齐报告（白皮书 §21 Phase 1），仅需 A/B 两臂 + 非劣化判定。

## 9. 证据格式

- 每臂一份或合并一份符合 [performance-report.schema.json](../../specs/schemas/performance-report.schema.json) 的报告；收益声明必须填充 `comparison` 块（arms、primary_endpoint、paired、relative_change_percent、confidence_interval、effect_size、workload_family、preregistration_ref、ablation_results、claim_level）；
- `claim_level` 只能取：`hypothesis`（无数据/未达门槛）、`non_inferiority`（达 §5.3）、`significant_benefit`（达 §5.2）；
- 原始计数、分母、失败清单随 EpisodePackage 或等价证据包归档，敏感内容按 §17.5 脱敏；
- 报告缺失治理开销数据时不得声称"治理开销可忽略"（REQ-PERF-004）。

## 10. 与规范资产的关系

- 本文是 [REQ-PERF-005](../../specs/registry/requirements.yaml) 的 owner_spec；
- 契约向量：[PERF-REPORT-CONTRACT-001](../../conformance/vectors/performance-report-contract.json)（含 comparison 正例与"无对照收益声明"负例断言）；
- 本文变更遵循 [normative-source-and-versioning](../standards/normative-source-and-versioning.md)：门槛、臂定义、claim_level 语义变化为不兼容变更，需新版本与迁移说明。

---

### 版本说明

- **0.1.0-draft.1（2026-07-20）**：初版。四臂设计、W1/W2/W3 workload、六条显著收益门槛、非劣化门槛、workload-specific 开销预算候选值、机制 A/B/C/D 分级、gaming 防护、kill criteria、发布阻断与证据格式。
