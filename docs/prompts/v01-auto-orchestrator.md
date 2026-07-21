# V01 Auto — Orchestrator（Batch-A / 合成）

> 粘贴到干净 worktree 的新 Cursor Agent 会话。工作目录 = 仓库根。

---

你是 CognitiveOS 工程代理。接入：`AGENTS.md` → `PROGRESS.md` → [V01-AUTO-RUN-VERIFY-PERF-PLAN.md](../plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)。

## 目标

维护一键编排入口与证据布局：

1. `pnpm run verify:local` → `scripts/v01-auto-run-entry.mjs` → `.ps1` / `.sh`
2. 阶段门：Boot→Connect→Verify→PerfAuto→Summary；硬失败写 summary 后非零退出
3. 入口证据：`git fetch` tip、CI、worktree；禁止 personal-blog 基线
4. 平台探测 → F-017 标签；Windows 禁止 sandbox native pass
5. 输出 `artifacts/evidence/v01-auto-run/<run_id>/{summary.json,summary.md,sha256-manifest.json,...}`
6. CI 新 job = `HUMAN-CI-JOB-ADD`（默认仅本地）

## Tip 钉死命令（勿臆造）

见父计划 §Tip 启动面硬事实与 §B 各 WP 入口。

## 禁止

- 与改向量同 PR；Console 实现；静默宣称升格；dirty/`personal-blog/**` 推送

## DoD

- 单命令可达 L3 report-ready（non-claim）或诚实失败分级
- 文档/PROGRESS/handoff 联动；逐路径 commit；开 PR（docs+scripts）
