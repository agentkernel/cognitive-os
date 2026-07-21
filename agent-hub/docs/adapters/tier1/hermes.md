# Adapter Dossier — Hermes Agent

> 类别：informative research ｜ 日期：2026-07-20（2026-07-21 AH-CTR-02 文档级回填）｜ owner：Lane-CON
>
> 状态用语：**接口已核验（文档级）** / product-only / not-implemented / **evidence not-run**。
>
> ### 身份指认（decided-with-rationale）
>
> **协调者决定（2026-07-21）**：Tier 1「Hermes Agent」指认 **`NousResearch/hermes-agent`**（https://github.com/NousResearch/hermes-agent ）。
>
> **理由**：名称与「Hermes Agent」一致；MIT；活跃 coding-agent CLI/gateway；官方 session 存储文档；与 OpenClaw 生态有迁移命令（`hermes claw migrate`）；为当前唯一符合规模与命名的主流候选。Interface Auditor 原标 partial/推断，现由协调者升格为 **decided**。
>
> **仍需 PoC**：无对外控制 API 的实证、`state.db` WAL 只读安全、原生 Windows 路径、gateway 凭据落盘——见 Open PoC；不得因文档级指认宣称实现或 Profile 符合。

## 身份

- 官方仓库：https://github.com/NousResearch/hermes-agent （Python；MIT；查询日 2026-07-20）。
- 适用基线：release **v2026.7.7.2**（2026-07-08，CalVer）。
- 官方文档：https://hermes-agent.nousresearch.com/docs/ （cli / sessions）；session 存储设计：仓库 `website/docs/developer-guide/session-storage.md`。
- 维护状态：活跃。
- 许可：**MIT**。

## 官方控制接口（一手）

- **CLI/TUI**：`hermes`；`hermes chat -q`（非交互）；`--continue` / `--resume <id_or_title>`；`hermes sessions list/rename`；`-w` worktree 并行。
- **Gateway**：`hermes gateway`（Telegram/Discord/Slack/WhatsApp/Signal/Email 等消息渠道；非通用控制面）。
- **对外程序化控制 API：无文档化稳定合同** → **L1 结构化通道不可达**；可编程面 = 子进程 CLI + 只读 SQLite。
- 稳定性：CLI/resume/sessions 为文档化能力；dashboard 读 `state.db` 但不构成公开 HTTP/WS 控制协议。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | **不可达** | 无对外控制 API |
| L2 Host-launched | 目标 | Host 启动 `hermes chat -q` / TUI |
| L3 session 采用 | 条件 | 官方 `--resume`；写需单 writer 证明 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 只读 | SQLite `~/.hermes/state.db` |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- **SQLite**：`~/.hermes/state.db`（WAL；表含 sessions/messages/FTS 等）；`HERMES_HOME` 可改根目录。
- 官方明示该库取代早期 per-session JSONL。
- 辅助：`~/.hermes/sessions/sessions.json`（gateway 路由索引）；`saved/*.json`（`/save` 导出）。
- WAL 并发只读安全 → PoC。

## 账号与凭据

- 工具无账号墙；模型：Nous Portal / OpenRouter / 自有 endpoint。
- 仅 opaque handle；不云同步凭据。

## 平台

- 官方口径：**Linux、macOS、WSL2**；原生 Windows **未见官方支持声明**（待 PoC / 标风险）。

## 未决与 Open PoC

- 确无对外控制端口；`state.db` schema_version 迁移；WSL2/Windows；gateway 凭据权限位。
- Open PoC：POC-SESS-001、POC-FILE-002、POC-SEC-003——状态 **not-run**。
- gate：Adapter 实现仍 `blocked`（PoC + 后端/ADR/条款）；文档级指认不解除实现 gate。

## 产品映射

- 适配形态 ≈ L2 Host-launched CLI + 条件 L3（`--resume`）+ L5 SQLite 只读 + L7；**不承诺 L1**；证据全 not-run。
