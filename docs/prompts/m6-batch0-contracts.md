# M6 Batch-0A：合同消费准备与冻结裁决（Lane-CTR）

> 用法：粘贴到干净 worktree 的新 Cursor Agent 会话。工作目录 = 仓库根。本批 **只改契约生成与文档裁决**，不实现 installer/sandbox。

---

你是 CognitiveOS Lane-CTR 工程代理。开工前：`git fetch origin main`；`git status`；保护未提交改动；逐路径 `git add`，禁 `git add -A`。从最新 `origin/main` 建分支 `lane/ctr-m6-bindings`（干净 worktree）。

**接入**：`AGENTS.md` → `docs/plan/PROGRESS.md` → `docs/plan/M6-PLAN.md` WP0 → 最近 handoff。对照 `PARALLEL-LANES.md`：本批只动 `crates/cognitive-contracts`、`packages/contracts-ts`、codegen 工具、必要 docs（ledger/PROGRESS/handoff）。

## 目标

1. 将**既有** M6 相关 schema 纳入 Rust/TS codegen CORE_SET（修正型绑定，零 schema/registry/vector 语义变更）：
   - `agent-package-manifest.schema.json`
   - `agent-installation.schema.json`
   - `agent-compatibility-report.schema.json`
   - `performance-report.schema.json`
   - `profile-manifest.schema.json`（若尚未在生成集）
2. 出具合同裁决并登记台账：
   - **无** `specs/transitions/*installation*` 机器表 → 实现按 companion prose 状态序列；禁止声称“安装迁移表已消费”；是否构成修正型漏登由本批证明，**默认不新增表**（IMP-01 冻结）。
   - **无** readiness REQ/schema/vector/carrier → M6 readiness 证据归 milestone e2e/fault；禁止新增 carrier 除非可证明修正型漏登。
3. 事务中断错误码：对照 `specs/registry/errors.yaml` 列出可用码；**禁止虚构**新码。

## 禁止

- 不改向量负例 expected；不新增 REQ/Profile/错误码/transition（除非裁决证明为修正型漏登并走 docs-sync）。
- 不实现 runtime/installer；不碰 `cognitive-runtime`/`cognitive-kernel`/`cognitive-store`。
- 不读 `History/`；不混入 `personal-blog/**`。

## 第一个动作（测试先行）

1. 写失败测试：断言上述 schema 尚无生成模块 / SCHEMA_DIGEST 缺失（或文档化当前缺口）。
2. 扩展 codegen CORE_SET；再生成 Rust/TS；regenerate-and-diff 门绿。
3. 双语言 contract/round-trip 测试通过。
4. 写裁决：`docs/checkpoints/YYYYMMDD-lane-ctr-m6-bindings-handoff.md` + findings-ledger 漂移/备注（若需新 D 条目）。
5. 更新 PROGRESS（CTR 行 + M6 入口句，不虚报实现）；`pnpm run check:consistency`；`gen-matrix --check`；`git diff --check`。
6. 逐路径 commit；开 PR，关联 REQ-AGENT-INSTALL-001/002、REQ-AGENT-COMPAT-001、REQ-PERF-004、REQ-CONF-001/003、IMP-01。

## DoD

- 生成绑定入库；CI regenerate-diff 空。
- 裁决明文写入 handoff；实现车道消费边界清晰。
- 无规范表面扩张；四类状态用语诚实。
- PR 可合入 main 后方启动 [m6-batch1-installer.md](m6-batch1-installer.md)。
