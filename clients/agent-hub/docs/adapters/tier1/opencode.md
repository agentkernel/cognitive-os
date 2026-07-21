# Adapter Dossier — OpenCode

> 类别：informative research ｜ 日期：2026-07-20（2026-07-21 AH-CTR-02 文档级回填）｜ owner：Lane-CON
>
> 状态用语：**接口已核验（文档级）** / product-only / not-implemented / **evidence not-run**。

## 身份

- 目标：OpenCode（开源 coding agent；含 server/session/ACP）。
- 官方仓库：https://github.com/anomalyco/opencode （原 `sst/opencode`，301 重定向有效；MIT；查询日 2026-07-20）。
- 适用基线：release **v1.18.3**（2026-07-16）；默认分支 `dev`；近乎每日发版。
- 维护状态：活跃。
- 许可：**MIT**（开源构建）；托管 Zen 服务另受 ToS。

## 官方控制接口（一手）

- **`opencode serve`**：headless HTTP（默认 `127.0.0.1:4096`）；OpenAPI 3.1 于 `/doc`；Basic auth 经环境变量（**默认无认证**）；`--mdns` / `--cors`。
- **REST**：`/session` CRUD + fork/abort/share/revert/message/`prompt_async`；**`/session/:id/permissions/:permissionID`**；SSE `/event`；**`/tui/*`** 远程驱动 TUI；`/auth/:id` 写凭据。
- **CLI**：`opencode run`（`--format json`、`--attach`、`--auto`）；`opencode acp`（stdio ACP server）；`opencode db` / `db path`；session export/import（`--sanitize`）。
- 来源：https://opencode.ai/docs/server/ 、/cli/ 、/providers/ 、/troubleshooting/ 。
- 稳定性：高频发版 + 2026 存储迁移 → 漂移风险高。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标 | server OpenAPI / ACP（文档级） |
| L2 Host-launched | 目标 | Host spawn `opencode serve` |
| L3 session 采用 | 条件 | `--attach` 连接已运行 server；写需单 writer/lease 证明 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 只读 | **SQLite `opencode.db`（已核）**；迁移期可能残留 JSON 双布局 |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- **当前存储（≥v1.2.x）**：**SQLite** `~/.local/share/opencode/opencode.db`（Windows：`%USERPROFILE%\.local\share\opencode\`）。证据：官方 `opencode db path` + 仓库 issue #13636/#13654。
- 遗留 JSON 布局可能残留（`storage/session|message|part/...`）——L5 parser 须容忍双布局；不得以已过时 troubleshooting 页为合同（上游文档漂移）。
- `auth.json`：同数据根目录。
- 只读约束：一致 read transaction；禁 checkpoint/主库单文件复制（POC-FILE-002）。

## 账号与凭据

- 工具无账号墙；模型 BYO API key / OAuth 或 OpenCode Zen。
- server 默认无认证 + `/tui` + `/auth` → 按不可信本地面设计；仅 opaque handle；不写 provider 配置。

## 平台

- 官方 README 提供 Windows（scoop/choco）、macOS、Linux 安装；性能问题 troubleshooting 建议 WSL——行为以 PoC 为准。

## 未决与 Open PoC

- 当前 `opencode.db` schema dump；WAL 只读快照；孤儿 JSON；默认无认证暴露面；SSE 事件词表。
- Open PoC：POC-SESS-001、POC-SESS-002、POC-FILE-002——状态 **not-run**。

## 产品映射

- server/API 为 L1 主路径；连接已运行 server 需 L3 条件；SQLite 仅 L5 只读；证据全 not-run。
