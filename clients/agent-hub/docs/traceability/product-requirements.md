# Agent Hub — 产品要求追踪

> 类别：informative traceability ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文登记 `CONSOLE-AGENTHUB-V1-PRD-*` 产品要求，并按三维状态（Contract / Implementation / Evidence）与 blocked_by 追踪。这些是 informative 产品要求，不进入 CognitiveOS normative registry；引用真实 `REQ-*` 处均标注其存在于 `specs/registry/requirements.yaml`。

## 状态图例

- Contract：`product-only`（仅产品文档）/ `partial`（部分对应已登记 `REQ-*`）/ `registered`。
- Implementation：`not-implemented` / `partial` / `available`。
- Evidence：`none` / `not-run` / `pass` / `fail`。

当前所有条目 Implementation=`not-implemented`、Evidence=`none`。

## 产品要求

| PRD | 要求 | Contract | blocked_by |
|---|---|---|---|
| PRD-001 | 每个可写页面持续显示 mode/Host/账号/workspace/事实来源/freshness | product-only | Console 实现 gate |
| PRD-002 | 两部署模式保证不可视觉混同 | product-only | Console 实现 gate |
| PRD-003 | 接管层级 L1–L8 与七结果标签如实呈现 | product-only | Host/Adapter |
| PRD-004 | 默认 Host-launched（L2） | product-only | Process Supervisor |
| PRD-005 | 官方 session 采用仅条件可写（L3） | product-only（`partial`: `REQ-AGENT-*` 仅 Governed） | Session Adopter + exclusive-lease 证明 |
| PRD-006 | L4 仅 Host-owned 终端 | product-only | Terminal Broker |
| PRD-007 | L5 opt-in 只读文件观察 | product-only | File Observer |
| PRD-008 | L6 v1 阻断 | product-only | 供应商写协议 + 法务 |
| PRD-009 | 外部 PID observe-only + 独立 emergency containment | product-only | emergency containment 设计 |
| PRD-010 | ownership generation + single controller lease | product-only | Host/Control/Ledger |
| PRD-011 | 本机控制面 OS peer 校验，不依赖 loopback | product-only | Host/Control |
| PRD-012 | per-user 非提权 Host | product-only | Host/Control |
| PRD-013 | 停止语义分层建模（interrupt/cancel/TERM/KILL/kill-tree/detach/release） | product-only | Process Supervisor |
| PRD-014 | 文件观察一致 snapshot + 版本化 parser + 敏感裁剪 | product-only | File Observer |
| PRD-015 | credential 仅 opaque handle + OS secure store + 不云同步 | product-only | Credential Broker |
| PRD-016 | 多账号切换默认只影响新 session；替代 handoff | product-only | Credential Broker |
| PRD-017 | Managed E2EE Relay + LAN/VPN；两端 matching code + PC-local approve | product-only | Relay/Pairing |
| PRD-018 | 手机只能请求；高后果动作 PC-local 确认 | product-only | Relay/Pairing + Host/Control |
| PRD-019 | Relay 幂等/anti-replay；push 仅 opaque hint | product-only | Relay/Pairing |
| PRD-020 | 单 Host 单层 Lead+Workers；Lead 仅 proposal | product-only | Multi-Agent 调度器 |
| PRD-021 | coding WorkItem 默认独立 worktree | product-only | Workspace Manager |
| PRD-022 | 桌面控制仅 selected-window；隔离浏览器 | product-only | Computer Control |
| PRD-023 | 完成判定双轴（checks + user acceptance）；完成语言分开 | product-only | Verifier |
| PRD-024 | Direct→Governed 仅 evidence-only 迁移 | product-only | Governed authority（M2/M4/M5） |
| PRD-025 | WCAG 2.2 AA + 各平台原生辅助技术关键旅程可完成 | product-only | 各平台实现 + 审计 |
| PRD-026 | Governed 模式第三方 Agent 受治理 Adapter，不绕过确定性入口 | partial（`REQ-AGENT-*`） | M6 |
| PRD-027 | Direct 对象术语与 CognitiveOS 机器术语严格分离 | product-only | 产品治理 |
| PRD-028 | 不可信内容隔离渲染，不共享系统控件安全边界 | product-only | Console 前端 |

## 与真实 REQ 的关系

- Governed 模式相关行为最终由 `specs/registry/requirements.yaml` 中已登记的 `REQ-*`（如 Task/Effect/Verification/Acceptance/Agent 兼容族）承载；本表 `partial` 项标注该依赖，但不复制或改写 registry。
- Direct 模式绝大多数为 `product-only`：无对应机器合同，且刻意不进入 registry（v0.1 前规范表面冻结，IMP-01）。
- 任何 PRD 达到实现/测试阶段前，必须先在契约流程中确认是否需要新增机器合同，且遵守规范冻结纪律。

## 证据

所有 PRD 的证据索引见 [evidence-index.md](./evidence-index.md)，当前全部 `none / not-run`。
