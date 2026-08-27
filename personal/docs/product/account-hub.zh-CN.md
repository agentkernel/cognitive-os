# Personal 账户中心

- 状态：已采纳的 Personal 2.0 产品目标
- 规范语言：英文
- 英文规范源：[account-hub.md](account-hub.md)
- 当前权威基础：[Provider Control Plane](provider-control-plane.md)
- 凭据导入边界：
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)

本文是英文规范源的忠实中文翻译；若表述冲突，以英文文档为准。

账户中心是 Settings 中管理 Provider 账户、凭据、自定义端点、模型、daemon
代理配置、路由范围、额度、用量、成本与账户恢复的界面。它默认面向初学者，
但不隐藏治理细节。

## 1. 现实台账

| 边界 | 账户中心事实 |
|---|---|
| **当前实现（Now）** | Provider Control Plane 已支持具名 OpenAI、Anthropic 和自定义 OpenAI-compatible 账户、API key 到 SecretStore 的单向交接、模型发现/手动模型、固定 Agent 绑定、用量、成本、软预算/告警、审计，以及当前 Providers UI。 |
| **已采纳的 Personal 2.0 目标** | 分层账户中心：更多预设、订阅/OAuth、API key、ADR-0055 导入、自定义端点、daemon 代理配置，以及全局/Agent/会话三级路由范围。 |
| **需要后端（Requires-backend）** | 新 Provider 适配器、订阅/OAuth 生命周期、现有凭据导入实现、配置层级、当前会话显式重绑定/重启，以及更广的额度读取。 |
| **有条件需要 core** | 只有新增或变更公开账户/配置/覆盖机器合同时才需要 P10-T02/Lane-CTR；Personal-private 投影未必需要 core 变更。 |

## 2. 首屏与 Provider 顺序

首屏优先展示最常用选择：

1. **OpenAI**
2. **Anthropic**
3. **Google**
4. **DeepSeek**

展开**更多 Provider**后展示：

- Qwen/Bailian；
- Kimi；
- Zhipu；
- SiliconFlow；
- Volcengine-Doubao；
- MiniMax；
- OpenRouter。

**自定义 OpenAI-compatible** 与上述选项同级，不隐藏在笼统的“其他”表单中。

展示某个预设只是已采纳的产品选择，不代表该适配器、订阅方式、额度 API 或
Agent 路径已经实现、通过资格验证、可用或具备 Provider 质量。每个预设只展示
daemon 实际支持的凭据方式与能力。

## 3. 凭据与端点方式

| 方式 | 产品行为 | 当前状态 |
|---|---|---|
| 订阅/OAuth | 用户通过受支持的 Provider 流程授权账户；刷新/撤销由 daemon 所有且全程不记录敏感材料。 | **需要后端** |
| API key | 通过隐藏输入单向交给 daemon；浏览器绝不读回。 | 当前 Provider 类型的**当前实现** |
| 导入现有凭据 | 用户选择一个精确现有来源；daemon 按 ADR-0055 读取并存储。 | 边界已采纳；实现**需要后端** |
| 自定义端点 | 用户配置一等的自定义 OpenAI-compatible 端点；需要时显式审查信任范围。 | 现有 compatible 路径的**当前实现** |

所有由 Personal 管理的方式最终都进入：

1. 经批准的 daemon SecretStore；
2. 非秘密账户元数据；
3. daemon 中介的代理配置；
4. 脱敏的 readiness、模型、用量与审计投影。

原始凭据绝不进入 Agent 配置、浏览器存储、URL、普通配置、SQLite、argv、环境变量、
日志、证据或聊天。Agent、adapter、MCP server 与全局 Agent Shell 都不能获得它。

## 4. 导入现有凭据

ADR-0055 固定以下导入边界：

- 用户发起，并对每个精确来源分别同意；
- 禁止推测性、后台或批量扫描凭据；
- 由 daemon 完成读取和 SecretStore 写入；
- 秘密材料只在来源到目标之间短暂存在于 daemon 进程内存；
- 审计/证据只记录脱敏的来源类型、目标存储、时间与结果；
- 默认保留来源；
- 安全删除来源是每次导入单独选择；
- 不得在来源和目标 SecretStore 之外创建新的明文副本。

导入成功只表示材料到达目标 SecretStore，不证明 Provider 可达、账户权益、模型可用、
额度或 Agent readiness。

## 5. Daemon 代理配置与范围层级

目标产品展示明确的路由范围层级：

1. **全局默认**——Personal 管理的新用途通常使用的默认项；
2. **Agent 覆盖**——指定 Agent 使用不同配置；
3. **会话覆盖**——一个会话使用不同配置。

