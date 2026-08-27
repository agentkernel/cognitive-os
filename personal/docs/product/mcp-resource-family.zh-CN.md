# MCP 资源族

- 状态：已采纳的 Personal 2.0 第七资源族产品目标
- 规范语言：英文
- 英文规范源：[mcp-resource-family.md](mcp-resource-family.md)
- 决策：
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)
- 相关文档：[认知资源模型](cognitive-resource-model.md)、
  [Resource Manager](resource-manager-design.md) 与
  [Agent 集成与会话](agent-integration-and-conversations.zh-CN.md)

本文是英文规范源的忠实中文翻译；若表述冲突，以英文文档为准。

Personal 2.0 将 MCP 采纳为真正的第七认知资源族。该资源族管理 MCP server
身份、安装、健康、权限、更新以及向兼容 Agent client 的投影。它不会把 MCP
变成权威平面或 host session 控制器。

## 1. 现实台账

| 边界 | MCP 事实 |
|---|---|
| **当前实现（Now）** | Linux 1.0 有六个资源族。P5-T03/P5-T04 已在 Tool 资源族内交付 MCP Tool transport 与有界 dynamic-Tool 路径；它们没有实现 MCP 资源族 manager、authority-backed MCP inventory、server lifecycle、permission/update workflow 或通用 client projection。 |
| **已采纳的 Personal 2.0 目标** | MCP 是 Library 中的第七资源族，具备 server install、health、permission、update、client projection、conflict handling 与 Activity。 |
| **需要后端（Requires-backend）** | 所有 Personal MCP runtime management、adapter/client projection、health、permission、update、synchronization 与 recovery 行为。 |
| **有条件需要 core** | ADR-0058 将 MCP family 保持为 Personal-private。日后若新增公开 MCP 机器面，需要新的 Lane-CTR 裁定。 |

## 2. 为什么 MCP 是资源族

MCP 具有不同于 Tool、Context 和 Agent 的产品生命周期：

- server 有 source、version、trust/provenance、compatibility、health、permission、
  update posture 与 projected client；
- 一个 server 可暴露多种 candidate capability；
- server 可以健康，但某个 permission 或 client projection 被拒绝或失败；
- 更新 server 不等于启用 Tool；
- 连接 server 不等于授权 capability；
- 配置 client 不等于控制 host session。

因此 MCP 不是 Tool alias、通用 transport label 或 Agent 属性。Tool 和 Context
仍是独立资源族；只有经过映射与授权后，才能使用 MCP 来源的 capability。

## 3. 产品位置

**Library → MCP** 提供：

- 已安装/已连接 server 列表；
- source、version、trust/provenance 与 compatibility；
- health 与 last observation；
- requested permission 与 admitted permission；
- Agent-client projection 及 freshness；
- update availability 与 current version；
- quarantine/requalification state、conflict、blocked reason、Activity 与 receipt。

相关 Agent inspector 展示投影到该 Agent 的 MCP server。Work 展示某 Task
实际接纳的 MCP-originated Tool 或 Context 事实。Settings 管理全局 permission/default
policy，不承载 server inventory。

## 4. Server 安装与连接

**已采纳的 Personal 2.0 目标**

1. 用户选择 server source。
2. Personal 展示 identity、version、trust/provenance、可用时的 license、
   adapter/client compatibility、requested permission、受影响 Agent client
   与 update behavior。
3. daemon 发出重要 install/connect preview。
4. 确认只授权精确 server 与 scope。
5. daemon 执行该资源族专属 lifecycle 并记录 durable result。
6. health、permission 与 client projection 分别评估。

Install/connect 不授予 Tool、Context、workspace、network、model、secret 或
host-session authority。仅有健康进程并不表示 server 可用；所需 permission 和
client projection 也必须有效且最新。

精确 acquisition 与 trust mechanism 属于后端/core 决策，本文不虚构。

## 5. 健康与兼容性

Health 回答受管 server 在观察时刻能否提供所声明的 MCP 服务。它不回答：

- 某个 Agent client 是否已配置；
- permission 是否已接纳；
- capability 是否已映射为 Personal Tool 或 Context source；
- Task 是否可使用；
- host Agent session 是否运行或可控；
- outcome 是否 verified。

产品分别保留：

- server lifecycle/health；
- protocol/client compatibility；
- permission；
- projection/configuration；
- mapped capability availability；
- Task-specific authorization。

Unknown 与 stale 不等于 healthy。Process exit 或 handshake success 是 observation，
不是 Task completion。

## 6. 权限

按 scope 与 consequence 审查 permission：

- server 运行所需 process/network access；
- server 可暴露或消费的数据/资源类别；
- 可接收配置的兼容 Agent client；
- capability 可映射进入的 Personal family；
- write-back/configuration target；
- 适用时的 retention 与 update behavior。

安装 server 不会隐式授予任何上述权限。Permission expansion 始终属于重要动作，
必须获得新的 daemon preview 与用户确认。MCP server、Agent、Skill 或 adapter
都不能自行扩大 scope。

## 7. Client 投影

Projection 表示配置兼容 Agent client 以认识该 server；不表示控制 Agent 的 live session。

优先顺序：

1. 存在且有界时使用 **vendor-native session/configuration API**；
2. 能提供等价 governed semantics 时使用 **managed vendor adapter path**；
3. 使用 **MCP 加 vendor rules** 作为 cooperative fallback。

Fallback 可以通过 daemon 的 governed write-back path 准备或变更受支持配置。
它不能 interrupt、pause、resume、restart 或以其他方式控制 host Agent session。
如果 host 需要 reload/restart，UI 必须明确说明并请求单独受支持动作。

