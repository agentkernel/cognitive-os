# Adapter Dossier — OpenAI Codex

> 类别：informative research ｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：product-only / not-implemented / evidence none
>
> 接口事实部分来自竞品实现观察（[../../sources/paseo-and-comparables-ledger.md](../../sources/paseo-and-comparables-ledger.md)）；官方一手接口须在 [../../sources/provider-interfaces-ledger.md](../../sources/provider-interfaces-ledger.md) 补齐，未补齐标 `待核验`。

## 身份

- 目标：OpenAI Codex（CLI + App Server）。
- 适用基线：官方 CLI/App Server version 待核验。
- 许可 / 条款：OpenAI 官方条款，须核验第三方 Host 启动/接管/读取的允许性。

## 官方控制接口（部分来自竞品观察，待官方核验）

- **Codex App Server**：多工具经 App Server 驱动 Codex（Happy、Vibe、Open WebUI Computer 观察到）。
- **Codex thread resume/fork**：`codex resume <thread-id>`、`thread/fork`（Vibe、amux、tmux-agent-tools、AoE 观察到）。
- session/thread ID 由 provider 返回，工具保存后下轮回传。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标 | App Server |
| L2 Host-launched | 目标 | Host 启动 Codex 会话 |
| L3 session 采用 | 条件 | 官方 thread resume；写采用需单 writer 证明 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 待核验 | native 存储格式/位置待核验 |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- resume/fork：官方 thread ID（新进程）。
- fork 产生新 thread ID 时须显示新旧映射，不得显示为“继续原 session”。
- native 文件格式/位置：待核验。

## 账号与凭据

- 登录：Codex 原生登录；凭据由原 CLI 管理（`auth.json` 等，位置待核验）。
- 规则：不复制/刷新/回写凭据（Paseo Codex fetcher 反例）；仅 opaque handle。

## 平台

- Windows/macOS/Linux 待核验；Windows 终端后端须固定 ConPTY。

## 未决与 Open PoC

- 官方 App Server/CLI 版本与接口合同；thread fork 语义；native 文件格式；凭据文件位置与只读边界。
- Open PoC：POC-SESS-001、POC-SESS-002——状态 not-run。

## 产品映射

- App Server 为 L1 主路径；thread resume 为 L3 条件；native 写归 L6 阻断。
