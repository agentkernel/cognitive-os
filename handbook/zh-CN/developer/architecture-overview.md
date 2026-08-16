---
doc_id: dev.architecture-overview
locale: zh-CN
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: docs/architecture/personal/system-architecture.md
  - path: apps/kernel-server/src/personal/mod.rs
  - path: crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:4f32265d6d8d5be498b9051181a7c093d346350413cb5638d7deb61c5fb60d66"
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

目标设计（[`system-architecture.md`](../../../docs/architecture/personal/system-architecture.md)）
画了五层：体验客户端 → Task/Resource 应用服务 → 六域服务 → sidecar/调度/执行器/验证
器执行层 → SQLite + artifact + secret + Linux 端口。

今天真实存在的：

- **体验层**：`cognitive` CLI、Pi 扩展、TypeScript SDK/Shell 库——全部是经 loopback
  HTTP、持通道绑定 bearer 的真实客户端。`implemented`。
- **应用服务**：`TaskApi`（record/interpret/preview/admit + watch）与私有六族资源投
  影 + Memory/Skill 路由。上述操作 `implemented`；`control`/`query_intent` 与通用
  `ResourceApplicationService` 词汇（bind/unbind/enable/…）未暴露。
- **域服务**：六族的权威存储 + kernel 服务齐备（见各域页面）。存储/服务层
  `implemented`。
- **执行层**：每个原语都存在（调度 CAS lease、封存 Context、candidate 准入、工具执行
  器、verifier 接缝、恢复），但连接它们的自主循环未接线——`partial`；见
  [执行链状态](./execution-chain-status.md)。
- **平台端口**：SQLite WAL（双库）、文件系统 artifact CAS、Linux Secret Service、
  systemd 用户服务。`implemented`。

## 解释"意外"的设计决策

- 单 canonical 服务 + 固定 loopback 端口 48181（ADR-0034）——早期 UDS 与双 unit 晋升
  设计（ADR-0019/0032/0033）文本尚存，但产品路径已被取代。
- Pi 刻意双角色：shell 宿主（客户端）与受管 agent（受治理运行时），身份绝不合并
  （ADR-0035）。
- 六族、无通用 `Resource` 表（ADR-0037）；per-agent sidecar 作为集成边界
  （ADR-0038）。
- MVP-first 授权：owner-local、单 principal、task-scoped；RBAC 与审批链明确推迟。
