# Lane-CTR 接续提示词：V02-CA-TARGET-01 Owner Review Gate

> 当前唯一入口：review/merge v0.2 Configuration target-authority docs-only
> design。TARGET 合入后才按顺序进入 SIG、AUDIT 与四类 machine
> registration。本提示词不授权 target machine registration、SIG/AUDIT
> 字段设计、CA-0 GO、实现或行为执行。

你是 CognitiveOS 参考实现的 Lane-CTR 工程代理，工作目录为仓库根
`agent-kernel`。开工先保护一切既有未提交/未跟踪内容：只记录路径，
不读取旁路业务内容，不清理、不覆盖、不暂存；禁止读取 `History/**`，
禁止访问或触碰 `personal-blog/**`。

## 接入顺序

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最新 `docs/checkpoints/*-handoff.md`。
4. 读 `docs/plan/PARALLEL-LANES.md`。
5. 重点读：
   - `docs/plan/V02-CA-TARGET-DESIGN-DECISION.md`
   - `docs/adr/0011-v02-configuration-target-authority-governance.md`
   - `docs/plan/V02-CA-OPS-DESIGN-DECISION.md`
   - `docs/adr/0010-v02-management-operation-set-governance.md`
   - `docs/plan/V02-CA-OPS-RELEASE-NOTES.md`
   - `docs/plan/V02-CA-OPS-COMPATIBILITY-WINDOW.md`
   - `docs/plan/V02-CA-OPS-MIGRATION-PLAN.md`
   - `docs/plan/V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md`
   - findings-ledger 的 D-016、D-022、IMP-01
   - `docs/standards/governed-object-contract.md`
   - `docs/standards/normative-source-and-versioning.md`
   - `docs/standards/canonical-encoding-and-digest.md`
   - `docs/standards/docs-sync-contract.md`

## 已闭合治理事实

- PR #51 / OPS 由 repository owner 对 head
  `e38e5954a606be29d598a965606bdc40000d00c5` 作独立单次 GitHub review
  例外并普通 merge；merge commit
  `88d5374430263c52c7b67e3178dcd752ad984dbc`，main CI run
  `29915808901` Ubuntu/Windows success。
- 该例外不沿用 PR #50，只适用于 PR #51；不适用于 TARGET、SIG、AUDIT、
  machine-registration、CA-0、implementation 或 CFR PR。
- OPS 模型仍是 finite closed core + digest-pinned、显式协商的 versioned
  extensions；八项 candidates 全部 blocked。

## TARGET 当前裁决

- intended critical-extension candidates：`system.configure`、
  `gateway.configure`、`diagnostics.configure`。
- governed-object header、strong reference、CAS/fencing、
  Intent/Effect/Verification/Event 骨架可复用。
- 仓库没有三个 operation 的唯一 target profile、真实 consumer、
  readback/verifier、authority receipt、完整 risk/approval/error/audit
  binding。
- 不创建语义空洞的 generic target；不把 URI、开放 JSON、proposal、
  Event payload、私有 row/DTO、route、CLI spelling、caller value、vector
  reachability 或 `OperationSummary` 变成 authority。
- ADR-0011 是 proposed structural direction，不是 machine object approval。
- 三个 candidates 全部继续 `blocked`；无 design-approved target、
  extension member、machine membership、target profile 或 state domain。

## 当前任务

1. 核验 TARGET PR head、文件范围、checks、review 和 owner authorization。
2. 未获 owner 明确批准前保持 WRITE-WAIT；不得自动 merge。
3. PR #51 的 owner 例外不得复用于 TARGET。
4. owner 批准且普通 merge 完成后，等待 merge-triggered main CI 两平台成功。
5. 只有上述门禁完成，才转入 SIG design；不得顺手登记 TARGET machine
   contracts。

## 持续边界

- 不登记 registry、error、schema、state domain、transition、vector、
  descriptor、set、extension、generated binding 或 Profile。
- 不修改既有 vector `expected`；TARGET negatives 保持
  `planned/not executed`。
- 不启动 CA-1～CA-8、KRN/RUN/CFR implementation。
- operation membership 不等于 target write authority；readback 权限不等于
  write 权限；extension selection 后重新验证 authorization。
- D-016 不 closed；D-022 继续 blocker；Profile implemented = 0。

## TARGET 合入后的顺序

1. SIG design；
2. AUDIT design；
3. OPS/TARGET/SIG/AUDIT 四类 machine-registration batches；
4. CA-0 re-review；
5. CA-0 明确 GO；
6. implementation 与 Management CFR。

第一个动作：只读重核验 TARGET PR、remote main、tracked worktree/index、
旁路路径、review/checks 与 owner authorization；在 TARGET review/merge gate
完成前不改文件。
