# AgentOS v5.0：从第一性原理到认知操作系统

**——一份诚实的智能体运行时架构设计**


## 摘要

本文提出AgentOS v5.0——一个为AI智能体提供生命周期管理、约束执行、通信协议和可观测性的**运行时环境（Runtime Environment）**架构设计。本设计从第一性原理出发，直面当前Agent技术的真实瓶颈（协调税、可靠性、安全性、Token效率），融合操作系统理论、认知科学洞见与工程实践经验，提出三个核心抽象：**(1) Agent Kernel**——一个五模块最小认知单元（模块划分基于关注点分离的工程原则，而非认知科学的理论必然），支持按任务复杂度选择运行模式（完整/标准/轻量），作为系统中一切可调度、可组合、可观测实体的统一原型；**(2) Unified Event Protocol**——统一事件协议，以"单一消息信封格式+作用域命名空间+结构化短格式+上下文句柄"确保所有Agent间交互可通过同一套trace体系进行因果链追踪（协议统一而非物理通道统一）；**(3) Constraint-Propagating Hierarchy**——约束传播式分层控制，任务委托必须携带显式约束包（含上下文传递模式），逐层收窄、不可放宽，从协议层面杜绝级联失控。六条设计公理中，新增的"Token即资源"公理将Token效率从性能优化项提升为架构级约束，确保系统默认行为是Token高效的。本文在设计理念上追求"诚实的复杂度管理"——不回避Agent系统的本质困难，不夸大多Agent协作的收益，不隐藏架构的取舍边界，并诚实标注AgentOS的增量价值仅在多Agent级联、统一可观测性、系统级Token管理和内核级安全执行等特定条件下才能体现。最终目标：让Agent享有运行时级的生命周期管理、通信保障、资源隔离和可观测性。

> **agentos-desktop 落地注**：IM 网关（如飞书）下，`SessionStore` 频道转写由 `gateway/platform_pipeline` 与 `gateway/gateway_kernel_context` 注入 Kernel 规划/执行的 `memory_summary`（**时间序对话行在前**、**`### Feishu session transcript (channel memory)`** 脚注与 **`session_key`** 在后；`runtime/context_scheduler` 在预算紧张时对该块做 **保首尾** 截断以尽量保留最近轮），并由 `AgentKernel` 的 `gateway_runtime_prompt_append` 补充部署边界说明，以区分「频道聊天记录」与「本轮认知步摘要（IncrementalSummarizer）」。**单次 LLM 调用的调度输入上限**由 `runtime/llm_context_budget` 结合 **`AGENTOS_LLM_CONTEXT_WINDOW`**（或 LiteLLM **`get_model_info`**）与 **`budget_tokens`** / **`KernelModeProfile`** 推导，再以 `ContextScheduler` 组装 messages。记忆与 KB **当前实现**路径、存储与主链关系另见同目录 [AGENTOS_MEMORY_KB_CURRENT_ARCHITECTURE.md](./AGENTOS_MEMORY_KB_CURRENT_ARCHITECTURE.md)（与蓝图区分）。**as-built（已实现能力与测试基线）**以同目录 [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md) 与 `scripts/baseline_pytest_passed.golden` 为准；本文 **§十六** Phase 1 最小规格表等为**历史验证起点**，勿当作当前代码规模。详见 [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md)（Phase 3.1 / 6.10.x）与 [GUIDE_SOURCE_MAP.md](./GUIDE_SOURCE_MAP.md)。

---


## 一、现实审视：Agent技术的当下与真实缺口

在进行任何架构设计之前，必须诚实地评估我们面对的技术现实。过度乐观的假设是Agent系统设计中最常见的失败模式。

### 1.1 2025-2026年Agent技术全景

当前Agent技术已进入"基础设施成熟但系统工程不成熟"的阶段：

**已经解决或正在快速解决的问题：**

- **LLM基础能力**。GPT-4o、Claude 3.5/4系列、Gemini 2.0、DeepSeek-V3/R1等模型已具备可靠的工具调用（function calling）、结构化输出（structured output）和长上下文处理能力。模型层不再是主要瓶颈。
- **工具集成标准化**。Anthropic的Model Context Protocol（MCP）已于2025年底捐赠给Linux基金会Agentic AI Foundation，成为事实标准。MCP将N个AI应用×M个工具的N×M集成问题简化为N+M，超过500个公开MCP Server可用。OpenAI、Google DeepMind均已支持MCP。
- **跨Agent通信协议**。Google主导的Agent2Agent（A2A）协议v1.0已发布，基于HTTP/SSE/JSON-RPC构建，获得Atlassian、Salesforce、SAP等50+企业支持。A2A解决了Agent能力发现、交互模态协商和长时任务管理的标准化问题。
- **基础框架可用**。LangGraph提供图编排，AutoGen提供对话式多Agent模式，OpenAI Agents SDK提供极简原语（Agent、Handoff、Guardrail、Tracing），CrewAI提供角色化团队模式。

**核心未解问题——AgentOS存在的理由：**

| 未解问题 | 现状 | 本质原因 |
|---------|------|---------|
| **协调税（Coordination Tax）** | 多Agent系统的协调开销占总耗时42%，Token消耗为单Agent的1.5-15倍（视架构而定）。Google/MIT 2025年研究表明，顺序推理任务中多Agent比单Agent性能下降39-70%。 | 缺乏OS级的调度和通信抽象，协调逻辑与业务逻辑纠缠 |
| **级联不可控** | 深层嵌套的Agent调用链缺乏统一的预算管理、深度限制和失败传播机制，"失败雪崩"频发 | 缺乏约束传递协议，子Agent的行动空间无法被父Agent有效约束 |
| **Subagent ≠ Agent** | 现有框架的Subagent本质是函数调用——无独立状态、无记忆边界、无生命周期 | 缺乏对Agent作为"完整认知单元"的结构化定义 |
| **可观测性碎片化** | 任务调度用一套日志，工具调用用另一套，记忆操作无追踪，跨Agent追踪几乎不可能 | 缺乏统一的事件流和追踪协议 |
| **安全边界模糊** | Agent的权限管理依赖应用层代码，无系统级强制执行；工具调用缺乏沙箱隔离 | 缺乏内核态的安全策略执行机制 |

### 1.2 为什么现有框架不够——以及诚实承认它们"够用"的场景

现有Agent框架的共同局限在于：**它们在应用层解决本应由系统层解决的问题。**

LangGraph/AutoGen/CrewAI提供了编排模式，但不提供进程级的生命周期管理。OpenAI Agents SDK刻意保持极简（"few enough primitives to make it quick to learn"），但这意味着复杂场景的调度、隔离、恢复能力完全由开发者自行实现。AIOS最接近"OS级"思维，但其四层模块化设计（LLM层、Memory层、Storage层、Tool层）按资源类型划分，而非按Agent的认知结构划分。

**诚实声明**：对于单Agent + 少量工具调用的场景，LangGraph + Redis + tracing + guardrail的"拼装"方案完全够用，且启动成本更低。AgentOS的增量价值只在以下条件**同时成立**时才能体现：

- 存在多Agent级联（depth ≥ 2），需要约束沿委托链传播而非每层手动配置
- 需要跨Agent的统一可观测性（一个trace_id贯穿整条调用链），而非手动关联多套日志
- 需要Token预算的系统级管理（防止子Agent耗尽全局预算），而非每个Agent自行计数
- 需要行动空间的内核级强制（子Agent不能调用父Agent未授权的工具），而非依赖prompt约束

如果你的场景不满足这些条件，直接使用现有框架是更务实的选择。AgentOS不追求"替代一切"，而是为**复杂度已超过拼装可控范围的场景**提供系统级保障。

AgentOS的定位：**不是替代这些框架，而是为它们提供底层运行时**——正如JVM不替代Spring Boot，但为它提供内存管理、安全沙箱和类加载机制。


---


## 二、第一性原理推导：从"什么是Agent"到"需要什么OS"

### 2.1 Agent的本质定义

回归最基本的问题：**什么是Agent？**

剥离所有框架术语和营销修饰，Agent的不可约定义是：

> **Agent = 能够感知环境、形成意图、规划行动、执行操作、积累经验、监控自身的自主计算实体。**

这个定义中的六个动词——感知、意图、规划、执行、积累、监控——不是功能列表，而是认知活动的不可约分解。缺少任何一项，系统就退化为更简单的抽象：缺少"意图"和"规划"的是反应式程序（reactive program）；缺少"积累"的是无状态函数；缺少"感知"的是批处理作业；缺少"监控自身"的是开环执行器（open-loop executor）——可以行动，但无法识别自身行为的问题，无法从失败中学习。

六个动词与五模块的映射：

| 认知活动 | 对应模块 | 说明 |
|---------|---------|------|
| 感知 | 感知模块（Perception） | 理解输入和环境状态 |
| 意图 + 规划 | 控制模块（Control） | 形成目标、制定计划、管理执行状态 |
| 执行 | 行动模块（Action） | 通过工具和委托作用于外部世界 |
| 积累 | 记忆模块（Memory） | 存储和检索经验与知识 |
| 监控自身 | 元认知模块（Meta） | 评估自身表现、检测异常、触发反思 |

### 2.2 从认知实体到操作系统进程

如果Agent是具有上述六种认知能力的计算实体，那么在计算机系统的语境中，它最接近什么？

**答案是进程（Process）。**

这不是隐喻，而是结构同构：

| 进程属性 | Agent对应 |
|---------|----------|
| 独立地址空间 | 独立记忆边界 |
| 程序计数器（执行位置） | 状态机（当前认知阶段） |
| 系统调用接口 | 工具调用接口 |
| 进程控制块（PCB） | Agent执行句柄（Execution Handle） |
| 由内核调度 | 由AgentOS调度 |
| 通过IPC与其他进程通信 | 通过Event Fabric与其他Agent通信 |
| 受权限和资源配额约束 | 受行动空间和Token预算约束 |

既然Agent ≈ 进程，那么**缺少的就是操作系统内核**——一个为Agent提供统一的生命周期管理、调度、通信、隔离和可观测性的基础设施层。这正是AgentOS要构建的东西。

### 2.3 OS隐喻的有效边界

必须诚实地标注OS隐喻**不成立**的地方，以避免过度类比导致的设计错误：

| 传统OS进程 | AI Agent | 差异对设计的影响 |
|-----------|---------|----------------|
| 执行确定性程序 | 调用非确定性LLM | Agent的调度不能假设执行时间可预测；需要token-aware调度而非仅CPU-aware |
| 通信内容是结构化数据 | 通信内容可能是自然语言 | 消息协议必须支持非结构化payload；类型系统需要更灵活 |
| 崩溃 = 段错误/异常 | "崩溃"可能是幻觉、拒绝、质量退化 | 失败模型必须包含LLM特有的失败模式 |
| 进程间不共享记忆 | Agent可能需要共享知识 | 记忆模型不能完全照搬进程隔离，需要"可控的记忆投影" |
| 上下文切换成本低 | LLM上下文重建成本高 | 抢占式调度的成本远高于传统OS，需要谨慎使用 |

这些差异不会推翻OS隐喻的核心价值，但会深刻影响具体的调度算法、通信协议和失败处理策略。

### 2.4 更精确的定位：Agent Runtime而非Agent OS

必须进一步追问：Agent系统**到底更像操作系统，还是更像分布式应用框架？**

操作系统出现的根本原因是：多程序竞争硬件资源（CPU/内存/IO），需要强隔离和抢占调度。Agent系统的现实与此有显著差异：核心瓶颈资源是Token（软资源，非硬件），调度是协作式的（LLM调用期间不可中断），隔离是逻辑的（命名空间，非页表）。逐项核实：

| OS职责 | Agent系统是否有同等需求 | 同构程度 |
|--------|---------------------|---------|
| 资源调度 | 需要（多Agent竞争LLM API配额和Token预算）| **低**——不是时间片轮转，而是预算分配+优先级仲裁 |
| 强隔离 | 需要（Agent不应读取其他Agent的内部记忆）| **中**——命名空间隔离+投影规则，不是页表隔离 |
| 抢占调度 | 需要（失控Agent不能耗尽全局Token预算）| **低**——只能在LLM调用间隙，实际是cooperative+超时kill |
| 系统调用拦截 | 需要（Agent只能通过Action模块调用外部工具）| **高**——工具调用前的内核校验与传统syscall高度同构 |

四类职责中三类的实现语义与传统OS有本质差异，只有"系统调用拦截"（约束的内核级强制执行）是高度同构的。

同时，分布式应用框架（Spring Cloud、Akka）也不够用——它不提供约束传播（"只收不放"）、Token预算管理、认知状态机、记忆隔离与投影、行动空间约束。Agent系统需要从OS取资源管理和安全隔离，从分布式框架取服务发现和消息路由，再加上两者都不提供的认知结构定义、约束传播和记忆投影。

因此，AgentOS更精确的定位是**Agent Runtime Environment**——一个为AI Agent提供生命周期管理、约束执行、通信协议和可观测性的运行时层。类比不是Linux，而更接近**JVM/CLR/Erlang OTP**——为特定计算模型提供运行时保障的中间层：

| 参照系 | 核心职责 | AgentOS的对应 |
|--------|---------|-------------|
| JVM的Class结构 | 定义计算实体的结构规范 | Agent Kernel的五模块结构 |
| OTP的消息传递 | 统一的通信协议 | Event Fabric的Message Envelope |
| JVM的GC和内存管理 | 自动资源管理 | Token预算调度和上下文管理 |
| JVM的安全管理器 | 强制安全策略 | 约束传播+内核拦截 |

本文继续使用"AgentOS"这一名称保持与前序版本的连续性，但读者应将其理解为"Agent Runtime"而非传统意义上的操作系统。后文中使用"内核"、"进程"等OS术语，是为了借用成熟的概念框架降低理解成本，而非声称与传统OS在实现层面对等。

### 2.5 设计公理

基于以上分析，推导出AgentOS的六条设计公理：

> **公理1（结构完整性）**：系统中的每个可调度实体都必须是结构完整的Agent Kernel实例——具备感知、记忆、控制、行动、元认知五个模块。不存在"半个Agent"。**但模块实例化深度应与任务复杂度匹配**：当任务可被静态判定为低复杂度（单步工具调用、确定性转换）时，Kernel以轻量模式运行——感知模块做格式解析而非语义理解，控制模块走预设路径而非LLM决策，元认知只执行Tier 1检查，行动模块跳过路由直接调用。这不违反结构完整性——五个模块始终存在，但允许以最小开销实例化。类比：Linux内核中，每个进程都有完整的`task_struct`，但轻量级线程（LWP）通过`clone()`选择性共享地址空间和页表，避免了完整进程的资源开销。"结构完整"不等于"每次都完整实例化"。

> **公理2（通信统一性）**：系统中所有信息流——任务分发、结果回传、状态变更、记忆共享、追踪日志——都必须在同一套协议上流转。差异仅在于作用域和交付语义。

> **公理3（约束只收不放）**：当Agent A委托任务给Agent B时，B的行动空间、Token预算和执行时限必须是A所赋予约束的子集。约束沿委托链单调递减。

> **公理4（内核态强制执行）**：安全约束（行动空间、资源配额）的执行不依赖Agent自身的"自觉"，而由内核在系统调用层面强制拦截。

> **公理5（可观测性是免费的）**：Agent的状态转换、工具调用、记忆操作必须自动产生追踪事件，无需Agent主动埋点。可观测性是架构的内建属性，不是可选的附加功能。

> **公理6（Token即资源）**：LLM Token是系统中最昂贵的计算资源。系统的一切信息传递机制——消息payload、上下文注入、记忆投影、反思触发——都必须以**最小必要Token**为默认设计目标。自然语言全文传递是显式的升级选项，而非隐式的默认行为。结构化短格式优先于自然语言描述，引用句柄优先于内容全文，增量delta优先于全量重述。

公理6的引入理由：前五条公理覆盖了结构完整性、通信统一性、约束单调性、内核强制执行、可观测性——但Token效率作为影响系统经济可行性的基本约束，缺乏公理级声明。将其提升为公理意味着：Token效率不再是"性能优化项"，而是与安全性、可观测性同等级别的架构约束，影响所有设计决策的默认方向。正如传统OS将内存视为一级资源（而非"有了更好"的附加品），AgentOS将Token视为一级资源——每一个消耗Token的设计决策都需要显式的成本-收益论证。


---


## 三、设计哲学

### 3.1 诚实的复杂度管理

AgentOS的核心设计哲学不是"追求优雅的极简"，也不是"追求功能的完备"，而是**诚实的复杂度管理**：

- **该简则简**：核心抽象（Kernel、Fabric、Constraint）追求最小化。不引入任何"可能有用但当前无法验证"的概念。
- **该繁则繁**：在失败处理、安全隔离、可观测性这些决定系统可靠性的地方，不走捷径。
- **明确标注"不做"**：对于系统边界之外的东西（LLM推理优化、业务逻辑编排、工具具体实现），明确声明不接管。

### 3.2 "单Agent优先"原则

这是对当前"多Agent热"的理性回应。Google/MIT 2025年的研究明确表明：顺序推理任务中多Agent方案的性能比单Agent下降39-70%。协调税是真实的、显著的、不可忽略的。

因此AgentOS的设计遵循：

> **单Agent优先原则**：优先让单个Agent在充足的资源和工具下完成任务。仅当任务具有天然的并行结构、或需要异构能力组合、或单Agent的上下文窗口/知识边界不足时，才引入多Agent协作。多Agent不是默认选项，而是"有充分理由时"的升级路径。

