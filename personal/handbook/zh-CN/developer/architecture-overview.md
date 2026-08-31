---
doc_id: dev.architecture-overview
locale: zh-CN
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: personal/docs/architecture/system-architecture.md
  - path: personal/docs/architecture/resource-manager-architecture.md
  - path: personal/docs/product/resource-manager-design.md
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: personal/docs/architecture/web-ui-architecture.md
  - path: personal/docs/architecture/multi-agent-orchestration.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/project-role-employee.md
  - path: personal/docs/architecture/conversation-memory-vault.md
  - path: personal/docs/architecture/windows-host-background.md
  - path: personal/docs/architecture/x-twitter-connector.md
  - path: personal/docs/architecture/routine-trigger-missed-run.md
  - path: personal/docs/architecture/personal-2.0-opc-v9-implementation-mapping.md
  - path: personal/docs/architecture/personal-2.0.0-dev-prep-index.md
  - path: personal/apps/kernel-server/src/personal/mod.rs
  - path: personal/apps/kernel-server/src/personal/resource_manager.rs
  - path: core/crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:d08507651d814ac1b11931e754299bda4090a0481a098cb3e3f1a97a9cd05ea7"
non_claims:
  - 目标架构文档记录意图；本页跟踪哪些部分已存在。两者都不是 Gate/release 证据。
---

# 架构总览

## 一切悬挂其上的不变量

> 概率组件只能产出 candidate 或 observation。只有确定性的 Rust daemon 才能授权、执行
> CAS、推进生命周期状态、授予预算或能力、持久化并对账 Effect、验收 Task。

具体而言：每次权威变更都流经 `cognitive-kernel` 的 `TransitionEngine` 十步门，进入
`cognitive-store` 的单写者 SQLite WAL 适配器，并对照 `cognitive-domain` 内嵌的
digest 钉住转移表与 `cognitive-contracts` 的 canonical digest 校验。

## 目标分层 vs 当前组合

目标设计（[`system-architecture.md`](../../../docs/architecture/system-architecture.md)）
画了五层：体验客户端 → Task/Resource 应用服务 → 六域服务 → sidecar/调度/执行器/验证
器执行层 → SQLite + artifact + secret + Linux 端口。

今天真实存在的：

- **体验层**：`cognitive` CLI、Pi 扩展、TypeScript SDK/Shell 库——全部是经 loopback
  HTTP、持通道绑定 bearer 的真实客户端。`implemented`。
- **应用服务**：`TaskApi`（record/interpret/preview/admit + watch）与私有六族资源投
  影 + Memory/Skill 路由，以及 `resource_manager.rs` 中的 management Resource Manager
  信封（`list`/`inspect`/`bind`/`unbind`/`enable`/`disable`/`revoke`）。上述操作
  `implemented`；`control`/`query_intent` 仍未暴露。watch 仍走 `/resource/v1/watch`。
- **域服务**：六族的权威存储 + kernel 服务齐备（见各域页面）。存储/服务层
  `implemented`。
- **执行层**：每个原语都存在（调度 CAS lease、封存 Context、candidate 准入、工具执行
  器、verifier 接缝、恢复），但连接它们的自主循环未接线——`partial`；见
  [执行链状态](execution-chain-status.md)。
- **平台端口**：SQLite WAL（双库）、文件系统 artifact CAS、Linux Secret Service、
  systemd 用户服务。`implemented`。

## Personal 2.0 Windows OPC 组合——不是当前实现

目标 dependency direction 为 Windows UI/Assistant/engine/connector -> daemon
application port -> daemon-owned Project/execution/memory/provider domain。Windows
host、DSH、Pi、Vault 与 connector adapter 都不拥有 authority。

- Project 拥有 Charter、Goal、Plan revision、manager Assignment 与 employee identity；
  Task/Attempt/Effect/verification 仍由 daemon 治理。
- Pi 是 hidden、candidate-only 的 Personal Assistant engine。
- DSH 是**隐藏托管** Member 执行引擎（不是可见 Installed Agent / 原生 DSH UI /
  engine store）：exact audited artifact、isolated child、bounded stdio broker、
  daemon Provider proxy 以及 update/rollback。没有 native DSH UI/conversation、
  raw secret、MCP/base tool、HMR 或 home patch。
- Personal 拥有 scoped Conversation archive/index/retrieval、Project Markdown Vault
  integration 与 semantic Memory admission/correct/forget。
- Routine/Trigger 使用 daemon-owned no-overlap、queue-latest、missed/coalesced fact 与
  risk-based resume。Engine checkpoint 不是 authority。
- Provider binding 按 global→Project→employee→Task；subscription、account、
  billing/quota、budget 与 actual usage 分离。
