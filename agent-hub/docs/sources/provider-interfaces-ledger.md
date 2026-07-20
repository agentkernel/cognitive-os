# Source Ledger — Provider 接口

> 类别：informative research source ledger ｜ 查询日：2026-07-20 ｜ owner：Lane-CON
>
> 本 ledger 登记六个 Tier 1 Agent 的官方控制/session 接口一手事实，以及跨 Agent 协议（MCP/ACP/A2A）。**当前多数 Tier 1 一手接口为 `待核验`**：本轮研究主要通过竞品实现间接观察（见 [paseo-and-comparables-ledger.md](./paseo-and-comparables-ledger.md)），间接观察不等于官方合同。禁止把间接观察或营销描述当作已核验接口。填充时每条须含标题/完整 URL/查询日/version 或 commit。

## 1. 跨 Agent 协议

### MCP（Model Context Protocol）

- 事实：当前稳定协议版本固定为 **2025-11-25**（本产品采用基线）。查询日 2026-07-20 存在计划中的后续 RC，**尚未生效**，仅登记不采用。
- 官方来源 URL / 具体 RC 标识：待补齐（modelcontextprotocol 官方规范页 + 查询日）。
- 产品影响：MCP 是 Agent↔工具接口，不是 Host 对 Agent 的接管通道；不得把未生效 RC 当作合同。

### ACP（Agent Client Protocol）

- 事实：客户端↔Agent 的 JSON-RPC over stdio 会话协议；OpenHands、Agent of Empires、Vibe Kanban 等经 ACP 与 Agent 交互（间接观察）。
- 官方来源 URL / version：待补齐。
- 产品影响：可映射 L1/L2，取决于是否 Host 启动子进程。

### A2A（Agent-to-Agent）

- 事实：Agent 间协作协议。
- 官方来源 URL / version：待补齐。
- 产品影响：v1 不作为接管通道；多 Agent 用 Host 内确定性调度器。

## 2. Tier 1 Agent 官方接口

> 下表 `间接观察` 列来自竞品实现（可信度中，需官方确认）；`官方一手` 列为待核验，须填 URL/version。

| Agent | 间接观察到的接口 | 官方一手（待核验） | native 存储（待核验） |
|---|---|---|---|
| OpenAI Codex | App Server；`codex resume`/thread fork | 待核验 | 待核验（`auth.json` 位置见 comparables，需官方确认） |
| OpenCode | server（可 spawn 或连已运行）；session API；ACP | 待核验 | 疑 SQLite，待核验 |
| Claude Agent SDK | Agent SDK 主路径；Claude Code `--resume`；JSONL session | 待核验 | JSONL（`~/.claude/projects`，需官方确认） |
| Hermes Agent | 无（本轮未观察） | 待核验 | 待核验 |
| OpenClaw | 无（本轮未观察） | 待核验 | 待核验 |
| OpenHands | Agent Server(HTTP/WS)；ACP；真 pause/resume | 见 dossier（仓库/release 已核） | 平台自有 conversation（非 provider native） |

## 3. 填充规则

- 每个 Tier 1 Adapter 在其 version/PoC gate 前，必须把本表对应 `待核验` 用官方仓库/文档 + 查询日 + version/commit 补齐。
- 官方接口稳定性分级：稳定 / 实验 / 未在 release 升格。
- session resume/fork 语义、双 writer 保证、凭据文件位置与只读边界都属必填项。
- 缺一手核验的接口不得在能力矩阵声明为 `目标` 之外的任何“已支持”。

## 4. 当前状态

- Tier 1 官方一手接口核验：`未完成`（OpenHands 除仓库/release 外，通用 conversation list API 仍不完整）。
- 因此所有 Adapter 任务保持 `blocked`，等待接口核验 + 条款 + PoC gate。
