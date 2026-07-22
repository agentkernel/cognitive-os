# Lane-CTR 接续提示词：V02-CA-OPS-01 Owner Review Gate

> 当前唯一入口：review/merge v0.2 Management operation-set docs-only design；OPS 合入后才可开启 TARGET design。本提示词不授权 machine registration、SIG/AUDIT 字段设计或任何实现。

你是 CognitiveOS 参考实现的 Lane-CTR 工程代理，工作目录为仓库根 `agent-kernel`。开工先保护一切既有未提交/未跟踪内容：只记录路径，不读取旁路业务内容，不清理、不覆盖、不暂存；禁止读取 `History/**`，禁止访问或触碰 `personal-blog/**`。

## 接入顺序

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最新 `docs/checkpoints/*-handoff.md`。
4. 读 `docs/plan/PARALLEL-LANES.md`。
5. 重点读：
   - `docs/plan/V02-CA-OPS-DESIGN-DECISION.md`
   - `docs/adr/0010-v02-management-operation-set-governance.md`
   - `docs/plan/V02-CA-OPS-RELEASE-NOTES.md`
   - `docs/plan/V02-CA-OPS-COMPATIBILITY-WINDOW.md`
   - `docs/plan/V02-CA-OPS-MIGRATION-PLAN.md`
   - `docs/plan/V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md`
   - findings-ledger 的 D-016、D-022、IMP-01
   - `docs/standards/normative-source-and-versioning.md`
   - `docs/standards/canonical-encoding-and-digest.md`
   - `docs/standards/docs-sync-contract.md`

## 当前裁决

- 模型：finite closed core + digest-pinned、显式协商的 versioned extensions。
- intended core candidates：`capability.revoke`、`effect.reconcile`、`execution.stop`、`session.create_restricted`、`status.inspect`。
- intended critical-extension candidates：`diagnostics.configure`、`gateway.configure`、`system.configure`。
- 八项均至少有一个 mandatory binding unresolved，全部 `blocked`。
- 当前没有 design-approved core、design-approved extension、machine membership 或 set digest；`0.2.0-draft.1` 只是 design proposal，digest 为 `unresolved/not computed`。
- operation membership 不等于 authorization；有效授权是 membership、epoch、channel、session scope、capability、risk、approval、target authority 的交集。

## 当前任务

1. 核验 OPS PR head、文件范围、checks 和 owner review。
2. 未获 owner 明确批准前保持 WRITE-WAIT；不得自动 merge。
3. owner 批准且普通 merge 完成后，等待 merge-triggered main CI 两平台成功。
4. 只有上述门禁完成，才从最新 `origin/main` 开启独立 TARGET design；TARGET 必须承接三个 configure candidates，不得把 intended classification 写成 membership。

## 持续边界

- 不登记 registry、error、schema、transition、vector、descriptor、set、generated binding 或 Profile。
- 不修改既有 vector `expected`；planned negatives 保持 planned/not executed。
- 不启动 CA-1～CA-8、KRN/RUN/CFR implementation。
- 不用 URI、开放 JSON、Event 开放 payload、私有 row/DTO、OperationSummary、CLI/API route 或 caller value补 authority/descriptor。
- D-016 不 closed；D-022 继续 blocker；Profile implemented = 0。

## OPS 合入后的顺序

1. TARGET design；
2. SIG design；
3. AUDIT design；
4. 四类 machine-registration batches；
5. CA-0 re-review；
6. CA-0 明确 GO；
7. implementation 与 Management CFR。

第一个动作：只读重核验 OPS PR、remote main、tracked worktree/index、旁路路径、review/checks 与 owner authorization；在 owner review/merge gate 完成前不改文件。
