# Agent Hub — Master Development Plan

> 类别：plan（informative，Lane-CON 激活前例外）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 状态：`planned / implementation not-implemented / evidence none / all lanes blocked`
>
> 本计划编排 Agent Hub（Direct Takeover + CognitiveOS Governed）的开发车道、依赖与 gate。**当前不启动任何实现**：所有实现任务默认 `blocked`，直至满足 [gate](#4-全局-gate)。canonical 产品/架构/安全设计见 [apps/cognitiveos-console/docs/agent-hub/](../docs/README.md)。

## 1. 范围与不变量

- 只两种部署模式：Direct Takeover、CognitiveOS Governed；无中间 `cognitive-kernel` 模式。
- 概率组件（Lead/LLM/ranker）只产 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing、最终提交由确定性代码执行。
- Direct 产品对象不冒充 CognitiveOS authority；完成语言分开；迁移只 evidence-only。
- 首发：Direct、Windows、iPhone 先于 Android；Tier 1 六 Agent 各自独立 gate。
- 不实现 L6（v1 阻断）、不实现 L8（永久禁止）。

## 2. 车道地图

| 车道 | 代码 | 计划 | 关键交付 |
|---|---|---|---|
| 产品治理 | GOV | [agent-hub/lane-governance.md](lane-governance.md) | 术语/状态/完成语言/文案守则、决策同步 |
| 合同与能力协商 | CTR | [agent-hub/lane-contract-capability.md](lane-contract-capability.md) | 能力模型、接口核验、Governed `REQ-AGENT-*` 对接 |
| Host/Control/Ledger | HOST | [agent-hub/lane-host-control-ledger.md](lane-host-control-ledger.md) | Takeover Host、控制面、ownership generation、lease、ledger |
| Process+Terminal | PROC | [agent-hub/lane-process-terminal.md](lane-process-terminal.md) | spawn/身份/containment/停止、ConPTY/pty |
| Session+File | SESS | [agent-hub/lane-session-file.md](lane-session-file.md) | 官方 session 采用、只读文件观察 |
| Credential+Workspace+Verifier | CRED | [agent-hub/lane-credential-workspace-verifier.md](lane-credential-workspace-verifier.md) | opaque handle、worktree、checks |
| Relay/Pairing | RELAY | [agent-hub/lane-relay-pairing.md](lane-relay-pairing.md) | E2EE Relay、配对、设备身份、恢复 |
| Desktop | DESK | [agent-hub/lane-desktop.md](lane-desktop.md) | Windows 首发桌面客户端 |
| iOS | IOS | [agent-hub/lane-ios.md](lane-ios.md) | iPhone companion |
| Android | AND | [agent-hub/lane-android.md](lane-android.md) | Android phone companion |
| Multi-Agent | MULTI | [agent-hub/lane-multi-agent.md](lane-multi-agent.md) | Lead+Workers 调度器 |
| Quality/Release/Migration | QRM | [agent-hub/lane-quality-release-migration.md](lane-quality-release-migration.md) | PoC、发布 gate、Direct→Governed 迁移 |

### Tier 1 Adapter 子车道

| Adapter | 计划 |
|---|---|
| Codex | [agent-hub/adapter-codex.md](adapter-codex.md) |
| OpenCode | [agent-hub/adapter-opencode.md](adapter-opencode.md) |
| Claude Agent SDK | [agent-hub/adapter-claude-agent-sdk.md](adapter-claude-agent-sdk.md) |
| Hermes | [agent-hub/adapter-hermes.md](adapter-hermes.md) |
| OpenClaw | [agent-hub/adapter-openclaw.md](adapter-openclaw.md) |
| OpenHands | [agent-hub/adapter-openhands.md](adapter-openhands.md) |

支撑文档：[README](README.md)、[progress](progress.md)、[milestones](milestones.md)、[dependency-dag](dependency-dag.md)、[risk-register](risk-register.md)、[evidence-index](evidence-index.md)。

## 3. 里程碑（高层）

| 里程碑 | 目标 | 出口 gate |
|---|---|---|
| AH-M0 | 文档、计划、提示词、契约核验清单（本轮） | 文档一致、gate 明确 |
| AH-M1 | 接口一手核验 + 威胁模型 + Open PoC 全部执行留证 | 六 Adapter 接口核验、PoC pass、法务 gate |
| AH-M2 | Host/Control/Ledger + Process/Terminal 骨架（Windows） | HOST/PROC 安全负例 pass |
| AH-M3 | Session/File + Credential/Workspace/Verifier | SESS/CRED 安全负例 pass |
| AH-M4 | Desktop Direct v1（单 Agent 全旅程） | 桌面无障碍 + 安全 + 恢复证据 |
| AH-M5 | Relay/Pairing + iPhone companion | Relay 安全负例 + iOS 无障碍 |
| AH-M6 | Multi-Agent + Android + Governed 迁移 | 全 PoC pass、发布 gate |

里程碑细节见 [milestones.md](milestones.md)；依赖见 [dependency-dag.md](dependency-dag.md)。

## 4. 全局 gate

任一实现任务在下列 gate 未全满足前保持 `blocked`：

1. **Console 后端 gate**：`docs/plan/DEVELOPMENT-PLAN.md` Console 依赖组 1/2/7 已交付且过 M5 出口评审。
2. **平台 PoC gate**：目标平台 Open PoC / GA gate 用真实 API/真实 OS 行为留可复现证据。
3. **ADR gate**：Agent Hub 技术栈（Host 语言/运行时、客户端框架、Relay）ADR 已批准。
4. **接口 gate**：目标 Agent 官方接口一手核验（[sources/provider-interfaces-ledger.md](../docs/sources/provider-interfaces-ledger.md)）。
5. **法务 gate**：Paseo/AGPL 与第三方组件义务评估通过（若复用）；供应商条款允许性核验。
6. **契约 gate**：Governed 相关行为的适用 `REQ-*`/schema/vector 已登记（经 Lane-CTR），不新增违反冻结的规范表面。

禁止用 mock、原型、代码存在或 Agent 自述冒充 gate 通过或任务完成。

## 5. 分批与提交纪律

- 逐路径 `git add`，禁 `git add -A`；不覆盖他人未提交改动。
- 每个 PR 关联 REQ-ID / PRD / DEC / F-IMP 或文档条目；确无关联时写明原因。
- 会话结束写 handoff 到 `docs/checkpoints/`。
