# 20260721 V01 Auto-Run L3 Handoff

## 1. 本次会话完成

- **P0 目标达成**：在 WSL2 Linux guest（原生盘 `~/agent-kernel` 工作副本，tip 与 `lane/doc-v01-auto-run` 对齐）跑通 `pnpm run verify:local` → **L3 Perf-report-ready（non-claim）**
- Canonical 证据 run：**`20260721-192142-492`**
  - `level=L3`，`stopped=false`，`release=non_claim_preserved`，`profile_implemented=0`
  - `platform_label=windows_wsl2_linux_guest`（**不**冒充 Windows-native sandbox）
  - pins：84 / **55 pass** / **29 not-run**；self-check `must_flip=36`
  - VERIFY-PINS / VERIFY-SELFCHECK / REGRESS-V01 / F017-CLAIM-FREEZE = `auto_pass`
  - PERF004：`auto_pass` + `campaign=not_executed`；PERF005 + HUMAN-CI/PERF004/PERF005 = `skipped_nonclaim`
- 最小脚本硬化（docs/scripts only；**无** REQ/向量/pins 变更）：
  1. 编排器默认 `CI=true`（无 TTY 时避免 `ERR_PNPM_ABORTED_REMOVE_MODULES_DIR_NO_TTY`）
  2. `m5_http_sse` 以 `--test-threads=1` 运行（降低 sdk-ts live 后并行 `--once` 偶发 `ConnectionReset`）
- 旁路 dirty（`.cursor/skills/**`、职校选型 md/xlsx、`artifacts/_local/**`）**未**暂存、未提交
- 关联计划：[V01-AUTO-RUN-VERIFY-PERF-PLAN.md](../plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)；v0.1 [GO-with-explicit-non-claim](20260721-v01-rereview.md)

## 2. 未完成 / 进行中

- PR [#37](https://github.com/agentkernel/cognitive-os/pull/37) 合并状态：以本 handoff 推送后 CI 结论为准（见 §6 合并记录）
- `HUMAN-CI-JOB-ADD`：仍未新增独立 GitHub Actions job（默认仅本地）
- PERF-004 全 HW campaign / PERF-005 benefit：保持 non-claim（**未**执行 `V01-PERF-CAMPAIGN-PLAN` 升格）
- Windows-native host：`x86_64-pc-windows-gnu` 缺 mingw/libgcc 时 BOOT 诚实 L0 fail 仍可预期；本 L3 **不是** Win-native sandbox 证据

## 3. 测试与证据状态

| 项 | 结果 |
|---|---|
| ENTRY HEAD | `693c069`（脚本硬化提交后以 push tip 为准） |
| origin/main tip（入口） | `f933d3c`；CI run `29801983501` success |
| L3 summary | gitignored：`artifacts/evidence/v01-auto-run/20260721-192142-492/`（及工作树副本 `…-wsl2-full/`） |
| pins | 84 / 55 / 29；self-check ≥36 |
| Profile implemented | **0**（auto green ≠ Profile implemented） |
| `check:consistency` | 提交前再跑 |

### summary.json 关键字段摘录

```json
{
  "run_id": "20260721-192142-492",
  "level": "L3",
  "stopped": false,
  "release": "non_claim_preserved",
  "profile_implemented": 0,
  "platform_label": "windows_wsl2_linux_guest"
}
```

## 4. 未决风险与漂移

- 继承 v0.1 explicit non-claims（仍全部有效）：
  1. Windows-native sandbox = unsupported
  2. WSL2 guest sandbox = not_tested（本 run 仅为 Linux guest 编排绿，**不**扩 F-017 声明集）
  3. Durable install = in-process ledger only
  4. REQ-PERF-004 full HW campaign = not executed
  5. REQ-PERF-005 agent benefit = not emitted
  6. Profile implemented = 0
  7. D-018 governance object ports residual
  8. Console / clients / Agent Hub / M7+ 不在范围
- `/mnt/d`（9p）上直接跑可能更慢且 CONNECT 更易抖；推荐 WSL 原生盘工作副本 + 同步 tip（与既有 bootstrap 一致）
- tip `kernel-server` 仅 `--once --bind`；禁止发明 `--data-dir` / `/health` / `/ready`

## 5. 下一步入口

- **本战役收束**：合并 PR #37 后停止；**不要**开启 PERF 战役或清空 29 not-run
- 若需窄面 P1：仅在合并后另开会话；默认仍 `docs/prompts/v01-auto-orchestrator.md` 只读复跑，不升格
- 工作分支：`lane/doc-v01-auto-run`（合并后可删）

## 6. 快照

- PROGRESS 已更新：是
- PR：https://github.com/agentkernel/cognitive-os/pull/37 — **MERGED**
- 分支 tip（合并前）：`101d71c`（`docs(v01-auto-run): close P0 with L3 non-claim evidence`）
- **merge commit**：`5b5401a26fec3a2f7108cdfb4f542d917c32cc0b`（2026-07-21T11:34:01Z）
- CI（合并前）：push `29826034378` + PR `29826038420` — ubuntu/windows verify SUCCESS
- 本次提交列表：`0fb0c59` → `693c069` → `101d71c` → merge `5b5401a`
