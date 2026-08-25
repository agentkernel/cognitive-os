---
doc_id: ai.entry
locale: zh-CN
kind: navigation
audience: [ai]
generated: false
---

# AI 入口

为在本仓库工作的 AI 编程工具（Cursor、Claude Code、Codex 等）提供的紧凑导引。编辑任何
内容前先读完这五页：

1. [事实源优先级](source-of-truth.md) —— 来源冲突时以谁为准，哪些事实绝不能凭记忆
   复述。
2. [代码地图](code-map.md) —— 每个 crate、app、package 的真实职责与调用链。
3. [安全编辑边界](safe-editing.md) —— 不可变公理、受保护目录、lease 规则，以及绝对
   禁止的改动。
4. [验证命令](validation-commands.md) —— 各平台本地可运行什么、什么必须路由到 CI 或
   native Linux。
5. [文档影响](docs-impact.md) —— 代码变更何时必须在同一 PR 内同步手册或旧文档。

机器可读伴侣：[`personal/handbook/_meta/manifest.json`](../../_meta/manifest.json)（文档清单）、
[`personal/handbook/_meta/source-map.json`](../../_meta/source-map.json)（变更 → 文档路由）、
[`personal/handbook/_meta/source-coverage.json`](../../_meta/source-coverage.json)（全树分类），
以及仓库根 [`llms.txt`](../../../../llms.txt)。

任何摘要都不能弱化的硬规则：Rust daemon 是唯一权威写入者；概率组件只产 candidate；
secret 绝不进入 argv、配置、SQLite、日志、测试或证据；绝不读取或引用 `History/`；绝不
复制 `docs/plan/PROGRESS.md` 的动态状态。
