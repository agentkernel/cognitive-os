# Source Ledger — Paseo 与竞品

> 类别：informative research source ledger ｜ 查询日：2026-07-20 ｜ owner：Lane-CON
>
> 每条记录含标题/URL/查询日/基线与事实-推论-未决-产品影响分离。事实来自官方仓库/文档核验；推论明确标注。竞品仅作行为参考与反例，本产品不复用其代码；许可复用须过法务 gate（[../security/licensing-and-terms.md](../security/licensing-and-terms.md)）。

## 0. 通用结论

核验的全部项目中**未发现**支持任意 PID 内存注入或抢占普通进程 stdin 的实现。可靠进程控制层级依次为：daemon/server-owned child > official provider session resume（启动新进程传 session/thread ID）> native file observation（只读候选）> terminal multiplexer attach（tmux/screen，仅字节通道）。

## 1. Paseo（对照基线）

- 基线：`getpaseo/paseo` main `3d86c738ff70a9815cdd86c5602c9a5c420df619`（2026-07-19）；稳定版 v0.1.110（2026-07-16）；最新预发布 v0.2.0-beta.1（2026-07-17）。
- 来源：`https://github.com/getpaseo/paseo`、`https://paseo.sh/docs/{providers,cli,security}`、release/源码页（查询日 2026-07-20）。

事实：

- 本地 daemon 启动并监管已安装 Agent CLI（原生适配器或 ACP/JSON-RPC stdio），不附着任意外部 PID；`attach` 是 timeline 流，非终端 stdin。
- provider session import 创建新 Paseo Agent 再调用 provider resume/load。
- 默认监听 `127.0.0.1:6767`、默认无密码；能到达 socket 即可控制 daemon（loopback 非身份认证）。
- 普通 direct/Relay client 被建立为 `kind:"trusted"`、`scopes:["*"]`；`clientId` 来自客户端 hello，普通路径未用设备签名证明。
- 文件预览权限=daemon 可读的任意 regular file；客户端可把 root 扩展到 `/`/盘符根/UNC root；文件服务做 lexical + realpath + symlink 检查，POSIX 最终 open 加 `O_NOFOLLOW`。
- Relay：客户端每连接生成 Curve25519 ephemeral key，`nacl.box` + XSalsa20-Poly1305；trust anchor 是 pairing offer 中的 daemon public key，offer 无 expiry/single-use/device 字段；live session 无 nonce cache/replay window。
- daemon key/`serverId` 持久化，普通 restart 复用（不自然轮换）；无 per-device identity/单设备 revoke/offer expiry/受支持 daemon-key rotation。
- SECURITY 称不管理 provider auth，但 quota fetcher 会读取/刷新/回写 Claude `.credentials.json`、Codex `auth.json`、Kimi、GitHub CLI `hosts.yml`、Cursor SQLite 等；custom provider 可在 `config.json` 明文放 API key。
- 许可：AGPL-3.0-or-later（package metadata），第三方组件服从原许可；§13 网络交互须提供 Corresponding Source。

推论：Paseo 是“daemon-owned child + official session resume”，非 terminal multiplexer 或 PID takeover；普通 client 是等权 trusted operator，不是最小权限多角色控制面。

未决：Relay live-session replay 窗口、逐设备撤销、长期 pairing capability 的完整安全证明；Windows provider 终端是否稳定依赖 ConPTY 无公开合同；无独立密码学审计。

产品影响：可 clean-room 借鉴 daemon/provider/worktree/E2EE companion、QR host-key trust anchor、Host allowlist + CORS/Origin、密码 hash；**不可复制**等权 trusted-client、loopback=authority、客户端自选 root、URL/env/AsyncStorage 明文密码、无 fencing 的 import、无 replay 防护、未经授权读写 provider 凭据；复用任何代码前须过 AGPL 法务 gate。

## 2. 竞品逐项（参考/反例）

以下每项均查询日 2026-07-20，基线 commit/release 见括号。仅记录对本产品设计有影响的事实与产品影响。

### Happy（`slopus/happy` main `3f161de…`，cli-1.1.10；MIT）

