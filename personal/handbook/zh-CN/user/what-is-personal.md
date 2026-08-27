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
fingerprint: "sha256:17492e3ebdab7f7cd7d73121927fbfe47e3f1466a258ecc0d966f61dbceb9edf"
non_claims:
  - 不构成 Gate、release、Profile、Windows 对等或 agent 收益声明；Linux 1.0 目标组合由正式计划拥有。
---

# Personal 是什么（不是什么）

## 是什么

一个本地 daemon 加确定性客户端，让 agent 工作**可审计、有预算、可恢复、不可虚假完成**：

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
- 今天不可在 Windows 安装：1.0 的产品目标只有 Linux x86_64。仓库中已存在 Windows 安
  装表面（Credential Manager secret 后端、可检查的安装器与 scheduled-task 模板）且
  通过 CI，但其端到端安装战役（B01-W）尚未执行，因此不提供也不声明 Windows 安装。

## 当前形态（诚实概括）

总体 `partial`：安装、daemon、CLI、secret、Provider 代理、Pi 对话、Task 准入与六类
权威存储已实现并有测试；完全自主的 Task **执行**（调度驱动的工具执行与独立验证的端到
端接线）尚未接通——见 [Task 与执行](tasks-and-execution.md)。稳定产品意图由
[`personal/docs/product/`](../../../docs/product/README.md) 拥有；本页跟踪
代码今天真实做到的部分。

## 当前产品与已采纳的 Personal 2.0 目标

必须分开理解两条基线：

- **当前 Linux 1.0/当前 API：** 六个资源族；Pi 是唯一已资格化 Agent；daemon、CLI、
  Pi 路径、Provider Control Plane，以及位于 `clients/pc/web/` 的同源 `/ui/` SPA
  都是当前实现。Web UI 不属于 Linux 1.0 release 声明。
- **已采纳的 Personal 2.0 目标 — `Requires-backend`：** 桌面优先 Control Plane
  重设计；带逐来源用户同意凭据导入的 Account Hub；MCP 第七族；厂商专用 Agent 对话
  适配器；持久 Goal 与 Plan revision；多 Agent 监督；以及联邦资源。

采纳只确定产品方向，不构成实现证据。当前没有 Goal/Plan API、MCP 资源族 API、Account
Hub 导入 API 或多 Agent 监督路径，现有 `/ui/` 也尚未完成目标重设计。
