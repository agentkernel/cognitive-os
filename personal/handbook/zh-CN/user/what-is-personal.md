---
doc_id: user.what-is-personal
locale: zh-CN
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/product-design.md
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/apps/kernel-server/src/personal/server.rs
    symbols: ["serve_personal_loopback"]
  - path: personal/docs/product/linux-1.0-scope.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
fingerprint: "sha256:ae98fd0a48d1d134e1e1aa5202a52272f1995de80b12f8b9d15745ae05d23cb2"
non_claims:
  - 不构成 Gate、release、Profile、Windows 对等或 agent 收益声明；Linux 1.0 目标组合由正式计划拥有。
---

# Personal 是什么（不是什么）

## 是什么

一个面向单一 owner 的跨平台本地 Agent、账户、资源与受治理工作管理产品。当前 release
是 Linux 1.0；Personal 2.0 完整目标对 Windows、macOS 与 Linux 路径分别独立资格化。
本地 daemon 加确定性客户端让 Agent 工作**可审计、有预算、可恢复、不可虚假完成**：

- 一个 Rust daemon（`kernel-server --personal`）只绑定 loopback，是权威状态的唯一写入
  者（XDG 目录下的 SQLite WAL 数据库）。
- 其余一切——`cognitive` CLI、Pi 对话壳、SDK、sidecar——都是客户端。客户端提议；
  daemon 授权、持久化、调度、对账、验收。
- 六类用户可见资源分别治理：Memory、Skill、Tool、Context、Task、Runtime/Process。
  预算、权限、artifact、Intent/Effect、证据与事件横切其间。
- 你的 Provider API key 只存在于批准的 secret store（Linux Secret Service），绝不出
  现在配置文件、数据库、进程参数、日志或 Pi 进程中。

## 不是什么

- 不是云服务、账号体系或多租户控制面——一切本地、单一 owner。
- 不是通用 agent 市场：Linux 1.0 只资格化一个 agent（钉住版本的 Pi 包）及其
  sidecar；其他 agent 需要独立资格化。
- 不是 Linux 内核替代、驱动框架或 eBPF 控制面。
- 今天尚无 Windows 或 macOS 产品资格：1.0 的产品目标只有 Linux x86_64。仓库中已存在 Windows 安
  装表面（Credential Manager secret 后端、可检查的安装器与 scheduled-task 模板）且
  通过 CI，但其端到端安装战役（B01-W）尚未执行，因此不提供也不声明 Windows 安装。

## 当前形态（诚实概括）

总体 `partial`：安装、daemon、CLI、secret、Provider 代理、Pi 对话、Task 准入与六类
权威存储已实现并有测试；完全自主的 Task **执行**（调度驱动的工具执行与独立验证的端到
端接线）尚未接通——见 [Task 与执行](tasks-and-execution.md)。稳定产品意图由
[`personal/docs/product/`](../../../docs/product/README.md) 拥有；本页跟踪
代码今天真实做到的部分。

## 当前产品与 Personal 2.0 完整承诺

必须分开理解两条基线：

- **当前 Linux 1.0/当前 API：** 六个资源族；Pi 是唯一已资格化 Agent；daemon、CLI、
  Pi 路径、Provider Control Plane，以及位于 `clients/pc/web/` 的同源 `/ui/` SPA
  都是当前实现。Web UI 不属于 Linux 1.0 release 声明。
- **Personal 2.0 完整目标 — `Requires-backend`：** 各自独立资格化的 Windows、
  macOS、Linux 本地产品路径；精确 Pi、DeepSeek Harness Developer Preview 与受官方
  平台限制的 Codex desktop 路径；Account Hub；MCP 第七族；嵌入式对话；持久
  Goal -> Plan revision -> Task -> Attempt；多 Agent 监督；统一 Activity、控制与联邦
  资源。

完整版本承诺使每项都成为 release blocker，但仍不构成实现证据。固定 AI-window 分母
保持八个场景；Codex desktop 场景为平台受限场景（owner 决策 2026-08-27）——在活动执行
范围内没有受支持的 Codex desktop 平台期间记 `not-run (platform-conditional)`，
Linux 主线验收以七个平台可执行场景全过加该处置收口。即使完整 8/8 通过也只构成模拟
产品验收，不证明人类 desirability、usability、adoption、WTP 或 release/Gate 技术就绪。
