---
doc_id: user.system-overview
locale: zh-CN
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: docs/product/personal/README.md
  - path: docs/product/personal/cognitive-resource-model.md
  - path: docs/architecture/personal/README.md
  - path: apps/kernel-server/src/personal/resource_api.rs
  - path: apps/kernel-server/src/personal/task_api.rs
tests:
  - apps/kernel-server/tests/p2_t02_resource_projection.rs
  - apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - apps/kernel-server/tests/p2_t28_end_to_end_journey.rs
fingerprint: "sha256:4f72ff9d8badb8656fbe390643e814b996065842cf38f3daaa880f122a35e9f2"
non_claims:
  - 本页用于建立概念，不构成 release、Gate、Profile 或 agent 收益声明。
  - 调度器驱动的完全自主执行与独立验证仍为 partial；见 Task 与执行。
  - Linux 1.0 不声明 Windows 安装对等、Web UI、MCP/动态工具或多 Agent 编排。
---

# 系统总览

CognitiveOS Personal 是本地、单一 owner 的认知资源操作系统：为 agent
提供一个受治理的地方来保存记忆、加载 Skill、使用 Tool、组装 Context、
接纳 Task，并运行受管理的进程。产品由 Rust daemon 与客户端组成；daemon
拥有权威状态，`cognitive`、Pi 和 SDK 等客户端只能请求或提出操作。

## 一张图理解

```text
使用者 -> cognitive CLI 或 Pi Shell -> 本地 Rust daemon -> 六个领域服务
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

## 一次交互如何流动

1. 先配置 Provider 和 selected model；key 只进入批准的操作系统 secret store。
2. Pi 通过 daemon Provider 代理发送受限对话请求。Pi 看不到 key，也不能通过原生
   shell 或文件工具绕过 daemon。
3. Task 只有在 daemon 记录 intent、边界、预算和 runnable 状态后才会准入。
4. 受治理 Tool 操作会在外部派发前创建 Intent/Effect；daemon 在 fencing 下对账结果。
5. verifier 读取持久状态和证据，之后 authority 才能把 Task 推进到完成。

## 今天应该如何理解它

当前已实现并测试的基础包括 daemon、CLI、secret、Provider 代理、Pi 对话、六类权威
存储、Task 准入以及资源/任务投影。端到端完全自主执行仍为 `partial`；请使用 Task
watch 和 evidence 命令观察持久事实，不要把一次对话回答理解成 Task 已完成。

接下来按[快速上手](./getting-started.md)走最短路径；精确命令见[CLI 基础](./cli-basics.md)
和[参考手册](../reference/README.md)。