- 事实：wrapper/daemon 启动 Agent；Claude 走 Agent SDK、Codex 走 App Server；session scanner 只读监视 Claude JSONL；fork/rewind 写新 native JSONL；main 有 worktree 代码但文档称不管理（漂移）；Relay E2EE（NaCl secretbox/AES-GCM）；`happy-agent` token 存 `0600` JSON 非 keychain。
- 产品影响：借鉴 E2EE、离线消息、SDK/App Server 双适配；native JSONL fork + 外部终端同写=高风险双 writer 反例。

### Vibe Kanban（`BloopAI/vibe-kanban`；Apache-2.0；已 sunset 转社区）

- 事实：本地 server 创建独立子进程；`portable-pty`；Claude CLI 控制协议 + Codex App Server + 通用 ACP；读取 `~/.claude/projects` JSONL；MCP API 直写 Claude/Codex/Gemini/OpenCode 配置；每 workspace 独立 worktree；Relay 是 Ed25519 签名 + nonce（完整性，非 E2EE）；2026-04-24 公司关闭。
- 产品影响：借鉴 worktree/多 session UX；供应商配置直写、同 workspace 多 writer、把签名 Relay 称 E2EE 均反例；依赖按 sunset 评估。

### Agent Deck（`asheshgoplani/agent-deck`，v1.10.10；MIT）

- 事实：TUI 创建 `agentdeck_*` tmux session（可选独立 socket）；`switch-account` 复制原生 conversation file 到另一 `CLAUDE_CONFIG_DIR` 并预写 folder trust 再 `--resume`；Docker 模式复制 host auth，macOS 从 Keychain 提取 OAuth token 写 `.credentials.json`；远程靠 SSH/Telegram/Slack，无自有 E2EE。
- 产品影响：借鉴 socket 隔离/worktree/远程；账号切换复制会话、Keychain secret 落盘=禁止路径（对应 [CONSOLE-AGENTHUB-V1-DEC-021](../decisions/decision-log.md)）。

### Agent of Empires（`agent-of-empires/agent-of-empires`，v1.13.0；MIT）

- 事实：双路径——tmux terminal view + `aoe serve` daemon 的 ACP 子进程；可扫描 `$CLAUDE_CONFIG_DIR/projects` 并 `claude-agent-acp session/load` 恢复（不复制原文件）；import 不要求旧 runtime 停止（双 writer 风险）；装 status hooks 到 provider 配置；sandbox 复制凭据（也有 host hook 短期凭据模式）；PWA 经 Tailscale/Cloudflare Tunnel；原生 Windows 仅 WSL2。
- 产品影响：借鉴 structured/terminal 双视图、设备撤销、DNS rebinding 防护；hooks 写配置、共享凭据目录、import 无 writer 存活检查需更严格收口。

### Claude Squad（`smtg-ai/claude-squad`，v1.0.19；AGPL-3.0）

- 事实：无 daemon，创建 `claudesquad_*` tmux + PTY attach；pause/resume 实为提交+worktree 重建；不扫描/写 provider 配置；每 instance 独立 worktree；无 Web/mobile/Relay；凭据由启动环境继承；`--autoyes` 风险。
- 产品影响：借鉴最小 tmux/worktree；pane parsing/AutoYes/UI pause 标签不可升级为可信 lifecycle。

### amux（`mixpeek/amux`，v0.9.108；MIT + Commons Clause=source-available）

- 事实：Agent 在 `amux-*` tmux；可迁移/attach 既有匹配 tmux session；状态来自 `capture-pane`，YOLO 默认开；dashboard 默认无认证（创建 `auth_token` 才启用）；secrets 明文 `~/.amux/server.env`；README“automatic worktree/MIT”与源码/LICENSE 冲突（漂移）。
- 产品影响：借鉴自愈/移动 fleet/原子 board claim；默认无认证、YOLO、明文 secret、parser 驱动控制=反例；Commons Clause 非无限制开源。

### tmux-agent-tools（`ohyeh/tmux-agent-tools`，v0.35.0；无 LICENSE）

- 事实：shell wrapper 创建前缀 tmux；`resume` 走 `claude --resume`/`codex resume`；默认 `--dangerously-skip-permissions`/`--yolo`；`--secret` 从 1Password/pass/文件注入 tmux 环境并脱敏 capture；无 LICENSE（无可依赖复用授权）。
- 产品影响：借鉴 nonce 等待、前缀清理、hash-chain audit；默认 bypass 权限、无许可证=不可接受。

