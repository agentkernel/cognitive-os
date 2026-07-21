# M7 Batch：Memory runtime 最小竖切

读 `AGENTS.md` → `PROGRESS.md` → `M7-PLAN.md` WP-M7-MEM → Batch-0B handoff。

## 范围（Lane-KRN 主导）

最小竖切：**candidate → deterministic admission → writer-private RYW → quarantine/reject**。

必须行为证明：
- 模型摘要/Agent 自述不可直接发布长期 memory（`MEMORY_ADMISSION_DENIED`）
- pending candidate：writer RYW true；跨 Conversation/ResourceScope 不可见
- 失败候选 quarantine，非静默丢弃
- authority 不变；capability 不扩张

证据：KRN 行为测试 +（若 CFR 已接线）向量执行记录。

## 禁止

跨 scope 晋升/派生失效全量（可后续批）；durable InstallationStore；PERF-005 收益声明；改负例；新增对象族。

## 出口

竖切证据可复现后，再开 lifecycle 批或 `m7-discovery-delta-and-stagnation.md` / `m7-cfr-vectors-and-self-check.md`。
