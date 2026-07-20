# Agent Hub — 规划入口

> 类别：informative ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文是 Agent Hub 开发计划与提示词的索引。计划本体位于 `docs/plan/`，提示词位于 `docs/prompts/agent-hub/`，均受 gate 阻断，未满足前不得启动编码。

## 1. 计划入口

- Master Development Plan：[docs/plan/agent-hub-development-plan.md](../../plan/agent-hub-development-plan.md)
- 宏车道与子车道计划目录：[docs/plan/agent-hub/](../../plan/)
  - README、milestones、dependency-dag、progress、risk-register、evidence-index
  - 12 个宏车道计划 + 6 个 Tier 1 Adapter 子车道计划
- 提示词目录：[docs/prompts/agent-hub/](../../prompts/)
  - README + 12 宏车道提示词 + 6 Adapter 提示词

## 2. gate 摘要

所有实现任务在以下任一 gate 未满足时保持 `blocked`（详见 [../GOVERNANCE.md](../GOVERNANCE.md#7-实现-gate不可跳过)）：

1. Console 依赖组 1/2/7 未交付；
2. M5 出口未过；
3. 目标平台 Open PoC / GA gate 未用真实 API/OS 行为留证；
4. 技术栈 ADR 未批准；
5. 适用 machine contract / implementation / evidence 未达门槛；
6. Paseo/AGPL 复用未过法务与第三方组件义务评估。

## 3. 文档↔计划映射

| 设计文档 | 对应宏车道 |
|---|---|
| product/* | 产品治理车道 |
| deployment-modes-and-guarantees / adapters | 合同与能力协商车道；6 Adapter 子车道 |
| architecture/takeover-architecture | Host/Control/Ledger 车道 |
| architecture/process-and-terminal | Process+Terminal 车道 |
| architecture/session-and-file-adoption | Session+File 车道 |
| security/security-and-credentials + collaboration + workspace | Credential+Workspace+Verifier 车道 |
| architecture/relay-pairing-and-migration | Relay/Pairing 车道 |
| platforms + states-content-and-accessibility | Desktop / iOS / Android 车道 |
| collaboration/lead-workers | Multi-Agent 车道 |
| 迁移 / 发布 / 质量 | Quality/Release/Migration 车道 |
