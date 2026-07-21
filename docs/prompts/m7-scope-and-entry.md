# M7 Batch-0A：入口证据与范围冻结

读 `AGENTS.md` → `docs/plan/PROGRESS.md` → `docs/plan/M7-PLAN.md` → `docs/checkpoints/20260721-m7-planning-handoff.md`。

## 范围

1. 干净 worktree ← `origin/main`（禁止 dirty / `personal-blog/**` 基线）。
2. 实测：`git fetch origin main`；`git rev-parse origin/main`；`gh run list --commit … --limit 5`；`git status --short --branch`。
3. 实测 pins / self-check / `pnpm run check:consistency` / `git diff --check`；更新 handoff 入口句（勿抄旧数）。
4. 冻结 M7 REQ/schema/vector inventory（existing-only）；确认无开放 PR 阻塞。

## 禁止

改向量负例；新增 Profile/REQ 域/对象族；Console/clients 实现；扩 F-017 声明；实现 Memory/Discovery 代码（本批仅文档/入口）。

## 出口

入口证据落入 handoff；下一会话：`m7-memory-contract-and-failing-tests.md`。
