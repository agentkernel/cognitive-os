# Agent 集成与会话

- 状态：已采纳的 Personal 2.0 产品目标
- 规范语言：英文
- 英文规范源：
  [agent-integration-and-conversations.md](agent-integration-and-conversations.md)
- 架构：
  [Universal Agent Adapter Contract](../architecture/agent-adapter-contract.md) 与
  [Multi-Agent orchestration](../architecture/multi-agent-orchestration.md)
- 相关文档：[Web UI 产品设计](web-ui-design.md)、
  [用户旅程](user-journeys.md) 与 [账户中心](account-hub.zh-CN.md)

本文是英文规范源的忠实中文翻译；若表述冲突，以英文文档为准。

Personal 2.0 让 Control Plane 成为所有者各类 Agent 的桌面主要入口与监督界面，
同时保留每个 Agent 的原生会话与 harness 行为。集成绝不把 Agent、adapter 或会话
变成权威写入者。

## 1. 现实台账

| 边界 | Agent 事实 |
|---|---|
| **当前实现（Now）** | Pi 是 Linux 1.0 已资格验证的 Agent/sidecar 路径。`/ui/` 有 Agent inventory 和 dossier，展示有界 Runtime/dsh 事实，但没有 lifecycle control 或内嵌会话。原生 `cognitive dsh web` 面板是独立界面。 |
| **已采纳的 Personal 2.0 目标** | Agents 包含签名 onboarding、连接现有 Agent、通用 capability view、adapter-backed 原生会话/历史、Runtime 监督、所有者请求 Goal 后由 daemon 接纳、handoff 与删除选择。 |
| **需要后端（Requires-backend）** | Catalog onboarding、通用会话/历史投影、限定范围的原生 session 观察与 daemon 接纳、完整 Agent lifecycle HTTP、Goal -> Plan revision -> Task -> Attempt 编排、Multi-Agent graph/handoff 和目标控制。 |
| **有条件需要 core** | 复用现有 Core Conversation/ConversationBinding。只有新增公开 Agent capability、会话扩展、Goal、Plan、Run、Harness、attempt 或 handoff 机器合同才需要 P10-T02/Lane-CTR；Personal-private 投影未必需要 core 变更。 |

## 2. 产品模型

**Agents** 空间围绕 Agent dossier 组织：

- 签名来源与安装/连接身份；
- adapter 兼容性和 capability matrix；
- 当前 Provider/代理配置与 workspace scope；
- 原生会话与历史；
- Agent runtime engine、进程观察与健康；
- 当前 Goal、Plan revision、Task、每个 Task 的 attempt 与 handoff；
- permission、federated resource、Activity 与 evidence；
- 已支持的 lifecycle 与恢复动作。

默认使用易懂标签：**会话**、**执行流**、**Agent runtime engine**。Package、
installation、registration、instance、sidecar、execution、process、session、
epoch、digest 和原始脱敏投影保留在 inspector 中。不存在 Basic/Expert 模式。

## 3. Adapter 投影

每个 vendor adapter 保留原生 harness，并投影以下内容。

### 通用核心

- Agent 展示身份以及精确原生/受管身份事实；
- 来源、版本、adapter compatibility；
- 可以观察时的原生会话列表与选中会话；
- 可以观察时的当前响应/活动与健康；
- adapter 能如实暴露的 Provider/profile、workspace、permission 与资源绑定事实；
- 已支持的 lifecycle、conversation、Context、Tool 与 synchronization capability；
- 明确的 unsupported、unknown、stale 与 native-only facet。

适用时，通用投影复用或引用现有 Core `Conversation` 与
`ConversationBinding` 身份。Vendor 原生 conversation/thread ID 保持为不透明的
origin binding，不创建第二套公开 Conversation 模型。其他原生/通用投影状态在
P10-T02 另行决定前保持 Personal-private。

### Capability matrix

UI 展示能力属于哪一种：

- vendor session API 直接支持的原生能力；
- 受管 adapter 路径支持；
- 通过 MCP 加 vendor rules 协作支持；
- 只能观察；
- 不可用。

矩阵只是描述，不授予 capability。它绝不能从进程存活或 MCP 连接推导主机 session 控制。

### Vendor 扩展槽

无法忠实映射到通用核心的 vendor 特有概念可以出现在 extension inspector 中。
扩展保留原生语义和来源标签，不能覆盖 daemon 权威，也不能掩盖缺失的通用行为。