这个原则直接影响架构设计：AgentOS的核心抽象（Kernel）必须首先是一个强大的单Agent运行时，多Agent协作是在此基础上的自然扩展，而非相反。

**量化决策框架**（公理6的直接推论）：

```yaml
multi_agent_decision:
  prefer_single_agent_when:
    - estimated_coordination_tokens > 0.3 * estimated_task_tokens
    - task_is_sequential_reasoning: true
    - subtask_count <= 2
  prefer_multi_agent_when:
    - task_has_natural_parallelism: true AND parallel_speedup > 1.5x
    - required_capabilities_count > single_agent_tool_limit
    - single_agent_context_requirement > 0.8 * context_window_size
  coordination_tax_estimate:
    per_delegation: ~500-1500 tokens  # 任务描述+约束传递+结果回传
    per_additional_agent: ~800-2000 tokens  # 系统提示+工具schema+角色定义
  delegation_payback_threshold:
    formula: "(single_agent_cost - multi_agent_total_cost) / multi_agent_total_cost"
    min_value: 0.2  # 多Agent至少比单Agent节省20%才值得委托
```

原则需要量化支撑。没有量化判断框架，"何时用多Agent"完全取决于实现者的直觉——而大多数开发者的直觉倾向于高估多Agent的收益、低估协调税的成本。

### 3.3 与现有生态的关系定位

AgentOS不是封闭生态，而是一个**开放的运行时层**。它与现有技术栈的关系是：

```
┌──────────────────────────────────────────────────────────┐
│                     用户应用 / Agent逻辑                     │
│   （使用LangGraph编排 / AutoGen对话 / 自定义逻辑）            │
├──────────────────────────────────────────────────────────┤
│                     AgentOS Runtime                        │
│   Kernel · Event Fabric · Scheduler · Memory · Security   │
├──────────────────────────────────────────────────────────┤
│                     外部协议与服务                           │
│   MCP（工具调用） · A2A（跨系统Agent通信） · LLM API         │
└──────────────────────────────────────────────────────────┘
```

- **向上**：为Agent应用提供系统服务（调度、通信、隔离、观测）
- **向下**：通过MCP连接工具生态，通过A2A连接外部Agent系统，通过LLM API连接推理服务
- **并行**：与LangGraph/AutoGen等编排框架共存——它们负责业务编排逻辑，AgentOS负责底层运行时


---


## 四、系统边界：AgentOS接管什么，不接管什么

在详述架构之前，必须明确划定系统边界。模糊的边界是系统膨胀和失败的根源。

### 4.1 系统边界定义

**一句话**：AgentOS是一个面向AI Agent的认知运行时（Cognitive Runtime），负责Agent的生命周期管理、资源调度、安全隔离、消息路由和状态观测。

| 维度 | AgentOS接管（内核空间） | 开发者负责（用户空间） |
|------|---------------------|-------------------|
| **执行单元管理** | 创建、暂停、恢复、销毁Agent Kernel实例 | 实现Kernel五模块的具体逻辑（提示词、推理策略、业务规则） |
| **调度** | 决定哪个Agent获得计算资源；提供阻塞、唤醒、让出原语 | 声明任务的优先级和资源需求 |
| **通信** | 提供统一事件织网（Event Fabric），保证消息路由和交付语义 | 定义消息的业务内容和处理逻辑 |
| **记忆基础设施** | 管理命名空间、存储配额、投影管道 | 定义记忆格式、检索策略、投影规则 |
| **安全** | 强制执行行动空间约束，隔离执行环境 | 声明所需权限和能力边界 |
| **可观测性** | 自动采集状态转换、工具调用、消息流的Trace数据 | 定义业务指标和告警规则 |

### 4.2 明确不接管

- **LLM推理优化**：提示词工程、模型选择、推理参数调优由Agent自行决定。
- **业务编排逻辑**：任务如何分解、子任务如何组合、结果如何聚合由上层编排框架或Agent逻辑负责。
- **工具具体实现**：AgentOS通过MCP连接工具，但不实现工具逻辑。
- **跨组织Agent通信**：跨组织边界的Agent发现和通信由A2A协议处理，AgentOS在组织内部提供运行时。
- **向量检索算法**：记忆基础设施提供存储和命名空间，具体的嵌入和检索算法由用户空间的记忆模块实现。

### 4.3 与MCP/A2A的边界划分与跨协议衔接

AgentOS对MCP和A2A的依赖存在双重不确定性，必须诚实面对并明确增量价值。

**与MCP的边界**：MCP已提供工具调用的标准化接口和基础安全校验（如MCPSHIELD覆盖的7个威胁类别）。AgentOS在MCP之上的**增量价值**限定于三个维度：

| 维度 | MCP提供 | AgentOS增加 |
|------|--------|------------|
| 工具调用权限 | 单次调用的权限校验 | 跨Agent委托链的权限单调递减 |
| 资源管理 | 无Token预算概念 | Token/时间/成本的全局预算管理 |
| 行为监控 | 单次调用的输入输出校验 | 跨步骤的行为模式异常检测（循环、退化） |
| 上下文管理 | 无 | 上下文调度、记忆投影、信息增益评估 |

**与A2A的衔接**：当Agent A委托任务给通过A2A暴露的外部Agent时，A2A Gateway负责将约束包映射为A2A Task元数据（如`budget.tokens` → `a2a.task.metadata.max_tokens`）。核心原则是：**对外部Agent的响应视为不可信输入**——感知模块重新校验，不直接注入上下文。因为对A2A外部Agent，AgentOS只能"请求"而非"强制"其遵守约束——这是跨组织信任边界的本质限制。

**对MCP生态不确定性的对冲**：MCP Connector作为可替换的Adapter层实现。工具调用的语义（invoke、result、error）在AgentOS内部有独立的抽象表示，不直接耦合MCP的wire format。如果MCP被取代，只需替换Adapter。

### 4.4 开发者体验：AgentOS如何为应用开发者做"减法"

传统OS对应用开发者的核心价值是**减法**——你不需要理解页表、中断处理和调度算法，OS替你做了。AgentOS的设计必须提供同等的减法体验，否则五模块、两套FSM、约束传播、投影规则就只是**从应用层搬到概念层的复杂度**，而非被消除的复杂度。

AgentOS区分两类用户，提供不同的抽象层次：

| 用户角色 | 需要理解的概念 | 不需要关心的概念 | 类比 |
|---------|-------------|--------------|------|
| **Agent应用开发者** | 模块接口（5个函数签名）、约束声明（allowed_tools + budget）、记忆读写API | FSM细节、Event Fabric路由、投影规则、调度算法、生命周期状态映射 | Linux应用开发者：只需`open()`/`read()`/`write()`，不需要理解VFS和页缓存 |
| **AgentOS内核开发者** | 全部架构细节 | 无 | Linux内核开发者 |

面向应用开发者的SDK应将Agent定义的概念负担降到与现有框架相当的级别：

```python
# 目标：用AgentOS SDK定义一个Agent，概念负担 ≈ OpenAI Agents SDK
from agentOS import Agent, tool, constraint

@Agent(
    name="research_agent",
    tools=["web_search", "fetch_page"],       # 行动模块：声明可用工具
    budget={"tokens": 10000, "time_ms": 60000}, # 约束声明
)
async def research(query: str) -> str:
    """执行网络调研并生成摘要"""          # 感知模块：从函数签名自动推导
    ...                                    # 控制模块：SDK提供默认FSM
                                           # 元认知：SDK自动注入Tier 1检测
                                           # 记忆：SDK提供默认工作记忆
```

SDK的设计原则：
- **显式优于隐式，但默认值必须合理**——五个模块都有开箱即用的默认实现，开发者只需覆盖有业务需求的部分
- **渐进式暴露复杂度**——初级用户只写装饰器和函数体；需要自定义FSM或投影规则的高级用户可以逐步深入
- **内核复杂度对SDK用户不可见**——两套FSM的映射、Event Fabric路由、约束的内核级拦截都在SDK层之下

**诚实声明**：SDK层的设计不在本文的架构范围内（本文定义的是Runtime层），但SDK的可行性是Runtime设计是否过于复杂的试金石——如果五模块结构无法被封装为上述级别的简洁API，那说明模块划分需要简化。Phase 1应同时验证Runtime实现和SDK封装的可行性。


---


## 五、架构全景

### 5.1 三层架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                          应用层（User Space）                          │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │  Agent A    │  │  Agent B    │  │  Agent C    │  │  Agent D  │  │
│  │  (Kernel    │  │  (Kernel    │  │  (Kernel    │  │  (Kernel  │  │
│  │   实例)     │  │   实例)     │  │   实例)     │  │   实例)   │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────┬─────┘  │
│         │                │                │               │         │
├─────────┴────────────────┴────────────────┴───────────────┴─────────┤
│                       Event Fabric（事件织网）                         │
│                                                                      │
│    统一消息协议 · 作用域命名空间 · 投影规则 · 流控与降维                  │
├──────────────────────────────────────────────────────────────────────┤
│                      AgentOS Kernel（内核层）                          │
│                                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Scheduler │ │ Memory   │ │ Security │ │Lifecycle │ │  Trace   │  │
│  │          │ │ Manager  │ │ Manager  │ │ Manager  │ │ Collector│  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
├──────────────────────────────────────────────────────────────────────┤
│                       外部接口层（Adapter Layer）                       │
│                                                                      │
│  ┌────────────────┐  ┌───────────────┐  ┌────────────────────────┐  │
│  │ MCP Connector  │  │ A2A Gateway   │  │ LLM Provider Adapter  │  │
│  │ （工具生态）    │  │（跨系统通信）  │  │  （推理服务）           │  │
│  └────────────────┘  └───────────────┘  └────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

**应用层**：运行Agent Kernel实例。每个Kernel是一个结构完整的认知单元，由开发者实现具体的感知、推理和行动逻辑。

**事件织网**：系统的信息中枢。所有Agent间通信、状态变更通知、记忆投影、追踪数据都通过统一的事件协议流转。

**内核层**：提供系统服务——调度（决定谁执行）、记忆管理（提供存储基础设施）、安全管理（强制执行约束）、生命周期管理（创建/暂停/恢复/销毁）、追踪采集（可观测性）。

**外部接口层**：适配外部协议——MCP用于工具调用，A2A用于跨系统Agent互操作，LLM Provider Adapter统一不同推理服务的调用接口。

### 5.2 核心抽象之间的关系

```
                    ┌─────────────────┐
                    │  Constraint      │
                    │  Bundle          │
                    │ （约束包）        │
                    └────────┬────────┘
                             │ 沿委托链传递
                             ▼
┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Agent      │◄──►│  Event Fabric   │◄──►│   Agent      │
│   Kernel     │    │  （事件织网）     │    │   Kernel     │
│  （认知单元） │    └─────────────────┘    │  （认知单元） │
└──────┬───────┘             ▲              └──────┬───────┘
       │                     │                     │
       ▼                     │                     ▼
┌──────────────┐    ┌────────┴────────┐    ┌──────────────┐
│  Execution   │    │   AgentOS       │    │  Execution   │
│  Handle      │◄──►│   Kernel        │◄──►│  Handle      │
│ （执行句柄）  │    │  （系统内核）    │    │ （执行句柄）  │
└──────────────┘    └─────────────────┘    └──────────────┘
```

三个核心抽象的关系简洁而清晰：
- **Agent Kernel** 是认知单元，由开发者实现
- **Event Fabric** 是信息流通的唯一通道
- **AgentOS Kernel** 通过Execution Handle管理Agent Kernel的生命周期


---


## 六、Agent Kernel：最小认知单元

### 6.1 从六单元到五模块：一次诚实的重构

v3.0-v4.5定义了六单元结构（Task、Memory、State、Control、Tool、Env）。v5.0将其重构为五模块，变更如下：

- **重命名"任务单元"为"感知模块"（Task → Perception）**：原"Task"单元的名称暗示Agent只处理任务分配，但实际上它的职责是理解所有类型的输入——用户指令、环境状态、父Agent的消息。"感知"更准确地反映了这个模块的认知语义：Agent感知和理解外部世界的窗口。
- **重命名"工具单元"为"行动模块"（Tool → Action）**：原"Tool"单元的名称过于实现导向。Agent作用于外部世界的方式不仅是调用工具，还包括委托子任务、发布事件等。"行动"覆盖了所有对外输出的语义。
- **合并"状态单元"与"控制单元"为"控制模块"（State + Control → Control）**：状态机本质上是控制逻辑的一部分——FSM的状态转换、局部策略决策、升级规则都是"Agent如何控制自身行为"的不同方面。将它们分离增加了概念负担而无实际收益。
- **将"环境单元"下沉为内核服务（Env → AgentOS Kernel）**：执行环境（沙箱、资源限制）不应由Agent自身管理。Agent只需声明需求，环境的实际配置和隔离由AgentOS内核负责。这更符合OS的设计原则——进程不管理自己的页表。
- **新增"元认知模块"（Meta）**：见6.3节。这是v5.0的核心新增，将Agent的自我监控和反思能力从"附加功能"提升为认知完整性的必要条件。

**五模块划分的工程依据——反证法**：上述重构的论证基于"认知活动的不可约分解"，但必须诚实补充工程层面的论证。模块划分的真正依据不仅是认知模型的理论对称性，更是**关注点分离的工程收益**（可测试性、可替换性、强制性）。以下通过反证法——如果去掉某个模块会发生什么——来论证每个模块的必要性：

| 如果去掉... | 替代方案 | 工程代价 |
|-----------|---------|---------|
| **Meta（元认知）** | 将循环检测、步数限制做成middleware/decorator | 安全检查变为可选——开发者可以不挂或关掉。Meta作为FSM必经状态（`EXECUTING → REFLECTING`）的价值在于其**不可绕过性**，这是middleware模式无法保证的 |
| **Perception（感知）** | 将输入解析合并进Control | Control膨胀为"解析+状态管理+决策+升级"的超级模块，丧失可测试性（无法单独测试输入解析）和可替换性（无法在不动控制逻辑的前提下换解析策略） |
| **Control（控制）** | 简化为while循环 | 加上阻塞语义（WAITING）、升级出口（escalation）、预算感知模式切换后，循环会自然演化为有状态转换规则的状态机——即回到Control模块 |
| **Memory（记忆）** | 使用外部存储服务 | 丧失记忆所有权归属Agent的隔离保障——Agent A可以随意读取Agent B的存储，行动空间约束可被间接绕过 |
| **Action（行动）** | 直接调用API | 丧失统一的pre/post hook链（输入校验、限流、熔断、输出清洗、调用追踪），每个工具调用都需要开发者手动添加这些保障 |

**诚实声明**：五模块之间不存在数学意义上的"不可约性"证明。Perception和Action可以合并为"IO模块"，Meta可以降级为Control的子组件——系统仍然能工作。五模块的数量是**关注点分离粒度的工程选择**，在"过粗导致职责混杂"和"过细导致概念负担"之间取平衡。当前划分的具体验证标准是：Phase 1中每个模块是否可以独立实现、独立测试、独立替换——如果可以，则粒度合适。

重构后的五模块结构：

```
┌─────────────────────────────────────────────────────────────────┐
│                    Agent Kernel（五模块认知结构）                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  感知模块        │  │  记忆模块        │  │  控制模块        │  │
│  │  (Perception)   │  │  (Memory)       │  │  (Control)      │  │
│  │                 │  │                 │  │                 │  │
│  │  理解输入       │  │  存储与检索经验   │  │  状态+决策+约束  │  │
│  │  解析任务目标   │  │  管理记忆生命周期 │  │  FSM状态转换    │  │
│  │  识别环境状态   │  │  投影到全局       │  │  局部策略执行    │  │
│  └─────────────────┘  └─────────────────┘  │  升级规则判定    │  │
│                                             └─────────────────┘  │
│  ┌─────────────────┐  ┌─────────────────┐                       │
│  │  行动模块       │  │  元认知模块      │                       │
│  │  (Action)       │  │  (Meta)         │                       │
│  │                 │  │                 │                       │
│  │  工具注册与路由  │  │  自我评估       │                       │
│  │  调用前后钩子   │  │  执行质量追踪   │                       │
│  │  限流与熔断     │  │  反思与学习触发  │                       │
│  └─────────────────┘  └─────────────────┘                       │
│                                                                  │
│               所有模块通过 Local Event Channel 互联                │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Kernel运行模式分级

公理1的补充条款（"模块实例化深度应与任务复杂度匹配"）需要在Kernel层面提供明确的运行模式。这直接回应了一个关键的Token效率问题：如果每个可调度实体都以完整模式启动——携带完整的系统提示、完整的工具schema、完整的反思流程——那么高频短任务的**固定prompt开销/有效推理Token**比率将急剧恶化。

```yaml
kernel_execution_modes:
  full_mode:
    description: "完整认知流程，适用于复杂、高不确定性任务"
    perception: semantic_understanding     # LLM辅助理解
    control: llm_planning + full_fsm       # 完整规划和状态转换
    action: capability_routing             # 能力匹配路由
    meta: tier1 + tier2 + optional_tier3   # 全层级元认知
    memory: full_read_write_project        # 完整读写投影
    typical_overhead: 1500-3000 tokens     # 系统提示+工具+角色

  standard_mode:
    description: "标准认知流程，适用于中等复杂度任务"
    perception: template_parsing           # 模板化解析
    control: llm_planning + simplified_fsm # 简化FSM（跳过可选状态）
    action: direct_invocation              # 直接调用指定工具
    meta: tier1 + tier2                    # 确定性+统计检测
    memory: working_only                   # 仅工作记忆
    typical_overhead: 800-1500 tokens

  lightweight_mode:
    description: "最小认知流程，适用于确定性单步任务"
    perception: format_validation          # 格式校验
    control: preset_path                   # 预设执行路径，无LLM决策
    action: direct_single_tool             # 单工具直接调用
    meta: tier1_only                       # 仅确定性检查
    memory: pass_through                   # 结果直接回传，不写记忆
    typical_overhead: 200-500 tokens

  mode_selection:
    criteria:
      - task_complexity: { low: lightweight, medium: standard, high: full }
      - delegation_depth: { ">=3": prefer_lightweight_for_leaves }
      - remaining_budget_ratio: { "<0.2": force_lightweight }
    default: standard_mode
