---
doc_id: user.index
locale: zh-CN
kind: navigation
audience: [user]
generated: false
---

# 用户指南

CognitiveOS Personal 是本地单用户的**认知资源操作系统**：一个 Rust daemon 统一治理你
的 AI agent 知道什么（Memory）、可复用什么（Skill）、可做什么（Tool）、看到什么
（Context）、在做什么（Task）、以什么身份运行（Runtime/Process）。本指南只记录代码、
合同与测试共同支持的行为；每页都带能力标签（`implemented`、`partial`、`designed`、
`unavailable`）。

从这里开始：

1. [Personal 是什么（不是什么）](what-is-personal.md)
2. [快速上手](getting-started.md) —— Linux 上的最短支持路径
3. [安装并到达首次对话](install-and-first-conversation.md)
4. [CLI 基础](cli-basics.md) —— `cognitive init | status | doctor | daemon | pi | resource | task`
5. [Provider 与 secret](provider-and-secrets.md)
6. [Provider Control Plane](provider-control-plane.md) —— 命名账户、密钥、binding、用量（仅 CLI/daemon；无 Web 面板）
7. [Pi 对话壳](pi-shell.md)

理解模型：

8. [系统总览](system-overview.md)
9. [六类资源](six-resources.md)
10. [Task 与执行](tasks-and-execution.md)

运维：

11. [运维与恢复](operations-and-recovery.md)
12. [安全边界](security-boundaries.md)
13. [已知限制](known-limitations.md)

精确的命令、路由、错误与文件参考见[参考手册](../reference/README.md)。项目当前状态由
[`docs/plan/PROGRESS.md`](../../../../docs/plan/PROGRESS.md) 拥有，本指南有意不复制。
