# CognitiveOS Console

CognitiveOS Console（工作名）是面向 Agent 操作者的可视化客户端：以对话开始任务，同时让监督边界、Agent 生命周期、authority 状态和安全恢复保持可见。

## 当前产品切片

- 首发平台：Windows 桌面；
- 首要用户：Agent 操作者；
- 首要任务：对话与创建任务、监督/纠偏/暂停、Agent 安装/升级/回滚/卸载；
- 节点形态：一个本机共享 CognitiveOS Windows Service；
- 风险范围：R0/R1；R2/R3 仅识别并阻断；
- 交互方向：对话主画布 + 可折叠任务/上下文侧栏；
- 当前状态：`planned`。

macOS、受限 Linux、iPhone 与列名 Android phone 已形成独立的 `planned/blocked` 产品切片；它们不是 Windows v1 能力，也不表示 Console 实现已启动。移动 v1 是受限远程 Console：支持 Conversation/Task、监督纠偏、tenant/node 选择和远端 Agent 生命周期，但手机不承载 node/daemon/authority，且只执行 authority 判定的 R0/R1。完整企业治理、Memory、Knowledge、Multi-Agent 和 R2/R3 仍属于后续路线图。

## 状态声明

本目录只包含 informative 产品设计。文档存在不表示：

- 规范已登记；
- Console 或所需后端实现已提供；
- 测试已执行；
- 任何 Profile 已符合。

具体能力必须分别核对 contract、implementation 和 evidence 状态。

## 文档入口

- [PRODUCT-DESIGN.md](./PRODUCT-DESIGN.md)：v2 兼容入口，保留旧 §17 / §20.3 anchor；
- [产品简报](./docs/product-brief.md)：用户、JTBD、价值、边界和成功指标；
- [Windows v1 范围](./docs/windows-v1-scope.md)：发布切片、Service/账号/TOFU、能力和 gate；
- [信息架构](./docs/information-architecture.md)：落点、导航、Shell 和术语；
- [旅程与页面](./docs/journeys-and-screens.md)：核心流程、页面清单和状态矩阵；
- [Design System](./docs/design-system.md)：品牌、布局、组件、动效和无障碍；
- [可信与安全体验](./docs/trust-safety-ux.md)：authority、R0/R1、失联、包来源和错误边界；
- [产品要求与追踪](./docs/requirements-traceability.md)：v2 ID、旧 ID 映射和上游阻断；
- [路线图](./docs/roadmap.md)：非 Windows v1 feature briefs；
- [决策记录](./docs/decision-log.md)：本轮已确认和被替代的产品决策。
- [跨平台产品设计](../../docs/platforms/README.md)：macOS/Linux/iPhone/Android phone 范围、要求、决策、parity matrix 和真实 PoC gate。
- [PC + 手机客户端目录索引](../../docs/clients/README.md)：客户端、共享 SDK/契约、平台设计与 Agent Hub 目录的 canonical 导航。

## 技术状态

Tauri 2 + React/TypeScript 是 Windows v1 首选候选，不是已批准 ADR。技术栈、源码目录和发布包只能在 Windows Service/IPC、WebView 隔离、可访问性、升级和安全 PoC 通过后冻结。
