# V01 Auto — Verify + Regress（Batch-C）

> 粘贴到干净 worktree 的新 Cursor Agent 会话。工作目录 = 仓库根。

---

你是 CognitiveOS 工程代理。接入：`AGENTS.md` → `PROGRESS.md` → [V01-AUTO-RUN-VERIFY-PERF-PLAN.md](../plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)。对照 `.github/workflows/ci.yml` honesty pins。

## 目标

完善/验证编排器 **WP-VERIFY**：

1. `pnpm run check:consistency`
2. `cargo run --locked -p cognitive-conformance --bin conformance-runner`
3. pins 精确匹配：`summary` = `{84, pass:55, fail:0, not-applicable:0, documented-degradation:0, not-run:29}`
4. `--self-check`：`must_flip.length ≥ 36` 且 corrupted 全翻 fail
5. REGRESS-V01：`MGMT-APPROVAL-R1-009` / `SELF-010` / `FATIGUE-011`；`AGENT-INSTALL-001` / `BYPASS-002` / `OOB-001` 均为 `pass`
6. F-017：`cargo test -p cognitive-runtime --lib sandbox::tests`
7. 失败 → 流水线停止；不得仍报 L2 green

## 禁止

- 改负例 expected / 降 pins 地板；把 not-run 写成 pass；扩 F-017 声明集

## DoD

- L2 Verify-green 可复现；summary 分区诚实；handoff + 逐路径 commit
