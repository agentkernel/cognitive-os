# 客户端证据指针索引

> 类别：informative evidence pointer index ｜ owner：Lane-CON ｜ 日期：2026-07-20（2026-07-21：PoC runbook / 共享执行模板指针）
>
> 规则：证据口径以 [conformance-evidence](../../docs/standards/conformance-evidence.md) 为准；证据文件不入库、以 digest 引用。本索引只登记指针；**当前全部客户端证据状态为 `none` / `not-run`**，无一例外。
>
> 共享执行记录模板：[shared/docs/poc-execution-record.md](../shared/docs/poc-execution-record.md)（模板已提供 ≠ 已执行）。

| 证据域 | 指针（迁移前现址） | 状态 |
|---|---|---|
| Agent Hub Open PoC / implementation / Profile | [agent-hub evidence-index](../agent-hub/docs/traceability/evidence-index.md)；准备清单：[poc-prep-checklist](../agent-hub/docs/traceability/poc-prep-checklist.md) | implementation `not-implemented`；Open PoC 全 `not-run`/`planned`；evidence `none`；Profile `not implemented` |
| macOS PoC（`MAC-POC-01..12`） | 定义：[macOS §13](../pc/docs/platforms/macos/macos-product-design.md#13-open-poc-and-ga-gates)；执行骨架：[macos-poc-runbook](../pc/docs/platforms/macos/macos-poc-runbook.md) | 全部 `not-run`；evidence `none` |
| Linux PoC（`LNX-POC-01..12`） | 定义：[Linux §13](../pc/docs/platforms/linux/linux-product-design.md#13-open-poc-and-ga-gates)；执行骨架：[linux-poc-runbook](../pc/docs/platforms/linux/linux-poc-runbook.md) | 全部 `not-run`；evidence `none` |
| iPhone PoC（`IOS-POC-01..18`） | 定义：[iPhone §18](../mobile/ios/docs/ios-product-design.md#18-open-poc-与-ga-gates)；执行骨架：[ios-poc-runbook](../mobile/ios/docs/ios-poc-runbook.md) | 全部 `not-run`；evidence `none` |
| Android PoC（`CONSOLE-AND-V1-POC-001..018`） | 定义：[Android §18](../mobile/android/docs/android-product-design.md#18-open-poc-与-ga-gates)；执行骨架：[android-poc-runbook](../mobile/android/docs/android-poc-runbook.md) | 全部 `not-run`；evidence `none` |
| Windows release gate（`WIN-RG-01..10`） | 定义：[windows-v1-scope §10](../pc/docs/platforms/windows/windows-v1-scope.md#10-技术候选与-release-gate)；执行骨架：[windows-poc-runbook](../pc/docs/platforms/windows/windows-poc-runbook.md) | 未满足；全部 `not-run`；evidence `none` |
| conformance 向量（全局，非客户端专属） | [PROGRESS 向量分层计数](../../docs/plan/PROGRESS.md) | 84 份：46 `pass` / 38 `not-run`；不构成客户端平台证据 |
| 客户端端到端测试（tests/e2e 占位） | [tests/e2e](../../tests/e2e/README.md) | 占位（M4/M5）；未执行 |

目录、README、计划或提示词的存在不构成证据；静态一致性检查通过不构成实现、PoC、向量执行或 Profile 证据。
