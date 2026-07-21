# V01 Auto — Boot + Connect（Batch-B）

> 粘贴到干净 worktree 的新 Cursor Agent 会话。工作目录 = 仓库根。

---

你是 CognitiveOS 工程代理。接入：`AGENTS.md` → `PROGRESS.md` → [V01-AUTO-RUN-VERIFY-PERF-PLAN.md](../plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)。

## 目标

完善/验证编排器 **WP-BOOT + WP-CONNECT-MGMT + WP-CONNECT-SHELL**：

1. 构建：`cargo build --workspace --locked`；`pnpm install --frozen-lockfile`；`pnpm -r build`
2. tip 启动面：**仅** `kernel-server --once --bind`（禁止发明 `--data-dir` / `/health`）
3. Mgmt：`cargo test -p admin-cli --test m5_deterministic_fallback`；`cargo test -p cognitive-management --test m5_fallback_verbs`
4. Shell：`KERNEL_SERVER_BIN=... pnpm --filter @cognitiveos/sdk-ts test`（http_live）；`cargo test -p kernel-server --test m5_http_sse`
5. bin 缺失导致 live skip → **auto_fail**（不得当 skip 放过）
6. `CONNECT-FULL-DEMO` / `CONNECT-WATCH` 保持 `skipped_nonclaim`

## 禁止

- Console/clients 实现；改向量；Windows-native sandbox pass；personal-blog 混入提交

## DoD

- `pnpm run verify:local` 至少达到 L1（或记录诚实失败）
- 证据在 `artifacts/evidence/v01-auto-run/<run_id>/`
- handoff + PROGRESS 更新；逐路径 commit
