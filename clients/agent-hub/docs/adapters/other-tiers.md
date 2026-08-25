# Agent Hub — 其它分级 Agent 汇总

> 类别：informative research/design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文汇总 Tier 1 之外的 Agent 分级。所有分级为设计判断，事实以对应 source ledger 查询日为准；未核验项标 `待核验`。不承诺 Tier 1 之外首发接入。

## 1. 分级定义

- **Tier 2 / 候选**：具备官方接口且条款可能允许，但非首发；待 Tier 1 稳定后评估。
- **Experimental**：仅在隔离 PoC/受限环境评估，不进入 GA。
- **Managed-terminal**：主要经受管终端交互（L4 为主），结构化控制弱。
- **Launch-only**：只能 Host-launched（L2），无稳定接管/恢复接口。
- **Observe-only**：仅 L5/L7 只读。
- **Blocked / 排除**：不满足安全或接口前提，v1 排除。

## 2. 明确分级项

| Agent | 分级 | 依据（详见 source ledger） |
|---|---|---|
| WorkBuddy | **Launch-only** | 无公开控制面/稳定接管接口；只能 Host 启动后监管，不承诺 session 恢复或结构化控制 |
| Aider | **Managed-terminal** | 主要经终端交互；采用仅限 Host-owned 受管终端（L4），不抢占既有 console |
| Roo Code | **Blocked（v1 排除）** | v1 不接入；接口/条款/安全前提未满足，待未来重新评估 |

## 3. 竞品项目（作为参考，非接入目标）

以下项目在研究中作为**行为参考与反例来源**（[../sources/paseo-and-comparables-ledger.md](../sources/paseo-and-comparables-ledger.md)），本产品不接入、不复用其代码：Paseo、Happy、Vibe Kanban、Agent Deck、Agent of Empires、Claude Squad、amux、tmux-agent-tools、Nimbalyst、Omnara、Opcode/Claudia、Open WebUI/Computer。

它们主要示范以下模式（详见 ledger）：daemon/server-owned child、official provider session resume、native file observation、terminal multiplexer attach、platform-owned conversation、remote/mobile；均**未发现**任意 PID 内存注入或普通进程 stdin 抢占的安全实现。

## 4. 分级纪律

- 任何 Agent 从其它分级升入 Tier 1 需：官方接口一手核验 + 条款允许性 + 独立 gate。
- Launch-only/Observe-only Agent 的 UI 不得显示高层级接管徽章。
- Blocked/排除项不得以“临时接入”绕过安全前提。
