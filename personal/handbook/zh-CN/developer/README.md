---
doc_id: dev.index
locale: zh-CN
kind: navigation
audience: [developer]
generated: false
---

# 开发者指南

实现的真实运作方式，逐文件映射到源码与测试。能力标签是诚实的：`partial` 页面会精确说
明缺少哪段接线。

阅读任何 Personal 2.0 陈述时都要对照当前边界：当前 Linux/API 组合为六族、只有 Pi
资格化，且已有位于 `clients/pc/web/` 的同源 `/ui/` SPA。Personal 2.0 是
Windows-first OPC target，包含 Project/Role/Employee/Routine/Attempt authority、
Personal-owned Conversation/Vault/Memory、hidden Pi Assistant engine、preinstalled
managed DSH child、Provider/budget hierarchy 与 OPC UI。每个缺失项都是
`Requires-backend`/`Requires-environment`，不是实现事实。MCP advanced/deferred；
native mobile/E2E relay remote 属于 2.1。

导引：

1. [仓库地图](repository-map.md)
2. [架构总览](architecture-overview.md) —— 目标设计 vs 当前组合
3. [开发环境](development-environments.md) —— 什么在哪运行
4. [贡献工作流](contributing-workflow.md) —— lease、分支、CI、文档联动

权威内核：

5. [权威内核](authority-kernel.md) —— 转移门、intent chain、预算、恢复
6. [存储与迁移](store-and-migrations.md) —— SQLite 布局 v1–v25
7. [Task 流水线](task-pipeline.md) —— record → interpret → preview → admit → watch
8. [执行链状态](execution-chain-status.md) —— 什么已接线、什么没有

域与表面：

9. [daemon 与 HTTP](daemon-and-http.md)
10. [Context 与 Artifact](context-and-artifact.md)
11. [Memory 与 Skill](memory-and-skill.md)
12. [Agent 与 Pi 生命周期](agent-and-pi-lifecycle.md)
13. [安装器与服务](installer-and-service.md)
14. [管理面](management-plane.md)
15. [TypeScript 客户端](typescript-clients.md)
16. [合同与代码生成](contracts-and-codegen.md)
17. [符合性与测试](conformance-and-testing.md)
18. [性能面](performance-surfaces.md)

机器参考（生成）：[参考手册](../reference/README.md)。
