# CognitiveOS Console v2 产品决策记录

> 状态：Accepted product decisions
>
> 日期：2026-07-20
>
> 地位：Informative。本文记录 Console 产品层决策，不新增或修改 CognitiveOS 的 REQ、schema、transition table 或 conformance vector。
>
> Agent Hub 关系：Agent Hub 决策用独立 `CONSOLE-AGENTHUB-V1-DEC-*` 命名空间，记录于 [agent-hub/decisions/decision-log.md](../../../agent-hub/docs/decisions/decision-log.md)，沿用本文 `CONSOLE-V2-DEC-*` 基线（如 persona、两模式区分），不改写或重编号本文决策。

## 1. 使用方式

- 本文是 v2 产品文档的决策基线。其他文档不得以未注明方式覆盖这些决定。
- 决策只约束 Console 产品设计；凡涉及节点、认证、监督 lease、Windows Service、包验证或风险准入的机器行为，均须在相应 normative 资产中登记后才可称为“规范已登记”。
- 决策状态：
  - `accepted`：产品方向已确认；
  - `superseded`：已被后续决策取代；
  - `blocked`：方向成立，但缺少机器契约、实现或证据，不能作为已提供能力；
  - `future`：不属于 Windows v1。

## 2. Windows v1 决策

### `CONSOLE-V2-DEC-001` 首要用户与核心任务

- 状态：`accepted`
- 首要 persona：Agent 操作者 / 高级终端用户。
- 首要任务：
  1. 继续对话、创建受治理任务；
  2. 监督、纠偏、暂停或安全停止任务；
  3. 查看并管理 Agent 的安装、升级、回滚和卸载。
- 管理员、运维、审批人、审计员和发布者保留为建议 persona 模板，不是跨部署固定 RBAC 枚举。真实入口和动作始终由 authority projection 决定。

### `CONSOLE-V2-DEC-002` 发布平台与范围

- 状态：`accepted`
- 第一可交付版本仅面向 Windows 桌面。
- macOS、Linux、iOS、Android、远程节点和多工作区移入 roadmap，不在 Windows v1 主页面规格中表现为已冻结能力。
- 首版支持一个本机共享节点；远程与多节点连接后续提供。

### `CONSOLE-V2-DEC-003` 本地节点承载模型

- 状态：`accepted`
- 本地节点由独立签名的 Windows Service 承载，生命周期不依赖任一 Console 窗口或 Windows 登录会话。
- Console 是该 Service 的客户端，只能通过确定性 allowlist IPC 请求发现、启动状态、停止、升级和恢复等受控操作。
- Renderer 不取得 Service Manager、进程创建、任意文件写入或提权权限。
- Service 查询和用户级任务操作可使用节点授权；Service 安装/卸载/stop/restart、机器级更新、bootstrap bundle mint、信任重置和恢复必须同时通过 Windows 管理员/UAC、签名来源与 anti-rollback 校验。CognitiveOS Owner 不自动等于 Windows 管理员，反之亦然。
- 早期“Console 主进程直接启动共享子进程”的选择已被本决策取代；用户级子进程无法安全承载多 Windows 用户共享节点。
- 机器合同状态：`blocked`。Windows Service、IPC、升级与恢复合同尚未登记。

### `CONSOLE-V2-DEC-004` 本地身份与账号所有权

- 状态：`accepted`
- 共享本地节点是账号、密码验证和 AuthenticationSession 的 authority；Console 不保存密码哈希，不成为 IdP。
- 每个 Windows 用户映射到独立 CognitiveOS 本地账号；节点负责账号隔离、授权和会话撤销。
- 第一个 Owner 通过安装器或 Admin CLI 生成的一次性本机 bootstrap bundle 原子领取。Bundle 固定目标 Service endpoint key，并把 secret 绑定目标 Windows SID；Service 从受认证 IPC peer token 取得 SID，禁止信任请求正文自报 SID。
- 禁止“任意首个连接者自动成为 Owner”，也禁止向未匹配 bundle endpoint key 的端点发送 secret。
- 账号恢复只通过独立 Admin CLI；恢复动作必须轮换相关会话/密钥并写入 authority 审计。
- 机器合同状态：`blocked`。本地账号、Windows SID 绑定、bootstrap secret、恢复和 AuthenticationSession wire contract 尚未登记。