- UI 是 Today/Projects/Knowledge、底部 Settings 与持久右侧会话。Team 与 Inbox
  不是一级导航。2026-08-30 设计定档的当前 chrome 是 **CognitiveOS Personal
  2.0.0**（os-personal 2.0.0）。canvas 文件名可保留
  `personal-20-opc-e2e-optimized-v9` 作为历史文件名；勿再称 v9 为产品版本。
  architecture/formal-plan 用词对账推迟到 **完成后**。单模块维护入口：
  [`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md)。
  该索引同时收录设计 Agent / Owner 旅程难点研判
  （[`13-personal-20-agent-design-difficulty-and-journey-assessment.md`](../../../../clients/docs/design/opc-2.0/13-personal-20-agent-design-difficulty-and-journey-assessment.md)；
  hypothesis；不是 Gate、不是 P11 领取）。
  Scene → daemon 映射见
  [`personal-2.0-opc-v9-implementation-mapping.md`](../../../docs/architecture/personal-2.0-opc-v9-implementation-mapping.md)
  （历史路径名含 v9；informative；Owner 批准 ≠ 后端已存在；Project 聚合 walking skeleton 为 Personal-private；Markdown Vault import/index/conflict（`P11-T10`）已在 `main`（文件不是 Project 权威）；scoped Memory admission/privacy/forget（`P11-T11`）已在 `main`；Routine/Trigger walking skeleton（`P11-T08`）已在 `main`（复用 daemon `scheduler_entries`；不是 Inbox 一级）；Dual Track `/ui/` IA（`P11-T13`）已在 `main`（无权威诚实空态；不是完整 IA 验收）；Windows host/tray/background walking skeleton（`P11-T02`）已在 `main`（原生 install/tray/sleep/SecretStore E2E `not-run`）；X/Twitter connector walking skeleton（`P11-T14`）已在 `main`（live X API E2E `not-run`；不是 P0 hero））。
  开发前期索引（计划卡、测试/环境硬门、窗口提示词）：
  [`personal-2.0.0-dev-prep-index.md`](../../../docs/architecture/personal-2.0.0-dev-prep-index.md)。
- 权威对象英文 id 为 **Employee**。产品表面在完成后对账前仍可写
  **Member Runtime**。本页不改写产品旅程。
- HITL **只**在项目中心画布；Today 用深链进入。不要把独立
  `#/hitl/:approvalId` 写成产品一级或默认路由。
- `state-lab` 在 Settings 高级、默认隐藏、非一级，不是纯开发构建开关。
- Personal Conversation archive：在 **P11-T05 内**做新的 Personal private
  projection version；禁止重解释 `conversation-projection/0.1`；不要先开
  独立 Lane-CTR。只有必须改 **core 公共** conversation schema 的那一块才走
  Lane-CTR；T05 其余私有投影继续。此处不改 `core/specs`。
- `P11-T02`–`T15` 计划卡已于 **2026-08-30 按 Personal 2.0.0 chrome 对齐**
  （不以 Team/Inbox 一级或成员级 budget stop 为当前 chrome）。Project 聚合 walking skeleton
  为 Personal-private；Markdown Vault D01 已在 `main`（文件不是 Project 权威）；
  scoped Memory admission D01 已在 `main`；
  Routine/Trigger walking skeleton（`P11-T08`）已在 `main`；
  Dual Track `/ui/` IA（`P11-T13`）已在 `main`；
  Windows host walking skeleton（`P11-T02`）已在 `main`（原生 E2E `not-run`）；
  X/Twitter connector walking skeleton（`P11-T14`）已在 `main`（live X `not-run`；不是 P0 hero）。前期索引：
  [`personal-2.0.0-dev-prep-index.md`](../../../docs/architecture/personal-2.0.0-dev-prep-index.md)。

ADR-0058 的 MCP/private/fail-closed/P5-no-migration 边界保留；只 supersede dsh
first-conversation-slice 角色，`conversation-projection/0.1` 不重解释。MCP 从 OPC P0
deferred 为 advanced。

全部缺失能力仍为 `Requires-backend`；Windows host/DSH/connector validation 还
`Requires-environment`。native mobile/E2E relay remote 属于 2.1。future fixed
denominator 是 N=15，在 qualified Windows revision 执行前不构成任何证据。

## 解释"意外"的设计决策

- 单 canonical 服务 + 固定 loopback 端口 48181（ADR-0034）——早期 UDS 与双 unit 晋升
  设计（ADR-0019/0032/0033）文本尚存，但产品路径已被取代。
- Pi 刻意双角色：shell 宿主（客户端）与受管 agent（受治理运行时），身份绝不合并
  （ADR-0035）。
- 当前 Linux 1.0/API 为六族、无通用 `Resource` 表（ADR-0037）。ADR-0057 采纳 MCP
  为 Personal 2.0 第七族；ADR-0058 将其保持 Personal-private，不折叠各族权威；
  per-Agent sidecar 仍是集成边界（ADR-0038）。
- MVP-first 授权：owner-local、单 principal、task-scoped；RBAC 与审批链明确推迟。
