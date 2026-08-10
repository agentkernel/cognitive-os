# ADR-0008: 代理自动提交与自动 push 政策

- Status: Accepted（historical authorization）; **workflow semantics superseded by**
  Operating Model `CHECKPOINT-DELIVERY-01` and standing continuous-delivery rules
  in [DEVELOPMENT-OPERATING-MODEL.md](../governance/DEVELOPMENT-OPERATING-MODEL.md)
- Date: 2026-07-21
- Supersession note (2026-08-10, P8-T01): keep this ADR as the original owner
  authorization record. Do not use it as the executable workflow. Current commit,
  push, Draft PR, ready/merge, and dirty-handoff rules live only in the Operating
  Model.
- Decision owners: 仓库所有者 + CognitiveOS 参考实现维护者
- Classification: 仓库治理决策。只约束本仓库的开发流程；不是 CognitiveOS 规范要求。

## Context

参考实现由多个并行代理车道推进（`docs/plan/PARALLEL-LANES.md`）。每个原子任务完成后等待人工逐次批准提交/推送，会把所有者变成流水线瓶颈，且延迟使并行车道的 docs 合并冲突面变大。仓库已具备足够的机器门禁：CI 两 OS 矩阵、静态一致性检查、逐路径 staging 纪律与 push 前影响面检查。

## Decision

1. 仓库所有者明确授权：通过相应验证阶段的原子批由代理自动提交并自动 push，无需逐次请示。任务是否 `done`、远端 CI 是否完成、产品 Gate 是否通过分别记账，不再互相替代。
2. 授权附带的硬条件（违反任何一条即失去本授权的适用性）：
   - 提交前必须通过受影响 package 的 focused tests/lint/format 与相关负例；禁止提交已知受影响失败。远端 CI pending、非支持本地环境或明确无影响的检查记为 `not-run`，不虚报通过，也不自动禁止隔离 commit；
   - 禁止 `git add -A` / `git add .`，一律逐路径 `git add`；禁止混入无关改动或他人工作区状态；
   - 禁止 force-push 任何已推送分支；
   - 禁止推送 `personal-blog/**`（独立子工程）；`clients/**` 仅允许核心 gate 变化引起的最小 Markdown 状态同步；
   - push 前必须运行 `git log --name-only origin/main..HEAD`（或 lane 分支对其远端比对）逐文件核对推送面。
3. 直推 `main` 仅限 docs-only 低风险批；代码批沿用 lane 分支 + PR + CI 全绿后合并的惯例。若分支保护拒绝直推，自动改走 lane 分支 + PR，不得绕过。
4. 工具无关执行细则以 [Development Operating Model](../governance/DEVELOPMENT-OPERATING-MODEL.md) 为准；`.cursor/rules/` 如存在，仅是编辑器适配层。

## Validation stages

1. **Pre-commit:** failure-first 证据（行为变更）、受影响测试/lint/format、相关 secret/diff 检查通过；无已知受影响失败。
2. **Pre-push:** 在可用受支持工具链执行相关广域回归与 consistency；未执行项及原因写入 handoff/PR；核对 staged 和 branch push surface。
3. **Merge/task completion:** required protected CI 全绿，状态/PROGRESS/handoff 对齐。任何 required red check 禁止 merge 或完成声明。

实现与 closure documentation 属于同一 atomic delivery/PR，但不要求同一 commit。允许一个实现 commit 后跟一个 handoff/docs commit，以便 handoff 记录前者的 immutable hash。

## Alternatives considered

### 每次提交/推送人工批准

拒绝：所有者成为串行瓶颈；并行车道的共享文档（PROGRESS/ledger）冲突窗口被人为拉长；批准动作本身不产生质量信号（质量由 CI 与向量证据承载）。

### 只允许本地提交、推送一律人工

拒绝：跨会话交接依赖远端状态（handoff 协议）；本地滞留提交在多 worktree 布局下反而增加漂移与误合并风险。

## Consequences

- 代理在 pre-commit/pre-push 条件满足后提交并 push；远端 CI 结论用于 merge/任务完成门禁。
- 红灯处置责任随授权转移给代理：push 后 CI 红必须立即修复或回退。
- 所有既有红线（`AGENTS.md` 与 tracked governance documents）不因本授权放宽；本 ADR 只取消"逐次请示"这一步。
