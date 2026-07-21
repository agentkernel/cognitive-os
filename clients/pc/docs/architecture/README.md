# clients/pc/docs/architecture — 客户端架构（薄入口）

> 类别：informative ｜ owner：Lane-CON ｜ 状态：`planned`；implementation `not-implemented`

- **用途**：PC 客户端架构文档的保留入口。当前无独立架构文档；canonical 正文分布在：
  - 部署边界与 Service/IPC 拓扑：[windows-v1-scope §2 部署边界](../platforms/windows/windows-v1-scope.md#2-部署边界)；
  - 客户端非 authority 与信任边界：[trust-safety-ux §2 信任边界](../security/trust-safety-ux.md#2-信任边界)；
  - Agent Hub 接管架构（跨产品线）：[takeover-architecture](../../../agent-hub/docs/architecture/takeover-architecture.md)。
- **技术栈候选比较（非正式 ADR）**：[tech-stack-comparison.md](tech-stack-comparison.md) — **候选比较，非已批准 ADR**；不得据此启动实现。
- **边界**：技术栈（Tauri 2 + React/TypeScript）仅为候选，非已批准 ADR；本目录不新造架构事实，不含实现。
- **gate**：[Console 实现 gate](../../../governance/readiness-gates.md#console-实现-gate)。
