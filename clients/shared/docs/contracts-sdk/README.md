# shared/docs/contracts-sdk — SDK 与契约消费说明

> 类别：informative ｜ owner：Lane-CON（package 事实归 Lane-TSC/Lane-CTR）｜ 状态：`planned`

- **用途**：客户端如何消费 TypeScript SDK 与机器契约的说明入口。
- **canonical 指针**（不复制正文）：
  - 客户端边界规则：[.cursor/rules/11-typescript-clients.mdc](../../../../.cursor/rules/11-typescript-clients.mdc)；
  - canonical 编码：[ADR-0004 canonical JSON](../../../../docs/adr/0004-canonical-json.md)；代码生成：[ADR-0006](../../../../docs/adr/0006-code-generation-policy.md)；
  - 跨语言夹具：[tests/golden/README.md](../../../../tests/golden/README.md)；
  - 入口：[contracts-ts](../../../../packages/contracts-ts/src/index.ts)、[sdk-ts](../../../../packages/sdk-ts/src/index.ts)、[agent-shell](../../../../apps/agent-shell/src/index.ts)。
- **边界**：依赖方向固定 `agent-shell → sdk-ts → contracts-ts`；canonical encoding/digest 只消费 `contracts-ts`，客户端不得重实现；task/management 双通道隔离与 snapshot/watch/cursor/reconnect 语义以 SDK/契约 companion 为准，本目录不新造。
- **状态**：sdk-ts/agent-shell 已有 M5 前客户端骨架与包内测试，真实 transport 集成仍 `blocked`；contracts-ts 实现已提供（包级）；conformance 分布以全局 PROGRESS 为准，不等于客户端平台证据。