### `CONSOLE-V2-DEC-005` 首次信任

- 状态：`accepted`
- loopback / 个人本地节点可使用 TOFU 固定后续身份连续性；Owner bootstrap 优先使用安装器 bundle 中的 endpoint key，不能先向未认证 TOFU 端点发送 bearer secret。身份变化时必须阻断并显示旧/新身份和恢复路径。
- TOFU 不扩展到未来远程或企业节点。远程/企业连接必须使用组织预置信任、受管发现或带外指纹验证。
- TOFU 只建立节点身份连续性，不授予用户权限，也不替代 Owner bootstrap。

### `CONSOLE-V2-DEC-006` 风险范围与 R1 确认

- 状态：`accepted`
- Windows v1 只支持 R0/R1。
- R2/R3 proposal 可以被识别并解释，但所有执行入口必须禁用；不得降级为 R1，不提供聊天批准或临时旁路。
- R1 默认使用结构化摘要和明确动作按钮。只有 authority/policy 明确要求时才增加一次性、digest-bound number matching。
- 风险下界由节点 authority 计算或验证；客户端、Agent、包来源和用户警告都不能降低风险。

### `CONSOLE-V2-DEC-007` Agent 来源与全生命周期

- 状态：`accepted`
- Windows v1 提供 Agent 查看、安装、升级、回滚、卸载的完整用户旅程。
- 用户可输入任意来源（Catalog、URL、Git、本地文件）用于获取和检查包。
- 获取本身是受治理 acquisition proposal/Effect，由无 ambient credential 的低权限 broker 执行，并受本地路径、SSRF/UNC、重定向和资源预算策略约束。
- “允许获取”不等于“允许运行”：签名、provenance、静态检查、compatibility、sandbox 和 authority risk admission 必须分别展示。
- 只有 authority 判定风险不高于 R1 且所有适用 gate 通过的包才能进入执行路径；R2/R3、缺少必要证据或 gate 失败时明确阻断。
- 旧 installation 和 rollback point 在新版本提交前保持可用；卸载不得删除未决 Effect 或审计证据。

### `CONSOLE-V2-DEC-008` 失联暂停与监督 lease

- 状态：`accepted`
- 用户期望是：失去 Console 监督后，任务默认在安全检查点暂停。
- 该语义不能由断线后的客户端命令实现，必须依赖节点侧 supervision lease/heartbeat：
  - Console 只有在 task/principal/SID/logon-session/channel/client-epoch 绑定仍有效、AuthenticationSession 当前、watch freshness 满足门禁且 UI 可响应时续租；
  - lease 到期后节点进入 `pause_pending`，在安全检查点确认暂停；
  - UI 在收到 authority 证据前不得显示“已暂停”；
  - 无法安全暂停时必须显示原因、仍可能发生的动作和建议遏制路径。
- 机器合同状态：`blocked`。监督 lease、heartbeat、safe-checkpoint pause 和恢复合同尚未登记。

### `CONSOLE-V2-DEC-009` 窗口关闭与通知

- 状态：`accepted`
- 关闭主窗口默认最小化到系统托盘并继续监督 lease。
- 显式动作命名为“退出并请求暂停”。退出流程显示受影响任务，并有界等待 pause request 被 authority 接收；超时默认留在托盘，用户若仍强制退出则明确依赖 lease 到期、保留稳定请求引用且不得宣称已暂停。
- Windows 系统通知由每用户 notification broker 投递，只包含脱敏提示和一次性 opaque handle。打开后必须重新认证/授权、消费 handle 并从 authority 重取状态。
- Windows 通知不得直接执行 R1 动作。

### `CONSOLE-V2-DEC-010` 离线与敏感快照

