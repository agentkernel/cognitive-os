# 20260721 Lane-CFR M5 Intent Authority Handoff

## 1. 本次会话完成

- **战役** `CFR-M5-INTENT-AUTHORITY-SLICE`（Lane-CFR；分支 `lane/cfr-m5-intent-authority-slice`）。
- **行为执行脱 not-run（2）**：
  - `INTENT-SUPERSEDE-002`（`REQ-INTENT-SUPERSEDE-001` / `REQ-SHELL-CORRECTION-001` / `REQ-AKP-INTENT-001`）→ `IntentSupersedeBehavior`
  - `INTENT-ACCEPTANCE-007`（`REQ-INTENT-ACCEPT-001` / `REQ-SHELL-STATUS-001`）→ `IntentAcceptanceBehavior`
- **实现**：`crates/cognitive-conformance/src/exec/behavior_m5_intent.rs`；`exec.rs` 分类/dispatch/`CORRUPTED_MODES`；CI honesty pins **57/27**；self-check **38**。
- **只读消费** KRN/store 公开 API（`supersede_task_contract` / `mint_intent` / `dispatch_effect` / `TransitionEngine`）；**未**改 vector expected、REQ/schema/transition；**未**跨车道改 KRN/RUN 业务实现。
- **证据**：runner `conformance-report.json` sha256 `fcd53f14228e0cb005fa7410885bd1917d0541ae266efe5e020228b6d30852e5`；self-check sha256 `37120db6de024149a9d19a8e70292c275e16c2d0ef49119946a803bee0d6b7f9`（gitignored `artifacts/evidence/conformance/`）。
- **文档联动**：`docs/traceability/matrix.yaml`（两 REQ impl/impl_tests/evidence/notes）、`PROGRESS.md`、`PARALLEL-LANES.md`、`POST-V01-NEXT-PHASE-PLAN.md` 状态回填。

## 2. 未完成 / 进行中

- PR 合并与 CI 双 OS 观察（本 handoff 写于提交前；合并后以 CI 为准）。
- P1 后续候选未执行：`SHELL-CHANNEL-ISOLATION-003`、`SHELL-TARGET-AMBIGUITY-001`、HUMAN-CI-JOB-ADD、工具链债。
- 继承全部 v0.1 explicit non-claims（Win-native sandbox、WSL2 sandbox 扩声明、durable install、PERF-004/005、D-018 residual、Console/clients/M7+）。
- Profile implemented 仍为 **0**。

## 3. 测试与证据状态

| 项 | 结果 |
|---|---|
| `cargo test -p cognitive-conformance` | **12/12 pass**（含 `m5_intent_authority_vectors_execute_against_kernel_store`） |
| conformance-runner | **84 / pass 57 / not-run 27**；目标两向量 `pass` |
| `--self-check` | **must_flip 38 / flipped 38 / corrupted_but_still_passing=[]** |
| `pnpm run check:consistency` | **OK**（273/55/61/84） |
| Profile implemented | **0** |
| 平台 | WSL2 Linux guest 原生盘 `~/agent-kernel`（Windows GNU linker 缺 libgcc；按计划走 WSL） |

## 4. 未决风险与漂移

- **无新 F/IMP/D**；无规范资产语义变更。
- Supersede 行为用 epoch **1→2** 行使与向量声明 4→5 相同的 fencing 语义；`expected` 不钉 epoch 数值；证据记录 `vector_declared_epochs`。
- `INTENT-ACCEPTANCE-007` **不得**用 `GW-REMOTE-COMPLETE-001` 冒充；本批为独立 vector-specific adapter。
- 旁路 dirty（skills、职校选型、`artifacts/_local/**` 等）**未**暂存。

## 5. 下一步入口

- 建议：按 [POST-V01-NEXT-PHASE-PLAN.md](../plan/POST-V01-NEXT-PHASE-PLAN.md) §B P1 选壳 channel/target 或工具链债；勿批量清 not-run。
- 工作分支：`lane/cfr-m5-intent-authority-slice`（待 PR → main）。
- 第一个动作：CI 绿后合并；下一战役先 `git status` 保护 dirty。

## 6. 快照

- PROGRESS 已更新：是
- 工作分支：`lane/cfr-m5-intent-authority-slice`
- 关联 REQ：`REQ-INTENT-SUPERSEDE-001`、`REQ-INTENT-ACCEPT-001`（及向量附属 REQ）
- 提交哈希：见本 PR / `git log`（写 handoff 时尚未 commit）
