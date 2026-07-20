# Agent Hub 接续提示词索引

> 类别：prompt（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 本目录含 12 宏车道 + 6 Tier 1 Adapter 的自包含接续提示词。每份可直接粘贴到新 Cursor 会话。**全部提示词以 `blocked` 起步**：共享接口未冻结、Console/M5/平台/ADR/AGPL/契约 gate 未满足前，只允许 informative 文档/计划/核验工作，不得启动编码或用 mock 解阻。

## 使用

1. 每份提示词已内联公共前缀（源头 [../common-prefix.md](../../../docs/prompts/common-prefix.md)）。
2. 先读 canonical 文档（[clients/agent-hub/docs/README.md](../docs/README.md)）、Master 计划（[../../plan/agent-hub-development-plan.md](../plan/agent-hub-development-plan.md)）与对应车道计划。
3. 确认 gate 状态；未过 gate 只做文档/核验，产出证据前不声明实现/测试/Profile。

## 宏车道提示词

- [lane-governance.md](./lane-governance.md)
- [lane-contract-capability.md](./lane-contract-capability.md)
- [lane-host-control-ledger.md](./lane-host-control-ledger.md)
- [lane-process-terminal.md](./lane-process-terminal.md)
- [lane-session-file.md](./lane-session-file.md)
- [lane-credential-workspace-verifier.md](./lane-credential-workspace-verifier.md)
- [lane-relay-pairing.md](./lane-relay-pairing.md)
- [lane-desktop.md](./lane-desktop.md)
- [lane-ios.md](./lane-ios.md)
- [lane-android.md](./lane-android.md)
- [lane-multi-agent.md](./lane-multi-agent.md)
- [lane-quality-release-migration.md](./lane-quality-release-migration.md)

## Adapter 提示词

- [adapter-codex.md](./adapter-codex.md)
- [adapter-opencode.md](./adapter-opencode.md)
- [adapter-claude-agent-sdk.md](./adapter-claude-agent-sdk.md)
- [adapter-hermes.md](./adapter-hermes.md)
- [adapter-openclaw.md](./adapter-openclaw.md)
- [adapter-openhands.md](./adapter-openhands.md)
