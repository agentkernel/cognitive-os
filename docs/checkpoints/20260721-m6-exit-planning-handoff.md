# 20260721 M6-EXIT Planning / Claim-Freeze Handoff

## 1. 本次会话完成

- Canonical 出口计划：[docs/plan/M6-EXIT-PLAN.md](../plan/M6-EXIT-PLAN.md)（A–G）
- WP0-LAND：#31/#32 merged；#33 CI 红 → clippy/rustfmt 修复 → 超集为 [#34](https://github.com/agentkernel/cognitive-os/pull/34)（`lane/run-m6-installer-ci`）；#33 closed superseded
- WP-CLAIM + WP-F017：冻结发布声明集；linux_native 三 deny digests + 复现命令；unit `f017_claim_freeze_digests_are_stable`
- 默认 non-claim：InstallationStore、PERF HW 战役、D-018 交换面
- WP-REVIEW：[20260721-v01-rereview.md](20260721-v01-rereview.md) → **GO-with-explicit-non-claim**
- 分批提示词：`docs/prompts/m6-exit-batch0..3*.md`
- 联动：PROGRESS、M6-PLAN 指针、docs/README、findings-ledger F-017、f017-platform-matrix

## 2. 未完成 / 进行中

- #34 须 CI 绿后合入 `main`（本 handoff 写作时以 PR 状态为准）
- 合入后重测 `gh run list --commit $(git rev-parse origin/main)` 与 pins
- 可选后续：InstallationStore / PERF 战役 / D-018 闭合（见 batch2；默认不触发）

## 3. 测试与证据状态

- CI：#34 待绿；本地 Windows mingw linker 不可靠，以 CI 为准
- 向量：pins 目标 55/29；self-check ≥36（M6 交付已钉；合入后复核）
- F-017：closed-for-release-claim-set；digests `sha256:evidence-{network,secrets,tool_proxy}`

## 4. 未决风险与漂移

- 扩大 F-017 声明集未补 digest → 立即 reopen / NO-GO
- D-018 / durable install / PERF 仍为 explicit non-claim
- 禁止从含 personal-blog 的 dirty main 推送

## 5. 下一步入口

- 合入 #34 后：干净 worktree sync main；若需战役则 `m6-exit-batch2-optional-store-perf.md`
- 否则发布笔记引用 v01-rereview non-claim 列表

## 6. 快照

- PROGRESS 已更新：是
- 分支：`lane/doc-m6-exit`（叠在 #34 tip 上；合入序：先 #34 再本 docs PR，或同 PR 若 #34 未合）
