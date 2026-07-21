# M7 接续提示词：受治理记忆与认知发现

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = **干净 worktree / 仓库根**，基于 `origin/main`）。自包含，不依赖历史对话。
>
> **权威计划**：[docs/plan/M7-PLAN.md](../plan/M7-PLAN.md)。本提示词是执行入口，不以本文件覆盖计划验收矩阵。

---

你是 CognitiveOS 参考实现的工程代理。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**基线硬要求**：

1. `git fetch origin main`；确认 tip 至少含 v0.1 GO-with-explicit-non-claim（`f933d3c` 一带或更新 tip）；pins **pass 55 / not-run 29 / self-check ≥36**（以实测为准）。
2. 在干净 worktree 建车道分支；**禁止**从含 `personal-blog/**` 的本地 dirty `main` 推送。
3. 排除：`personal-blog/**`、`History/`、clients/Console 产品实现。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` 与 `docs/plan/M7-PLAN.md` → ③ 读最近 handoff（含 [20260721-v01-rereview.md](../checkpoints/20260721-v01-rereview.md) / [20260721-m7-planning-handoff.md](../checkpoints/20260721-m7-planning-handoff.md)），对照 `docs/plan/PARALLEL-LANES.md`。

**硬纪律**：确定性边界；四类状态用语；测试先行；schema-valid ≠ behavior-pass；**existing-only**（禁新增 Profile/REQ 域/对象族）；禁改写负例；禁跨平台合并 sandbox 声明；禁静默升级 v0.1 non-claim。

**会话结束**：更新 PROGRESS → 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（见 M7-PLAN）

- MemoryCandidate 准入 / 异步 admission + read-your-write / 生命周期晋升与失效（F-019 行为侧；IMP-03）。
- Discovery delta / 停滞 / discover-read 分离负例（含 `DISC-DELTA-SCOPE-003`）。
- REQ-PERF-005：**默认继续 non-claim**（见 `m7-perf005-decision.md`）。
- v0.1 残留（InstallationStore / PERF-004 campaign / D-018 / F-017 扩声明）**隔离 tracking**，不混仓。

## 禁止越界

R2/R3、M8+、Console/clients 实现、Windows-native sandbox 宣称、改写负例、扩 F-017、借 M7 新增对象族/Profile/REQ 域。

## 分批入口（勿一次做完）

| 批次 | 提示词 | 车道 |
|---|---|---|
| 0A 入口与范围 | [m7-scope-and-entry.md](m7-scope-and-entry.md) | DOC |
| 0B Memory 失败测试 | [m7-memory-contract-and-failing-tests.md](m7-memory-contract-and-failing-tests.md) | KRN + CFR |
| Memory runtime slice | [m7-memory-runtime-slice.md](m7-memory-runtime-slice.md) | KRN |
| Discovery | [m7-discovery-delta-and-stagnation.md](m7-discovery-delta-and-stagnation.md) | RUN + KRN |
| CFR 向量 | [m7-cfr-vectors-and-self-check.md](m7-cfr-vectors-and-self-check.md) | CFR |
| PERF-005 裁决 | [m7-perf005-decision.md](m7-perf005-decision.md) | DOC + CFR |
| 出口评审 | [m7-exit-review.md](m7-exit-review.md) | DOC |
