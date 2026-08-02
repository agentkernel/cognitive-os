# 交接 / 里程碑评审模板

命名规则（放在本目录）：

- 会话交接：`YYYYMMDD-<车道或里程碑>-handoff.md`（如 `20260801-lane-ctr-handoff.md`）
- 里程碑评审：`YYYYMMDD-<里程碑>-milestone-review.md`（如 `20260901-m1-milestone-review.md`）

交接文档是跨会话操作连续性载体：写给一个**没有本次对话历史**的接续代理。它只记录
交接时事实，不是当前状态源，不能覆盖正式 Personal 台账、`PROGRESS.md` Current
snapshot 或活动 lease 台账。

---

## Handoff 模板

```markdown
# YYYYMMDD <车道/里程碑> Handoff

## Record metadata

- record_type: handoff
- project_id: cognitiveos-personal
- task_id: <P*-T* 或 governance id>
- lease_id: <lease/personal/...>
- status_at_handoff: <not-started/in-progress/blocked/done/cancelled>
- development_track_at_handoff: <production-path/experimental-local-only/not-applicable>
- implementation_evidence_at_handoff: <none/provided/tested-local/tested-supported-ci>
- gate_status_at_handoff: <not-run/running/pass/fail/blocked/not-applicable>
- claim_scope_at_handoff: <non-claim/product-gate/release/profile>
- task_definition_source: docs/plan/PERSONAL-DEVELOPMENT-PLAN.md
- current_status_source: docs/plan/PROGRESS.md Current snapshot
- blocked_paths: <列表或 none>
- blocked_task_ids: <列表或 none>
- blocked_gate_ids: <列表或 none>
- blocker_owner: <owner 或 none>
- next_executable_action: <具体动作>
- supersedes: <较早 handoff 或 none>
- superseded_by: <后续 handoff 或 none-known-at-write-time>

## 1. 本次会话完成
- （逐条：交付物 + 涉及 REQ-ID/F/IMP 条目 + 提交哈希）

## 2. 未完成 / 进行中
- （逐条：状态、卡点、剩余步骤）

## 3. 测试与证据状态
- 每个检查：<命令或 campaign；pass/fail/not-run；环境；摘要>
- CI：<pass/fail/not-run + 链接>
- 向量：<not-run/pass 计数变化>
- 证据：<路径、immutable digest、collector/version、redaction>

## 4. 未决风险与漂移
- （新发现漂移是否已登记 findings-ledger；开放风险）

## 5. 下一步入口
- 正式任务：<task ID 与 formal-plan anchor>
- matching handoff 规则：<project_id + task_id + lease_id + date/supersedes>
- 工作分支：<branch>
- 第一个动作：<next_executable_action 的具体命令或文件>

## 6. 快照
- PROGRESS 已更新：<是/否>
- active lease：<已关闭/移交给 lease_id/仍 active 及原因；merged PR 不得遗留 active>
- 本次提交列表：<hash 列表>
- immutable implementation commit：<hash 或 not-applicable>
- remote visibility：<PR/remote commit/pending/not-applicable>
```

---

## Milestone review 模板

```markdown
# YYYYMMDD <里程碑> Milestone Review

## 1. 范围回顾
（对照 docs/plan/DEVELOPMENT-PLAN.md 该里程碑的范围与交付物）

## 2. 验收判据逐条对照
| # | 判据 | 结果（通过/未通过） | 证据（路径/digest/提交） |
|---|---|---|---|

## 3. 安全负例清单
（本里程碑新增/执行的负例与结果）

## 4. 未通过项与阻断
（每项未通过 = 阻断项，列出阻断的下一里程碑车道）

## 5. 漂移与规范变更
（本里程碑登记/闭合的漂移，findings-ledger 链接）

## 6. 指标快照
（REQ 覆盖、向量分层通过、开放 P0/P1、性能指标（M6 起））

## 7. 结论
（GO / NO-GO 到下一里程碑；遗留条件）
```
