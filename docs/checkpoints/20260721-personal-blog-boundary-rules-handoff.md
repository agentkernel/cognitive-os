---
title: personal-blog boundary rules (anti-wipe / anti-scatter)
date: 2026-07-21
lane: Lane-DOC
status: complete
---

# Handoff — personal-blog 边界规则补强

## 已完成

- 新增 alwaysApply 规则 [`.cursor/rules/19-personal-blog-boundary.mdc`](../../.cursor/rules/19-personal-blog-boundary.mdc)：唯一路径、固定远程 `agentkernel/blog`、嵌套 git + 根 ignore、破坏性操作前保全、禁 `D:\blog-*` 散落、禁混入 Cos 推送。
- 联动：`AGENTS.md` 目录地图/红线；`02-workflow-docs-sync` 开工检查；`18-auto-commit` 禁毁；`PARALLEL-LANES` §2.7；`PROGRESS` 隔离子工程行；`docs/_local/README.md`；blog 仓 `README.md` 本机路径说明。

## 根因（本次要防的）

既有纪律只覆盖「勿推 personal-blog / 勿用 dirty blog 基线推 Cos」，未覆盖：

1. Cos `reset`/`clean`/删目录会物理毁掉嵌套仓（ignore 挡不住删除）。
2. 多份 `D:\blog-*` / `agentkernel-blog` 平行副本导致路径漂移。
3. 远程与 canonical 工作副本未写成硬规则。

## 未完成 / 未决

- Cos 侧本批规则改动尚未提交（待所有者明示）。
- blog 仓内 `docs/notes/`（职校报告等）仍为嵌套仓未跟踪文件；需在 blog 仓单独处理。
- `git fetch` 对 `agentkernel/blog` 曾 403：本机 GitHub 鉴权需所有者配置，与规则无关。

## 验证

- `git check-ignore -v personal-blog` → `.gitignore` `/personal-blog/`
- `git -C personal-blog remote get-url origin` → `https://github.com/agentkernel/blog.git`

## 下一步

- 所有者审阅后提交 Cos 文档/规则批（docs-only）。
- 需要时在 blog 嵌套仓提交 `docs/notes/` 与 README 路径说明。
