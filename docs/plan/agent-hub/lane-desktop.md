# 车道计划 — Desktop（DESK）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：Windows 首发桌面客户端，覆盖单 Agent 全旅程与所有页面状态。设计见 [product/journeys-and-screens.md](../../../apps/cognitiveos-console/docs/agent-hub/product/journeys-and-screens.md)、[states-content-and-accessibility.md](../../../apps/cognitiveos-console/docs/agent-hub/product/states-content-and-accessibility.md)。

## 范围与路径

- 允许（激活后）：Agent Hub 桌面客户端模块（复用 Console 设计系统）。
- 禁止：他人车道代码；不可信内容与系统控件共享安全边界；通用全桌面控制。
- 依赖：HOST、PROC、SESS、CRED。gate：AH-B1、AH-B2、AH-B3。

## 任务

### AH-DESK-01 Agent Hub shell 与信息架构
- owner/lane：Lane-CON / DESK｜depends_on：AH-HOST-02｜blocked_by：AH-B1,AH-B3
- 交付物：工作/Agent Hub/群组/收件箱/主机/系统 入口；每写页面持续显示 mode/Host/账号/workspace/来源/freshness
- 安全负例：Direct/Governed 保证不可视觉混同（PRD-002）
- oracle：页面状态矩阵全覆盖｜evidence：none

### AH-DESK-02 接管预览与 consent
- owner/lane：Lane-CON / DESK｜depends_on：AH-DESK-01,AH-PROC-01｜blocked_by：AH-B2
- 交付物：PAGE-003 必显字段（Agent/PID/session/owner/signature/version/cwd/account/level/动作/读文件/信号/不保证/release/generation）；危险按钮非默认 Enter
- 安全负例：普通既有进程不显示可 send input
- oracle：七结果标签如实呈现｜evidence：none

### AH-DESK-03 会话详情与不可信内容渲染
- owner/lane：Lane-CON / DESK｜depends_on：AH-DESK-01,AH-PROC-04｜blocked_by：—
- 交付物：终端/diff/artifact/usage；结构化摘要优先；raw terminal 可暂停；隔离低权限渲染面
- 安全负例：TM-020 注入内容不触发系统动作/伪 permission
- oracle：raw terminal 不持续抢焦点｜evidence：none

### AH-DESK-04 桌面无障碍
- owner/lane：Lane-CON / DESK｜depends_on：AH-DESK-01,AH-DESK-02｜blocked_by：AH-B2
- 交付物：纯键盘、Narrator、High Contrast、100–225% 缩放、reduced motion；高速事件聚合播报
- 失败测试先行：关键旅程纯键盘可完成
- 安全负例：状态不只靠颜色/图标/动画
- oracle：WCAG 2.2 关键旅程通过｜evidence：none

### AH-DESK-05 cancel/kill/unknown 与恢复 UX
- owner/lane：Lane-CON / DESK｜depends_on：AH-DESK-02,AH-PROC-03｜blocked_by：—
- 交付物：分别建模停止动作；超时显示 unknown 不显示已停止；Host 崩溃/睡眠/锁屏恢复先 resnapshot
- 安全负例：旧 controller 输入被拒
- oracle：恢复演练证据｜evidence：none
