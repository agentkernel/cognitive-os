# 20260721 Post-v0.1 Next Phase Planning Handoff

## 1. 本次会话完成

- **规划会话交付（docs-only）**：落地 Post-v0.1 / Post-L3 下一阶段可执行任务计划，**未**执行代码战役、**未**改向量/pins、**未**跑 PERF。
- Canonical 计划：[docs/plan/POST-V01-NEXT-PHASE-PLAN.md](../plan/POST-V01-NEXT-PHASE-PLAN.md)（A–D：目标边界、候选排序、主战役工作包、文档联动与提示词指针）。
- 下一窗口执行提示词：[docs/prompts/cfr-m5-intent-authority-slice.md](../prompts/cfr-m5-intent-authority-slice.md)。
- 规划入口提示词（可复用）：[docs/prompts/post-v01-next-phase-planning.md](../prompts/post-v01-next-phase-planning.md)。
- 推荐**唯一**主战役：`CFR-M5-INTENT-AUTHORITY-SLICE`（Lane-CFR；建议分支 `lane/cfr-m5-intent-authority-slice`）。
- 目标向量（真实存在）：`INTENT-SUPERSEDE-002`（`REQ-INTENT-SUPERSEDE-001` 等）、`INTENT-ACCEPTANCE-007`（`REQ-INTENT-ACCEPT-001` 等）。
- 候选取舍已强制：P0 主战役；P1 shell-channel/target、HUMAN-CI-JOB-ADD、工具链债；P2 D-018 / InstallationStore / TSC HTTP / F-017 扩表；defer PERF、MGMT-FALLBACK 全动词、disk-full、migration/delta；Console tracking-only。
- 旁路 dirty（skills、职校选型、artifacts/_local 等）**未**暂存、未提交。

## 2. 未完成 / 进行中

- **未执行**主战役代码/runner 行为接入（留给下一 CFR 窗口）。
- pins 仍为规划基线 **84 / 55 pass / 29 not-run**；self-check ≥36；Profile implemented = 0。
- 继承全部 v0.1 explicit non-claims（Win-native sandbox、WSL2 sandbox 扩声明、durable install、PERF-004/005、D-018 residual、Console/clients/M7+）。

## 3. 测试与证据状态

| 项 | 结果 |
|---|---|
| 本会话测试/战役 | **未跑**（规划-only 落盘） |
| pins（继承 tip） | 84 / 55 / 29；self-check ≥36 |
| Profile implemented | **0** |
| L3 证据（既有） | run `20260721-192142-492`；`platform_label=windows_wsl2_linux_guest` |

## 4. 未决风险与漂移

- 无新 F/IMP/D 登记；无规范资产语义变更。
- 主战役执行时若 vector expected 与实现冲突：先登记漂移，禁止改 expected 迎合。
- `MGMT-FALLBACK-008` 仍缺 gateway/diagnostics.configure → 保持 defer。
- tip `kernel-server` 仅 `--once --bind`；禁止发明 `--data-dir` / `/health` / `/ready`。

## 5. 下一步入口

- 建议提示词：`docs/prompts/cfr-m5-intent-authority-slice.md`
- 工作分支：`lane/cfr-m5-intent-authority-slice`（由下一 CFR 会话创建）
- 第一个动作：`git status --short --branch` → 接入三步 → 只读核对目标 vector/registry/matrix → 先失败测试再接入 runner

## 6. 快照

- PROGRESS 已更新：是
- 工作分支（本规划落盘）：`lane/doc-post-v01-next-phase`
- 本次提交：`dd5458f`（`docs(plan): land Post-v0.1 next-phase plan and CFR slice prompt`）
- 关联文档条目：`POST-V01-NEXT-PHASE-PLAN`（无新 REQ；规划文档联动）
