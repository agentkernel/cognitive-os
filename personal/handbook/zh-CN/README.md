---
doc_id: index
locale: zh-CN
kind: navigation
audience: [user, developer, ai]
generated: false
---

# CognitiveOS Personal 手册（中文）

一个让 Agent 工作可审计、有预算、可恢复的本地单 Owner 系统。本手册严格分开当前定稿
Linux 1.0/API 与已采纳的 Windows-first Personal 2.0 OPC target。仅目标行为标为
`Requires-backend`、`Requires-environment` 或 deferred。

**状态边界：** Linux 1.0 与当前 API 仍是六族模型，Pi 是唯一已资格化 Agent。当前同源
`/ui/` SPA 已存在于 `clients/pc/web/`。Personal 2.0 target 包含
Today/Projects/Team/Knowledge/Inbox、Project/Role/Employee/Routine/Attempt authority、
Personal-owned Conversation/Vault/Memory、Pi-backed Personal Assistant、preinstalled
managed DSH、global→Project→employee→Task Provider/budget 与固定 Windows acceptance。
MCP advanced/deferred；native mobile/E2E relay remote 属于 2.1。上述都不是 current
implementation 或 Windows/DSH support。

- **[用户指南](user/README.md)** —— 安装、首次对话、CLI、secret、Provider Control
  Plane、Pi 对话壳、资源模型、运维、安全、限制。
- **[开发者指南](developer/README.md)** —— 仓库地图、权威内核、存储、HTTP 面、执
  行链状态、各域、测试、工作流。
- **[参考手册](reference/README.md)** —— 生成的 CLI/HTTP/错误/配置/环境变量/状态
  机/schema/工具参考，以及能力与兼容性矩阵。
- **[AI 入口](ai/README.md)** —— 事实源优先级、代码地图、安全编辑、验证命令、文档
  影响。

English entry: [`personal/handbook/en/`](../en/README.md)。机器元数据：
[`personal/handbook/_meta/manifest.json`](../_meta/manifest.json)。项目动态状态由
[`docs/plan/PROGRESS.md`](../../../docs/plan/PROGRESS.md) 拥有，此处绝不复制。
