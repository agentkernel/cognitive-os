# Source Ledger — 条款与许可

> 类别：informative research source ledger（非法律意见）｜ 查询日：2026-07-20/21 ｜ owner：Lane-CON
>
> 本 ledger 登记供应商服务条款与开源/source-available 许可事实。不构成法律结论或法务批准；任何复用前须过 [../security/licensing-and-terms.md](../security/licensing-and-terms.md) 的法务 gate。**POC-LIC-001..003 仍为 not-run**（评估材料已整理，法务评估未执行）。填充每条须含标题/URL/查询日/version。

## 1. 供应商服务条款与软件许可（Tier 1）

| Agent / 供应商 | 软件许可（文档级已核） | 服务条款 / 允许性问题 | 状态 |
|---|---|---|---|
| OpenAI Codex | 仓库与 `@openai/codex-sdk`：**Apache-2.0**（https://github.com/openai/codex ；查询日 2026-07-20） | 第三方 Host 启动/接管/读取 session；ChatGPT 订阅路径下 Host「二次包装」是否触 ToS；凭据处理；rate limit/配额 | 软件许可已核；**服务条款允许性仍待法务评估（外部阻断候选）** |
| OpenCode | **MIT**（https://github.com/anomalyco/opencode ；查询日 2026-07-20）；开源构建 vs Zen 托管服务见 https://opencode.ai/legal/terms-of-service （2026-03 更新） | 第三方 server/session 使用；工具本身无账号墙（模型 BYO） | 软件许可已核；托管服务条款按需另评 |
| Anthropic Claude Agent SDK | **TS SDK：Anthropic Commercial ToS**（npm README License 节；非 OSS，不可再分发捆绑二进制）；**Python wrapper：MIT**（驱动的 Claude Code CLI 仍受 Commercial ToS） | 第三方 Host 启动/接管；订阅（非 API key）路径自动化是否属「另行明确许可」；native JSONL 只读；凭据不共享 | **TS Commercial ToS 已核（文档级）**；订阅自动化需 Anthropic 书面确认（外部阻断） |
| Hermes Agent | **MIT**（https://github.com/NousResearch/hermes-agent ；查询日 2026-07-20） | 工具无账号墙；模型 BYO / Nous Portal / OpenRouter；Nous Portal ToS 本轮未逐条取证 | 软件许可已核；Portal ToS unverified |
| OpenClaw | **MIT**（OpenClaw Foundation；LICENSE 原文 + THIRD_PARTY_NOTICES.md 随附义务；https://github.com/openclaw/openclaw ） | 非 source-available；模型 BYO；消息渠道平台 ToS 另评 | 软件许可已核；渠道 ToS 按需另评 |
| OpenHands | **software-agent-sdk MIT**；**agent-canvas MIT**；旧主仓库 `enterprise/`：**PolyForm Free Trial 1.0.0**（每年最多 30 天、**禁分发**） | 第三方 Host 对 Agent Server/ACP 的程序化使用（官方设计支持）；Cloud 条款仅在使用托管时适用 | SDK/Canvas MIT 已核；enterprise 目录维持不复用红线 |

共同约束（产品设计遵循）：使用官方接口、不绕过登录/计费/安全/组织策略、不抽取/存储/传输凭据（除官方 opaque profile）、不写 provider 未公开支持的内部数据、不上传私有 binary/hash。只读本机 session 文件：六家条款均未发现明确禁止（ToS 主要约束云端 Services）——gate 仍由技术风险（双 writer、版本漂移）驱动。

## 2. 开源 / source-available 许可（含竞品，仅参考不复用代码）

| 项目 | 许可 | 复用性提示 |
|---|---|---|
| Paseo（`getpaseo/paseo`） | **AGPL-3.0-or-later**（`package.json` metadata）；根 LICENSE = 自定义头 + AGPL-3.0 全文（GitHub 可能显示 Other/NOASSERTION） | 复用须过 AGPL 法务 gate；§13 网络交互须提供 Corresponding Source；非 AGPL 复用需与版权人协商双许可 |
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

Paseo = `getpaseo/paseo`；LICENSE 核验：AGPL-3.0（全文）+ package metadata **or-later**；分发源码/object code 各有义务；§13 网络交互须显著提供 Corresponding Source；第三方组件服从原许可。复用方向与 gate 见 [../security/licensing-and-terms.md](../security/licensing-and-terms.md) 与 [CONSOLE-AGENTHUB-V1-DEC-024](../decisions/decision-log.md)。

## 4. 填充规则

- source-available / 附加条款（Commons Clause、Elastic、PolyForm、Open WebUI License、Commercial ToS）不得当作无限制开源。
- 无 LICENSE 的项目视为无复用授权。
- 复用任何代码/文档/测试/协议实现前，法务 gate 未过则对应任务 `blocked`。
- 「文档级已核」≠ 法务批准；POC-LIC 三项在法务评估执行并留证前保持 `not-run`。
