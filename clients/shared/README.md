# clients/shared — 共享客户端能力说明域

> 类别：informative ｜ owner：Lane-CON（涉及 package 事实归属 Lane-TSC/Lane-CTR）｜ 状态：文档域 `planned`

- **用途**：跨 PC/手机客户端的共享能力说明：SDK/契约消费关系、身份/session、Relay/配对、设计系统、安全隐私、遥测与证据口径、共用测试策略。只放 informative 消费侧说明。
- **边界（重申，违者拒收）**：
  - 机器合同只在 `specs/**`、`conformance/**` 与 [contracts-ts](../../packages/contracts-ts/src/index.ts)；本目录**不复制**任何 schema、digest 规则或错误码定义；
  - `apps/agent-shell`、`packages/sdk-ts`（Lane-TSC）与 `packages/contracts-ts`（Lane-CTR）**不迁移、不重命名、不在此建立代码副本**；
  - 依赖方向固定 `agent-shell → sdk-ts → contracts-ts`；客户端不得自行重定义 authority 状态、授权、完成或 canonical digest。
- **子目录**：[contracts-sdk/](docs/contracts-sdk/README.md)、[identity-session/](docs/identity-session/README.md)、[relay-pairing/](docs/relay-pairing/README.md)、[design-system/](docs/design-system/README.md)、[security-privacy/](docs/security-privacy/README.md)、[telemetry-evidence/](docs/telemetry-evidence/README.md)、[plan/](plan/README.md)。
- **gate**：各消费方 gate（[Lane-TSC gate](../../docs/plan/PARALLEL-LANES.md)、[Console 实现 gate](../../docs/platforms/README.md#console-实现-gate)）。
