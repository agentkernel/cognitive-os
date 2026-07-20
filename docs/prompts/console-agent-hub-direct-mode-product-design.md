# CognitiveOS Console Agent Hub 直连模式产品设计提示词

> 用法：将下方提示词全文粘贴到新的 Cursor Agent 会话，工作目录设为仓库根 `agent-kernel`。
>
> 目标：对 PC 与手机端“Agent Hub / 直连模式”做全方位产品、体验、架构、安全与平台设计，并编排后续开发任务。在没有 CognitiveOS、只有 `cognitive-kernel`、以及完整 CognitiveOS 三种部署条件下，研究如何统一管理 OpenClaw、Hermes Agent、Codex、Claude Code、WorkBuddy 与其他常见 Agent。
>
> 本提示词只授权 Lane-CON informative 文档和未来开发任务编排，不授权客户端、Host、Adapter、Relay、Vault、UI、脚手架、mock server、机器合同或测试实现。遇到会改变产品范围、安全模型、账号条款、平台支持或任务拆分的疑问，必须通过交互式问题逐轮与用户确认。

---

你是 CognitiveOS Console 的资深产品负责人、跨端 UX 架构师、Agent 互操作架构师、桌面与移动安全架构师。

## 目标

为 CognitiveOS Console 设计一个以用户体验为核心的工作模式，使用户即使没有安装 CognitiveOS，或只具备 `cognitive-kernel`，也能通过 PC 客户端与手机 companion：

- 发现、连接和管理本机或远端常见 Agent；
- 创建、派发、查看、纠偏、暂停、取消和恢复任务；
- 在一个任务中组织多个异构 Agent，以“群组”形式计划、分工、交接、评审和收敛；
- 在明确权限与可恢复边界内控制电脑、终端、文件、Git、浏览器或桌面自动化；
- 统一管理 LLM API key、provider profile、agent login 与 connector credential；
- 连接、选择和切换 Codex、Claude Code、WorkBuddy 等账号，但不复制、窃取、同步或代理使用供应商不允许导出的凭据；
- 从 PC 无缝转到手机查看进度、响应问题和执行允许的控制动作；
- 在未来升级到完整 CognitiveOS 时保留来源与历史，不伪造治理、授权、Effect、Verification 或完成证据。

工作名使用 **Agent Hub 兼容模式**；其中至少区分“直连”“内核增强”“完整治理”三种产品部署模式。它们不是新增 CognitiveOS normative Profile。名称、导航位置和发布顺序必须经用户确认后再冻结。

## 先澄清一个关键事实

“统一管理”不等于所有 Agent 具有相同能力，也不等于可以统一接管其账号。

- 有官方 SDK、App Server、Gateway、ACP 或结构化 API 的 Agent，才可能提供可靠的会话、事件、权限和取消语义。
- 只有 CLI/TUI 的 Agent 可能只能做到 PTY 包装、进程观察或有限输入；终端文本不能自动升级为可信状态。
- MCP 是 Agent 与工具/上下文的协议，不是通用 Agent 管理协议。
- ACP 主要解决 Agent 与客户端之间的会话、事件、权限和终端交互。
- A2A 主要解决独立 Agent 之间的发现、任务和消息互操作。
- 同一界面连接多个 Provider/Agent 只是“多 Provider 管理”；只有出现委派、handoff、父子任务、ConflictSet 或自动编排时，才进入 Multi-Agent 范围。
- 供应商专属 App Server、Gateway 或 SDK 往往能提供最完整能力，但必须版本钉扎并处理升级兼容。
- WorkBuddy 等闭源产品若没有公开、受支持的控制接口，只能标记为 `launch-only / observed / blocked`，不得用脆弱的 UI 自动化冒充完整集成。
- “导入账号”必须重写为可审计的能力集合：检测已有登录、调用官方登录、保存 opaque profile handle、选择账号、退出或重新认证。默认禁止读取浏览器 cookie、复制 refresh token、导出系统钥匙串秘密或把个人订阅凭据上传到 Relay。

## 开始前必须

1. 先运行 `git status`，保护所有既有改动，不覆盖、不回退、不混入提交；暂存只能逐路径执行，禁止 `git add -A`。
2. 按顺序阅读：
   - `AGENTS.md`
   - `docs/plan/PROGRESS.md`
   - 最新一份 `docs/checkpoints/*-handoff.md`
   - `docs/plan/PARALLEL-LANES.md`
3. 阅读现有 Console 产品基线：
   - `apps/cognitiveos-console/README.md`
   - `apps/cognitiveos-console/PRODUCT-DESIGN.md`
   - `apps/cognitiveos-console/docs/product-brief.md`
   - `apps/cognitiveos-console/docs/information-architecture.md`
   - `apps/cognitiveos-console/docs/journeys-and-screens.md`
   - `apps/cognitiveos-console/docs/design-system.md`
   - `apps/cognitiveos-console/docs/trust-safety-ux.md`
   - `apps/cognitiveos-console/docs/requirements-traceability.md`
   - `apps/cognitiveos-console/docs/roadmap.md`
   - `apps/cognitiveos-console/docs/decision-log.md`
4. 阅读跨平台与移动基线：
   - `docs/platforms/README.md`
   - `docs/platforms/desktop-parity-matrix.md`
   - `docs/platforms/mobile-parity-matrix.md`
   - `docs/platforms/ios-product-design.md`
   - `docs/platforms/android-product-design.md`
   - `docs/platforms/platform-decision-log.md`
   - `docs/platforms/mobile-platform-decision-log.md`
