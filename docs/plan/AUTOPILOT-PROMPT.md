# CognitiveOS Personal 自动推进提示词

> Status: active process template, documentation-only. This file does not
> contain permanent task facts and does not override the formal plan,
> `PROGRESS.md` current snapshot, or the Development Operating Model.

复制下方代码块到新窗口。新窗口必须先重建事实，再领取任务；不得依赖本
提示词、handoff 或聊天历史中的过期状态。

```text
你在 D:\agent-kernel 工作，负责继续推进 CognitiveOS Personal。

必读顺序：
1. AGENTS.md；
2. docs/governance/DEVELOPMENT-OPERATING-MODEL.md；
3. docs/plan/PERSONAL-DEVELOPMENT-PLAN.md；
4. docs/plan/PROGRESS.md 的 Current snapshot；
5. docs/plan/PARALLEL-LANES.md 的 active ownership leases；
6. 与你准备领取的 task/lane 匹配的最新 handoff；
7. plan.md 对应任务卡。

启动前执行并以实际结果为准：
- git status --short --branch
- git fetch origin
- git log --oneline --decorate -12
- git log --oneline origin/main..HEAD
- git worktree list

若发现未声明的用户改动、路径重叠 lease、基线漂移或 secret 风险，立即停止写入，
保护现状并重建最小安全路径。禁止读取或引用 `History/`。已获授权、已审查且属于
当前 task lease 的改动可作为正常工作的一部分测试、暂存和提交。

当前治理状态（2026-07-30）：
- P1-T09 task_status = in-progress；development_track = experimental-local-only；
  implementation_evidence = tested-local；claim_scope = non-claim；
- B01 gate_status = not-run；GMVP-LINUX = not-run；Profile implemented = 0；
- 当前已存在 Pi configure/launch admission 与 focused local evidence，但不声称真实
  Pi Extension load、Provider conversation、native Secret Service、B01、release 或 Profile。
以上是快照，不得假设仍是最新；以文件和 git/CI 事实复核。

领取任务规则：
- 任务的第一个真实设计/实现/测试 slice 即可把 not-started 改为 in-progress；
- acceptance_requires 和 promotion_requires 不阻止满足 implementation_requires 的
  experimental-local-only 实现；
- 一个 task 只能有一个 primary lane/branch/active lease，可声明 cohesive secondary paths；
- normative contracts/schema/registry/transitions/vector 仍只能走 Lane-CTR；
- 先写 failure-first test，再实现；概率组件只能产 proposal，确定性服务拥有授权、CAS、
  状态迁移、预算、幂等、fencing、Effect 与完成验收；
- secret 永不进入 argv、普通 config、SQLite、日志、CI 或 evidence。

验证按影响面分阶段：
- commit 前：affected tests、focused negatives、affected lint/format、diff/secret checks；
- push 前：可用的相关广域回归与 consistency，未执行项准确记录 not-run 及原因；
- merge/task done 前：required protected CI 全绿、status/PROGRESS/handoff 对齐；
- remote CI pending 可以存在 commit；required red check 禁止 merge 或完成声明。

文档联动：声明 implementation-only/corrective/normative-semantic/structural；实现型
变更必须同步任务证据并声明 normative surface unchanged，不得为凑联动修改合同资产。
实现 commit 与 closure docs 可以是同一 delivery/PR 的不同 commit；closure docs 必须
记录 immutable implementation hash、测试、non-claims、风险和 remote visibility。

结束时：
1. 更新正式任务状态、implementation evidence、Gate/claim 边界；
2. 更新 PROGRESS Current snapshot；
3. 写 docs/checkpoints/YYYYMMDD-<task>-handoff.md；
4. 逐路径 git add，检查 push surface，不得 git add -A；
5. 按 ADR-0008 提交并 push；代码批走 PR，docs-only 低风险批可按分支保护规则处理；
6. 在当前批真正结束后，再生成下一批动态提示词，不提前硬编码未来状态。
```
