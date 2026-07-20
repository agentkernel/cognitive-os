# Agent Hub — 产品设计

> 类别：informative product design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：`accepted product direction / implementation not-implemented / evidence none / Profile not implemented`

## 1. 一句话定义

Agent Hub 让用户在 PC 和手机上统一发现、安全接管、创建、监督、纠偏和收敛他们有权管理的成熟第三方 Agent，同时始终可见当前部署模式、接管层级、事实来源和不保证事项。

Agent Hub 扩展现有 Console `/agents` 入口（[CONSOLE-V2-DEC-007](../../../../../clients/pc/docs/product/decision-log.md)），不是替换；它显式区分 Direct Takeover 与 CognitiveOS Governed 两种部署模式，保证矩阵见 [deployment-modes-and-guarantees.md](./deployment-modes-and-guarantees.md)。

## 2. 首要用户与 JTBD

首要 persona：Agent 操作者 / 高级终端用户（沿用 [CONSOLE-V2-DEC-001](../../../../../clients/pc/docs/product/decision-log.md)）。核心 JTBD：

1. 发现已安装 Agent、可安全接管的运行进程/终端会话/native session；
2. 启动新的受管 Agent，或通过官方接口接管已有 session；
3. 附着受管终端、在无法安全接管时只读观察；
4. 创建、派发、监督、纠偏、暂停、取消和恢复任务；
5. 查看终端、diff、文件变化、产物、费用、配额和等待事项；
6. 组织群组式多 Agent 协作，控制 worktree、文件、Git、允许的电脑资源；
7. 统一查看和选择模型、账号及 LLM API key；
8. 从手机远程监督和控制 PC 上的受管 Agent；
9. 未来升级到完整 CognitiveOS 时保留来源和历史。

## 3. 产品原则

1. **先回答用户问题，再展示系统结构。** 页面先回答：正在发生什么、该 Agent 是否真正受管、是哪种接管层级、哪个账号/主机/工作区、我能做什么、哪些不安全、如何释放控制、如何验证结果。
2. **模式与保证必须显式。** 每个写页面持续显示 mode、Host、账号、workspace、事实来源和 freshness；Direct 与 Governed 的保证不可视觉混同。
3. **接管分级、可审计、可降级。** 七个结果标签（见治理）不能用一个 “Supported” 徽章掩盖；不安全时降级为只读观察，不伪装成功。
4. **不可信输入永远是数据。** Agent 文本、终端字节、网页、文件、包来源、session 文件均为不可信输入。
5. **不确定性是一级状态。** `result-unknown`、`pause-pending`、`stale`、`redacted`、`conflict` 各自独立表达。
6. **最小权限与本机确认。** 首次外部介入、扩权、observe→write、信号、桌面控制、新 credential、提权默认 PC-local 确认。
7. **安全路径也必须可访问。** 键盘、屏幕阅读器、高对比、缩放、reduced motion 用户可完成同等强度的确认/暂停/恢复/对账。

## 4. 范围（Windows Direct v1）

- 部署模式：Direct Takeover 先行，Governed 保持完整迁移设计并等待后端 gate。
- 首发平台：Windows；iPhone 先于 Android（均保留完整设计）。
- Windows GA floor：Windows 11 25H2+ 为主 GA；Windows 11 24H2+ 兼容（按 edition/build 动态移除）；Windows 10 22H2+ 仅在有效 ESU 下作 Experimental/PoC。见 [platforms/product-scope.md](../platforms/product-scope.md)。
- Tier 1 目标 Agent（各自独立 gate）：OpenAI Codex、OpenCode、Anthropic Claude Agent SDK、Hermes Agent、OpenClaw、OpenHands。
- 允许接管层级：L1–L5 + L7（详见保证矩阵）；L6 未来条件；L8 永久禁止。
- 默认接管路径：Host-launched（L2）。

## 5. 非目标（v1）

- 不实现任何客户端、Host、Adapter、Relay、Vault、UI、脚手架、mock、机器合同或测试代码（本文件是 informative 设计）。
- 不做任意 PID 注入、内存 patch、二进制篡改、token/cookie/keychain 抽取、绕过登录/计费/安全/组织策略。
- 不把 Direct WorkItem 命名为 Task、Host action 命名为 Effect，或把 Direct 记录当 authority。
- 不承诺 Tier 1 之外 Agent 首发接入；不把 WorkBuddy 当作有公开控制面（launch-only）。
- 不引入通用桌面 GUI 全控制（首发仅 selected-window，见 [security/computer-control.md](../security/computer-control.md)）。

## 6. 信息架构（PC）

Agent Hub 专属 shell 使用任务导向一级入口，扩展而非改写现有六分组导航：

| 入口 | 用户问题 | 主要内容 |
|---|---|---|
| 工作 | 我现在要做什么？ | 统一工作项、最近工作、创建入口、Conversation |
| Agent Hub | 我有哪些能力、如何接管？ | 安装/进程/session/file 候选、Adapter、接管向导与预览 |
| 群组 | 多 Agent 如何协作？ | Lead+Workers、worktree、handoff、conflict、review（未启用时隐藏） |
| 收件箱 | 哪些事项等我？ | permission、clarification、result-unknown、Host offline、credential、local-confirm-required |
| 主机 | 本机/远端 Host 是否可用？ | Host health、process tree、terminals、ownership generation、paired devices |
| 系统 / 设置 | 账号密钥、电脑控制、来源与保证 | 账号与密钥、电脑控制、来源与保证、隐私、更新 |

手机作为 remote companion，使用 Work / Tasks / Agents / Inbox / More 五入口，Team 置于 Agents 二级；详见 [journeys-and-screens.md](./journeys-and-screens.md)。

## 7. 成功定义

在获得真实 baseline 前不虚构数值目标。候选指标（分模式分层）：

- 首次可用任务时间；
- 可安全接管会话比例、`managed-from-start` / `officially-adopted` / `observe-only` 比例；
- 错误 PID/session 接管数、越界文件读写数、ownership generation 冲突数（目标为零类）；
- 跨端恢复成功率、permission 等待时间、并行冲突率、Adapter 升级回归率；
- unknown outcome 收敛时间、误报完成数、secret 泄露数（安全类目标为零）；
- 无障碍关键旅程成功率。

约束：安全失败不能被效率收益抵消；Direct 的 `user-accepted` 不计入 Governed 北极星指标（每位周活操作者成功收敛的受验证任务数）。

## 8. 相关文档

- 保证矩阵：[deployment-modes-and-guarantees.md](./deployment-modes-and-guarantees.md)
- 旅程与页面：[journeys-and-screens.md](./journeys-and-screens.md)
- 状态/内容/无障碍：[states-content-and-accessibility.md](./states-content-and-accessibility.md)
- 架构：[../architecture/takeover-architecture.md](../architecture/takeover-architecture.md)
- 安全：[../security/threat-model.md](../security/threat-model.md)
- 决策：[../decisions/decision-log.md](../decisions/decision-log.md)
