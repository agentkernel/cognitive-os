---
doc_id: user.system-overview
locale: zh-CN
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/README.md
  - path: personal/docs/product/cognitive-resource-model.md
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: personal/docs/architecture/README.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/product/opc-product-model.md
  - path: personal/docs/product/knowledge-memory-vault.md
  - path: personal/docs/product/long-running-operations.md
  - path: personal/apps/kernel-server/src/personal/resource_api.rs
  - path: personal/apps/kernel-server/src/personal/task_api.rs
tests:
  - personal/apps/kernel-server/tests/p2_t02_resource_projection.rs
  - personal/apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs
fingerprint: "sha256:668a7fc2e06d13a163f4f9833b41f58bc9e37ffcd9435515cbd12e9f2d00f497"
non_claims:
  - 本页用于建立概念，不构成 release、Gate、Profile 或 agent 收益声明。
  - 调度器驱动的完全自主执行与独立验证仍为 partial；见 Task 与执行。
  - Linux 1.0 声明组合不包含 Windows 安装对等、Web UI、MCP 资源族或多 Agent 编排；当前 `/ui/` 已存在不改变该边界。
---

# 系统总览

Routine 是项目运行子菜单（P11-T08 walking skeleton），不是 Inbox 一级，也不是 Team。

CognitiveOS Personal 是本地、单一 Owner 的受治理 Agent 工作系统。当前 Linux 1.0
管理六个认知资源族；Personal 2.0 目标新增 Windows-first OPC 业务控制台，让 Owner
运营 Project 与 Digital Employee。Rust daemon 拥有 authority；UI、Assistant、engine、
adapter 与 connector 只能请求、提案或观察。

## 一张图理解

```text
使用者 -> cognitive CLI 或 Pi Shell -> 本地 Rust daemon -> 当前六个领域服务
                                      |                 -> Provider 代理
                                      |                 -> SQLite 权威存储
                                      `-> Intent/Effect、预算、证据、事件
```

daemon 是唯一的权威写入者。客户端响应、Provider 响应、Pi `agent_end` 或进程
退出都不单独代表 Task 完成。变更操作必须先持久化再派发，并在完成前对账；只有
独立验证器基于当前证据给出结果后，权威状态才能完成 Task。

## 六类资源

| 资源族 | 用来做什么 | 当前用户面 |
|---|---|---|
| Memory | 保存带来源、范围和遗忘语义的持久知识 | daemon `remember`/`forget`/explain 路由与权威搜索 |
| Skill | 管理版本化本地包及绑定 | import、bind、revoke、explain；包本身不会自行执行 |
| Tool | 使用受界定的工作区读/搜/写/patch 等操作 | 静态目录、生命周期 overlay、校验器和受治理调用 |
| Context | 为 Task 组装经过授权的输入 | daemon 侧过滤、重新授权、digest 绑定和有界视图 |
| Task | 持久化意图、预览、合同、进度和验收 | 准入、watch、evidence 和调度状态 |
| Runtime/Process | 管理 agent 包、安装、实例、sidecar 与进程尝试 | 受管理 Pi 生命周期和 Runtime 投影 |

预算、Permission、Model、Artifact、Intent/Effect、Evidence 和 Event 是横切对象，
不是第七类通用 Resource 表。

## Personal 2.0 Windows OPC 目标（`Requires-backend`）

目标保持一个 daemon，并组织：

```text
Owner
  -> Project -> Charter / Goal / Plan / Routine -> Task -> Attempt
  -> Role Blueprint -> Assignment -> Digital Employee
       -> managed DSH runtime
       -> Personal-owned Conversation and Memory
```

UI 是 Today / Projects / Knowledge、底部 Settings 与持久右侧会话。Team 与 Inbox
不是一级导航。Canvas v9 是冻结设计原型；文件名可保留
`personal-20-opc-e2e-optimized-v9`。产品源是 daemon `/ui/`。Dual Track L1 是
**Now / hypothesis chrome**。权威仍是 P11 walking skeleton。创建顺序为项目 → 流程 →
成员 → 分环测试 → 联调。单模块 PM/UI 维护入口：
[`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md)。
Pi 是 hidden、candidate-only 的 Assistant engine。DSH 是隐藏托管 Member 执行引擎
（不是可见 Installed Agent），采用 exact audited artifact、isolated
child、stdio broker 与 daemon Provider proxy。Conversation、archive/index/retrieval、
Memory、Task、Effect 与 completion 属于 Personal。

Knowledge 分开 Owner-shared source、Project Markdown Vault 与 employee-private Memory。
Routine 支持 manual/schedule/qualified-event Trigger、no-overlap、queue-latest 与 visible
missed work。HITL 只在画布。Provider binding 按 global→Project→employee→Task 解析；
actual usage 与 Provider quota 分离；未知费用不得显示为 0；成员级预算属 2.1 / Deferred。

Dual Track L1 chrome 在 daemon `/ui/` 上是 **Now / hypothesis chrome**。权威仍是
P11 walking skeleton：无权威则诚实 empty / Requires-backend；零假 Create /
Activate / Approve。Windows host/DSH qualification 仍是 `Requires-environment`。
MCP 保留为 advanced deferred seventh family；native mobile/E2EE relay remote 属于 2.1；
future Agent 需要独立 qualification。NVDA/200%/host-theme 仍 `not-run`。`P11-T15`
独立 / not-started。

## 一次交互如何流动

1. 先配置 Provider 和 selected model；key 只进入批准的操作系统 secret store。
2. Pi 通过 daemon Provider 代理发送受限对话请求。Pi 看不到 key，也不能通过原生
   shell 或文件工具绕过 daemon。
3. Task 只有在 daemon 记录 intent、边界、预算和 runnable 状态后才会准入。
4. 受治理 Tool 操作会在外部派发前创建 Intent/Effect；daemon 在 fencing 下对账结果。
5. verifier 读取持久状态和证据，之后 authority 才能把 Task 推进到完成。

## 今天应该如何理解它

当前已实现并测试的基础包括 daemon、CLI、secret、Provider 代理、Pi 对话、六类权威
存储、Task 准入以及资源/任务投影。`GET /management/usage` 现返回带标签费用
（`actual` | `estimated` | `unknown`；unknown 绝不为 `0`）、四层 binding 说明
（缺失的 Project/employee/Task 层为 unbound），以及分开的账户与配额字段。Settings
用量 chrome 仍是 T13。端到端完全自主执行仍为 `partial`；请使用 Task
watch 和 evidence 命令观察持久事实，不要把一次对话回答理解成 Task 已完成。

接下来按[快速上手](getting-started.md)走最短路径；精确命令见[CLI 基础](cli-basics.md)
和[参考手册](../reference/README.md)。
