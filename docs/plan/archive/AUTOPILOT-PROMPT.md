# CognitiveOS Personal 自动推进提示词

> Status: active process template, documentation-only. It contains no current
> task facts and cannot override project identity, the formal Personal plan,
> `PROGRESS.md`, the lease ledger, or the Development Operating Model.

复制下方代码块到新窗口。提示词故意不写任务状态，接续代理必须从 canonical sources
重建当前事实，避免复制旧快照造成循环。

```text
你在 D:\agent-kernel 工作。CognitiveOS 设计、规范和通用内核是架构/合同参考层；
cognitiveos-personal 是当前唯一活动实现项目。不要启动第二个 CognitiveOS 产品 backlog。

按顺序读取：
1. AGENTS.md；
2. docs/governance/PROJECT-IDENTITY.md；
3. docs/governance/DEVELOPMENT-OPERATING-MODEL.md；
4. docs/plan/PERSONAL-DEVELOPMENT-PLAN.md；
5. docs/plan/PROGRESS.md 的 Current snapshot；
6. docs/plan/PARALLEL-LANES.md 的 active lease table；
7. 所选 Personal task 的最新 handoff；
8. docs/plan/plan.md 的同 ID 任务卡。

启动时检查实际 git status、分支、origin/main 差异和现有 worktree。正式计划决定任务和
Gate；PROGRESS 决定当前事实；Parallel Lanes 决定可写路径；handoff 与 docs/plan/plan.md 不是状态源。
禁止读取或引用 History/。未知改动不能覆盖、回退或混入；用户明确授权且已审查的改动
可以整合进当前 lease。

只从 Personal 正式计划选择下一任务。选择满足 implementation_requires 的最小垂直
slice，不得把 acceptance_requires 或 promotion_requires 当成实现互斥锁。纯研究、讨论
和未提交计划不改变 task status；首个真实实现或 failure-first 测试 slice 开始时，将
任务设为 in-progress 并在同一 delivery 对齐正式计划。

写入前在 PARALLEL-LANES 活动表领取稳定 lease_id，确认 writable paths 不与 active lease
重叠，并让 PROGRESS 只引用该 lease_id。规范合同变化走 Lane-CTR；实现型变更不得为凑
联动修改 registry/schema/transition/vector。

每个会话必须交付以下之一：
- 最小垂直实现 slice + focused verification；
- failure-first regression/negative test + fix；
- 可验证治理修正；
- bounded blocker，列明 blocked_paths、blocked_task_ids、blocked_gate_ids、owner、
  evidence 和 next executable action。
任务、依赖和安全路径已明确后，不得继续扩大审计或另建平行计划。

保持硬边界：daemon 是唯一 authority；Pi/CLI/SDK/UI/fixture 是客户端；secret 不进入
argv/config/SQLite/log/CI/evidence；外部 mutating operation 使用 persist-before-dispatch
Intent/Effect；外部成功或 Pi agent_end 不等于 Task 完成；local/WSL/fixture/ordinary CI
默认只能产生 non-claim evidence。

验证按影响面分阶段。未执行项准确写 not-run。required red CI 禁止 merge 或 done 声明。
结束时更新正式计划、PROGRESS Current snapshot、handoff 和 lease closure；handoff 使用
status_at_handoff/current_status_source/supersedes/superseded_by 字段，不制造当前状态副本。
逐路径暂存，禁止 git add -A、force-push 和将 personal-blog/ 推入本仓库。
```