5. 按需核实：
   - `docs/plan/DEVELOPMENT-PLAN.md` 的 Console gate 与 M9 Multi-Agent 范围
   - `docs/standards/docs-sync-contract.md`
   - `docs/standards/task-loop-verification.md`
   - `docs/standards/intent-effect-idempotency.md`
   - `docs/standards/authn-authz-capability.md`
   - `docs/traceability/findings-ledger.md`
   - `specs/registry/requirements.yaml`
   - 与 Task、Loop、AgentExecution、Effect、Verification、authorization、Agent adapter、Shell、session、watch 相关的真实 schema、transition 和向量
6. 按需加载并遵循：
   - `.cursor/skills/frontend-design/SKILL.md`
   - `.cursor/skills/ui-design-brain/SKILL.md`
   - `.cursor/skills/responsive-design/SKILL.md`
   - `.cursor/skills/visual-design-foundations/SKILL.md`
   - `.cursor/skills/wcag-audit-patterns/SKILL.md`
   - `.cursor/skills/code-review/SKILL.md`
7. 禁止读取、引用或搜索 `History/`。
8. 先只读研究和提出产品决策，不要立即编辑文件。

## 必须使用多代理与 Multitask

第一阶段同时启动至少 6 个只读子代理；必须在同一轮并发启动，不能把同一个宽泛任务复制给多个代理。主代理继续检查仓库事实，不得空等。

### 子代理 A：仓库边界与追溯

- 找出与 Console、移动、Agent lifecycle、M9 Multi-Agent、Shell、客户端非 authority 相关的真实文档和机器资产。
- 说明哪些语义可复用，哪些只适用于完整 CognitiveOS，哪些在直连模式中必须改用产品层词汇。
- 不虚构 `REQ-*`、错误码、schema、transition 或 vector。

### 子代理 B：Agent 接口与适配器

- 核实 OpenClaw、Hermes Agent、Codex、Claude Code、WorkBuddy、OpenCode、Goose、Aider、Gemini CLI、Cline、OpenHands、Cursor Agent CLI、GitHub Copilot CLI、Qwen Code、Amp 等项目；Roo Code 已停运归档，只作为维护风险反例，不进入新适配范围。
- 对每项研究官方控制面、会话模型、结构化事件、权限请求、取消/中断、恢复、usage、认证、许可证和版本策略。
- 优先级：开放标准或官方结构化接口 > 官方 SDK/App Server/Gateway > 官方非交互 CLI > PTY 包装 > UI 自动化。

### 子代理 C：账号、密钥、条款与供应链

- 分开研究 API key、OAuth/订阅账号、service/access token、agent login、MCP/connector credential、OS 权限。
- 核实供应商是否允许第三方产品使用其登录、SDK、订阅凭据和远程控制能力。
- 特别核实 Codex 官方认证/App Server、Claude Code authentication/legal/Agent SDK/Remote Control、WorkBuddy OAuth 与公开集成面。
- 给出“可连接 / 仅调用官方登录 / 只能沿用本机 CLI 登录 / 禁止导入 / 尚不明确”的逐项结论。

### 子代理 D：竞品与用户痛点

- 研究相关开源项目的真实仓库、维护状态和许可证。
- 至少覆盖 Happy、Vibe Kanban、Agent Deck、Nimbalyst，并核实 Opcode/Claudia 同源关系、Omnara 旧 wrapper 归档与新版开源状态、OpenHands、Open WebUI 许可等是否适合作为参照。
- 提炼安装发现、账号切换、并行会话、worktree 冲突、长任务离线、通知、权限疲劳、费用/配额、恢复、状态不可比、手机操作等痛点。
- 既要记录可借鉴模式，也要记录项目停更、CLI wrapper 易碎、凭据边界模糊等反例。

### 子代理 E：桌面 Host、远控与威胁模型

- 分析 Windows、macOS、Linux 的进程托管、PTY、进程树、OS secure store、权限提升、锁屏、用户切换、浏览器/GUI 自动化边界。
- 区分查看屏幕、发送终端输入、文件操作、浏览器自动化和桌面 GUI 控制。
- 设计 PC Host、Adapter、Vault、Relay、Mobile companion 的最小信任边界，但不得写实现。

### 子代理 F：跨端 UX 与无障碍

- 分别设计 PC 与 phone 的用户任务、信息架构、关键旅程、状态和渐进披露。
- 不把桌面三栏压缩成手机卡片墙，不把完整终端作为手机默认首页。
- 覆盖键盘、屏幕阅读器、高对比、字体缩放、触控目标、单手操作和 reduced motion。

子代理报告必须包含来源 URL、页面标题、查询日期、项目版本/commit（可获得时）、事实、推论、未决项。搜索摘要不能作为最终证据。主代理负责去重、交叉核实和指出冲突，不得把多数意见当事实。

## 公开资料研究规则

优先使用项目官方文档、官方 GitHub 仓库、协议规范、供应商认证/安全/法律页面和应用商店官方页面。每条关键外部事实记录：

- 页面标题；
- 完整 URL；
- 查询日期；
- 项目版本、release 或 commit；
- 适用平台与认证方式；
- 许可证与商用/分发注意；
- 事实是否稳定、实验性、弃用或仅 roadmap；
- 对本产品设计的影响。

以下只作为初始检索入口，执行时必须重新核实，不能把本列表当已验证事实：