只有 daemon 已接纳时，更窄范围才优先。该层级不是自动 fallback、负载均衡、
任意逐请求 Provider 选择或权限扩张。

每个选定配置都把 secret 解析与 Provider egress 留在 daemon 内。仅原生使用 Agent
可以继续处于 Personal 之外，但必须标为 Native/Observed，不能表示成受治理代理流量。

### 当前会话行为

配置变更在确认前说明影响：

- 仅应用于新会话；
- 可通过受支持的原生/adapter 路径重绑定当前会话；或
- 需要显式重启会话/runtime。

当前会话绝不会静默切换 Provider、账户、模型、端点或凭据。重绑定/重启保留既有会话和
受治理 attempt 历史。

该层级和当前会话协调均**需要后端**。

## 6. 账户设置流程

账户设置既可直接进入，也可嵌入 Agent onboarding：

1. 选择 Provider/自定义端点和凭据方式；
2. 通过经批准的 daemon 路径输入或授权材料；
3. 审查脱敏端点信任、模型/配置选择和路由范围；
4. 运行有界检查，并分别展示可达性、凭据、模型发现和能力结果；
5. 保存配置并返回发起流程的 Agent 或 Settings 上下文。

可恢复错误后保留非秘密输入。当定价、额度或高级端点细节不是首聊必需条件时，
它们不得阻塞首聊。

## 7. 账户状态与恢复

UI 将 daemon 的精确事实转成易懂分组，但不创建新的权威状态：

- 可用；
- 降级；
- 凭据缺失/已撤销；
- SecretStore 锁定/不可解析；
- 需要端点信任；
- 模型不可用；
- 未知或过期。

网络连接成功本身绝不表示账户可用。模型刷新失败保留最后已知 catalog 与当前绑定。
存在活动绑定时仍须先处理影响，不能直接删除账户。

轮换、撤销、端点变更和账户删除会展示受影响的 Agent/会话及精确当前会话后果。
全局 Agent Shell 可以解释恢复；只有 daemon 能预览并执行。

## 8. 额度、用量与成本

三者是独立读数：

| 读数 | 必须保持的诚实边界 |
|---|---|
| **额度** | Provider 有数据时显示 Provider 报告的总额、余量、重置周期与来源；否则显示不可用 |
| **用量** | 显示实测或估算消耗、周期、账户/模型/Agent 范围与计量来源 |
| **成本** | 显示价格版本、货币/计价依据、估算/报告状态及不可用状态 |

不得从用量推断额度，不得从成本推断用量，未知不等于零。只有声明 denominator
时才能显示百分比、cache hit rate 或剩余比例。软预算和告警不得静默阻断或改路请求。

## 9. 必需产品状态

| 状态 | 账户中心行为 |
|---|---|
| 空 | 解释为何没有账户；提供四个首要预设与自定义端点 |
| 加载中 | 说明正在加载 SecretStore、Provider、模型 catalog、额度、用量还是审计 |
| 部分可用 | 保留可用账户/配置事实，并标明不可用来源 |
| 权限 | 解释精确导入、端点信任或路由范围；允许拒绝或选择更窄路径 |
| 错误 | 保留非秘密输入；说明错误类别与安全的重试/编辑路径 |
| 过期 | 显示最后已知 catalog/额度/用量时间；重要变更前要求刷新 |
| 成功 | 显示脱敏 receipt、所选范围、受影响 Agent/会话与下一动作 |

## 10. 后端能力缺口

### 后端缺失

- 订阅/OAuth 获取、刷新与撤销；
- ADR-0055 的具体导入读取器；
- 当前支持范围之外的 Provider adapter 和权益/额度读取器；
- 全局/Agent/会话配置层级；
- 当前原生会话的显式重绑定/重启协调。

### API 已存在但 UI 未覆盖，或可复用

当前账户、API key、端点信任、模型、绑定、用量、预算、告警和审计能力已经支撑当前
Provider UI。账户中心可以把这些能力重新组织到 Settings，但不能因此声称缺失的目标方式
已经存在。

### 合同/core 缺口

只有新增或变更公开账户、代理配置、订阅或覆盖机器语义时，才有条件需要
P10-T02/Lane-CTR；Personal-private 投影未必需要 core 变更。

## 11. 固定边界与非声明

- 浏览器、Agent、adapter、MCP server 或 Shell 均不保管 secret。
- 无 ambient fallback、负载均衡、任意 auth header 或静默当前会话切换。
- 用量台账不保留 prompt/completion。
- Provider 成功、额度状态、模型响应或进程退出都不等于 Task 完成。
- Personal 不提供多用户/RBAC 或远程公开管理。
- 展示预设和采纳产品目标不构成实现、Provider 质量、Gate、release、Profile、
  性能或 Agent-benefit 声明。
