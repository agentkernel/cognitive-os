# 20260721 V01 Auto-Run Handoff

## 1. 本次会话完成

- Canonical 计划：[docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md](../plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)（A–G）
- 战役附录：[docs/plan/V01-PERF-CAMPAIGN-PLAN.md](../plan/V01-PERF-CAMPAIGN-PLAN.md)（默认不执行）
- 一键入口：`pnpm run verify:local` → `scripts/v01-auto-run-entry.mjs` → `scripts/v01-auto-run.ps1` / `.sh`
- Summary 模板：[docs/plan/templates/v01-auto-run-summary.schema.json](../plan/templates/v01-auto-run-summary.schema.json)
- 提示词：`docs/prompts/v01-auto-{boot-connect,verify-regress,perf-report,orchestrator}.md`
- 证据布局说明更新：`artifacts/evidence/README.md`；`docs/README.md`；`AGENTS.md`；`PROGRESS.md`
- 关联：v0.1 GO-with-explicit-non-claim；REQ-PERF-004 sample；F-017 / F-026 / IMP-04 / IMP-18 non-claim 继承；**无**向量/pins 变更
- 编排器冒烟：入口/平台探测/`summary.json` 分区与人闸门默认 skip 已写通；本机 `x86_64-pc-windows-gnu` 缺 mingw `libgcc` → BOOT `cargo build` exit 101 → 诚实停在 L0（`release=blocked`）。**不以本机 L0 失败冒充 CI 绿**；tip CI `29801983501` 仍为入口参考。

## 2. 未完成 / 进行中

- 在具备 MSVC/Linux CI 等价工具链的干净环境重跑至 L3 report-ready
- `HUMAN-CI-JOB-ADD`：未新增独立 GitHub Actions job（默认仅本地）
- PERF-004 全 HW campaign / PERF-005 benefit：保持 non-claim

## 3. 测试与证据状态

- CI tip（入口）：`origin/main` `f933d3c`；run `29801983501` success；pins 55/29；self-check ≥36（未改 pins）
- `pnpm run check:consistency`：OK（273/55/61/84）
- 本地 auto-run 样例：`artifacts/evidence/v01-auto-run/20260721-153827-8922901b/`（L0 BOOT fail；gitignored）
- Profile implemented：仍为 0

## 4. 未决风险与漂移

- tip `kernel-server` 仅 `--once`：编排已按此钉死
- Windows host：sandbox 相关保持 unsupported/skip
- 本机 gnu 链接器缺口：编排正确 fail-closed；修复工具链后应能继续 Connect/Verify/Perf
- 工作区旁路 dirty（`.cursor/skills/**` 等）不得混入本 PR

## 5. 下一步入口

- 建议提示词：`docs/prompts/v01-auto-orchestrator.md`
- 工作分支：`lane/doc-v01-auto-run`
- 第一个动作：干净 worktree + 可用工具链上 `pnpm run verify:local`；确认 `summary.json` level=L3 且 non-claim；开 PR（docs+scripts only）

## 6. 快照

- PROGRESS 已更新：是
- 本次提交列表：见本分支 PR
