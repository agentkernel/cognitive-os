# Agent Hub 风险 / Blocker 登记

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 登记开发风险与 blocker；与仓库 [findings-ledger](../../../docs/traceability/findings-ledger.md) 的关系：Agent Hub 特有风险在此，触碰全仓 F/IMP 时同步 ledger。

## 风险

| ID | 风险 | 对冲 | 状态 |
|---|---|---|---|
| AH-R1 | Direct 记录被误当 CognitiveOS authority | GOV 车道冻结术语/完成语言；威胁 TM-019 | open |
| AH-R2 | 供应商接口频繁变更（wrapper 脆弱，Omnara 反例） | 优先官方 SDK/App Server；跨版本漂移进 Adapter 未决；CTR gate | open |
| AH-R3 | 无 exclusive lease 的双 writer 写坏 session | SESS 只在证明单 writer 时写；TM-006 | open |
| AH-R4 | same-UID 恶意进程绕过 Host | 文档披露限制；强隔离需独立 principal；不虚称强隔离 | open |
| AH-R5 | Paseo/AGPL 复用触发源码提供义务 | 法务 gate 前只 clean-room 借鉴；DEC-024 | open |
| AH-R6 | 平台差异导致虚假安全声明（Job vs 进程组 vs cgroup） | 逐平台分别声明与测试，不跨平台外推 | open |
| AH-R7 | 手机静默越权 | 高后果动作 PC-local 确认；TM-013 | open |
| AH-R8 | Relay MITM/replay/丢失设备 | E2EE + matching code + anti-replay + 单设备 revoke；TM-010/011/012 | open |
| AH-R9 | 计数/文档漂移（如“74 vectors”） | 从全局 PROGRESS 读实测数；docs-sync 联动；已在 findings-ledger D-012 登记并修正 AGENTS/DEVELOPMENT-PLAN/IMP-17 为 76 | mitigated |
| AH-R10 | Hermes/OpenClaw 接口事实缺失导致臆造 | 标 `待核验`，接口 gate 前 Adapter `blocked` | open |

## Blocker（全局 gate）

| ID | Blocker | 阻断范围 | 解除条件 |
|---|---|---|---|
| AH-B1 | Console 后端组 1/2/7 + M5 未交付 | 全部实现车道 | 后端交付 + M5 出口 |
| AH-B2 | 平台 PoC/GA gate 未留证 | 目标平台实现 | 真实 API/OS PoC pass |
| AH-B3 | 技术栈 ADR 未批准 | HOST/DESK/RELAY/IOS/AND | ADR 批准 |
| AH-B4 | 接口未一手核验 | 对应 Adapter + 高层能力 | provider-interfaces-ledger 补齐 |
| AH-B5 | Paseo/AGPL 法务未过 | 复用 Paseo 的车道 | 法务评估通过 |
| AH-B6 | Governed 契约缺失 | Governed 迁移/受治理 Adapter | Lane-CTR 登记 `REQ-AGENT-*` 等 |

## P0 门禁

任何开放 P0（本登记或全仓 findings-ledger）未闭合前，对应子系统不得进入实现里程碑。
