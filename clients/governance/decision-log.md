# 客户端结构决策日志（CLIENTS-DEC-*）

> 类别：informative decision log ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 本文件是 `CLIENTS-DEC-*`（客户端项目根结构/治理决策）的 canonical 定义点。**本文件不是第五本产品决策日志**：产品决策仍归 §2 所列四本，不得混入。

## 1. 结构决策

### CLIENTS-DEC-001 建立 clients/ 项目根并迁移客户端文档

- **日期**：2026-07-20
- **状态**：accepted（用户批准文件级迁移方案）
- **决策**：在仓库根建立 `clients/` 作为唯一客户端项目根（PC/mobile/shared/Agent Hub/governance/plan/prompts 七域）；将分散在 `docs/clients/`、`apps/cognitiveos-console/docs/`、`docs/platforms/`、`docs/plan/agent-hub*`、`docs/prompts/agent-hub/` 的客户端 informative 文档按 [canonical-sources.md](canonical-sources.md) 分批 `git mv` 迁入；旧路径保留 4 个兼容 stub（deprecated + successor，不复制正文）；不移动 Lane-TSC/Lane-CTR 代码 package；手机代码载体从"无已分配路径"变为 `app/` 保留入口（无任何实现）。
- **理由**：客户端文档分散四处，目录索引维护成本高；单一项目根降低跨会话导航与 canonical 漂移风险。
- **不改变**：任何实现 gate、四类状态、产品 ID、机器合同；checker 不扫 `clients/` 的缺口登记为 Lane-CFR `planned` 任务。
- **落地记录**：批次哈希见 [MIGRATION-MAP §1](../MIGRATION-MAP.md#1-批次与提交哈希)；ADR-0007 随治理联动批（B7）登记。

## 2. 四本产品决策日志索引（canonical 各自独立）

| 决策域 | canonical 文件（迁移前现址） | 命名空间 | 迁移批次 |
|---|---|---|---|
| Console v2（Windows） | [decision-log](../pc/docs/product/decision-log.md) | `CONSOLE-V2-DEC-*`（17 项实测） | B2（已落位） |
| 桌面平台（macOS/Linux） | [platform-decision-log](../pc/docs/platforms/platform-decision-log.md) | `CONSOLE-MAC-V1-DEC-*`（11）/`CONSOLE-LNX-V1-DEC-*`（11） | B2（已落位） |
| 移动平台（iPhone/Android） | [mobile-platform-decision-log](../mobile/shared/docs/mobile-platform-decision-log.md) | `CONSOLE-IOS-V1-DEC-*`（16）/`CONSOLE-AND-V1-DEC-*`（16） | B3（已落位） |
| Agent Hub | [decisions/decision-log](../agent-hub/docs/decisions/decision-log.md) | `CONSOLE-AGENTHUB-V1-DEC-*`（26） | B5（已落位） |

产品决策的新增/替代仍写入各自 canonical 文件并遵循其 `superseded` 规则；本日志只登记 `clients/` 结构与治理决策。