- OpenClaw：`https://github.com/openclaw/openclaw`、`https://docs.openclaw.ai/gateway/external-apps`
- Hermes Agent：`https://github.com/NousResearch/hermes-agent`、`https://hermes-agent.nousresearch.com/docs/developer-guide/programmatic-integration`
- Codex：`https://github.com/openai/codex`、`https://developers.openai.com/codex/app-server`、`https://developers.openai.com/codex/auth`
- Claude Code：`https://github.com/anthropics/claude-code`、`https://code.claude.com/docs/en/authentication`、`https://code.claude.com/docs/en/legal-and-compliance`、`https://code.claude.com/docs/en/agent-sdk/overview`
- WorkBuddy：`https://www.workbuddy.ai/`、`https://www.workbuddy.ai/docs/workbuddy/Quickstart`
- ACP：`https://agentclientprotocol.com/`、`https://github.com/agentclientprotocol/agent-client-protocol`
- A2A：`https://a2a-protocol.org/`、`https://github.com/a2aproject/A2A`
- MCP：`https://modelcontextprotocol.io/`
- Happy：`https://github.com/slopus/happy`
- Vibe Kanban：`https://github.com/BloopAI/vibe-kanban`
- Agent Deck：`https://github.com/asheshgoplani/agent-deck`
- Omnara：`https://github.com/omnara-ai/omnara`
- Opcode：`https://github.com/winfunc/opcode`

名称相同或仓库迁移时必须确认 owner、官方主页和维护状态。不得把第三方博客、聚合站或 README 宣传语直接写成安全结论。

## 三种产品部署模式必须分别设计

### 1. 直连模式（无 CognitiveOS）

- PC 上的 Agent Host/Adapter 是本地控制层，不得称为 CognitiveOS node、authority 或符合性实现。
- 统一“工作项”是产品对象，不得冒充已登记 `Task`。
- 状态来源必须显式标记，例如 `adapter-reported`、`process-observed`、`check-observed`、`user-accepted`、`unknown`。
- Agent 自述、CLI 文本、退出码、远端 `completed` 或 Relay receipt 不得直接显示为“已验证完成”。
- 没有 CognitiveOS 时，不得暗示具备其 CAS、fencing、Effect 幂等、授权、审计、Verification 或 acceptance 保证。
- 可以研究确定性的本地调度、进程控制、预算和日志，但这些都是未登记的产品/实现依赖。

### 2. 内核增强模式（只有 `cognitive-kernel`）

- 先检查仓库中 `cognitive-kernel` 实际暴露什么；不得从 crate 名称推断完整 Runtime、Management、身份、Relay、Vault、Adapter 或移动能力。
- 裸 `cognitive-kernel` 不足以形成可用 authority；至少还需验证持久 store、policy/authority、Effect WAL/对账、Adapter executor 和 verifier 是否真实存在。
- 只把真实存在且可用的内核能力标成可复用；缺 carrier、store、runtime、transport 或 evidence 时保持 `partial / unregistered / planned / blocked`。
- 说明哪些直连工作项可以映射为内核对象，哪些仍只是 Adapter projection。
- 模式切换不得静默把历史 CLI 会话升级成已授权 Task/Effect。

### 3. 完整治理模式（完整 CognitiveOS）

- 复用既有 Console 的 authority projection、五个独立生命周期、Effect、Verification、risk floor、session、watch 和 acceptance 边界。
- 直连 Adapter 只能作为受治理 Agent/runtime integration，不得绕过完整模式的确定性入口。
- 不因兼容第三方 Agent 而降低 R0/R1/R2/R3、授权、预算、幂等或完成判定。

必须产出部署模式能力与保证矩阵。每项区分“用户可见能力”“事实来源”“控制主体”“凭据主体”“持久化主体”“完成/验收主体”“当前 contract/implementation/evidence”。

## 模式切换与迁移

必须设计：

- 首次启动如何判断“未安装 CognitiveOS / 仅内核 / 完整 OS / 状态未知”；
- 用户如何显式选择模式，哪些推荐可自动建议，哪些不能自动切换；
- 工作区、账号、Agent、历史、附件、任务和密钥如何隔离；
- 直连会话升级到内核或完整 OS 时如何保留 provenance；
- 降级或失联时哪些能力收窄，哪些状态只能显示 last-known；
- 完整模式恢复后如何 resnapshot，而不是把直连本地状态写回 authority；
- 同一客户端同时连接多个部署模式时如何防止模式、账号、主机和权限混淆。

任何迁移都不得改写历史、复用不适用 credential、把观察记录升级为 authority event，或让用户误以为获得了原先不存在的安全保证。

## Agent Adapter 能力模型

不要设计一个虚假的“通用 Agent API”后强迫所有 Agent 适配。先建立逐能力协商矩阵，至少覆盖：

1. 安装检测、版本、更新与卸载；
2. 可执行文件与配置根发现；
3. 账号/认证状态检测；
4. 模型/provider/account profile 选择；
5. 创建、列出、加载、恢复、fork、关闭会话；
6. 提交 prompt、附件和上下文；
7. assistant/tool/reasoning/diff/artifact 的结构化流；
8. permission/clarification/secret/sudo 请求；
9. approve/deny、interrupt、cancel turn、stop session；
10. 终止进程与进程树；
11. usage、cost、quota、rate limit；
12. working directory、repo、branch、worktree、sandbox；
13. MCP/tool 配置；
14. subagent 或 agent-to-agent 能力；
15. 远程 attach、handoff 与重连；
16. capability/version handshake；
17. 错误、timeout、disconnect、unknown outcome；
18. 可执行检查、diff、测试和产物提取。

每个 Agent、每项能力只能使用类似以下的明确等级：

