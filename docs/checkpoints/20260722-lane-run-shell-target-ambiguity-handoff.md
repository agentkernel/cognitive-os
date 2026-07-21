# 20260722 Lane-RUN Shell Target Ambiguity Authority Handoff

## 1. 本次会话完成

- **战役** `RUN-SHELL-TARGET-AMBIGUITY-AUTHORITY-THEN-CFR` **阶段 A（Lane-RUN）**。
- 新增确定性 authority 门禁：`crates/cognitive-runtime/src/target_resolution.rs`
  - `admit_target_selector` / `request_from_target_vector_input` / `is_strong_reference`
  - selector `"stop it"` + `visible_candidates=["execution://a","execution://b"]`
    → `clarification_required` + `SHELL_TARGET_AMBIGUOUS`（category=shell）+ `dispatch=false`
  - 零 top-1 猜测；概率组件零参与
- 负例/正例单元测试 4 项（`shell_target_*`）全绿。
- **未**改 pins（保持 **84 / 58 / 26**）；**未**接 CFR 行为向量；单元绿 ≠ vector pass。

## 2. 未完成 / 进行中

- 阶段 B（Lane-CFR）：`SHELL-TARGET-AMBIGUITY-001` 行为执行脱 not-run（须等本批合入 main 后 rebase）。
- 继承 v0.1 explicit non-claims（Win-native、WSL2 sandbox 扩表、durable install、PERF、D-018 residual、Console/clients/M7+）。

## 3. 测试与证据状态

| 项 | 结果 |
|---|---|
| `cargo test -p cognitive-runtime` | **36 pass**（含 4 项 shell target；WSL 原生盘 `~/agent-kernel`） |
| `cargo clippy -p cognitive-runtime --all-targets -- -D warnings` | **pass** |
| pins | **未变** 84 / 58 / 26 |
| Profile implemented | **0** |

## 4. 未决风险与漂移

- 无新 F/IMP/D；无 vector/REQ/schema 变更。
- Windows GNU linker 缺 libgcc — 本批验证走 WSL（旁路调试债，非产品能力）。
- 旁路 dirty（skills / 职校 / `artifacts/_local`）未暂存。
- PR #36（M7 plan）隔离，未混写。

## 5. 下一步入口

- 合入本 PR 后：开 `lane/cfr-shell-target-ambiguity`，消费 `admit_target_selector` 接入 `SHELL-TARGET-AMBIGUITY-001`。
- 第一个动作：`git fetch origin main && git checkout main && git pull`；新开 CFR 分支。

## 6. 快照

- PROGRESS 已更新：是（本批）
- 工作分支：`lane/run-shell-target-ambiguity-authority`
- 关联 REQ：`REQ-SHELL-TARGET-001`、`REQ-SHELL-AMBIGUITY-001`；错误码 `SHELL_TARGET_AMBIGUOUS`
