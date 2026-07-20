# Agent Hub 开发计划目录

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> Master 计划见 [../agent-hub-development-plan.md](../agent-hub-development-plan.md)。本目录含支撑文档与 12 宏车道 + 6 Adapter 子车道计划。**全部实现任务默认 `blocked`。**

## 支撑文档

- [progress.md](./progress.md) — 统一进度（局部；工程真相以全局 PROGRESS 为准）
- [milestones.md](./milestones.md) — AH-M0~M6 里程碑与出口 gate
- [dependency-dag.md](./dependency-dag.md) — 车道依赖 DAG
- [risk-register.md](./risk-register.md) — 风险/blocker 登记
- [evidence-index.md](./evidence-index.md) — 证据/PoC 索引（全部 not-run）

## 宏车道计划（12）

1. [lane-governance.md](./lane-governance.md)
2. [lane-contract-capability.md](./lane-contract-capability.md)
3. [lane-host-control-ledger.md](./lane-host-control-ledger.md)
4. [lane-process-terminal.md](./lane-process-terminal.md)
5. [lane-session-file.md](./lane-session-file.md)
6. [lane-credential-workspace-verifier.md](./lane-credential-workspace-verifier.md)
7. [lane-relay-pairing.md](./lane-relay-pairing.md)
8. [lane-desktop.md](./lane-desktop.md)
9. [lane-ios.md](./lane-ios.md)
10. [lane-android.md](./lane-android.md)
11. [lane-multi-agent.md](./lane-multi-agent.md)
12. [lane-quality-release-migration.md](./lane-quality-release-migration.md)

## Tier 1 Adapter 子车道（6）

- [adapter-codex.md](./adapter-codex.md)
- [adapter-opencode.md](./adapter-opencode.md)
- [adapter-claude-agent-sdk.md](./adapter-claude-agent-sdk.md)
- [adapter-hermes.md](./adapter-hermes.md)
- [adapter-openclaw.md](./adapter-openclaw.md)
- [adapter-openhands.md](./adapter-openhands.md)

## 任务格式

所有任务用 [任务模板](../../../apps/cognitiveos-console/docs/agent-hub/templates/development-task.md)：ID `AH-<lane>-<seq>`，含 owner/lane/depends_on/blocked_by/允许禁止路径/交付物/失败测试/安全负例/oracle/evidence/commit/文档/handoff。

## 提示词

对应自包含接续提示词见 [../../prompts/agent-hub/README.md](../../prompts/agent-hub/README.md)。