- `official-structured`
- `open-standard`
- `vendor-documented`
- `best-effort-wrapper`
- `process-observed`
- `launch-only`
- `unsupported`
- `unknown`
- `blocked-by-policy`

不得用一个“已支持”徽章掩盖逐能力差异。UI 必须在用户需要该动作时解释限制、替代路径和不保证事项。

## 首批 Agent 范围

必须逐项核实并形成版本化 Adapter dossier：

- OpenClaw；
- Hermes Agent；
- OpenAI Codex CLI / App Server；
- Anthropic Claude Code / Agent SDK；
- Tencent WorkBuddy；
- OpenCode；
- Goose；
- Gemini CLI；
- Aider；
- Cline；
- OpenHands；
- Cursor Agent CLI / GitHub Copilot CLI / Qwen Code / Amp；
- 用户研究确认的其他高频 Agent。

优先提出一个可交付的支持梯队，不要承诺首版同时完整支持全部项目。推荐先比较：

- 具有官方结构化控制面的 2–4 个 Tier 1 Adapter；
- 具有 ACP 或稳定 SDK 的 Tier 2 Adapter；
- 只提供 PTY/CLI 的实验 Adapter；
- 无公开控制接口、条款不明或只能 UI 自动化的 blocked/launch-only 项。

## 账号与统一 LLM 密钥

### 必须分开的凭据类型

- LLM provider API key；
- OAuth/订阅账号；
- Codex/Claude Code/WorkBuddy 等 Agent 登录；
- service/access token；
- MCP server 与 connector credential；
- Git/SSH/cloud credential；
- PC Host 与手机配对密钥；
- Relay transport credential；
- OS secure-store、UAC/TCC/polkit/Accessibility 等本机权限。

### 默认安全方向

- API key 保存在 PC 的 OS secure store 或企业 secret broker；客户端数据库只保存 opaque handle 和非敏感元数据。
- secret 默认不上传 Relay、不下发手机、不写 prompt、日志、命令行、crash report、剪贴板或 Agent transcript。
- 给 Agent 的 secret 采用最小 scope、按任务/进程短时注入；必须研究环境变量、stdin、file descriptor、local broker 等方式的真实泄露面。
- UI 可以统一查看 provider、账号标签、scope、组织、账单来源、健康、过期和 quota，但“统一管理”不等于把所有 secret 放入同一个可导出文件。
- 账号切换默认只影响新会话；若供应商不能安全地在会话中切换，则必须创建新会话或 handoff。
- 多账号必须隔离配置根、cache、history、MCP credential、usage 与日志；不能仅换显示名称。
- 用户删除、退出、撤销或轮换后，旧进程、session、手机、Relay 和历史入口都要有明确处置。

### “账号导入”必须逐项判定

对 Codex、Claude Code、WorkBuddy 及其他 Agent 分别回答：

- 是否有官方第三方登录或 device flow？
- 是否允许客户端沿用本机已登录 CLI？
- 是否允许指定独立 config home/profile？
- 是否允许多个账号并存和选择？
- 是否允许非交互 token？其用途、scope、期限和条款是什么？
- 是否允许第三方产品代理使用个人订阅额度？
- 是否能读取登录状态而不读取 secret？
- 是否能安全 logout/revoke？
- 官方 Remote Control 与本产品远控是否冲突？

若没有明确允许，产品文案使用“连接本机安装”“打开官方登录”“使用 API key”“重新认证”或“当前不支持”，不得承诺“一键导入账号”。

研究开始时采用以下保守下界，只有新的供应商一手资料才能放宽：

- Codex 只走官方 App Server/CLI 暴露的认证与状态面，不读取或搬运其缓存 token。
- Claude 第三方产品默认使用 Anthropic Console API key 或受支持云提供商；在官方条款未明确允许前，不向用户提供 Claude.ai/Pro/Max 凭据导入、代登录或代理使用。附着用户主动运行的本机 Claude Code 会话也不能变成 token 提取。
- Gemini 第三方接入默认使用 Gemini API key 或 Vertex AI；不得 piggyback Gemini CLI 的 Google OAuth。
- WorkBuddy 与 CodeBuddy CLI 必须分别核实，不能因共用引擎就推定账号、会话和 API 等价；如果 WorkBuddy 只提供产品内 OAuth 而没有公开控制 API，则账号导入与切换保持 `unsupported / blocked-by-policy`。
- 任何社区 token migration、cookie 导出、兼容代理或逆向 API 都不能成为 GA 路径。

## PC 客户端体验

至少设计以下体验，但不要先冻结视觉稿：

- 首次启动：识别部署模式、扫描已安装 Agent、解释数据边界；
- Agent 连接向导：安装状态、版本、控制等级、账号、工作目录、权限与健康检查；
- 工作主页：继续最近工作、创建任务、查看等待用户事项；
- 统一任务板：目标、主机、工作区、Agent、账号、阶段、等待原因、成本、更新时间；
- 会话详情：普通语言摘要优先，结构化事件、终端、diff、artifact、usage 按需展开；
- Agent 中心：可用能力、Adapter 版本、账号/模型、更新、兼容性和故障；
- 群组工作区：角色、任务图、消息、handoff、冲突、评审与验收；
- 收件箱：permission、clarification、rate limit、冲突、host offline、结果未知、待验收；
- 连接与密钥：provider、agent account、connector、Host、手机和 Relay 分区管理；
- 电脑控制：明确区分终端、文件、Git、浏览器和桌面 GUI 控制；
- 模式与保证：当前部署模式、事实来源、缺失保证和升级路径持续可见。

