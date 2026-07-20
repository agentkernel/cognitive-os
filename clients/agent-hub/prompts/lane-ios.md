# 接续提示词 — Agent Hub iOS 车道（IOS）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界；手机不承载 runtime/authority/完整 Vault。
2. 规范优先级；冲突取保守解释。
3. 四类状态用语严格区分。
4. 测试先行；无障碍与安全先行。
5. 规范表面冻结；漂移登记后修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；手机不得直接扩权/发信号/扩大文件范围。

## 本车道任务

- canonical：[product/journeys-and-screens.md §5](../docs/product/journeys-and-screens.md#5-手机-remote-companion-旅程要点)、[states-content-and-accessibility.md §4.2](../docs/product/states-content-and-accessibility.md#42-iphone)
- 计划：[docs/plan/agent-hub/lane-ios.md](../plan/lane-ios.md)
- 目标：iPhone remote companion（先于 Android）：配对/多 Host、监督/请求/裁剪投影、无障碍、回前台安全恢复。

## gate 与允许范围（当前 blocked）

依赖 RELAY + DESK；未满足后端/iOS 平台 PoC/GA/ADR gate 前不得写实现、不得 mock 解阻。安全负例（不可豁免）：手机不能直接扩权、不暴露其他用户/凭据/raw auth、push 仅 opaque hint、过期请求不静默补发。无障碍（不可豁免）：VoiceOver/Voice Control/Switch Control/Full Keyboard Access/外接键盘完成关键旅程；最大 Dynamic Type 不截断关键动词。任务 AH-IOS-01..04 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
