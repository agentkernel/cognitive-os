# shared/docs/relay-pairing — Relay / E2EE / 配对说明

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`；implementation `not-implemented`

- **用途**：PC↔手机 Relay、端到端加密、配对/撤销边界的说明入口。
- **canonical 指针**（不复制正文）：[Relay/配对/迁移](../../../../apps/cognitiveos-console/docs/agent-hub/architecture/relay-pairing-and-migration.md)（B5 迁移前现址；迁移后为 `clients/agent-hub/docs/architecture/relay-pairing-and-migration.md`，由 B5 批同批改写本指针）。
- **边界**：Relay 不是 authority；配对/撤销保证只由 canonical 文档定义；手机侧受 remote companion 边界约束（[mobile/README.md](../../../mobile/README.md)）。
- **gate**：Agent Hub gate（[GOVERNANCE §7](../../../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md#7-实现-gate不可跳过)）+ 相关 PoC（`POC-RELAY-001..004`，全部 `not-run`）。
