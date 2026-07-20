# clients/pc/app — 保留的未来实现根（无任何实现）

> 类别：informative placeholder ｜ owner：Lane-CON ｜ 状态：`blocked`；implementation `not-implemented`

- **用途**：声明 PC 客户端未来实现代码的落位。当前**无任何实现**：无 package manifest、无源码、无 UI 组件、无 mock、无构建脚手架。
- **NO-GO 条件**（全部满足前禁止创建任何实现文件）：
  1. [DEVELOPMENT-PLAN Console 节](../../../docs/plan/DEVELOPMENT-PLAN.md) 依赖组 1、2、7 交付；
  2. M5 出口评审通过；
  3. Windows 真实平台 PoC 用真实 API/真实 OS 行为留下可复现实测证据（[windows-v1-scope §10](../docs/platforms/windows/windows-v1-scope.md#10-技术候选与-release-gate)）；
  4. PC 技术栈 ADR 已批准（Tauri 2 + React/TypeScript 仅为候选，非已批准 ADR）。
- **gate 权威**：[Console 实现 gate](../../governance/readiness-gates.md#console-实现-gate)。
- 本 README 不含也不得被解读为任何"技术栈已批准"或"实现已启动"的暗示。
