# 车道计划 — 合同与能力协商（CTR）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON（契约经 Lane-CTR）｜ 状态：blocked
>
> 目标：固化能力模型、逐能力矩阵、接口一手核验，并对接 Governed 模式的已登记 `REQ-AGENT-*` 契约。

## 范围与路径

- 允许：`.../agent-hub/adapters/`、`.../agent-hub/sources/`、本计划。
- 禁止：直接改 `specs/registry/`、`specs/schemas/`、`conformance/vectors/`（一律经 Lane-CTR 流程）；他人车道代码。
- 依赖：GOV。gate：接口 gate（provider-interfaces-ledger 补齐）、契约 gate（Governed）。

## 任务

### AH-CTR-01 能力模型与逐能力矩阵冻结
- owner/lane：Lane-CON / CTR｜depends_on：AH-GOV-01｜blocked_by：接口 gate
- 允许路径：`adapters/README.md`、`adapters/capability-matrix.md`、`adapters/interface-layering.md`
- 交付物：33 能力定义、接口分层（MCP 2025-11-25 pin）、矩阵读法规则
- 失败测试先行：矩阵完整性 lint（每能力每 Agent 有状态）
- 安全负例：`待核验` 不得被写成 `已支持`
- oracle：矩阵与 dossier 一致｜evidence：none

### AH-CTR-02 六 Adapter 接口一手核验清单
- owner/lane：Lane-CON / CTR｜depends_on：AH-CTR-01｜blocked_by：接口 gate
- 交付物：每 Agent 官方接口 URL/version/commit、稳定性、session 语义、凭据位置补齐 provider-interfaces-ledger
- 安全负例：不得把竞品间接观察当官方合同
- oracle：六 Adapter dossier 无 `待核验` 关键项｜evidence：not-run

### AH-CTR-03 Governed `REQ-AGENT-*` 对接
- owner/lane：Lane-CON + Lane-CTR / CTR｜depends_on：AH-CTR-01｜blocked_by：契约 gate（外部 M6）
- 交付物：Governed 受治理 Adapter 与已登记 `REQ-AGENT-*`/schema/vector 的映射；不新增违反冻结的规范表面
- 安全负例：不虚构 REQ-ID/错误码/schema/vector
- oracle：`pnpm run check:consistency` 通过；引用的 REQ 真实存在｜evidence：not-run

### AH-CTR-04 接口漂移监测流程
- owner/lane：Lane-CON / CTR｜depends_on：AH-CTR-02｜blocked_by：—
- 交付物：跨版本接口漂移登记流程（进各 Adapter 未决 + Open PoC）
- oracle：漂移有登记入口｜evidence：none