## 4. 原生会话是交互来源

会话从 **Native** 开始并保持 Native，除非用户请求 **Manage with Personal**、
确认 daemon 预览，并由 daemon 接纳受治理结果。

- 有 vendor-native session API 时，内嵌视图优先使用它。
- 原生 Agent 应用始终可继续使用。
- 当前原生 session 与不透明 vendor ID，和基于 Core
  Conversation/ConversationBinding 的 Personal 投影、Goal、Task、Agent runtime
  engine、process 保持不同身份。
- 原生 Agent plan 即使显示在 Work 中仍然是 Native。
- Adapter 观察只产生 **Observed** 事实，不产生 governed Task、permission、
  Memory 或 completion。

### 使用 Personal 管理

**Manage with Personal** 动作：

1. 标识所选原生会话与期望结果；
2. 请求 daemon 预览持久 Goal；
3. 让所有者确认该精确重要预览；
4. 由 daemon 接纳 Goal 并建立 daemon-owned Plan revision；
5. 由 daemon 创建一个或多个受治理 Task；每个 Task 拥有自己的保留 attempt；
6. 只绑定已接纳的 Context、Agent、workspace、Provider/profile、permission、
   budget 与 acceptance criteria；
7. 保留回到原生会话的来源链接，但不复制 secret，也不虚构原生权威。

一个 Goal 可以跨 conversation、session 与 Agent。daemon 拥有 Multi-Agent graph
和 handoff。Agent 之间不能转移 lease、permission 或 completion authority。

该目标**需要后端**。新增公开机器语义时才有条件需要
P10-T02/Lane-CTR；Personal-private 投影未必需要 core 变更。

## 5. 被观察的原生 session

原生 Agent 可以在 Personal-managed execution 之外继续使用。连接 Agent 时建立一个
明确的 observation scope；adapter 只能在该 scope 内自动观察受支持的原生 session。

观察规则：

- 连接 Agent 时显示并授权精确 source 与 observation scope；
- 禁止推测性/全局 session 扫描，也禁止每个 session 意外自动加入；
- 只显示 adapter 能如实读取的能力；
- observed session 绝不自动变成 governed；
- 原生 plan、Tool result、process exit 或 final text 绝不提升为 Task completion；
- **Manage with Personal** 是从 observation 请求新 governed Goal 的唯一产品入口；
  所有者确认预览，且只有 daemon 能接纳；
- 不支持的原生 session 保持 native-only，不能通过猜测或 process signal 控制。

## 6. Agent onboarding 不超过三步

### 第 1 步——选择来源

选择：

- signed upstream catalog record；或
- **Connect existing**。

每个 catalog record 展示 source、version、digest、signature、license 与 adapter
compatibility。Catalog listing 不授予 permission，也不转移 qualification evidence。

### 第 2 步——一次审查

在同一处审查 Provider/代理配置、Standard Workspace 与请求的 permission。
可选细节留在 inspector。用户可以拒绝或缩小 permission；安全时保留 native-only 路径。

### 第 3 步——首次会话

打开内嵌原生会话。**Ready** 表示收到第一次真实响应。仅有安装字节、process health、
adapter handshake、model discovery 或 synthetic probe 都不算 ready。

### 两个激活里程碑

1. **First chat**——真实原生会话响应。
2. **First governed and verified Task**——daemon 已接纳，并具有当前独立验证和已
   reconcile Effect 的工作。

产品不得把两个里程碑压成一个 readiness badge。

## 7. Runtime 与 lifecycle

Package、installation、registration、instance、sidecar、execution、process、
native session、基于 Core Conversation/ConversationBinding 的 Personal 投影、
Goal、Plan revision、Task 与 Task-owned attempt 保持不同。
共址或共享字节不会合并 identity、permission、epoch 或 completion。

已采纳的目标控制包括：

- interrupt 当前会话交互；
- 请求 Task pause/resume；
- cancel Task；
- detach observation，但不改变工作；
- 从 checkpoint retry/fork，形成保留历史的新 attempt；
- restart/recover Agent runtime engine；
- disconnect 或 uninstall Agent。

这些控制目前都**需要后端**。当前 `/ui/` 必须继续解释它们不可用，而不是绘制虚假控件。

## 8. Disconnect 与 uninstall

每个移除流程都先询问：