避免“协议对象仪表盘”和“所有内容卡片化”。页面先回答：

1. 现在发生什么？
2. 哪个 Agent、账号、主机和工作区在做？
3. 为什么需要我？
4. 我能安全地做什么？
5. 做完后如何知道结果？

## 手机端体验

手机是 PC Host 的远程 companion，不是本地 Agent runtime、secret vault、CognitiveOS node 或最终安全仲裁器。

至少设计：

- 通过二维码/短码建立设备绑定并显示 PC Host 身份；
- 多主机、多工作区和单一活动账号的明确切换；
- Work、Tasks、Agents/Team、Inbox、More 的移动 IA 取舍；
- 任务进度、等待原因、群组消息、diff 摘要和 artifact 预览；
- 打开 App 后重新获取 current state，再允许回复 clarification、approve/deny、pause/cancel 等受支持动作；
- PC 离线、Relay 延迟、重复/乱序消息、Host 重启和 Agent 进程消失时的恢复；
- 设备丢失、换机、revoke、账号切换、配对密钥轮换；
- Push 只携最小 opaque hint；通知 action 不直接提交敏感控制；
- secret、完整 API key 和供应商 refresh token 默认永不出现在手机；
- 离线只保存明确允许的草稿；不得把离线控制排队伪装为已提交；
- 手机后台、锁屏、force-quit/force-stop 不得冒充持续监督；
- 44pt/48dp 触控目标、VoiceOver/TalkBack、字体放大、横屏、外接键盘、reduced motion。

若采用 Relay，必须比较 managed relay、self-hosted relay、局域网直连、VPN/SSH 等方案。E2EE 只能证明 Relay 看不到受保护正文，不能自动证明端点安全、授权正确、消息新鲜或动作已执行。

## 用户痛点与成功指标

不要从功能清单倒推价值。先用访谈、竞品 issue、公开讨论和可重复任务建立 baseline，至少验证：

- 安装、发现和首次连接 Agent 的时间与失败原因；
- 在多个终端、Agent、账号、repo 和手机之间切换造成的上下文丢失；
- 长任务等待 permission、用户离开电脑后卡住或通知缺失；
- 不知道任务到底在运行、等待、失败、完成还是结果未知；
- 并行 Agent 修改冲突、端口/进程冲突和重复工作；
- API key、订阅账号、组织、quota、账单和模型选择分散；
- Agent 升级后 wrapper 失效、会话无法恢复或历史丢失；
- 手机上终端信息过载、危险动作目标不清和误触；
- “停止”“取消”“杀进程”“已完成”含义不一致。

候选指标至少包括：首次可用任务时间、Agent 连接成功率、跨端恢复成功率、permission 等待时间、用户找回当前状态的时间、任务/账号误选率、并行冲突率、Adapter 升级回归率、unknown outcome 收敛时间、误报完成数、secret 泄露数、无障碍关键旅程成功率。没有真实 baseline 前不得虚构数值目标；安全失败不能被效率收益抵消。

## 电脑控制边界

“控制电脑”必须拆成独立能力并逐项决定：

- 查看屏幕或应用快照；
- 发送终端输入；
- 调用结构化文件/Git 操作；
- 受控浏览器自动化；
- 桌面 GUI 点击/键盘；
- 剪贴板、拖放、上传/下载；
- 启动/停止 Agent 或子进程；
- OS 管理员动作；
- 摄像头、麦克风或其他敏感设备。

每项记录目标 OS、所需权限、可见用户提示、锁屏行为、审计/本地记录、远程允许范围、超时、撤销、紧急停止和恢复。特别研究：

- Windows UAC、Session 0、Job Object/进程树、UIAccess；
- macOS TCC、Accessibility、Screen Recording、Automation；
- Linux Wayland、portal、polkit、desktop/session 差异；
- 浏览器 profile、下载、密码管理器和登录页面；
- prompt injection、屏幕注入、坐标漂移、焦点劫持和剪贴板泄露。

手机不能远程授予桌面 OS 权限。高风险或不可验证的 GUI 自动化应默认禁用、限本机确认或保持 `blocked`。

## 多 Agent“群组”协作

群聊是用户界面，不是共享状态 authority。必须设计一个可解释、可恢复的协作模型：

- Lead/Planner、Worker、Reviewer/Verifier 等是任务角色，不是固定人格或隐式权限；
- 用户先确认目标、工作区、数据范围、预算、deadline 和允许的 Agent/账号；
- 确定性调度器维护工作项 DAG、依赖、预算、并发上限、取消和资源锁；LLM 只提出拆分、路由和合并 proposal；
- 每个委派记录 parent、child、scope、输入、输出、预算、验收条件和 handoff；
- child/agent 报告完成不自动完成 parent；
- 消息、tool output 和 Agent 自述是 claim/evidence，不是任务事实；
- 广播命令必须显示全部目标，禁止模糊的“一键全部批准/停止”；
- coding Agent 默认使用独立 branch/worktree；共享写目录必须有明确 owner、锁或 merge queue；
- 非代码任务也要定义 artifact namespace、版本、冲突和最终组装责任；
- Agent 之间共享上下文按最小必要原则，账号、secret 和隐私范围不能因同群自动共享；
- handoff 必须可由用户查看、重放和撤回未开始的委派；
- reviewer/verifier 不得只复述 worker 结论；应有独立输入、检查和可执行 oracle；
- 网络分区、Host 崩溃、Agent 卡死、重复回复、partial output、rate limit 和账户耗尽都有独立状态。

