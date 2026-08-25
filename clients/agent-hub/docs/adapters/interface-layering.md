# Agent Hub — 接口分层

> 类别：informative research/design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文把各类 Agent 互操作接口按“控制强度/信任语义”分层，并明确它们与 CognitiveOS authority 的关系。协议版本事实见 [../sources/provider-interfaces-ledger.md](../sources/provider-interfaces-ledger.md)。

## 1. 分层（从强控制到弱观察）

| 层 | 接口 | 控制语义 | 信任 | 接管层级 |
|---|---|---|---|---|
| A | 官方 SDK / App Server / Gateway / REST / SSE / JSON-RPC / headless | 结构化控制（send/cancel/interrupt/permission/资源） | 官方通道，结构化事实 | L1 |
| B | 官方 session API（list/import/resume/fork） | 会话生命周期 | provider-reported | L3 |
| C | Host-launched 进程 + 官方 stdio/SDK | Host 从启动持有 | host-managed | L2 |
| D | 受管终端（Host-owned ConPTY / 独立 socket tmux） | 字节级输入/capture | terminal-observed（不可信文本） | L4 |
| E | native session 文件只读观察 | 只读 snapshot | file-observed | L5 |
| F | OS 进程 API（观察/信号） | 进程存在/健康/紧急终止 | process-observed | L7 |
| G | UI automation（selected-window） | 窗口级输入/截屏 | 高风险，受限 | 见 computer-control |
| — | 任意 PID 注入 / stdin 抢占 / 内存注入 | — | 禁止 | L8 |

## 2. Agent 互操作协议定位

- **MCP（Model Context Protocol）**：工具/资源/提示的模型上下文协议，属于 Agent 与工具间接口，不是 Host 对 Agent 的接管通道；当前稳定版本固定为 **2025-11-25**。查询日 2026-07-20 另存在计划中的 RC（尚未生效），仅登记不采用；不得把未生效 RC 当作已生效合同。
- **ACP（Agent Client Protocol）**：客户端↔Agent 的 JSON-RPC over stdio 会话协议（OpenHands、AoE、Vibe 等使用），可映射到 L1/L2，取决于是否 Host 启动子进程。
- **A2A（Agent-to-Agent）**：Agent 间协作协议；v1 不作为接管通道，多 Agent 协作用 Host 内确定性调度器（见 [../collaboration/lead-workers.md](../collaboration/lead-workers.md)），不启用自治 A2A。
- **Vendor API（provider 专有）**：各 Agent 官方控制/ session API，属 A/B 层。

## 3. 与 CognitiveOS authority 的关系

- 以上所有层在 **Direct** 模式都不是 authority：最多产生 `provider-reported` / `host-managed` / `*-observed` 事实标签。
- 在 **Governed** 模式，这些接口作为受治理 Adapter 的输入，但 Task/Effect/Verification/Acceptance 仍由确定性 authority 推进（`REQ-AGENT-*` 等已登记合同），Adapter 不绕过确定性入口。
- 任何层的输出（包括 SDK 的 structured done、session 的 completed、终端文本、文件变化）都不能自动成为 Verification/Acceptance/`COMPLETED`。

## 4. 版本纪律

- 每个协议/接口记录查询日与版本；跨版本行为漂移进入对应 Adapter dossier 的“未决/Open PoC”。
- MCP 稳定版 = 2025-11-25；未生效 RC 只记录不采用。
- 不把 informative 白皮书或营销描述当作接口合同。
