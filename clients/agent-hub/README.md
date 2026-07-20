# clients/agent-hub — Agent Hub 项目入口

> 类别：informative ｜ owner：Lane-CON + Lane-DOC（治理）｜ 状态：`planned`；implementation `not-implemented`；Open PoC `not-run`；Profile `not implemented`

- **用途**：Agent Hub（Direct Takeover + CognitiveOS Governed 两部署模式，第三方 Agent 接管 L1–L8）的项目薄入口。docs/plan/prompts 三子树将在迁移批 B5 迁入本目录。
- **canonical 入口**（B5 迁移前现址）：
  - 文档树：[Agent Hub README](../../apps/cognitiveos-console/docs/agent-hub/README.md) → [GOVERNANCE](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md) → [decision log](../../apps/cognitiveos-console/docs/agent-hub/decisions/decision-log.md)；
  - 计划树：[Master plan](../../docs/plan/agent-hub-development-plan.md) 与 [plan README](../../docs/plan/agent-hub/README.md)；
  - 提示词树：[prompt README](../../docs/prompts/agent-hub/README.md)（全部 `blocked`）。
- **边界**：两部署模式与 L1–L8 层级只由 [部署模式与保证](../../apps/cognitiveos-console/docs/agent-hub/product/deployment-modes-and-guarantees.md) 定义；本入口不复制能力矩阵、不新造保证。
- **gate**：[Agent Hub 实现 gate](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md#7-实现-gate不可跳过)（依赖组 1/2/7 + M5 + 平台 PoC + 技术栈 ADR + 合同门槛 + Paseo/AGPL 法务）；另需 Tier 1 provider 一手接口核验（`AH-CTR-02`）。
