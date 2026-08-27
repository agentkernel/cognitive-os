---
doc_id: user.index
locale: zh-CN
kind: navigation
audience: [user]
generated: false
---

# 用户指南

CognitiveOS Personal 是跨平台本地、单一 owner 的 Agent、账户、认知资源与受治理工作
管理产品。一个 Rust daemon 统一治理 Agent 知道什么（Memory）、可复用什么（Skill）、
可做什么（Tool）、看到什么（Context）、在做什么（Task）、以什么身份运行
（Runtime/Process）。这是当前 Linux 1.0/API 的六族模型。本指南把代码、合同与测试共同
支持的当前行为（`implemented`、`partial`、`designed`、`unavailable`）与仍需 backend
或 core 工作的 Personal 2.0 完整版本承诺（`Requires-backend`、`Requires-core`）分开。

Personal 2.0 要求各自独立资格化的 Windows、macOS、Linux 本地路径；精确 Pi、DeepSeek
Harness Developer Preview 与受官方平台限制的 Codex desktop 路径；嵌入式 Agent 对话；
Account Hub；MCP 第七族；Goal/Plan/Task/Attempt 与多 Agent 监督；统一 Activity 与联邦
资源。当前同源 `/ui/` 客户端真实存在于 `clients/pc/web/`；这些目标增量尚未实现。

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
