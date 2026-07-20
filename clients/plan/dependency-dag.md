# 客户端依赖 DAG（指针）

> 类别：plan pointer ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 本文件只登记指针，不复制依赖表正文。

- **Console 后端九组依赖**（canonical）：[DEVELOPMENT-PLAN Console 节](../../docs/plan/DEVELOPMENT-PLAN.md)——§20.3 依赖组 1..9 与提供方里程碑（M2/M4/M5/M6/M7）；激活规则=组 1/2/7 交付 + M5 出口评审 + 目标平台真实 PoC。
- **Agent Hub 依赖 DAG**（canonical）：[dependency-dag](../../docs/plan/agent-hub/dependency-dag.md)（B5 迁移前现址；迁移后 `clients/agent-hub/plan/dependency-dag.md`）。
- **车道依赖**：合并顺序 CTR → {KRN, CFR, TSC} → RUN（[PARALLEL-LANES §2](../../docs/plan/PARALLEL-LANES.md)）；客户端实现在 TSC/RUN 之后。
- **上游阻断明细**：[requirements-traceability §3 上游阻断登记](../../apps/cognitiveos-console/docs/requirements-traceability.md#3-上游阻断登记)（B2 迁移前现址；`CONSOLE-V2-BLK-*` 14 项）。
