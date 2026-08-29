---
doc_id: user.index
locale: zh-CN
kind: navigation
audience: [user]
generated: false
---

# 用户指南

CognitiveOS Personal 是本地、单一 Owner 的受治理 Agent 工作系统。一个 Rust daemon
拥有 authority。Memory、Skill、Tool、Context、Task、Runtime/Process 是当前 Linux
1.0/API 六族。本指南分开 current behavior 与 Windows-first OPC target
（`Requires-backend`、`Requires-environment` 或 deferred）。

Personal 2.0 target 包含 Today/Projects/Knowledge（Team 与 Inbox 不是一级导航）；Project、Role、
Digital Employee、Routine 与 preserved Attempt；Personal-owned Conversation/Vault/
Memory；Pi-backed Personal Assistant；preinstalled managed DSH；Provider/budget
hierarchy 与固定 Windows acceptance path。当前同源 `/ui/` 真实存在，但这些 OPC
增量尚未实现。owner 批准的交互原型是 `personal-20-opc-e2e-optimized-v5`
（Canvas 规格，不是 `/ui/`）。单模块维护入口：
[`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md)。

从这里开始：

1. [Personal 是什么（不是什么）](what-is-personal.md)
2. [快速上手](getting-started.md) —— Linux 上的最短支持路径
3. [安装并到达首次对话](install-and-first-conversation.md)
4. [CLI 基础](cli-basics.md) —— `cognitive init | status | doctor | daemon | pi | resource | task`
5. [Provider 与 secret](provider-and-secrets.md)
6. [Provider Control Plane](provider-control-plane.md) —— 当前命名账户、密钥、binding、用量与同源 `/ui/`；已采纳 Account Hub 目标
7. [Pi 对话壳](pi-shell.md)

理解模型：

8. [系统总览](system-overview.md)
9. [当前六族与目标第七 MCP 族](six-resources.md)
10. [Task 与执行](tasks-and-execution.md)

运维：

11. [运维与恢复](operations-and-recovery.md)
12. [安全边界](security-boundaries.md)
13. [已知限制](known-limitations.md)
14. [Linux RC 操作地图](rc-and-support.md) —— 安装/初始化/Provider/Pi/Task/恢复/更新/卸载索引

精确的命令、路由、错误与文件参考见[参考手册](../reference/README.md)。项目当前状态由
[`docs/plan/PROGRESS.md`](../../../../docs/plan/PROGRESS.md) 拥有，本指南有意不复制。
