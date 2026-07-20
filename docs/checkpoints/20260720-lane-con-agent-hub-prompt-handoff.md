# 20260720 Lane-CON Agent Hub Prompt Handoff

## 1. 本次会话完成

- 新增 `docs/prompts/console-agent-hub-direct-mode-product-design.md`：可直接粘贴到新 Cursor 会话的 PC/手机 Agent Hub 全方位产品、体验、架构、安全、平台设计与未来开发任务编排提示词。
- 提示词显式区分直连、内核增强、完整治理三种产品部署模式；它们不是新增 normative Profile。无 OS 时只使用带来源的产品状态；裸 `cognitive-kernel` 不被推定为完整 authority。
- 提示词要求在同一轮并发启动仓库边界、Agent 接口、凭据条款、竞品痛点、Host/远控安全、跨端 UX 六类只读子代理，并通过交互式问题每轮确认 1–2 个关键决策。
- Adapter 研究范围覆盖 OpenClaw、Hermes Agent、Codex、Claude Code、WorkBuddy/CodeBuddy、OpenCode、Goose、Gemini CLI、Aider、Cline、OpenHands 及其他主流 Agent；逐能力区分官方结构化接口、开放标准、wrapper、进程观察、launch-only 与 policy blocker。
- 凭据方向已收口为“连接账户”而非 token/cookie 导入；Claude/Gemini 订阅 OAuth、WorkBuddy/CodeBuddy 身份边界、Roo Code 停运、真正 pause 与 interrupt/resume 的区别均写入提示词。
- 提示词要求产出 product design、Adapter matrix、security/credentials、journeys、collaboration、decision log、platform parity，以及只编排不执行的开发任务 DAG 和分车道自包含提示词。
- 更新 `docs/README.md` 提示词索引和 `docs/plan/PROGRESS.md`。核心提交：`6617ee0`。
- 本批是 prompt/informative 文档，无关联 REQ/F/IMP；未修改 normative 机器资产或实现代码。

## 2. 未完成 / 进行中

- Agent Hub 产品研究、交互决策、产品正文、架构选择、开发任务 DAG 与分车道接续提示词尚未执行。
- Tier 1 Agent、首发 PC/手机平台、Relay 形态、电脑控制范围、账号连接、首版群组模型和直连完成判定仍须在执行提示词时逐轮确认。
- Console 依赖组 1/2/7、M5 出口和目标平台实现 gate 均未满足；不得据此启动 Console、Host、Adapter、Vault、Relay 或移动实现。

## 3. 测试与证据状态

- `pnpm run check:consistency`：通过；273 requirements、55 error codes、56 schemas、76 vectors，Markdown links 与 traceability 已验证。
- `git diff --cached --check`：核心提交前通过。
- ReadLints：提示词、文档索引和 PROGRESS 无诊断。
- 公开资料研究使用官方文档、官方仓库和供应商认证/法律页面；它们是提示词编写依据，不是产品实现或平台测试证据。
- 未执行 Agent 集成、模拟器、真机、Relay、凭据、远控、无障碍或安全 PoC。
- 76 个既有 conformance vectors 状态未改变，仍全部为 `not-run`；Console implementation 未提供，Profile 未符合。

## 4. 未决风险与漂移

- 未发现需要修改 normative 资产的新漂移；产品依赖缺口不自动构成 registry 漂移。
- 无 CognitiveOS 与 kernel-only 不能静默降级/升级为完整治理语义；裸内核还缺 store、policy/authority、Effect WAL/对账、executor、verifier 与 transport。
- Claude 第三方产品默认应使用 API key/受支持云提供商；Gemini CLI OAuth 不得被第三方 piggyback；Codex 只走官方认证面；WorkBuddy 与 CodeBuddy API/账号能力不能互相外推。
- Roo Code 已停运归档，不进入新适配范围；Opcode/Claudia、Omnara 旧/新版、Nimbalyst、Open WebUI 等项目身份、维护和许可证在执行日仍须重新核实。
- 多 Provider 管理不自动等于 Multi-Agent；委派、handoff、父子任务和自动编排仍受 M9/Phase I 边界约束。
- 退出码、Agent reported done、interrupt、进程 kill、通知或 Relay receipt 均不是 Verification/acceptance；只有官方接口明确支持时才使用持久 pause/resume 文案。

## 5. 下一步入口

- 建议提示词：`docs/prompts/console-agent-hub-direct-mode-product-design.md`
- 工作范围：Lane-CON informative 文档例外；实现仍无可用车道。
- 第一个动作：在新 Cursor 会话粘贴提示词，完成仓库接入后并发启动六类只读研究；随后用交互式问题确认首要用户与三种部署模式的产品位置。

## 6. 快照

- PROGRESS 已更新：是。
- Agent Hub 状态：提示词已提供；产品设计/开发编排未执行；implementation `not-implemented`；product evidence `none`；Profile 未符合。
- 本次核心提交：`6617ee0`。
- 本 handoff 与 PROGRESS 最新入口提交哈希见会话最终报告。