```

**预期收益**：叶子节点任务（在多层级联架构中占任务总量60-80%）的Kernel开销降低60-80%。高频短任务场景下，单任务Token消耗可降低40-60%。

**与公理1的关系**：三种模式都保持五模块的完整结构——区别在于每个模块的**实例化深度**。这正如Linux的`clone()`系统调用：可以创建共享地址空间的轻量级线程，也可以创建完全独立的进程，但`task_struct`始终完整。

**lightweight_mode的行为验证标准**：一个合理的质疑是——如果感知模块只做格式校验、控制模块走预设路径、元认知只执行Tier 1检查，lightweight_mode下的Kernel是否仍然是"Agent"？验证标准如下：(1) **循环检测仍然有效**——即使走预设路径，Tier 1检查仍能检测工具调用的重复模式并触发中止；(2) **预算约束仍被强制执行**——lightweight_mode不绕过内核约束拦截；(3) **失败仍能有序传播**——错误通过escalation策略上报父Agent。lightweight_mode放弃的是"LLM辅助的自适应决策"，保留的是"确定性的自我保护和有序失败"。如果一个实体连循环检测和预算遵守都做不到，它就不是lightweight Agent，而是裸函数调用——后者不需要AgentOS。

**运行时模式切换**：Kernel模式不是静态配置。`mode_selection`中的`remaining_budget_ratio`规则意味着：正在以full_mode执行的Agent会在预算压力下自动降级到lightweight_mode——预算压力转化为模式降级而非任务失败。反向升级也是允许的：lightweight_mode发现任务复杂度超出预设路径能力时，可在预算允许的前提下升级到standard_mode。

### 6.3 为什么引入"元认知模块"

v3.0-v4.5虽然在状态单元中提到了`self_rating`和反思，但未将其提升为一等模块。v5.0显式引入**元认知模块（Meta Module）**，理由基于两项关键研究发现：

1. **Reflexion（Shinn et al., 2023）** 证明了Agent的自我反思能力可以将任务成功率提升20%以上。反思不是"附加功能"，而是认知闭环的必要组成。
2. **Inner Monologue（Huang et al., 2023）** 表明，Agent对自身执行过程的持续监控和评估，是实现长序列任务稳定性的关键。

**与传统监控的本质区别**：一个合理的质疑是——如果元认知的Phase 1-2主要依靠确定性检测（循环检测、步数限制、格式校验），那它与传统软件工程中的断言、监控和健康检查有什么本质区别？区别在于三点：(1) **回路闭合性**——传统断言失败后终止程序或抛异常，等待人类干预；元认知检测到异常后触发控制模块的REFLECTING状态，Agent可以自主重规划。断言是开环告警，元认知是闭环自愈。(2) **经验积累性**——传统监控不修改程序行为；元认知的`extract_lesson`将执行经验写入语义记忆，影响未来决策。这是"学习"能力，不是"监控"能力。(3) **跨步骤的趋势感知**——传统断言检查单一不变量；Tier 2的统计检测分析多步执行轨迹的趋势，识别渐进式质量退化——这种退化不会触发任何单点断言，只有跨步骤的模式分析才能捕获。简言之：传统监控回答"这一步是否出错"，元认知回答"我的整体执行策略是否在恶化，以及如何调整"。

元认知模块的职责：
- **执行质量评估**：每一步执行后估算输出质量（置信度）
- **异常模式识别**：检测重复失败、循环行为、质量退化趋势
- **反思触发**：当质量低于阈值或检测到异常模式时，触发控制模块的REFLECTING状态
- **经验提炼**：将成功的执行策略抽象为可复用的经验，写入记忆模块

### 6.4 各模块精确定义

#### 6.4.1 感知模块（Perception Module）

**认知语义**：Agent感知和理解外部世界的窗口。将原始输入（用户指令、环境状态、父Agent的TASK消息）转化为结构化的内部表示。

```yaml
perception_module:
  # 静态能力声明——Agent对外暴露的"感知范围"
  capability:
    name: "research_agent"
    description: "执行网络调研并生成结构化报告"
    input_schema:
      query: string
      depth: { type: int, default: 2 }
      format: { type: string, enum: ["summary", "report", "data"] }
    output_schema:
      result: string
      sources: list[string]
      confidence: float

  # 运行时接口
  interface:
    parse(raw_input) -> TaskDefinition       # 将原始输入解析为结构化任务
    validate(task_def) -> ValidationResult   # 校验任务是否在能力范围内
    perceive(context) -> PerceivedState       # 感知当前上下文状态
```

**设计决策**：感知模块显式分离了"静态能力声明"和"运行时解析"。能力声明用于Agent发现（类似A2A的AgentCard），运行时解析用于实际的任务理解。这使得系统可以在不实例化Agent的情况下了解其能力——这对调度和任务路由至关重要。

#### 6.4.2 记忆模块（Memory Module）

**认知语义**：Agent积累和利用经验的机制。管理分层的记忆空间，并通过投影规则选择性地向全局共享。

```yaml
memory_module:
  # 三层记忆结构
  working_memory:
    description: "当前任务的在线工作区"
    implementation: ring_buffer
    capacity: { max_tokens: 4000 }
    lifetime: task_scoped     # 任务结束时清理

  episodic_memory:
    description: "执行轨迹——发生了什么"
    implementation: append_only_log
    capacity: { max_entries: 1000 }
    lifetime: session_scoped  # 会话结束时可选持久化

  semantic_memory:
    description: "可复用知识——学到了什么"
    implementation: key_value + vector_index
    capacity: { max_entries: 10000 }
    lifetime: persistent       # 跨会话持久化

  # 投影规则——哪些记忆对外可见
  projection:
    - source: episodic_memory
      destination: "global.memory.episodic.{agent_id}"
      filter: "success == true AND confidence > 0.8"
    - source: semantic_memory
      destination: "global.memory.semantic.{domain}"
      filter: "reuse_count > 3"
    deduplication:
      method: content_hash         # 基于内容哈希去重
      window: task_scoped          # 同一任务窗口内不重复投影
      budget_gate: true            # 低预算状态下暂停非关键投影
    promotion_criteria:            # 从情景升入语义的条件
      min_reuse_count: 3
      min_confidence: 0.8
      future_reuse_probability: "> 0.5"

  # 增量摘要策略（公理6：避免同一信息的重复全量摘要）
  summarization:
    strategy: incremental          # 增量而非全量
    state:
      base_summary: string         # 截至上一个检查点的摘要
      delta_log: list[string]      # 自上一个检查点以来的增量
      version: int                 # 摘要版本号
    update_protocol:
      on_new_result: append_to_delta_log
      on_delta_exceeds_threshold:  # delta累积超过阈值时合并
        merge_delta_into_base
        reset_delta_log
        increment_version
      threshold: 500 tokens        # delta超过500 tokens时触发合并

  # 运行时接口
  interface:
    write(key, value, memory_type) -> void
    read(query, memory_type) -> List[MemoryEntry]
    project() -> List[ProjectedMemoryEvent]    # 由内核定期调用
    summarize_incremental() -> Summary         # 增量摘要
```

**设计决策**：记忆的三层划分（工作/情景/语义）借鉴了认知科学的记忆类型学和MemGPT等分层记忆管理研究，但关键区别在于**记忆所有权归Agent自身**。每个Agent拥有独立的记忆命名空间，跨Agent的记忆共享必须通过显式投影——不允许Agent A直接读取Agent B的内部记忆。这对应操作系统中"进程不能直接读取其他进程的地址空间"的基本安全原则。

**与LLM上下文窗口的关系**：工作记忆（working_memory）直接对应LLM的上下文窗口管理。当工作记忆接近容量上限时，记忆模块负责执行压缩策略（摘要、淘汰、归档到情景记忆）。这个过程对Agent的其他模块透明。

**增量摘要与投影去重**（公理6的推论）：同一信息在分层记忆中可能以多种文本变体存在——原始结果写入工作记忆，摘要写入情景记忆，经验提炼写入语义记忆，进度投影到Task Scope，父Agent汇总时再生成高层摘要。如果没有版本化和增量机制，同一段事实以4-5个文本变体存在，并在后续检索时被重复注入上下文——这不是存储成本问题，而是**每次检索都可能为同一信息重复付Token的问题**。增量摘要将全量重写替换为delta合并，投影去重通过内容哈希防止同一信息的重复投影。

#### 6.4.3 控制模块（Control Module）

**认知语义**：Agent的"执行控制中枢"。统一管理状态转换、局部决策策略和升级规则。

```yaml
control_module:
  # 有限状态机（FSM）
  fsm:
    states:
      - IDLE         # 等待任务
      - PERCEIVING   # 解析输入
      - PLANNING     # 生成行动计划
      - EXECUTING    # 执行行动步骤
      - WAITING      # 等待外部事件（子任务结果、LLM响应）
      - REFLECTING   # 反思与调整（由元认知触发）
      - COMPLETED    # 任务成功完成
      - FAILED       # 任务失败

    transitions:
      RECEIVE_TASK:      IDLE → PERCEIVING
      TASK_PARSED:       PERCEIVING → PLANNING
      PLAN_READY:        PLANNING → EXECUTING
      NEED_SUBTASK:      EXECUTING → WAITING
      SUBTASK_DONE:      WAITING → EXECUTING
      ALL_STEPS_DONE:    EXECUTING → REFLECTING    # 必经的反思检查点
      QUALITY_OK:        REFLECTING → COMPLETED
      QUALITY_LOW:       REFLECTING → PLANNING     # 重新规划
      UNRECOVERABLE:     * → FAILED

  # 局部自治策略——Agent在此边界内自主决策，无需上报
  local_policy:
    max_retries: 3
    retry_strategy: exponential_backoff
    allowed_autonomous_decisions:
      - retry_on_transient_error
      - cache_lookup_before_tool_call
      - query_rewrite_on_empty_result
      - output_format_normalization
    fallback_action: "return_partial_result_with_explanation"

  # 升级策略——何时必须上报父Agent
  escalation:
    triggers:
      - condition: "action_not_in_allowed_space"
        response: block_and_escalate
      - condition: "budget_remaining < threshold"
        response: abort_and_return_partial
      - condition: "consecutive_failures >= max_retries"
        response: pause_and_request_guidance
      - condition: "confidence < 0.3"
        response: escalate_with_context

  # 行动空间约束（由内核强制执行）
  action_space:
    allowed_tools: ["web_search", "fetch_page", "parse_html"]
    forbidden_tools: ["execute_code", "send_email"]
    max_steps_per_task: 15
    max_delegation_depth: 3

  # 运行时接口
  interface:
    transition(event) -> NewState
    decide(context) -> Action | Escalation
    validate_action(action) -> bool
    snapshot() -> ControlStateSnapshot
```

**设计决策**：控制模块是v5.0中"受控自治"理念的核心载体。通过将`local_policy`和`escalation`分离，实现了"在局部范围内自主决策，超出边界时有序上报"的模式。这解决了多Agent系统中的一个根本性矛盾：如果所有异常都回传父Agent，通信成本爆炸；如果子Agent完全自主，行为不可控。

**FSM中的强制反思**：注意状态转换中，`EXECUTING → REFLECTING`是必经的检查点。这不是可选的"反思功能"，而是架构层面的强制要求——每次任务执行完毕后，都必须经过元认知评估再进入COMPLETED状态。分层控制理论与工程实践均表明，这种强制性的"验证-反思"环节可以显著降低级联失败率——我们的模拟估算显示降幅约47%。

#### 6.4.4 行动模块（Action Module）

**认知语义**：Agent作用于外部世界的"手臂"。管理可用工具的注册、路由、调用前后的钩子链以及限流熔断。

```yaml
action_module:
  # 工具注册表
  tools:
    - name: "web_search"
      protocol: "mcp"                         # 通过MCP协议调用
      endpoint: "mcp://search-server/search"
      schema_ref: "web_search_v2"
      pre_hooks: [validate_query, check_rate_limit, log_invocation]
      post_hooks: [sanitize_output, extract_citations, record_latency]

    - name: "delegate_subtask"
      protocol: "internal"                     # 通过Event Fabric委托给其他Agent
      target_resolution: "capability_match"    # 按能力匹配目标Agent

  # 工具路由策略
  routing:
    strategy: "capability_match"    # 或 explicit_name / round_robin
    fallback_on_failure: true
    fallback_tool: "cached_result_lookup"

  # 调用约束
  limits:
    max_calls_per_task: 30
    max_tokens_per_call: 8000
    rate_limit: "20/minute"
    circuit_breaker:
      failure_threshold: 5         # 连续5次失败触发熔断
      recovery_timeout_seconds: 60

  # 工具描述注入策略（公理6：按需切片，非全量注入）
  tool_injection:
    strategy: adaptive_slice
    rules:
      - when: "current_step_has_explicit_tool"
        inject: only_specified_tool_schema
      - when: "planning_phase"
        inject: tool_name_and_one_line_description  # 只注入名称和简介
      - when: "execution_phase"
        inject: full_schema_of_candidate_tools      # 最多2-4个候选
      - when: "remaining_budget < 0.3 * total"
        inject: minimal_tool_signatures             # 极简签名
    common_params_template:
      compress: true
      template_ref: "shared_param_patterns"

  # 运行时接口
  interface:
    invoke(tool_name, params) -> ToolResult
    list_available() -> List[ToolDescriptor]
    list_relevant(context) -> List[ToolDescriptor]  # 上下文相关的工具子集
