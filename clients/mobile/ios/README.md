# clients/mobile/ios — iPhone 客户端

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`；implementation `not-implemented`；evidence `none`；Profile `not implemented`

- **用途**：iPhone-only 客户端的产品文档、计划与保留实现入口。
- **canonical 入口**（B3 已迁入）：[iPhone 产品设计](docs/ios-product-design.md)（范围、APNs/设备绑定/R1、`CONSOLE-IOS-V1-PRD-*` 38 项、`IOS-TM-*` 16 项、`IOS-POC-01..18`）；决策见 [移动决策日志](../shared/docs/mobile-platform-decision-log.md)。
- **边界**：remote companion（见 [mobile/README.md](../README.md)）；不承载 runtime/authority/node/Vault。
- **gate**：[Console 实现 gate](../../../docs/platforms/README.md#console-实现-gate) + [iPhone Open PoC 与 GA gates](docs/ios-product-design.md#18-open-poc-与-ga-gates)（全部 `not-run`）+ iOS 技术栈 ADR（不存在）。
- **子目录**：[app/](app/README.md)（保留入口，无任何实现）；`docs/`（产品设计已迁入）；[plan/](plan/README.md)。
