# Agent Hub — 逐能力矩阵

> 类别：informative research/design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本矩阵是**设计目标**视图，不是已核验事实或已实现能力。单元格语义：`目标`=首发设计目标待逐 Agent version/PoC 核验；`条件`=仅在满足安全条件时；`只读`=仅观察；`阻断`=v1 policy 阻断；`禁止`=永久禁止；`待核验`=接口事实尚未核验。逐 Agent 事实见各 [dossier](./tier1/)，接口层见 [interface-layering.md](./interface-layering.md)。

## 能力清单（33 项）

发现类：D1 安装发现 / D2 运行进程发现 / D3 native session 发现 / D4 受管终端发现 / D5 外部 PID metadata。
控制类：C1 Host-launch(L2) / C2 官方 session 只读采用(L3) / C3 官方 session 写采用(L3) / C4 受管终端 attach(L4) / C5 send input / C6 interrupt/cancel turn / C7 stop session / C8 force kill / kill tree。
会话/文件：S1 session list / S2 session resume / S3 session fork / S4 native 文件读 JSONL / S5 native 文件读 SQLite / S6 provider 文件写(L6)。
账号/凭据：A1 检测登录 / A2 官方登录连接 / A3 多账号 profile / A4 opaque credential handle。
监督：M1 创建 WorkItem / M2 监督纠偏 / M3 pause / M4 cancel / M5 resume。
协作：G1 Lead+Workers(单层) / G2 worktree 隔离。
远程：R1 手机监督 / R2 手机请求扩权。
完成：V1 checks + user acceptance 双轴。

## 矩阵

| 能力 | Codex | OpenCode | Claude Agent SDK | Hermes | OpenClaw | OpenHands |
|---|---|---|---|---|---|---|
| D1 安装发现 | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| D2 进程发现 | 只读 | 只读 | 只读 | 只读 | 只读 | 只读 |
| D3 session 发现 | 目标 | 目标 | 目标 | 待核验 | 待核验 | 条件（平台自有会话） |
| D4 终端发现 | 条件 | 条件 | 条件 | 条件 | 条件 | 条件 |
| D5 外部 PID metadata | 只读 | 只读 | 只读 | 只读 | 只读 | 只读 |
| C1 Host-launch(L2) | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| C2 session 只读采用 | 目标 | 目标 | 目标 | 待核验 | 待核验 | 条件 |
| C3 session 写采用 | 条件 | 条件 | 条件 | 待核验 | 待核验 | 条件 |
| C4 终端 attach(L4) | 条件 | 条件 | 条件 | 条件 | 条件 | 条件 |
| C5 send input | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| C6 interrupt/cancel | 目标 | 待核验 | 目标 | 待核验 | 待核验 | 目标 |
| C7 stop session | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标（真 pause/resume） |
| C8 force kill/tree | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| S1 session list | 目标 | 目标 | 目标 | 待核验 | 待核验 | 条件（自有 conversation） |
| S2 session resume | 目标 | 目标 | 目标 | 待核验 | 待核验 | 条件 |
| S3 session fork | 待核验 | 待核验 | 条件 | 待核验 | 待核验 | 待核验 |
| S4 文件读 JSONL | 只读 | 待核验 | 只读 | 待核验 | 待核验 | 未使用（不扫描 native） |
| S5 文件读 SQLite | 待核验 | 只读 | 待核验 | 待核验 | 待核验 | 待核验 |
| S6 provider 文件写(L6) | 阻断 | 阻断 | 阻断 | 阻断 | 阻断 | 阻断 |
| A1 检测登录 | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| A2 官方登录连接 | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| A3 多账号 profile | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| A4 opaque cred handle | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| M1 创建 WorkItem | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| M2 监督纠偏 | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| M3 pause | 待核验 | 待核验 | 待核验 | 待核验 | 待核验 | 目标（官方 pause） |
| M4 cancel | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| M5 resume | 目标 | 目标 | 目标 | 待核验 | 待核验 | 目标 |
| G1 Lead+Workers | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| G2 worktree 隔离 | 目标 | 目标 | 目标 | 目标 | 目标 | 条件（可挂载既有 worktree） |
| R1 手机监督 | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| R2 手机请求扩权 | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |
| V1 完成双轴 | 目标 | 目标 | 目标 | 目标 | 目标 | 目标 |

## 读法规则

- 全表 Implementation=`not-implemented`、Evidence=`none / not-run`。
- `目标`/`条件` 都需对应 Adapter 的 version/account/license/PoC gate 通过后才可声明支持。
- OpenHands 的 D3/S1/S2 标 `条件`，因为其恢复的是**平台自有 conversation**，不是第三方 provider native session takeover（来源见 [../sources/paseo-and-comparables-ledger.md](../sources/paseo-and-comparables-ledger.md)）。
- Hermes、OpenClaw 大量 `待核验`：本轮未取得其官方接口一手事实，禁止臆造；须在各自 dossier 用查询日/version/commit 补齐。