- **Disconnect**——移除 Personal 管理/观察绑定，保留原生安装和原生数据；
- **Uninstall**——经 daemon 影响预览和 lifecycle procedure 后，移除 Personal-managed
  installation。

预览区分 conversation、Goal、Plan revision、Task、Task-owned attempt、Agent
runtime engine、pending Effect、binding 与 retained data。除非另行明确确认
retention/purge 动作，受治理历史保留。Receipt 明确 removed、retained、unknown
和 incomplete 结果。

## 9. Multi-Agent 工作与 handoff

对 daemon 已接纳 Goal，daemon 可以调度多个独立支持的 Agent：

- 每个 Task 拥有其 attempt；每个 attempt 绑定一个精确 Agent/runtime/epoch；
- handoff 是带来源和目标的显式 event；
- 上游 handoff 失败时，下游 governed work 等待；
- Agent 分歧保持 Native/Observed proposal，直到 daemon 接纳 Plan decision；
- 每个 Task/body 都重新授权共享资源；
- Agent 不能给另一 Agent 授予 permission 或 acceptance。

Multi-Agent 是已采纳的 Personal 2.0 目标并**需要后端**；它不是 Linux 1.0 或非 Pi
qualification 声明。

## 10. 时间线与完成

会话和工作视图共享同一时间线语法：

| Badge | 含义 |
|---|---|
| **Native** | vendor Agent/session 内容或 plan |
| **Observed** | adapter/daemon 观察，未接纳为权威 |
| **Governed** | daemon 接纳、授权、变更与 Effect reconciliation |
| **Verified** | 仅限当前独立验证与 daemon acceptance |

Badge 是 provenance/authority label，不是进度。不得从模型文本生成虚假百分比或 ETA。
只有声明 denominator 才能显示计数。Agent final text、native harness result、
Tool result、Provider response 或 process exit 都不等于 completion。

## 11. 必需状态

| 状态 | 必需处理 |
|---|---|
| 空 | 提供 signed catalog/connect-existing 动作并解释 native-only |
| 加载中 | 说明正在加载 catalog、adapter、conversation、runtime 还是 governed source |
| 部分可用 | 展示受支持的通用核心，并列出不可用原生 facet |
| 权限 | 展示精确 Provider/workspace/resource/native-session scope，并提供拒绝/缩小路径 |
| 错误 | 保留 source/review/conversation context，并提供受支持恢复 |
| 过期 | 显示最后观察时间，阻止不安全推断/动作 |
| 冲突 | fail closed，调用 Agent Shell 解释，解决前要求 daemon preview |
| 成功 | 区分 first chat 与 first governed/verified Task |

## 12. 后端能力缺口

### 后端缺失

- signed catalog onboarding 与 connect-existing workflow；
- 复用 Core ConversationBinding 的通用 conversation/history projection；
- connection-scoped 原生 session 观察与 daemon 接纳；
- Control Plane 上的完整 Agent lifecycle；
- Goal -> Plan revision -> Task -> Attempt 与 Multi-Agent graph/handoff orchestration；
- interrupt/pause/resume/cancel/retry/fork/restart/recover 控制；
- 通用 federated resource synchronization。

### API/原生界面已存在但 UI 未覆盖，或仅部分覆盖

- 原生 dsh panel 是现有独立交互界面。
- 当前 Runtime 与 dsh projection 提供有界 Agent 事实，但没有目标 conversation 或
  lifecycle model。
- 当前 Provider binding 与 Task evidence 可以链接进 dossier，但不能填补缺失的
  Agent 语义。

### 合同/core 缺口

Personal-private 投影复用现有 Core Conversation 与 ConversationBinding。只有新增或
变更公开 common capability、会话扩展、Goal、Plan、Run、Harness、attempt 或
handoff 机器面时，才有条件需要 P10-T02/Lane-CTR。

## 13. 固定边界与非声明

- daemon 是唯一权威写入者。
- 保留原生应用使用；观察与治理都必须显式。
- MCP 加 rules 不能控制 host Agent session。
- Agent、adapter、Shell 或原生 plan 都不能自行接纳、扩大 permission、commit Effect
  或接受 completion。
- Pi 仍是 Linux 1.0 已资格验证的 Agent 路径；其他 Agent 必须独立 qualification。
- 本目标不构成实现、Gate、release、Profile、性能、containment 或 Agent-benefit 声明。