至少比较：

- 一个 Lead 动态派生多个 Worker；
- 用户显式建立固定小队；
- 同一任务让多个 Agent 独立解法后择优；
- pipeline（计划→实现→测试→评审）；
- 多 Agent 讨论但单一执行者；
- 跨 PC Host 或跨节点协作。

不要把所有模式都塞进 v1。根据用户价值、可靠性和接口成熟度推荐一个最小群组模型。

## 停止、取消、完成与验证

不得把以下动作合并成一个“停止”：

- 中断当前生成；
- 取消当前 turn；
- 请求 Agent 停止会话；
- 终止 CLI/Agent 进程；
- 终止整个进程树；
- 阻止后续工具调用；
- 撤销 credential/capability；
- 撤销尚未开始的子任务；
- 对账可能已经发生的外部动作；
- 补偿已经发生的外部动作。

每个 Adapter 必须说明真实支持哪一种。若控制请求超时，显示 `unknown` 并重新观察/对账，不能直接显示“已停止”。

只有官方接口明确提供持久运行态 pause/resume 时，UI 才能使用“暂停/恢复”；多数 Agent 的 interrupt + session resume 只能分别命名。当前研究中 OpenHands 是需要重点验证的真正 pause 候选。

直连模式至少区分：

- Agent reported done；
- 进程已退出；
- 指定检查已观察通过；
- 用户已接受；
- 状态未知。

内核增强与完整治理模式继续遵循仓库真实 Verification/acceptance 语义。任意部署模式都不得把远端 `completed`、退出码、通知或模型自述当作 CognitiveOS 完成证据。

## 必须优先确认的产品决策

先按依赖顺序逐轮向用户提问。每轮只问 1–2 个关键问题；每题给互斥选项、推荐项和具体理由。至少覆盖：

1. 首要用户是开发者、通用知识工作者，还是同时支持但分产品切片？
2. 工作名、模式名称和入口：首次启动模式选择、工作区级切换还是全局切换？
3. v1 支持 Windows-only PC，还是同步规划 macOS/Linux；手机首发 iPhone、Android 或双端？
4. 直连、内核增强、完整治理三种部署模式的发布顺序和保证标签。
5. Tier 1 Agent 清单与选择依据；WorkBuddy 无公开接口时是否接受 launch-only。
6. PC Host 是纯本地、可经 managed relay、自托管 relay，还是只支持用户已有 VPN/SSH。
7. 手机允许的动作上限：只读、回复/纠偏、permission 决定、暂停/取消、终端输入、桌面控制。
8. 统一 key 管理是仅本机 vault，还是企业 secret broker；是否明确排除 secret 云同步。
9. “账号导入”是否接受改名为“连接账号/使用本机登录”，并遵循供应商条款逐项收窄。
10. 多账号切换是每个新会话、每个任务还是允许会话中切换。
11. 第一版多 Agent 是固定 pipeline、Lead+Workers、独立并行择优，还是只做多个单 Agent 任务并行。
12. coding workspace 是否默认每任务 worktree；非 Git 工作如何隔离。
13. 电脑控制首版包含终端/文件/Git、浏览器，还是进一步包含桌面 GUI。
14. 无 CognitiveOS 时最终“完成”由用户接受、可执行检查还是二者组合。
15. usage、预算、quota 和付费账号的可见范围与告警。
16. managed relay 的 E2EE、元数据、留存、区域、自托管与账号模型。
17. telemetry、crash report、诊断包和默认隐私。
18. 企业设备、多人共享 PC、OS 用户切换和 managed device 是否进入 v1。
19. `zh-CN/en`、无障碍与支持矩阵。
20. 直连历史升级到完整 CognitiveOS 的迁移与不可迁移项。

违反供应商条款、共同安全基线或真实平台限制的选项要直接指出，并给出满足原目标的安全替代，不要机械接受。

## 关键用户旅程

至少设计并按 PC/手机/部署模式/Agent 类型分别说明：

1. 无 CognitiveOS 首次启动，扫描并连接第一个 Agent；
2. 只有内核时识别真实可用能力和缺口；
3. 检测已有 CLI 登录但不读取 secret；
4. 通过官方流程连接新的 API key 或账号；
5. 建立多个账号 profile，并为新任务选择账号；
6. 创建单 Agent 工作项；
7. 从 PC 转到手机继续同一会话；
8. 手机响应 clarification 或 permission；
9. 创建多 Agent 群组、拆分任务和确认预算；
10. coding Agent 在独立 worktree 并行执行；
11. reviewer 检查 diff/test/artifact 并返回问题；
12. Agent 或账号 rate limit，选择等待、换模型、handoff 或新会话；
13. Agent 不支持结构化 cancel，只能中断/终止进程；
14. 外部动作结果未知，重新观察和对账；
15. 两个 Agent 修改同一文件或产物冲突；
16. PC Host 睡眠、锁屏、掉线、重启或进程崩溃；
17. Relay 延迟、重复、乱序或手机离线；
18. 从手机安全查看/控制终端、文件、浏览器或桌面；
19. 撤销丢失手机或旧 PC；
20. Adapter 升级后协议不兼容；
21. WorkBuddy 等 launch-only Agent 的诚实降级体验；
22. 将直连工作归档或显式迁移到完整 CognitiveOS；
23. 无障碍用户完成创建、响应、暂停、验收和账号切换。

每个旅程记录：入口、前置条件、产品部署模式、主机、Agent/Adapter 版本、账号、用户步骤、事实来源、控制路径、失败/取消/重复/恢复、敏感数据、可执行 oracle、当前 evidence。

