# Agent Hub 风险 / Blocker 登记

> 类别：plan（informative）｜ 日期：2026-07-20（2026-07-21 Phase 0 外部阻断回填）｜ owner：Lane-CON
>
> 登记开发风险与 blocker；与仓库 [findings-ledger](../../../docs/traceability/findings-ledger.md) 的关系：Agent Hub 特有风险在此，触碰全仓 F/IMP 时同步 ledger。

## 风险

| ID | 风险 | 对冲 | 状态 |
|---|---|---|---|
| AH-R1 | Direct 记录被误当 CognitiveOS authority | GOV 车道冻结术语/完成语言；威胁 TM-019；**POC-GOV-001 planned** | open |
| AH-R2 | 供应商接口频繁变更（wrapper 脆弱，Omnara 反例） | 优先官方 SDK/App Server；跨版本漂移进 Adapter 未决；CTR gate；AH-CTR-04 样本（Claude V2 移除、OpenCode 存储迁移） | open |
| AH-R3 | 无 exclusive lease 的双 writer 写坏 session | SESS 只在证明单 writer 时写；TM-006；**POC-HOST-001 / POC-SESS-002** | open |
| AH-R4 | same-UID 恶意进程绕过 Host | 文档披露限制；强隔离需独立 principal；不虚称强隔离 | open |
| AH-R5 | Paseo/AGPL 复用触发源码提供义务 | 法务 gate 前只 clean-room 借鉴；DEC-024；§13 两要件已材料化 | open |
| AH-R6 | 平台差异导致虚假安全声明（Job vs 进程组 vs cgroup） | 逐平台分别声明与测试，不跨平台外推 | open |
| AH-R7 | 手机静默越权 | 高后果动作 PC-local 确认；TM-013 | open |
| AH-R8 | Relay MITM/replay/丢失设备 | E2EE + matching code + anti-replay + 单设备 revoke；TM-010/011/012；**POC-RELAY-005 planned** | open |
| AH-R9 | 计数/文档漂移（如长期分支中的旧向量数） | 从全局 PROGRESS 读实测数；docs-sync 联动；分支集成编号冲突与 61/84 对齐记录于 findings-ledger D-019 | mitigated |
| AH-R10 | Hermes/OpenClaw 接口事实缺失导致臆造 | 2026-07-21：文档级一手已回填；Hermes 指认 decided；**runtime PoC 仍缺** | mitigated（文档级）/ open（PoC） |
| AH-R11 | 「21 威胁项实测」类越级措辞 | 统一改为「21 项威胁已规范登记，oracle/evidence 全 not-run」；见 threat-test-oracles.md | mitigated（口径） |

## Blocker（全局 gate）

| ID | Blocker | 阻断范围 | 解除条件 | 2026-07-21 注记 |
|---|---|---|---|---|
| AH-B1 | Console 后端组 1/2/7 + M5 未交付 | 全部实现车道 | 后端交付 + M5 出口 | 仍阻断 |
| AH-B2 | 平台 PoC/GA gate 未留证 | 目标平台实现 | 真实 API/OS PoC pass | 仍阻断 |
| AH-B3 | 技术栈 ADR 未批准 | HOST/DESK/RELAY/IOS/AND | ADR 批准 | 仍阻断 |
| AH-B4 | 接口一手核验 / runtime 确认 | 对应 Adapter + 高层能力 | provider-interfaces-ledger 文档级补齐 **且** 关键 runtime PoC | **文档级已完成**；runtime/PoC 仍阻断 Adapter 实现 |
| AH-B5 | Paseo/AGPL 法务未过 | 复用 Paseo 的车道 | 法务评估通过（POC-LIC pass） | 材料已整理；评估 **not-run** |
| AH-B6 | Governed 契约缺失 | Governed 迁移/受治理 Adapter | Lane-CTR 登记 `REQ-AGENT-*` 等 | 仍阻断 |

## 外部阻断（法务/权利人/平台协议）

| ID | 阻断项 | 影响 | 状态 |
|---|---|---|---|
| AH-EXT-01 | **Anthropic**：订阅（非 API key）路径下第三方 Host 驱动 Claude Code 是否属「另行明确许可」 | Claude Adapter 订阅路径；POC-LIC-002 | open（需 Anthropic 确认） |
| AH-EXT-02 | **OpenAI**：以 ChatGPT 计划为底时 Host「二次包装」Codex 的 ToS 意见；API key 路径风险较低 | Codex Adapter 订阅路径；POC-LIC-002 | open（需法务意见） |
| AH-EXT-03 | **Apple PLA**（Developer Program License Agreement）及 iOS 分发义务核验 | iPhone 客户端候选路径 | open（需开发者账号取证） |
| AH-EXT-04 | **非 AGPL 复用 Paseo**：须与版权人协商**双许可** | 若产品拒绝 AGPL 传染仍想复用代码 | open（外部权利人） |

## P0 门禁

任何开放 P0（本登记或全仓 findings-ledger）未闭合前，对应子系统不得进入实现里程碑。