- 状态：`accepted`
- 应用不主动把敏感正文和对象快照持久化到磁盘；敏感 projection 只在当前已解锁会话的受控 renderer/内存中使用。
- 进程退出、登出、锁屏或账号切换会 teardown 敏感 renderer、零化应用管理的 buffer 并清理临时 profile；冷启动只显示非敏感连接元数据。Windows pagefile/hibernation/crash dump 与 WebView2 UDF/cache 风险必须通过平台配置和 PoC 收窄，不能宣称绝对“只存在于物理内存”。
- 断线后的内存快照必须显示 authority、版本和 `as_of`，所有写入口禁用；不可达不等于任务已暂停或停止。

### `CONSOLE-V2-DEC-011` 持久化/审计故障

- 状态：`accepted` direction / `blocked` contract
- authority state store 或审计持久化不可用时，普通任务写、Agent 安装/升级/卸载、配置、账号/bootstrap/recovery 和 Service 变更全部 fail closed。
- pause/stop 只有在 store 健康时预铸的限时遏制 capability 仍可验证、绑定 target/version/fencing、policy 明确该动作降低风险且独立应急日志可写时继续；allowlist 和日志本身不构成授权。
- 无法验证授权/撤销/fencing 时，仅允许 Service 自身已登记的 supervision lease-expiry safe pause；所有应急结果返回稳定引用和结果未知语义，恢复后对账而不补造历史。
- 当前 `state-store-degradation.json` 无条件期望 deterministic stop/revoke 可用，与上述授权收口存在合同漂移；在规范澄清前本决策保持 `blocked`。

## 3. 体验与品牌决策

### `CONSOLE-V2-DEC-012` 首页与导航

- 状态：`accepted`
- Agent 操作者在节点 `USER_READY`/`OPERATIONAL` 且授权允许时进入 Shell。
- 其他 persona 和异常 readiness 使用角色/readiness 感知落点；不再把 Shell 写成所有用户、所有状态的统一首页。
- 一级导航采用任务导向的稳定分组：工作、任务、Agent、收件箱、记录、系统。“记录”是“治理”的用户语言标签；Windows v1 只显示与当前授权和范围相关的入口。
- 产品用语以用户任务为主；`AgentExecution`、`Effect`、`Verification` 等机器术语在详情、来源说明和审计中保留。

### `CONSOLE-V2-DEC-013` Shell 工作区

- 状态：`accepted`
- Shell 采用对话主画布 + 可折叠任务/上下文侧栏，不固定三栏宽度。
- 安全关键 proposal、结果未知、暂停待确认和 freshness 信息可以临时固定展开；普通上下文按需披露。
- 默认舒适密度；任务和 Agent 列表可切换紧凑密度，语义与可操作面积不因密度变化而降低。

### `CONSOLE-V2-DEC-014` 品牌与动效

- 状态：`accepted`
- “CognitiveOS Console”仍是工作名，Public Beta 前需独立品牌决策。
- 品牌性格：亲和易懂、未来感、强大可控。
- 采用 CognitiveOS 自有跨平台视觉语言；Windows v1 遵循平台行为、输入和无障碍惯例，但不复制 Fluent 外观。
- 动效以功能性反馈为主，只保留一个可识别的“受治理任务流转”签名动效；必须提供完整 reduced-motion 等价路径。
- 不使用持续环境动画暗示 authority 正在推进，不用动画、颜色或位置作为唯一状态信号。

### `CONSOLE-V2-DEC-015` 语言与技术候选

- 状态：`accepted`
- Windows v1 同期支持简体中文与英文。
- Tauri 2 + React/TypeScript 是首选候选，不是已批准技术决策；须经 Windows Service/IPC、WebView 隔离、可访问性、升级和安全 PoC 后由 ADR 冻结。

## 4. 文档与追踪决策

### `CONSOLE-V2-DEC-016` 文档结构

- 状态：`accepted`
- v2 使用模块化文档；`PRODUCT-DESIGN.md` 保留为精简入口和兼容层。
- 非 Windows v1 能力移入 roadmap/feature brief，保留安全约束和依赖，不保留未经验证的完整 UI 规格。
- 现有 `§17`、`§20.3` anchor 必须继续可达，以兼容仓库现有引用。

### `CONSOLE-V2-DEC-017` v2 ID