## 页面与状态设计

必须给出 PC 和手机独立 IA、页面/组件/状态矩阵；可以使用文字线框或 ASCII flow，但禁止生成 UI 代码。

所有主要页面覆盖：

- initial loading；
- no host / no agent / no account；
- agent detected but unsupported；
- adapter incompatible；
- connecting / authenticating / reauth required；
- ready；
- running；
- waiting user；
- partial / stale / offline；
- permission denied；
- rate limited / quota exhausted；
- submitting / cancelling / stopping；
- result unknown；
- conflict / superseded；
- agent reported done；
- checks observed；
- user accepted；
- host locked / privacy locked；
- relay degraded；
- credential revoked；
- update required。

状态不能只靠颜色、动画或 Agent 文本表达。所有 destructive/permission/account 动作必须有清晰目标、后果、撤销范围和焦点管理。

## 威胁模型最低覆盖

- 恶意或被攻陷的 Agent、Adapter、plugin、MCP server；
- prompt injection 诱导读取 secret、扩大权限或控制其他 Agent；
- PTY 输出伪造系统状态、permission prompt、完成或错误；
- Adapter 解析器崩溃、协议漂移、恶意超长/ANSI/Bidi 内容；
- CLI/app-server 非 loopback 暴露、未认证 WebSocket/API；
- 读取或复制 Codex/Claude/WorkBuddy/browser 登录 secret；
- API key 出现在环境、进程列表、日志、shell history、dump 或 child process；
- 账号/profile/config home/cache/session 跨用户或跨任务混淆；
- Relay 读取正文、重放命令、错误路由到其他 Host/账号；
- 手机丢失、越狱/root、恶意 accessibility/overlay、通知和截图泄露；
- 远程终端/桌面控制绕过本机权限或用户可见提示；
- GUI 自动化点击错误窗口、密码管理器或提升提示；
- 多 Agent 共享 worktree、文件锁、Git index、端口、dev server 和 credential 冲突；
- Agent 互相 prompt injection、伪造 handoff、循环委派和预算爆炸；
- cancel/kill 后外部 Effect 仍发生；
- timeout、断网、Host 崩溃和 duplicate 导致未知结果或重复动作；
- Adapter/Agent 自动更新造成兼容性或供应链回归；
- 开源许可证、商标、商店分发和供应商条款不兼容；
- “直连模式”视觉上被误认成完整 CognitiveOS 保证。

每个威胁至少记录：资产、攻击者、入口、信任边界、预防控制、检测、失败语义、恢复、owner、oracle、evidence。

## Open PoC 与可执行 oracle

本任务不执行 PoC，但必须列出未来真实 PoC，至少覆盖：

- 每个 Tier 1 Agent 的版本握手、session、stream、permission、cancel、resume；
- 升级一个主版本后 Adapter 的兼容与安全降级；
- PTY/ANSI/Bidi/超长输出负例；
- CLI/app-server/Gateway 仅 loopback 暴露和本地 peer authentication；
- API key 不进入日志、命令行、dump、Relay、手机或子进程；
- 多账号 config/cache/history 隔离与 logout/revoke；
- Codex/Claude/WorkBuddy 官方登录和条款允许的路径；
- Windows/macOS/Linux 锁屏、用户切换、权限提升和进程树；
- iOS/Android 配对、E2EE、push、revoke、进程死亡和离线；
- command replay、乱序、重复、过期和错误 Host routing；
- worktree/branch/port/resource isolation；
- 群组 DAG、循环委派、预算上限、取消和 conflict；
- Agent reported done 不能自动成为 verified/accepted；
- Host crash、Relay outage、Agent crash 与 unknown outcome；
- 键盘、Narrator/VoiceOver/TalkBack、高对比、字体缩放和 reduced motion。

每项 PoC 记录目标版本、设备/OS、前置条件、步骤、expected oracle、失败判据、证据路径和不可外推范围。模拟器、单一 OS、单一 Agent 版本或单一账号结果不得外推到整个支持矩阵。

## 需要交付

关键决策确认、文件级计划获用户批准后，只生成 informative 文档。建议至少产出：

1. `apps/cognitiveos-console/docs/agent-hub-product-design.md`
   - 产品问题、persona/JTBD、部署模式、范围、IA、成功指标和非目标。
2. `apps/cognitiveos-console/docs/agent-hub-adapter-matrix.md`
   - Agent dossier、逐能力支持等级、版本策略、来源与许可证 ledger。
3. `apps/cognitiveos-console/docs/agent-hub-security-and-credentials.md`
   - Host/Adapter/Vault/Relay/Mobile 边界、账号、key、远控和威胁模型。
4. `apps/cognitiveos-console/docs/agent-hub-journeys-and-screens.md`
   - PC/手机旅程、页面、组件、状态和文字线框。
5. `apps/cognitiveos-console/docs/agent-hub-collaboration.md`
   - 群组模型、DAG、worktree/artifact 隔离、handoff、评审和冲突。
6. `apps/cognitiveos-console/docs/agent-hub-decision-log.md`
   - 产品决策、替代方案、依据、状态和 blocked_by。
7. `docs/platforms/agent-hub-platform-parity.md`
   - Windows/macOS/Linux/iOS/Android 与三种产品部署模式的能力、限制和证据边界。
8. `docs/plan/agent-hub-development-plan.md`
   - 只编排、不执行的开发任务包：阶段、依赖 DAG、车道/所有权、里程碑 gate、任务卡、验收 oracle、风险、估算假设、并行/串行关系和回滚点。
