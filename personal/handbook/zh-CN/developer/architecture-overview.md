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
  - path: personal/apps/kernel-server/src/personal/mod.rs
  - path: personal/apps/kernel-server/src/personal/resource_manager.rs
  - path: core/crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:1a32a17a44666535b098c1c67342eccfbfedd6d668b716e756f598c474024d0b"
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

## Personal 2.0 完整组合——不是当前实现

Personal 2.0 保留上述不变量，同时承诺以下产品边界：

- Windows、macOS、Linux 是各自独立资格化的本地产品路径，平台与 Agent 证据都不转移；
- 初始精确 Agent 集是 Pi、DeepSeek Harness Developer Preview，以及受官方平台限制的
  Codex desktop；CLI、Provider、model、account、adapter 或 bridge 证据不能资格化另
  一个产品；
- Account Hub 凭据导入是 ADR-0055 下 daemon 独占的来源到 SecretStore 操作。UI 只
  提供精确来源选择与同意，绝不读取或拿到导入材料；
- MCP 成为带联邦来源身份、trust、availability 与 policy 的第七个用户可见资源族。
  当前 Resource Manager 与权威服务仍保持六族，直到类型化 backend/core 工作落地；
- 厂商专用对话适配器保留每个 Agent 的协议与身份。Pi 仍是唯一已资格化 Agent；dsh
  实现证据与通用适配器合同都不转移资格；
- 嵌入式原生对话只有经显式 admission 才进入受治理工作；
- Goal -> 不可变 Plan revision -> Task -> 保留 Attempt 组合受治理工作；daemon-owned
  多 Agent 监督负责分配、fencing、budget、对账与验证；
- 统一 Activity 以声明的覆盖范围区分 Native、Observed、Governed、Verified provenance。

这些缺失能力仍为 `Requires-backend`；公开权威或合同增量仍为 `Requires-core`。完整
版本承诺与固定 8/8 AI-window 模拟验收都不构成实现、人类 usability、release 或 Gate
证据。

## 解释"意外"的设计决策

- 单 canonical 服务 + 固定 loopback 端口 48181（ADR-0034）——早期 UDS 与双 unit 晋升
  设计（ADR-0019/0032/0033）文本尚存，但产品路径已被取代。
- Pi 刻意双角色：shell 宿主（客户端）与受管 agent（受治理运行时），身份绝不合并
  （ADR-0035）。
- 当前 Linux 1.0/API 为六族、无通用 `Resource` 表（ADR-0037）。ADR-0057 采纳 MCP
  为 Personal 2.0 第七族，但不折叠各族权威；per-Agent sidecar 仍是集成边界
  （ADR-0038）。
- MVP-first 授权：owner-local、单 principal、task-scoped；RBAC 与审批链明确推迟。
