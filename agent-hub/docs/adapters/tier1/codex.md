# Adapter Dossier — OpenAI Codex

> 类别：informative research ｜ 日期：2026-07-20（2026-07-21 AH-CTR-02 文档级回填）｜ owner：Lane-CON
>
> 状态用语：**接口已核验（文档级）** / product-only / not-implemented / **evidence not-run**。不得写成实现已提供或 Profile 已符合。

## 身份

- 目标：OpenAI Codex（CLI + App Server + `@openai/codex-sdk`）。
- 官方仓库：https://github.com/openai/codex （Rust；Apache-2.0；查询日 2026-07-20）。
- 适用基线：release **`rust-v0.144.6`**（2026-07-18）；SDK `@openai/codex-sdk` 0.144.6（与 CLI 锁步）。
- 维护状态：活跃（0.x 高频发版）。
- 许可：仓库与 SDK **Apache-2.0**；服务条款允许性仍待法务评估（POC-LIC-002）。

## 官方控制接口（一手）

- **`codex exec`（headless）**：`--json` JSONL 事件流；`codex exec resume`；sandbox 模式；官方非交互文档。
- **`codex resume`**：交互式按 thread ID / `--last` 恢复。
- **`codex app-server`**：JSON-RPC 2.0；传输 stdio（默认）、**ws experimental/unsupported**、Unix socket；方法族含 `thread/*`、`turn/*`（含 steer/interrupt）、`command/exec/*`；`thread/fork` 支持 ephemeral；schema 可 `generate-ts` / `generate-json-schema`。
- **远端 / daemon**：`codex --remote`；`codex remote-control start`。
- **MCP**：`codex mcp`（管理）与 `codex mcp-server`（自身作 MCP server）。
- **SDK**：官方定位自动化/CI 用 SDK；深度集成优先 app-server。
- 稳定性：app-server 大量 experimental 字段；WS 明确 unsupported——漂移监测必覆盖（AH-CTR-04）。
- 来源：https://developers.openai.com/codex/cli/reference 、/noninteractive 、/app-server 、/auth 、/environment-variables 。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标 | App Server / SDK（文档级） |
| L2 Host-launched | 目标 | Host 启动 Codex 会话 |
| L3 session 采用 | 条件 | 官方 thread resume/fork；写采用需单 writer 证明 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 目标（条件） | rollout JSONL + SQLite 状态库并存；角色关系需 PoC |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | `externalAgentConfig/import` 可写他家配置/会话 → L6 类风险关注 |

## session / 文件事实

- **rollout JSONL**：`$CODEX_HOME/sessions/YYYY/MM/DD/rollout-<timestamp>-<thread-uuid>.jsonl`（`CODEX_HOME` 默认 `~/.codex`）。
- **SQLite 状态库并存**：`CODEX_SQLITE_HOME`（默认同 `CODEX_HOME`）；`thread/list` 有 `useStateDbOnly`——L5 采集须处理双存储（PoC）。
- resume/fork：官方 thread ID（新进程回放）；fork 须显示新旧映射。
- `thread/resume` 不更新 rollout mtime（turn 开始才更新）——文件观察器可用语义。

## 账号与凭据

- 登录：ChatGPT 订阅额度或 API key；企业 access token 用于非交互。
- 凭据：`~/.codex/auth.json`（官方明示 plaintext 可选）或 OS keyring（`cli_auth_credentials_store`）。
- 规则：不复制/刷新/回写凭据；仅 opaque handle。

## 平台

- 官方支持 Windows/macOS/Linux（CLI 文档口径）；Windows 终端后端须固定 ConPTY（行为 PoC）。

## 未决与 Open PoC

- JSONL 行 schema 稳定性；SQLite/JSONL 主从；双 writer；`--ephemeral`（unverified）；ToS 包装边界。
- Open PoC：POC-SESS-001、POC-SESS-002、POC-SEC-003——状态 **not-run**。

## 产品映射

- App Server 为 L1 主路径；thread resume 为 L3 条件；native 写归 L6 阻断；证据全 not-run。