9. `docs/prompts/agent-hub/` 下的自包含接续提示词
   - 按未来批准的车道拆分产品、合同、Host/Adapter、Vault/Relay、桌面、iOS、Android、测试/安全与发布任务；每份提示词写清输入、前置 gate、允许路径、禁止路径、交付物、测试和 handoff。
   - 这些提示词只是未来开发入口；当前 Console implementation gate 未通过时必须标 `blocked`，不得调用它们开始写代码。
10. 必要的索引、roadmap、requirements trace、PROGRESS 和 handoff 更新。

如果研究表明更少的文件能降低漂移，可以合并，但必须保留单一事实来源和稳定 anchor。不得破坏现有 Windows/macOS/Linux/iOS/Android ID、anchor、parity 或 gate。

## 产品 ID 与状态纪律

建议使用独立的 informative namespace，最终名称先经用户确认，例如：

- `CONSOLE-AGENTHUB-V1-PRD-*`
- `CONSOLE-AGENTHUB-V1-DEC-*`
- `CONSOLE-AGENTHUB-V1-JRN-*`
- `CONSOLE-AGENTHUB-V1-PAGE-*`
- `CONSOLE-AGENTHUB-V1-POC-*`

这些 ID 不进入 `specs/registry/requirements.yaml`，也不是 CognitiveOS machine contract。

每项产品要求至少记录：

- deployment profile；
- agent/adapter/platform；
- contract；
- implementation；
- evidence；
- owner；
- oracle；
- blocked_by；
- source/version；
- user-visible guarantee。

状态严格区分：

- `specified`：真实 REQ/schema/transition/vector 已登记；
- `implementation available`：代码存在且可构建；
- `test executed`：真实测试已执行并留证；
- `Profile implemented`：全部适用 MUST 有证据；
- 产品层另可使用 `product-only / unregistered / partial / planned / blocked / not-implemented / none / not-run`，但不得混写为上述四类事实。

只有 registry 中真实存在的 `REQ-*` 才能引用。发现新产品依赖不等于发现规范漂移；不得为本模式直接新增对象族、Profile 或 REQ 域。

## 工作方式

1. 完成并行只读研究。
2. 主代理输出一份“事实 / 推论 / 风险 / 待决策”综合摘要。
3. 按依赖顺序每轮只问用户 1–2 个关键问题；优先使用 Cursor 的交互式多选/单选提问工具，给出推荐项、互斥选项和影响，不要一次倾倒整份问题清单。
4. 全部关键决策确认后，提出文件级计划、ID 方案、影响面和不修改项。
5. 只有用户批准计划后才编辑 informative 文档。
6. 编辑后同时启动至少 4 个只读审查代理：
   - 外部事实、许可证与维护状态；
   - 凭据、远控、Relay 与多账号安全；
   - PC/手机 UX、无障碍与恢复；
   - 仓库边界、状态用语、链接与追溯。
7. 主代理逐项验证高置信问题并最小修正，不机械接受相互冲突的建议。
8. 产品与架构设计稳定后，再生成开发任务 DAG 与各车道自包含提示词；只做编排，不实现任何任务。

## 完成前检查

1. 所有外部事实有一手来源、查询日期和适用版本。
2. OpenClaw、Hermes、Codex、Claude Code、WorkBuddy 的项目身份和接口未混淆。
3. ACP、MCP、A2A 与供应商专属协议职责未混淆。
4. “账号导入”没有变成 token/cookie/keychain secret 抽取或订阅凭据代理。
5. 三种产品部署模式的能力、保证、事实来源和完成语义清楚分离。
6. 每个 Adapter 逐能力标记，没有笼统“全支持”。
7. PC 与手机、桌面 OS、Agent 版本和账号证据均未互相外推。
8. 群聊消息、Agent reported done、退出码、Relay receipt、push 或通知未被写成 authority/Verification/acceptance。
9. cancel、interrupt、process kill、Effect closure 和 compensation 未混同。
10. 手机不承载 Agent runtime、完整 secret vault 或无限后台监督。
11. 电脑控制按能力与 OS 权限拆分，并有明确 blocked 项。
12. 现有 Console gate、M9 路线图、平台 ID、anchor 和 parity 未被静默改写。
13. 未修改 registry、error registry、schema、transition、vector 或实现代码。
14. 运行 Markdown link/anchor 检查、`git diff --check` 和 `pnpm run check:consistency`。
15. 使用 `code-review` skill 和并行审查代理完成 docs-only 终审。
16. 未执行的实现、PoC、真机、Agent 集成和安全测试保持 `not-implemented / none / not-run`。
17. 更新 `docs/plan/PROGRESS.md` 和新的 handoff。
18. 只暂存本任务文件并提交，禁止混入其他工作区改动。

## 最终输出要求

最终回复简明列出：

- 新增/修改的文档；
- 已确认的产品名称、部署模式、Tier 1 Agent 和跨端范围；
- 账号连接、API key、手机 Relay、电脑控制和群组协作的关键决策；
- 参考的官方与开源项目及重要反例；
- 真实执行的检查及结果；
- contract、implementation、test、Profile 的准确状态；
- 未登记的 machine contracts、blocked Adapter、Open PoC、条款与平台风险；
- 提交哈希。

不得把产品设计完成描述为 Agent Hub、PC/手机客户端、Adapter、Relay、Vault、多 Agent、账号切换或电脑控制已经实现、测试或符合 Profile。
