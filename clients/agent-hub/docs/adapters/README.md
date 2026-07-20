# Agent Hub — Adapter 能力模型与分级

> 类别：informative research/design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：`accepted product direction / 接口已核验（文档级，AH-CTR-02） / implementation not-implemented / evidence not-run`。逐能力矩阵见 [capability-matrix.md](./capability-matrix.md)；接口分层见 [interface-layering.md](./interface-layering.md)；来源事实见 [../sources/](../sources/)。Adapter 事实以各 dossier 的查询日/version/commit 为准；文档级核验 ≠ 已支持/已实现。

## 1. 能力模型

Adapter 是把某个第三方 Agent 的官方接口映射到 Agent Hub 统一控制面的确定性适配层。每个 Adapter 声明：

- 支持的官方控制接口（SDK / App Server / Gateway / ACP / REST / SSE / JSON-RPC / headless / 官方 session API）；
- 适用接管层级（L1–L5、L7）及其安全条件；
- session/文件事实（格式、documented root、resume/fork 语义、双 writer 风险）；
- 账号/凭据模型（能否 opaque handle 化）；
- 平台与版本兼容范围；
- 条款/许可影响；
- 未决与 Open PoC。

Adapter 不得：抢占任意 PID stdin、注入内存、抽取凭据、写 provider 未公开支持的内部数据、绕过登录/计费/安全/组织策略（L8 永久禁止）。

## 2. 分级

### 2.1 Tier 1（首发目标，各自独立 gate）

六个目标 Agent（[CONSOLE-AGENTHUB-V1-DEC-005](../decisions/decision-log.md)），各有独立 dossier：

| Agent | dossier | 主控制接口方向（文档级已核；runtime PoC 仍需） |
|---|---|---|
| OpenAI Codex | [tier1/codex.md](./tier1/codex.md) | CLI / App Server / rollout JSONL + SQLite 状态库 |
| OpenCode | [tier1/opencode.md](./tier1/opencode.md) | `opencode serve` OpenAPI / ACP / SQLite `opencode.db` |
| Anthropic Claude Agent SDK | [tier1/claude-agent-sdk.md](./tier1/claude-agent-sdk.md) | Agent SDK + 官方 session API / JSONL（TS：Commercial ToS） |
| Hermes Agent | [tier1/hermes.md](./tier1/hermes.md) | CLI（chat -q / --resume）+ SQLite `~/.hermes/state.db`；**无对外控制 API（L1 不可达）**；指认 `NousResearch/hermes-agent` |
| OpenClaw | [tier1/openclaw.md](./tier1/openclaw.md) | Gateway WS（端口 18789）/ CLI / SQLite + transcript 只读 |
| OpenHands | [tier1/openhands.md](./tier1/openhands.md) | Agent Server（HTTP/WS）/ ACP / conversation search API |

### 2.2 其它分级

汇总见 [other-tiers.md](./other-tiers.md)：

- **Tier 2 / 候选**：满足官方接口与条款但非首发；
- **Experimental**：仅在 PoC/受限环境评估；
- **Observe-only**：仅 L5/L7 只读观察；
- **Launch-only**：只能 Host-launched，无稳定接管/恢复接口——含 **WorkBuddy（无公开控制面，仅 launch-only）**；
- **Managed-terminal**：主要经受管终端交互——含 **Aider（managed-terminal）**；
- **Blocked / 排除**：不满足安全或接口前提——含 **Roo Code（v1 排除）**。

## 3. 分级不是完成声明

列入 Tier 1 只表示“首发设计目标”。六 Adapter **接口已核验（文档级）** 不表示 Adapter 已实现、runtime PoC 已执行或条款已法务批准。每个 Agent 的 version/account/license/PoC gate 独立，未过实现 gate 前对应 Adapter 任务 `blocked`。