```

**设计决策**：行动模块与MCP深度集成。MCP已成为工具调用的事实标准，AgentOS不需要发明新的工具协议，而是将MCP作为一等公民嵌入行动模块。同时，"委托子任务"也被建模为一种"工具调用"——通过Event Fabric发送TASK消息给另一个Agent，这种统一简化了控制模块的决策逻辑。

**熔断器**：借鉴微服务架构的Circuit Breaker模式。当某个工具连续失败时，自动熔断并切换到备选路径，而不是继续消耗Token进行注定失败的重试。

**工具描述按需切片**（公理6的推论）：在三层角色架构下，如果每次LLM调用都完整注入所有工具的完整schema，**固定前缀可能占据单次调用Token的30-60%**。自适应切片策略将工具描述的注入量与当前执行阶段匹配：规划阶段只需要工具名称和一行简介即可做出选择，执行阶段只需要当前候选工具（2-4个）的完整schema。预期收益：工具schema注入的Token减少50-70%。

#### 6.4.5 元认知模块（Meta Module）

**认知语义**：Agent对自身认知过程的监控和评估能力——"思考自己是怎么思考的"。

**实现诚实性声明**：元认知能力存在一个明确的**可靠性光谱**。文档必须诚实区分"现在就能可靠实现的"和"本质上仍是开放问题的"：

| 检测能力 | 可靠性 | 实现方式 | 需要LLM？ | 阶段 |
|---------|--------|---------|----------|------|
| 循环检测（重复动作） | **极高** | 最近N步的行为模式匹配 | 否 | Phase 1 |
| 预算消耗异常 | **极高** | Token消耗速率的算术比较 | 否 | Phase 1 |
| 格式违规 | **高** | Schema / 正则校验 | 否 | Phase 1 |
| 步骤超限 | **高** | 计数器 | 否 | Phase 1 |
| 质量退化趋势 | **中** | 输出长度/结构变化的统计检测 | 否 | Phase 2 |
| 输出质量评估 | **低** | LLM self-critique（见下文限制） | 是 | Phase 3+ |
| 幻觉检测 | **极低** | 外部验证/事实核查/多模型交叉检验 | 是 | 开放问题 |

**对LLM self-critique的诚实评估**：`self_critique`本质上是"用一个不可靠的评估器去评估另一个不可靠的输出"。LLM的自我评估存在系统性偏差（倾向于高估自身输出质量），false positive率高，且会使Token成本翻倍、延迟增加。更关键的是，Reflexion等研究的成功部分依赖于**外部环境反馈**（代码执行结果、测试通过率），而非纯LLM self-critique——让LLM在没有任何客观信号的情况下评估自身输出，是一个有偏估计。因此：(1) Phase 1-2的元认知**只使用不依赖LLM的确定性检测**（循环、预算、格式、步数）；(2) LLM-based质量评估在Phase 3引入时，**必须绑定至少一个外部验证信号**（工具调用结果校验、测试执行结果、人类反馈、或跨模型校验）——没有外部信号的纯self-critique不应被启用；(3) 幻觉检测不纳入当前架构承诺——这是整个AI领域的开放问题，AgentOS不假装已解决。

```yaml
meta_module:
  # Tier 1: 确定性检测（Phase 1, 不依赖LLM, 可靠性极高）
  deterministic_checks:
    detect_loops: true                 # 最近N步行为模式匹配
    loop_detection_window: 5
    detect_budget_waste: true          # Token消耗速率异常
    max_steps_without_progress: 5      # 无实质进展的步数上限
    format_validation: true            # 输出Schema校验

  # Tier 2: 统计检测（Phase 2, 不依赖LLM, 可靠性中等）
  statistical_checks:
    detect_quality_decay: true         # 输出结构/长度的统计偏移
    decay_detection_window: 10         # 检测窗口大小
    baseline_method: "rolling_average" # 基线计算方法

  # Tier 3: LLM辅助评估（Phase 3+, 可选, 可靠性低, 成本高）
  llm_assisted:
    enabled: false                     # 默认关闭, 需显式启用
    evaluation_method: "self_critique"
    cost_budget_ratio: 0.1             # 评估成本不超过任务成本的10%
    confidence_threshold: 0.5
    # 外部反馈源要求：纯LLM self-critique有效性严重受限，
    # Tier 3启用时必须绑定至少一个外部验证信号
    required_external_signal:
      min_count: 1                     # 至少一个外部信号
      accepted_types:
        - tool_result_validation       # 工具调用结果是否符合预期schema
        - test_execution_outcome       # 代码/测试的实际执行结果
        - human_feedback               # 人类反馈（审批门或评分）
        - cross_model_verification     # 不同模型的交叉校验
      fallback_if_no_signal: "skip_tier3"  # 无外部信号时跳过Tier 3

  # 反思策略
  reflection:
    trigger_conditions:
      - "loop_detected"
      - "budget_anomaly"
      - "format_violation_count > 2"
      - "task_completed"               # 每次任务完成后反思

    # 进入REFLECTING状态后的执行协议（消除实现歧义的决策树）
    execution_protocol:
      step_1_deterministic:  # 必执行，0 LLM Token
        - run: tier1_checks  # 循环检测、步数、格式、预算
        - if: any_alert → replan_with_new_strategy
        - cost: 0 tokens

      step_2_statistical:    # 条件执行，0 LLM Token
        - condition: "task_steps >= 5"  # 足够数据才有统计意义
        - run: tier2_checks  # 质量退化趋势
        - if: decay_detected → replan_with_new_strategy
        - cost: 0 tokens

      step_3_llm_reflection: # 仅在高风险时执行
        - condition: >
            (task_cost > budget * 0.5 AND confidence < 0.6)
            OR parent_explicitly_requested
            OR task_category == "high_stakes"
        - run: tier3_llm_assessment
        - budget_cap: "min(task_cost * 0.1, 500 tokens)"
        - cost: 100-500 tokens

      step_4_lesson_extraction:  # 仅成功且有价值时执行
        - condition: "task_succeeded AND novel_strategy_used"
        - run: extract_lesson_to_semantic_memory
        - cost: 0-200 tokens

      default_path:  # 大多数任务走这条路径
        # step_1通过 → 直接COMPLETED
        # 预期Token成本：0

  # 运行时接口
  interface:
    check_deterministic(recent_trace) -> List[Alert]     # Tier 1, 每步必调
    check_statistical(trace_window) -> Optional[Trend]   # Tier 2, 周期调用
    assess_with_llm(step_result) -> Optional[QualityScore]  # Tier 3, 可选
    reflect(context) -> ReflectionOutcome
    extract_lesson(episode) -> Optional[SemanticMemoryEntry]
```

**设计决策**：元认知模块是AgentOS v5.0相对于前版本的核心创新之一。它不是一个"高级功能"，而是认知完整性的必要条件。缺少元认知的Agent就像缺少疼痛感的人——可以行动，但无法识别自身行为的问题。但"疼痛感"不必一开始就精确——知道"我在原地打转"（循环检测）和"我快没钱了"（预算异常）就已经比大多数现有Agent框架前进了一大步。高级的质量评估能力是渐进增强的，不是Phase 1的阻塞项。

**反思决策树的关键作用**：FSM中`EXECUTING → REFLECTING`是必经状态，但"进入REFLECTING"≠"执行LLM调用"。上述`execution_protocol`的决策树消除了这个实现歧义——实现者看到决策树，就不会把REFLECTING误实现为"每次都做LLM自评"。**预期大多数任务的反思阶段Token成本为0**（Tier 1确定性检查通过即退出，具体比例待Phase 1验证），只有高风险、高成本、低置信度且具备外部验证信号的任务才会触发Tier 3的LLM反思，且反思成本被硬性限制在任务成本的10%以内。这是公理6在元认知维度的具体体现。

### 6.5 Agent Kernel的生命周期

```
                      AgentOS Kernel管理（生命周期FSM）
 ┌──────────────────────────────────────────────────────────────┐
 │                                                              │
 │  [创建请求] → PENDING ──(资源分配成功)──→ INITIALIZING        │
 │                  │                           │               │
 │                  │(资源不足)                   │(初始化完成)    │
 │                  ▼                           ▼               │
 │               REJECTED                    READY              │
 │                                             │                │
 │                                   ┌─(让出/抢占)─┐            │
 │                                   │            │             │
 │                          (收到TASK)▼            │             │
 │                              RUNNING ──────────┘             │
 │                                 │                            │
 │                    ┌────────────┼────────────┐               │
 │                    │            │            │               │
 │              (阻塞调用)    (任务完成)    (错误/超时)           │
 │                    ▼            ▼            ▼               │
 │                 BLOCKED     COMPLETED     FAILED             │
 │                    │            │            │               │
 │               (事件到达)   (结果保留)    (重试策略)            │
 │                    │            │            │               │
 │                    ▼            ▼            ▼               │
 │                 READY       DRAINING     PENDING             │
 │                              │          (或TERMINATED)       │
 │                              ▼                               │
 │   ┌─────────┐          TERMINATED                            │
 │   │SUSPENDED│              │                                 │
 │   │         │              ▼                                 │
 │   │(checkpoint          [资源回收]                            │
 │   │ 已保存)  │                                                │
 │   └─────────┘                                                │
 │   从RUNNING/BLOCKED/READY经suspend()进入                      │
 │   经resume(checkpoint)恢复到READY                              │
 └──────────────────────────────────────────────────────────────┘
```

**与v4.0/v4.5的区别**：

1. 新增`REJECTED`状态：当系统资源不足时，创建请求被拒绝而非无限等待。这是对资源有限性的诚实响应。
2. 新增`DRAINING`状态：COMPLETED后不立即销毁，而是进入排水期——在此期间其他Agent仍可查询该Agent的结果和TRACE。排水期结束后进入TERMINATED。
3. `BLOCKED`状态的明确化：Agent在等待LLM响应、子任务结果或外部事件时进入BLOCKED。此时不消耗CPU时间片，但保留内存状态。
4. 新增`SUSPENDED`状态：当调度器需要回收资源或执行抢占时，通过`suspend()`将Agent的完整状态（包括认知FSM快照）保存为checkpoint，进入SUSPENDED。后续通过`resume(checkpoint)`恢复到READY状态。SUSPENDED与BLOCKED的区别在于：BLOCKED保留内存状态等待特定事件，SUSPENDED将状态持久化后释放内存。

**两套FSM的关系与一致性协议**：系统中存在两层状态机——上述生命周期FSM由AgentOS内核管理，6.4.3节中的认知FSM（IDLE→PERCEIVING→PLANNING→...）由Agent Kernel内部的控制模块管理。二者的映射关系为：

| 生命周期状态 | 认知FSM状态 | 说明 |
|------------|-----------|------|
| READY | IDLE | Agent已就绪，等待调度器分配执行资源 |
| RUNNING | PERCEIVING / PLANNING / EXECUTING / REFLECTING | Agent正在执行，认知FSM在内部状态间流转 |
| BLOCKED | WAITING | Agent主动让出CPU，等待外部事件（LLM响应、子任务结果） |
| SUSPENDED | 任意（已快照） | 认知FSM的完整状态保存在checkpoint中 |
| COMPLETED | COMPLETED | 认知FSM完成 → 触发生命周期进入COMPLETED |
| FAILED | FAILED | 认知FSM失败 → 触发生命周期进入FAILED |

**两套FSM的不一致性处理**：在分布式系统中，两层状态机的不同步是常态而非异常。核心处理原则是：**生命周期FSM是权威来源**——内核可以kill任何Agent，但Agent不能自行复活。当两者不一致时（如认知FSM已COMPLETED但生命周期FSM未收到信号），heartbeat中携带的`cognitive_state`使内核能够检测不一致并强制同步。`suspend()`中断认知FSM时，checkpoint保存完整的认知状态（包括进行中的计划草案和上下文快照），恢复时检查上下文是否已过期——如已过期则回到PLANNING入口重新获取。

### 6.6 Execution Handle：内核眼中的Agent

AgentOS内核通过Execution Handle管理每个Agent Kernel实例，类似于操作系统的PCB（进程控制块）。

```python
@dataclass
class ExecutionHandle:
    agent_id: str                              # 全局唯一标识
    kernel_ref: AgentKernel                    # 指向用户空间的Kernel实例
    lifecycle_state: LifecycleState            # PENDING / READY / RUNNING / ...
    schedule_params: ScheduleParams            # 优先级、时间类别
    constraint_bundle: ConstraintBundle        # 从父Agent继承的约束包
    resource_usage: ResourceMetrics            # Token消耗、延迟、内存
    checkpoint_ref: Optional[CheckpointID]     # 暂停时保存的状态快照
    parent_handle: Optional[ExecutionHandle]   # 父Agent的句柄（用于级联追踪）
    children_handles: List[ExecutionHandle]    # 子Agent句柄列表
    fabric_subscriptions: List[TopicPattern]   # 已订阅的Event Fabric Topic
    created_at: Timestamp
    last_active_at: Timestamp
```

Handle对外暴露的控制原语（由Scheduler调用，Agent无权直接调用）：

| 原语 | 语义 | 适用场景 |
|-----|------|---------|
| `suspend()` | 暂停执行，保存checkpoint到持久化存储 | 抢占、资源回收 |
| `resume(checkpoint)` | 从checkpoint恢复执行 | 从暂停恢复、故障恢复 |
| `kill(graceful=True)` | 终止Agent。graceful=True允许完成当前原子步骤 | 超时、手动终止 |
| `inject_event(event)` | 向Agent的Local Event Channel注入事件 | 解除BLOCKED状态、系统通知 |


---


## 七、Unified Event Fabric：系统的信息中枢

### 7.1 设计理念：统一协议，而非统一总线

v3.0提出的"统一总线+作用域投影"是一个正确的架构方向。v5.0将其重新命名为**Event Fabric（事件织网）**，但必须首先明确一个关键区分：

> **Event Fabric的核心价值是统一的消息信封协议（Message Envelope Protocol），而非统一的物理传输通道。**

一个合理的质疑是：为什么需要"统一事件协议"？替代方案——function call + shared state + direct async calls + localized message passing——在工程上完全可行。但考虑可观测性需求：当一个多Agent任务出问题时，你需要追踪"是哪个Agent在哪一步做了什么导致了失败"。在替代方案中，函数调用有函数调用的tracing，状态变更有状态变更的hook，异步回调有回调的关联，MCP工具调用有工具调用的span——**四种交互模式，四套追踪方式，四个集成点**。要实现"一个多Agent任务的完整因果链在Jaeger中可视化"，你需要确保这四套追踪使用相同的`trace_id`语义、相同的因果链关联、相同的上下文传播。手动统一这四套东西——本质上就是在重建Event Fabric的信封协议层。

因此，公理2（通信统一性）要求的是**协议统一**，不是物理通道统一。所有信息流——无论物理上走函数调用、asyncio.Queue还是Redis Stream——都必须携带相同的Message Envelope（包含trace、constraints、scope等结构化字段）。差异仅在于Event的Topic命名空间（决定可见性）和交付语义（决定可靠性）。Phase 1中，Event Fabric的实现就是`asyncio.Queue`加统一信封——其价值在信封格式，不在Queue本身。

### 7.2 作用域模型

```
┌──────────────────────────────────────────────────────────────┐
│                    Global Scope（全局作用域）                    │
│                                                               │
│  Topic命名空间: global.*                                       │
│  可见性: 所有Agent（受订阅规则过滤）                             │
│  典型用途: 跨Agent任务委托、全局记忆投影、系统事件                 │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 Agent Scope（Agent作用域）                 │  │
│  │                                                          │  │
│  │  Topic命名空间: agent.{agent_id}.*                        │  │
│  │  可见性: 仅该Agent自身及其直接父Agent                      │  │
│  │  典型用途: 内部模块通信、步骤追踪、工作记忆更新              │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │               Task Scope（任务作用域）                     │  │
│  │                                                          │  │
│  │  Topic命名空间: task.{task_id}.*                          │  │
│  │  可见性: 参与该任务的所有Agent                              │  │
│  │  典型用途: 任务生命周期事件、子任务协调、结果汇聚             │  │
│  └─────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**三个作用域的设计理由**：

- **Global Scope** 解决"Agent如何发现和联系彼此"的问题
- **Agent Scope** 解决"Agent内部模块如何通信"的问题（取代v3.0的Local Bus）
- **Task Scope** 解决"参与同一任务的多个Agent如何协调"的问题

Task Scope是v5.0新增的作用域，它解决了v3.0中的一个盲区：当多个Agent协作完成同一任务时，它们需要一个任务级别的共享信息空间，但这既不是Global（太宽）也不是Agent（太窄）。

### 7.3 统一消息协议：Message Envelope

所有事件遵循同一个信封格式：

```json
{
  "envelope": {
    "event_id": "evt_a1b2c3d4",
    "correlation_id": "task_xyz789",
    "causation_id": "evt_prev_step",
    "timestamp": "2026-04-09T10:30:00.000Z",
    "source": {
      "agent_id": "agent_research_01",
      "module": "action"
    },
    "topic": "task.xyz789.step_completed",
    "scope": "task"
  },

  "payload": {
    "type": "RESULT",
    "data": { }
  },

  "constraints": {
    "budget": {
      "tokens_remaining": 3500,
      "time_remaining_ms": 45000,
      "cost_remaining_usd": 0.03
    },
    "depth": {
      "current": 2,
      "max": 4
    }
  },

  "trace": {
    "span_id": "span_abc",
    "parent_span_id": "span_parent",
    "trace_id": "trace_root"
  }
}
```

**与v3.0的区别**：

1. **新增`causation_id`**：记录"是什么事件导致了这个事件"。这使得完整的因果链追踪成为可能——对于调试多Agent系统至关重要。
2. **新增`trace`字段**：内建OpenTelemetry兼容的分布式追踪标识。追踪不是事后添加的观测层，而是消息协议的原生组成部分。
3. **`scope`字段显式化**：不再依赖Topic前缀推断作用域，而是显式声明。

### 7.4 Payload设计范式：结构化短格式与上下文句柄

统一Envelope减少的是系统复杂度，不自动减少LLM Token。决定Token成本的不是有没有统一信封，而是`payload.data`有多长、是结构化的还是自然语言的、是否支持"引用已有上下文"而非"展开全文"。公理6要求我们在协议层面定义payload的**默认表示形式**——否则实现者的默认选择是自然语言全文，整个系统的Token效率基线就被拉低。

```yaml
payload_design_principles:
  default_representation: structured_short  # 默认：结构化短格式
  escalation_to_natural_language: explicit   # 升级到自然语言需显式请求

  # 各消息类型的默认payload格式
  PLAN:
    default: { dag: DAGStructure, budget_allocation: Map, success_criteria: list }
    avoid: 自然语言长文本计划描述
    text_rendering: only_when_entering_llm_context
    typical_tokens: 100-500

  TASK:
    default: { objective: "string(<100chars)", params: Map, context_handle: HandleID }
    avoid: 重复父Agent的完整计划描述
    context_resolution: "接收方通过context_handle按需拉取"
    typical_tokens: 50-200

  RESULT:
    default: { status: enum, output: structured_data, evidence_handle: HandleID }
    avoid: 执行过程的完整自然语言叙述
    detail_on_demand: "父Agent可通过evidence_handle拉取详细轨迹"
    typical_tokens: 30-150

  ERROR:
    default: { error_code: string, summary: "string(<200chars)", context_handle: HandleID }
    avoid: 完整的错误上下文堆栈内联
    typical_tokens: 30-200

  EVENT:
    typical_tokens: 20-100
  MEMORY:
    typical_tokens: 50-300
  TRACE:
    typical_tokens: 20-50  # 采样后

  # 上下文句柄机制——解决"上下文复印机"问题的核心
  context_handle:
    description: "轻量引用，指向存储在Memory中的完整内容"
    types:
      - task_handle: "引用任务的完整定义"
      - constraint_handle: "引用约束包的完整内容"
      - summary_handle: "引用摘要的完整文本"
      - evidence_handle: "引用执行轨迹或证据链"
    resolution: "接收方通过Memory/Context Scheduler按需拉取最小必要内容"
    benefit: "'系统间传递的信息量'与'喂给LLM的信息量'解耦"
```

**上下文句柄是本章最关键的设计**。在多层级联架构（Strategist → Tactician → Executor）中，如果每一层都在payload中内联完整的任务描述、约束文本和中间结果，同一信息会被多次重述——任务目标在委托时写一版，子Agent翻译成工作上下文时写一版，结果回传时再解释一版。句柄化传递从根本上解决了这个"上下文复印机"问题：消息在系统中流转时只携带轻量引用，只有在真正需要进入LLM上下文时，才由Context Scheduler拉取并渲染最小必要内容。

