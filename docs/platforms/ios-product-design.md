# CognitiveOS Console iPhone-only v1 产品设计

> 类别：informative product design
>
> 决策状态：accepted product direction
>
> 交付状态：`planned / implementation not-implemented / platform tests none / Profile not implemented`
>
> 官方资料查询日期：2026-07-20
>
> GA 形态：iPhone only；iOS 18+；arm64

本文定义 CognitiveOS Console 的 iPhone-only v1 产品切片。它不新增或修改任何 CognitiveOS `REQ-*`、错误码、schema、transition table、conformance vector 或 Profile，也不表示客户端、移动 carrier、后台 lease、APNs broker、R1 device signature、分发或恢复能力已经实现。

本文中的 `CONSOLE-IOS-V1-*` 仅是 informative 产品 ID。凡移动专属行为没有已登记机器合同时，均明确写为 `unregistered / planned / blocked`，不得从产品设计反推规范已存在。

## 0. 文档状态、查询日与非规范声明

### 0.1 当前工程事实

- 规范已登记：273 个 requirements、55 个 error codes、56 个 schemas、5 份 transition tables。
- conformance vectors：76 份；当前全部 `not-run`。
- Console iOS implementation：`not-implemented`。
- iOS simulator、真机、APNs、TestFlight、App Store、Custom App、MDM、VoiceOver、安全与恢复测试：`none`。
- CognitiveOS Console iOS Profile：`planned`，未实现、未测试、未符合。
- 本文只允许表达平台事实、产品决策、已登记合同引用、实现状态和证据状态；这五类信息不能互相替代。

### 0.2 证据口径

- Apple 平台事实只来自 §20 中于 2026-07-20 实际打开的一手 Apple 页面。
- 搜索摘要不作为证据；抓取失败的 HIG 页面只登记“需人工复核”，不据此编造规则。
- iOS 证据不能外推至 iPadOS、watchOS、Android、macOS，也不能由 simulator 外推到真机 Secure Enclave、APNs、锁屏、备份或生物识别行为。
- App Review、App Store Connect、Apple Platform Security 和动态 Developer Documentation 会变化；进入 TestFlight、审核和 GA 前必须按当日页面复核。

## 1. 产品角色、用户、任务与非目标

### 1.1 一句话定义

CognitiveOS Console iPhone v1 是面向 Agent 操作者的、更广但受限的远程 Console：用户可在手机上继续 Conversation、创建和监督 Task、纠偏、选择 tenant/node，并管理远端 Agent 的安装、升级、回滚和卸载；所有事实、风险、授权和提交仍由远端 authority 决定。

### 1.2 首要用户与核心任务

首要用户是 Agent operator。v1 提供：

1. 在 `zh-CN` 或 `en` 中继续 Conversation、形成受治理目标并创建 Task；
2. 选择当前账号下 authority 返回的 tenant 和 node；
3. 分离查看 Task、Loop、AgentExecution、Effect、Verification 五个独立 authority lifecycle 域和独立远端 Runtime projection，监督、纠偏、请求暂停并对账；
4. 查看 `CANDIDATE_COMPLETE`、Verification、Acceptance 与真正 `COMPLETED` 的差异；
5. 对 authority 判定为 R0/R1 的操作执行适用确认；
6. 查看并管理远端 Agent 的 install、upgrade、rollback、uninstall；
7. 从 APNs 通知安全打开待办，重新认证并从 authority 取当前状态；
8. 在离线、后台、锁屏、进程死亡、版本低于 floor 或设备异常时进入可解释的安全恢复。

### 1.3 角色边界

| 主体 | 可以做什么 | 不能因此获得什么 |
|---|---|---|
| iPhone App | 展示 projection、收集 proposal、签署 device-bound R1 challenge、提交请求 | authority、IdP、node、daemon、最终仲裁、完成事实 |
| Agent operator | 创建、监督、纠偏、管理远端 Agent 生命周期 | risk 降级、R2/R3、authority commit |
| upstream IdP | 认证账号，按策略支持 passkey | CognitiveOS operation authorization、设备绑定决定 |
| CognitiveOS authority | 返回 tenant/node、current state、risk floor、challenge、授权与最终结果 | iOS UI 或本机设备控制权 |
| APNs / notification surface | 提示有事项可查看 | delivery truth、session、authorization、acceptance evidence |
| Secure Enclave device key | 对固定 canonical bytes 产生不可导出私钥签名 | 用户身份全貌、risk 判定、authority decision |
| managed device / MDM | 安装、配置和收窄 managed App | 扩大能力上限、成为 CognitiveOS Owner |

### 1.4 明确非目标

- iPhone 承载 CognitiveOS node、Agent runtime、daemon、broker、authority、IdP 或 final security arbiter；
- R2/R3 执行、通知批准、聊天批准、设备密码降级批准；
- iPad、Apple Watch、widget、Live Activity、App Intents、Siri、Shortcuts 的 GA；
- 无限后台 watch、可靠后台 heartbeat 或永久在线 supervision；
- iPhone 下载、解释、执行、解压、扫描或转发 Agent executable bundle；
- raw HTML、JavaScript、iframe、remote image auto-load、native bridge 或内嵌任意网页；
- 离线控制动作队列、离线完成声明或敏感 authority snapshot 持久化；
- Enterprise Program 分发、应用内 executable 自更新或绕过 App Review；
- 把 jailbreak、App Attest、系统状态、生物识别或 APNs receipt 写成确定可信证明。

## 2. 支持矩阵与支持期限

| 维度 | iPhone v1 决策 |
|---|---|
| GA 设备 | 仅 iPhone |
| OS / CPU | iOS 18+；arm64 |
| 非 GA | iPad、Watch、widget、Live Activity、App Intents、Siri、Shortcuts：`planned/blocked` |
| 市场 | 美国、新加坡 |
| 语言 | `en`、`zh-CN`；machine enum/ref/digest 不翻译 |
| 设备所有权 | BYOD 与 managed device |
| 账号 | 单一活动账号；一个账号可访问多个 tenant/node |
| 风险 | authority 判定 R0/R1 可进入执行；R2/R3 显示并阻断 |
| 分发 | Public App Store、TestFlight、Apple Business Manager Custom App |
| 不提供 | Apple Developer Enterprise Program |
| 方向 | 竖屏为主；横屏必须完整可用 |
| 输入 | Touch、VoiceOver、Voice Control、Switch Control、Full Keyboard Access、外接键盘 |
| 后台 | 只允许系统能力执行 hint/resync；不续 supervision lease |

### 2.1 支持窗口

- 支持资格由短期有效、签名且防回滚的 build allowlist 决定，不只看 `iOS >= 18`。
- 产品目标为滚动 24 个月，但不得越过 Apple security、WebKit、App build、加密或服务端 protocol floor。
- 每月复核 iOS security、WebKit、App Review、隐私要求、APNs、Xcode SDK 和当前签名 build；发生高危事件时立即复核。
- allowlist metadata 使用短期 expiry；过期、签名无效、bundle identity 不匹配或版本低于 floor 时 fail closed。
- Public 与 managed 渠道分别评估，不因同一源代码而共享尚未证明的分发、推送、设备绑定或 MDM 证据。

## 3. 事实、决策、合同、实现与证据区分

| 类别 | 本文含义 | 当前 iPhone v1 |
|---|---|---|
| 平台事实 | Apple 官方页面在查询日描述的行为 | 见 §20；动态页面需发布前复核 |
| 产品决策 | `CONSOLE-IOS-V1-DEC-*` 冻结的产品方向 | accepted product direction |
| Machine contract | registry 中真实 `REQ-*` 及其 schema/vector/transition | 仅部分通用合同；移动 carrier 多数 `unregistered` |
| Implementation | 代码与集成是否存在并可构建 | `not-implemented` |
| Platform evidence | 真机、simulator、APNs、商店、AT、安全测试结果 | `none` |
| Existing vector evidence | runner 是否真实执行既有向量 | 76 份全部 `not-run` |
| Profile | 全部适用 MUST 是否有通过或有据不适用 | `planned / not implemented` |

合同字段使用：

- `partial: REQ-*`：存在相关通用机器合同，但不足以表达完整移动行为；
- `unregistered`：移动 carrier、设备签名、lease eligibility、APNs binding 等尚无机器合同；
- `product-only`：纯客户端布局、文案、无障碍或本地隐私决策；
- 任何 `partial` 都不等于移动行为已登记，更不等于实现或测试通过。

## 4. 冻结产品决策摘要

### 4.1 角色与能力上限

- Canonical：[CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001)
- 状态：`accepted / blocked on contracts and implementation`
- iPhone v1 是“更广但受限的远程 Console”，面向 Agent operator。
- 可提供 Conversation/Task、监督纠偏、tenant/node 选择和远端 Agent 全生命周期。
- App 不是 node、daemon、authority、IdP 或 final arbiter；手机本地状态不推进 authority 状态。

### 4.2 支持形态与期限

