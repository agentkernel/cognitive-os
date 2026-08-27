---
doc_id: index
locale: zh-CN
kind: navigation
audience: [user, developer, ai]
generated: false
---

# CognitiveOS Personal 手册（中文）

本地单用户的认知资源操作系统：一个 Rust daemon 统一治理 AI agent 知道什么、可复用什
么、可做什么、看到什么、在做什么、以什么身份运行。本手册把当前实现事实与已采纳的
Personal 2.0 目标严格分开；仅目标行为一律标为 `Requires-backend` 或
`Requires-core`。

**状态边界：** Linux 1.0 与当前 API 仍是六族模型，Pi 是唯一已资格化 Agent。当前同源
`/ui/` SPA 已存在于 `clients/pc/web/`。Personal 2.0 已采纳桌面优先重设计、第七个
MCP 资源族、Account Hub 凭据导入、厂商专用 Agent 对话适配器、Goal/Plan 与多 Agent
监督，以及联邦资源；这些目标增量（包括现有 UI 的重设计）均尚未实现。

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
