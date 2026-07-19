# 交接 / 里程碑评审模板

命名规则（放在本目录）：

- 会话交接：`YYYYMMDD-<车道或里程碑>-handoff.md`（如 `20260801-lane-ctr-handoff.md`）
- 里程碑评审：`YYYYMMDD-<里程碑>-milestone-review.md`（如 `20260901-m1-milestone-review.md`）

交接文档是跨会话唯一记忆载体：写给一个**没有本次对话历史**的接续代理。

---

## Handoff 模板

```markdown
# YYYYMMDD <车道/里程碑> Handoff

## 1. 本次会话完成
- （逐条：交付物 + 涉及 REQ-ID/F/IMP 条目 + 提交哈希）

## 2. 未完成 / 进行中
- （逐条：状态、卡点、剩余步骤）

## 3. 测试与证据状态
- CI：<绿/红 + 链接或本地命令输出摘要>
- 向量：<not-run/pass 计数变化>
- 证据：<artifacts/evidence/ 下产物与 digest>

## 4. 未决风险与漂移
- （新发现漂移是否已登记 findings-ledger；开放风险）

## 5. 下一步入口
- 建议提示词：docs/prompts/<文件>
- 工作分支：<branch>
- 第一个动作：<具体命令或文件>

## 6. 快照
- PROGRESS 已更新：<是/否>
- 本次提交列表：<hash 列表>
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
