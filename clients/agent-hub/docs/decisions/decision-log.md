# Agent Hub — 决策日志

> 类别：informative product decisions ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文登记 `CONSOLE-AGENTHUB-V1-DEC-*` 已接受产品决策。决策变化时在此追加 `superseded` 记录（保留原 ID 与原文），并更新受影响专题文档。这些决策不进入 CognitiveOS normative registry。

## 决策清单

### `CONSOLE-AGENTHUB-V1-DEC-001` 首要 persona = Agent 操作者 / 高级终端用户

沿用 Console v2 [CONSOLE-V2-DEC-001]；Agent Hub 不引入新首要 persona。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-002` 仅两种部署模式，无中间模式

只有 Direct Takeover 与 CognitiveOS Governed；禁止只有 `cognitive-kernel` 的中间模式；连接状态不构成第三模式。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-003` 平台优先级：Direct→Governed、Windows 首发、iPhone 先于 Android

移动为 remote companion。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-004` Windows GA 基线动态 gate

主 GA Windows 11 25H2；24H2 兼容（Home/Pro 2026-10-13 后阻断）；Windows 10 22H2 仅有效 ESU 下 Experimental。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-005` Tier 1 目标 Agent 六项

OpenAI Codex、OpenCode、Anthropic Claude Agent SDK、Hermes Agent、OpenClaw、OpenHands；各自独立 version/account/license/PoC gate。状态：accepted。备注：其余 Agent 分级见 [../adapters/README.md](../adapters/README.md)。

### `CONSOLE-AGENTHUB-V1-DEC-006` 接管层级 L1–L8 与结果标签

按 [../product/deployment-modes-and-guarantees.md](../product/deployment-modes-and-guarantees.md) §3 冻结八级与七结果标签。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-007` 默认 Host-launched（L2）

v1 默认接管路径为 Host 从任务开始启动并监管。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-008` 官方 session 采用仅条件可写（L3）

仅旧 writer inactive 或供应商提供 exclusive lease/fencing 时可写，否则只读 hydrate。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-009` L4 仅 Host-owned 终端

Windows 仅 Host 创建的 ConPTY；未来 tmux/screen 仅 Host 创建的独立私有 socket；不抢占普通既有 console。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-010` L5 原生文件仅 opt-in 只读发现

documented root、版本化 parser、一致 snapshot、敏感字段裁剪；默认关闭。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-011` L6 v1 阻断，仅未来条件能力

外部写 provider 文件在缺乏供应商版本/并发/CAS/rollback/migration 承诺前 `blocked-by-policy`。解锁条件见 [../architecture/session-and-file-adoption.md](../architecture/session-and-file-adoption.md) §3。状态：accepted。备注：本决策取代交互中一度出现的“vendor-documented-session-write 直接可用”倾向，冲突已按不扩大权限解释收敛。

### `CONSOLE-AGENTHUB-V1-DEC-012` 本机控制面不依赖 loopback 可达性

采用 named pipe / Unix socket + OS peer 校验；TCP 仅显式降级。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-013` per-user 非提权 Host

首发不采用 LocalSystem/Session-0 服务；不自动 UAC 提权；不支持管理员 Agent 自动接管。企业服务化为未来选项，需独立评审。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-014` 高后果动作默认 PC-local 确认

首次附着普通既有进程、扩大文件范围、observe→write、发送信号、启用桌面控制、访问新 credential、跨用户/提权、新设备配对 approve 均需本机确认。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-015` 手机只能请求扩权

手机不能直接扩权/发信号/扩大文件范围；PC-local 批准生成新 ownership generation。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-016` Managed E2EE Relay 为主 + LAN/VPN 为辅

Relay 只见密文与最小路由 metadata；不做公网明文控制面；self-host 为未来企业选项。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-017` 群组仅单 Host、一层 Lead+Workers

Lead 只产 proposal；确定性调度器执行派发。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-018` Direct→Governed 仅 evidence-only 迁移

Governed 新建 authority 对象；Host ledger 只作外部证据导入，不追认历史 authority/Verification。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-019` coding WorkItem 默认独立 worktree

每个受管 coding Agent 默认独立 Git worktree。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-020` secret 不云同步 + OS secure store + enterprise broker

credential 只存 opaque handle；ledger/日志/push 零 secret。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-021` 多账号切换默认只影响新 session

不支持官方未提供的 session 内热切换；替代为新 session + 显式 handoff。状态：accepted。备注：本决策取代交互中一度出现的 `mid-session-switch` 倾向（安全性不足）。

### `CONSOLE-AGENTHUB-V1-DEC-022` 桌面控制仅 selected-window

不提供通用全桌面 GUI 控制；secure desktop / 凭据界面不可控。状态：accepted。备注：本决策取代交互中一度出现的 `general-desktop-control` 倾向（风险过大）。

### `CONSOLE-AGENTHUB-V1-DEC-023` 第三方复用先过法务 gate

任何源码/文档/测试/协议复用前完成独立法务与第三方组件义务评估。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-024` Paseo 复用整体 AGPL（若复用）

Takeover Host/Provider/Relay 若复用 Paseo 则整体 AGPL-3.0-or-later；客户端许可另评估；未完成 gate 前只做 clean-room 借鉴。状态：accepted。

### `CONSOLE-AGENTHUB-V1-DEC-025` 外部 PID 紧急终止不算接管成功

外部 PID 保持 `unmanaged-observed`；仅提供 PC-local、精确 PID+creation time 的独立 emergency containment，结果显示 `result unknown`，不建模为接管成功。状态：accepted。备注：收敛交互中过宽的 `terminate-only-emergency` 理解。

### `CONSOLE-AGENTHUB-V1-DEC-026` 完成判定双轴

Direct 完成需 checks 观察 + user acceptance 两轴；不合并为单一 “done”；不显示 “Verified/CognitiveOS completed”。状态：accepted。

## 决策与文档映射

| DEC | 主要落地文档 |
|---|---|
| 002 / 006 | deployment-modes-and-guarantees.md |
| 004 | platforms/product-scope.md |
| 005 | adapters/README.md、adapters/tier1/* |
| 007–011 | architecture/takeover-architecture.md、session-and-file-adoption.md |
| 012–013 | security/security-and-credentials.md、architecture/takeover-architecture.md |
| 014–016 | architecture/relay-pairing-and-migration.md |
| 017 / 019 | collaboration/lead-workers.md |
| 018 | architecture/relay-pairing-and-migration.md |
| 020–021 | security/security-and-credentials.md |
| 022 | security/computer-control.md |
| 023–024 | security/licensing-and-terms.md |
| 025 | architecture/process-and-terminal.md |
| 026 | product/deployment-modes-and-guarantees.md |