**预期收益**：多层级联场景下，消息传递的Token消耗降低60-80%。

**与现有架构的兼容性**：完全兼容——句柄走Event Fabric，内容存Memory，准入由Context Scheduler控制。不需要新增核心抽象。

**实现复杂度的诚实声明**：上下文句柄的核心难点不在于句柄-内容的映射关系（这只是一个KV存储），而在于`resolution`阶段的`minimal_necessary`策略——"给定当前决策上下文，从Memory中检索最相关的最小子集"。这本质上是一个**语义检索与摘要子系统**，与8.4节描述的上下文调度器（Context Scheduler）中的`relevance_scoring`和`handle_resolution`是同一问题的两面。实现这个子系统需要：(1) 语义相关性评估（当前任务状态 × 候选内容的匹配度）；(2) 信息增益判断（候选内容是否为已有上下文的增量）；(3) Token预算感知的截断策略。Phase 1中，`minimal_necessary`可以退化为"按recency排序+Token预算截断"的简单实现；完整的语义检索能力是Phase 2+的渐进增强目标。本文不隐瞒这一复杂度——上下文句柄的价值在Phase 1的简单实现中就能体现（避免全文内联传递），但其完整潜力的释放依赖于语义检索能力的成熟。

**结构化短格式的语义密度权衡**：TASK的`objective: "string(<100chars)"`不是任务描述的总限制——100字符是**inline路由信息**，复杂任务的完整描述通过`context_handle`引用。inline部分只需让接收方知道这是什么类型的任务，以便决定是否接受和以什么模式处理。详情按需拉取，而非省Token传递不完整描述。

### 7.5 消息类型与交付语义

| 类型 | 语义 | 交互模式 | 交付保证 | 顺序保证 | 典型payload Token |
|-----|------|---------|---------|---------|-----------------|
| `TASK` | 任务委托——"请做这件事" | Request（期待RESULT回复） | At-least-once | 单Agent内FIFO | 50-200 |
| `RESULT` | 任务结果——"这是结果" | Response（关联到TASK） | Exactly-once（幂等接收） | 无（通过correlation_id匹配） | 30-150 |
| `PLAN` | 计划下达——"请按这个计划做" | Push（带约束包） | At-least-once | FIFO | 100-500 |
| `EVENT` | 状态变更通知——"这件事发生了" | Pub/Sub | At-most-once | 无 | 20-100 |
| `MEMORY` | 记忆投影——"我学到了这个" | Pub（投影到全局） | At-least-once | 无 | 50-300 |
| `TRACE` | 执行追踪——"我正在做这个" | Fire-and-forget | Best-effort | 无 | 20-50 |
| `ERROR` | 错误升级——"出问题了" | Push（高优先级） | At-least-once | 无 |

**TASK与PLAN的区别**：`TASK`是原子级的工作委托——"做这一件事并返回结果"，构成请求-响应对（TASK→RESULT），通过`correlation_id`关联。`PLAN`是结构化的工作分解——"按此计划执行这组步骤"，它包含一个子任务DAG和对应的约束分配，接收方根据PLAN中的结构逐步发出TASK消息。在三层架构中的典型流转为：Strategist通过`PLAN`向Tactician下达包含DAG结构和预算分配的计划，Tactician将PLAN拆解后通过`TASK`向Executor下达具体的工具调用任务。PLAN不期待单一RESULT回复，而是通过Task Scope中的进度事件追踪整体完成度。

发送TASK的Agent可以选择同步等待（进入BLOCKED状态）或异步处理（继续执行，后续通过回调处理RESULT）。`RESULT`的Exactly-once通过接收方的幂等性实现——记录已处理的`event_id`，重复到达时丢弃。

### 7.6 投影机制

投影定义了Agent内部事件如何"升维"到更宽的作用域，以及外部事件如何"降维"到Agent内部。

```yaml
projection_rules:
  uplink:   # Agent Scope → Task/Global Scope
    - agent_topic: "agent.*.step_completed"
      target_topic: "task.{task_id}.progress"
      transform: summarize       # 摘要后投影，避免信息过载
      sampling: 1.0              # 所有步骤完成事件都投影

    - agent_topic: "agent.*.memory.projection"
      target_topic: "global.memory.{domain}"
      filter: "entry.reuse_count > 3"

    - agent_topic: "agent.*.trace.*"
      target_topic: "global.trace.{agent_id}"
      sampling: 0.1              # 10%采样——TRACE量大但非关键

  downlink:  # Global/Task Scope → Agent Scope
    - source_topic: "task.{task_id}.assigned"
      agent_topic: "agent.{agent_id}.task.incoming"
      filter: "target_agent == agent_id"

    - source_topic: "global.event.system.*"
      agent_topic: "agent.{agent_id}.system_event"
      filter: "relevant_to(agent_id)"
```

### 7.7 流控与降维

无约束的多Agent通信量随Agent数量呈O(n²)增长。Event Fabric必须内建流控机制：

**生产者侧限流**：每个Agent的事件产出受速率限制。超限事件按优先级丢弃——TRACE最先丢弃，ERROR最后丢弃。

**消费者侧聚合**：在时间窗口内对同类事件进行聚合。例如，5秒内收到同一Agent的10条进度通知，聚合为1条摘要。

**背压传播**：当下游消费者处理不过来时，背压信号沿Event Fabric传播，使上游生产者降速。这防止了"消费者溺水"的问题。

**优先级队列**：Event Fabric维护多级优先级队列。ERROR和TASK占据高优先级通道，TRACE占据低优先级通道且允许溢出丢弃。

```yaml
flow_control:
  producer:
    max_events_per_second: 100
    burst_allowance: 200
  consumer:
    aggregation_window_ms: 5000
    deduplication: true
  priority:
    - { topics: ["*.error.*", "*.task.*"], weight: 10, drop_on_overflow: false }
    - { topics: ["*.result.*", "*.plan.*"], weight: 5, drop_on_overflow: false }
    - { topics: ["*.event.*", "*.memory.*"], weight: 3, drop_on_overflow: true }
    - { topics: ["*.trace.*"], weight: 1, drop_on_overflow: true }
```

### 7.8 演进注释：何时需要控制面/数据面分离

成熟的分布式系统（如Kubernetes、Service Mesh）通常将控制面（调度、配置、健康检查）和数据面（实际业务数据流转）物理分离。当前Event Fabric有意选择统一——在Agent数量为个位到百位的规模下，一套协议带来的工程简洁性远超分离带来的性能优势。

但当以下条件**同时满足**时，应考虑将TRACE通道物理分离为独立的数据管道：
- Agent数量超过500+
- TRACE事件占总事件量的80%以上
- TRACE的延迟不敏感特性与TASK/ERROR的延迟敏感需求产生显著资源竞争

分离方式：将TRACE事件导向独立的日志管道（如OpenTelemetry Collector），Event Fabric仅保留TASK/RESULT/PLAN/ERROR/MEMORY/EVENT等控制语义的消息。这种分离可以在不改变Message Envelope格式的前提下完成——只需在路由层按消息类型分流。


---


## 八、Constraint-Propagating Hierarchy：约束传播式分层控制

### 8.1 为什么需要分层

Agent系统的分层不是组织架构的模仿，而是**时间尺度分离**的工程需要：

- **战略层（秒到分钟级）**：理解复杂目标、分解为子任务DAG、分配资源预算——需要强推理能力（大模型）、低调用频率
- **战术层（百毫秒到秒级）**：实例化子任务、动态调整计划、聚合子结果——需要中等推理能力、中等频率
- **执行层（毫秒到百毫秒级）**：调用工具、解析响应、管理重试——需要快速响应（小模型或规则）、高频率

这三个时间尺度的自然分离导出了三层控制架构。但v5.0对此做了一个关键的灵活化处理：

> **三层是逻辑角色，不是物理实体。** 一个Agent Kernel可以同时扮演多个角色。简单任务中，单个Kernel可以同时承担战略、战术和执行的全部职责。只有当任务复杂度超过单Agent的处理能力时，才需要将角色分配给不同的Kernel实例。

这直接呼应了"单Agent优先"原则——不强制所有任务都必须经过三层流转。

### 8.2 三层控制架构

```
┌────────────────────────────────────────────────────────────────┐
│                     Strategist（战略角色）                         │
│  时间尺度: 秒-分钟 · LLM: 强推理模型 · 调用频率: 极低              │
│                                                                 │
│  职责: 理解复杂目标 · 分解DAG · 分配预算 · 跨Agent编排 · 反思      │
│  权限: 创建子任务 · 委托给Tactician · 修改全局计划                  │
│  约束: 不直接调用工具 · 不访问子Agent内部状态                       │
│                          ↓ PLAN消息（携带约束包）                   │
├────────────────────────────────────────────────────────────────┤
│                     Tactician（战术角色）                          │
│  时间尺度: 百毫秒-秒 · LLM: 性价比模型 · 调用频率: 中等             │
│                                                                 │
│  职责: 任务实例化 · 上下文调度 · 动态重规划 · 结果校验               │
│  权限: 在预算内重试 · 选择备选工具 · 重排子任务顺序                  │
│  约束: 不修改DAG结构 · 不突破预算 · 不放宽行动空间                  │
│                          ↓ TASK消息（携带收窄后的约束包）            │
├────────────────────────────────────────────────────────────────┤
│                     Executor（执行角色）                           │
│  时间尺度: 毫秒-百毫秒 · 推理: 规则引擎/小模型 · 调用频率: 高频     │
│                                                                 │
│  职责: 调用工具 · 解析响应 · 局部重试 · 状态上报                    │
│  权限: 在local_policy内自主决策 · 缓存 · 格式转换                  │
│  约束: 不跨出action_space · 必须遵守escalation · 不委托子任务       │
└────────────────────────────────────────────────────────────────┘
```

**角色级Token治理**（公理6在分层架构中的具体化）：不同角色有不同的Token预算天花板和注入策略，防止执行层把大量Token花在"解释自己为什么这么做"上。

```yaml
role_token_governance:
  strategist:
    model_tier: strong               # 大模型——需要强推理能力
    max_input_tokens: 8000           # 允许较大上下文（需要全局视角）
    max_output_tokens: 2000          # 允许生成长文本分析
    tool_injection: names_only       # 只注入工具名称（不直接调工具）
    kernel_mode: full_mode           # 完整Kernel模式

  tactician:
    model_tier: balanced             # 性价比模型
    max_input_tokens: 4000
    max_output_tokens: 800
    tool_injection: candidate_subset # 注入候选工具子集
    kernel_mode: standard_mode       # 标准Kernel模式

  executor:
    model_tier: fast                 # 小模型或规则引擎
    max_input_tokens: 2000
    max_output_tokens: 300           # 强制短输出
    tool_injection: single_tool      # 只注入当前步骤的工具
    output_format: structured_only   # 禁止自然语言长篇输出
    kernel_mode: lightweight_mode    # 轻量Kernel模式
```

角色级治理与6.2节的Kernel运行模式自然对应：Strategist用full_mode，Tactician用standard_mode，Executor用lightweight_mode。这使得三层架构的Token效率从"取决于实现者的习惯"提升为"协议级的默认行为"。

### 8.3 约束传播：核心机制

约束传播是AgentOS分层控制的灵魂。它的核心规则极其简单：

> **约束只能收窄，不能放宽。**

当Strategist通过PLAN消息将子任务委托给Tactician时，消息必须携带一个**约束包（Constraint Bundle）**：

```yaml
constraint_bundle:
  # 资源预算——从父Agent的剩余预算中划拨
  budget:
    tokens: 5000          # 允许消耗的最大Token数
    time_ms: 60000        # 允许的最大执行时间
    cost_usd: 0.05        # 允许的最大成本
    llm_calls: 10         # 允许的最大LLM调用次数

  # 行动空间——只能是父Agent行动空间的子集
  action_space:
    allowed_tools: ["web_search", "fetch_page"]
    forbidden_patterns: ["*execute*", "*delete*", "*send*"]

  # 级联深度——防止无限递归
  depth:
    current: 1
    max: 4

  # 升级策略——定义失败时的处理方式
  escalation:
    on_budget_exceeded: "abort_and_return_partial"
    on_action_violation: "block_and_escalate"
    on_timeout: "return_best_effort"

  # 上下文传递约束（公理6：最小Token传递是协议级要求）
  context_transfer:
    mode: handle_by_default          # 默认传句柄，不传全文
    max_inline_tokens: 200           # 内联内容不超过200 tokens
    require_structured_format: true  # 强制要求结构化格式
```

**接收方必须执行的校验**：

1. **预算检查**：估算任务所需资源。若预算明显不足，拒绝执行并返回`BUDGET_INSUFFICIENT`。
2. **行动空间取交集**：将自身的`action_space`与传入约束取交集。结果只能更窄，不能更宽。
3. **深度递增**：`depth.current += 1`。若达到`depth.max`，禁止继续委托子任务。
4. **升级策略合并**：将传入的升级策略与自身策略合并，取更严格的一方。

**约束协商（Constraint Negotiation）**：约束"只收不放"是安全公理，但不意味着子Agent只能被动接受不合理的约束。AgentOS提供**接受前协商**机制：

```yaml
negotiation_protocol:
  # 子Agent在accept之前可以发起negotiation
  on_receive_task:
    step_1: estimate_resource_need(task)          # 估算任务所需资源
    step_2: compare_with_budget(estimate, bundle)  # 对比约束
    step_3:
      if estimate <= budget:
        accept_and_execute
      elif estimate <= budget * 1.5:               # 略微不足
        negotiate:
          send_to_parent:
            type: "BUDGET_NEGOTIATION"
            requested: { tokens: estimate, reason: "..." }
            minimum_viable: { tokens: min_estimate }
          parent_options:
            - grant_additional(amount)             # 同意追加（从自身预算划拨）
            - reject_and_simplify(simplified_task) # 拒绝并简化任务
            - reject_and_abort                     # 拒绝并放弃
      else:                                        # 严重不足
        reject_with_estimate(BUDGET_INSUFFICIENT, estimate)
```

协商的关键约束：(1) 父Agent追加预算必须从**自身剩余预算**中划拨——约束的总量在全局层面仍然只收不放；(2) 行动空间（allowed_tools）不可协商——安全边界是硬约束；(3) 协商有次数上限（默认1轮），防止协商本身成为协调税。

**约束的历史学习**（Phase 3+）：基于同类任务的历史Token消耗分布，父Agent在委托时自动估算合理预算，减少协商频率：

```yaml
budget_estimation:
  method: "historical_percentile"
  percentile: 80                 # 以历史P80消耗作为默认预算
  fallback: "parent_heuristic"   # 无历史数据时由父Agent启发式估算
```

**约束的内核级强制执行**：约束不依赖Agent的"自觉遵守"。AgentOS内核在以下系统调用点进行强制校验：

- **工具调用前**：检查工具是否在`allowed_tools`中
- **Token消耗后**：检查是否超出`budget.tokens`
- **委托子任务前**：检查`depth.current < depth.max`
- **执行时间**：由Scheduler的超时机制强制执行

### 8.4 上下文调度：Token-Aware的决策上下文管理

上下文调度本质上是记忆管理与资源管理的交叉领域，将其放在分层控制章节而非记忆架构章节是因为：**上下文窗口的内容决定了每一层Agent的决策质量**。在分层架构中，战略层、战术层、执行层各需不同粒度的上下文——战略层需要全局概览，执行层需要具体参数。上下文调度直接服务于分层决策的有效性。同时，上下文窗口中Token配额的管理也是约束传播的延伸——父Agent的Token预算约束着子Agent可用的上下文空间。

在LLM Agent系统中，"上下文窗口"是最稀缺的资源之一。每次LLM调用时，送入的上下文决定了决策质量，但上下文的Token消耗直接影响成本和延迟。

AgentOS将上下文管理定义为一组可配置的调度策略：

```yaml
context_scheduler:
  # 准入策略：什么信息进入LLM调用的上下文
  admission:
    always_include:
      - system_prompt
      - current_task_definition
      - active_constraints
    priority_include:
      - recent_step_results           # 按相关性排序
      - relevant_memory_entries       # 按重要性排序
      - error_context                 # 如果有错误，优先包含
    never_include:
      - other_agent_internal_trace
      - expired_memory
    # 信息增益评估（公理6：每个Token都应对当前决策有信息增益）
    relevance_scoring:
      method: recency_weighted_similarity
      min_score_for_admission: 0.3
      score_factors:
        - recency: exponential_decay(half_life=3_steps)
        - task_relevance: cosine_similarity(item, current_task)
        - novelty: 1.0 - max_similarity_to_already_included

  # 压缩策略：上下文使用率高时如何压缩
  compression:
    trigger_at: 0.7                   # Token使用率超70%触发
    strategies:
      - type: "summarize_old_steps"
        target: "step_results older than 3 steps"
        method: incremental_summary   # 使用增量摘要，不重写全文
      - type: "evict_low_relevance"
        target: "memory entries with relevance < 0.3"
      - type: "deduplicate_semantic"  # 语义去重
        target: "entries with cosine_similarity > 0.9"
        action: keep_most_recent

  # 预留策略：为关键信息预留Token
  reservation:
    for_system_prompt: 500
    for_tool_results: 2000
    for_error_handling: 500

  # 句柄解析策略：收到context_handle时如何按需拉取
  handle_resolution:
    strategy: minimal_necessary       # 只拉取当前决策所需的最小内容
    cache_resolved: true              # 缓存已解析的内容，避免重复拉取
    max_resolved_tokens: 0.3 * available_context  # 句柄解析内容不超过可用上下文的30%
```

