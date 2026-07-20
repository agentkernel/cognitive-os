# 接续提示词 — Agent Hub Android 车道（AND）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界；手机不承载 runtime/authority。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；无障碍与安全先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；手机不得直接扩权。

## 本车道任务

- canonical：[states-content-and-accessibility.md §4.3](../docs/product/states-content-and-accessibility.md#43-android-phone)
- 计划：[clients/agent-hub/plan/lane-android.md](../plan/lane-android.md)
- 目标：Android phone remote companion（后于 iPhone，形态与 iOS 对齐）：配对/多 Host、投影、无障碍、恢复。

## gate 与允许范围（当前 blocked）

依赖 RELAY + DESK + IOS（形态对齐）；未满足后端/Android 平台 PoC/GA/ADR gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：手机不能直接扩权、不暴露凭据、push 仅 opaque hint。无障碍（不可豁免）：TalkBack/Switch Access/Voice Access/外接键盘完成关键旅程；Compose 用真实 role/state/action semantics（非 aria）；200% font 与最大 Display size 分别测试。任务 AH-AND-01..04 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
