# 车道计划 — Quality / Release / Migration（QRM）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：Open PoC 执行、发布 gate、Direct→Governed evidence-only 迁移。设计见 [relay-pairing-and-migration.md §6](../../../apps/cognitiveos-console/docs/agent-hub/architecture/relay-pairing-and-migration.md#6-direct--governed-迁移evidence-only)、[evidence-index](../../../apps/cognitiveos-console/docs/agent-hub/traceability/evidence-index.md)。

## 范围与路径

- 允许（激活后）：测试/PoC harness、发布 gate 校验、迁移工具。
- 禁止：把测试/检查写成实现证据；把 Host ledger 改写为 authority Event。
- 依赖：全部车道。gate：全局 gate + Governed 契约（AH-B6）。

## 任务

### AH-QRM-01 Open PoC 执行与留证
- owner/lane：Lane-CON / QRM｜depends_on：各车道对应任务｜blocked_by：AH-B2,AH-B4,AH-B5
- 交付物：27+ 项 `CONSOLE-AGENTHUB-V1-POC-*` 在真实环境执行，证据落 artifacts；安全负例不可豁免
- 安全负例：任一负例失败即阻断对应里程碑
- oracle：evidence-index 全部由 not-run 转 pass/documented｜evidence：not-run

### AH-QRM-02 发布 gate 校验
- owner/lane：Lane-CON / QRM｜depends_on：AH-QRM-01｜blocked_by：全局 gate
- 交付物：核对 Console 后端/平台 PoC/ADR/接口/法务/契约六 gate；无错误完成声明/跨用户泄露/重复 Effect 的真实证据
- 安全负例：不得用 mock/原型冒充 gate
- oracle：发布 checklist 全绿且有证据｜evidence：none

### AH-QRM-03 Direct→Governed evidence-only 迁移
- owner/lane：Lane-CON / QRM｜depends_on：AH-QRM-01｜blocked_by：AH-B6
- 交付物：Governed 新建 authority 对象；Host ledger/artifact/digest 作外部证据导入；迁移报告（导入/证据/被拒/降级）
- 安全负例：不追认历史 authority/Verification；不改写 ledger 为 Event（TM-019）
- oracle：迁移后 Direct 记录不成为 authority｜evidence：none

### AH-QRM-04 无障碍与恢复回归
- owner/lane：Lane-CON / QRM｜depends_on：AH-DESK-04,AH-IOS-03,AH-AND-03｜blocked_by：—
- 交付物：跨端 WCAG 2.2 + 原生 AT 关键旅程回归；崩溃/孤儿/丢失设备恢复演练
- oracle：关键旅程全通过且留证｜evidence：none