上下文压缩和记忆增强领域的研究（MemGPT, LongMem等）表明，精心设计的上下文调度可以在大幅减少Token使用的情况下接近甚至超越全上下文基线。AgentOS将这一洞见制度化——上下文调度不是"性能优化技巧"，而是架构的核心组件。

**信息增益评估**是公理6在上下文调度中的直接推论。仅靠"Token使用率触发压缩"是被动的——等到70%再压缩，已经有大量Token被浪费在无关信息上。增加`relevance_scoring`和`deduplicate_semantic`，使上下文调度从"被动限流"升级为"主动选优"：只有对当前决策有信息增益的内容才能进入上下文，语义重复的内容只保留最新版本。预期收益：上下文中的冗余信息减少20-40%。


---


## 九、自相似组合：何时递归，何时不递归

### 9.1 重新审视"自相似性"

v3.0对自相似性有一个精确的数学定义。v5.0保留这个概念，但增加了一个关键的务实约束：

> **自相似性是能力，不是义务。** 系统中的每个Agent都*有能力*被递归组合（因为它们都是结构完整的Kernel实例），但不是每个任务*需要*递归。

**自相似性的架构保障**：

1. 系统中的每个可调度实体都是完整的Agent Kernel实例（五模块结构）
2. 任意两个Agent之间的通信都使用同一套Message Envelope协议
3. 约束包沿委托链逐层传递并收窄
4. 级联深度受`max_depth`限制

**何时应该递归组合**：

| 场景 | 理由 | 示例 |
|-----|------|------|
| 任务可并行分解 | 多Agent并行执行子任务可获80%+性能提升 | 同时搜索5个数据源 |
| 需要异构能力 | 单Agent难以同时精通多个领域 | 编程Agent + 测试Agent + 文档Agent |
| 上下文窗口不足 | 单Agent的上下文无法容纳全部信息 | 分析100页文档 |
| 需要隔离的安全边界 | 不同子任务需要不同的权限级别 | 代码执行 vs 网络访问 |

**何时不应该递归组合**：

| 场景 | 理由 | 建议 |
|-----|------|------|
| 顺序推理链 | 多Agent协调税导致39-70%性能下降 | 单Agent + 足够上下文 |
| 简单工具调用 | 不需要完整Kernel的开销 | 直接调用工具（lightweight_mode） |
| 低延迟要求 | 每级级联增加数百毫秒到数秒延迟 | 扁平化处理 |
| **预估协调Token > 任务有效Token的30%** | 协调税超过收益阈值（公理6） | 单Agent + 工具直接调用 |

**委托决策的Token成本评估**（公理6的量化推论）：

```
delegation_payback = (single_agent_cost - multi_agent_total_cost) / multi_agent_total_cost

multi_agent_total_cost = Σ(sub_agent_effective_tokens)
                        + Σ(sub_agent_fixed_overhead)    # 系统提示+工具schema+角色定义
                        + coordination_tokens             # 任务描述+约束传递+结果回传+摘要

建议阈值: delegation_payback > 0.2（即多Agent至少比单Agent节省20%总成本才值得委托）
```

在实际运行中，Strategist在决定是否委托时，应结合3.2节的量化框架和上述公式，估算委托的Token成本收益比。当协调开销可能超过收益时，宁可让单Agent在lightweight_mode或standard_mode下完成任务。

### 9.2 级联工作流示例

```
用户: "分析这三篇论文，比较它们的方法论差异并生成报告"

[ResearchAgent — Strategist角色]
├── 理解目标: 多论文比较分析
├── 分解DAG:
│   ├── 子任务1: 分析论文A → delegate(PaperAnalysisAgent, 约束: tokens=3000)
│   ├── 子任务2: 分析论文B → delegate(PaperAnalysisAgent, 约束: tokens=3000)
│   ├── 子任务3: 分析论文C → delegate(PaperAnalysisAgent, 约束: tokens=3000)
│   └── 子任务4: 比较&生成报告 → 自行完成（需要全局视角）
├── 子任务1-3并行执行
│
├── [PaperAnalysisAgent — Executor角色] (×3 并行)
│   ├── 校验约束: tokens=3000, depth=2/4 ✓
│   ├── 调用工具: fetch_paper → parse_pdf → extract_methods
│   ├── 记忆: 将分析结果写入episodic_memory
│   ├── 元认知: 评估分析质量 confidence=0.85
│   └── 返回RESULT: 结构化分析报告
│
├── 聚合三份分析结果
├── 执行比较分析（自行完成，不委托）
├── 元认知: 评估报告质量
└── 返回最终报告给用户
```

注意：子任务1-3是天然可并行的，适合多Agent；子任务4需要全局视角，由Strategist自行完成——这就是"单Agent优先"原则的具体体现。

### 9.3 级联稳定性保障

| 保障机制 | 实现方式 | 防御目标 |
|---------|---------|---------|
| **深度限制** | `depth.current`递增，达到`max`时禁止继续委托 | 防止无限递归 |
| **预算熔断** | 剩余Token低于阈值时禁止新的LLM调用 | 防止成本失控 |
| **超时熔断** | 每级Agent设置执行超时，超时后强制返回部分结果 | 防止无限等待 |
| **循环检测** | 在调用链中记录已访问的Agent类型，检测重复 | 防止循环委托 |
| **背压传播** | 当子Agent返回`BUDGET_EXCEEDED`时，父Agent不再委托新任务 | 防止"饿死"已在执行的子任务 |


---


## 十、记忆架构

### 10.1 记忆层次模型

AgentOS的记忆架构基于认知科学的记忆层次理论，但做了面向工程的适配：

| 记忆层 | 认知对应 | 存储特性 | 生命周期 | 可见性 |
|-------|---------|---------|---------|--------|
| **工作记忆** | 前额叶在线保持 | 内存Ring Buffer，容量受限 | 任务存续期 | 仅当前Agent |
| **情景记忆** | 海马体情景编码 | 追加日志，可持久化 | 会话/可跨会话 | 可投影到Task Scope |
| **语义记忆** | 皮层分布式存储 | KV + 向量索引，持久化 | 长期 | 可投影到Global Scope |

### 10.2 记忆与LLM上下文窗口的关系

这是现有Agent OS设计中常被忽略的关键问题。LLM的上下文窗口（128K-1M+ tokens）已经非常大，但这不意味着"记忆管理不再重要"——恰恰相反：

1. **长上下文不等于好上下文**。研究表明（"Lost in the Middle", Liu et al., 2024），LLM在超长上下文中对中间位置的信息检索能力显著下降。将所有记忆塞入上下文窗口不是最优策略。
2. **上下文是有成本的**。每个Token都消耗计算资源和费用。Token-aware的上下文管理直接影响系统的经济可行性。
3. **上下文是有延迟的**。更长的上下文意味着更高的首Token延迟（TTFT）。对于实时交互场景，精简的上下文更优。

因此，AgentOS的记忆架构明确将"上下文窗口管理"作为一等问题：

```
                     ┌─────────────────────────────┐
                     │   LLM上下文窗口               │
                     │   (Token-aware管理)           │
                     │                              │
                     │  ┌──────────────────────┐    │
                     │  │ 系统提示 (预留)       │    │
                     │  ├──────────────────────┤    │
                     │  │ 当前任务定义          │    │
                     │  ├──────────────────────┤    │
  工作记忆 ─────────►│  │ 近期步骤结果(摘要)    │    │
                     │  ├──────────────────────┤    │
  情景记忆 ─(检索)──►│  │ 相关历史片段          │    │
                     │  ├──────────────────────┤    │
  语义记忆 ─(检索)──►│  │ 相关知识条目          │    │
                     │  ├──────────────────────┤    │
                     │  │ 可用工具描述          │    │
                     │  ├──────────────────────┤    │
                     │  │ 预留空间(错误处理等)   │    │
                     │  └──────────────────────┘    │
                     └─────────────────────────────┘
```

上下文调度器（见8.4节）负责决定从三层记忆中检索什么、以什么形式（原文/摘要）放入上下文窗口、在Token预算紧张时淘汰什么。

**记忆检索的Token预算意识**（公理6的推论）：没有预算控制的记忆检索会变成"往上下文里塞噪音"——检索到的内容可能与当前决策无关，但仍然消耗Token并稀释有效信息的注意力权重。

```yaml
memory_retrieval_budget:
  per_call_budget:
    max_memory_tokens: 0.25 * available_context_tokens  # 记忆最多占上下文的25%
    max_entries: 5                                        # 每次最多检索5条
    prefer_summaries_over_originals: true                # 优先检索摘要而非原文
  retrieval_effectiveness_tracking:
    track: true
    metric: "was_retrieved_memory_referenced_in_output"   # 检索的记忆是否真正影响了输出
    low_hit_rate_action: "reduce_retrieval_count"         # 命中率低时减少检索
```

### 10.3 一致性模型

AgentOS采用**最终一致性**：

- **Agent内部**：工作记忆对自身立即可见（强一致）
- **投影到Task/Global Scope**：最终一致，延迟上限可配置（默认5秒）
- **冲突解决**：Last-Write-Wins（基于Lamport时间戳）

选择最终一致性的理由：Agent的记忆操作主要是追加写（轨迹记录），并发冲突写极少。强一致性的同步开销与LLM的高延迟、高成本特性矛盾。

### 10.4 记忆边界：隐私的架构保障

每个Agent拥有独立的记忆命名空间。跨Agent记忆共享必须通过显式投影规则（见7.6节），不允许直接读取。这是安全性的基础保障——如果Agent A能直接读取Agent B的内部记忆，那么行动空间约束就可以被绕过（通过读取敏感信息间接获取不应拥有的能力）。


---


## 十一、调度与资源管理

### 11.0 诚实声明：Agent调度 ≠ 传统OS调度

在详述调度机制之前，必须诚实地标注Agent调度与传统OS调度的**本质差异**，以校准读者的预期：

| 维度 | 传统OS调度 | Agent调度 | 影响 |
|------|-----------|----------|------|
| 执行单元可预测性 | 高——程序行为基本确定 | 低——LLM输出非确定性 | 无法基于历史精确预测执行时间 |
| 可中断性 | 高——任意指令边界可中断 | 极低——LLM API调用期间不可中断 | 抢占只能在LLM调用的间隙发生 |
| 时间片粒度 | 毫秒级（10ms） | 秒级（LLM调用100ms-60s） | "时间片"概念不适用 |
| 核心瓶颈资源 | CPU周期 | LLM API调用配额 + Token预算 | 调度的核心是"谁先获得下一次LLM调用" |

因此，AgentOS的"调度"更准确地说是**协程调度器 + 令牌桶限速器 + 优先级仲裁器**的组合。它的真实价值不在于精确的CPU时间分配（LLM调用的延迟远大于本地计算），而在于：

1. **并发度管理**：在LLM API rate limit下，决定同时运行多少个Agent
2. **优先级仲裁**：前台交互Agent vs 后台批处理Agent，谁先获得LLM调用配额
3. **阻塞/唤醒管理**：Agent等待外部事件时释放执行权，事件到达后恢复
4. **预算执行**：确保单个Agent不会耗尽全局Token预算

以下使用传统OS术语描述调度原语，是为了概念上的统一性，但读者应理解其实际语义已适配LLM执行模型的特点。

### 11.1 调度原语

AgentOS的调度器管理两类资源：**计算时间**（CPU/GPU）和**认知预算**（Token、LLM调用次数、费用）。

| 原语 | 语义 | 触发场景 |
|-----|------|---------|
| `admit(agent, priority)` | Agent进入就绪队列 | 创建完成、从BLOCKED恢复 |
| `dispatch(agent)` | 分配执行线程给Agent | 调度器选中 |
| `block(agent, wait_for)` | Agent让出CPU，等待指定事件 | 发起LLM调用、等待子任务RESULT |
| `unblock(agent, event)` | 将Agent移回就绪队列 | 等待的事件到达 |
| `yield(agent)` | Agent主动让出当前时间片但保持就绪 | 长计算中的协作让出点 |
| `preempt(agent)` | 强制剥夺执行权，保存checkpoint | 更高优先级Agent就绪 |

### 11.2 调度算法

MVP采用**协作式调度 + 优先级队列**，理由如下：

- Agent的执行模式以"调用LLM → 等待响应 → 处理结果"为主，自然产生阻塞点（yield point）
- LLM调用的延迟（100ms-10s）远大于传统进程的CPU时间片（10ms），抢占式调度的收益有限
- 协作式调度的实现复杂度远低于抢占式，适合MVP阶段

```yaml
scheduler:
  mode: "cooperative"           # cooperative / preemptive (Phase 2+)
  priority_classes:
    critical:    { weight: 10, max_queue_size: 5 }
    interactive: { weight: 5,  max_queue_size: 20 }
    background:  { weight: 1,  max_queue_size: 100 }
  starvation_prevention:
    boost_after_wait_ms: 30000  # 等待超30秒的Agent优先级提升
```

### 11.3 Token经济学

传统OS调度关注CPU时间；AgentOS还必须关注**Token消耗**——这是Agent系统最显著的运行成本。

```yaml
token_economics:
  # 全局Token预算池
  global_budget:
    total_tokens: 1000000      # 系统级Token预算
    alert_threshold: 0.8       # 80%消耗时告警
    hard_limit: 0.95           # 95%消耗时拒绝新任务

  # 每任务预算分配
  task_budget_policy:
    estimation: "model_based"  # 基于历史数据估算任务Token需求
    overcommit_ratio: 1.2      # 允许20%超额分配
    reclaim_on_completion: true # 任务完成后回收未使用预算

  # 成本追踪
  cost_tracking:
    granularity: "per_llm_call"
    metrics: ["input_tokens", "output_tokens", "model_tier", "latency"]
    export: "opentelemetry"

  # 重复前缀成本治理（公理6：防止固定开销失控）
  prefix_cost_governance:
    tracking:
      - system_prompt_tokens_per_call
      - tool_schema_tokens_per_call
      - role_description_tokens_per_call
      - constraint_template_tokens_per_call
    metrics:
      fixed_prefix_ratio: "prefix_tokens / total_input_tokens"
      alert_when: "fixed_prefix_ratio > 0.5"  # 固定前缀超过一半时告警
    optimization:
      provider_prefix_caching: true   # 利用Provider侧前缀缓存
      prompt_template_stabilization: true  # 稳定化提示模板以提高缓存命中
      shared_prefix_extraction: true  # 提取公共前缀
```

### 11.4 Provider侧缓存利用策略

LLM Provider Adapter是接入前缀缓存（Prefix Caching）和响应缓存的最佳位置。这是**最低成本、最高回报的Token效率优化**——不改变架构，只在Adapter层增强。

```yaml
provider_cache_strategy:
  prefix_stabilization:
    - 系统提示内容保持稳定（不随请求变化的部分前置）
    - 角色提示作为系统提示的一部分（而非每次动态拼接）
    - 工具schema按固定顺序排列
  response_caching:
    - 对确定性请求（相同输入+相同系统提示）缓存响应
    - cache_key: "hash(system_prompt + user_message + tools)"
    - ttl: configurable, default 1 hour
  expected_savings:
    - Anthropic Prompt Caching: 前缀Token成本降低90%
    - OpenAI Cached Tokens: 缓存命中时延迟降低50%+
```

即使逻辑上还在重复发送系统提示和工具schema，Provider侧的缓存机制可以让**账单直降30-50%**，且延迟显著下降。这是Phase 1就应该启用的优化——只需在Adapter层进行一行配置级的改动。


---


## 十二、失败模型与韧性设计

### 12.1 LLM特有的失败模式

这是Agent OS设计中最被低估的领域。传统OS的进程要么正常运行，要么崩溃——边界清晰。LLM Agent的失败模式远更复杂：

| 失败模式 | 表现 | 检测难度 | AgentOS的应对 |
|---------|------|---------|-------------|
| **幻觉（Hallucination）** | 输出自信但错误的结果 | 极高——需要外部验证 | 元认知模块的质量评估 + 关键步骤的外部验证钩子 |
| **拒绝（Refusal）** | 模型拒绝执行合法请求 | 低——易检测 | 控制模块的重试策略 + 备选模型切换 |
| **质量退化（Quality Decay）** | 长上下文或多轮交互后输出质量下降 | 中——需要趋势分析 | 元认知模块的异常检测 + 上下文压缩 |
| **无限循环（Infinite Loop）** | Agent反复执行相同行动 | 中——需要行为模式分析 | 控制模块的循环检测 + 步数限制 |
| **格式违规（Format Violation）** | 输出不符合预期格式 | 低——Schema校验 | 行动模块的post_hook校验 + 重试 |
| **上下文污染（Context Pollution）** | 错误信息进入上下文，影响后续决策 | 高——需要因果追踪 | 记忆模块的可回溯写入 + 上下文快照 |

### 12.2 系统级失败与传播

