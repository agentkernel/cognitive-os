# 客户端 owner 指针矩阵

> 类别：informative pointer matrix ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> **权威唯一**：[PARALLEL-LANES §3 所有权表](../../docs/plan/PARALLEL-LANES.md#3-所有权表当前)。本文件只是指针矩阵，不复制、不另立事实；冲突时以 PARALLEL-LANES 为准。

| 域 | owner 车道 | 说明 |
|---|---|---|
| `clients/**` 全部 Markdown | Lane-CON（治理文件由 Lane-DOC 协作） | informative 文档例外范围内维护；实现受 gate 阻断 |
| `apps/cognitiveos-console/`（stub）、`docs/platforms/`（B6 后 stub） | Lane-CON | 兼容入口维护 |
| `apps/agent-shell/`、`packages/sdk-ts/` | Lane-TSC | 不迁移；README 缺口待 Lane-TSC 补 |
| `packages/contracts-ts/` | Lane-CTR | 不迁移；接口变更只经 Lane-CTR 契约流程 |
| `tools/`（含 consistency checker） | Lane-CFR | clients 扫描自动化任务须经 Lane-CFR 领取 |
| `specs/**`、`conformance/**` | 契约资产（Lane-CTR；向量增补可经 Lane-CFR） | clients 只引用 |
| `docs/plan/*`、`docs/traceability/*`、`AGENTS.md`、根 README | Lane-DOC（随各车道 PR） | 治理入口联动走最小更新 |

跨车道变更流程：接口/代码目录移动须经所属车道；`clients/` 文档发现缺口 → 登记 findings-ledger + 通告对应车道，不代替登记。
