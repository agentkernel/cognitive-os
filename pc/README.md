# clients/pc — PC 客户端项目

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`；implementation `not-implemented`；evidence `none`；Profile `not implemented`

- **用途**：PC 客户端（CognitiveOS Console）的产品/平台/UX/安全文档域与保留实现入口。Windows 是首发平台；macOS 与受限 Linux 是 parity 切片（`planned/blocked`），不属于 Windows v1。
- **边界**：Console 不是节点、authority、IdP、Runtime 或最终安全仲裁器；只消费 authority projection；`CANDIDATE_COMPLETE` ≠ `COMPLETED`；风险下界由 authority 决定，首版只执行 R0/R1。
- **canonical 入口**（B2 已迁入本子树）：
  - [Console README](../../apps/cognitiveos-console/README.md) 与 [PRODUCT-DESIGN](../../apps/cognitiveos-console/PRODUCT-DESIGN.md)（兼容入口）；
  - [Windows v1 范围](docs/platforms/windows/windows-v1-scope.md)；
  - [macOS](docs/platforms/macos/macos-product-design.md) / [Linux](docs/platforms/linux/linux-product-design.md) 产品设计与 [桌面 parity](docs/platforms/desktop-parity-matrix.md)、[桌面决策](docs/platforms/platform-decision-log.md)。
- **PoC 执行骨架**（全部 not-run）：[windows](docs/platforms/windows/windows-poc-runbook.md) / [macos](docs/platforms/macos/macos-poc-runbook.md) / [linux](docs/platforms/linux/linux-poc-runbook.md)；共享模板 [poc-execution-record](../shared/docs/poc-execution-record.md)。
- **技术栈候选比较（非 ADR）**：[tech-stack-comparison](docs/architecture/tech-stack-comparison.md)。
- **gate**：[Console 实现 gate](../governance/readiness-gates.md#console-实现-gate)（依赖组 1/2/7 + M5 + 目标平台真实 PoC + 技术栈 ADR）。
- **子目录**：[app/](app/README.md)（保留实现根，无任何实现）；`docs/`（product/ux/security/platforms/accessibility/quality/release/architecture）；[plan/](plan/README.md)。

目录与 README 的存在不表示实现已提供、测试已执行或 Profile 已符合。
