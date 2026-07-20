# 客户端产品 ID 命名空间总表

> 类别：informative traceability ｜ owner：Lane-CON ｜ 日期：2026-07-20（计数一律实测，IMP-17）
>
> 规则：产品 ID 不进入 CognitiveOS normative registry，不得与真实 `REQ-*` 混称；一经发布不重编号、不重用、不删除。canonical 定义点随迁移批次落位后同批改写本表。

| 命名空间 | 实测计数 | canonical 定义点（迁移前现址） | 迁移批次 |
|---|---|---|---|
| `CONSOLE-V2-PRD-*` | 49 | [requirements-traceability §2](../../apps/cognitiveos-console/docs/requirements-traceability.md) | B2 |
| `CONSOLE-V2-BLK-*` | 14 | [requirements-traceability §3](../../apps/cognitiveos-console/docs/requirements-traceability.md) | B2 |
| `CONSOLE-V2-DEC-*` | 17 | [decision-log](../../apps/cognitiveos-console/docs/decision-log.md) | B2 |
| `CONSOLE-V2-JRN-*` | 10 | [journeys-and-screens §2](../../apps/cognitiveos-console/docs/journeys-and-screens.md) | B2 |
| `CONSOLE-V2-PAGE-*` | 19 | [journeys-and-screens §3](../../apps/cognitiveos-console/docs/journeys-and-screens.md) | B2 |
| `CONSOLE-V2-CMP-*` | 12 | [design-system §4](../../apps/cognitiveos-console/docs/design-system.md) | B2 |
| `CONSOLE-MAC-V1-PRD-*` / `CONSOLE-LNX-V1-PRD-*` | 24 / 24 | [macos](../../docs/platforms/macos-product-design.md) / [linux](../../docs/platforms/linux-product-design.md) 产品设计 | B2 |
| `CONSOLE-MAC-V1-DEC-*` / `CONSOLE-LNX-V1-DEC-*` | 11 / 11 | [platform-decision-log](../../docs/platforms/platform-decision-log.md) | B2 |
| `MAC-POC-*` / `LNX-POC-*` | 12 / 12 | 各自产品设计 §13 | B2 |
| `CONSOLE-IOS-V1-PRD-*` | 38 | [ios-product-design](../../docs/platforms/ios-product-design.md) | B3 |
| `CONSOLE-IOS-V1-DEC-*` | 16 | [mobile-platform-decision-log](../../docs/platforms/mobile-platform-decision-log.md) | B3 |
| `IOS-POC-*` / `IOS-TM-*` | 18 / 16 | ios-product-design §18 / 威胁模型节 | B3 |
| `CONSOLE-AND-V1-PRD-*` | 40 | [android-product-design](../../docs/platforms/android-product-design.md) | B3 |
| `CONSOLE-AND-V1-DEC-*` | 16 | [mobile-platform-decision-log](../../docs/platforms/mobile-platform-decision-log.md) | B3 |
| `POC-001..018`（Android） / `AND-TM-*` | 18 / 22 | android-product-design §18 / 威胁模型节 | B3 |
| `CONSOLE-AGENTHUB-V1-PRD-*` | 28 | [product-requirements](../../apps/cognitiveos-console/docs/agent-hub/traceability/product-requirements.md) | B5 |
| `CONSOLE-AGENTHUB-V1-DEC-*` | 26 | [agent-hub decision-log](../../apps/cognitiveos-console/docs/agent-hub/decisions/decision-log.md) | B5 |
| `CONSOLE-AGENTHUB-V1-TM-*` | 21 | [threat-model](../../apps/cognitiveos-console/docs/agent-hub/security/threat-model.md) | B5 |
| `POC-LIC/PROC/TERM/SESS/FILE/CRED…-*`（Agent Hub Open PoC 族） | 28（evidence-index 汇总，全部 `not-run`） | [agent-hub evidence-index](../../apps/cognitiveos-console/docs/agent-hub/traceability/evidence-index.md) | B5 |
| `AH-<lane>-<seq>` + `AH-R*` + `AH-M0..M6` | 68 个唯一值（任务/风险/里程碑） | [agent-hub 计划树](../../docs/plan/agent-hub/README.md) 各文件 | B5 |
| `CLIENTS-DEC-*` | 1 | [decision-log.md](decision-log.md)（本域 canonical） | B1（已落位） |

注：`CONSOLE-AGENTHUB-V1-TM-*` 实测 21 项（TM-001..TM-021）；上一会话 handoff 记为 20 项，属计数漂移，以实测为准。旧 `CONSOLE-PRD-001..034`、`A-01..34` 已停止新增和复用（映射见 requirements-traceability §5）。
