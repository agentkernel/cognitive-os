# 接续提示词 — Agent Hub Desktop 车道（DESK）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界；系统状态只来自 authority/Host projection。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；无障碍与安全先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；不可信内容与系统控件不共享安全边界；不做通用全桌面控制。

## 本车道任务

- canonical：[product/journeys-and-screens.md](../docs/product/journeys-and-screens.md)、[product/states-content-and-accessibility.md](../docs/product/states-content-and-accessibility.md)、[security/computer-control.md](../docs/security/computer-control.md)
- 计划：[clients/agent-hub/plan/lane-desktop.md](../plan/lane-desktop.md)
- 目标：Windows 首发桌面客户端，单 Agent 全旅程与全部页面状态，复用 Console 设计系统。

## gate 与允许范围（当前 blocked）

依赖 HOST/PROC/SESS/CRED；未满足后端/平台 PoC/ADR gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：Direct/Governed 保证不可视觉混同、普通既有进程不显示可 send、注入内容不触发系统动作/伪 permission、旧 controller 输入拒绝。无障碍（不可豁免）：纯键盘/Narrator/High Contrast/100–225% 缩放/reduced motion 完成关键旅程。任务 AH-DESK-01..05 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
