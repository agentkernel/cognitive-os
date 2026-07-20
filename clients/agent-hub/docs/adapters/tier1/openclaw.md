# Adapter Dossier — OpenClaw

> 类别：informative research ｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：product-only / not-implemented / evidence none
>
> **警告：本轮未取得 OpenClaw 的官方接口一手事实。** 下列条目为待核验占位，须在 [../../sources/provider-interfaces-ledger.md](../../sources/provider-interfaces-ledger.md) 用官方仓库/文档 + 查询日 + version/commit 补齐后方可声明任何支持。禁止臆造接口、session 格式、条款或许可。

## 身份

- 目标：OpenClaw。
- 官方仓库 / 产品页：待核验。
- 适用基线（main commit / release / 查询日）：待核验。
- 维护状态：待核验。
- 许可 / 条款：待核验（若为 source-available/附加条款，不得当作无限制开源）。

## 官方控制接口

- SDK / server / CLI / ACP / session API：待核验。
- 稳定性：待核验。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 待核验 | |
| L2 Host-launched | 待核验 | 若无稳定接口，先按 launch-only 评估 |
| L3 session 采用 | 待核验 | |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 待核验 | |
| L7 observe-only | 是 | 至少可只读观察 |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件 / 账号 / 凭据 / 平台

- 全部 `待核验`；opaque credential handle 与不云同步为默认约束。

## 未决与 Open PoC

- 官方仓库与接口存在性、稳定性、条款允许性、session 格式、平台支持——全部待核验。
- gate：OpenClaw Adapter 在接口事实核验前保持 `blocked`；不得以占位内容启动实现。

## 产品映射

- 在接口核验前，OpenClaw 至多按 launch-only / observe-only 处理，不承诺高层级接管。