| 失败类型 | 检测方式 | 传播方式 | 恢复动作 |
|---------|---------|---------|---------|
| Agent Crash | 心跳超时（10秒） | 父Agent收到ERROR事件 | 根据escalation策略重试或降级 |
| Budget Exceeded | 内核Token计数 | 约束包中budget归零 | 中止执行，返回部分结果 |
| Action Violation | 内核在工具调用前拦截 | 阻断+向父Agent发送ERROR | 父Agent重新规划 |
| Timeout | Scheduler定时器 | 强制中断+返回TIMEOUT | 可重试或降级 |
| Partial Success | Agent自行判定 | RESULT消息中标记`partial: true` | 父Agent决定是否接受 |
| Cascade Failure | 多个子Agent连续失败 | ERROR事件累积触发阈值 | 父Agent启动降级策略或中止整个任务 |

### 12.3 补偿语义

对于已产生副作用但后续步骤失败的任务，AgentOS提供补偿记录机制：

```yaml
plan:
  steps:
    - id: "step_1"
      action: "tool:database.write"
      compensation: "tool:database.rollback"
      compensation_params: { transaction_id: "{step_1.result.txn_id}" }
    - id: "step_2"
      action: "tool:email.send"
      compensation: "tool:email.recall"
  on_failure: "execute_compensations_in_reverse_order"
```

**重要限定**：AgentOS内核只提供补偿信息的**记录和触发机制**。补偿动作的具体编排由父Agent的控制逻辑负责。内核不自动执行补偿——因为补偿本身可能失败，需要业务层面的判断。

### 12.4 韧性设计模式

AgentOS内建以下韧性模式：

**降级（Graceful Degradation）**：当最优路径失败时，自动尝试次优路径。例如：强模型不可用时切换到弱模型；精确搜索失败时退回到模糊搜索。

**隔离（Bulkhead）**：不同优先级的Agent运行在不同的资源池中。后台批处理Agent的崩溃不影响前台交互Agent。

**熔断（Circuit Breaker）**：见6.4.4节行动模块。连续失败自动熔断。

**超时（Timeout）**：每一级调用都有明确的超时限制。超时后不等待，立即返回部分结果或错误。


---


## 十三、安全与人机协作

### 13.1 安全边界模型

AgentOS的安全模型基于**最小权限原则**：Agent只能做它被明确授权做的事。

```
                    ┌─────────────────────────┐
                    │  系统级安全策略           │
                    │  （内核强制执行）         │
                    ├─────────────────────────┤
                    │  ● 全局工具黑名单        │
                    │  ● 全局资源配额          │
                    │  ● 沙箱隔离等级          │
                    │  ● 网络访问策略          │
                    └────────────┬────────────┘
                                 │ 继承 + 收窄
                    ┌────────────▼────────────┐
                    │  Agent级安全边界         │
                    │  （约束包定义）          │
                    ├─────────────────────────┤
                    │  ● allowed_tools        │
                    │  ● forbidden_patterns   │
                    │  ● resource_limits      │
                    │  ● escalation_rules     │
                    └────────────┬────────────┘
                                 │ 继承 + 收窄
                    ┌────────────▼────────────┐
                    │  子Agent安全边界         │
                    │  （父约束 ∩ 自身声明）   │
                    └─────────────────────────┘
```

**内核强制执行点**：

- **工具调用拦截**：每次`action_module.invoke()`调用前，内核检查工具是否在允许列表中
- **网络访问控制**：通过沙箱的网络策略限制Agent的网络访问范围
- **文件系统隔离**：每个Agent只能访问其被授权的文件系统路径
- **资源配额执行**：内核追踪每个Agent的Token/内存/CPU消耗，超限时强制中止

### 13.2 Agent特有威胁模型

传统OS的安全威胁（缓冲区溢出、权限提升、拒绝服务）在Agent系统中有对应物，但Agent系统还存在**传统OS安全模型中不存在的攻击向量**。必须诚实面对这些威胁：

| 威胁 | 攻击方式 | 内核级约束能否防御 | AgentOS的应对 |
|------|---------|-----------------|-------------|
| **提示注入（Prompt Injection）** | 攻击者通过用户输入或工具返回结果注入恶意指令，诱导Agent执行越权操作 | **部分**——内核拦截作用于工具调用层，但提示注入发生在LLM推理层，在Agent"决定"调用什么工具之前已影响决策 | 输入消毒（感知模块的pre_hook）+ 输出约束（行动空间仍由内核强制）+ 关键操作的Approval Gate |
| **记忆投毒（Memory Poisoning）** | 恶意Agent通过Event Fabric向Global Scope投影虚假记忆条目（可伪造高置信度） | **有限**——投影规则的filter可过滤一部分，但无法验证内容真实性 | 记忆条目签名验证（来源Agent的身份绑定）+ 跨源交叉校验 + 记忆条目的信任衰减机制 |
| **级联授权绕过** | 子Agent试图通过非工具路径（直接LLM推理）绕过`allowed_tools`约束 | **是**——`delegate_subtask`被建模为工具调用，受`allowed_tools`约束；LLM推理本身不产生外部副作用，约束作用于产生副作用的系统调用点 | 所有对外部世界的影响必须通过action_module，内核在此层拦截 |
| **信息泄露（通过记忆投影）** | Agent将不应共享的内部信息投影到Global Scope | **是**——投影规则由内核审核，可设置敏感字段过滤 | 投影内容的sensitivity标注 + 自动脱敏 |

**诚实的防御边界**：提示注入是LLM层的攻击，AgentOS的内核级防御只能**限制其影响半径**（Agent可能被误导做出错误决策，但不能执行越权操作），不能完全阻止。彻底防御需要LLM本身的鲁棒性提升——这是模型层的问题，不是OS层的问题。记忆投毒的防御依赖来源签名验证和跨源交叉校验，但这增加了记忆检索的复杂度，需要在Phase 3+的生产化阶段权衡。

**与MCP安全能力的边界**：MCP已提供工具调用层的安全校验。AgentOS的增量价值限定于三个维度：(1) 跨Agent委托链的权限单调递减；(2) Token预算作为安全约束（MCP不管Token）；(3) 跨步骤的行为模式异常检测。两者互补而非替代。

### 13.3 人在回路（Human-in-the-Loop）

生产级Agent系统必须支持人类介入。AgentOS定义了三种人机协作模式：

**审批门（Approval Gate）**：在执行高风险操作前暂停，等待人类审批。

```yaml
approval_gates:
  - trigger: "tool_category == 'destructive'"    # 写数据库、发邮件等
    action: pause_and_request_human_approval
    timeout: 300s
    on_timeout: abort

  - trigger: "cost_estimate > 1.0 USD"
    action: pause_and_request_human_approval
```

**监督窗口（Supervision Window）**：人类可以实时观察Agent的执行过程，随时介入。

```yaml
supervision:
  trace_streaming: true          # 实时推送TRACE到监督界面
  intervention_api:
    pause()                      # 暂停Agent执行
    modify_plan(new_plan)        # 修改Agent的计划
    inject_context(context)      # 注入额外上下文
    abort()                      # 中止任务
```

**反馈回路（Feedback Loop）**：Agent完成任务后收集人类反馈，用于改进。

```yaml
feedback:
  collect_after: task_completion
  feedback_types: ["accept", "reject", "modify"]
  feed_to: semantic_memory       # 将反馈写入语义记忆，影响未来决策
```

### 13.4 审计与合规

所有Agent的行动都自动产生不可篡改的审计日志：

- **决策审计**：每个状态转换的原因和上下文
- **工具调用审计**：每次工具调用的参数、结果和耗时
- **数据访问审计**：每次记忆读写的内容摘要
- **约束违规审计**：每次被内核拦截的违规行为详情

审计日志独立于Agent的TRACE（Agent可以控制TRACE的粒度和采样率，但审计日志由内核强制产生，Agent无法关闭）。


---


## 十四、可观测性：理解系统在做什么

### 14.1 三层可观测性

```
┌──────────────────────────────────────────────────────────┐
│  层级3: 业务可观测性（用户定义）                              │
│  ● 任务成功率 · 结果质量 · 端到端延迟 · 用户满意度            │
├──────────────────────────────────────────────────────────┤
│  层级2: Agent可观测性（Kernel自动产生）                       │
│  ● 状态转换时间线 · 工具调用图 · 记忆读写轨迹 · 决策路径     │
├──────────────────────────────────────────────────────────┤
│  层级1: 系统可观测性（内核自动产生）                          │
│  ● 调度延迟 · Token消耗 · Bus吞吐量 · 资源使用率              │
└──────────────────────────────────────────────────────────┘
```

层级1和层级2由AgentOS自动产生（公理5）。层级3由用户在Agent逻辑中定义。

### 14.2 分布式追踪（与OpenTelemetry对齐）

每个Message Envelope内建追踪字段（`trace_id`, `span_id`, `parent_span_id`），与OpenTelemetry的Trace模型原生兼容。这意味着：

- 一个多Agent任务的完整调用链可以在标准追踪工具（Jaeger、Zipkin）中可视化
- 每一级Agent的内部执行步骤构成子Span
- 跨Agent的TASK/RESULT消息自动传播追踪上下文

### 14.3 调试体验

AgentOS的设计目标之一是让多Agent系统的调试体验接近单进程应用的调试体验：

- **时间线视图**：可视化所有Agent的状态转换时间线，包括并行和等待关系
- **因果链追踪**：通过`causation_id`链，从任何一个事件回溯到触发它的完整因果链
- **状态快照**：在任意时间点查看任何Agent的完整状态（五模块的快照）
- **重放能力**：基于TRACE日志，可以重放Agent的执行过程（确定性的部分）

### 14.4 Token效率度量框架

公理6要求Token效率成为可量化、可观测的系统属性。以下指标构成Token效率的核心度量体系，由AgentOS内核自动采集（公理5的延伸）：

| 指标 | 定义 | 作用 | Phase 1 |
|------|------|------|---------|
| `effective_token_ratio` | 用于实际问题求解的Token / 总Token | 衡量系统整体效率 | **采集基线** |
| `coordination_tax_ratio` | (委托+摘要+回传+反思) Token / 任务总Token | 衡量多Agent协调开销 | Phase 2+ |
| `fixed_prefix_ratio` | 重复系统提示+工具描述 / 总输入Token | 衡量固定提示税 | **采集基线** |
| `reflection_roi` | 反思触发的纠错收益 / 反思消耗的Token | 判断反思是否值得 | Phase 2+ |
| `memory_hit_rate` | 检索后被实际引用的记忆条目 / 检索总数 | 判断检索是否浪费 | Phase 2+ |
| `summary_compression_ratio` | 原始过程Token / 摘要Token | 判断摘要质量 | Phase 2+ |
| `delegation_payback` | (单Agent成本 - 多Agent总成本) / 多Agent总成本 | 判断委托是否划算 | Phase 2+ |
| `context_utilization` | 上下文中影响输出的Token / 注入总Token | 判断上下文质量 | Phase 2+ |
| `duplicate_content_ratio` | 上下文中语义重复的Token / 总Token | 检测重复注入 | Phase 2+ |

**关于目标值的诚实声明**：上表有意不设定具体的目标值（如`> 0.6`或`< 0.3`）。在没有任何生产数据支撑的情况下设置量化目标，存在将"期望"伪装为"标准"的风险。正确的做法是：**Phase 1只采集数据、建立基线**——用A/B对比中baseline和treatment两组的实际数据，确定各指标的现实分布范围；**Phase 2基于Phase 1的数据设定目标值**。Phase 1只需实现`effective_token_ratio`和`fixed_prefix_ratio`两个基础指标（几个计数器的事），后续Phase逐步补全。


---


## 十五、与现有工作的系统性定位

### 15.1 学术定位

| 研究 | 核心洞见 | AgentOS的继承 | AgentOS的超越 |
|------|---------|-------------|-------------|
| **AIOS** (Rutgers, 2024) | Agent应被OS内核管理，享有调度、记忆、安全等系统服务 | 内核/用户空间分离；调度原语设计 | 将Agent定义为结构完整的认知单元（五模块），而非仅是可调度任务 |
| **分层控制理论 + 约束传播** | 约束传递可显著降低分层系统的失败级联 | 约束包逐层传递且只收不放 | 将约束嵌入消息协议并由内核强制执行，而非仅作为架构建议 |
| **ROMA** | 递归任务分解+角色标准化可干净地分离编排与执行 | 自相似组合能力 | "单Agent优先"原则——递归是能力不是义务 |
| **MemGPT + 记忆增强研究** | 分层记忆管理和精心设计的上下文调度可大幅降低Token消耗 | 三层记忆模型 + 上下文调度 | 将记忆所有权下沉到Agent，添加投影机制实现可控共享 |
| **Reflexion** (Shinn et al.) | Agent自我反思可提升20%+任务成功率 | 元认知模块作为一等模块 | 强制反思检查点——不是可选功能，而是FSM的必经状态 |

### 15.2 工程定位

| 项目/协议 | AgentOS的关系 | 边界 |
|----------|-------------|------|
| **MCP** (Anthropic/Linux Foundation) | 作为工具调用的标准协议集成 | AgentOS不发明新的工具协议，MCP是行动模块的一等接口 |
| **A2A** (Google) | 作为跨系统Agent互操作的协议接口 | AgentOS在组织内部提供运行时；跨组织通信委托A2A |
| **OpenAI Agents SDK** | 互补关系——SDK提供极简编排原语，AgentOS提供运行时服务 | SDK中的Agent可以作为AgentOS的Kernel实例运行 |
| **LangGraph / AutoGen** | 互补关系——它们提供编排模式，AgentOS提供底层运行时 | 编排逻辑在用户空间，AgentOS在内核空间 |
| **AIOS** (agiresearch) | 理念一致，实现路径不同 | AIOS按资源类型分层（LLM/Memory/Storage/Tool），AgentOS按认知结构分模块 |

### 15.3 AgentOS的差异化贡献

1. **Agent Kernel的五模块认知结构与运行模式分级**：首次将"元认知"提升为Agent的一等模块，并将环境管理下沉为内核服务。Kernel支持full/standard/lightweight三级运行模式，在保持结构完整性的前提下实现Token效率的阶梯优化。
2. **Event Fabric的三作用域模型与payload设计范式**：在Global和Agent两层作用域之外引入Task Scope，解决协作任务的信息共享问题。通过结构化短格式和上下文句柄机制，从协议层面解决多层级联的"上下文复印机"问题。
3. **约束传播的内核级强制执行**：约束不依赖Agent的"自觉遵守"，而是由内核在系统调用点强制拦截。约束包新增上下文传递模式，使最小Token传递成为协议级的默认行为。
4. **"单Agent优先"原则的架构化与量化**：首次将"何时用多Agent"的决策从经验判断提升为带有量化框架的架构指导原则。
5. **LLM特有失败模式的系统化处理**：将幻觉、质量退化、上下文污染等LLM特有问题纳入正式的失败模型。
6. **"Token即资源"公理与协议级Token效率保障**：首次在Agent OS的公理体系中将Token效率提升为一级约束，并通过反思决策树、工具描述按需切片、增量摘要、信息增益评估等机制，确保Token高效行为是系统的默认路径而非可选优化。

### 15.4 模型能力增长与AgentOS的演进

必须诚实面对一个根本性趋势：**LLM自身的能力正在吞噬传统意义上的"系统层"功能。**

GPT-5级别的模型很可能直接内建规划（planning）、记忆管理（memory）和工具路由（tool routing）。当模型本身就能完成这些认知活动时，AgentOS的五模块Kernel和三层控制架构是否还有价值？

**回答是：AgentOS的价值会发生重心转移，而非消失。** 类比：现代CPU已内建分支预测、乱序执行、内存预取——这些在早期是OS或编译器的工作。但操作系统的调度、安全、隔离和可观测职责不但没有减少，反而因为CPU复杂性的增加而变得更重要。

模型越强，以下系统层职责越关键而非越冗余：

| 模型能力增强 | 对系统层的影响 | AgentOS的价值转移方向 |
|------------|-------------|-------------------|
| 模型自带规划 | 控制模块的LLM决策部分被模型吸收 | 控制模块聚焦于**约束执行和升级策略**——确保模型的规划不越界 |
| 模型自带记忆 | 内建长上下文减少外部记忆需求 | 记忆模块聚焦于**跨Agent隔离和投影**——模型自带记忆但不自带跨Agent的记忆边界 |
| 模型自带工具路由 | 行动模块的路由逻辑被模型吸收 | 行动模块聚焦于**熔断、限流和内核拦截**——模型能选工具但不能强制执行权限约束 |
| 单次调用成本升高 | Token预算管理更关键 | 调度和预算管理从"优化项"变为"生死线" |

简言之，AgentOS在强模型时代的核心定位收缩为三件事：**预算管理（谁能花多少）、安全边界（谁能做什么）、可观测性（谁做了什么）**。这恰好是传统OS的核心——调度器+安全模块+审计日志。如果AgentOS最终被"压缩"成这三样东西，那正好就是一个成功的Runtime该有的样子。

### 15.5 竞争路径分析：AgentOS为什么可能不是你来做

不可替代性不来自架构设计，而来自场景锁定和生态网络效应。必须诚实审视三条可能让AgentOS变得多余的竞争路径：

**路径A：框架进化**。LangGraph/AutoGen完全可以演进为内置memory管理+budget控制+multi-agent调度的系统——变成"事实上的AgentOS但不叫OS"。这条路径的威胁在于：框架已有用户基础和生态，演进比从零构建的新Runtime更容易获得采用。

