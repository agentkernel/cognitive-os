# clients/shared/plan — 共享能力计划（blocked）

> 类别：plan ｜ owner：Lane-CON ｜ 状态：`blocked`

- **用途**：共享客户端能力（SDK 消费、双通道、Relay、遥测口径）的计划入口。当前无独立排期：SDK/Shell 工作归 [Lane-TSC](../../../docs/plan/PARALLEL-LANES.md)（M5 集成），Relay/配对实现归 Agent Hub 对应车道（[lane-relay-pairing](../../agent-hub/plan/lane-relay-pairing.md)，`blocked`）。
- **边界**：本目录不给 Lane-TSC/Lane-CTR package 排期；跨车道任务只登记指针。
- 客户端全域里程碑见 [clients/plan/milestones.md](../../plan/milestones.md)。
