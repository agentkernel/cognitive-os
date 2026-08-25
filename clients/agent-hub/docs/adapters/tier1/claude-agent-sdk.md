# Adapter Dossier — Anthropic Claude Agent SDK

> 类别：informative research ｜ 日期：2026-07-20（2026-07-21 AH-CTR-02 文档级回填）｜ owner：Lane-CON
>
> 状态用语：**接口已核验（文档级）** / product-only / not-implemented / **evidence not-run**。

## 身份

- 目标：Anthropic Claude Agent SDK（及配套 Claude Code CLI 生态）。
- 官方文档：https://code.claude.com/docs/en/agent-sdk 、/agent-sdk/sessions （查询日 2026-07-20）。
- 适用基线：TS **`@anthropic-ai/claude-agent-sdk` 0.3.215**（`claudeCodeVersion` 2.1.215）；Python **`claude-agent-sdk` 0.2.123**（2026-07-19）。
- 维护状态：活跃（近乎每日发版；npm 捆绑平台原生二进制）。
- 许可：**TS SDK 受 Anthropic Commercial ToS**（README License；非 OSS）；**Python wrapper MIT**（CLI/服务仍受 Commercial ToS）。

## 官方控制接口（一手）

- **`query()`**（TS/Python）：options 含 `allowedTools`、`mcpServers`、`hooks`、`resume`、`forkSession`、`persistSession: false`（仅 TS 内存态）。
- **官方 session API**（L5/L3 首选合同面，优于裸扫文件）：`listSessions`、`getSessionMessages`、`getSessionInfo`、`renameSession`、`tagSession`、`forkSession`。
- **`SessionStore` adapter**：跨主机镜像 transcript（官方推荐）。
- **Hooks**：PreToolUse / PostToolUse / SessionStart / SessionEnd / UserPromptSubmit 等。
- **漂移样本**：experimental V2 `createSession` API 已于 TS **0.3.142 移除**。
- CLI 备选：`claude -p --output-format json`。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标 | Agent SDK（文档级） |
| L2 Host-launched | 目标 | Host 启动 SDK 会话 |
| L3 session 采用 | 条件 | 仅旧 writer inactive 或 exclusive lease；外部 `claude --resume` 同写为高风险 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 只读 | JSONL opt-in；优先官方 session API；敏感裁剪 |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | 写 native JSONL 属 L6，v1 阻断 |

## session / 文件事实

- 路径（官方）：`~/.claude/projects/<encoded-cwd>/<session-id>.jsonl`，或 `$CLAUDE_CONFIG_DIR/projects/...`；`<encoded-cwd>` = 绝对路径中非字母数字字符替换为 `-`。
- resume 需 `cwd` 匹配。
- fork：`forkSession: true` → **新 session ID 新文件，原 session 不变**（官方）。
- JSONL 行级 schema：**无官方文档**（POC-FILE-001）。
- 双 writer fencing：官方未给承诺——维持单 writer 前置（POC-SESS-002）。

## 账号与凭据

- 登录：Claude 订阅 OAuth 或 Anthropic API key。
- 规则：不复制 credentials/Keychain secret；仅 opaque handle。
- **外部阻断**：订阅路径下第三方 Host 自动化是否属「另行明确许可」——需 Anthropic 确认（POC-LIC-002）。

## 平台

- SDK 捆绑多平台二进制；Windows/macOS/Linux 行为与 ConPTY 须 PoC。

## 未决与 Open PoC

- JSONL schema；双 writer；rewind 行为；Commercial ToS 再分发/订阅自动化；V2 API 移除后的漂移监测。
- Open PoC：POC-SESS-001、POC-SESS-002、POC-FILE-001、POC-SEC-003——状态 **not-run**。

## 产品映射

- 以 Agent SDK + 官方 session API 为 L1/L5 主路径；JSONL 裸扫为后备；native 写归 L6；证据全 not-run。
