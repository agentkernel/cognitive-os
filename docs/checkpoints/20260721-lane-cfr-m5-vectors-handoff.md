# 20260721 Lane-CFR Handoff（M5 行为向量批：F-011 + shell cancel/detach/watch）

## 1. 本次会话完成

- **`behavior_m5.rs`**：6 向量脱 not-run → **pass**（真实调用 RUN 公开面，零改写向量）：
  - `MGMT-APPROVAL-R1-009` / `SELF-010` / `FATIGUE-011` → `ApprovalGate`
  - `SHELL-CANCEL-SEMANTICS-005` / `SHELL-DETACH-ATTACH-004` → `ShellService`
  - `SHELL-WATCH-RESUME-006` → `WatchLog`（`WATCH_CURSOR_STALE`）
- Runner：`ExecutionMode` +6、`CORRUPTED_MODES` 27→33、CI/runner pins **pass 52 / not-run 32**；self-check must_flip ≥33。
- 依赖：`cognitive-management` / `cognitive-runtime` / `cognitive-akp`（叶依赖，未改业务 crate）。
- **F-011 / IMP-05**：M5 行为部分闭合（非 Profile 声明）。

## 2. 未完成 / 进行中

仍 not-run（需更多 API 或诚实延期）：`MGMT-FALLBACK-008`（7 ops vs 4 verbs）、`INTENT-SUPERSEDE-002`、`INTENT-ACCEPTANCE-007`、`SHELL-CHANNEL-ISOLATION-003`、`SHELL-TARGET-AMBIGUITY-001`、`SHELL-EXECUTION-MIGRATION-009`、`DISC-DELTA-SCOPE-003`、store-degradation disk-full 等。

## 3. 测试与证据状态

- `cargo test -p cognitive-conformance`：runner_execution 10/10；workspace 相关绿。
- `conformance-runner`：84 | **pass 52** | fail 0 | **not-run 32**；self-check 33/33 flip。
- 状态用语＝行为执行已提供；**不构成 Profile 已符合**。

## 4. 未决风险与漂移

- 无新漂移。`SHELL-DETACH` 的 `watch_restored_from_cursor` 由 shell 非取消语义 + 客户端 cursor 保留联合说明（证据 note）。
- MGMT-FALLBACK 完整 7 动词可达性仍缺 gateway/diagnostics.configure。

## 5. 下一步入口

- M5 milestone review（DOC）→ M6 入口 gate。
- 建议提示词：`docs/prompts/milestone-m5.md` 出口评审 / `lane-cfr.md` 剩余向量。

## 6. 快照

- PROGRESS 已更新：是。
- 提交：behavior_m5 + CI pins → docs/ledger/handoff/milestone review。
