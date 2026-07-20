# 模板 — Development Task

> 复制到相应 `docs/plan/agent-hub/*` 车道计划。ID 使用 `AH-<lane>-<seq>`。所有实现任务默认 `blocked`，直至 gate 满足。

## `AH-<lane>-<seq>` <任务标题>

- owner / lane：
- status：blocked（默认）
- depends_on：<前置任务 ID>
- blocked_by：<gate：Console 组1/2/7 ｜ M5 ｜ 平台 PoC ｜ ADR ｜ contract/impl/evidence ｜ AGPL 法务>
- 允许路径（仅可修改）：
- 禁止路径（不得触碰，尤其他人车道 crate/package、`specs/**`、`conformance/**`、许可证）：
- 交付物：
- 失败测试先行（先写失败测试再实现）：
- 安全负例（不可豁免）：
- oracle（判定通过的确定性标准）：
- evidence（执行后留证路径，当前 none / not-run）：
- 关联 REQ-ID / PRD / DEC / F-IMP：
- commit / PR（关联可追溯项）：
- 文档联动（docs-sync 三分类义务）：
- handoff（会话结束更新）：

## 规则

- 概率组件输出只能是 candidate/proposal；authority 写入在确定性代码集中入口。
- 不得用 mock/原型/代码存在冒充 gate 通过或任务完成。
- 逐路径 `git add`，禁 `git add -A`；不覆盖他人未提交改动。