**路径B：模型内收**。未来模型可能直接支持structured planning、tool budget control、recursion limits。这条路径下，Agent Runtime被压缩为极薄的管理层。但如2.4节分析，即使模型内建了这些能力，**模型外部的预算分配、安全隔离和失败传播**仍然是系统层问题。

**路径C：平台托管**。OpenAI/Anthropic/Google提供Agent Runtime作为云服务。这条路径下，Runtime层变成云基础设施而非独立项目。

三条路径的共同启示不是"AgentOS的概念不成立"，而是"**AgentOS描述的功能一定会被实现，问题是被谁实现、以什么形式实现、在什么时间窗口**"。

AgentOS在这三条路径面前的真正护城河不可能来自技术设计本身（设计可以被复制），只能来自：(1) 场景锁定——在某个垂直场景中用AgentOS的约束传播和Token治理打出可量化的成本/可靠性优势；(2) 生态网络效应——如果Event Protocol成为多个Agent框架的事实互操作标准；(3) 数据飞轮——历史Token消耗数据和约束学习的积累。这三个护城河无一可以从架构文档中获得，都需要产品化运营验证。


---


## 十六、演进路径

### 16.1 双轨验证策略与分阶段实施

AgentOS的核心假设需要在两个层面验证：(1) 约束执行、Token治理、循环检测等**独立能力**是否有实用价值；(2) 五模块Kernel、Event Fabric、约束传播等**系统级架构**是否值得其复杂度。这两个层面的验证可以并行推进，但不应互相阻塞。

**快轨（插件验证，1-2周）**：将AgentOS中最实用的子集——约束检查、Token预算管理、循环检测、上下文优化——以现有Agent平台（如LangGraph middleware或OpenClaw plugin）的插件形式实现。这条轨道不验证Kernel结构或Event Fabric，只验证"这些独立能力在真实Agent场景中是否产生可量化的价值"。如果连这些最实用的子集都无法证明价值，完整Runtime更没有必要。

**主轨（Runtime验证，2-3周）**：按下表分阶段实施完整的AgentOS Runtime，执行A/B对比验证。

| 阶段 | 目标 | 关键产出 | 核心验证 | Token效率机制 |
|------|------|---------|---------|-------------|
| **Phase 1: 单Kernel闭环** | 实现一个完整的Agent Kernel + Agent Scope事件通道 | 一个能独立完成多步任务的Agent，具备完整的五模块和FSM | "感知→规划→执行→反思→记忆"闭环运行；A/B对比验证系统开销是否值得 | Kernel运行模式分级、反思决策树、工具描述按需切片、基础Token分布统计、Provider侧缓存、重复前缀成本追踪 |
| **Phase 2: 多Agent协作** | 实现Global/Task Scope + 投影规则 + TASK/RESULT通信 | 两个Kernel可通过Event Fabric协作 | 并行子任务的协作完成 | 消息payload结构化短格式、上下文句柄机制、增量摘要、投影去重 |
| **Phase 3: 约束传播** | 实现约束包传递 + 内核强制执行 | 三级级联Agent稳定运行 | 约束违规被正确拦截 | 角色级Token治理、上下文效用评估、委托决策量化 |
| **Phase 4: 生产化** | 持久化记忆 + 人在回路 + 审计日志 | 可部署的运行时 | 端到端可观测和可审计 | 完整Token效率度量框架、历史分布学习 |
| **Phase 5: 生态** | MCP集成 + A2A网关 + SDK + 开发工具 | 开发者可用的平台 | 外部开发者基于AgentOS构建Agent | Token效率基准测试套件 |

**快轨与主轨的关系**：快轨的数据直接输入主轨——如果插件验证表明约束检查和Token管理在真实场景中价值显著，就为Phase 1提供了信心和数据基线；如果插件验证表明这些能力的独立价值有限，Phase 1应重新审视设计假设，而非盲目推进。

**另一条值得独立推进的路径**：将"上下文句柄+结构化短格式"作为独立协议提案（RFC/SEP）推动其成为A2A或MCP的扩展标准。这两个设计不依赖AgentOS的存在即可产生价值，通用性最高、实现门槛最低，且影响力可能远超一个独立Runtime。

### 16.2 Phase 1 最小实现规格：什么必须做，什么先不做

> **历史快照（Phase 1 验证起点，非当前 as-built）**：下表描述架构验证初期的最小规格与代码量估算；当前实现规模与能力以 [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md)（v12.0.1 · golden **4861**）为准。

架构文档描述的是完整的目标状态。但Phase 1的目标是**用最小代码验证核心假设**。以下明确区分"必须实现"和"架构预留但Phase 1不实现"：

| 模块/组件 | Phase 1 实现 | Phase 1 **不**实现 | 预估代码量 |
|----------|-------------|-------------------|-----------|
| **Perception** | `parse(input) → TaskDefinition`——一个将字符串输入解析为结构化任务的函数 | 能力声明（AgentCard）、多模态输入 | ~30行 |
| **Memory** | 工作记忆 = Python `dict`；情景记忆 = Python `list`（append-only）；无语义记忆 | 向量索引、投影规则、跨Agent共享、持久化 | ~60行 |
| **Control** | `match/case` 实现的认知FSM（6个核心状态）；硬编码的local_policy（重试3次）；无escalation | 动态策略、约束协商、升级协议 | ~100行 |
| **Action** | 通过LiteLLM调用LLM + 1-2个MCP工具；无路由、无熔断 | 工具注册表、能力匹配路由、Circuit Breaker | ~80行 |
| **Meta** | Tier 1确定性检测：循环检测 + 步数限制 + 格式校验 | Tier 2统计检测、Tier 3 LLM辅助评估 | ~40行 |
| **Event通道** | `asyncio.Queue`——单Agent内部的Local Event Channel | Global/Task Scope、投影规则、流控 | ~50行 |
| **生命周期** | 简化FSM：READY → RUNNING → COMPLETED/FAILED | SUSPENDED、DRAINING、REJECTED、checkpoint | ~40行 |
| **调度** | 无——Phase 1只有单Kernel，不需要调度 | 优先级队列、并发管理、starvation prevention | 0行 |

**Phase 1 Token效率基线**（公理6要求从第一天就建立量化基线）：

```yaml
phase1_token_baseline:
  must_track:
    - tokens_per_task: { input: int, output: int }
    - fixed_prefix_tokens: int    # 系统提示+工具描述
    - effective_reasoning_tokens: int  # 除去固定前缀的有效Token
    - reflection_tokens: int      # 反思阶段消耗（Phase 1应≈0）
  must_report:
    - effective_token_ratio: "effective / total"
    - overhead_per_step: "fixed_prefix / steps_count"
  implementation_cost: "几个计数器，~20行代码"
```

如果Phase 1不建立Token效率基线，后续优化就没有参照点。

**Phase 1总估算**：~450行Python核心代码（含Token计数器） + ~200行测试 + ~100行胶水代码。一个有经验的开发者可以在**2-3周**内完成闭环验证。

**Phase 1的验证标准**（极简版）：给Agent一个多步任务（如"搜索三个关键词并生成摘要"），Agent能自主完成"感知输入 → 生成计划 → 调用工具 → 检测循环/超限 → 记录结果"的完整闭环，且在中间步骤出错时能自动重试或返回部分结果。Token效率方面，`effective_token_ratio > 0.5`且`reflection_tokens ≈ 0`（反思不应消耗LLM Token）。

**Phase 1的核心目标不是"五模块闭环跑通"，而是"证明有用"。** 架构的正确性可以从设计文档中论证，但有用性只能从场景对比中验证。因此Phase 1必须包含一个**A/B对比验证**：

```yaml
phase1_ab_test:
  scenario: "多步调研任务——搜索3个关键词，交叉验证信息，生成结构化摘要"
  
  baseline_A: "LangGraph + 直接LLM调用（无AgentOS）"
    # 开发者手动管理：重试逻辑、Token计数、循环检测、步数限制
    # 追踪方式：自行埋点

  treatment_B: "AgentOS单Kernel闭环"
    # Kernel自动管理：FSM状态转换、Meta模块检测循环/超限、Token预算追踪
    # 追踪方式：Event通道自动产生

  comparison_metrics:
    - token_efficiency: "完成相同任务的总Token消耗"
    - failure_recovery: "中间步骤出错时的恢复成功率"
    - observability: "事后调试一个失败任务需要的人工时间"
    - development_effort: "实现相同功能的代码行数和开发时间"

  success_criteria:
    # 至少在一个维度上展示出可量化的优势
    - "Token消耗不高于baseline的1.15x（AgentOS运行时开销可控）"
    - "失败恢复率高于baseline至少20%（Meta模块的循环/超限检测有效）"
    - "调试时间低于baseline至少50%（统一Event通道的可观测性价值）"

  failure_criteria:
    # 如果以下任一条成立，说明设计假设需要修正
    - "Token消耗超过baseline的1.3x → 五模块开销过重，需简化"
    - "开发时间超过baseline的2x → 抽象层过厚，需降级为更轻的框架"
```

这个A/B对比的意义不在于"证明AgentOS更好"，而在于**用数据回答"AgentOS的系统开销是否值得"这个根本问题**。如果Phase 1的数据表明AgentOS的运行时开销超过了它带来的可靠性和可观测性收益，那么设计假设需要修正——可能五模块太重、可能Event Fabric太厚、可能约束传播太贵。数据比论证更重要。

### 16.3 Phase 1的技术选型建议

```
语言: Python 3.11+（asyncio原生支持，LLM生态最丰富）
事件通道: asyncio.Queue（单进程MVP）→ Redis Streams（多进程扩展）
状态管理: Pydantic模型（类型安全 + 序列化）
记忆存储: 内存字典（MVP）→ SQLite（Phase 2）→ PostgreSQL + pgvector（Phase 4）
LLM调用: LiteLLM（统一多Provider接口）
追踪: OpenTelemetry SDK
测试: pytest-asyncio
```

### 16.4 评估指标

| 指标类别 | 具体指标 | 目标值 |
|---------|---------|-------|
| **功能** | 认知闭环完整性（五模块全部参与任务执行） | 100% |
| **效率** | 相比基线（无AgentOS的直接调用）的Token开销比 | < 1.15x |
| **稳定性** | 三级级联任务的成功率 | > 85% |
| **可观测性** | 关键决策点的TRACE覆盖率 | 100% |
| **安全性** | 行动空间违规的拦截率 | 100% |
| **延迟** | AgentOS运行时引入的额外延迟 | < 50ms/步 |


---


## 十七、结语：证明有用比证明正确更重要

AgentOS v5.0架构代表了对AI Agent系统的一次系统性重新思考。它的核心贡献不在于任何单一的技术创新，而在于**将分散在不同研究和工程实践中的洞见，统一在一个最小化、可实施、诚实面对技术限制的框架中**。

回顾本文的关键设计决策：

**Agent Kernel的五模块结构**，基于关注点分离的工程原则，为Agent提供可测试、可替换、可观测的内部结构。五模块划分不是认知科学的理论必然，而是"过粗导致职责混杂"和"过细导致概念负担"之间的工程平衡点——其合理性将在Phase 1中通过"每个模块是否可独立实现、独立测试、独立替换"来验证。

**Event Fabric的统一消息信封协议**，其核心价值不在于物理通道的统一，而在于协议格式的统一——确保任何两个Agent之间的交互都可以通过同一套trace_id/causation_id体系进行因果链追踪。

**约束传播的内核级强制执行**，确保了多Agent系统的安全性和可控性不依赖任何Agent的"善意"。这是AgentOS中与传统OS最为同构的部分。

**"单Agent优先"原则**，对多Agent系统的协调税给出了诚实的评估，避免了为复杂而复杂的过度工程。

**"Token即资源"公理**，将Token效率从"性能优化项"提升为与安全性、可观测性同等级别的架构约束。在模型能力持续增强、单次调用成本持续攀升的趋势下，Token管理的系统级保障只会变得更关键。

### 诚实的自我评估

本文已经充分论证了AgentOS在架构层面的**正确性**——设计公理自洽、模块划分有工程依据、OS隐喻在有效边界内成立。但正确性和有用性之间存在巨大鸿沟：

**本文能论证的**：
- Agent系统的复杂度正在超过拼装可控范围，需要系统层抽象
- 五模块结构在关注点分离上优于单体循环
- 约束传播和内核拦截在安全性上优于prompt层约束
- Token效率需要协议级的默认保障而非可选优化

**本文不能论证的**：
- AgentOS的运行时开销是否值得它带来的可靠性收益——只有Phase 1的A/B对比数据能回答
- AgentOS的形态是否是这些功能的最优载体——框架进化（路径A）、模型内收（路径B）、平台托管（路径C）都可能提供等价的解决方案（见15.5节）
- AgentOS在竞争环境中是否有护城河——技术设计可以被复制，护城河只能来自场景锁定和生态效应

正如Linus Torvalds所说——"Talk is cheap. Show me the code." 但比code更重要的是**data**：Phase 1的A/B对比数据将决定AgentOS是"一个有用的Runtime"还是"一套正确但过度设计的抽象"。

AgentOS v5.0的Agent Kernel——五个模块、一个状态机、一个事件协议——就是那个"足够小、足够对的起点"。让它先跑起来，然后用数据说话。


---


## 附录A：术语表

| 术语 | 定义 |
|-----|------|
| Agent Kernel | AgentOS中的最小可调度认知单元，由五个模块（感知、记忆、控制、行动、元认知）组成 |
| Event Fabric | 统一事件织网，系统中所有信息流通的协议层（核心价值在统一消息信封协议，而非统一物理通道） |
| Constraint Bundle | 约束包，沿任务委托链传递的预算和边界定义，只能逐层收窄 |
| Execution Handle | 执行句柄，内核对Agent实例的管理投影，类似传统OS的PCB |
| Message Envelope | 消息信封，Event Fabric中所有事件遵循的统一协议格式 |
| Scope | 作用域，Event Fabric中事件的可见性边界，分为Global、Agent、Task三级 |
| Projection | 投影，将Agent内部事件发布到更宽作用域的规则 |
| Escalation | 升级，Agent将超出其处理能力的情况上报父Agent的行为 |
| Meta Module | 元认知模块，Agent对自身认知过程的监控和评估能力 |
| Lifecycle FSM | 生命周期状态机，由AgentOS内核管理的Agent外部状态（PENDING→READY→RUNNING→...） |
| Cognitive FSM | 认知状态机，由Agent Kernel控制模块管理的内部认知状态（IDLE→PERCEIVING→PLANNING→...） |
| Coordination Tax | 协调税，多Agent系统中用于协调而非实际工作的开销 |
| Context Handle | 上下文句柄，轻量引用指向存储在Memory中的完整内容，避免消息传递时内联全文 |
| Kernel Execution Mode | Kernel运行模式，按任务复杂度分为full/standard/lightweight三级，控制模块实例化深度 |
| Payload Design Paradigm | Payload设计范式，定义消息体的默认表示形式（结构化短格式优先、句柄优先） |
| Token Efficiency Metrics | Token效率度量体系，包括effective_token_ratio、coordination_tax_ratio等核心指标 |
| Reflection Decision Tree | 反思决策树，定义进入REFLECTING状态后的分级执行逻辑，消除"反思=LLM调用"的实现歧义 |

## 附录B：参考文献与工程参考

### 学术文献

1. **AIOS: LLM Agent Operating System** — Mei et al., Rutgers University, 2024. COLM 2025 accepted. Agent操作系统的开创性工作。
2. **Constrained Hierarchical Agent Architectures** — 本文对约束传播降低级联失败的分析，综合了分层控制理论（Hierarchical Task Networks）和微服务架构中的约束传递模式（Bulkhead/Circuit Breaker）。"47%降低"为本设计中基于模拟估算的目标值，非单一论文的实验结论。
3. **ROMA: A Role-based Multi-Agent Framework** — 角色化多Agent任务分解框架。本文借鉴其角色分离与递归分解思想，但具体的"四角色"划分为AgentOS根据时间尺度分离原则独立推导。
4. **MemGPT / Cognitive Architectures for LLM Agents** — Packer et al., 2023; 以及相关记忆增强Agent研究。本文的三层记忆模型借鉴了认知科学的情景记忆/语义记忆划分以及MemGPT的分层记忆管理思路。"1% Token"的效率数据来源于上下文压缩相关研究的数量级估算。
5. **Reflexion: Language Agents with Verbal Reinforcement Learning** — Shinn et al., NeurIPS 2023. 自我反思提升20%+任务成功率。
6. **Lost in the Middle: How Language Models Use Long Contexts** — Liu et al., TACL 2024. LLM在长上下文中间位置信息检索能力下降的实证研究。
7. **Are More LLM Calls All You Need? Towards Scaling Laws for Compound AI Systems** — Chen et al., Google/Stanford, 2025. 多Agent系统协调开销的定量分析，表明复合AI系统存在显著的协调税。

### 工程参考

8. **Model Context Protocol (MCP)** — Anthropic, 2024. Linux Foundation Agentic AI Foundation, 2025. 工具调用标准协议。
9. **Agent2Agent Protocol (A2A)** — Google, 2025. 跨Agent通信开放协议。
10. **OpenAI Agents SDK** — OpenAI, 2025. 极简Agent编排原语。
11. **AIOS Server & Cerebrum SDK** — agiresearch, 2025. Agent运行时框架与开发工具包。
12. **LangGraph** — LangChain, 2024. 基于图的Agent编排框架。
13. **AutoGen** — Microsoft Research, 2024. 对话式多Agent框架。
