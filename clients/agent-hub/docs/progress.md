# Agent Hub 局部文档进度

> 类别：informative ｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 本页只跟踪 Agent Hub **文档** 交付进度。工程里程碑与实现状态真相以全局 [docs/plan/PROGRESS.md](../../../docs/plan/PROGRESS.md) 为准；不得在此声明实现、测试或 Profile 状态。

## 1. 文档交付状态

| 文档域 | 状态 | 备注 |
|---|---|---|
| 治理（README/GOVERNANCE/progress/planning） | done（v1 草案） | canonical 目录已建立 |
| 产品（product-design / deployment-modes / journeys / states） | done（v1 草案） | 两模式、旅程、状态、无障碍 |
| 架构（takeover / process-terminal / session-file / relay-migration） | done（v1 草案） | L1–L8、ownership generation、controller lease |
| 安全（threat-model / credentials / computer-control / licensing） | done（v1 草案） | 威胁逐项、密钥分层、AGPL gate |
| 协作 / 平台 / 决策 / 追踪 | done（v1 草案） | Lead+Workers、平台范围、DEC/PRD |
| Adapter（README / matrix / 6×tier1 / 其它 tier） | done（v1 草案） | 逐能力分级 |
| 来源 ledger（interfaces/terms/platform/paseo-comparables） | done（v1 草案） | 均含查询日与 URL |
| 模板（dossier/source/threat/poc/task） | done（v1 草案） | 供后续填充 |
| 开发计划与提示词 | done（v1 草案） | 见 [planning/README.md](./planning/README.md) |

## 2. 实现与证据状态（固定声明）

- Console/Agent Hub implementation：`not-implemented`。
- platform / PoC evidence：`none`。
- 既有 conformance vectors：`84`（全局 46 `pass` / 38 `not-run`）；不构成 Agent Hub implementation 或平台证据。
- Direct Takeover / Governed Profile：`not implemented`。

## 3. 更新规则

- 文档语义变化在此更新一行，并按 [GOVERNANCE.md](./GOVERNANCE.md) 决定是否联动全局 PROGRESS/findings-ledger。
- 不在此登记里程碑 GO/NO-GO；那属于全局 PROGRESS 和 milestone review。
