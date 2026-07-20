# 模板 — Adapter Dossier

> 复制本模板到 `../adapters/tier1/<agent>.md` 或相应 Tier 目录填充。所有事实必须附来源（标题 + 完整 URL + 查询日 + release/version/commit）。当前证据一律 `none / not-run`。

## 身份

- Agent 名称 / 官方仓库或产品页：
- 组织 / 维护状态（活跃 / 停滞 / 归档 / sunset）：
- 适用基线：main commit ｜ 最新稳定 release ｜ 查询日：
- 许可证（含 source-available / 附加条款）：

## 官方控制接口

- SDK / App Server / Gateway / ACP / REST / SSE / JSON-RPC / headless / 官方 session API（逐项 + 来源）：
- 稳定性（稳定 / 实验 / 未在 release 升格）：

## 接管层级适用性

| Level | 是否适用 | 条件 / 限制 | 来源 |
|---|---|---|---|
| L1 官方控制 | | | |
| L2 Host-launched | | | |
| L3 官方 session 采用 | | 是否提供 exclusive lease/fencing | |
| L4 受管终端 | | | |
| L5 只读文件 | | documented root / 格式 / 敏感字段 | |
| L7 observe-only | | | |
| L6 / L8 | 阻断 / 禁止 | | |

## session / 文件事实

- native session 存储格式（JSONL/SQLite/其他）与 documented root：
- resume / fork / list 语义与跨版本漂移风险：
- 双 writer 风险与官方并发保证：

## 账号与凭据

- 登录机制（OAuth/API key/CLI profile）：
- 凭据存储位置与是否可 opaque handle 化：
- 多账号支持：

## 平台

- Windows / macOS / Linux 支持与终端后端（ConPTY/pty）：
- 版本兼容范围：

## 条款 / 许可影响

- 第三方 Host 启动/接管/读取 session 的允许性（来源）：
- 复用其代码的许可义务：

## 未决与 Open PoC

- 未决事实：
- Open PoC（`CONSOLE-AGENTHUB-V1-POC-*`，状态 not-run）：

## 状态

- Contract / Implementation / Evidence：product-only / not-implemented / none