- 状态：`accepted`
- v2 重新编号，使用不同 namespace：
  - 产品要求：`CONSOLE-V2-PRD-xxx`
  - 决策：`CONSOLE-V2-DEC-xxx`
  - 旅程：`CONSOLE-V2-JRN-xxx`
  - 页面：`CONSOLE-V2-PAGE-xxx`
  - 组件：`CONSOLE-V2-CMP-xxx`
- 旧 `CONSOLE-PRD-*` 与 `A-*` 不再新增或复用；每个旧 ID 必须映射到 v2、future 或 deprecated。

## 5. 已替代假设摘要

- “五端同期 MVP” → Windows v1，其他平台进入 roadmap。
- “所有角色默认进入 Shell” → 角色/readiness 感知落点。
- “12 个稳定一级对象入口” → 任务导向 5–7 个分组。
- “固定桌面三栏” → 对话主画布 + 可折叠侧栏。
- “所有 R1 固定 number matching” → policy 驱动的自适应确认。
- “detach 后任务总是继续” → supervision lease 到期后安全暂停。
- “Console 直接承载共享节点子进程” → 独立 Windows Service。
- “任意首个连接者成为 Owner” → 一次性本机 secret 领取。
- “产品依赖”单一标签 → contract / implementation / evidence 三维状态。

## 6. 独立桌面平台决策

2026-07-20 在 Lane-CON 激活前 informative 文档例外内确认 macOS v1 与 Linux v1 产品方向。平台决策使用独立 namespace，不改写本文件既有 `CONSOLE-V2-DEC-*`：

- `CONSOLE-MAC-V1-DEC-*` 与完整范围见 [macOS v1 产品设计](../platforms/macos/macos-product-design.md)；
- `CONSOLE-LNX-V1-DEC-*` 与完整范围见 [Linux v1 产品设计](../platforms/linux/linux-product-design.md)；
- 两平台统一决策索引见 [平台产品决策记录](../platforms/platform-decision-log.md)；
- Windows 可复用/适配/替换/阻断关系见 [桌面 parity matrix](../platforms/desktop-parity-matrix.md)。

这些决策是 accepted product direction，但 machine contract、implementation、executed evidence 和 Profile conformance 均未闭合。它们不改变 `CONSOLE-V2-DEC-002` 的事实：Windows v1 仍是 Windows 产品切片；macOS/Linux 是独立、后置且当前 blocked 的平台切片。

## 7. 独立移动平台决策

2026-07-20 在同一 Lane-CON informative 文档例外内，确认 iPhone v1 与 Android phone v1 的独立产品方向。移动决策使用独立 namespace，不修改或复用本文件既有 `CONSOLE-V2-DEC-*`：

- `CONSOLE-IOS-V1-DEC-001..016` 与 canonical 记录见 [移动平台产品决策索引](../../../mobile/shared/docs/mobile-platform-decision-log.md#3-iphone-v1-决策)；
- `CONSOLE-AND-V1-DEC-001..016` 与 canonical 记录见 [移动平台产品决策索引](../../../mobile/shared/docs/mobile-platform-decision-log.md#4-android-phone-v1-决策)；
- iPhone 范围、要求、旅程、威胁、PoC 和 Apple 来源见 [iPhone-only v1 产品设计](../../../mobile/ios/docs/ios-product-design.md)；
- Android 范围、列名设备、要求、旅程、威胁、PoC 和 Google 来源见 [Android phone v1 产品设计](../../../mobile/android/docs/android-product-design.md)；
- Windows/macOS/Linux 的复用、移动适配、替换、不提供和阻断关系见 [移动 parity matrix](../../../mobile/shared/docs/mobile-parity-matrix.md)。

移动 v1 是 phone-only 的受限远程 Console，不是本地 node/daemon/authority。它提供 Conversation/Task、监督纠偏、tenant/node 选择和基于 authority catalog ref 的远端 Agent 生命周期；只执行 authority 判定的 R0/R1，R2/R3 一律阻断。所有移动 machine carrier、implementation、platform evidence 和 Profile conformance 当前仍未闭合；全局 conformance 分布不得外推为移动平台证据。