### Nimbalyst（`nimbalyst/nimbalyst`，v0.68.1；client MIT，sync server 许可未公开）

- 事实：Electron 拥有子进程；Claude 走 CLI+node-pty 或 SDK，Codex App Server/ACP；扫描 `~/.claude/projects` 并复制到 PGLite；直写 `.claude/settings*.json`；内建 worktree；移动 AES-256-GCM E2EE、QR seed，结构元数据不加密；sync seed 优先 safeStorage，不可用时明文 fallback；`wss://sync.nimbalyst.com` 单独项目许可未找到。
- 产品影响：借鉴移动 diff/工作区/E2EE；供应商 settings 直写、全量 transcript 复制、明文 fallback 需更严格；sync server 复用前须核验许可。

### Omnara（旧 `omnara-ai/omnara` 归档，Apache-2.0；当前产品闭源）

- 事实：旧版 `pty.fork` wrapper 启动 Claude/Codex，读 Claude JSONL 发到云 API，无 E2EE，JWT 无过期；因 Claude Code 频繁变更不可维护而归档；当前 remote/control-plane 产品无公开源码/许可。
- 产品影响：印证长期维护 CLI wrapper 困难、应迁 SDK；当前产品在开源/凭据/云迁移边界透明前仅作 UX 参考。

### Opcode/Claudia（`winfunc/opcode`，v0.2.0；AGPL-3.0）

- 事实：Tauri 扫描 `~/.claude/projects` 列历史，resume 启动新 `claude --resume`；stdout/stderr pipes 非 tmux/PTY/ACP；cancel 是进程终止；默认 `--dangerously-skip-permissions`；MCP 写 `.mcp.json`；Web server 绑 `0.0.0.0`、CORS Any、无认证/TLS、cancel 空实现；自 2025-10 无 main 提交（停滞）。
- 产品影响：借鉴 session 浏览/checkpoint UX；默认 bypass 权限、单进程状态、无认证 Web server=严重反例。

### OpenHands（见 [../adapters/tier1/openhands.md](../adapters/tier1/openhands.md)）

- 事实：Agent Server（HTTP/WS）拥有平台自有 conversation；ACP 子进程；真 `pause()`；不扫描/导入 provider native session；ACP 可用本机登录/Keychain 或注入 secret registry；ACP permission 默认自动批准；Canvas/SDK MIT，旧 enterprise 目录 PolyForm。
- 产品影响：Tier 1；借鉴真 pause/Agent Server 分层/secret-at-rest；自动批准、默认无 API auth 不可直接采用。

### Open WebUI（Core `open-webui/open-webui`；Computer `open-webui/computer`；Open Terminal MIT）

- 事实：Core 是聊天平台（非同类）；Computer 是 coding-agent harness：spawn Claude Agent SDK/Codex App Server，OpenCode 可 spawn 或连接已运行 server；服务端拥有 terminal（Unix pty/fork、Windows pywinpty）；显式 worktree API；认证后用户等同 SSH 全主机访问、无多用户隔离；Core=Open WebUI License（>50 用户限制），Computer=基于 Elastic License 2.0 的 Open Use License（source-available）。
- 产品影响：借鉴 provider profile/worktree/移动 workstation；全主机同权访问=反例；许可非无限制开源。

## 3. 模式分类（供设计引用）

1. terminal multiplexer attach：Agent Deck、AoE terminal、Claude Squad、amux、tmux-agent-tools。
2. daemon/server-owned child：Paseo、Happy、Vibe、AoE ACP、Nimbalyst、旧 Omnara、Opcode、OpenHands、Open WebUI Computer。
3. official provider session resume：Paseo、Happy、AoE、Nimbalyst、Opcode、tmux-agent-tools、Open WebUI Computer（均启动新进程传 session/thread ID）。
4. native file observation/import：Happy、Vibe、Agent Deck、AoE、Nimbalyst、Opcode（部分复制/修改，风险更高）。
5. platform-owned conversation：OpenHands、Open WebUI Core、部分新 Omnara。
6. remote/mobile：E2EE 已核实=Paseo/Happy/Nimbalyst；仅 TLS/VPN/tunnel=AoE/amux/OpenHands/Open WebUI；云端明文/未声明=旧 Omnara/Opcode Web。
7. arbitrary PID injection：全部为“无”。
