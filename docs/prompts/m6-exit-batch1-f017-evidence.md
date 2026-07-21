# M6-EXIT Batch-E1：F-017 证据固化

读 `M6-EXIT-PLAN.md` WP-F017 + `f017-platform-matrix.md` Claim freeze。

## 范围

1. 为声明集内每个 `denied_with_evidence` 行固化复现命令 + digest。
2. 保持 `refuse_cross_platform_merge` 可测。
3. 更新矩阵台账闭合口径（相对声明集）；更新 findings-ledger F-017。
4. 新 CI job 须单独 CFR workflow-scope 权限确认——默认本批不新增 job。

## 禁止

扩大声明集；跨平台合并；无 digest 的 deny；改写 AGENT-BYPASS-002 expected。

## 出口

F-017 closed-for-release-claim-set；handoff + PROGRESS；下一会话 `m6-exit-batch3-rereview.md`（或可选 store/PERF 批）。
