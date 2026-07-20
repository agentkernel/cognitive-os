# 客户端遥测、脱敏与留存政策（informative）

> 类别：informative policy ｜ owner：Lane-CON ｜ 日期：2026-07-20 ｜ 状态：telemetry implementation **`not-implemented`**
>
> 本文只固定政策边界，供未来实现与评审引用；不表示任何遥测、日志或证据管道已实现。遥测提供方、数据驻留与 retention 期限是未冻结产品选择（见 [PRODUCT-DESIGN §20.2](../../../../apps/cognitiveos-console/PRODUCT-DESIGN.md#202-尚未冻结的产品选择)），本文不预设结论。

## 1. 不进遥测与日志的内容（硬边界）

- secret、token、bootstrap 凭据、endpoint key、配对密钥材料：**永不**进入遥测、崩溃报告或客户端日志；
- 跨租户对象内容、其他用户/channel 的正文：不进任何客户端遥测；
- 敏感 snapshot 正文：Windows v1 不离线落盘（产品边界），同样不进遥测；
- 拒绝/错误路径不得在遥测中泄露资源存在性或 secret（与 [error-contract](../../../../docs/standards/error-contract.md) 拒绝同形原则一致）。

## 2. 展示脱敏

- 系统通知与托盘等 OS 面呈现一律脱敏（不含正文、不含 secret）；deep-link 只携带一次性 handle，解析在受信面完成；
- 投影展示遵循"客户端只消费 authority projection"边界，遥测不得记录比展示更高权限的内容。

## 3. 留存与证据分类

- 证据分类、五态结果与 digest 规则的 canonical 是 [conformance-evidence](../../../../docs/standards/conformance-evidence.md)：遥测/日志不是符合性证据；符合性证据只由 runner 生成并以 digest 引用 `artifacts/evidence/`；
- 客户端诊断数据留存期限、驻留区域与删除义务在提供方选定时以 ADR + 决策日志固化；在此之前任何留存承诺均为 `planned`；
- 审计事实唯一来源是 authority 侧 AuditRecord（[event-audit-watch](../../../../docs/standards/event-audit-watch.md)）；客户端遥测不得冒充审计。

## 4. 当前状态

无任何客户端遥测实现（`not-implemented`）；无遥测数据、无崩溃管道、无留存事实（`none`）。本政策在实现启动前按 docs-sync 义务随决策更新。
