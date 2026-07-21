# Source Ledger — Provider 接口

> 类别：informative research source ledger ｜ 查询日：2026-07-20 ｜ owner：Lane-CON
>
> 本 ledger 登记六个 Tier 1 Agent 的官方控制/session 接口一手事实，以及跨 Agent 协议（MCP/ACP/A2A）。**2026-07-21 回填**：六 provider 完成「接口已核验（文档级）」；runtime 行为与 Open PoC 仍为 `evidence not-run`。禁止把文档级核验写成实现已提供、测试已执行或 Profile 已符合。填充时每条须含标题/完整 URL/查询日/version 或 commit。

## 1. 跨 Agent 协议

### MCP（Model Context Protocol）

- 事实：当前稳定协议版本固定为 **2025-11-25**（本产品采用基线）。官方原文：「The current protocol version is 2025-11-25」。
- 官方来源 URL：https://modelcontextprotocol.io/specification/versioning （查询日 2026-07-20）。
- 计划中的后续 RC：具体 RC 标识本轮未核验，**仅登记不采用**。
- 产品影响：MCP 是 Agent↔工具接口，不是 Host 对 Agent 的接管通道；不得把未生效 RC 当作合同。

### ACP（Agent Client Protocol）

- 事实：客户端↔Agent 的 JSON-RPC over stdio 会话协议；**稳定协议版本 = 1**（库最新 release v1.4.0，2026-07-06）；v2 为 Active RFD 集合，未发布。
- 官方来源 URL：https://agentclientprotocol.com ；仓库 https://github.com/agentclientprotocol/agent-client-protocol （Apache-2.0；查询日 2026-07-20）。
- 消歧：与 Virtuals「Agent Commerce Protocol (ACP)」重名，不相干。
- 产品影响：可映射 L1/L2，取决于是否 Host 启动子进程。OpenCode / OpenHands 均有官方 ACP 入口。

### A2A（Agent-to-Agent）

- 事实：Agent 间协作协议。
- 官方来源 URL / version：待补齐（本轮未核验）。
- 产品影响：v1 不作为接管通道；多 Agent 用 Host 内确定性调度器。

## 2. Tier 1 Agent 官方接口

> `接口已核验（文档级）` = 官方仓库/文档一手取证已登记；不等于 runtime PoC pass，也不等于实现已提供。

| Agent | 官方一手（文档级） | native 存储（文档级） | 核验状态 |
|---|---|---|---|
| OpenAI Codex | 仓库 https://github.com/openai/codex （Apache-2.0）；release `rust-v0.144.6`（2026-07-18）；`codex exec` / `codex resume` / `codex app-server`（JSON-RPC；stdio 默认，WS experimental）；docs：https://developers.openai.com/codex/app-server 、CLI reference、auth | rollout JSONL：`$CODEX_HOME/sessions/YYYY/MM/DD/rollout-*.jsonl`（默认 `~/.codex`）；**SQLite 状态库并存**（`CODEX_SQLITE_HOME`）；凭据 `auth.json` 或 OS keyring | 接口已核验（文档级）；evidence not-run |
| OpenCode | 仓库 https://github.com/anomalyco/opencode （原 `sst/opencode`，MIT）；release v1.18.3（2026-07-16）；`opencode serve`（OpenAPI 3.1 `/doc`）；`opencode acp`；session REST + SSE；docs：https://opencode.ai/docs/server/ | **当前为 SQLite** `~/.local/share/opencode/opencode.db`（自 v1.2.x；JSON→SQLite 迁移，遗留 JSON 布局可能残留）；`auth.json` 同根 | 接口已核验（文档级）；evidence not-run |
| Claude Agent SDK | 文档 https://code.claude.com/docs/en/agent-sdk/sessions ；TS npm `@anthropic-ai/claude-agent-sdk` **0.3.215**（Commercial ToS）；Python **0.2.123**（wrapper MIT）；官方 session API：`listSessions` / `getSessionMessages` / `forkSession` 等 | JSONL：`~/.claude/projects/<encoded-cwd>/<session-id>.jsonl`（官方路径+编码规则已核） | 接口已核验（文档级）；evidence not-run |
| Hermes Agent | **决定指认** `NousResearch/hermes-agent`（MIT；release v2026.7.7.2）；CLI `hermes chat -q` / `--resume` / `sessions list`；gateway 为消息渠道；**无文档化对外控制 API（L1 不可达）**；docs：https://hermes-agent.nousresearch.com/docs/ | SQLite `~/.hermes/state.db`（WAL；官方明示取代早期 per-session JSONL） | 接口已核验（文档级）；身份 decided-with-rationale；evidence not-run（PoC 仍需） |
| OpenClaw | 仓库 https://github.com/openclaw/openclaw （MIT / OpenClaw Foundation；v2026.7.1）；**Gateway WS** 控制面（默认端口 18789）；CLI `openclaw sessions --json` 等；docs：https://docs.openclaw.ai/concepts/session 、reference/rpc | 运行时 SQLite `~/.openclaw/agents/<agentId>/agent/openclaw-agent.sqlite` + 归档 transcript `.../sessions/` | 接口已核验（文档级；Gateway WS 字段级 partial）；evidence not-run |
| OpenHands | SDK https://github.com/OpenHands/software-agent-sdk v1.36.1（MIT）；Canvas https://github.com/OpenHands/agent-canvas **v1.5.2**（MIT）；Agent Server HTTP/WS；**conversation search API 已有文档**（`GET /api/conversations/search`）；ACP 双向 | 平台自有 conversation（非第三方 native）；磁盘布局无公开合同（PoC） | 接口已核验（文档级）；evidence not-run |

## 3. 填充规则

- 每个 Tier 1 Adapter 在其 version/PoC gate 前，必须把本表对应项用官方仓库/文档 + 查询日 + version/commit 保持新鲜。
- 官方接口稳定性分级：稳定 / 实验 / 未在 release 升格（例：Codex app-server WS = experimental/unsupported）。
- session resume/fork 语义、双 writer 保证、凭据文件位置与只读边界都属必填项；文档级核验后仍须 Open PoC 闭合 runtime 断言。
- 缺一手核验的接口不得在能力矩阵声明为 `目标` 之外的任何“已支持”。文档级核验 ≠ 已支持。

## 4. 当前状态

- Tier 1 官方一手接口核验（文档级）：**六 provider 已完成回填**（AH-CTR-02 文档级进展；Hermes 身份由协调者决定为 `NousResearch/hermes-agent`）。
- runtime / Open PoC / 条款法务评估：全部 `not-run`；Adapter 实现任务保持 `blocked`（等 PoC + 条款 + 后端/ADR gate）。
- 间接观察（comparables ledger）仅作历史对照，不再作为接口事实来源。
