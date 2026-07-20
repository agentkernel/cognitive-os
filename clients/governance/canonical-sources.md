# 每域唯一 canonical 文件清单

> 类别：informative governance ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 规则：每域有且只有一个 canonical 文件；其他文档只链接不复制（[GOVERNANCE §1](../GOVERNANCE.md#1-canonical-唯一性)）。"现址"列指迁移批次落地前的真实路径；迁移批落地时同批改写本表。

| 域 | canonical 文件 | 现址（查询基准日） | 迁移批次 |
|---|---|---|---|
| 客户端项目地图/目录索引 | `clients/README.md` | [clients/README.md](../README.md) | B1（已落位） |
| Windows 产品（简报/范围/IA/旅程/DS/安全 UX/追踪/决策/roadmap） | `clients/pc/docs/**` + `clients/pc/plan/roadmap.md` 各文件 | [clients/pc](../pc/README.md) 九份 | B2（已落位） |
| 桌面平台切片（macOS/Linux/desktop-parity） | `clients/pc/docs/platforms/**` | [clients/pc/docs/platforms](../pc/README.md) 三份 | B2（已落位） |
| 桌面平台决策 | `clients/pc/docs/platforms/platform-decision-log.md` | [platform-decision-log](../pc/docs/platforms/platform-decision-log.md)（`CONSOLE-MAC/LNX-V1-DEC-*`） | B2（已落位） |
| 移动产品决策 | `clients/mobile/shared/docs/mobile-platform-decision-log.md`（iOS+AND 双命名空间单文件） | [mobile-platform-decision-log](../mobile/shared/docs/mobile-platform-decision-log.md) | B3（已落位） |
| iPhone / Android 产品设计 | `clients/mobile/{ios,android}/docs/*.md` | [ios](../mobile/ios/docs/ios-product-design.md) / [android](../mobile/android/docs/android-product-design.md) | B3（已落位） |
| 移动 parity | `clients/mobile/shared/docs/mobile-parity-matrix.md` | [mobile-parity-matrix](../mobile/shared/docs/mobile-parity-matrix.md) | B3（已落位） |
| 共用测试策略 / 遥测脱敏留存政策 | `clients/shared/docs/test-strategy.md`、`.../telemetry-evidence/telemetry-redaction-retention-policy.md` | 缺口（B4 新建） | B4 |
| Agent Hub 各专题（产品/架构/安全/协作/平台/决策/追踪/adapter/来源/模板） | `clients/agent-hub/docs/**` 各文件 | [agent-hub/docs](../agent-hub/docs/README.md) | B5（已落位） |
| Agent Hub 计划 / 提示词 | `clients/agent-hub/plan/**`、`clients/agent-hub/prompts/**` | [plan](../agent-hub/plan/README.md)、[prompts](../agent-hub/prompts/README.md) | B5（已落位） |
| Agent Hub 平台 parity | `clients/agent-hub/docs/platforms/agent-hub-platform-parity.md` | [agent-hub-platform-parity](../agent-hub/docs/platforms/agent-hub-platform-parity.md) | B5（已落位） |
| 客户端实现 gate | `clients/governance/readiness-gates.md` | [readiness-gates](readiness-gates.md#console-实现-gate) | B6（已落位） |
| 客户端结构决策（`CLIENTS-DEC-*`） | `clients/governance/decision-log.md` | [decision-log.md](decision-log.md) | B1（已落位） |
| readiness 双结论 | `clients/READINESS.md` | [READINESS.md](../READINESS.md) | B1（已落位） |
| 机器合同（REQ/错误码/schema/transition/vector） | `specs/**`、`conformance/**`（**不在 clients**） | [specs registry](../../specs/registry/requirements.yaml) | 不迁移 |

一个文件同时拥有多域职责时，先在本表决定唯一 owner 域，再迁移或拆成"canonical 正文 + 薄引用"，不得复制正文。