- Canonical：[CONSOLE-IOS-V1-DEC-002](./mobile-platform-decision-log.md#console-ios-v1-dec-002)
- 状态：`accepted`
- GA 仅 iPhone、iOS 18+、arm64；竖屏为主、横屏完整。
- iPad、Watch 及其他系统 surface 保持 `planned/blocked`。
- 支持目标滚动 24 个月，但以每月复核的 signed build allowlist 和 security/WebKit/app floor 为硬上限。

### 4.3 市场与设备所有权

- Canonical：[CONSOLE-IOS-V1-DEC-003](./mobile-platform-decision-log.md#console-ios-v1-dec-003)
- 状态：`accepted`
- 首发美国、新加坡；同期 `en` 与 `zh-CN`。
- BYOD 与 managed device 都在范围内；managed policy 只能收窄，不能扩大共同能力上限。
- OS 用户、Apple Account、设备所有者、MDM 管理员与 CognitiveOS Owner/operator 不得混同。

### 4.4 Account-first 身份与设备绑定

- Canonical：[CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004)
- 状态：`accepted / mobile carrier unregistered`
- 先经 `ASWebAuthenticationSession` / 系统浏览器完成 OIDC/OAuth 2.1 Authorization Code + PKCE；IdP passkey 可用。
- authority 返回 tenant/node；随后以 account、device key、authority-bound `install_generation` / app-install identity、session 和 authority-side push registration mapping 完成 device enrollment challenge。
- 单一活动账号可切换多个 tenant/node；换号、登出、reinstall、restore、换机必须 revoke/rebind。
- device key 使用 `ThisDeviceOnly` 保护而不能迁移到新设备；同机 restore/reinstall 后即使 Keychain artifact 仍存在，旧 `install_generation` 也不能继续使用。

### 4.5 iPhone 信息架构

- Canonical：[CONSOLE-IOS-V1-DEC-005](./mobile-platform-decision-log.md#console-ios-v1-dec-005)
- 状态：`accepted`
- 使用 iOS native shell、`TabView`、每 tab 独立 `NavigationStack` 和短决策 sheet。
- 五个一级 tab 固定为 Work、Tasks、Agents、Inbox、More。
- Task、Loop、AgentExecution、Effect、Verification 五个独立 authority lifecycle 域、独立远端 Runtime projection、Trust Strip 与 Flow Thread 的语义与桌面一致，但不复制桌面三栏或像素外观。

### 4.6 生命周期与 supervision lease

- Canonical：[CONSOLE-IOS-V1-DEC-006](./mobile-platform-decision-log.md#console-ios-v1-dec-006)
- 状态：`accepted / lease carrier unregistered`
- 仅当 scene active、设备解锁、AuthenticationSession 当前、watch/UI fresh 且 App 可响应时续租。
- inactive、background、lock、terminated、force-quit 均立即停止客户端续租资格。
- authority 可使用预先固定的短 grace；BGTask、background push、URLSession 和 continuous background task 都不得续租。

### 4.7 APNs 与通知

- Canonical：[CONSOLE-IOS-V1-DEC-007](./mobile-platform-decision-log.md#console-ios-v1-dec-007)
- 状态：`accepted / notification carrier unregistered`
- APNs payload 只含最小 opaque handle；通知正文通用化。
- raw APNs token 只按 App+device+environment/topic 定位；authority registration mapping 把它绑定到当前单一活动 account/device/channel，tenant/node 只进入 opaque handle audience。
- 唯一 action 是打开 App；打开后 reauth、原子解析 handle、resnapshot。
- receipt、APNs accepted、delivered、displayed、clicked、badge 均不作为 truth、authorization 或 evidence。

### 4.8 Digest-bound R1

- Canonical：[CONSOLE-IOS-V1-DEC-008](./mobile-platform-decision-log.md#console-ios-v1-dec-008)
- 状态：`accepted / signature and display carrier unregistered`
- versioned `CanonicalDisplayEnvelope` 固定完整显示/签名字段；同一 immutable decoded envelope 驱动 native UI，签名覆盖完整 envelope digest。
- Face ID/Touch ID 只解锁 Secure Enclave P-256 不可导出 device key；native UI + device signature 不证明 compromised client 向用户显示了同一内容。
- authority 最终验证并决定；passkey 仅作 upstream login。
- biometric enrollment 改变使 key 失效并要求 rebind；没有可用 key 时 R1 blocked，不提供 passcode 降级。
- 当前不宣称抵御 compromised client；若 threat model 要求可信显示，R1 保持 blocked，带外/硬件可信确认须另行决策。

### 4.9 Offline、内容与隐私

- Canonical：[CONSOLE-IOS-V1-DEC-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009)
- 状态：`accepted`
- 敏感 draft 永不持久化；可持久化项仅限非敏感连接元数据、stable refs、设置，以及由 install-bound non-migrating key 应用层加密、再配 `FileProtection.complete` 与 backup exclusion 的非敏感 draft。
- 敏感 last-good 只存在当前进程内并显示 `as_of`；离线不排队控制动作。
- 内容使用 native escaped text + allowlist Markdown；链接交给外部浏览器；file importer 只接收不可信数据上传。
- 通知、app switcher、pasteboard、capture、backup 和 diagnostics 采用 §11 的收窄策略。

### 4.10 远端 Agent 生命周期与 acquisition

- Canonical：[CONSOLE-IOS-V1-DEC-010](./mobile-platform-decision-log.md#console-ios-v1-dec-010)
- 状态：`accepted / partial machine contract`
- iPhone 只提交 authority catalog/package ref。
- 远端 node 负责下载、验签、检查、sandbox、安装、升级、回滚、卸载和 authority 状态迁移。
- iPhone 永不下载、解释、执行、解压或转发 Agent executable bundle；文件导入不得作为 package acquisition 旁路。

### 4.11 分发身份隔离

- Canonical：[CONSOLE-IOS-V1-DEC-011](./mobile-platform-decision-log.md#console-ios-v1-dec-011)
- 状态：`accepted`
- Public App Store 与 ABM Custom App 使用不同 bundle identity、APNs topic、Associated Domains、App Attest/device binding namespace 和 release records。
- 两渠道使用同一源代码和能力上限；managed configuration 只能收窄。
- TestFlight 服务对应目标 bundle；Enterprise Program 不提供。

### 4.12 更新、security floor 与 kill switch

- Canonical：[CONSOLE-IOS-V1-DEC-012](./mobile-platform-decision-log.md#console-ios-v1-dec-012)
- 状态：`accepted / floor carrier unregistered`
- App 不自行下载或执行更新 binary；更新只通过 App Store、TestFlight 或 ABM/MDM 分发。
- recommended minimum 只提示；signed hard floor 可选择性阻断 lease、R1、Agent lifecycle 和 protected writes。
- kill switch 保留安全只读、revoke、诊断预览和更新恢复；不得改变 authority Task/Effect 状态或终止已接受任务。

### 4.13 Recovery 与 integrity signals

- Canonical：[CONSOLE-IOS-V1-DEC-013](./mobile-platform-decision-log.md#console-ios-v1-dec-013)
- 状态：`accepted`
- jailbreak、App Attest、device key、系统完整性、MDM 与 OS 状态都只是风险信号。
- 明确异常可选择性阻断 lease、R1、Agent lifecycle 和 protected writes，同时保留安全只读、设备 revoke、登出和更新恢复。
- 丢失设备、账号切换、restore、reinstall、换机和 key anomaly 均以 authority revoke/rebind 收敛。

### 4.14 Accessibility、rotation 与 motion

- Canonical：[CONSOLE-IOS-V1-DEC-014](./mobile-platform-decision-log.md#console-ios-v1-dec-014)
- 状态：`accepted`
- 44 pt 触控目标；VoiceOver、Voice Control、Switch Control、Full Keyboard Access、最大 Dynamic Type、外接键盘、横竖屏与 Reduce Motion 静态等价是 GA gate。
- GA evidence 必须分别覆盖 Touch ID/Face ID、最小/最大支持屏幕、最旧 admitted iOS build/当前 build、Public/managed、portrait/landscape 和最大 Dynamic Type，不得由单机外推。
- 状态不只用颜色、位置或动画表达；motion 不能制造 authority 推进或完成感。
- R1、pause、reconcile、Agent lifecycle、revoke 和 update recovery 必须可由辅助技术独立完成。

### 4.15 Telemetry 与 diagnostics

- Canonical：[CONSOLE-IOS-V1-DEC-015](./mobile-platform-decision-log.md#console-ios-v1-dec-015)
- 状态：`accepted`
- 只使用最小、第一方、content-free telemetry；无广告、无 tracking、无第三方 analytics。
- crash/diagnostics 默认不自动上传敏感内容；生成字段预览，用户显式确认后上传。
- 隐私 manifest、required-reason API、Privacy Label 与实际依赖逐 build 对齐。

### 4.16 状态与证据纪律

- Canonical：[CONSOLE-IOS-V1-DEC-016](./mobile-platform-decision-log.md#console-ios-v1-dec-016)
- 状态：`accepted`
- specified、implementation available、test executed、Profile implemented 四态严格分离。
- `CANDIDATE_COMPLETE`、远端 completed、receipt、推送点击、生物识别成功和客户端缓存都不是完成证据。
- 当前固定声明：implementation not-implemented；platform evidence none；76 vectors not-run；Profile not implemented。

## 5. 信息架构、导航、页面、状态与布局

### 5.1 五个一级 tab

| Tab | 用户问题 | 主要内容 |
|---|---|---|
| Work | 我现在要做什么？ | Conversation、目标输入、preview、最近重点 Task |
| Tasks | 正在发生什么？ | 任务列表、五个 authority lifecycle 域、独立 Runtime projection、监督、纠偏、pause、reconcile |
| Agents | 远端能力如何变化？ | 已安装 Agent、catalog refs、install/upgrade/rollback/uninstall |
| Inbox | 哪些事项等我处理？ | R1、输入、pause pending、unknown、session/floor/integrity 事项 |
| More | 账号、设备与系统是否安全？ | tenant/node、devices、notifications、privacy、diagnostics、update/floor |

规则：

- 当前 account、tenant、node 和 freshness 在每个需要写操作的页面可发现；不只靠 tab 或颜色表达。
- 每个 tab 保留自己的 navigation path；账号、tenant/node 或 authorization epoch 改变时，相关 path、draft resource binding、preview 和 cache 原子失效。
- sheet 只用于短、单一、可取消决定；R1 使用可信原生确认面；Agent 生命周期和 `OUTCOME_UNKNOWN` 使用独立页面。
- 窄屏单列优先；详情逐层披露，不把桌面三栏缩成手机卡片墙。

### 5.2 页面信息层级

每个对象页依次呈现：

1. 用户摘要：发生了什么、是否需要行动；
2. 推荐动作：至多一个主动作及安全替代；
3. Trust Strip：account、tenant/node、authority、version、`as_of`、freshness；
4. 事实：目标、状态、预算、risk、deadline、失联策略；
5. 技术详情：机器对象、稳定 ref、digest、Effect、Verification 和证据。

### 5.3 任务状态表达

- 首层使用 Governed Flow Thread：目标固定 → 执行中 → 验证中 → 已接受。
- 展开后 Task、Loop、AgentExecution、Effect、Verification 是五个独立 authority lifecycle 域；Runtime 是独立远端 projection，不是第六个本地 authority 状态。
- `OUTCOME_UNKNOWN` 从主线分支到 Reconcile；不显示 Retry。
- `CANDIDATE_COMPLETE` 与 `COMPLETED` 使用不同文案、形状和可访问标签。
- pause request、pause pending、safe checkpoint paused 和 Runtime stopped 不混同；Runtime stop 不推进 Task 或 Effect。

## 6. 组件、数据流与信任边界

### 6.1 逻辑组件

| 组件 | 职责 | 禁止拥有 |
|---|---|---|
| Native App Shell | Tab/NavigationStack、页面状态、系统控件 | authority state、任意 HTML/JS |
| Authority Client | TLS 请求、snapshot/watch、稳定 ref 对账 | 本地 commit、risk 降级 |
| Identity Coordinator | ASWebAuthenticationSession、PKCE、单活动账号 | IdP 密码、跨账号 token 复用 |
| Device Binding Service | Secure Enclave key、Keychain handle、install_generation、challenge signature | 导出 private key、passcode fallback、跨 install generation 复用 |
| Lease Eligibility Monitor | scene/lock/session/watch/UI eligibility | 本地延长 grace、后台续租 |
| APNs Coordinator | token 注册、topic/environment、opaque handle | account truth、通知批准 |
| Content Renderer | native escaped text、allowlist Markdown | raw HTML、JS、iframe、bridge、remote auto-load |
| Draft Store | install-bound 应用层加密的非敏感 draft | 敏感 draft、authority snapshot、控制动作队列 |
| Diagnostics Builder | content-free telemetry、字段预览、显式上传 | 自动上传 secret/正文 |

### 6.2 数据流

```text
系统浏览器 / ASWebAuthenticationSession
        │ Authorization Code + PKCE
        ▼
upstream IdP ──> authority account/session
                         │
                         ├─ tenant/node projection
                         ├─ device enrollment challenge
                         └─ APNs registration binding
                                  │
                                  ▼
Native SwiftUI App ── current snapshot/watch ── authority
        │                         │
        │ envelope digest         ├─ risk floor / authorization
        ▼                         └─ Task/Effect/Verification/Acceptance
Secure Enclave key
        │ signature
        └────────────────────────> authority final decision
```

### 6.3 信任边界

- APNs、deep link、Universal Link、file URL、Markdown、Agent metadata、diagnostics 和 local cache 均为不可信输入。
- Native UI 可以可信地展示 app-owned fields，但其本地结论仍不是 authority。
- ContextView 是非 authority projection；授权与提交必须重新读取 current authority state/version。
- Public 与 managed App 的 credential、Keychain access group、raw APNs token、Associated Domains、device binding、install generation、cache 和 diagnostics namespace 不互通；同一 App 内 tenant/node 不要求不同 raw token 或 device key。
- App Attest key 与 R1 device key 分离；前者只提供风险信号，后者只签署固定 operation challenge。

## 7. 账号、设备绑定、revoke 与恢复

### 7.1 Account-first 登录

1. App 以 `ASWebAuthenticationSession` 打开 authority-approved HTTPS authorization endpoint。
2. 使用 Authorization Code + PKCE；state、nonce、redirect audience 和 callback 均固定。
3. IdP 可使用 passkey；App 不收集或验证 IdP 密码。
4. authority 完成 code exchange、签发短期 session，并返回该 principal 可发现的 tenant/node。
5. App 同时只保持一个活动账号；切换账号先 revoke/清理旧账号本地绑定。

Public 与 Custom 任一 target 的登录 flow 如果支持创建账号，都必须提供 App 内可发起的账户删除路径；如果完全不支持创建账号，则必须保留可验证的产品、IdP 配置和 App Review evidence。删除账号与 revoke 单一 device binding 是两个独立动作，不能互相替代。

### 7.2 Device enrollment

1. 每个 app install 在 App container 内创建高熵 `install_marker` / nonce；它不存入 Keychain，并设置 backup exclusion。Marker 只用于 honest reinstall/restore detection，不是 credential 或 authority。
2. 每次首次启动先读取 marker。marker 缺失，或其 digest 与 authority 当前 app-install binding 不匹配时，即使 Keychain key/session artifact 仍存在，也先进入隔离：停止 lease/R1/protected writes，丢弃旧 Keychain/session/notification handle 的本地引用，要求 fresh login/enrollment，并请求 authority revoke 旧 generation。
3. fresh enrollment 时，App 为当前 bundle identity、账号和 app-install 生成 Secure Enclave P-256 key。
4. private key 使用 `kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly` accessibility，以及 `privateKeyUsage` + `biometryCurrentSet` access-control flags；不允许 passcode fallback。
5. `ThisDeviceOnly` 保证 item 不迁移到新设备，但本文不推断同机 restore/reinstall 一定删除该 Keychain artifact；移除 device passcode 会使该保护级别的 item 不可用。
6. authority 返回一次性 enrollment challenge，固定 account、bundle/channel、install marker digest、device public key、session、nonce、expiry；server 在 fresh enrollment 中 mint 新的 `install_generation` / app-install identity。
7. App 签名并提交；authority 原子登记 device binding 与新 install generation。即使同机 restore/reinstall 后旧 Keychain artifact 仍存在，旧 generation 的签名也必须被拒绝。
8. raw APNs token 独立于 account/tenant/node，只属于 App+device+environment/topic。authority registration mapping 将 token 绑定到当前单一活动 account、device binding 与 channel；常规 token rotation 原子替换 routing mapping，不轮换 device key。
9. tenant/node 进入 opaque notification handle audience 和 R1 `CanonicalDisplayEnvelope`，不要求按 tenant/node 生成不同 raw token 或 device key。

Marker 只能检测 honest reinstall/restore 行为，不能证明 compromised client 没有伪造或恢复 marker。需要更强 app-instance 信号时可使用 App Attest 作为 risk signal，但它不代替用户账号、R1 key、设备绑定或 authority 决定；App Attest unsupported、server error 或 assertion error 必须按版本化 policy 阻断 R1/lease（可保留安全只读、revoke 与恢复），不能当作通过。

### 7.3 Revoke 与 rebind

以下事件必须停止 lease/R1/protected writes，清除适用 session、watch、cache、draft binding 和 notification handle，并要求 authority revoke/rebind 或更新 install generation：

- 登出或切换账号；
- biometric enrollment 改变；
- device key missing/invalidated；
- device passcode removal；
- App-container `install_marker` 缺失、与 authority binding 不匹配、App reinstall、同机 restore 或 app-install identity / `install_generation` 不匹配；
- 从 backup restore 到新设备；
- 换机或设备转让；
- Public/managed 渠道切换；
- authority device revoke、session revoke 或 tenant access removal；
- jailbreak/App Attest/system signal 达到明确阻断阈值。

丢失设备恢复从其他受信渠道发起 remote revoke；旧设备离线时依赖 session/device-binding expiry，不能假定已收到撤销。

首次启动触发上述 marker 隔离时，App 不继续使用可能残留的 Keychain key/session/notification handle：先丢弃本地 handle/reference，fresh enrollment 后由 server mint 新 generation，再恢复适用能力。该路径只闭合 honest reinstall/restore；compromised-client resistance 仍按前述 App Attest risk policy 和 §9 trusted-display 边界处理。

## 8. 生命周期、通知与 supervision lease

### 8.1 状态与 lease 资格

| 状态 | 客户端行为 | Lease |
|---|---|---|
| scene active + unlocked + fresh | snapshot/watch、交互、符合门禁时续租 | 可申请续租，最终由 authority 决定 |
| inactive | 立即停止新续租，保存最小非敏感 UI 状态 | 不续 |
| background | teardown/遮蔽敏感 UI；允许系统批准的 hint/resync | 不续 |
| locked / protected data unavailable | 清除敏感进程内 projection；标记 reauth | 不续 |
| suspended | 不执行代码 | 不续 |
| system terminated | 无可靠 final callback；下次冷启动恢复 | 不续 |
| user force-quit | 运行中的 continuous task/URLSession 等按系统规则取消或停止；无可靠取消通知 | 不续 |

authority 可在 lease 签发时固定短 grace，但：

- grace 不能由 App 本地 timer、APNs、BGTask 或 URLSession 延长；
- grace 结束后的 pause pending / safe checkpoint / unable-to-pause 必须来自 authority；
- iPhone 恢复前台后先 reauth、resnapshot、恢复 watch，再由用户显式恢复 supervision。

### 8.2 后台能力用途

- `BGAppRefreshTask`：最多用于拉取非敏感 Inbox 计数、刷新 token registration 状态或准备 resync hint；不提交写。
- `BGProcessingTask`：v1 默认不需要；不得借其维持 watch/lease。
- `BGContinuedProcessingTask`：远程监督不属于其用户可见本地长任务用途；v1 不使用。
- background URLSession：只用于用户显式发起的 diagnostics/data upload 或可恢复下载；不承载双向 watch。
- background push：只置“需要前台 resync”标记；不解析成 authority 状态，不续租。

### 8.3 APNs 与通知

- Public 与 managed 使用独立 bundle identity、topic、provider credentials 和 registration table；raw token 由 APNs 按 App+device+environment/topic 提供，不按 account、tenant 或 node 生成。
- 每次启动/系统返回 token 时上送当前 token；不把 token 当 device identity。authority 将 registration mapping 绑定到当前单一活动 account、device binding 与 channel。
- provider 校验 environment/topic，处理 token inactive/expired；常规 token rotation 原子替换 routing mapping，不轮换 device key。错 account mapping、错 channel、错 topic 全部拒绝。
- payload 只含随机、高熵、短时 opaque handle 和通用 `aps` 字段；不含 stable object ref、tenant/node alias、正文、risk、secret 或 approval data。
- tenant/node 只进入 opaque handle audience；它们不要求独立 raw token 或 device key。
- 通知只有默认“打开 App”行为，不注册控制 action。
- App 打开后验证当前 bundle、account、device binding、audience、expiry、single-use，再向 authority 解析事项并 resnapshot。
- Focus、Summary、preview、权限关闭、延迟、重复、乱序和丢失均不改变 Inbox、deadline 或 authority state。

## 9. R0 与 digest-bound R1

### 9.1 共同门禁

所有写操作都必须：

1. 重新读取 authority current state、target ref/version 和 authorization；
2. 由 authority 计算 risk floor；
3. 固定 proposal、parameter digest、nonce、expiry、session 和 idempotency；
4. 对 conflict、stale、revoked、wrong tenant/node、unknown outcome fail closed；
5. 返回稳定 request/effect refs，响应丢失时按原绑定查询。

### 9.2 R0

- 只有 authority 可判定 R0。
- R0 可使用明确按钮或在产品已说明的无外部可观察内部动作中自动提交。
- R0 提交仍生成 authority event/audit；失败、超时或响应丢失不显示成功。
- 通知、deep link、离线 draft 和本地 cache 不能直接触发 R0。

### 9.3 Versioned `CanonicalDisplayEnvelope`

每个 R1 proposal 必须由 authority 生成一个 versioned `CanonicalDisplayEnvelope`。Envelope 的 canonical encoding、field order、normalization、domain separation 和 digest algorithm 属于尚未登记的 machine contract；至少必须包含并固定：

- account 与 principal；
- tenant 与 node；
- bundle identity 与 channel；
- device binding 与 authority-bound `install_generation` / app-install identity；
- session；
- action；
- target 与 expected version；
- parameters；
- risk；
- budget；
- egress；
- deadline；
- Verification 与 Acceptance；
- supervision、cancel、reconcile 与 compensation 边界；
- nonce；
- expiry；
- idempotency；
- display profile version 与 App build。

App 必须先一次性严格解码 envelope，得到一个 immutable decoded envelope。渲染、可访问性文本、确认按钮语义、短 digest 和待签名 envelope digest 都必须从同一个 immutable decoded envelope 派生；禁止为显示和签名分别解析、规范化或读取可变对象。Device signature 必须覆盖完整 canonical envelope digest，而不是字段子集、显示摘要或另行重建的数据。

### 9.4 R1 原生确认

确认过程：

1. 初始焦点在标题和变化摘要，批准不是默认 Enter action。
2. 用户可取消或返回修改范围；任何变化创建新 proposal。
3. App 使用仅 biometrics 的 access-control 路径解锁 Secure Enclave key；不允许 device passcode fallback。
4. App 使用同一 immutable decoded envelope 驱动 native UI，并签署包含 device binding 与 `install_generation` 的完整 envelope digest。
5. authority 验证签名、当前授权、risk、freshness、single-use 和 expected version 后决定。
6. 用户取消、biometry lockout、enrollment change、key missing、过期或网络未知时不降级；进入取消、rebind 或原 proposal 查询。

Face ID/Touch ID 的成功结果只说明本地系统认证完成；只有完整 envelope digest 的有效 device signature 加 authority decision 才构成 R1 输入。

Native UI 与 Secure Enclave signature 能收窄解析分歧、重放和私钥导出风险，但不能证明 compromised client 向用户显示的内容与其签名 envelope 相同。iPhone v1 当前不宣称抵御 compromised client。若适用 threat model 要求可信显示，R1 必须保持 `blocked`，并对带外或硬件可信确认另行立项和决策；不能仅凭本地 native UI、biometric prompt 或 device signature 解阻。

### 9.5 R2/R3

- 只显示原因、所需可信确认能力和缩小范围路径。
- 无批准按钮、通知 action、聊天文本、passkey、设备密码或 debug bypass。
- 新范围必须由 authority 重新分类，客户端不能把 R2/R3 标成 R1。

## 10. Offline、idempotency 与 `OUTCOME_UNKNOWN`

### 10.1 Offline

- 可显示当前进程内敏感 last-good，但必须标出 authority、version、`as_of` 和“非实时”。
- 敏感 draft 永不持久化；冷启动只恢复非敏感连接元数据、stable refs、设置和可解密的非敏感 draft。
- 所有控制动作禁用；可编辑的持久 draft 必须保持非敏感，且不得显示 queued/submitted/accepted。
- 网络恢复后先 reauth、resnapshot、恢复 watch，再检查 draft 的 account/tenant/node/resource binding、app-install identity 与 `install_generation`。

### 10.2 Idempotency

- 每个写操作使用 authority 提供或合同定义的稳定 idempotency binding。
- same key + different parameters 必须拒绝。
- App 不生成新 key 来掩盖 timeout，也不本地合并相似请求。
- account、tenant/node、session、target version 或 proposal digest 改变使旧提交控件失效。

### 10.3 `OUTCOME_UNKNOWN`

- 页面保留原 proposal、target、parameter digest、idempotency、dispatch evidence、Effect ref 和最后 authority state。
- 禁止 Retry、换 key、假定失败、删除记录或以 Runtime stop 清除未知。
- 只允许按原绑定查询、reconcile、查看 evidence、进入 quarantine 或发起独立 compensation proposal。
- 收敛为 executed 后继续 Verification；收敛为 not-executed 后进入适用终态；仍未知则维持 unknown/quarantine。

## 11. Storage、内容、deep link、文件与隐私

### 11.1 本地存储

可持久化：

- bundle/channel 分区的 endpoint、非敏感 alias、tenant/node stable refs；
- UI 语言、通知偏好、上次非敏感路由；
- device key identifier/Keychain reference，不含 private key；
- 用户明确保存且经分类为非敏感的 draft：使用 install-bound non-migrating key 做应用层加密，按 account/tenant/node 与 `install_generation` 分区，再配 `FileProtection.complete` 和 backup exclusion；
- content-free crash marker 和 diagnostics manifest。

禁止持久化：

- access/refresh token 明文、R1 secret、nonce、approval input；
- Conversation/Task/Effect/Verification 敏感 snapshot；
- 任何敏感 draft；
- APNs payload 正文、notification handle 历史；
- Agent executable、package archive、raw HTML 或 remote image cache；
- jailbreak/App Attest 信号的未经限制长期画像。

### 11.2 内容

- 使用 SwiftUI/TextKit native escaped text 与版本化 allowlist Markdown。
- 允许基础段落、标题、列表、引用、受限 code block 和安全 link label。
- 拒绝 raw HTML、script、style、iframe、object/embed、event handler、data/file/custom scheme、SVG active content 和 native bridge。
- remote image 不自动加载；v1 默认不显示远程图片。
- Unicode/Bidi 控制符在稳定 ID、域名、digest 和可信字段中隔离并可显示 punycode/原始值。
- 链接先规范化和显示目标 host，再由用户动作交给系统外部浏览器；App 不预取。

### 11.3 Deep link 与 Universal Link

- 只允许独立 Public/managed Associated Domains 中的 HTTPS Universal Link。
- link 只携短时 opaque handle；不携 credential、stable object ref 或写参数。
- 所有 path/query/fragment、audience、bundle/channel、account、expiry 和 single-use 重新验证。
- Universal Link 不能直接删除、安装、暂停、批准或打开敏感正文；必须先 reauth/resnapshot。
- custom URL scheme 不作为生产写入口。

### 11.4 文件

- `fileImporter` 只用于用户明确选择的不可信数据附件上传。
- 限制 UTType、单文件/总字节、数量和读取时间；读取后及时释放 security-scoped access。
- 客户端不解释、渲染 active content、解压 archive 或执行文件。
- authority/远端安全服务负责内容检查；上传成功不表示内容可信。
- 检测为 Agent package、executable、archive acquisition 或路径旁路时拒绝；iPhone 不转发 Agent executable bundle。

### 11.5 通知、app switcher、pasteboard、capture 与 backup

- 通知标题/正文保持通用，不显示账号、tenant/node、Task、Agent、risk 或结果。
- scene inactive/background 前以全屏 privacy cover 替换 app-switcher snapshot。
- R1、secret、token、nonce、敏感正文无 Copy action；stable ref/digest 的复制与正文复制分开。
- 如复制允许的非敏感值，pasteboard 使用 `localOnly` 与短 expiry；不把它写成仅本 App 可见。
- `isCaptured` 为真时遮蔽 R1 和敏感详情并解释原因；截图通知发生在截图后，产品不宣称阻止所有截图。
- 敏感 draft 永不持久化。允许持久化的非敏感 draft 使用 install-bound non-migrating key 应用层加密，并同时配置 `FileProtection.complete` 与 backup exclusion。
- `isExcludedFromBackup` 只作 backup hygiene，不是安全证明；App 不依赖该 flag 保证 confidentiality、删除或不可迁移。
- restore/reinstall 后如果没有匹配的 app-install identity、`install_generation` 和解密 key，残留 draft ciphertext 必须保持不可解密并删除；不得尝试跨 generation 恢复。
- `ThisDeviceOnly` 阻止 key 迁移到新设备，但不据此声称同机 restore/reinstall 一定删除 artifact；device binding 仍必须由 authority generation 检查。

### 11.6 Telemetry 与 diagnostics

- 不集成广告、tracking 或第三方 analytics SDK。
- 第一方 telemetry 只记录页面/状态类、耗时 bucket、错误类别、build/channel、非内容计数；不记录正文、prompt、文件名、URL query、stable ref、digest、tenant/node alias 或 biometric result detail。
- diagnostics bundle 在本地生成字段清单和敏感度预览；用户显式选择后才上传。
- crash report、OS log、APNs token、App Attest key ID 和 device binding ID 均按最小 retention、访问控制和隐私披露处理。

## 12. Agent lifecycle、acquisition 与 App Store dynamic-code 边界

### 12.1 允许的移动交互

`authority catalog/package ref → current package evidence → lifecycle preview → R0/R1 → remote node transaction → authority installation/effect state`

iPhone 可：

- 浏览 authority 返回的 catalog metadata 和 installed Agent；
- 查看 package ref、publisher、signature/provenance、permissions、compatibility、sandbox evidence 和 risk；
- 提交 install、upgrade、rollback、uninstall proposal；
- 监督远端 transaction、Effect、Verification 和 rollback point；
- 对结果未知按原 ref reconcile。

### 12.2 严格禁止

- URL、Git、本地文件或分享扩展作为 Agent acquisition source；
- 在 iPhone 下载、缓存、解压、解释、执行、扫描或转发 Agent bundle；
- 把 Web content、JavaScript、插件或模型输出转换为 native capability；
- 让 remote metadata 隐藏或启用未经 App Review 披露的新客户端功能；
- 因 catalog trusted 就跳过远端签名、sandbox、compatibility、risk 或 authority gate。

### 12.3 App Review 收口

- App Store build 保持 self-contained；远端数据只驱动已审核、预先存在的 native UI 和服务端对象。
- 2.5.2 与 4.7 不能被解释为下载 native Agent code 的许可。
- 所有远端 Agent lifecycle 功能、demo account、测试 tenant/node 和 review notes 必须完整提供给 App Review。
- 如果审核将产品归类为 remote desktop、thin client、chatbot/plugin host 或动态软件目录，必须在 GA 前取得实际审核结果；本文不预测获批。

## 13. Distribution、signing、update、floor 与 kill switch

### 13.1 渠道

| 渠道 | 用途 | 身份 | 约束 |
|---|---|---|---|
| Public App Store | 公共客户与 BYOD | Public bundle/topic/domains/binding | App Review、公开 Privacy Label |
| TestFlight | Public 或 managed 对应 beta | 与目标 App 身份一致 | beta；external tester build review |
| ABM Custom App | 指定组织 managed 分发 | Managed bundle/topic/domains/binding | 每个版本 App Review；ABM/MDM |
| Enterprise Program | 不提供 | 无 | 不用于客户或本产品 GA |

Public 与 managed：

- 同源代码、同 R0/R1 上限、同无 Agent executable 原则；
- bundle ID、App Store Connect record、APNs provider/topic、Associated Domains、Keychain access group、App Attest namespace、device binding 和 cache 必须隔离；
- managed configuration 可关闭 Agent lifecycle、R1、diagnostics、external links、file upload 或要求更高 floor，不能启用 Public 不具备的能力。
- Public 与 Custom 任一 target 的登录 flow 如支持创建账号，都必须提供 App 内发起删除；如完全不支持创建账号，必须为每个 target 保留可验证 evidence。账户删除不替代 device revoke。

### 13.2 Signing 与更新

- 所有 executable 由 Apple 分发链签名验证；App 不加载 unsigned/self-modifying code。
- 普通 App 不下载、安装或执行自身更新；用户/MDM 通过 App Store 基础设施更新。
- 客户端显示 installed build、channel、signed floor metadata expiry 和恢复入口。
- App Store 审核/传播延迟不能成为安全正确性依赖。

### 13.3 Signed build allowlist

metadata 至少固定：

- channel 与 bundle identity；
- minimum/recommended build；
- minimum iOS/security/WebKit floor；
- protocol/schema compatibility window；
- revoked builds；
- issued/expiry、key ID、signature 和 anti-rollback epoch；
- capability-specific block policy。

行为：

- recommended minimum：提示更新，不自动声明不安全；
- hard floor：停止 lease renewal、R1、Agent lifecycle 和 protected writes；
- metadata expired/signature invalid/rollback：按 hard-floor unknown fail closed；
- 保留安全只读、设备 revoke、登出、diagnostics preview 和更新恢复；
- kill switch 不修改 Task、Loop、AgentExecution、Effect 或 Verification authority state。

## 14. Accessibility、motion、rotation 与 keyboard

### 14.1 GA accessibility gate

必须在支持矩阵中的真实 iPhone 上证明：

- VoiceOver 可完成登录、绑定、tenant/node 选择、创建 Task、R0、R1、pause、unknown reconcile、Agent lifecycle、revoke 和 update recovery；
- Voice Control 与 Switch Control 无需隐藏 gesture；
- Full Keyboard Access 可到达、识别并激活所有适用控件；它与“连接外接键盘后处理快捷键”是两个独立测试面；
- Dynamic Type 覆盖最大 accessibility sizes，无截断、重叠、丢失动作或横向正文滚动；
- 44×44 pt 最小触控目标，单手关键动作不依赖精细拖动；
- Bold Text、Increase Contrast、Differentiate Without Color 和 Button Shapes 可理解；
- 外接键盘可遍历、激活、取消；无单键破坏性/R1 action；
- 竖屏与横屏功能完整，rotation 后焦点、draft、proposal 和 navigation path 保持正确；
- Reduce Motion 下所有功能有静态等价。

### 14.2 GA evidence 等价类

GA evidence 必须按以下等价类分别留证，不得从单一设备、单一 biometric、单一 build、单一 channel 或单一方向外推：

| 维度 | 必测等价类 |
|---|---|
| Biometric | 至少一台 Touch ID iPhone；至少一台 Face ID iPhone |
| Screen | 支持矩阵中的最小屏幕；最大屏幕 |
| OS/build | 最旧 admitted iOS build；当前 admitted iOS build |
| Distribution | Public；managed Custom App |
| Orientation | portrait；landscape |
| Text | 默认 Dynamic Type；最大 accessibility Dynamic Type |
| Keyboard | Full Keyboard Access；外接键盘快捷键/按键路径 |

每个 release build 必须记录 device model、iOS build、App build、bundle/channel、biometric、screen class、orientation、Dynamic Type 和 AT 设置。未覆盖组合保持 `none/not-run`，不能由相邻组合推定。

### 14.3 VoiceOver 与动态状态

- 每页一个明确标题；路由后焦点移到标题或目标对象。
- 自定义状态提供 label、value、hint 和适用 action；机器 ref 可逐字符读取。
- 高频 watch 事件合并播报，不逐秒朗读 deadline，不因排序移动当前焦点。
- `OUTCOME_UNKNOWN`、pause pending、stale、R2/R3、floor-blocked 使用文字和结构，不只用颜色。
- R1 页面先读目标和变化，再到取消/确认；确认按钮不取得初始焦点。

### 14.4 Motion

- 只有新的 authority event 到达时，Flow Thread 执行一次短促更新。
- 无 event 时不播放“正在推进”的循环动画。
- Reduce Motion 时取消位移、缩放、parallax、shimmer、自动滚动和连续 spinner，改为静态替换、轮廓和一次 live announcement。
- 动画完成不表示 authority 完成。

## 15. Security threat model

当前所有 iOS threat-model evidence 均为 `none`；表中 oracle 是未来可执行验收，不是已通过测试。

| ID | 资产 | 攻击者 | 入口 | 信任边界 | 预防 | 检测 | 失败语义 | Owner | Oracle | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| `IOS-TM-01` | account/device notification routing | 恶意 provider、重放者 | forged/replayed push | APNs raw token → authority mapping → App | raw token 仅 App+device+environment/topic；mapping 绑定当前 account/device/channel；tenant/node 在 opaque audience | provider/APNs error、mapping generation、handle reuse、audience mismatch | 丢弃并前台 resnapshot；不改变事项 | UNASSIGNED — Notification | 错 account mapping/topic/environment/replay 全拒绝且不要求 tenant/node 独立 token/key | none |
| `IOS-TM-02` | deep-link target 与 session | 恶意站点/App | Universal Link、redirect | Associated Domain → App | HTTPS AASA、allowlist path、参数规范化、短 handle | malformed/audience/expiry 日志 | 无动作；回到安全首页 | UNASSIGNED — Mobile security | malformed、cross-channel、expired link 不能打开敏感对象或写 | none |
| `IOS-TM-03` | native UI 与 credential | 恶意 Agent/Markdown | raw HTML、script、remote image | untrusted content → native renderer | native escaped text、Markdown allowlist、无 WKWebView/bridge | sanitizer corpus、unexpected network monitor | 降级纯文本；写禁用 | UNASSIGNED — Content security | HTML/JS/iframe/SVG payload 无执行、网络或 native call | none |
| `IOS-TM-04` | R0/R1 gate | 锁屏持有者、通知伪造者 | notification action | OS notification → App | 唯一 action=open、前台 reauth/resnapshot | action identifier、session/freshness check | 只打开安全落点；不提交 | UNASSIGNED — R1/Notification | 锁屏点击不能批准、pause、retry、install | none |
| `IOS-TM-05` | CanonicalDisplayEnvelope 与用户所见内容 | phishing UI、compromised client | 显示/签名解析分歧、被攻陷 native client | authority envelope → native display / Secure Enclave → authority | versioned envelope、一次严格解码、同一 immutable object 渲染、签完整 envelope digest、nonce/session/expiry | envelope/digest/replay mismatch；无法由本机证明 compromised client 的真实显示 | 普通 mismatch 拒绝；若要求可信显示则 R1 保持 blocked | UNASSIGNED — R1 security | 字段变更/双解析/replay 均拒绝；明确记录 native UI+签名不证明 compromised-client display integrity | none |
| `IOS-TM-06` | device private key 与 app-install identity | backup、clone、enrollment attacker、compromised client | marker 缺失/伪造、新设备 restore、同机 restore/reinstall、passcode removal、新增 biometrics | App-container marker + Keychain/Secure Enclave + install_generation → authority | 非 Keychain/backup-excluded marker、`WhenPasscodeSetThisDeviceOnly`、`privateKeyUsage`、`biometryCurrentSet`、authority-bound generation | marker/generation/binding mismatch、key unavailable/invalidated、App Attest risk result | 隔离并丢弃旧 handles；停 lease/R1/write；revoke/fresh enrollment | UNASSIGNED — Device identity | honest reinstall/restore marker 缺失必触发新 generation；同机 artifact 不跨 generation；unsupported/error 不当通过；不宣称证明 compromised client | none |
| `IOS-TM-07` | app legitimacy 与 policy | jailbreak/tampered app | modified runtime/system signals | device signal → authority | App Attest/risk policy、server-side gate、短 session | attestation/assertion anomaly、key churn | 选择性阻断；保留 revoke/update/read-only | UNASSIGNED — Integrity | 信号缺失不被写成可信；明确异常触发固定收窄 | none |
| `IOS-TM-08` | R1/secret/projection | 本地观察者、恶意辅助 App | screenshot、recording、pasteboard、app switcher | UI → OS surfaces | privacy cover、capture mask、无敏感 copy、generic notification | isCaptured、screenshot-after-event、pasteboard audit | 遮蔽并提示；不宣称阻止截图 | UNASSIGNED — Privacy | app switcher/recording 无 R1；截图后状态无错误承诺 | none |
| `IOS-TM-09` | Effect/idempotency/lease | 网络攻击者、崩溃 | process death、offline、duplicate submit | App cache → authority | 无离线写队列、原 idempotency、lease expiry、resnapshot | stale cursor、duplicate/conflict、unknown state | `OUTCOME_UNKNOWN`；禁盲重试 | UNASSIGNED — Effect/Console | kill/network loss 后不重复 Effect、不显示假暂停/完成 | none |
| `IOS-TM-10` | account/token/install binding 与 draft ciphertext | backup/restore 混淆 | marker 缺失、restore、reinstall、换号、换机 | App-container marker/local storage/install generation → authority | marker 不进 Keychain且排除 backup、单活动账号、raw token 与 binding 分离、authority generation、非敏感 draft 用 install-bound non-migrating key 加密 | marker/app-install/generation/key mismatch、routing lookup、decrypt failure | 隔离并丢弃旧 handles，fresh enrollment；不可解密 ciphertext 删除 | UNASSIGNED — Identity | honest restore/reinstall 缺 marker 时旧 session/mapping/generation 不可用；无匹配 key 的 draft 不可解密且删除 | none |
| `IOS-TM-11` | supported build 与 WebKit/security floor | downgrade、审核延迟 | old build、stale metadata | App Store → App → authority | signed short-expiry allowlist、anti-rollback、kill switch | build/channel/metadata expiry | protected writes fail closed；保留恢复 | UNASSIGNED — Release security | revoked/expired/rollback build 不能续租或 R1 | none |
| `IOS-TM-12` | user content 与 diagnostics | SDK/vendor/insider | dependency、telemetry、crash upload | App/SDK → first-party diagnostics | 最小依赖、SDK manifest/signature allowlist、无 third-party analytics、content-free schema、预览后上传 | privacy report、dependency diff、egress inspection | 阻断 build/upload 并可删除 diagnostics bundle | UNASSIGNED — Privacy engineering | 未批准 SDK/build 无法发布；fixture 正文/ref/token 不出现在 telemetry/diagnostics | none |
| `IOS-TM-13` | App Store binary 与远端 node | 恶意 package/source | Agent bundle、archive、URL/Git、SSRF、ambient credential | iPhone → authority catalog → remote node | ref-only acquisition、客户端无 bundle path、远端 gate | network/file instrumentation、package MIME/path check | iPhone 拒绝；远端保持原 state | UNASSIGNED — Agent security | iPhone 零 Agent executable bytes；URL/Git/file 无 acquisition 入口 | none |
| `IOS-TM-14` | Public/managed tenant isolation | MDM admin、账号混淆 | managed config、bundle/channel switch | OS management → App → authority | 独立 bundle/topic/domains/binding；policy only narrows | channel/bundle/audience mismatch | 拒绝并要求正确 App/rebind | UNASSIGNED — Enterprise mobility | Public token/key/cache 不能用于 managed，反之亦然 | none |
| `IOS-TM-15` | authority truth 与 Runtime projection | compromised UI、Agent、用户误解 | local cache、remote completed、receipt、biometric success、Runtime stop | projection → native UI | 五个独立 authority lifecycle 域、独立远端 Runtime projection、authority refs/version、acceptance gate | inconsistent event/version、forced completion attempt | 保持 authority state；显示 conflict | UNASSIGNED — Console correctness | 非 authority “completed” 与 Runtime stop 均不能推进 Task/Effect 或显示 COMPLETED | none |
| `IOS-TM-16` | OS accessibility interaction | 恶意 overlay/误触、复杂 UI | Voice Control/Switch Control/keyboard | assistive input → native action | 标准控件、显式焦点、无手势唯一入口、R1非默认 action | accessibility audit + 真机 AT | 取消无决定；歧义不 dispatch | UNASSIGNED — Accessibility security | AT 可完成旅程且不能误触发 destructive/R1 | none |

## 16. 十五条关键 journeys

每条 journey 的 `Evidence` 都是当前真实状态；计划 oracle 不等于测试已执行。

### `CONSOLE-IOS-V1-JRN-001` 安装、登录与设备绑定

- **入口**：首次安装 Public/managed App、本地无有效 binding，或首次启动发现 `install_marker` 缺失/不匹配。
- **前置条件**：支持 build、HTTPS authority/IdP、当前 bundle/channel 配置有效。
- **可见步骤**：检查 App-container marker → 缺失/不匹配时隔离并丢弃旧 handles → 欢迎与角色边界 → 系统浏览器 fresh login → 选择/确认账号 → 展示设备绑定说明 → Face ID/Touch ID 创建 device key → 完成。
- **Authority 交互**：Authorization Code + PKCE；marker mismatch 时请求 revoke 旧 generation；返回 session/tenant/node；fresh enrollment 时 server mint 新 `install_generation` 并签发 challenge；原子登记 marker digest、device public key、app-install identity 和 channel。
- **OS surface**：ASWebAuthenticationSession、Secure Enclave、Keychain、LocalAuthentication、APNs registration。
- **成功结果**：单一活动账号和新 device binding；取得授权 snapshot。
- **失败/取消/重复/恢复**：marker 缺失绝不沿用残留 Keychain/session handle；登录取消无 session；challenge 过期重取；重复 callback/nonce 拒绝；key 创建失败则只读或退出；App Attest unsupported/error 按 policy 阻断 R1/lease而非当通过；不降级 passcode。
- **审计事件**：login/session issued、device-enrollment proposed/accepted/failed、push-binding registered。
- **可执行 oracle**：删除/替换 marker 后即使 Keychain artifact/session 存在也先隔离并 fresh enroll；cross-account、wrong bundle、replayed challenge、无 biometry、wrong generation 全拒绝；不把 marker 检测扩张为 compromised-client proof。
- **当前 evidence**：none；相关 management/session/capability vectors 为 not-run。

### `CONSOLE-IOS-V1-JRN-002` 选择 tenant/node 并进入移动主页

- **入口**：登录成功或从 More 切换工作范围。
- **前置条件**：authority 返回可发现 tenant/node 与对应权限。
- **可见步骤**：选择 tenant → 选择 node → 查看 capability/readiness 摘要 → 进入 Work 或安全待办。
- **Authority 交互**：每次选择重新授权；返回 scope-bound snapshot/watch cursor。
- **OS surface**：NavigationStack、sheet/selection list、Keychain session handle。
- **成功结果**：当前 scope 固定，Trust Strip 显示 account/tenant/node/freshness。
- **失败/取消/重复/恢复**：取消保留旧 scope；权限变化立即失效旧 route/cache/preview；node 不可达显示 last-good 而不切换成功。
- **审计事件**：scope-selected、scope-denied、watch-started。
- **可执行 oracle**：tenant A 的 cache、draft resource token、APNs handle 和 write 不得在 tenant B 复用。
- **当前 evidence**：platform none；通用 capability/cache vectors（例如 `CTX-REVOKE-CACHE-001`）不证明 mobile tenant/node isolation。

### `CONSOLE-IOS-V1-JRN-003` Conversation 创建 Task 与 R0 操作

- **入口**：Work 输入目标或继续 Conversation。
- **前置条件**：scene active/unlocked、session/watch fresh、task channel 可用。
- **可见步骤**：输入目标 → 澄清歧义 → authority preview → 显示 risk/预算/失联策略 → R0 自动或明确按钮提交 → 打开 Task。
- **Authority 交互**：固定 intent/proposal/target/version/idempotency；authority 判定 R0；返回稳定 refs。
- **OS surface**：SwiftUI form、keyboard、file importer（仅数据附件）。
- **成功结果**：至少返回 Task/AgentExecution refs；适用对象由 authority 创建。
- **失败/取消/重复/恢复**：歧义不 dispatch；stale preview 失效；响应丢失按原 ref 查询；offline 只保存经分类为非敏感、install-bound 加密的 draft，敏感 draft 仅留在进程内。
- **审计事件**：intent-fixed、preview-issued、R0-authorized/denied、task-created。
- **可执行 oracle**：ambiguous target、wrong scope、same-key different-parameters、offline submit 均无新 Effect。
- **当前 evidence**：none；`SHELL-TARGET-AMBIGUITY-001`、`EFF-IDEM-CONFLICT-001` not-run。

### `CONSOLE-IOS-V1-JRN-004` 查看 authority lifecycle 域、Runtime projection、监督、纠偏与请求暂停

- **入口**：Tasks 列表、Work 当前 Task 或安全 deep link。
- **前置条件**：授权 snapshot；若写则满足 lease eligibility。
- **可见步骤**：读用户摘要/Flow Thread → 展开 Task/Loop/AgentExecution/Effect/Verification 五个独立 authority lifecycle 域与独立远端 Runtime projection → 补充输入或请求 pause → 查看 pause pending/结果。
- **Authority 交互**：watch cursor/delta；纠偏创建新 governed input；pause 创建固定请求；只有 authority 推进状态。
- **OS surface**：NavigationStack、List、accessibility live region。
- **成功结果**：用户看到当前状态、下一 gate 和仍可能发生的 Effect。
- **失败/取消/重复/恢复**：watch gap 先 resnapshot；取消不提交；pause 超时保持 pending；Runtime stop 不推进 Task/Effect，也不显示 Task paused。
- **审计事件**：watch-resumed/gap、input-added、pause-requested/accepted/checkpointed。
- **可执行 oracle**：乱序/重复 delta 不造状态；五个 authority lifecycle 域与 Runtime projection 不被客户端合并。
- **当前 evidence**：none；`SHELL-WATCH-RESUME-006` 与相关 lifecycle vectors not-run。

### `CONSOLE-IOS-V1-JRN-005` 从 APNs 通知安全打开事项

- **入口**：用户点击通用通知。
- **前置条件**：当前 App channel 正确；opaque handle 未过期且未消费。
- **可见步骤**：打开 privacy-safe shell → 必要时登录/解锁 → 显示正在刷新 → authority 解析事项 → 打开 Inbox/对象页。
- **Authority 交互**：验证 account/device/channel/audience/expiry；原子消费 handle；current-state resnapshot。
- **OS surface**：APNs、UNUserNotificationCenter、Universal Link/default notification open。
- **成功结果**：打开当前事项，不依赖 payload 正文。
- **失败/取消/重复/恢复**：Focus/延迟/丢失不影响事项；handle 重放/错号显示安全失效；离线只进入 Inbox 框架。
- **审计事件**：notification-hint-issued、handle-consumed/rejected、item-opened。
- **可执行 oracle**：APNs duplicate/reorder/wrong topic/cross-account 不泄露正文、不执行动作、不改变 authority。
- **当前 evidence**：none；移动 notification carrier unregistered。

### `CONSOLE-IOS-V1-JRN-006` Digest-bound R1

- **入口**：authority 返回 R1 proposal，来自 Work、Task、Agent lifecycle 或 Inbox。
- **前置条件**：scene active/unlocked、fresh session/watch、有效 device binding/install generation、biometryCurrentSet 未改变；若 policy 要求 compromised-client-resistant trusted display，则本路径保持 blocked。
- **可见步骤**：严格解码 versioned `CanonicalDisplayEnvelope` → 同一 immutable decoded envelope 驱动目标/变化/risk/预算/egress/deadline/verification/lease 等完整原生显示 → 可取消/修改 → 明确动词按钮 → Face ID/Touch ID → submitting → authority 结果。
- **Authority 交互**：签发完整 envelope/challenge；验证完整 envelope digest 的 device signature、install generation、nonce/session/expiry/idempotency/risk/current version；最终决定。
- **OS surface**：SwiftUI native sheet/page、LocalAuthentication、Secure Enclave/Keychain。
- **成功结果**：authority 接受固定 proposal；返回稳定 decision/effect ref。
- **失败/取消/重复/恢复**：取消无决定；stale/expired/replayed/wrong session/double-decode mismatch 拒绝；biometry/key 失败进入 rebind 或 blocked；无 passcode fallback；可信显示要求未满足时保持 blocked。
- **审计事件**：R1-challenge-issued、user-cancelled、device-signed、authority-approved/denied。
- **可执行 oracle**：改变任一 envelope 字段、旧 nonce/session/key/generation、不同显示/签名解析、重复签名均不得 dispatch；测试明确不能把本机结果扩张为 compromised-client display integrity。
- **当前 evidence**：none；R1 mobile signature/display carrier unregistered。

### `CONSOLE-IOS-V1-JRN-007` `CANDIDATE_COMPLETE` 到 authority acceptance

- **入口**：Task 显示候选完成或收到 Inbox 提示。
- **前置条件**：固定 post-state；适用 Verification/Acceptance authority 可用。
- **可见步骤**：查看 candidate claim → 查看 Verification 与缺口 → 必要时提供验收输入 → 等待/查看 authority decision → 显示完成或继续。
- **Authority 交互**：验证 current Verification、fixed post-state 和 AcceptanceDecision；只有 authority 推进 `COMPLETED`。
- **OS surface**：Task detail、Inbox、外部 link（仅证据查看）。
- **成功结果**：只有通过完整 gate 才显示 `COMPLETED`。
- **失败/取消/重复/恢复**：inconclusive/expired verification 保持 candidate；远端 completed 只作 evidence；取消查看不改变状态。
- **审计事件**：candidate-reported、verification-recorded、acceptance-decided。
- **可执行 oracle**：缺 Verification 或 Acceptance 时任何本地/远端 completed 输入都不能显示完成。
- **当前 evidence**：none；`INTENT-ACCEPTANCE-007`、`GW-REMOTE-COMPLETE-001` not-run。

### `CONSOLE-IOS-V1-JRN-008` `OUTCOME_UNKNOWN` 对账

- **入口**：Effect response 丢失、dispatch 后断线或 authority 返回 unknown。
- **前置条件**：原 Effect/idempotency/parameter binding 可定位。
- **可见步骤**：高注意状态 → 查看原动作与证据 → 发起/等待 reconcile → 查看 executed/not-executed/still-unknown → 适用后续。
- **Authority 交互**：只按原 binding 查询 executor/Effect；Verification/compensation 是独立受治理步骤。
- **OS surface**：独立 Reconcile 页面、background hint 仅提示返回。
- **成功结果**：收敛到合法 Effect state，或保持 quarantine。
- **失败/取消/重复/恢复**：无 Retry；取消页面不取消 reconcile；App 重启后按 stable ref 恢复。
- **审计事件**：outcome-unknown、reconcile-requested/result、quarantined/compensation-proposed。
- **可执行 oracle**：换 key、停止 Runtime、重新安装 Agent 均不能清除或自动 commit 原 unknown Effect。
- **当前 evidence**：none；`EFF-UNK-003`、`EFFECT-STATE-CLOSURE-008` not-run。

### `CONSOLE-IOS-V1-JRN-009` inactive/background/lock/terminated/force-quit 后恢复

- **入口**：Home gesture、系统中断、锁屏、内存回收、崩溃或用户 app-switcher 关闭。
- **前置条件**：此前可能有活动 Task/lease。
- **可见步骤**：离开前 privacy cover → 停续租 → authority grace/expiry → 返回后登录/解锁 → resnapshot → 显示实际 pause/unknown/active。
- **Authority 交互**：不依赖 final callback；按 lease expiry 和 stable refs 收敛；返回时新 session/watch。
- **OS surface**：ScenePhase/UIScene lifecycle、protected-data notifications、app switcher。
- **成功结果**：恢复显示真实状态，不假定后台仍监督或已经暂停。
- **失败/取消/重复/恢复**：background task/push 不续租；force-quit 后无隐形恢复；冷启动清理旧敏感像素。
- **审计事件**：lease-renewal-stopped、grace-expired、pause-pending/paused、client-reattached。
- **可执行 oracle**：每种 lifecycle transition 后旧 instance 不能续租，且无错误暂停/完成声明。
- **当前 evidence**：none；supervision carrier unregistered。

### `CONSOLE-IOS-V1-JRN-010` Offline、弱网、push 延迟与 token 失效

- **入口**：网络不可达、high latency、APNs delayed/dropped、token rotated/inactive。
- **前置条件**：可能存在进程内敏感 last-good 与 install-bound 加密的非敏感 drafts。
- **可见步骤**：显示 `as_of`/offline → 禁用写 → 只编辑可持久化的非敏感 draft → 网络恢复 → reauth/resnapshot → 更新 authority routing mapping。
- **Authority 交互**：无网时零写；恢复后新 snapshot/watch；raw APNs token rotation 与 device key 分开，原子更新当前 account/device/channel mapping。
- **OS surface**：NW/URLSession reachability outcome、APNs registration、BGTask hint。
- **成功结果**：恢复 current state，非敏感 draft 经 scope、app-install identity 和 generation 校验后由用户提交。
- **失败/取消/重复/恢复**：token 失败不阻断前台核心功能；重复 resync 去重；draft 不自动发送；generation/key 不匹配的 ciphertext 删除。
- **审计事件**：client-offline/reattached、push-token-rotated/invalidated。
- **可执行 oracle**：离线期间 authority 零新 Effect；APNs 缺失不改变 Task truth。
- **当前 evidence**：none。

### `CONSOLE-IOS-V1-JRN-011` Secure storage、install marker 或 biometric enrollment 变化

- **入口**：install marker missing/mismatch、Keychain locked/missing、device passcode removed、Face ID/Touch ID 重注册、key invalidated 或 App Attest unsupported/error。
- **前置条件**：旧 device binding 可能仍在 authority。
- **可见步骤**：隔离并丢弃旧 handles → 进入 `device-rebind-required` → 解释受影响能力 → 允许安全只读/revoke → fresh login → 生成新 marker/key/install generation → rebind。
- **Authority 交互**：停止旧 binding 的 lease/R1；revoke old key/generation；server mint 新 generation；登记新 marker digest、public key、app-install identity 与 session。
- **OS surface**：Keychain、LocalAuthentication、Secure Enclave、protected data。
- **成功结果**：新 binding 生效，旧 key/session 不可再用。
- **失败/取消/重复/恢复**：用户取消保持 R1/lease/write blocked；App Attest unsupported/error 按 policy 收窄，不能标成通过；不写明文 key 或提供 passcode fallback。
- **审计事件**：key-invalidated、binding-revoked、rebind-proposed/accepted。
- **可执行 oracle**：marker 缺失/不匹配时残留 key/session 不可继续；enrollment/passcode removal 后 key 不可用；同机 artifact 不跨 generation；任何 fallback 都不能产生有效 R1。
- **当前 evidence**：none。

### `CONSOLE-IOS-V1-JRN-012` 丢失设备、换号、登出、reinstall、restore 与换机

- **入口**：用户从另一受信端 remote revoke，或本机检测身份生命周期事件。
- **前置条件**：可定位旧 device binding；authority 可撤销。
- **可见步骤**：选择设备/账号 → 查看影响 → revoke/登出 → 本机清理 marker/handles；新设备或同机 reinstall/restore 重新 JRN-001。
- **Authority 交互**：revoke session/device/push routing mapping/install generation；保留 Task/Effect/audit；新 app-install 经 fresh login 后由 server mint 新 generation。
- **OS surface**：系统登录/删除 App/backup restore；无本地 authority。
- **成功结果**：旧 generation 后续请求被拒；新设备不迁移 key；同机 restore/reinstall 不因残留 artifact 继承旧授权。
- **失败/取消/重复/恢复**：旧设备离线依赖 expiry；重复 revoke 幂等；取消不改变 binding。
- **审计事件**：device-revoke-requested/effective、session-revoked、new-device-enrolled。
- **可执行 oracle**：新设备 restore 不迁移 key；同机 restore/reinstall 缺 marker 时即使 artifact/session 存在也先隔离且无法跨 generation 复用；旧 routing/session 失效且 revoke 不终止 authority Task。
- **当前 evidence**：none；management session denial vectors not-run。

### `CONSOLE-IOS-V1-JRN-013` 远端 Agent install/upgrade/rollback/uninstall

- **入口**：Agents 中选择 authority catalog/package ref 或已安装 Agent。
- **前置条件**：远端 node online；package evidence current；authority 支持该 lifecycle action。
- **可见步骤**：查看 package/old-new diff → compatibility/sandbox/risk → R0/R1 → 远端进度 → installation/effect 结果。
- **Authority 交互**：固定 package/installation/version；remote node 下载验签执行；authority 写状态并返回 refs。
- **OS surface**：Native list/detail/page；无本地 file/download/WebView。
- **成功结果**：新 authority installation state；失败保留旧版本和未决 Effect。
- **失败/取消/重复/恢复**：R2/R3、证据缺失、floor、node 不可达阻断；unknown 进入 JRN-008；无 bundle bytes 到 iPhone。
- **审计事件**：lifecycle-proposed/authorized、package-verified、installation-transition、rollback-point。
- **可执行 oracle**：网络/file instrumentation 证明 iPhone 未接收 Agent executable；invalid signature 不 commit。
- **当前 evidence**：none；`AGENT-INSTALL-001`、`AGENT-BYPASS-002` not-run。

### `CONSOLE-IOS-V1-JRN-014` 版本低于 security floor 与更新恢复

- **入口**：启动、前台恢复或短期 metadata 刷新发现 build/floor 不符合。
- **前置条件**：signed metadata 可验证或已过期/失败。
- **可见步骤**：显示受影响能力与版本 → protected actions 禁用 → 打开 App Store/managed update → 更新后重新验证 → reauth/resnapshot。
- **Authority 交互**：拒绝低于 floor 的 lease/R1/lifecycle/write；不改变既有 Task/Effect。
- **OS surface**：App Store、TestFlight、ABM/MDM、UIApplication external store link。
- **成功结果**：受允许 build 恢复能力；旧 build 保持被拒。
- **失败/取消/重复/恢复**：审核/传播延迟保留只读/revoke；metadata 无效按 unknown hard floor；不旁载/自更新。
- **审计事件**：build-blocked、update-recovery-opened、build-revalidated。
- **可执行 oracle**：revoked/rollback/expired metadata 均不能 protected write，且不会终止远端任务。
- **当前 evidence**：none。

### `CONSOLE-IOS-V1-JRN-015` 辅助技术完成核心监督与确认

- **入口**：用户启用 VoiceOver、Voice Control、Switch Control、Full Keyboard Access、最大 Dynamic Type、外接键盘或 Reduce Motion。
- **前置条件**：真实支持 iPhone 与测试账号/tenant/node；测试计划覆盖 Touch ID/Face ID、最小/最大屏幕、最旧 admitted/current iOS build、Public/managed、portrait/landscape 和最大 Dynamic Type。
- **可见步骤**：登录绑定 → 选择 scope → 创建 Task → 查看五个独立 authority lifecycle 域和 Runtime projection → R1 → pause → unknown reconcile → Agent lifecycle → revoke/update。
- **Authority 交互**：与标准输入完全相同，不降低 risk 或省略 display fields。
- **OS surface**：VoiceOver、Voice Control、Switch Control、Full Keyboard Access、Dynamic Type、UIAccessibility、hardware keyboard。
- **成功结果**：所有旅程可完成，状态和决定与 touch 路径等价。
- **失败/取消/重复/恢复**：焦点丢失、截断、旋转或 AT 取消均无隐式决定；返回稳定位置。
- **审计事件**：只记录正常 authority actions；不采集用户 disability/AT 使用画像。
- **可执行 oracle**：每个 GA evidence 等价类分别完成适用 AT 矩阵；R1 初始焦点、取消和非默认批准符合要求；不得由单机外推。
- **当前 evidence**：none。

## 17. Page / state matrix

### 17.1 页面清单

| Page ID | 页面 | Tab/入口 | 主任务 | 安全约束 |
|---|---|---|---|---|
| `CONSOLE-IOS-V1-PAGE-001` | Welcome / Sign in | cold launch | 解释角色并登录 | ASWebAuthenticationSession；无内嵌 IdP 表单 |
| `CONSOLE-IOS-V1-PAGE-002` | Device binding | onboarding/rebind | 生成 key、绑定设备 | native；biometry-only；无 passcode fallback |
| `CONSOLE-IOS-V1-PAGE-003` | Tenant / Node picker | scope control / More | 固定当前 scope | authority discoverability；切换清 cache/preview |
| `CONSOLE-IOS-V1-PAGE-004` | Work / Conversation | Work | 对话、目标、draft、preview | Agent 内容无系统 action |
| `CONSOLE-IOS-V1-PAGE-005` | Task center | Tasks | 需要处理/运行/暂停/最近 | stable rows；保持焦点 |
| `CONSOLE-IOS-V1-PAGE-006` | Task detail | Tasks/Work | Flow Thread、五个 authority lifecycle 域、独立 Runtime projection、纠偏/pause | projection only；Runtime stop 不推进 Task/Effect |
| `CONSOLE-IOS-V1-PAGE-007` | R1 confirmation | sheet/full page | CanonicalDisplayEnvelope 显示与签名 | native、device key、非默认批准；不宣称抵御 compromised client |
| `CONSOLE-IOS-V1-PAGE-008` | Effect reconcile | Task/Inbox | `OUTCOME_UNKNOWN` 对账 | 无 Retry；原 binding |
| `CONSOLE-IOS-V1-PAGE-009` | Agent center | Agents | installed/catalog refs | metadata only；无 bundle |
| `CONSOLE-IOS-V1-PAGE-010` | Agent evidence | Agents | package/compatibility/sandbox | unknown 不显示 passed |
| `CONSOLE-IOS-V1-PAGE-011` | Agent lifecycle | Agents | install/upgrade/rollback/uninstall | remote transaction；R0/R1 only |
| `CONSOLE-IOS-V1-PAGE-012` | Inbox | Inbox/notification | R1/input/pause/unknown/degradation | acknowledged 不等于 handled |
| `CONSOLE-IOS-V1-PAGE-013` | Account / Devices | More | 当前账号、binding、device revoke、账户删除 | 单活动账号；Public/Custom 任一支持 account creation 的 target 提供 App 内发起删除，不支持创建则保留证据；删除账号与 device revoke 分离 |
| `CONSOLE-IOS-V1-PAGE-014` | Notifications / Privacy | More | permission、preview、capture、backup | 通用通知；无控制 action |
| `CONSOLE-IOS-V1-PAGE-015` | System / Update | More | readiness、build、floor、恢复 | floor 不改变 authority task |
| `CONSOLE-IOS-V1-PAGE-016` | Diagnostics preview | More/System | 审阅并显式上传 | content-free；字段级预览 |

### 17.2 通用状态

| 状态 | 呈现 | 动作规则 | 恢复 |
|---|---|---|---|
| `initial-loading` | 匹配真实结构的静态 skeleton | 无假控件 | 等待 snapshot |
| `refreshing-last-good` | 保留 last-good + `as_of` | freshness 不足的写禁用 | 新 snapshot |
| `authoritative-empty` | authority 确认空集 | 显示适用创建入口 | 用户动作 |
| `filtered-empty` | 保留 filter | 清除/修改 filter | 本地 |
| `partial` | 显示范围与缺口 | 依赖缺口的写禁用 | 补齐数据 |
| `redacted` | 不泄露标题/正文 | 仅安全返回/请求权限 | reauthorize |
| `stale-offline` | 非实时 Trust Strip | 所有写禁用 | reauth/resnapshot |
| `permission-denied` | 安全原因类别 | 切换账号/scope 或返回 | authority |
| `submitting` | 固定 proposal/ref | 禁重复提交 | 查询原 ref |
| `outcome-unknown` | 原 binding/evidence | reconcile only | authority |
| `conflict/superseded` | 显示变化 | 旧控件失效 | 新 preview |
| `success` | authority ref/next step | 查看对象 | 不抢焦点 |
| `privacy-locked` | 全屏遮蔽 | 无敏感动作 | 解锁后 reauth |
| `reauth-required` | 非敏感框架 | 登录 | 新 session/snapshot |
| `device-rebind-required` | key/binding 原因 | revoke/rebind；R1/lease/write 禁用 | JRN-001/011 |
| `risk-blocked` | R2/R3 或 integrity 原因 | 缩小范围/安全只读 | authority reevaluate |
| `floor-blocked` | build/floor/expiry | update/revoke/diagnostics | signed allowlist |
| `service-error` | 已登记 code 或未知类别 | 安全恢复 | 不显示敏感 raw error |

## 18. Open PoC 与 GA gates

所有 PoC 当前 `execution = not-run`、`evidence = none`。这些 ID 是产品计划，不是 conformance vectors。

| PoC ID | 必须证明 | 当前 |
|---|---|---|
| `IOS-POC-01` | iOS 18+ arm64 Public/managed 两个 bundle 在最旧 admitted 与当前 iOS build 上可签名、安装、升级且数据完全隔离 | not-run / none |
| `IOS-POC-02` | ASWebAuthenticationSession + OIDC/OAuth Code+PKCE、callback/state/nonce/cancel 负例 | not-run / none |
| `IOS-POC-03` | App-container、非 Keychain、backup-excluded marker 缺失/不匹配时隔离旧 handles 并 fresh enroll；P-256 key 使用 `WhenPasscodeSetThisDeviceOnly` + `privateKeyUsage` + `biometryCurrentSet`；新设备不迁移，同机残留 artifact 不跨 generation；App Attest unsupported/error 不当通过 | not-run / none |
| `IOS-POC-04` | CanonicalDisplayEnvelope 一次严格解码、同一 immutable object 渲染、签完整 digest、字段/install generation/nonce/session/expiry/idempotency/build replay 负例；同时证明产品不把结果扩张为 compromised-client trusted display | not-run / none |
| `IOS-POC-05` | biometric enrollment 改变、lockout、cancel、key missing 无 passcode/file fallback | not-run / none |
| `IOS-POC-06` | active/inactive/background/lock/suspend/system-kill/force-quit 下 lease eligibility 与 authority grace | not-run / none |
| `IOS-POC-07` | APNs sandbox/production、topic、token rotation、delay/drop/duplicate/reorder/410 与 opaque handle | not-run / none |
| `IOS-POC-08` | offline、watch gap、idempotency conflict、response loss 和 unknown reconcile 无重复 Effect | not-run / none |
| `IOS-POC-09` | marker 排除 backup 且不进 Keychain；honest restore/reinstall 缺 marker 时旧 handles 隔离；敏感 draft 零持久化；非敏感 draft 使用 install-bound non-migrating key + FileProtection.complete + backup hygiene；generation/key mismatch 时不可解密并删除 | not-run / none |
| `IOS-POC-10` | app-switcher privacy cover、recording mask、screenshot-after-event、pasteboard expiry | not-run / none |
| `IOS-POC-11` | native Markdown、Universal Link、external browser、file importer malicious corpus 与零 native bridge | not-run / none |
| `IOS-POC-12` | Agent lifecycle 只传 refs；iPhone 网络/文件系统中无 executable/package/archive bytes | not-run / none |
| `IOS-POC-13` | Public App Store、TestFlight、Custom App 的 review/demo/account/privacy/distribution 可行性；任一支持 account creation 的 target 可在 App 内发起删除，否则保留不支持创建的证据 | not-run / none |
| `IOS-POC-14` | signed allowlist、short expiry、anti-rollback、review delay、kill switch 和 update recovery | not-run / none |
| `IOS-POC-15` | App Attest/jailbreak/system/MDM signal 仅作风险，unsupported/异常按固定策略收窄 | not-run / none |
| `IOS-POC-16` | VoiceOver、Voice Control、Switch Control、Full Keyboard Access 完成 15 条 journeys 的适用路径，并分别覆盖 Touch ID 与 Face ID | not-run / none |
| `IOS-POC-17` | 最小/最大支持屏幕、最旧 admitted/当前 iOS build、Public/managed、portrait/landscape、最大 Dynamic Type、外接键盘、Reduce Motion 的 GA evidence 等价类 | not-run / none |
| `IOS-POC-18` | privacy manifests、required-reason API、Privacy Label、content-free telemetry、diagnostics preview | not-run / none |

以下均为发布前条件，当前未满足：

- 必须闭合通用 Console backend gate 与适用 M1–M6 合同/实现 gate；
- 必须登记移动 carrier 的 account/device/session/push/lease/R1/floor/revoke 合同；
- 必须真实执行并通过适用既有 vectors；安全负例不得豁免；
- 必须在支持矩阵中的真实 iPhone/build/channel 上通过所有 `IOS-POC-*` 并保留证据；
- 必须分别完成 Public 与 managed 的商店、APNs、Associated Domains、backup、AT、MDM 和升级演练；
- 必须完成两次跨 floor/update/revoke 恢复演练；
- 必须证明零错误完成声明、零跨 account/tenant/channel 泄露、零重复 Effect、零本地 risk 降级。

## 19. iPhone v1 产品要求与追踪

`implementation` 全部为 `not-implemented`。`evidence=not-run` 仅表示存在相关已登记 vector 且当前未执行；移动平台或产品级 oracle 当前均为 `none`。

| ID | 原子要求 | contract | implementation | evidence | owner | oracle | blocked_by |
|---|---|---|---|---|---|---|---|
| `CONSOLE-IOS-V1-PRD-001` | iPhone App 不拥有 authority、IdP、node、daemon、commit 或 completion fact | partial: `REQ-SHELL-CHANNEL-001`, `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Console architecture | 本地 cache/Agent 文本/biometric/push 不能改变 authority 或启用越权写 | backend gate; `IOS-POC-01` |
| `CONSOLE-IOS-V1-PRD-002` | account/tenant/node/channel 的 session、cache、watch、preview 隔离；raw APNs token 仅 App+device+environment/topic，authority mapping 绑定当前 account/device/channel | partial: `REQ-SHELL-CHANNEL-001`, `REQ-CAP-005` | not-implemented | not-run | UNASSIGNED — Identity/Console | cross-scope cache/handle/write 拒绝；tenant/node 不靠独立 token/key 隔离 | mobile binding carrier; `IOS-POC-01`, `IOS-POC-02`, `IOS-POC-07` |
| `CONSOLE-IOS-V1-PRD-003` | Task/Loop/AgentExecution/Effect/Verification 是五个独立 authority lifecycle 域；Runtime 是独立远端 projection | partial: `REQ-RUN-009`, `REQ-EFF-STATE-001` | not-implemented | not-run | UNASSIGNED — Runtime/Console | 任一 authority 域变化不本地推进其他域；Runtime stop 不推进 Task/Effect | lifecycle projection carrier |
| `CONSOLE-IOS-V1-PRD-004` | `COMPLETED` 只来自 current Verification 与 acceptance authority | partial: `REQ-INTENT-ACCEPT-001`, `REQ-RUN-009` | not-implemented | not-run | UNASSIGNED — Task/Console | remote completed/receipt/Agent claim 无法显示 COMPLETED | acceptance carriers |
| `CONSOLE-IOS-V1-PRD-005` | risk floor 由 authority；仅 R0/R1 可执行 | partial: `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Risk/Console | 客户端降 risk、错 scope、missing step-up 全不 dispatch | R1 carrier; `IOS-POC-04` |
| `CONSOLE-IOS-V1-PRD-006` | R2/R3 只解释并阻断，无通知/聊天/passcode/debug 旁路 | product-only + partial: `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Risk UX | 所有 R2/R3 fixture 无批准控件和 Effect | trusted confirmation future |
| `CONSOLE-IOS-V1-PRD-007` | 登录使用系统浏览器 OIDC/OAuth Code+PKCE；App 不验证 IdP 密码；支持 account creation 的 Public/Custom target 可在 App 内发起删除，否则保留无创建证据 | unregistered: mobile auth carrier | not-implemented | none | UNASSIGNED — Identity | callback/state/nonce/PKCE/cancel 负例；account creation/delete 能力与每个 target 证据一致 | auth contract; `IOS-POC-02`, `IOS-POC-13` |
| `CONSOLE-IOS-V1-PRD-008` | device enrollment 使用 App-container、非 Keychain、backup-excluded marker；缺失/不匹配先隔离旧 handles，再由 server fresh mint generation | partial: `REQ-CAP-001`, `REQ-MGMT-SESSION-001`, `REQ-MGMT-SESSION-002`, `REQ-MGMT-SESSION-003` | not-implemented | not-run | UNASSIGNED — Device identity | marker missing/mismatch 即使 key/session 存在也不继续；fresh enrollment 固定 account/bundle/channel/key/session/nonce/expiry | mobile device carrier; `IOS-POC-03` |
| `CONSOLE-IOS-V1-PRD-009` | 单一活动账号；换号/登出/reinstall/restore/换机以 marker + authority-bound app-install identity / install_generation revoke/rebind | partial: `REQ-MGMT-SESSION-002`, `REQ-MGMT-SESSION-003`, `REQ-CAP-005` | not-implemented | not-run | UNASSIGNED — Identity | honest reinstall/restore 缺 marker 时旧 session/routing/generation/cache 不可用；同机残留 artifact 不跨 generation | revoke carrier; `IOS-POC-03`, `IOS-POC-09` |
| `CONSOLE-IOS-V1-PRD-010` | supervision lease 只在 active/unlocked/fresh eligibility 下续 | partial: `REQ-CAP-003`; mobile supervision lease carrier unregistered | not-implemented | not-run（generic）；platform none | UNASSIGNED — Supervision | inactive/background/lock/kill/force-quit 后旧实例零续租 | lease contract; `IOS-POC-06` |
| `CONSOLE-IOS-V1-PRD-011` | grace 仅由 authority 预先固定，BGTask/push/URLSession 不续租 | partial: `REQ-CAP-003`; mobile grace/eligibility carrier unregistered | not-implemented | not-run（generic）；platform none | UNASSIGNED — Supervision | background subsystem 触发不改变 lease expiry | lease contract; `IOS-POC-06` |
| `CONSOLE-IOS-V1-PRD-012` | APNs 只携 opaque hint；raw token 仅 App+device+environment/topic，authority routing mapping 绑定当前 account/device/channel，tenant/node 在 handle audience | unregistered: APNs registration/handle carrier | not-implemented | none | UNASSIGNED — Notification | wrong mapping/topic/account、duplicate/replay/expired handle 拒绝；rotation 不换 device key | `IOS-POC-07` |
| `CONSOLE-IOS-V1-PRD-013` | 通知唯一 action 是打开 App；重认证/resnapshot 后才显示事项 | product-only + unregistered handle resolution | not-implemented | none | UNASSIGNED — Notification/Console | 锁屏 action 不触发任何 write/R1 | `IOS-POC-07` |
| `CONSOLE-IOS-V1-PRD-014` | versioned CanonicalDisplayEnvelope 经一次严格解码，由同一 immutable object 渲染并签完整 digest；若要求 compromised-client trusted display 则 R1 blocked | unregistered: canonical display/device-signature carrier | not-implemented | none | UNASSIGNED — R1 security | 任一 envelope 字段、解析、generation、replay/stale/wrong key 变化拒绝；不宣称证明 compromised-client 用户所见 | R1 contract; `IOS-POC-04` |
| `CONSOLE-IOS-V1-PRD-015` | P-256 key 使用 `WhenPasscodeSetThisDeviceOnly` + `privateKeyUsage` + `biometryCurrentSet`；marker 只作 honest reinstall/restore detection；App Attest 是 risk signal；新设备不迁移、同机 artifact 不跨 generation、无 passcode fallback | product-only + unregistered binding semantics | not-implemented | none | UNASSIGNED — iOS security | marker 缺失隔离旧 handles；enrollment/passcode removal/new-device restore 失效；App Attest unsupported/error 按 policy 阻断 R1/lease而非通过 | `IOS-POC-03`, `IOS-POC-05` |
| `CONSOLE-IOS-V1-PRD-016` | passkey 仅作 upstream login，不替代 device key 或 authority | product-only | not-implemented | none | UNASSIGNED — Identity UX | passkey assertion 不能直接批准 operation | `IOS-POC-02`, `IOS-POC-04` |
| `CONSOLE-IOS-V1-PRD-017` | offline 只保留 last-good/as_of 与 install-bound 加密的非敏感 draft，所有写禁用且不排队 | partial: `REQ-SHELL-WATCH-001`, `REQ-CAP-005` | not-implemented | not-run | UNASSIGNED — Offline/Console | offline fixture 无 dispatch；敏感 draft 不落盘；恢复先 snapshot/generation check | `IOS-POC-08`, `IOS-POC-09` |
| `CONSOLE-IOS-V1-PRD-018` | `OUTCOME_UNKNOWN` 禁换 key/盲重试，只按原 binding reconcile | partial: `REQ-EFF-004`, `REQ-EFF-STATE-001` | not-implemented | not-run | UNASSIGNED — Effect/Console | unknown 下 Retry 不存在，非法 commit 拒绝 | `IOS-POC-08` |
| `CONSOLE-IOS-V1-PRD-019` | 敏感 projection/draft 仅进程内；非敏感 draft 用 install-bound non-migrating key 应用层加密，再配 FileProtection.complete 与 backup hygiene | product-only | not-implemented | none | UNASSIGNED — Privacy/storage | cold launch/backup/restore 无敏感 snapshot；generation/key mismatch ciphertext 不可解密并删除 | `IOS-POC-09` |
| `CONSOLE-IOS-V1-PRD-020` | 内容仅 native escaped text + allowlist Markdown，无 HTML/JS/iframe/bridge/remote auto-load | product-only | not-implemented | none | UNASSIGNED — Content security | malicious content corpus 零脚本、网络、native action | `IOS-POC-11` |
| `CONSOLE-IOS-V1-PRD-021` | Universal Link 仅 opaque handle，规范化后 reauth/resnapshot，不直接写 | product-only + unregistered handle carrier | not-implemented | none | UNASSIGNED — Mobile security | malformed/cross-channel/deletion link 均无动作 | `IOS-POC-11` |
| `CONSOLE-IOS-V1-PRD-022` | file importer 只上传不可信数据，不解释/执行/解压/转发 Agent bundle | product-only | not-implemented | none | UNASSIGNED — File/content security | active content/package/archive 输入被拒或只作受限数据 | `IOS-POC-11`, `IOS-POC-12` |
| `CONSOLE-IOS-V1-PRD-023` | 通知、app switcher、pasteboard 不泄露敏感内容；敏感 draft 不落盘，backup exclusion 只作 hygiene | product-only | not-implemented | none | UNASSIGNED — Privacy | lock/switch/backup/pasteboard fixture 无 secret/R1/body；restore 无匹配 generation/key 时 ciphertext 删除 | `IOS-POC-09`, `IOS-POC-10` |
| `CONSOLE-IOS-V1-PRD-024` | capture 时遮蔽 R1 并提示，但不宣称阻止全部截图 | product-only | not-implemented | none | UNASSIGNED — Privacy UX | recording mask 生效；screenshot-after-event 文案准确 | `IOS-POC-10` |
| `CONSOLE-IOS-V1-PRD-025` | acquisition 只使用 authority catalog/package ref，iPhone 零 Agent executable bytes | partial: `REQ-AGENT-INSTALL-001`, `REQ-AGENT-SANDBOX-001` | not-implemented | not-run | UNASSIGNED — Agent lifecycle | URL/Git/file 无入口，网络/存储无 bundle bytes | `IOS-POC-12`, `IOS-POC-13` |
| `CONSOLE-IOS-V1-PRD-026` | install/upgrade/rollback/uninstall 由远端 node 验证执行并保留旧状态/Effect | partial: `REQ-AGENT-INSTALL-001`, `REQ-AGENT-SANDBOX-001`, `REQ-EFF-004` | not-implemented | not-run | UNASSIGNED — Agent lifecycle | invalid signature/bypass/unknown 不 commit，失败不覆盖旧 installation | M6; `IOS-POC-12` |
| `CONSOLE-IOS-V1-PRD-027` | Public 与 managed bundle/topic/domains/key/cache/binding 隔离且 managed 只收窄 | unregistered: mobile distribution identity | not-implemented | none | UNASSIGNED — Release/Enterprise mobility | cross-channel credential/config 负例拒绝 | `IOS-POC-01`, `IOS-POC-13` |
| `CONSOLE-IOS-V1-PRD-028` | 更新只经 App Store/TestFlight/ABM；App 不下载或执行更新 binary | product-only | not-implemented | none | UNASSIGNED — Distribution | 网络响应不能写/加载 executable 或触发旁载 | `IOS-POC-13`, `IOS-POC-14` |
| `CONSOLE-IOS-V1-PRD-029` | signed short-expiry floor 选择性阻断危险能力，不改变远端任务 | unregistered: build allowlist/floor carrier | not-implemented | none | UNASSIGNED — Release security | revoked/expired/rollback build 无 protected write，Task state 不变 | `IOS-POC-14` |
| `CONSOLE-IOS-V1-PRD-030` | jailbreak/App Attest/system/MDM 信号只作风险，明确异常按固定能力矩阵收窄 | unregistered: integrity risk carrier | not-implemented | none | UNASSIGNED — Integrity | unsupported 不伪装通过；异常不授予/提交，保留恢复 | `IOS-POC-15` |
| `CONSOLE-IOS-V1-PRD-031` | GA 仅 iPhone iOS18+ arm64，滚动24月但不越过 signed floor，并按 biometric/screen/OS/channel/orientation/text 等价类留证 | product-only | not-implemented | none | UNASSIGNED — Release | Touch ID/Face ID、最小/最大屏幕、最旧/current build、Public/managed、portrait/landscape、最大 Dynamic Type 均有独立证据 | `IOS-POC-01`, `IOS-POC-14` |
| `CONSOLE-IOS-V1-PRD-032` | 五 tab + NavigationStack/sheet 提供完整移动 IA，横竖屏等能力 | product-only | not-implemented | none | UNASSIGNED — Product design | 页面/route/state fixtures 无桌面缩放或入口丢失 | `IOS-POC-17` |
| `CONSOLE-IOS-V1-PRD-033` | VoiceOver/Voice Control/Switch Control/Full Keyboard Access/最大字体/外接键盘/Reduce Motion 完成核心 journeys | product-only | not-implemented | none | UNASSIGNED — Accessibility | 各 GA evidence 等价类真机 AT matrix 通过且 R1 强度不降；不得单机外推 | `IOS-POC-16`, `IOS-POC-17` |
| `CONSOLE-IOS-V1-PRD-034` | 第一方 content-free telemetry；无广告/tracking/第三方 analytics | product-only | not-implemented | none | UNASSIGNED — Privacy engineering | egress fixture 无正文/ref/token，ATT 事实与 Label 一致 | `IOS-POC-18` |
| `CONSOLE-IOS-V1-PRD-035` | diagnostics 在字段预览和用户显式确认后上传 | product-only | not-implemented | none | UNASSIGNED — Support/Privacy | cancel 零上传；bundle 与预览字段一致 | `IOS-POC-18` |
| `CONSOLE-IOS-V1-PRD-036` | watch gap/stale/权限变化先 authorized resnapshot，重复事件去重 | partial: `REQ-SHELL-WATCH-001`, `REQ-CAP-005` | not-implemented | not-run | UNASSIGNED — Watch/Console | stale cursor 不 silent resume；revoked cache 不重现 | `IOS-POC-08` |
| `CONSOLE-IOS-V1-PRD-037` | detach/background 不取消 Task、不恢复 privilege；返回需新 session/watch | partial: `REQ-MGMT-SESSION-002`, `REQ-MGMT-SESSION-003`, `REQ-SHELL-WATCH-001` | not-implemented | not-run | UNASSIGNED — Session/Console | detach 后 Task truth 不变，旧 privilege 不恢复 | lease contract; `IOS-POC-06` |
| `CONSOLE-IOS-V1-PRD-038` | specified/implementation/test/Profile 四态在 UI、文档与 release claim 中分离 | product-only | not-implemented | none | UNASSIGNED — Traceability | 无实现/证据时不出现 implemented/pass/conformant | M1 runner; `IOS-POC-18` |

## 20. Apple 官方来源 ledger

以下页面均在 2026-07-20 实际打开；该日期是下表每一行的固定“查询日期”字段，而不是页面发布日期。平台事实不自动成为 CognitiveOS machine contract。

| 页面准确标题 | 完整 URL | 适用版本/页面性质 | 可安全引用事实 | 不得过度推断 |
|---|---|---|---|---|
| Managing your app’s life cycle | https://developer.apple.com/documentation/uikit/managing-your-app-s-life-cycle | 动态；正文区分 iOS 13+ scenes 与 iOS 12- app delegate | scene 从 foreground 到 background/suspended，系统可回收 | 不提供永久后台或 lease 保证 |
| Preparing your UI to run in the background | https://developer.apple.com/documentation/uikit/preparing-your-ui-to-run-in-the-background | 动态 iOS | 后台应尽少工作；系统生成 app-switcher snapshot，应移除敏感内容 | snapshot 遮蔽不等于内存/截图绝对安全 |
| ScenePhase | https://developer.apple.com/documentation/swiftui/scenephase | 动态 SwiftUI；正文未固定 introduced version | active/inactive/background 可观察；background 时应准备终止 | phase 不是 authority 或剩余运行时间 |
| Extending your app’s background execution time | https://developer.apple.com/documentation/uikit/extending-your-app-s-background-execution-time | 动态 iOS | entering background 回调约 5 秒；延长时间有限且会过期 | 不能用作持续 heartbeat |
| applicationWillTerminate(_:) | https://developer.apple.com/documentation/uikit/uiapplicationdelegate/applicationwillterminate(_:) | 动态 iOS | 支持后台执行的 App 一般不能依赖退出时收到此回调 | 不能依赖 goodbye 消息收敛 lease |
| Using background tasks to update your app | https://developer.apple.com/documentation/uikit/using-background-tasks-to-update-your-app | iOS 13+ | AppRefresh 为短刷新、Processing 为较长任务；都有 expiration | 不保证具体启动时间 |
| earliestBeginDate | https://developer.apple.com/documentation/backgroundtasks/bgtaskrequest/earliestbegindate | 动态 BackgroundTasks | 只保证任务不早于该时间，不保证届时启动 | 不能实现精确定时 supervision |
| Performing long-running tasks on iOS and iPadOS | https://developer.apple.com/documentation/backgroundtasks/performing-long-running-tasks-on-ios-and-ipados | 2026 动态页；正文未显示 introduced version | Continuous task 必须前台用户动作启动，可被用户/系统取消；app-switcher 关闭会取消且 App 不获通知 | 不是 daemon 或隐形远程监督通道 |
| Downloading files in the background | https://developer.apple.com/documentation/foundation/downloading-files-in-the-background | 动态 Foundation/iOS | transfer 在独立进程；系统终止 App 后可重启交付；仅 HTTP(S)，file upload | 不支持双向 watch 或 operation commit |
| URL Session Background Task Cancellation Reasons | https://developer.apple.com/documentation/foundation/url-session-background-task-cancellation-reasons | 动态 Foundation | 明确包含 user force-quit、资源不足、后台更新禁用等取消原因 | 不保证 force-quit 后传输继续 |
| Pushing background updates to your App | https://developer.apple.com/documentation/usernotifications/pushing-background-updates-to-your-app | 动态 iOS/APNs | background push 低优先、无保证、可节流/合并/丢弃，处理约 30 秒 | 不能续 lease 或证明 current state |
| Registering your app with APNs | https://developer.apple.com/documentation/usernotifications/registering-your-app-with-apns | 动态 APNs | token 对 App+device 唯一；每次启动注册；restore/reinstall 等可变化 | token 不是稳定 device identity |
| Sending notification requests to APNs | https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns | 动态 APNs HTTP/2 | sandbox/production endpoint、topic、priority；APNs best effort 可乱序/节流/存储一条 | APNs 200 不等于展示、处理或完成 |
| Handling notification responses from APNs | https://developer.apple.com/documentation/usernotifications/handling-notification-responses-from-apns | 动态 APNs | token 必须匹配 environment/topic；410 表示不再活跃 | response 不是 authority evidence |
| Asking permission to use notifications | https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications | 动态 UserNotifications | alert/sound/badge 需授权；用户可随时改设置 | 授权不保证及时呈现 |
| Declaring your actionable notification types | https://developer.apple.com/documentation/usernotifications/declaring-your-actionable-notification-types | 动态 iOS | action 可让 App 后台处理，用户可在锁屏选择 | action 不是 app-specific auth/R1 |
| authenticationRequired | https://developer.apple.com/documentation/usernotifications/unnotificationactionoptions/authenticationrequired | 动态 UserNotifications | 该 option 只要求设备解锁后通知 App | 解锁不等于 CognitiveOS authorization |
| didReceive(_:withContentHandler:) | https://developer.apple.com/documentation/usernotifications/unnotificationserviceextension/didreceive(_:withcontenthandler:) | 动态 NSE | extension 最多约 30 秒；超时显示原始内容 | 不能依赖 extension 补救敏感 payload |
| Change notification settings on iPhone | https://support.apple.com/guide/iphone/change-notification-settings-iph7c3d96bab/ios | 当前默认 iOS 26；页面可切换 iOS 12–26 | preview 可 Always/When Unlocked/Never；可 Scheduled Summary | App 不能控制用户最终呈现 |
| Allow or silence notifications for a Focus on iPhone | https://support.apple.com/guide/iphone/allow-or-silence-notifications-for-a-focus-iph21d43af5b/ios | 当前 iOS 26；页面列 iOS 16–26 | Focus 可允许或静默 App，用户控制 time-sensitive | 不保证突破 Focus |
| ASWebAuthenticationSession | https://developer.apple.com/documentation/authenticationservices/aswebauthenticationsession | 动态 AuthenticationServices | 系统呈现 web auth，并只把 callback 交给调用 session | Apple API 不替代 OIDC state/nonce/PKCE/server validation |
| Authenticating a User Through a Web Service | https://developer.apple.com/documentation/authenticationservices/authenticating-a-user-through-a-web-service | 动态；正文涉及 iOS 13+ strong retention 行为 | 可使用系统 web auth、取消、callback；ephemeral 是请求 | 示例 query token 不是本产品安全合同 |
| Restricting keychain item accessibility | https://developer.apple.com/documentation/security/restricting-keychain-item-accessibility | 动态 Security | WhenUnlocked/AfterFirstUnlock/WhenPasscodeSetThisDeviceOnly 与迁移行为可选 | 单一 class 不能同时满足所有后台/R1需求 |
| Keychain data protection | https://support.apple.com/guide/security/keychain-data-protection-secb0694df1a/web | 动态 Apple Platform Security | keychain secret 受 Secure Enclave round trip；WhenPasscodeSetThisDeviceOnly 不同步/不备份 | Keychain 不等于无泄露或 authority |
| Protecting keys with the Secure Enclave | https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave | iPhone A7+；动态 | Security framework enclave key 为 P-256、须在内生成、不能导入/导出明文 private key | Apple 不定义 CognitiveOS canonical operation |
| Signing and Verifying | https://developer.apple.com/documentation/security/signing-and-verifying | 动态 Security | 可对 data/digest 使用 private key 生成并验证签名 | 签名有效不自动授权业务动作 |
| biometryCurrentSet | https://developer.apple.com/documentation/security/secaccesscontrolcreateflags/biometrycurrentset | 动态 Security | Touch ID 增删或 Face ID 重注册会使 item 失效 | 不自动完成 authority revoke/rebind |
| Logging a User into Your App with Face ID or Touch ID | https://developer.apple.com/documentation/localauthentication/logging-a-user-into-your-app-with-face-id-or-touch-id | 动态 LocalAuthentication | policy 可允许或禁止 passcode fallback；biometry 可取消/失败 | biometric success 不是 operation signature |
| Accessing Keychain Items with Face ID or Touch ID | https://developer.apple.com/documentation/localauthentication/accessing-keychain-items-with-face-id-or-touch-id | 动态 LocalAuthentication | `kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly` 要求设备 passcode，移除 passcode 后 item 不可用；`ThisDeviceOnly` 不进入 iCloud Keychain，设备 backup restore 到新设备时不迁移 | 不据此推断同机 restore/reinstall 一定删除 Keychain artifact，也不证明业务显示完整性 |
| Supporting passkeys | https://developer.apple.com/documentation/authenticationservices/supporting-passkeys | 动态；WKWebView check 正文 iOS 16.4+ | passkey 使用 relying-party account key；需要 webcredentials associated domain | 同步 passkey 不是不可迁移 device R1 key |
| iCloud Keychain security overview | https://support.apple.com/guide/security/icloud-keychain-security-overview-sec1c89c6f3b/web | 动态 Apple Platform Security | passwords/passkeys 端到端加密跨设备同步 | 不能据此声称 passkey device-bound |
| Establishing your app’s integrity | https://developer.apple.com/documentation/devicecheck/establishing-your-app-s-integrity | 动态 App Attest | server challenge、hardware-backed key/assertion；不支持设备需兼容处理；key 不跨 reinstall/migration/restore | App Attest 不证明用户身份、设备完全可信或 operation authorization |
| Supporting associated domains | https://developer.apple.com/documentation/xcode/supporting-associated-domains | 动态；正文 iOS 14+ 通过 Apple CDN 获取 AASA | entitlement+AASA 双向关联；HTTPS、有效证书、无 redirect | 关联不认证具体 link payload |
| Supporting universal links in your app | https://developer.apple.com/documentation/xcode/supporting-universal-links-in-your-app | 动态 iOS/Xcode | Apple 明确要求验证参数并避免 link 直接删除/访问敏感信息 | Universal Link 不是 authorization |
| Preventing Insecure Network Connections | https://developer.apple.com/documentation/security/preventing-insecure-network-connections | 链接 iOS 9+ SDK | ATS 默认要求安全 TLS；部分例外需 App Review 说明；低层 Network 不自动覆盖 | Apple 未要求所有 App pinning |
| WKWebsiteDataStore | https://developer.apple.com/documentation/webkit/wkwebsitedatastore | 动态 WebKit | 默认 store 持久化；nonPersistent 仅内存 | nonPersistent 不消除所有 OS 残留 |
| WKContentWorld | https://developer.apple.com/documentation/webkit/wkcontentworld | 动态 WebKit | 隔离 JavaScript global variables，但 DOM 仍共享 | 不是完整安全边界 |
| Discover WKWebView enhancements | https://developer.apple.com/videos/play/wwdc2020/10188/ | 固定 WWDC20 | App-Bound Domains 限制深层交互；其他域仍可加载 | 仍需目标 iOS 真机复验 |
| WKNavigationDelegate | https://developer.apple.com/documentation/webkit/wknavigationdelegate | 动态 WebKit | delegate 可 allow/cancel navigation/download | navigation policy 不使内容可信 |
| Security of runtime process in iOS, iPadOS, and visionOS | https://support.apple.com/guide/security/security-of-runtime-process-sec15bfe098e/web | 动态 Apple Platform Security | 第三方 App sandboxed；后台只经系统 API | sandbox 不替代业务授权/内容验证 |
| fileImporter(isPresented:allowedContentTypes:allowsMultipleSelection:onCompletion:onCancellation:) | https://developer.apple.com/documentation/swiftui/view/fileimporter(ispresented:allowedcontenttypes:allowsmultipleselection:oncompletion:oncancellation:) | 动态 SwiftUI | 返回 security-scoped URL，需显式开始/结束访问 | URL scope 不校验文件内容 |
| complete | https://developer.apple.com/documentation/foundation/fileprotectiontype/complete | 动态 Foundation | 文件锁屏/启动时不可读写 | Data Protection 不自动排除 backup |
| UIPasteboard.OptionsKey | https://developer.apple.com/documentation/uikit/uipasteboard/optionskey | 动态 UIKit | localOnly 阻止 Handoff；expirationDate 可移除 item | localOnly 不表示仅本 App 可读 |
| isCaptured | https://developer.apple.com/documentation/uikit/uiscreen/iscaptured | 动态 UIKit | 可观察录屏、镜像、AirPlay 并采取遮蔽 | 不代表可阻止所有 capture |
| userDidTakeScreenshotNotification | https://developer.apple.com/documentation/uikit/uiapplication/userdidtakescreenshotnotification | 动态 UIKit | 通知在截图发生后发送 | 不能事前阻止截图 |
| iCloud Backup security | https://support.apple.com/guide/security/icloud-backup-security-sec2c21e7f49/web | 动态 Apple Platform Security | iCloud Backup 包含 app data；标准与 Advanced Data Protection 不同 | 加密不等于数据不进 backup |
| isExcludedFromBackupKey | https://developer.apple.com/documentation/foundation/urlresourcekey/isexcludedfrombackupkey?language=swift | 动态 Foundation | 可排除不必要 app-support/cache；部分保存操作后需重设 | 不应用于伪装应保留的用户文档 |
| App Review Guidelines | https://developer.apple.com/app-store/review/guidelines/ | 动态 living document | 2.5.2 self-contained/no feature-changing code；2.5.4 后台预定用途；4.5.4 push 非必要且不发敏感信息；5.1.1 account deletion | 不保证本产品或 Agent 模型获批 |
| Offering account deletion in your app | https://developer.apple.com/support/offering-account-deletion-in-your-app | 动态 Apple Developer Support | 支持 account creation 的 App 必须让用户在 App 内发起删除；高度监管行业可增加 customer-service flow 来确认和协助删除 | 高度监管行业的附加确认不是通用豁免；不能只提供停用账号或普通客服阻碍删除 |
| TestFlight - Apple Developer | https://developer.apple.com/testflight/ | 动态 TestFlight | 最多 100 internal、10,000 external；首个 external build review | TestFlight 不是 GA 或长期渠道 |
| Get Started - Business - Apple Developer | https://developer.apple.com/business/get-started/ | 动态渠道页 | App Store、Custom、Unlisted、in-house 渠道边界不同 | 渠道可用不等于审核通过 |
| Distribute Custom Apps to Apple devices | https://support.apple.com/guide/deployment/distribute-custom-apps-dep0113f6e18/web | 动态 Deployment | Custom App 仅指定组织可见，每个版本仍 App Review | “通常 1–2 天”不是 SLA |
| Apple Developer Enterprise Program | https://developer.apple.com/programs/enterprise/ | 动态 Program | 仅自有组织员工、特定未被其他渠道满足的场景，含资格要求 | 不可用于客户分发 |
| Distribute proprietary in-house apps to Apple devices | https://support.apple.com/guide/deployment/distribute-proprietary-in-house-apps-depce7cefc4d/web | 动态；正文含 iOS 18+ 手工 trust 行为 | Enterprise profile/证书/MDM/manifest 更新边界 | 不适用于本产品 GA 渠道 |
| App code signing process in iOS, iPadOS, tvOS, visionOS and watchOS | https://support.apple.com/guide/security/app-code-signing-process-sec7c917bf14/web | 动态 Platform Security | executable 必须用 Apple-issued certificate 签名，防 unsigned/self-modifying code | 签名不代表业务内容可信 |
| Privacy manifest files | https://developer.apple.com/documentation/bundleresources/privacy-manifest-files | 动态 Bundle Resources | PrivacyInfo.xcprivacy 记录 collected data、tracking、required-reason API | manifest 不替代 Privacy Label/consent |
| Describing use of required reason API | https://developer.apple.com/documentation/bundleresources/describing-use-of-required-reason-api | 动态；提交门槛自 2024-05-01 | 每个 API 类别必须准确声明 approved reason，不得 fingerprint | 无广告不等于无需声明 |
| Third-party SDK requirements | https://developer.apple.com/support/third-party-SDK-requirements | 动态 SDK 清单 | 列名 SDK 要 manifest，binary dependency 还要签名 | 清单会变化，需逐 build 复核 |
| App Privacy Details - App Store - Apple Developer | https://developer.apple.com/app-store/app-privacy-details/ | 动态 App Store | App/第三方数据实践必须披露；WebView data 有专门规则 | “用于功能”仍可能需声明 |
| App Tracking Transparency | https://developer.apple.com/documentation/apptrackingtransparency | 动态 ATT | 跨其他公司 App/网站 tracking 需 ATT | 第一方安全 telemetry 不自动等于 tracking，但仍需披露 |
| Navigation | https://developer.apple.com/documentation/swiftui/navigation | 动态 SwiftUI | NavigationStack/TabView 提供 stack 与 top-level 切换 | API 行为不是全部 HIG 规范 |
| TabView | https://developer.apple.com/documentation/swiftui/tabview | 动态 SwiftUI | iOS compact 建议限制 tab 使其可容纳 | 不规定 CognitiveOS 固定 tab 数 |
| UI Design Dos and Don’ts | https://developer.apple.com/design/tips/ | 动态 Apple Design | hit target 至少 44×44 pt | 不能替代完整 HIG/AT 测试 |
| Performing accessibility testing for your app | https://developer.apple.com/documentation/accessibility/performing-accessibility-testing-for-your-app | 动态 Accessibility | VoiceOver 需真机；仅靠 VoiceOver 应能完成全部任务并正确读名称/状态 | audit 零问题不保证完全可访问 |
| Testing system accessibility features in your app | https://developer.apple.com/documentation/accessibility/testing-system-accessibility-features-in-your-app | 动态 Accessibility | Accessibility Inspector 可切换 Full Keyboard Access；应使用 external hardware keyboard 验证导航和动作 | 设置切换与键盘局部测试不替代真实 iPhone 上的完整 journey/等价类证据 |
| Get started with Dynamic Type | https://developer.apple.com/videos/play/wwdc2024/10074/ | 固定 WWDC24 | system text styles、reflow、避免 truncation/overlap | 视频不替代目标 build 真机测试 |
| isReduceMotionEnabled | https://developer.apple.com/documentation/uikit/uiaccessibility/isreducemotionenabled | 动态 UIKit | 可读取系统 Reduce Motion 状态 | 仍需产品提供语义等价 |
| supportedInterfaceOrientations | https://developer.apple.com/documentation/uikit/uiviewcontroller/supportedinterfaceorientations | 动态 UIKit | iPhone 默认 allButUpsideDown；最终为 App/view/device 交集 | 未证明 landscape 是平台强制 |
| Adjusting your layout with keyboard layout guide | https://developer.apple.com/documentation/uikit/adjusting-your-layout-with-keyboard-layout-guide | 动态 UIKit | keyboard layout guide 可动态避让键盘 | 不保证自定义布局自动可访问 |
| Handling key presses made on a physical keyboard | https://developer.apple.com/documentation/uikit/handling-key-presses-made-on-a-physical-keyboard | 动态 UIKit | physical key events 经 active responder chain | 不应覆盖系统/AT 保留快捷键 |

### 20.1 HIG 抓取失败、需人工复核

以下动态 Apple HIG 页面于 2026-07-20 实际打开，但抓取只返回 `An unknown error occurred`；本文件未使用搜索摘要补证：

| 页面准确标题 | 完整 URL | 适用 | 当前处理 |
|---|---|---|---|
| Accessibility | https://developer.apple.com/design/human-interface-guidelines/accessibility | 动态 HIG | 发布前交互浏览器人工复核 |
| Navigation and search | https://developer.apple.com/design/human-interface-guidelines/navigation-and-search | 动态 HIG | 发布前人工复核 |
| Tab bars | https://developer.apple.com/design/human-interface-guidelines/tab-bars | 动态 HIG | 发布前人工复核 |
| Motion | https://developer.apple.com/design/human-interface-guidelines/motion | 动态 HIG | 发布前人工复核 |
| Layout | https://developer.apple.com/design/human-interface-guidelines/layout | 动态 HIG | 发布前人工复核 |
| Keyboards | https://developer.apple.com/design/human-interface-guidelines/keyboards | 动态 HIG | 发布前人工复核 |
| Virtual keyboards | https://developer.apple.com/design/human-interface-guidelines/virtual-keyboards | 动态 HIG | 发布前人工复核 |

## 21. 最终状态声明

- iPhone v1 产品方向与 `CONSOLE-IOS-V1-DEC-001..016`：已记录。
- `CONSOLE-IOS-V1-PRD-001..038`：informative 产品要求；不进入 normative registry。
- 移动 account/device/push/lease/R1/floor/revoke carrier：多数 `unregistered / planned / blocked`。
- Console iOS implementation：`not-implemented`。
- iOS platform tests、PoC、APNs、商店、真机、安全、无障碍 evidence：`none`。
- 当前 76 份 conformance vectors：全部 `not-run`。
- CognitiveOS Console iOS Profile：`planned / not implemented`。
- 本文完成不表示 App 已实现、已测试、已获 App Review、已符合 Profile 或可进入 GA。
