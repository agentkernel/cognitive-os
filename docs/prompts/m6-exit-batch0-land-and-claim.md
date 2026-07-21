# M6-EXIT Batch-E0/E1：合入栈 + 声明冻结

读 `AGENTS.md` → `docs/plan/PROGRESS.md` → `docs/plan/M6-EXIT-PLAN.md` → 最近 handoff。

## 范围

1. 干净 worktree ← `origin/main`（禁止 dirty / `personal-blog/**` 基线）。
2. 确认 #31/#32 已合入；#34（或后继）CI 绿后合入。
3. 实测 pins / self-check；更新 PROGRESS 入口句。
4. WP-CLAIM：冻结 F-017 发布声明集（见 `docs/traceability/f017-platform-matrix.md`）。

## 禁止

改向量负例；新增 transition/readiness carrier；Console/clients 实现；把 WSL2 写成 Windows-native。

## 出口

声明集冻结表落地；下一会话入口：`m6-exit-batch1-f017-evidence.md`（若 digests 未齐）或 `m6-exit-batch3-rereview.md`。
