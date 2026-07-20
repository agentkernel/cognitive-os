# 客户端证据指针索引

> 类别：informative evidence pointer index ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 规则：证据口径以 [conformance-evidence](../../docs/standards/conformance-evidence.md) 为准；证据文件不入库、以 digest 引用。本索引只登记指针；**当前全部客户端证据状态为 `none` / `not-run`**，无一例外。

| 证据域 | 指针（迁移前现址） | 状态 |
|---|---|---|
| Agent Hub Open PoC / implementation / Profile | [agent-hub evidence-index](../../apps/cognitiveos-console/docs/agent-hub/traceability/evidence-index.md) | implementation `not-implemented`；Open PoC 全 `not-run`；evidence `none`；Profile `not implemented` |
| macOS PoC（`MAC-POC-01..12`） | [macos-product-design §13](../../docs/platforms/macos-product-design.md#13-open-poc-and-ga-gates) | 全部 `not-run`；evidence `none` |
| Linux PoC（`LNX-POC-01..12`） | [linux-product-design §13](../../docs/platforms/linux-product-design.md#13-open-poc-and-ga-gates) | 全部 `not-run`；evidence `none` |
| iPhone PoC（`IOS-POC-01..18`） | [ios-product-design §18](../../docs/platforms/ios-product-design.md#18-open-poc-与-ga-gates) | 全部 `not-run`；evidence `none` |
| Android PoC（`POC-001..018`） | [android-product-design §18](../../docs/platforms/android-product-design.md#18-open-poc-与-ga-gates) | 全部 `not-run`；evidence `none` |
| Windows release gate | [windows-v1-scope §10](../../apps/cognitiveos-console/docs/windows-v1-scope.md#10-技术候选与-release-gate) | 未满足；无平台专属测试证据 |
| conformance 向量（全局，非客户端专属） | [PROGRESS 向量分层计数](../../docs/plan/PROGRESS.md) | 76 份全部 `not-run` |
| 客户端端到端测试（tests/e2e 占位） | [tests/e2e](../../tests/e2e/README.md) | 占位（M4/M5）；未执行 |

目录、README、计划或提示词的存在不构成证据；静态一致性检查通过不构成实现、PoC、向量执行或 Profile 证据。
