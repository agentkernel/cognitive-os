# Source Ledger — 条款与许可

> 类别：informative research source ledger（非法律意见）｜ 查询日：2026-07-20 ｜ owner：Lane-CON
>
> 本 ledger 登记供应商服务条款与开源/source-available 许可事实。不构成法律结论；任何复用前须过 [../security/licensing-and-terms.md](../security/licensing-and-terms.md) 的法务 gate。填充每条须含标题/URL/查询日/version。

## 1. 供应商服务条款（Tier 1，待逐项核验）

| Agent / 供应商 | 需核验的条款问题 | 状态 |
|---|---|---|
| OpenAI Codex | 第三方 Host 启动/接管/读取 session；凭据处理；rate limit/配额 | 待核验 |
| OpenCode | 开源许可 SPDX；第三方 server/session 使用 | 待核验 |
| Anthropic Claude Agent SDK | 第三方 Host 启动/接管；native JSONL 读取；凭据 | 待核验 |
| Hermes Agent | 许可与条款存在性 | 待核验 |
| OpenClaw | 许可与条款（是否 source-available/附加条款） | 待核验 |
| OpenHands | MIT（Canvas/SDK）；PolyForm Free Trial（旧 enterprise 目录，禁分发） | 部分已核（见 dossier） |

共同约束（产品设计遵循，事实待逐条补齐）：使用官方接口、不绕过登录/计费/安全/组织策略、不抽取/存储/传输凭据（除官方 opaque profile）、不写 provider 未公开支持的内部数据、不上传私有 binary/hash。

## 2. 开源 / source-available 许可（含竞品，仅参考不复用代码）

| 项目 | 许可 | 复用性提示 |
|---|---|---|
| Paseo | AGPL-3.0-or-later（+第三方组件） | 复用须过 AGPL 法务 gate；§13 网络交互须提供 Corresponding Source |
| Happy | MIT | — |
| Vibe Kanban | Apache-2.0 | 项目已 sunset |
| Agent Deck | MIT | — |
| Agent of Empires | MIT | — |
| Claude Squad | AGPL-3.0 | AGPL 义务 |
| amux | MIT + Commons Clause | source-available，禁售功能性服务 |
| tmux-agent-tools | 无 LICENSE | 无可依赖复用授权 |
| Nimbalyst | client MIT；sync server 许可未公开 | sync server 复用前须核验 |
| Omnara（旧） | Apache-2.0（归档）；当前产品闭源 | 旧许可不外推当前产品 |
| Opcode/Claudia | AGPL-3.0 | AGPL 义务 |
| Open WebUI Core | Open WebUI License（>50 用户限制） | 非 OSI |
| Open WebUI Computer | Open Use License（基于 Elastic 2.0） | source-available |
| Open Terminal | MIT | — |

## 3. AGPL / Paseo 复用义务（详见安全文档）

Paseo LICENSE 核验：AGPL-3.0-or-later；分发源码/object code 各有义务；§13 网络交互须显著提供 Corresponding Source；第三方组件服从原许可。复用方向与 gate 见 [../security/licensing-and-terms.md](../security/licensing-and-terms.md) 与 [CONSOLE-AGENTHUB-V1-DEC-024](../decisions/decision-log.md)。

## 4. 填充规则

- source-available / 附加条款（Commons Clause、Elastic、PolyForm、Open WebUI License）不得当作无限制开源。
- 无 LICENSE 的项目视为无复用授权。
- 复用任何代码/文档/测试/协议实现前，法务 gate 未过则对应任务 `blocked`。
