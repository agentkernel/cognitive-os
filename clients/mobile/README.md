# clients/mobile — 手机 remote companion 项目

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`；implementation `not-implemented`；evidence `none`；Profile `not implemented`

- **用途**：iPhone 与 Android phone 客户端的产品文档域、共享决策与保留实现入口。
- **共同边界（不可降级）**：手机是 Takeover Host / CognitiveOS 节点的 **remote companion**——不承载 Agent runtime、CognitiveOS authority、CognitiveOS node 或完整 Vault；只执行 authority 判定的 R0/R1；高后果动作保持 PC-local。保证的 canonical 定义只在 [Agent Hub 平台范围](../agent-hub/docs/platforms/product-scope.md) 与 [Relay/配对/迁移](../agent-hub/docs/architecture/relay-pairing-and-migration.md)，本目录不新造。
- **canonical 入口**（B3 已迁入本子树）：[iPhone 产品设计](ios/docs/ios-product-design.md)、[Android 产品设计](android/docs/android-product-design.md)、[移动决策](shared/docs/mobile-platform-decision-log.md)、[移动 parity](shared/docs/mobile-parity-matrix.md)。
- **gate**：[Console 实现 gate](../governance/readiness-gates.md#console-实现-gate) + 各平台 Open PoC/GA gate（iPhone `IOS-POC-01..18`、Android `POC-001..018`，全部 `not-run`）。
- **子目录**：[shared/](shared/README.md)（双平台共享决策与 parity）；[ios/](ios/README.md)、[android/](android/README.md)（各平台文档/计划/保留入口）。
- `ios/app/` 与 `android/app/` 保留入口已分配但**无任何实现**，不得出现 manifest、源码或脚手架。
