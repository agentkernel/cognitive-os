# 车道计划 — 产品治理（GOV）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked（文档可先行）
>
> 目标：冻结 Agent Hub 术语、状态用语、完成语言、文案守则与决策同步，贯穿所有车道。任务用 [模板](../docs/templates/development-task.md)。

## 范围与路径

- 允许：`apps/cognitiveos-console/docs/agent-hub/`（治理/产品/决策/追踪）、本计划。
- 禁止：`specs/**`、`conformance/**`、他人车道 crate/package、实现代码。
- 依赖：无（先行）。gate：文档一致即可；不含实现。

## 任务

### AH-GOV-01 冻结术语与状态用语表
- owner/lane：Lane-CON / GOV｜status：in-progress（文档）｜depends_on：—｜blocked_by：—
- 允许路径：`.../agent-hub/GOVERNANCE.md`、`states-content-and-accessibility.md`
- 禁止路径：`specs/**`、实现代码
- 交付物：四态 + 三维 + Direct 事实来源标签 + 结果标签 + Direct 专用词汇表
- 失败测试先行：文档 lint（术语表存在、无 authority 术语混用）
- 安全负例：文案不得出现 “Verified/CognitiveOS completed” 用于 Direct
- oracle：全仓 grep Direct 页面无 authority 完成措辞
- evidence：none｜commit/PR：关联本任务｜文档联动：docs-sync｜handoff：会话末更新

### AH-GOV-02 完成语言与不保证事项守则
- owner/lane：Lane-CON / GOV｜depends_on：AH-GOV-01｜blocked_by：—
- 交付物：五类完成语言分开展示规范；每写页面 mode/Host/账号/来源/freshness 持续显示规范
- 失败测试先行：UI 文案快照校验（未来实现时）
- 安全负例：不得合并完成语言为单一 “done”
- oracle：文案审查通过｜evidence：none

### AH-GOV-03 决策日志同步机制
- owner/lane：Lane-CON / GOV｜depends_on：AH-GOV-01｜blocked_by：—
- 交付物：`CONSOLE-AGENTHUB-V1-DEC-*` 变更→受影响文档联动流程；superseded 规则
- 安全负例：不得静默改写既有 `CONSOLE-V2/MAC/LNX/IOS/AND-*` 决策
- oracle：决策↔文档映射表无断链｜evidence：none

### AH-GOV-04 文案国际化术语表（zh-CN/en）
- owner/lane：Lane-CON / GOV｜depends_on：AH-GOV-01｜blocked_by：—
- 交付物：统一术语表；机器 enum/错误码/REQ-ID 保留原文旁注解释
- 安全负例：不得拼接半句、不得本地化机器标识符
- oracle：术语表覆盖所有页面动词/状态｜evidence：none
