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
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/product/opc-product-model.md
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
fingerprint: "sha256:8e9adca952669badfc66e8d4aa96ce3eb4bb60668b95a8612cdaf211bef32a82"
non_claims:
  - 不构成 Gate、release、Profile、Windows 对等或 agent 收益声明；Linux 1.0 目标组合由正式计划拥有。
---

# Personal 是什么（不是什么）

## 是什么

一个让 Agent 工作可审计、有预算、可恢复且不可虚假完成的本地单 owner 系统。当前定稿
release 边界是 Linux 1.0；已采纳的 Personal 2.0 目标是**面向一人公司与个人开发者的
Windows-first 业务控制台**：Owner 在主机在线时用业务语言运营受治理 Project 与长期
Digital Employee。
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

## 当前产品与 Personal 2.0 OPC 目标

必须分开理解两条基线：

- **当前 Linux 1.0/当前 API：** 六个资源族；Pi 是唯一已资格化 Agent；daemon、CLI、
  Pi 路径、Provider Control Plane，以及位于 `clients/pc/web/` 的同源 `/ui/` SPA
  都是当前实现。Web UI 不属于 Linux 1.0 release 声明。
- **Personal 2.0 OPC 目标 — `Requires-backend` /
  `Requires-environment`：** Today / Projects / Knowledge、底部 Settings（Team 与
  Inbox 不是一级导航）、持久右侧会话；Project/Charter/Goal/Plan/Routine/Task/Attempt；
  Role Blueprint/Assignment/Digital Employee；Personal-owned Conversation/archive/
  Vault/admitted Memory；global→Project→employee→Task Provider/budget；以及固定
  Windows 验收路径。
- **Agent 边界：** Pi 是 hidden、candidate-only 的 Personal Assistant engine。DSH 是
  preinstalled managed Installed Agent 与默认员工 runtime，采用 exact audited artifact、
  isolated child、bounded stdio broker 与 daemon Provider proxy。Conversation、Memory、
  Task 和 completion 属于 Personal。Hermes、Codex、Cursor 等仅为 future qualification
  candidates。
- **Deferred：** MCP 保留为 advanced seventh-family target，但不是 OPC P0。native
  mobile、device pairing 与 E2E relay remote 从 Personal 2.1 开始。
- **当前交互原型（未交付）：** 2026-08-30 owner 批准的 chrome 是
  `clients/docs/design/opc-2.0/` 下的 `personal-20-opc-e2e-optimized-v9`。这是
  Canvas 规格，不是 daemon `/ui/`。创建顺序为 ① 项目 → ② 流程 → ③ 成员 → ④ 测试 →
  ⑤ 联调。单模块维护入口是
  [`clients/docs/design/opc-2.0/00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md)，
  不是已交付 UI。Owner 原型批准不是可用性、Gate 或 release 证据。

本文不声称 OPC backend 或 Windows/DSH qualification。Phase 11 的 future fixed
denominator 是 15 个场景；Canvas 与 ordinary CI 不执行也不提升它。没有 human
desirability、usability、adoption、WTP、support、release、Gate、Profile 或
Agent-benefit evidence。
