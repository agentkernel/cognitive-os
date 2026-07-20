# Adapter Dossier — OpenCode

> 类别：informative research ｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：product-only / not-implemented / evidence none
>
> 接口事实部分来自竞品实现观察（[../../sources/paseo-and-comparables-ledger.md](../../sources/paseo-and-comparables-ledger.md)）；官方一手接口须在 [../../sources/provider-interfaces-ledger.md](../../sources/provider-interfaces-ledger.md) 补齐，未补齐标 `待核验`。

## 身份

- 目标：OpenCode（开源 coding agent，含 server/session 能力）。
- 适用基线：官方 version/commit 待核验。
- 许可：开源许可待核验（在 terms-and-licenses ledger 登记具体 SPDX）。

## 官方控制接口（部分来自竞品观察，待官方核验）

- **OpenCode server**：可 spawn server 或连接用户已运行的 OpenCode server（Open WebUI Computer 观察到）。
- session list/管理 API（Paseo/Vibe 生态观察到）。
- 部分工具经通用 **ACP** 与 OpenCode 交互（待核验具体版本）。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标 | server/API / ACP |
| L2 Host-launched | 目标 | Host spawn server |
| L3 session 采用 | 条件 | 连接已运行 server 属条件采用；写需单 writer/lease 证明 |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 只读 | native 存储疑为 SQLite（待核验格式/位置） |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- native 存储：疑为 SQLite（待核验）；若 SQLite 则只读一致 read transaction、禁 checkpoint/主库单文件复制。
- 连接已运行 server：属 L3 条件采用，须确认单 writer 或官方并发语义。

## 账号与凭据

- 登录/凭据：待核验；仅 opaque handle；不写 provider 配置（Vibe MCP 直写配置为反例）。

## 平台

- Windows/macOS/Linux 待核验；终端后端待核验。

## 未决与 Open PoC

- 官方 server/session API 与版本；native 存储格式/位置；连接已运行 server 的并发/writer 语义。
- Open PoC：POC-SESS-001、POC-FILE-002——状态 not-run。

## 产品映射

- server/API 为 L1 主路径；连接已运行 server 需 L3 条件与单 writer 证明；SQLite 仅 L5 只读。
