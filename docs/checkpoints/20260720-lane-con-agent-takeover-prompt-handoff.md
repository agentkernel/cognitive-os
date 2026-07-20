# 20260720 Lane-CON Agent Takeover Prompt Handoff

## 1. 本次会话完成

- 按用户决策取消 `cognitive-kernel` 中间模式；Agent Hub 只保留“无 CognitiveOS 安全直连接管”与“完整 CognitiveOS 治理”两种产品部署模式。
- 重构 `docs/prompts/console-agent-hub-direct-mode-product-design.md`，将“直接介入进程/文件”收口为分级、可审计的接管模型：官方控制面 → Host 启动监管 → 官方 session adopt → 受管终端 attach → 只读文件观察 → 有文档约束的文件写入 → observe-only；进程注入、二进制篡改、token 抽取和未记录内部文件写入明确禁止。
- 以 `getpaseo/paseo` 官方仓库、Providers/Custom Providers、CLI 与 SECURITY 文档为主要参考：确认 Paseo 以 daemon 启动/监管已安装已认证的 CLI，通过 native Adapter 或 ACP/stdio 控制，并支持 provider session import、worktree、移动端和 E2EE Relay；它不是任意 PID 注入工具。
- 将 Paseo 的已知边界写入提示词：默认 loopback 依赖可达性、客户端被视为 daemon 用户的 trusted operator、文件预览可触达 daemon 用户可读文件、Relay 暴露元数据且 live session replay tracking 尚未完整，以及 AGPL-3.0 与单维护者风险。
- Paseo 源码深挖进一步修正：`attach` 是 timeline 订阅而非进程 stdin；import 会新建 Paseo Agent 并调用 provider resume/load，旧 runtime/双 writer 互斥未被证明；quota fetcher 会读取、刷新或重写部分 provider 凭据；配对链接、持久 daemon key、逐设备撤销和文档/源码表述存在额外边界。提示词已要求 inactive/exclusive lease gate、禁止 quota token 抓取并补 per-device identity/anti-replay/rotation。
- 增补 Agent of Empires、Claude Squad、amux、tmux-agent-tools、Happy、Vibe Kanban、Agent Deck 等 daemon/tmux/worktree/remote-control 项目作为接管模式对照，要求执行提示词时逐项目重新核实官方来源、许可和维护状态。
- 新增 Takeover Host 最小拓扑与职责分离：Control API、Process Supervisor、Terminal Broker、Session Adopter、File Observer、Workspace Manager、Credential Broker、Local Event Ledger、Verifier。
- 新增进程 identity/ownership generation、single-controller lease、文件 snapshot/lock/digest/parser、symlink/TOCTOU、session import、移动本机确认、接管状态、威胁、PoC、可执行 oracle 和开发车道要求。
- 在提示词末尾追加“文档先行、计划治理与多代理开发”阶段协议：先独立研究并完善/优化/局部或整体重构现有产品文档，再建立 Agent Hub 独立文档目录、canonical/derived/owner/traceability/source/template 治理体系，生成 Master 与 11 类分开发计划、统一进度表和分车道自包含提示词；只有产品/计划批准且仓库 gate 满足后才能启动多代理实现。
- 末尾另加单段“最终强制执行提示”，把 `@docs` 产品设计审查/重构、独立文档治理、具体开发计划/进度表和 gate 后按计划调用多代理开发收敛为不可跳过的执行顺序，避免长提示词中关键阶段被忽略。
- 更新 `docs/README.md` 与 `docs/plan/PROGRESS.md`。核心提交：`7bd6903`。
- 本批为 prompt/informative 文档，无关联 REQ/F/IMP；未修改 normative 机器资产或实现代码。

## 2. 未完成 / 进行中

- Agent Hub 产品正文、Paseo/同类项目完整源码尽调、独立文档治理目录、Adapter dossier、两模式 parity、接管 UX、分开发计划、进度表、开发任务 DAG 和分车道提示词尚未执行。
- 首发接管层级、Tier 1 Agent、Takeover Host OS 身份、session 文件允许范围、Relay 形态、手机控制上限和供应商条款仍须在执行提示词时逐轮确认。
- Console implementation gate 未满足；不得据此启动 Takeover Host、Adapter、Relay、Vault、桌面或移动实现。

## 3. 测试与证据状态

- `pnpm run check:consistency`：通过；273 requirements、55 error codes、56 schemas、76 vectors，Markdown links 与 traceability 已验证。
- `git diff --cached --check`：核心提交前通过。
- ReadLints：提示词、索引和 PROGRESS 无诊断。
- 外部研究使用 Paseo 官方仓库/文档以及相关项目官方 GitHub；研究事实只用于提示词边界，不是产品实现或安全测试证据。
- 未执行 Agent 进程接管、session import、文件观察、PTY/tmux、Relay、真机、凭据或安全 PoC。
- 76 个既有 conformance vectors 仍全部为 `not-run`；Console/Takeover implementation 未提供，Profile 未符合。

## 4. 未决风险与漂移

- 未发现需要修改 normative 资产的新漂移；接管 Host、进程/文件 carrier、Relay、Vault 与 Adapter 仍为 `unregistered / planned / blocked`。
- “用户拥有本机文件/进程”不自动等于供应商允许第三方自动化、再分发、订阅凭据复用或内部格式写入；每个 Agent/版本/账号类型必须单独做条款与许可证复核。
- Paseo 的安全模型不能原样外推：本产品应优先 OS-authenticated IPC、最小客户端能力、文件范围授权和 live-session anti-replay，而不是仅依赖 loopback 或把所有客户端视为等权 operator。
- session import 若不能证明旧 runtime inactive 或取得 provider-supported exclusive lease，必须拒绝写接管并保持 observe-only，避免两个进程同时写同一 native session。
- 对无受支持 IPC/PTY/session handle 的既有 PID，只能 observe-only；通过调试器、内存、任意 stdin 或内部数据库强行接管不进入产品方案。
- session 文件只读解析仍面临半写、锁、损坏、schema drift、敏感字段、symlink/TOCTOU 和跨用户泄露风险。
- Agent reported done、进程退出、terminal capture、文件变化、通知或 Relay receipt 均不是 CognitiveOS Verification/acceptance。

## 5. 下一步入口

- 建议提示词：`docs/prompts/console-agent-hub-direct-mode-product-design.md`
- 工作范围：先执行 Lane-CON informative 产品/文档/计划阶段；实现仍无可用车道，只有后续全部 gate 满足后才按批准计划启动多代理开发。
- 第一个动作：在新 Cursor 会话粘贴完整提示词，并发完成 Paseo/同类源码、Agent 接口、供应商条款、Host 安全与跨端 UX 只读研究；随后通过交互式问题确认首发 takeover 层级。

## 6. 快照

- PROGRESS 已更新：是。
- Agent Hub 状态：两模式及阶段化多代理开发提示词已提供；产品设计/文档治理/开发编排未执行；implementation `not-implemented`；product evidence `none`；Profile 未符合。
- 本次核心提交：`7bd6903`。
- 本 handoff 与 PROGRESS 最新入口提交哈希见会话最终报告。