每个 projected client 独立报告 success、failure、permission denial、incompatibility
和 staleness。Partial projection 绝不能显示成 complete。

Agent client 观察仅限于连接该 Agent 时建立的明确 observation scope。Personal
不执行推测性/全局原生 session 扫描，也不会把新发现的 session 意外自动加入。

## 8. 管理员预授权配置

server/client/scope 的第一次配置需要显式授权。授权之后，只有下列条件全部保持不变时，
Personal 才可自动应用兼容配置：

- server identity/version compatibility；
- target Agent client；
- 精确 permission 与 write-back scope；
- endpoint/trust boundary；
- 已批准 configuration class。

这是 admin-preauthorized automation，不是 ambient authority。任何 permission
expansion、新 client、更广 target、变更 trust boundary 或 incompatible update
都需要新的 daemon preview 与确认。

每次 configuration write-back 仍是持久化 Intent/Effect 操作，具有 reconciliation
与 durable receipt。全局 Agent Shell 可以解释或提出动作，但不能执行。

## 9. Capability 映射

MCP server 可以描述 capability，但 Personal 将其视为 candidate：

- operation 只有经过 Tool registration、descriptor、availability、permission、
  budget 与 dispatch policy 才能映射为 Tool；
- data/retrieval 只有经过 authorization-before-ranking、provenance、freshness、
  loss 与 Task-specific selection 才能映射为 Context；
- prompt 或 reusable instruction 作为 Skill candidate 进入 Skill package/revision、
  provenance、binding、enablement 与 admission 规则；
- returned content 不会自动成为 Memory；
- server/client state 不会成为 Runtime authority；
- MCP output、Tool result 或 server success 不会完成 Task。

映射保留 server origin 与 version，使 Activity 和 evidence 可以追溯来源。

## 10. 更新、恢复与移除

### 更新

目标在确认前展示 current version、available version、compatibility、permission
变化、受影响 client/work 与 recovery expectation。扩大 permission 或 trust scope
的更新始终需要确认。

### 恢复

产品区分 server unhealthy、quarantined/requalification required、
client projection stale、permission denied、configuration conflict 与
host reload required。未知外部结果之后绝不盲目 redispatch。

### 移除

移除前预览受影响 Agent client、Tool/Context mapping、active Task、pending Effect、
configuration write-back 与 retained history。移除 server 不会静默删除无关原生配置
或 governed evidence。

所有 lifecycle 行为都**需要后端**。

## 11. 联邦所有权与冲突

- server/origin 拥有原生内容和 protocol behavior。
- Personal 拥有 admitted governance、binding、permission、synchronization intent
  与 authority receipt。
- read/change detection 只能在连接 Agent 时建立的明确 observation scope 内自动进行。
- 每次 Personal-to-native configuration write-back 都由 daemon 通过 Intent/Effect
  执行。在精确 daemon grant/risk policy 不变时可以自动执行；新增、更广、破坏性或
  冲突 scope 必须获得预览与确认。
- 并发或不兼容变更 fail closed。
- 全局 Agent Shell 解释 conflict，并请求 daemon 提供 family-specific resolution
  preview。
- 不假定 last-writer-wins 或由模型选择解决方案。

Bidirectional synchronization 是已采纳目标并**需要后端**。

## 12. Activity 与完成

MCP event 使用共享时间线 badge：

- **Native**——server 或 vendor-client fact；
- **Observed**——adapter/daemon observation；
- **Governed**——已接纳的 permission、configuration、lifecycle 或 mapping；
- **Verified**——有定义时的当前独立验证。

只有声明 denominator 才显示 health、installation、projection 和 permission count。
不得推导虚假百分比或 ETA。MCP server success、Agent final text、Tool result 或
process exit 不等于 Task completion。

## 13. 必需状态

| 状态 | MCP 行为 |
|---|---|
| 空 | 解释该资源族并提供 install/connect |
| 加载中 | 说明正在加载 server、health、permission、client 或 update source |
| 部分可用 | 列出每个成功/失败/未知 client projection |
| 权限 | 展示精确 requested scope、拒绝/缩小路径与受影响 client |
| 错误 | 保留 source/configuration input，并提供安全恢复 |
| 过期 | 显示 observation age；阻止不安全 update/write-back 推断 |
| 冲突 | fail closed，并要求 daemon-backed resolution |
| 成功 | 展示 durable receipt、health/permission 区别、projected client 与下一动作 |

## 14. 后端能力缺口

### 后端缺失

- MCP server inventory 与 lifecycle；
- health/compatibility projection；
- permission 与 update workflow；
- vendor-native 和 cooperative client projection；
- admin-preauthorized configuration；
- capability mapping 与 federated conflict handling。

### API/原生界面已存在但 UI 未覆盖，或可复用

Vendor Agent 可能已有原生 configuration/session API，部分 MCP server 也独立于
Personal 存在。这些界面只是 integration input，不是 Personal governance 或
host-session control。

### 合同/core 缺口

MCP 已是采纳的第七产品资源族，实现仍**需要后端**。
[ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)
将 family 保持为 Personal-private；日后若新增公开 lifecycle、permission、
mapping、projection、error 或 transition 机器面，需要新的 Lane-CTR 裁定。

## 15. 固定边界与非声明

- daemon-only authority 与 persist-before-dispatch 不变。
- MCP 永远不能获得原始 Provider credential 或 SecretStore access。
- MCP 加 rules 不能控制 host Agent session。
- 连接、health、configuration 或 capability discovery 本身不授予 permission。
- MCP support 不会资格验证 Agent、Tool、server 或 release。
- Linux 1.0 仍是六资源族且仅 Pi 已资格验证。
- 本目标不构成实现、Gate、release、Profile、性能、containment 或 Agent-benefit 声明。
