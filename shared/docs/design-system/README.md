# shared/docs/design-system — 共享设计系统说明

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`

- **用途**：跨平台共享 design tokens 与平台原生差异的说明入口。
- **canonical 指针**（不复制正文）：
  - PC Design System（品牌/布局/组件/动效/无障碍）：[design-system.md](../../../pc/docs/ux/design-system.md)（B2 已迁入）；
  - 平台差异：[桌面 parity](../../../pc/docs/platforms/desktop-parity-matrix.md) 与 [移动 parity](../../../mobile/shared/docs/mobile-parity-matrix.md)。
- **边界**：设计 token 尚无独立共享定义文件；平台原生控件差异以各平台产品设计为准；本目录不新造视觉规范；**不写完整 token 大文件**。
- **gate**：[Console 实现 gate](../../../governance/readiness-gates.md#console-实现-gate)。

## planned 缺口登记（最小）

| 缺口 | 状态 | 说明 |
|---|---|---|
| 共享 design token 完整规格 | planned | 颜色/字号/间距/运动 token；待实现 gate 接近时另开 informative 任务 |
| 暗色模式策略 | planned | PC design-system 有无障碍/主题线索；跨平台暗色合同未共享固化 |
| 图标体系（语义 + 平台映射） | planned | 无共享 icon 清单；禁止在 gate 前塞实现资源 |
| 跨平台术语表全文 | planned | 产品用语与状态名对照；现仅分散在各产品设计 |
| WCAG SC 映射表 | planned | 旅程级 a11y 要求在各平台 PoC；缺共享 SC↔组件映射 |
| iOS `outcome-unknown` → `result-unknown` 收敛 | planned（建议） | PC/Agent Hub/Android 多用 `result-unknown`；iOS 产品设计仍见 `outcome-unknown`（如 §18 附近状态表）。**建议**在实现前统一为 `result-unknown`，避免 UI/文档/遥测三套词；本登记不修改产品设计正文 |

缺口登记 ≠ token 已提供；不得据此声称设计系统 implemented。
