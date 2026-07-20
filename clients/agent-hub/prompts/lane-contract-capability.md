# 接续提示词 — Agent Hub 合同与能力协商车道（CTR）

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

## 硬纪律（全程有效）

1. 确定性边界：概率组件只产 candidate/proposal；授权/CAS/迁移/预算/幂等/fencing/提交由确定性代码执行。
2. 规范优先级：机器合同 > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议；冲突取不扩大权限/范围/风险/预算/完成声明。
3. 四类状态用语严格区分。
4. 测试先行；schema-valid ≠ behavior-pass。
5. 规范表面冻结；漂移先登记 findings-ledger 再修正。
6. P0 门禁。
7. 可追溯提交。
8. 红线：禁 `History/`；禁虚构规范资产；禁改写向量/删负例。

## 本车道任务

- canonical：[adapters/README.md](../docs/adapters/README.md)、[capability-matrix.md](../docs/adapters/capability-matrix.md)、[interface-layering.md](../docs/adapters/interface-layering.md)、[sources/provider-interfaces-ledger.md](../docs/sources/provider-interfaces-ledger.md)
- 计划：[docs/plan/agent-hub/lane-contract-capability.md](../plan/lane-contract-capability.md)
- 目标：能力模型/矩阵冻结、六 Adapter 接口一手核验、Governed `REQ-AGENT-*` 对接、接口漂移监测。

## gate 与允许范围

接口核验属研究文档；对接 `REQ-AGENT-*` 一律经 Lane-CTR 契约流程，禁止直接改 `specs/**`、`conformance/**`。禁止把竞品间接观察或营销描述当官方合同；MCP 稳定版固定 2025-11-25，未生效 RC 只记录。任务 AH-CTR-01..04 见车道计划。

## 会话结束协议

更新 PROGRESS → 写 handoff → 逐路径分批提交。
