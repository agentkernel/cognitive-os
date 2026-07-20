# Adapter Dossier — Anthropic Claude Agent SDK

> 类别：informative research ｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：product-only / not-implemented / evidence none
>
> 本 dossier 的接口事实部分来自竞品实现观察（[../../sources/paseo-and-comparables-ledger.md](../../sources/paseo-and-comparables-ledger.md)），Claude 官方 SDK/CLI 的一手接口合同须在 [../../sources/provider-interfaces-ledger.md](../../sources/provider-interfaces-ledger.md) 用查询日/version 补齐，未补齐项标 `待核验`。

## 身份

- 目标：Anthropic Claude Agent SDK（及配套 Claude Code CLI 生态）。
- 适用基线：官方 SDK/CLI version 待核验（provider ledger 填充）。
- 许可 / 条款：Anthropic 官方条款，须核验第三方 Host 启动/接管/读取 session 的允许性。

## 官方控制接口（部分来自竞品观察，待官方核验）

- 多个成熟工具经 **Claude Agent SDK** 主路径驱动 Claude（Happy、Nimbalyst、Open WebUI Computer 观察到）。
- Claude Code CLI 提供 `--resume <session-id>` 恢复（Opcode、AoE、tmux-agent-tools 观察到）。
- native session 存储为 **JSONL**（`~/.claude/projects` 等；多工具只读扫描）。
- fork/rewind 会写新的 native JSONL（Happy 观察到）——属修改 provider session 存储，需谨慎。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标 | Agent SDK |
| L2 Host-launched | 目标 | Host 启动 SDK 会话 |
| L3 session 采用 | 条件 | 仅旧 writer inactive 或有 exclusive lease；双 writer（外部 `claude --resume` 同写同 session）为高风险反例 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 只读 | JSONL opt-in、documented root、敏感裁剪 |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | 写 native JSONL 属 L6，v1 阻断 |

## session / 文件事实

- 格式：JSONL（待核验具体 schema/版本）。
- resume：官方 session ID 恢复（新进程）。
- 双 writer 风险：外部并行 `claude --resume` 与 SDK 同时写同一 session（Happy 反例）——采用前必须证明单 writer。

## 账号与凭据

- 登录：Claude 原生登录（OAuth/token），凭据由原 CLI 管理。
- 规则：不复制 `.credentials.json`/Keychain secret（Paseo quota fetcher、Agent Deck 反例）；仅 opaque handle。

## 平台

- Windows/macOS/Linux 待核验；Windows 终端后端须固定为 ConPTY（多工具未给公开 ConPTY 合同）。

## 未决与 Open PoC

- 官方 SDK/CLI 版本与接口合同；fork/rewind 对 native JSONL 的确切写行为；双 writer fencing。
- Open PoC：POC-SESS-001、POC-SESS-002、POC-FILE-001——状态 not-run。

## 产品映射

- 以 Agent SDK 为 L1 主路径；JSONL 仅 L5 只读；native 写归 L6 阻断。
