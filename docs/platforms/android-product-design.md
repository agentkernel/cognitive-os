# CognitiveOS Console Android phone v1 产品设计

> 类别：informative product design
>
> 产品方向：accepted / planned
>
> 交付状态：`not-implemented / platform tests none / related vectors not-run / Profile not implemented`
>
> 官方资料查询日期：2026-07-20
>
> 支持形态：Android phone only

本文定义 CognitiveOS Console Android phone v1 的产品边界、平台适配、安全模型、关键旅程和发布门禁。它不新增或修改任何 CognitiveOS machine contract，不表示 Android 实现已提供、测试已执行或 Profile 已符合。

本文中的 Android App 始终是**更广但受限的远程 Console**：它服务 Agent operator，但不是 CognitiveOS node、后台 daemon、authority、IdP 或 final arbiter。AgentExecution、Task、Loop、Effect、Verification 是五个独立 authority lifecycle 域；远端 Runtime/process 是与它们分离的 projection。上述生命周期和 Agent lifecycle 的权威状态只来自远端 authority。

## 0. 状态、证据与非规范声明

### 0.1 五种内容口径

- **平台事实**：只来自 §20 已实际打开的一手 Android、AOSP、Firebase、Google Play、Material 页面，查询日期统一为 2026-07-20。
- **产品决策**：使用 `CONSOLE-AND-V1-DEC-*`；属于 informative product direction，不进入 normative registry。
- **产品要求**：使用 `CONSOLE-AND-V1-PRD-*`；只作产品追踪。
- **Machine contract**：只有 `specs/registry/requirements.yaml` 中真实存在的 `REQ-*` 才可引用。Android device enrollment、FCM routing、mobile supervision lease、digest-bound device signature、mobile notification/deep-link carrier 等未登记能力统一写 `unregistered`。
- **实现与证据**：Android Console implementation 为 `not-implemented`；平台 PoC 和真机证据为 `none`；已存在且相关的 conformance vector 只能写 `not-run`。

### 0.2 当前仓库状态快照

- 规范已登记：273 requirements、55 error codes、56 schemas、5 transition tables。
- conformance vectors：76，全部 `not-run`。
- Android Console implementation：`not-implemented`。
- Android emulator/physical-device/Play review/Managed Google Play/a11y/security evidence：`none`。
- Android Console Profile：`planned / not implemented`。

文档存在、产品方向 accepted、schema 存在或 vector 被枚举，都不等于实现、测试或符合性证据。

## 1. 产品角色、用户、任务与非目标

### 1.1 一句话定义

CognitiveOS Console Android phone v1 是面向 Agent operator 的远程监督与受限控制客户端：用户可在手机上选择 tenant/node，继续 Conversation、查看和监督 Task、纠偏、处理安全待办，并对 authority 判定为 R0/R1 的动作执行受治理控制；R2/R3 只显示、解释和阻断。

### 1.2 首要用户与核心任务

首要 persona 是 Agent operator：

1. 登录账号并绑定当前 App install/device key；
2. 在单一活动账号下选择多个 tenant/node；
3. 继续 Conversation，创建、查看和监督 Task；
4. 分离查看 AgentExecution、Task、Loop、Effect、Verification 五个 authority lifecycle 域，以及独立的远端 Runtime projection；
5. 纠偏、请求暂停、恢复和处理 `OUTCOME_UNKNOWN`；
6. 处理 R0/R1，理解 R2/R3 阻断；
7. 请求远端 Agent install、upgrade、rollback、uninstall；
8. 丢失设备、换机、重装、换号或 profile 变化时 revoke/rebind；
9. 在版本、安全补丁或完整性不满足 floor 时进入安全恢复。

### 1.3 明确非目标

- 手机不承载 Agent runtime、CognitiveOS node、authority store、daemon 或最终安全仲裁。
- 远端 Runtime/process stop 只改变该 Runtime projection，不推进 Task、Effect 或其他 authority lifecycle。
- App、FCM、系统通知、BiometricPrompt、Play Integrity 和 Android Keystore attestation 都不能推进 authority 状态。
- 不支持 R2/R3 执行、降级确认、通知批准、聊天批准或“我理解风险”旁路。
- 不支持 tablet、foldable、watch、widget、桌面模式或 DeX；这些均为 `planned/blocked`。
- 不在 App 内下载、解释、执行、缓存或转发 Agent executable bundle。
- 不提供 raw HTML、JavaScript、iframe、WebView bridge、远程图片自动加载或 App 内任意网页浏览。
- 不排队离线控制动作，不把草稿表示成已提交。
- direct APK 不进入 GA。

## 2. GA 支持矩阵、市场与支持期限

### 2.1 发布矩阵

| 维度 | Android phone v1 决策 |
|---|---|
| 市场 | 美国、新加坡 |
| 语言 | `zh-CN`、`en` 同期 GA |
| 形态 | phone only |
| 最低系统 | Android 13 / API 33 |
| target SDK | Android 16 / API 36 |
| CPU ABI | `arm64-v8a` |
| Google 能力 | GMS、官方 Play Store、Play Protect certification required |
| 所有权 | BYOD + Android Enterprise managed |
| Public 渠道 | Public Google Play；internal/closed testing 先行 |
| Enterprise 渠道 | Managed Google Play private app |
| Direct APK | 不进入 GA；仅研究/受控开发测试 |
| 支持期 | GA 起滚动 24 个月，但不越过 OEM/GMS/browser provider/app security floor 或 WebView no-use gate |
| 复核 | 每月复核；signed support metadata 使用短期 expiry |

### 2.2 phone allowlist

以下是 GA 候选列名，不是已测试声明：

| OEM | phone allowlist | 当前证据 |
|---|---|---|
| Google | Pixel 9、Pixel 9a、Pixel 10、Pixel 10a | none |
| Samsung | Galaxy S25、S25+、S25 Ultra、S26、S26+、S26 Ultra | none |

所有 Fold 机型排除。设备列名本身不足以进入支持面；每个组合还必须固定并验证：

- OEM/carrier firmware build；
- Android security patch level；
- GMS/Play Store/Play Protect build 与 certification；
- OIDC/外链实际使用的 browser/Custom Tabs provider package、signing certificate digest 和 version；
- build/runtime 证明未引入、未实例化、未使用 WebView；只有未来另行决策启用 WebView 时才加入 WebView provider floor；
- CognitiveOS app signed build/application ID/channel；
- arm64 ABI、语言、地区、work/personal profile；
- signed support metadata version、issued-at、expiry、authority-held monotonic `floor_epoch`。

Pixel 证据不得外推 Samsung；同一 Samsung 型号的 unlocked firmware 不得外推 carrier firmware。AOSP/CTS 事实也不得替代上述真机矩阵。

### 2.3 支持 floor

支持资格是以下 floor 的交集：

```text
device allowlist
∩ signed app build
∩ Android security patch floor
∩ OEM/carrier firmware floor
∩ GMS/Play Protect floor
∩ browser/Custom Tabs provider package + signer + version floor
∩ WebView not-introduced/not-used build and runtime gate
∩ application/channel allowlist
∩ unexpired signed support metadata
∩ authority-held floor_epoch high-water mark
```

24 个月只是最长产品窗口；任一 security floor 更早失效时，能力立即收窄。recommended minimum 与 mandatory security floor 分开：

- recommended minimum：提醒升级，不自动阻断安全只读；
- security floor：选择性阻断 lease、R1、Agent lifecycle 和 protected writes；
- kill switch：只收窄 Android 客户端能力，不能终止或回滚 authority 已接受的 Task/Effect。
- signed metadata 必须把 monotonic `floor_epoch` 绑定 distribution channel 与 device binding。authority 保存每个 binding 已接受的 high-water mark；旧但签名有效且未过期的低 epoch 仍拒绝，clear-data、reinstall、restore 或本地时钟回退不能重置该值。

## 3. 平台事实、产品决策与合同映射

### 3.1 已登记且可引用的通用要求

| Requirement | 本文只引用的真实语义 | Android 适配边界 |
|---|---|---|
| `REQ-SHELL-CHANNEL-001` | 普通任务与特权管理通道隔离 credential/cache/proposal/approval/audit binding | 不代表 Android profile、OIDC 或 FCM carrier 已登记 |
| `REQ-MGMT-GATE-001` | 每个管理写依次通过 session、binding、scope、risk、version、step-up、approval、expected-version、idempotency gate | 不代表 BiometricPrompt/device signature carrier 已登记 |
| `REQ-MGMT-SESSION-001`, `REQ-MGMT-SESSION-002`, `REQ-MGMT-SESSION-003` | session 完整性、timeout/revocation、重连须新 session/版本与重新认证 | 不代表 Android AuthenticationSession wire contract 已登记 |
| `REQ-CAP-001`, `REQ-CAP-003`, `REQ-CAP-005` | capability 最小绑定、lease/expiry/revocation/fencing、参数/目标/schema/purpose 变化重判 | 不代表 mobile supervision lease 已登记 |
| `REQ-SHELL-WATCH-001` | authorized snapshot + cursor 增量；处理重复、缺口、乱序、过期 | FCM 不是 watch carrier |
| `REQ-EFF-004` | `OUTCOME_UNKNOWN` 不盲重试、不报成功，必须 reconcile/quarantine | Android timeout/process death 不改变该义务 |
| `REQ-EFF-STATE-001` | 拒绝跳过 reconcile/verification 的 commit 和非法 unknown→committed | 客户端不得本地闭合 Effect |
| `REQ-RUN-009` | verifier 对固定后态通过后 Task 才可 completed | 不代表 App 可验收 |
| `REQ-INTENT-ACCEPT-001` | completed 绑定 intent、contract、fixed post-state、verification 与 acceptance authority | FCM/remote completed/biometric success 不是 acceptance |
| `REQ-AGENT-INSTALL-001` | 安装验证 digest、signature/provenance、manifest、adapter/sandbox、compatibility 并由 management authority commit | Android App 只提交 catalog/package ref |
| `REQ-AGENT-SANDBOX-001` | 声称拦截的边界须有负例证据，否则降级 Profile/capability | 负例在远端 node 执行，不在手机执行 Agent |

### 3.2 未登记的 Android carrier

以下均为 `unregistered / planned / blocked`：

- account-first OIDC/PKCE mobile session carrier；
- device enrollment、device-key binding、revoke/rebind carrier；
- FCM registration 与 account/profile/tenant routing carrier；
- Android supervision lease/foreground eligibility carrier；
- versioned `CanonicalDisplayEnvelope` 与 digest-bound R1 Android device signature carrier；
- App Links/notification opaque handle carrier；
- signed mobile support metadata、security floor 与 kill-switch carrier；
- public/managed application identity binding；
- work/personal profile isolation evidence carrier。

## 4. 冻结产品决策摘要

### 4.1 角色与权限上界

- Canonical：[CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001)
- 状态：accepted product direction / delivery blocked。
- Android App 是更广但受限的远程 Console，首要用户为 Agent operator。
- 只执行 authority 判定的 R0/R1；R2/R3 只解释并阻断。
- 手机不是 node、daemon、authority、IdP 或 final arbiter。

### 4.2 支持矩阵与形态

- Canonical：[CONSOLE-AND-V1-DEC-002](./mobile-platform-decision-log.md#console-and-v1-dec-002)
- GA 仅 §2 所列 phone、API 33+、target API 36、`arm64-v8a`、GMS/Play Protect certified。
- tablet/foldable/watch/widget/desktop mode 均 `planned/blocked`。
- 支持资格由短期 signed metadata、实际 browser/Custom Tabs provider floor、WebView no-use gate 和其他全部 floor 共同决定。

### 4.3 市场、设备所有权与 profile

- Canonical：[CONSOLE-AND-V1-DEC-003](./mobile-platform-decision-log.md#console-and-v1-dec-003)
- 首发美国、新加坡；`zh-CN/en`。
- 同时支持 BYOD 与 Android Enterprise managed。
- personal/work profile 是不同 UID、App install、storage、session、FCM registration 和 device binding；禁止跨 profile 复制或恢复。

### 4.4 Account-first 身份与设备绑定

- Canonical：[CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004)
- OIDC/OAuth 2.1 Authorization Code + PKCE 通过 Custom Tabs/system browser；Credential Manager/passkey 可作为 upstream login。
- authority 返回可用 tenant/node；单一活动账号可访问多个 tenant/node。
- device enrollment key generation 必须使用 authority fresh、single-use、短期 nonce 作为 `attestationChallenge`；服务端验证 challenge value、single-use、expiry 和 key attestation 中实际可用的 package、signing certificate digest 与 version。
- `attestationApplicationId` 只记录 key 生成时 app identity，不证明更新后的 current build。app version/build/signing identity 变化后必须重新 attestation/rebind key，或在每次危险操作提供 fresh request-bound Play Integrity app integrity/version 作为附加 build evidence；无法验证 current expected app identity 时 R1/lease/Agent lifecycle 保持 blocked。
- 常规 FCM registration token rotation 只原子更新 notification-routing mapping；只有 FID/install/profile/key identity 变化才触发 device revoke/rebind。
- logout、账号切换、reinstall、restore、换机或 profile 变化必须 revoke/rebind。

### 4.5 信息架构

- Canonical：[CONSOLE-AND-V1-DEC-005](./mobile-platform-decision-log.md#console-and-v1-dec-005)
- 五个一级目的地：Work、Tasks、Agents、Inbox、More。
- compact `NavigationBar`，system/predictive back，edge-to-edge + insets。
- Task detail 分离 AgentExecution、Task、Loop、Effect、Verification 五个 authority lifecycle 域与独立远端 Runtime projection；Runtime/process stop 不推进 Task/Effect。
- 竖屏为主但横屏完整；能力不因布局变化而改变。

### 4.6 生命周期与 supervision lease

- Canonical：[CONSOLE-AND-V1-DEC-006](./mobile-platform-decision-log.md#console-and-v1-dec-006)
- 仅 Activity `RESUMED`、设备已解锁、session/watch/UI fresh 时续租。
- background、lock、process death、Settings Force stop/hibernation、Android 13+ Active apps Task Manager Stop 与 Recents/process kill 都停止客户端续租；这些停止机制的 stopped-state 与后续 resync 能力不同，不得混称。
- WorkManager、FGS、high-priority FCM 只用于 hint/resync，永不续租。
- Settings Force stop/hibernation stopped state 后不宣称任何后台能力；Task Manager Stop 后即使 scheduled jobs/alarms 仍可运行，也只允许 resync。

### 4.7 FCM 与系统通知

- Canonical：[CONSOLE-AND-V1-DEC-007](./mobile-platform-decision-log.md#console-and-v1-dec-007)
- FCM payload 只含最小 opaque handle 和通用显示文案。
- 唯一 action 是打开 App；打开后 reauth、消费/解析 handle、resnapshot。
- message ID、accepted、received、displayed、clicked 都不是 authority truth/evidence。
- PendingIntent immutable/one-shot 仍不替代 nonce/idempotency。

### 4.8 Digest-bound R1

- Canonical：[CONSOLE-AND-V1-DEC-008](./mobile-platform-decision-log.md#console-and-v1-dec-008)
- authority 签发 versioned `CanonicalDisplayEnvelope`；同一 immutable decoded envelope 同时驱动 Native Compose 渲染与完整 envelope digest 签名。
- envelope 固定 account/principal、tenant/node、app/channel、session、action、target/version、parameters、risk、budget、egress、deadline、verification/acceptance、supervision/cancel/reconcile/compensation、nonce/expiry/idempotency、display-profile/app-build。
- BiometricPrompt 仅接受 Class 3 biometric，用于解锁 auth-per-use、non-exportable、hardware-backed Android Keystore signing key；StrongBox optional。
- envelope 的 app-build 必须由 current expected application identity evidence 支撑；旧 key attestation 只证明 key-generation-time identity，不能证明更新后的 current build。不能通过 re-attestation/rebind 或 fresh request-bound Play Integrity 附加证据验证 current build 时，R1 blocked。
- Compose + BiometricPrompt + Keystore 不能证明 compromised client 向用户显示了与签名相同的内容；本产品不宣称抵御 compromised client。若 threat model 要求可信显示，R1 保持 blocked。
- Android Protected Confirmation 或带外确认只能作为未来 PoC/另行产品决策，不能偷改当前范围。
- passkey 只用于 upstream login；无合格 key 或 enrollment/key invalidation 时 R1 blocked 并 rebind，不以 device credential 降级。

### 4.9 Offline、内容与隐私

- Canonical：[CONSOLE-AND-V1-DEC-009](./mobile-platform-decision-log.md#console-and-v1-dec-009)
- 仅在 credential-encrypted storage 持久化非敏感连接 metadata、stable refs、设置和所有用户 draft。
- 默认不注册 direct-boot-aware FCM 或业务 component；device-encrypted storage 最多保存固定、无账号/用户内容且确有 boot 前用途的 build/support metadata。
- 敏感 last-good 只在进程内并标 `as_of`；不排队控制动作；Direct Boot 不放 token、binding 或用户内容。
- native escaped text + allowlist Markdown；无 raw HTML/JS/iframe/remote image auto-load/WebView bridge。
- R1/secret 禁止 copy；Recents snapshot 全遮蔽；敏感数据、FCM/device binding 排除 backup。

### 4.10 Agent lifecycle 与 acquisition

- Canonical：[CONSOLE-AND-V1-DEC-010](./mobile-platform-decision-log.md#console-and-v1-dec-010)
- 用户可远程请求 Agent install、upgrade、rollback、uninstall。
- 输入只能是 authority catalog/package ref；远端 node 下载、验签、检查、sandbox、安装和 commit。
- Android App 永不下载、解释、执行、缓存或转发 Agent executable bundle。

### 4.11 分发 identity

- Canonical：[CONSOLE-AND-V1-DEC-011](./mobile-platform-decision-log.md#console-and-v1-dec-011)
- Public Google Play 与 Managed Google Play private app 使用不同 application ID、FCM project、App Links、device binding 和发布记录。
- 两渠道使用同一代码与能力上限；managed policy 只能收窄。
- direct APK 不进入 GA。

### 4.12 更新、floor 与 kill switch

- Canonical：[CONSOLE-AND-V1-DEC-012](./mobile-platform-decision-log.md#console-and-v1-dec-012)
- Public/managed 均使用 AAB、Play App Signing 和各自 signing identity。
- recommended minimum 与 security floor 分离；monthly review，signed metadata 短期 expiry；authority 维护 channel/device-binding-bound monotonic `floor_epoch` high-water mark，拒绝低 epoch rollback。
- floor 验证实际 browser/Custom Tabs provider package/signer/version，并把 WebView 保持为 build/runtime no-use gate；只有未来启用时才加入 WebView provider floor。
- 低于 floor 时阻断危险客户端能力，保留安全只读、revoke、更新与恢复。
- 客户端 kill switch 不改变 authority state。

### 4.13 恢复与完整性

- Canonical：[CONSOLE-AND-V1-DEC-013](./mobile-platform-decision-log.md#console-and-v1-dec-013)
- root、bootloader、Play Integrity、key attestation 只作风险信号。
- 明确异常可选择性阻断 lease、R1、Agent lifecycle 和 protected writes。
- 必须保留安全只读、device revoke、sign-out 和 update recovery。
- 任何 integrity 服务失败都不得自动解释为攻击成功或安全通过。

### 4.14 Accessibility 与 motion

- Canonical：[CONSOLE-AND-V1-DEC-014](./mobile-platform-decision-log.md#console-and-v1-dec-014)
- TalkBack、Switch Access、Voice Access、Android 14+ 200% font、每台设备最大可用 Display size、high contrast、color correction、外接键盘、横屏和 Remove animations 是 GA gate。
- 触控目标至少 48dp；状态不只靠颜色/动画。
- reduced motion 使用静态 Flow Thread、文本和 live announcement，不降低确认强度。

### 4.15 Telemetry 与 diagnostics

- Canonical：[CONSOLE-AND-V1-DEC-015](./mobile-platform-decision-log.md#console-and-v1-dec-015)
- 只使用最小第一方、content-free telemetry；无广告、tracking 或 third-party analytics。
- crash/diagnostics 默认不上传敏感正文。
- diagnostics 在本地生成字段预览，用户显式确认后上传。

### 4.16 状态与追踪纪律

- Canonical：[CONSOLE-AND-V1-DEC-016](./mobile-platform-decision-log.md#console-and-v1-dec-016)
- 本文始终区分 product decision、registered contract、implementation、evidence、Profile。
- Android carrier 缺失写 `unregistered`，不得借通用 REQ 冒充已登记。
- implementation 统一 `not-implemented`；平台/PoC evidence 统一 `none`；相关既有 vectors 统一 `not-run`；Profile `planned/not implemented`。

## 5. IA、导航、页面、状态与布局

### 5.1 五目的地

| 目的地 | 用户问题 | 主要内容 | 关键约束 |
|---|---|---|---|
| Work | 我现在要做什么？ | Conversation、任务创建、当前重点 | 对话不冒充 Task 状态 |
| Tasks | 正在发生什么？ | Task 列表/详情、纠偏、pause、verification、Runtime projection | 五个 authority lifecycle 域与 Runtime 分离 |
| Agents | 远端节点有哪些能力？ | catalog、installation、upgrade/rollback/uninstall | 只传 package ref |
| Inbox | 哪些事项等我处理？ | R1、输入、unknown、lease/floor/security | acknowledged 不等于 handled |
| More | 账号、tenant/node、系统和恢复 | node switch、security、sessions、notifications、diagnostics、update | safe recovery 常驻 |

### 5.2 页面清单

| ID | 页面 | 主任务 | 主要状态 |
|---|---|---|---|
| `CONSOLE-AND-V1-PAGE-001` | Sign in | Custom Tab OIDC/PKCE、Credential Manager/passkey | signed-out/authorizing/error |
| `CONSOLE-AND-V1-PAGE-002` | Device enrollment | 建立 account/install/device-key/FCM binding | pending/bound/rebind-required |
| `CONSOLE-AND-V1-PAGE-003` | Tenant & node picker | 在单一账号内选择 tenant/node | loading/empty/partial/selected |
| `CONSOLE-AND-V1-PAGE-004` | Work | Conversation、目标输入、重点任务 | ready/offline/reauth |
| `CONSOLE-AND-V1-PAGE-005` | Tasks | 需要处理/运行/暂停/最近结果 | loading/partial/stale |
| `CONSOLE-AND-V1-PAGE-006` | Task detail | 五个独立 authority lifecycle 域、独立 Runtime projection、Flow Thread、pause/correct/reconcile | active/pause-pending/unknown/completed/runtime-stopped |
| `CONSOLE-AND-V1-PAGE-007` | Agents | catalog 与 installations | empty/update/blocked |
| `CONSOLE-AND-V1-PAGE-008` | Agent lifecycle | install/upgrade/rollback/uninstall preview/progress | fresh/submitting/unknown/blocked |
| `CONSOLE-AND-V1-PAGE-009` | Inbox | R1、输入、security/floor/unknown | unread/acknowledged/handled/expired |
| `CONSOLE-AND-V1-PAGE-010` | R1 canonical confirmation | immutable `CanonicalDisplayEnvelope` + Class 3 biometric signature；不宣称抵御 compromised client | fresh/authenticating/submitting/expired/trusted-display-blocked |
| `CONSOLE-AND-V1-PAGE-011` | Effect reconciliation | `OUTCOME_UNKNOWN` 对账 | unknown/reconciling/resolved/quarantined |
| `CONSOLE-AND-V1-PAGE-012` | System & support | app/GMS/patch/browser provider/WebView no-use gate/floor/lease | supported/warning/blocked |
| `CONSOLE-AND-V1-PAGE-013` | Account & security | session、device binding、revoke、profiles | active/revoked/rebind |
| `CONSOLE-AND-V1-PAGE-014` | Diagnostics review | content-free bundle 预览与显式上传 | local/reviewing/uploading |
| `CONSOLE-AND-V1-PAGE-015` | Update recovery | Play update、security floor、恢复 | recommended/required/unavailable |
| `CONSOLE-AND-V1-PAGE-016` | Unsupported/block | R2/R3、device/floor/GMS/profile 阻断 | blocked + safe recovery |

### 5.3 phone layout

- `NavigationBar` 固定五个目的地；不使用 hamburger 作为主导航。
- portrait：单列主画布，详情进入新页面或 bottom sheet；R1/Agent lifecycle/unknown 使用独立页面。
- landscape：仍保留完整操作和内容；可使用 list-detail，但不得引入 tablet/foldable 承诺。
- edge-to-edge；所有关键控件处理 status/navigation/IME/cutout insets。
- system back/predictive back 遵循导航栈；返回 R1 页面等于无决定取消，不提交。
- Task/Agent 列表使用稳定行，不使用同权卡片墙；动态更新不移动当前 TalkBack/keyboard focus。

## 6. 组件、数据流与信任边界

### 6.1 组件

| 组件 | 责任 | 明确不拥有 |
|---|---|---|
| Native Compose shell | 页面、导航、系统卡、R1 envelope display；同一 decoded envelope 驱动显示与签名输入 | trusted display proof、authority、node runtime |
| Custom Tabs/system browser | OIDC/OAuth 2.1 Authorization Code + PKCE | App 内 IdP、R1 |
| Credential Manager | passkey/password/federated upstream login carrier | device binding、R1 signer |
| Android Keystore | non-exportable device signing key、auth-per-use policy | CognitiveOS identity/authority |
| BiometricPrompt | Class 3 本地 user-auth 解锁 key operation | canonical display integrity、最终批准 |
| App-private persistence | 非敏感 metadata/ref/settings/drafts | sensitive last-good、control queue |
| FCM client | registration、opaque hint 接收 | watch、truth、completion |
| Authority API/watch | 五个独立 lifecycle 域、Runtime projection、proposal、Effect、verification、acceptance | Android OS policy |
| Remote node management | Agent package 获取、验证、sandbox、lifecycle Effect | 手机文件/执行环境 |
| Play distribution | App bundle、签名、更新 | CognitiveOS authority decision |

### 6.2 数据流

```text
OIDC/PKCE via Custom Tab
        │
        ▼
authority account/session ── tenant/node projection
        │
        ├── enrollment challenge ── Android Keystore public key/attestation
        │                              │
        │                              └── Class 3 auth-per-use R1 signature
        │
        ├── authorized snapshot + watch ── Native Compose projection
        │
        ├── FCM opaque hint ── open App ── reauth/resnapshot
        │
        └── Agent package ref ── remote node fetch/verify/install
```

### 6.3 信任边界

- FCM、deep link、clipboard、SAF URI、Markdown、diagnostics 和 remote text 都是不可信数据。
- 只有 Native Compose system components 可以渲染真实操作、R1、floor 和 authority status。
- App Links 验证域关联，不验证 operation、account、tenant 或 authorization。
- Android OS account、screen lock、biometric enrollment、profile owner 和 device owner 不等于 CognitiveOS Owner。
- Play Integrity、hardware attestation、root/bootloader signal 进入 server risk policy，不成为本地 allow/deny authority。
- Public 与 managed 安装互不共享 application data、key、FCM registration、App Link identity 或 device binding。

## 7. 账号、device binding、revoke 与恢复

### 7.1 Account-first 登录

1. App 打开 system browser/Custom Tab；
2. authority 完成 Authorization Code + PKCE；
3. App 只接收短期 session carrier；
4. authority 返回当前账号可发现的 tenant/node；
5. 用户选择 tenant/node 后获取 authorized snapshot；
6. 切换 tenant/node 时使旧 preview、watch、draft target 和 capability binding 失效或重判。

Passkey 只证明 upstream account authentication；它可能由 credential provider 跨设备同步，因此不能替代 install-bound device key。

### 7.2 Device enrollment

authority 为每次 device-key generation 签发 fresh、single-use、短期 enrollment nonce。客户端必须把该 nonce 原样设置为 Android Keystore `attestationChallenge`；服务端消费并验证 challenge value、audience/binding、single-use 和 expiry，stale/replayed/unknown challenge 一律拒绝。enrollment challenge 至少在产品层绑定：

- account/principal；
- application ID/distribution channel；
- Android profile/install identity；
- generated device public key 和 key properties；
- session；
- FCM notification-routing mapping 及其 registration token/FID/install identity；
- nonce、issued-at、expiry；
- tenant/node enrollment target；
- attestation/Play Integrity risk signals（若可用）；
- server binding ref/version。

服务端必须从 key attestation 中验证 `attestationApplicationId` 所承载、且在实际 API/attestation 版本可用的 package name、signing certificate digest 和 package version，并将预期 Public/Managed application identity 与 device key、distribution channel 和 binding 原子绑定。重打包、错误 signing certificate、错误 package/version、旧 key + 新 App version、channel mismatch，以及 stale/replayed challenge 必须拒绝 enrollment 或危险操作。

`attestationApplicationId` 是 key-generation-time evidence，不证明 App 更新后的 current build。app version/build/signing identity 变化时必须重新生成并 attestation/rebind device key；替代路径只能是在**每次** R1、lease eligibility、Agent lifecycle 或其他危险操作中，使用 fresh、request-bound Play Integrity `appIntegrity`/app version 作为附加 current-build evidence。Play Integrity `PLAY_RECOGNIZED` 等始终只是 risk signal；`UNEVALUATED`、空 verdict、API error、unavailable、非 fresh 或未绑定当前 request 不得当作通过，也不得替代 attestation application identity 与 authority binding。无法验证 current expected app identity 时，R1、lease 和 Agent lifecycle 保持 blocked。完整 mobile enrollment carrier 尚未登记。绑定成功之前，App 只能显示安全登录/诊断/恢复。

### 7.3 Revoke/rebind 与 current-build evidence 触发

- logout 或账号切换；
- authority/session revoke；
- reinstall、clear data、restore、换机；
- work/personal profile 创建、删除或迁移；
- device key invalid/missing/rotated；
- biometric enrollment change 导致 key invalidation；
- FID/install/profile/key identity 变化；
- distribution channel 变化；
- device lost/stolen、root/bootloader 明确异常；
- support floor 认为 binding 不再可信。

常规 FCM registration token rotation 不触发 device revoke/rebind；客户端与 authority 只在确认 account/profile/install/FID/key identity 未变后，原子替换 notification-routing mapping 并作废旧 token。无法证明 identity continuity 时按 rebind 处理。

app version/build/signing identity 变化使旧 `attestationApplicationId` 只剩 key-generation-time 价值。默认执行 fresh challenge re-attestation/rebind；若保留原 key，则每次 R1、lease、Agent lifecycle/其他危险操作都必须获得 fresh request-bound Play Integrity current-build evidence。任一路径无法闭合 current expected identity 时维持 blocked；`UNEVALUATED`/error 不算闭合。

rebind 不恢复旧 R1 challenge、idempotency key、pending action 或 lease。旧 authority Task/Effect 保留并按 stable ref 对账。

### 7.4 Work/personal profile

- 每个 profile 视为独立安装、UID、storage、Keystore namespace、FCM registration、session 和 device binding。
- 禁止跨 profile content provider、clipboard、backup、App Link continuation 或 shared file 隐式传递 token。
- Managed policy 可以禁用 screenshot、clipboard、diagnostics、tenant/node 或能力，但不能扩大 authority risk ceiling。
- 用户必须在 UI 中看见当前 profile/channel，且该标签不是授权来源。

## 8. Activity 生命周期、通知与 supervision lease

### 8.1 Lease eligibility

只有全部满足时才续租：

- App 当前 Activity 为 `RESUMED` 且可交互；
- 设备已解锁，当前 Android profile active；
- account/session 当前且未 revoke；
- tenant/node/task binding 未变化；
- authorized snapshot + watch cursor fresh，无 gap/stale；
- UI health 可响应，R1/unknown/floor 状态没有禁止续租；
- authority 返回该 task 当前允许续租；
- 当前 signed app/device support metadata 未过期。
- current expected package/signing identity/app build 已由同 build re-attestation/rebind 或本次 fresh request-bound Play Integrity 附加证据闭合；否则不续租。

### 8.2 停止续租

以下任一发生即停止客户端续租：

- `onPause/onStop` 进入后台；
- 锁屏、profile/user 切换；
- process death/crash；
- Settings **Force stop** 或 app hibernation 进入 stopped state；
- Android 13+ **Active apps / Task Manager Stop**；
- Recents swipe；
- network loss、session revoke、watch stale/gap；
- UI hang、support floor 失效；
- 用户显式停止监督。

客户端最后回调不是安全保证。authority 只能使用预先固定、短期、有上限的 grace，并在 expiry 后进入其已登记安全路径；Android App 不本地延长 grace。

### 8.3 Background API 真实语义

- WorkManager：只用于 eventual resync、registration maintenance 或 diagnostics；受 Doze、standby bucket、quota 和 Settings Force stop/hibernation stopped state 影响。
- FGS：只用于平台和 Play policy 允许、用户发起、可见且可终止的短时工作；不用于永久 supervisor；不续租。
- high-priority FCM：只作 time-sensitive hint；可能降级、延迟或丢弃；不续租。
- Settings Force stop/app hibernation：进入 stopped state，jobs、alarms、push 受阻；下次用户显式启动后才重新注册、reauth、resnapshot，不预期后台 resync。
- Android 13+ Active apps Task Manager Stop：系统杀掉整个进程、FGS、activity back stack 和 FGS notification，但 scheduled jobs/alarms 仍按计划执行且没有 stop callback。后续 job/alarm 只能做有限 resync，仍不得续租；下次启动检查 `ApplicationExitInfo.REASON_USER_REQUESTED` 并完整 resnapshot。
- Recents swipe：只表示用户移除 task，不等同 Settings Force stop 或 stopped state；进程是否随后被杀受系统/OEM 影响。无论进程是否存活，Activity 不再 `RESUMED` 即停止续租。
- system/OEM process kill/crash：进程终止但 scheduler 可能按平台规则稍后重启工作；任何后台重启只允许 resync，直到用户重新 foreground resumed 才可能恢复 lease。

### 8.4 FCM 与通知

- payload 不含 Task title、tenant/node alias、stable object ref、credential、R1 secret 或 operation parameters。
- opaque handle 高熵、短期、audience/account/profile/channel-bound、单次消费。
- notification 内容统一为通用提示，例如“有一项 CognitiveOS 事项需要查看”。
- lockscreen 默认 private/secret；用户/OEM 仍有最终控制，故不得放敏感正文。
- 唯一 `PendingIntent` 使用 explicit Activity、immutable、one-shot；服务端仍执行 nonce/expiry/idempotency/authz。
- notification permission/channel 关闭只改变提示可达性，不改变 Inbox、deadline、Task 或 Effect。

## 9. R0/R1、app-owned canonical 确认与风险下界（非硬件可信显示）

### 9.1 R0

- R0 只能由 authority 判定；客户端、Agent、managed policy 或用户不能把 R1+ 降为 R0。
- 即使 R0，也必须有 current session、唯一 fixed target、expected version、idempotency 和 authority response。
- 外部可观察写不得因“来自手机”被视为 R0。
- 响应丢失时进入 query/reconcile，不换 key 重试。

### 9.2 Versioned `CanonicalDisplayEnvelope`

authority 为每个 R1 签发 immutable、versioned `CanonicalDisplayEnvelope`。它是 planned/unregistered mobile carrier，至少固定：

- account/principal；
- tenant/node；
- application ID、distribution channel、app signing/build identity；
- session ref/version/digest；
- action；
- target strong ref + expected/current version；
- canonical parameters 与 parameters digest；
- risk floor；
- budget；
- egress/data destination；
- deadline；
- verification/acceptance conditions；
- supervision、cancel、reconcile、compensation 边界；
- nonce、issued-at、expiry；
- idempotency key/binding；
- display-profile version；
- app build/version。

客户端只解码一次，并把同一个 immutable decoded envelope 同时传给 Native Compose renderer 与完整 envelope digest 计算/签名路径；不从第二份 DTO、localized string、notification payload 或缓存重建签名输入。UI 显示 digest 短码并可展开 envelope 的 canonical technical view。初始焦点在标题/变化摘要，批准不是 Enter 默认动作；Back/Escape/取消均为无决定退出。

这一设计减少 accidental display/signature drift，但不构成 trusted display：Compose、BiometricPrompt 和 Keystore 不能证明 compromised client 向用户显示的内容与被签名 envelope 相同。本文明确不宣称抵御 compromised client。若适用 threat model 要求可信显示或抵御 compromised client，Android R1 保持 blocked；Android Protected Confirmation 或带外确认只允许进入未来 PoC 和独立产品决策，不得被本文隐式纳入当前范围。

### 9.3 Device signature flow

```text
authority versioned CanonicalDisplayEnvelope
    → decode once to one immutable envelope
    → Native Compose display and full-envelope digest from same object
    → user taps explicit action
    → BiometricPrompt Class 3
    → unlock auth-per-use Keystore signing operation
    → signature over complete CanonicalDisplayEnvelope digest
    → authority verifies app identity, key binding, signature, envelope freshness, current state and policy
    → authority decides
```

- key 必须 non-exportable、hardware-backed；StrongBox 仅在设备支持且性能/算法满足时使用。
- 不允许 `DEVICE_CREDENTIAL` fallback；无 Class 3 或无合格 key 时 R1 blocked。
- BiometricPrompt success 只允许签名继续，不是 approval result。
- enrollment change/key invalidation 后旧 key revoke，执行 rebind；不得静默生成新 key继续旧 proposal。
- authority 还必须验证 enrollment 时绑定的 package/signing certificate/version/channel/app-build identity。`attestationApplicationId` 只证明 key 生成时 identity；旧 key + 新 App version/build/signing identity 不能直接证明 current build。
- app version/build/signing identity 变化后，必须先使用 authority fresh `attestationChallenge` 重新生成并 attestation/rebind key；或在每次危险操作验证 fresh、request-bound Play Integrity `appIntegrity`/app version 作为附加 current-build evidence。两条路径都无法验证 current expected identity 时，R1、lease、Agent lifecycle blocked。
- attestation root、certificate、revocation 和 Play Integrity 是 enrollment/risk evidence，不是每次操作的 authority；stale/replayed attestation challenge、`appIntegrity=UNEVALUATED`、非 request-bound verdict 或 API error 不得当作通过。
- 签名完整 envelope digest 不证明 compromised client 的显示完整性；需要 trusted display 时 R1 仍 blocked。
- mobile R1 envelope/signature carrier 当前 `unregistered`。

### 9.4 R2/R3

- 页面显示 risk、缺少的独立 authority/approval 和安全缩小范围选项。
- 无批准按钮、biometric fallback、notification action 或 managed-policy override。
- 用户可返回修改范围，形成 superseding proposal；旧 digest/nonce/challenge 失效。

## 10. Offline、幂等与 `OUTCOME_UNKNOWN`

### 10.1 Offline 数据

允许持久化：

- credential-encrypted storage 中的非敏感 endpoint/tenant/node alias 与 stable refs；
- credential-encrypted storage 中的 language、theme、a11y、notification 等设置；
- credential-encrypted storage 中的非敏感 Conversation/control drafts；
- credential-encrypted storage 中的不含正文 diagnostics metadata；
- device-encrypted storage 最多保存固定、无 account/user content、且确有 boot 前用途的 app build/support metadata。

仅进程内：

- 敏感 last-good projection，必须显示 authority/version/`as_of`；
- 当前 R1 display data；
- sensitive Conversation/Task/Effect detail。

禁止：

- 离线控制队列；
- pending R0/R1 signature；
- credential、refresh token、device binding secret、FCM token backup；
- 通过 UI 把 draft 显示成 accepted/submitted。

默认不声明 direct-boot-aware FCM service、business receiver/service 或其他业务 component；Direct Boot 不读取任何用户 draft。

### 10.2 Reconnect

`network available → reauth if needed → validate app/device/profile binding → authorized snapshot → cursor resume or fresh snapshot → reconcile pending refs → enable eligible writes`

网络恢复、FCM 到达或 WorkManager 完成均不能跳过此序列。

### 10.3 Idempotency 与 unknown

- 每个 control 使用 authority 分配/接受的稳定 proposal/ref、parameter digest 和 idempotency key。
- timeout、process death、Activity recreation、notification duplicate 不生成新 key。
- `OUTCOME_UNKNOWN` 页面保留原 target、digest、dispatch evidence、key、last authority state 和 available reconcile。
- 禁止普通 Retry；`NOT_EXECUTED` 只有 authority 对账确认后才可重新形成新 proposal。
- Effect 未安全闭合时 Task 不显示 `COMPLETED`。

## 11. Storage、内容、deep link、文件与隐私

### 11.1 Storage 与 backup

- token、device private key、FCM registration、device binding、R1 material、sensitive projection 全部排除 cloud backup、D2D 和 cross-platform transfer。
- Android Keystore private key 不导出；restore 后发现 local metadata 与 key/binding 不一致时 revoke/rebind。
- 所有用户 draft 和账号相关 metadata 只进入 credential-encrypted storage。
- 默认无 direct-boot-aware FCM 或业务 component；device-encrypted storage 最多放固定、无账号/用户内容且确有 boot 前用途的 app build/support metadata，不放 draft、token、binding、account content 或 R1 data。
- app-specific cache 退出/登出/revoke 时清理；不承诺 OS、root 或 forensic 层无残留。

### 11.2 内容 renderer

- Agent/authority user-facing content 使用 native escaped text + allowlist Markdown。
- 移除 raw HTML、script、iframe、object/embed、event handler、remote image、CSS、data/file/content/javascript scheme。
- 不创建 App 内 WebView；构建依赖扫描和运行时 instrumentation 必须证明未引入、未实例化、未使用 WebView。发现传递依赖或未来变更引入 WebView 时阻断发布并要求独立产品决策。
- 链接先规范化 scheme/host/IDN，显示目标，再交给 support metadata allowlist 中实际验证过 package、signing certificate digest 和 version 的 system browser/Custom Tabs provider。
- 只有未来明确启用 WebView 时，才增加 WebView provider/package/version security floor；当前版本不以“已安装某 WebView provider”替代 no-use gate。
- 外部浏览器不继承 R1、device key、App session bearer 或 management capability。

### 11.3 App Links 与 intents

- Public/managed 分别使用独立 verified App Links domain/path、application ID 和 signing certificate。
- custom scheme 不承载登录回调、R1 或 protected write。
- 所有 incoming URI/Intent/ClipData 视为不可信，只复制 allowlisted primitive fields。
- exported components 最小化；内部 Activity/Service/Receiver/Provider 显式 `exported=false`。
- App Link 打开后重新验证 account、profile、session、tenant/node、handle expiry 和 authorization。

### 11.4 SAF 文件

- SAF picker 只用于用户显式选择的 diagnostics 或不可信数据上传。
- App 不执行、解压、解释或预览可执行内容；不把 URI 路径或 MIME 当信任证明。
- 上传前显示目标、文件名/类型/大小和数据分类；authority/remote service 执行服务端检查。
- 不使用 broad storage access 或 `MANAGE_EXTERNAL_STORAGE`。

### 11.5 隐私 surfaces

- 通知无敏感标题/正文。
- Recents/app switcher snapshot 全遮蔽；离开前 teardown R1/secret surface。
- R1、secret、token、device binding 禁 copy；其他敏感复制默认不提供。
- R1/sensitive 页面使用 `FLAG_SECURE`、`HIDE_OVERLAY_WINDOWS` 和 obscured/partially-obscured touch checks。
- 这些控制不宣称抵御恶意 AccessibilityService、root、外部摄像、所有 OEM 截图或所有 screen recording。
- diagnostics 先展示字段清单；用户显式确认后上传。

## 12. Agent lifecycle、acquisition 与 Play dynamic-code 边界

### 12.1 Authority-only acquisition

Android 输入面只允许：

- authority catalog item ref；
- authority package/version ref；
- 当前 installation ref；
- lifecycle action 与固定 target version。

App 不接受 URL、Git、custom repository、本地 Agent bundle、archive、APK、DEX、JAR、SO、WASM、Python/Lua/JavaScript Agent code 或 clipboard package ref。

### 12.2 远端节点责任

远端 node/management authority：

1. 解析 catalog/package ref；
2. 下载并固定 source/digest；
3. 验证 signature/provenance/manifest/schema；
4. 检查 dependency、adapter、sandbox、compatibility、security floor；
5. 执行负例与 admission；
6. 形成 AgentInstallation/lifecycle Effect；
7. authority commit；
8. 返回 projection、verification、rollback ref。

Android App 只消费 projection 和提交受治理 proposal，不接触 executable bundle。

### 12.3 Lifecycle 操作

- Android client 的 current expected app identity evidence 必须 fresh；旧 key + 新 App build 且无 re-attestation/rebind 或本次 request-bound Integrity evidence 时，所有 Agent lifecycle 动作 blocked。
- install：显示 publisher、target node、version、capability/profile ceiling、sandbox、compatibility、degradation。
- upgrade：并列 old/new、权限/依赖/任务影响、rollback point。
- rollback：必须高于 floor，解释数据/Effect 不可逆；不回滚 authority history。
- uninstall：处理运行依赖、retention/legal hold 和 pending Effect；不删除历史证据。
- 任一 response unknown 进入 §10.3；不从手机重复发起替代操作。

### 12.4 Google Play policy

- Public/managed Play build 不自更新 executable。
- 不从 Play 外下载 DEX/JAR/SO。
- 解释器例外不允许远程新增未审核或政策违规能力。
- AccessibilityService、FGS、overlay、all-files access 都不用于绕过 power/security policy。

## 13. Distribution、签名、更新、floor 与 kill switch

### 13.1 Channel identity

| 属性 | Public Google Play | Managed Google Play |
|---|---|---|
| application ID | 独立 | 独立 |
| Play listing | public | private organization |
| Play App Signing key | 独立 | 独立 |
| upload key | 独立 | 独立 |
| FCM project/credentials | 独立 | 独立 |
| App Links domain/assetlinks | 独立 | 独立 |
| device binding namespace | 独立 | 独立 |
| telemetry dataset | 逻辑/访问隔离 | 逻辑/访问隔离 |
| capability ceiling | 相同 | 相同或 managed 收窄 |

同一物理 phone 上同时安装两个 channel 时，必须显示两个独立设备记录；不得合并 key、FCM registration、lease、draft 或 session。每个 channel 的 enrollment policy 必须固定 expected package name、app-signing certificate digest、允许的 package version/build 和 device-binding namespace；服务端使用 authority fresh single-use `attestationChallenge`，并从 key attestation 的 `attestationApplicationId`（按实际 API/attestation 版本可用字段）验证这些值后，才把 key 绑定到对应 Public/Managed identity。该 evidence 固定 key 生成时 app identity，不证明后续 current build；Play Integrity `PLAY_RECOGNIZED` 只作 fresh request-bound 附加风险信号。

### 13.2 Release path

`internal testing → closed testing → channel-specific production/private release`

- AAB + Play App Signing；
- app-signing certificate fingerprint 注册到各自 App Links/FCM/API provider；
- pre-launch report 只作补充，不替代真机矩阵；
- rollout 前验证 key upgrade/rotation、rollback、staged rollout 和 update ownership；
- direct APK 只可用于开发/受控研究，不算 GA evidence。

### 13.3 Update 与 floor

- target API 36；发布时重新核实 Play target requirement。
- signed metadata 包含 minimum/recommended/security-floor、channel/application ID、build digest/version、device binding ref/version、device/OEM/carrier、patch、GMS、实际 browser/Custom Tabs provider package/signing certificate digest/version、WebView no-use gate、issued-at、expiry、monotonic `floor_epoch`。
- authority 为每个 distribution channel/device binding 持久化已接受 `floor_epoch` high-water mark；验签和 expiry 通过后仍必须拒绝低于 high-water mark 的 metadata。clear-data、reinstall、restore、本地 storage 删除或时钟回退不能降低 server-held high-water mark。
- metadata 过期时 fail closed 到安全只读/更新恢复，不沿用旧 allow。
- build/runtime WebView no-use gate 失败即阻断发布；只有未来独立决策启用 WebView 后，WebView provider 才进入 provider security floor。
- app version/build/signing identity 变化使 key-generation-time attestation application identity 变旧；发布/更新恢复必须触发 fresh challenge re-attestation/rebind，或为每次危险操作提供 fresh request-bound Play Integrity current-build evidence。无法闭合时 R1、lease、Agent lifecycle 保持 blocked。
- Play 审核或 rollout 延迟时，服务端 kill switch 可禁 lease/R1/Agent lifecycle/protected writes。
- floor/kill switch 不自动 cancel、pause、complete 或 rollback authority Task。
- key rotation 后更新 App Links、FCM/API fingerprints；旧 build 的 recovery 路径必须可解释。

## 14. Material 3、accessibility、motion、rotation 与输入

### 14.1 Visual language

- 继承 CognitiveOS 的 Trust Strip、Governed Flow Thread、用户任务语言和 machine detail。
- 使用 Material 3 native components、system surfaces、dynamic color 可用时的安全配色。
- 不模仿系统警告，不使用 Agent 内容生成真实按钮/icon/status chrome。
- 每屏一个推荐主动作；R1、Agent lifecycle、unknown 使用独立页面。

### 14.2 GA accessibility gate

- TalkBack 可按逻辑顺序到达全部标题、状态、动作、表格/列表项。
- Switch Access/Voice Access 可完成登录后核心旅程，不依赖 swipe/drag/long-press。
- interactive target 至少 48dp；焦点样式和状态不只靠颜色。
- Android 14+ 200% font 与每台支持设备的最大可用 Display size 下不截断关键动词、digest、expiry、risk 或 recovery。
- high contrast 与 color correction 下保留文字、形状、焦点和状态层级；颜色不是唯一信号。
- portrait/landscape 均完整；IME 打开不遮挡字段/动作。
- 外接键盘 Tab/arrow/Enter/Escape 顺序可预测，所有核心操作无触摸可达。
- Remove animations 下无位移、缩放、parallax、shimmer、连续 ambient；使用瞬时更新、静态 Flow Thread 和 live announcement。
- 高速 watch event 聚合播报，不逐条抢焦点。
- Compose loading/indeterminate progress 使用适用的 `progressSemantics`、`ProgressBarRangeInfo.Indeterminate` 和 `stateDescription`；只有重要且低频的状态变化使用适度 `liveRegion`，不使用 Web `aria-*` 术语或逐帧播报。

## 15. Security threat model

当前所有 evidence 均为 `none` 或既有 vector `not-run`。

| ID | 资产 | 攻击者 | 入口 | 信任边界 | 预防 | 检测 | 失败语义 | Owner | Oracle | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| `AND-TM-001` | account/profile/FCM binding | 控制旧 token 或恶意 sender | replay/misrouted FCM | FCM→App→authority | opaque one-shot handle、account/profile/channel binding、无正文 | token/FID change、consume conflict、routing mismatch | 丢弃 hint，reauth/resnapshot，不改 truth | Notification/Identity | 旧/错 profile token 不能打开对象或续租 | none |
| `AND-TM-002` | tenant/node isolation | 合法账号内越权用户 | tenant/node switch、stale deep link | App session→authority | 单一账号、每次 switch 重授权、旧 preview/watch 失效 | authorization denial、epoch/version mismatch | fail closed，保留安全导航 | Identity/Authorization | tenant A handle 在 B 无对象泄露 | none |
| `AND-TM-003` | work/personal profile | 另一 profile 的 App/管理员 | backup、clipboard、App Link、shared file | Android profile UID boundary | 独立 app ID/install/binding/storage，无跨 profile transfer | profile/channel mismatch | revoke/rebind；不恢复旧数据 | Android Enterprise/Identity | 两 profile key/FCM/session 完全不同 | none |
| `AND-TM-004` | login/session | 恶意 App/browser | OIDC redirect/deep link | browser→exported Activity→authority | Custom Tabs、PKCE、verified App Links、state/nonce、no custom scheme | state/issuer/audience/redirect mismatch | 登录失败且不创建 binding | Identity | 重放/错误 app/domain 回调失败 | none |
| `AND-TM-005` | protected action | 恶意 App | nested/implicit Intent、PendingIntent replay | Android IPC→App | explicit immutable one-shot PendingIntent、exported minimum、server nonce/idempotency | duplicate/expired handle、unexpected component | 只打开安全入口，不执行动作 | Android security | 捕获 PendingIntent 不能提交 R0/R1 | none |
| `AND-TM-006` | system UI/content isolation | 恶意 Agent/HTML | Markdown、raw HTML、remote image | untrusted content→Compose system components | escaped text + allowlist Markdown、无 WebView/JS/bridge | sanitizer corpus、network monitor | 降级纯文本，阻断危险内容 | Renderer security | XSS/HTML/JS corpus 无执行/网络/系统控件能力 | none |
| `AND-TM-007` | App data/remote service | 恶意文件提供者 | SAF URI、MIME、provider replacement | DocumentsProvider→App→upload | picker-only、size/type/authority checks、不执行/解压 | URI grant/provider/file identity changes | 上传取消或服务端 quarantine | File ingestion | file/URI/path/archive 负例不执行 | none |
| `AND-TM-008` | canonical R1 与显示完整性 | overlay/钓鱼 App 或 compromised client | 假 biometric prompt、operation/display swap、双 DTO drift | authority envelope→Compose→BiometricPrompt→Keystore | versioned immutable `CanonicalDisplayEnvelope` 同源驱动显示与完整 envelope digest；auth-per-use key；明确不宣称抵御 compromised client | server envelope/app-build/key/session/nonce mismatch；trusted-display requirement | mismatch deny/rebind；若威胁要求 trusted display，则 R1 blocked，未来 Protected Confirmation/带外确认另行决策 | R1 security | accidental swap/replay 被拒；测试不得声称 compromised client 下 user-display 等价已证明 | none |
| `AND-TM-009` | device key 与 current app identity | backup/clone/旧设备持有者或旧 key 搭配新 App build | restore、reinstall、enrollment change、app update | Keystore/install→authority | non-exportable hardware key、authority fresh single-use `attestationChallenge`、no backup、revoke epoch；build/signing identity 变化 re-attest/rebind 或危险操作 fresh request-bound Integrity evidence | challenge replay/expiry、key absent/invalid、attestation app identity/current expected build mismatch | R1/lease/Agent lifecycle blocked，安全 revoke/rebind | Identity/R1 | restore、stale/replayed challenge、旧 key + 新 App version/build/signing identity 均不能沿用旧 binding/signature 执行危险操作 | `context-revocation-cache-reuse.json` not-run |
| `AND-TM-010` | risk policy | rooted/unlocked/modified device | platform compromise | integrity signal→server risk gate | layered Play Integrity/attestation + app policy，不作唯一机制 | verdict/attestation/patch anomalies | 选择性阻断危险写，保留 recovery | Security | signal fail/unknown 不误报可信或任务完成 | none |
| `AND-TM-011` | R1/control UI | malicious accessibility/overlay | AccessibilityService、SAW、tapjacking | other App→window/input | no AccessibilityService use、HIDE_OVERLAY_WINDOWS、obscured touch checks、FLAG_SECURE | occlusion/access-risk signal、touch flags | 取消 R1，要求清除风险后新 proposal | Android security | full/partial overlay 不能触发确认 | none |
| `AND-TM-012` | sensitive projection | screen recorder、肩窥、clipboard reader | screenshot、recording、Recents、copy | window/clipboard/OS surfaces | generic notification、snapshot遮蔽、no-copy、FLAG_SECURE | screen-capture/risk signal（可用时）、privacy audit | teardown/遮蔽；不宣称绝对防截 | Privacy | R1/secret 不进入 Recents/clipboard/notification | none |
| `AND-TM-013` | supervision lease | OS/OEM/process killer 或用户 stop | background、lock、Settings Force stop/hibernation、Active apps Task Manager Stop、Recents swipe、process kill | Activity lifecycle→authority lease | foreground-only renewal、short server grace、所有 stop path 零 renewal；按 stopped-state 区分 resync | missed heartbeat、lifecycle/lock、`ApplicationExitInfo`、fresh snapshot | lease expires；Force stop 无后台 resync 预期；Task Manager Stop/jobs/process kill 仅可 resync、不续租 | Supervision | 每类 stop 都零 renewal，且 jobs/alarms/resync 预期与官方语义一致 | none |
| `AND-TM-014` | Effect idempotency | 重复点击/网络重放 | timeout、process death、offline queue | App→management gate→Effect | fixed key/digest/target/version、no offline queue | duplicate/conflict/current Effect state | `OUTCOME_UNKNOWN` reconcile/quarantine，无 retry | Effect/Console | repeated submit 不产生第二 Effect | `effect-unknown-outcome.json` not-run |
| `AND-TM-015` | authority projection | stale cache/FCM hint | cursor gap、乱序、token delay | watch/FCM→UI | snapshot+cursor、as_of、FCM not watch | stale cursor/gap/dedupe mismatch | resnapshot，写禁用 | Watch/Console | gap 不能静默继续或显示完成 | `shell-watch-resume-006.json` not-run |
| `AND-TM-016` | account/device binding | restore/reinstall attacker | Auto Backup、D2D、cross-device restore | backup transport→new install | exclude token/key/binding，new install ID，mandatory rebind | local key/ref mismatch、new FID | clear restored sensitive refs，rebind | Identity/Privacy | restored device 无法取得旧 R1/lease | none |
| `AND-TM-017` | supported runtime 与 current expected app build | downgrade/old OEM build、旧 key + 新 build 或 metadata rollback | stale Play rollout、app update、carrier firmware、browser provider、旧低 epoch metadata、意外 WebView | device floor/app identity→App/authority gate | signed short-lived metadata、authority-held channel/binding `floor_epoch` high-water mark、browser provider allowlist、WebView no-use gate、version/signing change re-attest/rebind 或 fresh request-bound Integrity build evidence | monthly metadata/floor/browser signer-version checks、epoch rollback、attested key-generation identity vs current build mismatch、Integrity freshness/request binding、build/runtime WebView detection | safe read-only/update recovery；低 epoch或无法验证 current expected app identity 时 R1/lease/Agent lifecycle blocked | Release security | clear-data/reinstall 后低 epoch 仍拒绝；旧 key + 新 App version、unsupported browser、意外 WebView 或 old build 无危险能力 | none |
| `AND-TM-018` | service safety | store review/rollout delay | vulnerable App remains installed | server policy→client capabilities | server kill switch、recommended/security floor split | fleet build/version telemetry without content | 收窄客户端能力，不改 authority Task | Release/Authority | kill switch 不终止已接受 Task | none |
| `AND-TM-019` | user data | third-party SDK/crash collector | analytics、logs、diagnostics | App→telemetry backend | first-party content-free only、no ads/tracking/3P analytics、preview upload | schema/egress review、payload inspection | discard/block upload | Privacy/Supply chain | generated telemetry 无正文/token/ID overcollection | none |
| `AND-TM-020` | device/node supply chain | malicious catalog/package/source | Agent install/upgrade | App ref→authority→remote node | App only sends authority ref；remote digest/signature/sandbox gates | remote verification/compatibility evidence | deny/quarantine，authority unchanged | Agent lifecycle | App never receives executable bytes；invalid signature no commit | `agent-installation-verification.json` not-run |
| `AND-TM-021` | remote acquisition service | SSRF/archive/ambient credential attacker | malicious package metadata | authority catalog→remote acquisition | remote allowlist/SSRF/path/archive/credential budgets；phone no URL/file input | remote acquisition logs/evidence | remote quarantine；手机不回退本地下载 | Agent lifecycle/Security | malicious source cannot cause phone download/execute | `agent-adapter-bypass.json` not-run |
| `AND-TM-022` | authority acceptance | malicious client/remote node | remote completed、receipt、biometric success | projection→acceptance authority | fixed post-state + current Verification + acceptance decision | missing/expired verification、unclosed Effect | 保持 active/candidate/unknown，不完成 | Task/Verification | remote completed 不推进 local Task | `remote-completed-not-acceptance.json` not-run |

## 16. 十六条关键旅程

### `CONSOLE-AND-V1-JRN-001` 安装、登录与 device enrollment

- **入口**：从 Public/Managed Play 安装后首次启动。
- **前置条件**：support metadata 有效；GMS/Play Protect/API/device/profile 在 allowlist；authority/IdP 可达。
- **可见步骤**：语言/隐私摘要 → Custom Tab OIDC/PKCE → 返回 App → 选择创建 device binding → 获取 authority fresh enrollment nonce → 以 nonce 作为 `attestationChallenge` 生成 key → Class 3/Keystore readiness 检查 → enrollment result。
- **Authority 交互**：签发 login state/PKCE session、tenant/node projection 和 fresh single-use/expiring enrollment challenge；验证 challenge、public key、`attestationApplicationId` package/signing digest/version 与附加 risk evidence，返回 binding ref。
- **OS surface**：Play install、Custom Tab/system browser、Credential Manager（可选）、Keystore/BiometricPrompt、notification permission 在后续情境请求。
- **失败/取消/重复/恢复**：取消登录无 binding；redirect/state mismatch fail closed；stale/replayed/unknown challenge、重打包、错证书/版本/channel 拒绝；进程死亡重新开始并废弃旧 challenge。
- **审计事件**：产品计划记录 login/enrollment attempt、channel/profile、result、binding ref；不记录 token/biometric。
- **可执行 oracle**：错误 app/profile/state/nonce/key、stale/replayed challenge 或旧 key + 新 App version/build/signing identity 任一不匹配时无新 binding、R1、lease、Agent lifecycle。
- **当前 evidence**：none；`CONSOLE-AND-V1-POC-003`、`CONSOLE-AND-V1-POC-004`。

### `CONSOLE-AND-V1-JRN-002` 选择 tenant/node 并进入主页

- **入口**：binding 有效的登录完成或从 More 切换。
- **前置条件**：active session；authority 返回可发现 tenant/node。
- **可见步骤**：选择 tenant → 选择 node → 查看 authority/freshness/support strip → 进入 Work。
- **Authority 交互**：每次选择重新授权并获取 snapshot/watch；返回 allowed operations。
- **OS surface**：Native Compose list、system back。
- **失败/取消/重复/恢复**：无可用节点显示 authoritative empty；权限变化返回 picker；重复选择不复用 stale preview。
- **审计事件**：计划记录 active tenant/node binding change 与 authority version。
- **可执行 oracle**：tenant/node 切换后旧 watch/preview/capability 不能提交。
- **当前 evidence**：none。

### `CONSOLE-AND-V1-JRN-003` 创建并监督 Task

- **入口**：Work Conversation 输入目标。
- **前置条件**：session/binding/watch fresh；authority 允许普通 task channel。
- **可见步骤**：输入目标 → clarification/preview → R0/R1 → 返回 stable refs → Task detail 分离显示五个 authority lifecycle 域和独立 Runtime projection。
- **Authority 交互**：固定 UserIntentRecord/interpretation/TaskContract、risk、budget、deadline、Effect/verification；返回 Task/AgentExecution refs 及独立 Runtime projection。
- **OS surface**：Compose input、IME、NavigationBar、predictive back。
- **失败/取消/重复/恢复**：歧义/版本漂移重新 preview；R2/R3 阻断；响应未知按 proposal/idempotency 查询。
- **审计事件**：计划记录 proposal/decision/Task refs，不记录 Conversation 敏感正文到客户端 telemetry。
- **可执行 oracle**：Agent 文本、客户端卡、remote completed 或 Runtime/process stop 均不能推进 Task/Effect 或显示 Task completed。
- **当前 evidence**：`intent-acceptance-007.json`、`remote-completed-not-acceptance.json` not-run；平台 none。

### `CONSOLE-AND-V1-JRN-004` 从通知安全打开事项

- **入口**：通用 FCM notification。
- **前置条件**：notification permission/channel 可用不是必须；opaque handle 未过期。
- **可见步骤**：点击通知 → 打开 App → 解锁/reauth → consume handle → resnapshot → 打开 Inbox item。
- **Authority 交互**：验证 account/profile/channel/audience/expiry，原子消费 handle，解析 stable item ref，再授权读取。
- **OS surface**：FCM、notification channel、explicit immutable one-shot PendingIntent、lockscreen。
- **失败/取消/重复/恢复**：permission disabled 事项仍在 Inbox；重复/错 profile handle 拒绝；离线显示通用恢复，不显示缓存正文。
- **审计事件**：计划记录 handle consumed/expired/mismatch；display/click 不作事项处理证据。
- **可执行 oracle**：notification tap 不直接执行 R0/R1，且 replay 无对象泄露。
- **当前 evidence**：none；`CONSOLE-AND-V1-POC-007`、`CONSOLE-AND-V1-POC-011`。

### `CONSOLE-AND-V1-JRN-005` 执行 R0

- **入口**：fresh proposal 被 authority 判为 R0。
- **前置条件**：session、target/version、policy、idempotency 当前；App foreground。
- **可见步骤**：显示简短变化摘要 → 用户触发适用动作或 authority 允许的自动治理记录 → submitting → authority result。
- **Authority 交互**：完整 management gate、Effect/commit/verification。
- **OS surface**：Native Compose；无 BiometricPrompt。
- **失败/取消/重复/恢复**：取消无决定；response unknown 进入 JRN-008；duplicate 使用原 key 查询。
- **审计事件**：计划记录 proposal、risk=R0、result ref。
- **可执行 oracle**：客户端标 R0 或参数变化不能绕过 authority risk/gate。
- **当前 evidence**：`management-gate-denials.json` not-run；平台 none。

### `CONSOLE-AND-V1-JRN-006` Digest-bound R1

- **入口**：R1 proposal 或 Agent lifecycle R1。
- **前置条件**：foreground/unlocked；fresh versioned `CanonicalDisplayEnvelope`；display-profile/app-build 受支持；Class 3 可用；device key valid/hardware-backed/bound；current expected app identity 已由同 build re-attestation/rebind 或本次 fresh request-bound Play Integrity 附加证据闭合；当前 threat model 不要求抵御 compromised client 的 trusted display。
- **可见步骤**：单次解码 immutable envelope → 同一对象驱动完整 Native Compose confirmation 和完整 envelope digest → 点具体动作 → BiometricPrompt Class 3 → 签名 submitting → authority decision。
- **Authority 交互**：签发 envelope/nonce/expiry；验证 app/channel/current build identity evidence、attestation challenge/binding、account/device/session、完整 envelope digest、idempotency、signature、current state/risk；最终决定。
- **OS surface**：Native Compose、BiometricPrompt、Android Keystore、FLAG_SECURE/HIDE_OVERLAY_WINDOWS。
- **失败/取消/重复/恢复**：Back/取消无决定；device credential 不降级；key invalid/enrollment/app identity change 转 re-attest/rebind；stale/replayed challenge、旧 key + 新 App build、Integrity `UNEVALUATED`/error/非 request-bound 且无其他 current-build evidence 时 blocked；response unknown 进入 JRN-008；若要求 trusted display/compromised-client resistance，则 R1 blocked，Protected Confirmation/带外确认只能进入未来 PoC/决策。
- **审计事件**：计划记录 proposal/signature verification/decision refs；不记录 biometric template 或 private key。
- **可执行 oracle**：双 DTO/display-signature drift、swap parameters/app-build/session/device、stale/replayed nonce/attestation challenge、旧 key + 新 App version、expired challenge、weak/device credential 均零 dispatch；测试结果不得宣称证明 compromised client 向用户显示了签名内容。
- **当前 evidence**：`management-gate-denials.json` not-run；mobile signature carrier none；`CONSOLE-AND-V1-POC-005`。

### `CONSOLE-AND-V1-JRN-007` Candidate complete 到 authority acceptance

- **入口**：Task 显示 `CANDIDATE_COMPLETE`。
- **前置条件**：fixed post-state、Verification lifecycle 和 acceptance authority 可查询。
- **可见步骤**：查看候选结果 → 查看 Verification/Effect → 等待或提交适用 acceptance input → authority completed。
- **Authority 交互**：验证 current Verification、Effect closure、AcceptanceDecision。
- **OS surface**：Task detail、Flow Thread、accessibility live region。
- **失败/取消/重复/恢复**：verification expired/failed、Effect unknown、remote completed 均保持非 completed；重新验证不覆盖旧报告。
- **审计事件**：计划记录 verification/acceptance refs。
- **可执行 oracle**：缺任一 required evidence 时 completed label/action 均不存在。
- **当前 evidence**：`loop-verify-003.json`、`intent-acceptance-007.json`、`effect-state-closure-008.json` not-run。

### `CONSOLE-AND-V1-JRN-008` 处理 `OUTCOME_UNKNOWN`

- **入口**：Effect dispatch 后断线/timeout/response loss。
- **前置条件**：原 Effect/proposal/idempotency binding 可查询。
- **可见步骤**：进入安全分支 → 查看原目标/dispatch evidence → 发起/观察 reconcile → 收敛 executed/not-executed/still-unknown。
- **Authority 交互**：按原 key/ref 查询 executor/receipt/world state，执行 verification/quarantine。
- **OS surface**：独立 reconciliation page；无 Retry 主按钮。
- **失败/取消/重复/恢复**：关闭页面不清除 unknown；重复 reconcile 去重；仍未知进入 quarantine/human authority。
- **审计事件**：计划记录 reconcile attempt/result 与原 Effect ref。
- **可执行 oracle**：换 key、停止 App、重新安装 Agent 都不能使原 Effect 成功/失败。
- **当前 evidence**：`effect-unknown-outcome.json`、`effect-idempotency-conflict.json` not-run。

### `CONSOLE-AND-V1-JRN-009` 后台、锁屏、用户停止、process death 与恢复

- **入口**：Home、锁屏、切 App、旋转重建、Settings Force stop/hibernation、Android 13+ Active apps Task Manager Stop、Recents swipe、系统/OEM process kill 或 crash。
- **前置条件**：可能存在 active supervision lease。
- **可见步骤**：离开 `RESUMED` 即停止 renewal/遮蔽敏感 surface；恢复时识别可用 exit reason 与 stop 类别 → reauth → resnapshot/watch → 显示实际 lease/Task 状态。
- **Authority 交互**：短 grace/expiry、安全 checkpoint 由 authority；客户端恢复只查询。Settings Force stop/hibernation 不预期后台 resync；Task Manager Stop、scheduled job/alarm 或普通 process restart 只允许 resync。
- **OS surface**：Activity/Compose lifecycle、saved state、`ApplicationExitInfo`、Active apps Task Manager、Settings Force stop、Recents、lock state。
- **失败/取消/重复/恢复**：没有 stop/lifecycle callback 也由 server expiry 收敛；saved state 只恢复轻量 UI/ref；Recents swipe 不假定 stopped state；任何 scheduled job/后台重启都不续租。
- **审计事件**：计划记录 lease renewal stopped、可用 exit reason、stop category、resume reconciliation，不把本地 clock 当 authority。
- **可执行 oracle**：所有 stop/background/lock/death 路径均零 renewal；Settings Force stop 阻断 job/alarm/push 的预期与 Task Manager Stop 保留 scheduled job/alarm 的预期分别验证；恢复前无 protected write。
- **当前 evidence**：none；`CONSOLE-AND-V1-POC-006`、`CONSOLE-AND-V1-POC-008`、`CONSOLE-AND-V1-POC-009`。

### `CONSOLE-AND-V1-JRN-010` Offline、弱网、Doze 与 FCM routing 变化

- **入口**：网络丢失、Doze、FCM stale/expired/rotated。
- **前置条件**：可能有进程内 last-good 和非敏感 draft。
- **可见步骤**：显示 `as_of`/offline → 禁写 → 保存 credential-encrypted draft（非队列）→ 网络恢复 → 分类 registration token rotation 与 FID/install/profile/key identity change → 更新 routing 或 rebind → reauth/resnapshot。
- **Authority 交互**：常规 token rotation 在 identity continuity 成立时原子替换 authority notification-routing mapping 并作废旧 token；FID/install/profile/key identity 变化执行 revoke/rebind；随后查询 refs、恢复 watch。
- **OS surface**：Connectivity、Doze、WorkManager、FCM callbacks。
- **失败/取消/重复/恢复**：WorkManager 延迟不显示已同步；old token 作废；无法证明 identity continuity 则不只更新 token而转 rebind；onDeletedMessages/full sync 不推断漏失事项内容。
- **审计事件**：计划记录 registration version、sync result，不记录 token 值到 telemetry。
- **可执行 oracle**：普通 token rotation 不 revoke device binding；FID/install/profile/key 变化不能只换 routing；offline draft 与 old token 永不产生 Effect。
- **当前 evidence**：none；`CONSOLE-AND-V1-POC-007`、`CONSOLE-AND-V1-POC-009`。

### `CONSOLE-AND-V1-JRN-011` Key invalidation 与 secure-storage 降级

- **入口**：biometric enrollment change、key invalid/missing、screen lock 不合格、restore。
- **前置条件**：原 device binding 存在。
- **可见步骤**：显示 R1/binding blocked → 保留安全只读/revoke → reauth → revoke old key → 新 enrollment。
- **Authority 交互**：检查 old binding/revocation epoch，签发新 challenge，建立新 binding。
- **OS surface**：Keystore、BiometricPrompt、device lock settings。
- **失败/取消/重复/恢复**：不降级 device credential/software key；取消 rebind 保持 R1/lease blocked。
- **审计事件**：计划记录 key invalid/revoke/rebind refs，不记录 private key。
- **可执行 oracle**：旧 key 或新未绑定 key 的 signature 全拒绝。
- **当前 evidence**：`context-revocation-cache-reuse.json` not-run；平台 none。

### `CONSOLE-AND-V1-JRN-012` 丢失设备、登出、换机、重装与 profile 变化

- **入口**：More→Revoke device，或另一可信端发起 remote revoke。
- **前置条件**：用户可通过 upstream account 恢复；authority 可定位 binding。
- **可见步骤**：确认 revoke 范围 → authority revoke → 本地清 session/ref/draft → 新设备重新 JRN-001。
- **Authority 交互**：推进 session/key/binding revocation epoch，停止未来 lease/R1。
- **OS surface**：Account settings、app data clear/uninstall/managed profile controls。
- **失败/取消/重复/恢复**：remote revoke 即使旧设备离线也在服务端生效；重复 revoke 幂等；restore 不恢复 binding。
- **审计事件**：计划记录 revoke initiator/target/result。
- **可执行 oracle**：被 revoke 设备即使仍有 token/key/cache也不能续租或提交。
- **当前 evidence**：`management-session-denials.json`、`context-revocation-cache-reuse.json` not-run；平台 none。

### `CONSOLE-AND-V1-JRN-013` 远端 Agent install

- **入口**：Agents→authority catalog/package detail→Install。
- **前置条件**：target node、package/version fixed；remote evidence 可用；risk ≤ R1。
- **可见步骤**：查看 publisher/signature/provenance/compatibility/sandbox → R0/R1 → 观察 remote lifecycle → installation ref。
- **Authority 交互**：catalog resolve、remote acquisition、verification、management gate、AgentInstallation commit。
- **OS surface**：Native Compose；无 SAF/WebView/download。
- **失败/取消/重复/恢复**：invalid signature/gate/R2+ 阻断；App 不 fallback 下载；response unknown 进入 JRN-008。
- **审计事件**：计划记录 package/target/evidence/proposal/installation refs。
- **可执行 oracle**：手机网络中无 Agent bundle bytes；invalid signature authority unchanged。
- **当前 evidence**：`agent-installation-verification.json`、`agent-adapter-bypass.json` not-run；平台 none。

### `CONSOLE-AND-V1-JRN-014` Agent upgrade、rollback、uninstall

- **入口**：Installed Agent detail。
- **前置条件**：current installation、target package/rollback ref、dependent Task/Effect fixed。
- **可见步骤**：old/new diff 或 uninstall impact → R0/R1 → remote transaction → verify new installation/removal。
- **Authority 交互**：expected-version、floor、dependency、pending Effect、rollback/retention gates。
- **OS surface**：Native Compose lifecycle page、BiometricPrompt for R1。
- **失败/取消/重复/恢复**：floor 下 rollback、pending unknown、R2/R3、stale version 阻断；失败保留旧 installation。
- **审计事件**：计划记录 lifecycle action/proposal/Effect/installation/rollback refs。
- **可执行 oracle**：失败不覆盖旧版本；uninstall 不删除未决 Effect/历史证据。
- **当前 evidence**：`agent-installation-verification.json`、`effect-unknown-outcome.json` not-run；平台 none。

### `CONSOLE-AND-V1-JRN-015` Security floor、更新恢复与 accessible control

- **入口**：启动/周期 floor 检查失败，或用户从 System 打开 Update recovery。
- **前置条件**：signed support metadata/floor result；Play update path 可查询。
- **可见步骤**：读取阻断原因 → 在 TalkBack/keyboard、Android 14+ 200% font、每设备最大 Display size、high contrast/color correction、portrait/landscape、Remove animations 下选择 Update、Revoke、Sign out 或 safe read-only → Play 更新 → re-evaluate/re-attest/rebind。
- **Authority 交互**：server kill switch/capability narrowing、device revoke；不修改 Task/Effect。
- **OS surface**：Play Store/Managed Play、Material 3、TalkBack/Switch/Voice Access、system browser。
- **失败/取消/重复/恢复**：商店审核/rollout 不可用时保留 safe recovery；metadata 过期不继续危险写；更新后重新验证全部 floor。
- **审计事件**：计划记录 floor decision、client build、recovery choice/result，不记录敏感内容。
- **可执行 oracle**：全部辅助输入可完成 recovery；低于 floor 无 lease/R1/Agent lifecycle，但 authority Task 不被客户端终止。
- **当前 evidence**：none；`CONSOLE-AND-V1-POC-016`、`CONSOLE-AND-V1-POC-017`、`CONSOLE-AND-V1-POC-018`。

### `CONSOLE-AND-V1-JRN-016` 辅助技术完成核心受治理旅程

- **入口**：用户启用 TalkBack、Switch Access、Voice Access、外接键盘、Android 14+ 200% font、设备最大 Display size、high contrast、color correction 或 Remove animations 后启动 App。
- **前置条件**：支持矩阵内设备/build；测试账号、tenant/node、R0/R1、unknown、Agent lifecycle、revoke/update fixtures 可用；当前 evidence 仍为 none。
- **可见步骤**：无触摸完成登录 → tenant/node 选择 → Task 创建/监督 → R1 阅读与确认/取消 → pause → reconcile → Agent install/upgrade/rollback/uninstall → device revoke → floor/update recovery。
- **Authority 交互**：使用与默认 UI 完全相同的 session、proposal、risk、R1 envelope、Effect、Verification、acceptance 和 revoke/update gate；辅助技术路径不降低风险或跳过字段。
- **OS surface**：TalkBack、Switch Access、Voice Access、hardware keyboard、Compose semantics、system browser/Custom Tabs、BiometricPrompt、Play update surface。
- **失败/取消/重复/恢复**：focus/announcement/layout/motion 问题不得改为弱确认或隐藏字段；取消保持无决定；进程/旋转恢复后焦点落到页面标题或明确恢复点，不重复提交。
- **审计事件**：只记录正常 authority action/result 与必要 correlation ref；不记录辅助技术配置、disability profile、spoken content、biometric 或敏感正文。
- **可执行 oracle**：上述九类核心旅程在 portrait/landscape、Android 14+ 200% font、每设备最大 Display size、high contrast/color correction、Remove animations 和各辅助输入下可完成；loading 使用 `progressSemantics`/`ProgressBarRangeInfo.Indeterminate`/`stateDescription` 与适度 `liveRegion`，无 `aria-*`。
- **当前 evidence**：none；`CONSOLE-AND-V1-POC-017`。

## 17. Page/state matrix

| 状态 | 页面呈现 | 动作规则 | 恢复与无障碍 |
|---|---|---|---|
| `initial-loading` | 匹配真实布局的静态 skeleton 或 indeterminate progress | 无可点击假控件 | Compose `progressSemantics`、`ProgressBarRangeInfo.Indeterminate`、`stateDescription`；仅重要变化适度 `liveRegion`，不使用 `aria-*` |
| `refreshing-last-good` | 保留进程内 last-good + `as_of` | 依 freshness 决定只读；protected write 默认禁用 | 不移动焦点 |
| `authoritative-empty` | authority 返回空集 | 提供适用创建/切换动作 | 标题解释空原因 |
| `filtered-empty` | 保留 filter | 清除/修改筛选 | 不误称 authority 无数据 |
| `partial` | 显示已加载范围与缺口 | 依赖缺口的动作禁用 | 缺口可读 |
| `redacted` | 通用占位 | 重新授权/切账号 | 不泄露对象存在/标题 |
| `stale-offline` | 明显非实时 Trust Strip | 无 protected write/lease renewal | draft 明确未提交 |
| `permission-denied` | 安全原因类别 | 返回/切账号/profile | 错误不泄露对象 |
| `submitting` | 固定 proposal/ref | 防重复点击 | live announcement |
| `result-unknown` | 独立安全分支 | 只 query/reconcile；无 Retry | 焦点进入解释 |
| `conflict/superseded` | old/new diff | 获取新 preview；旧控件失效 | 宣告内容已变化 |
| `candidate-complete` | 显示候选与缺失 acceptance | 查看 Verification/等待 | 不使用完成图标/文案 |
| `completed` | authority acceptance + current verification | 查看证据 | 客户端不自行生成 |
| `privacy-locked` | 敏感 surface teardown/遮蔽 | 解锁后 reauth | 通知/Recents 无正文 |
| `reauth-required` | 非敏感 frame | OIDC/PKCE 后 resnapshot | 不恢复旧 projection 为 current |
| `R2/R3-blocked` | risk 和缺少 authority | 仅修改范围/返回 | 无隐藏旁路 |
| `floor-blocked` | build/patch/GMS/browser provider/app floor、floor epoch 或 WebView no-use gate | safe read-only/revoke/update/sign-out | recovery 全输入模式可达 |
| `service-error` | 已登记 code（若存在）或通用未登记错误 | 安全恢复 | 不显示 raw sensitive error |

## 18. Open PoC 与 GA gates

### 18.1 Open PoC

| ID | 必须证明 | 当前 evidence |
|---|---|---|
| `CONSOLE-AND-V1-POC-001` | API 33–36、arm64、GMS/Play Protect support detection 与 fail-closed | none |
| `CONSOLE-AND-V1-POC-002` | personal/work profile 独立 UID/install/storage/FCM/key/binding | none |
| `CONSOLE-AND-V1-POC-003` | OIDC/OAuth 2.1 Authorization Code + PKCE Custom Tab 与 verified App Link | none |
| `CONSOLE-AND-V1-POC-004` | Keystore hardware-backed key、authority fresh single-use/expiring `attestationChallenge`、attestation root/CRL、`attestationApplicationId` package/signing digest/version、stale/replayed challenge、旧 key + 新 App version、repack/wrong-cert/wrong-version negatives、enrollment invalidation、rebind | none |
| `CONSOLE-AND-V1-POC-005` | versioned immutable `CanonicalDisplayEnvelope` 单源显示/完整 digest 签名、Class 3 auth-per-use、current app-build re-attestation 或 fresh request-bound Integrity evidence、stale challenge/旧 key+新 build/replay/swap/双 DTO negatives，并证明测试不宣称抵御 compromised client | none |
| `CONSOLE-AND-V1-POC-006` | foreground-only lease across background/lock/death/crash/rotation/UI hang | none |
| `CONSOLE-AND-V1-POC-007` | FCM delay/drop/duplicate/token rotation/onDeletedMessages/permission/channel | none |
| `CONSOLE-AND-V1-POC-008` | Settings Force stop/app hibernation stopped state 下 job/alarm/push 受阻且 lease 零 renewal | none |
| `CONSOLE-AND-V1-POC-009` | Active apps Task Manager Stop 保留 scheduled jobs/alarms、Recents swipe/process kill、WorkManager/FGS/Doze/standby/OEM restrictions 只 resync 不续租 | none |
| `CONSOLE-AND-V1-POC-010` | backup/D2D/restore/reinstall/clear-data 不恢复 key/token/binding | none |
| `CONSOLE-AND-V1-POC-011` | App Links/Intent/PendingIntent/notification handle 跨 app/profile/replay 负例 | none |
| `CONSOLE-AND-V1-POC-012` | allowlist Markdown、build/runtime WebView no-use gate、实际 browser/Custom Tabs provider package/signer/version、无 remote image、SAF upload | none |
| `CONSOLE-AND-V1-POC-013` | FLAG_SECURE/HIDE_OVERLAY_WINDOWS/occlusion/Recents/clipboard 降级与限制文案 | none |
| `CONSOLE-AND-V1-POC-014` | 远端 Agent install/upgrade/rollback/uninstall，手机无 executable bytes | none |
| `CONSOLE-AND-V1-POC-015` | Public/Managed 独立 app/FCM/App Links/binding/signing identity；fresh challenge enrollment 验证 expected package/signing digest/version，拒绝 stale/replay/repack/wrong-cert/wrong-version/旧 key+新 build | none |
| `CONSOLE-AND-V1-POC-016` | target API 36、Play App Signing、app update 后 re-attestation/rebind 或危险操作 fresh request-bound current-build Integrity evidence、key rotation、channel/device-binding-bound monotonic floor epoch、browser provider floor、WebView no-use gate、kill switch/update recovery | none |
| `CONSOLE-AND-V1-POC-017` | TalkBack/Switch/Voice Access/keyboard、Android 14+ 200% font、每设备最大 Display size、high contrast/color correction、rotation/Remove animations、Compose progress semantics | none |
| `CONSOLE-AND-V1-POC-018` | Pixel/Samsung 每型号、carrier build、patch、GMS、browser/Custom Tabs provider package/signer/version、WebView no-use、app build 月度矩阵 | none |

### 18.2 GA gates

- 以下全部是发布前条件，当前均未满足。
- Console 通用 backend/contract gate 必须满足；
- mobile enrollment、lease、FCM routing、R1 signature、support-floor carrier 必须登记；
- `CONSOLE-AND-V1-POC-001..018` 必须全部有可定位 pass evidence；
- 相关既有 conformance vectors 必须真实执行且无 release-blocking fail；
- Public 与 Managed 两渠道分别完成 Play review、signing、App Links、FCM、binding 和 rollout 演练；
- 美国/新加坡、`zh-CN/en`、BYOD/managed、personal/work profile 均有覆盖；
- 列名设备及每个 carrier firmware/build 独立通过，且 metadata 未过期；
- 两次 app/GMS/browser provider/OEM security-floor 与 WebView no-use gate 升级演练，以及一次 key rotation/revoke/rebind/floor-epoch anti-rollback 演练；
- Settings Force stop/hibernation、Active apps Task Manager Stop、Recents/process kill、Doze、notification denied、FCM delayed/dropped 不产生错误 supervision/完成声明，且各 stop path 的 resync 预期必须分别验证；
- accessibility 核心旅程必须全部通过；
- direct APK、tablet/foldable/watch/widget 未被商店文案或 UI 暗示支持。

## 19. Android phone v1 PRD traceability

`Implementation` 全部为 `not-implemented`。`Evidence` 中 `not-run` 只引用仓库已有 vector；`none` 表示没有平台/PoC 执行结果。

| ID | 原子要求 | Contract | Implementation | Evidence | Owner | Oracle | blocked_by |
|---|---|---|---|---|---|---|---|
| `CONSOLE-AND-V1-PRD-001` | Android App 非 node/daemon/authority/IdP/final arbiter | partial: `REQ-SHELL-CHANNEL-001`, `REQ-MGMT-GATE-001` | not-implemented | none | UNASSIGNED — Android/Console security | local cache/FCM/biometric 不能写 authority state | mobile architecture carrier unregistered |
| `CONSOLE-AND-V1-PRD-002` | AgentExecution/Task/Loop/Effect/Verification 五个 authority lifecycle 域独立显示；Runtime 是独立远端 projection，完成只来自 authority | partial: `REQ-RUN-009`, `REQ-EFF-STATE-001`, `REQ-INTENT-ACCEPT-001` | not-implemented | `loop-verify-003.json`, `effect-state-closure-008.json`, `intent-acceptance-007.json` not-run | UNASSIGNED — Runtime/Console | 缺 Verification/acceptance 或有 unknown 时无 completed；Runtime/process stop 不推进 Task/Effect | backend lifecycle carriers |
| `CONSOLE-AND-V1-PRD-003` | GA 仅 §2 phone/API/ABI/market/language 矩阵 | product-only | not-implemented | none | UNASSIGNED — Android release | unsupported device/form factor/channel fail closed | `CONSOLE-AND-V1-POC-001`, `CONSOLE-AND-V1-POC-018` |
| `CONSOLE-AND-V1-PRD-004` | GMS、Play Store、Play Protect certification 为 GA 前提 | product-only | not-implemented | none | UNASSIGNED — Android release/security | 无/旧 GMS 或 uncertified device 无 GA capability | `CONSOLE-AND-V1-POC-001`, `CONSOLE-AND-V1-POC-018` |
| `CONSOLE-AND-V1-PRD-005` | account-first OIDC/PKCE 与 tenant/node projection | partial: `REQ-MGMT-SESSION-001`, `REQ-MGMT-SESSION-002`, `REQ-MGMT-SESSION-003`; mobile carrier unregistered | not-implemented | `management-session-denials.json` not-run | UNASSIGNED — Identity | redirect/session/tenant/node mismatch 无数据/写 | OIDC/mobile session carrier |
| `CONSOLE-AND-V1-PRD-006` | work/personal profile 为独立 install/UID/binding，禁止跨 profile | unregistered | not-implemented | none | UNASSIGNED — Android Enterprise/Identity | 两 profile 无共享 key/token/session/cache | `CONSOLE-AND-V1-POC-002` |
| `CONSOLE-AND-V1-PRD-007` | supervision lease 仅 foreground resumed/unlocked/fresh 时续 | partial: `REQ-CAP-003`; mobile lease carrier unregistered | not-implemented | none | UNASSIGNED — Supervision | background/lock/death/Settings Force stop/Task Manager Stop/Recents/UI stale 零 renewal | `CONSOLE-AND-V1-POC-006`, `CONSOLE-AND-V1-POC-008`, `CONSOLE-AND-V1-POC-009` |
| `CONSOLE-AND-V1-PRD-008` | WorkManager/FGS/high FCM 只 hint/resync，不续租 | product-only + mobile carrier unregistered | not-implemented | none | UNASSIGNED — Android runtime | background APIs 触发时 server renewal count 不变 | `CONSOLE-AND-V1-POC-007`, `CONSOLE-AND-V1-POC-009` |
| `CONSOLE-AND-V1-PRD-009` | FCM 只携 opaque handle；delivery/click 非 truth/evidence | unregistered | not-implemented | none | UNASSIGNED — Notification | delay/drop/duplicate/replay 不改变 Inbox/Task truth | `CONSOLE-AND-V1-POC-007`, `CONSOLE-AND-V1-POC-011` |
| `CONSOLE-AND-V1-PRD-010` | PendingIntent explicit/immutable/one-shot，唯一 action 打开 App | product-only + carrier unregistered | not-implemented | none | UNASSIGNED — Android security | notification action 无直接 R0/R1 dispatch | `CONSOLE-AND-V1-POC-011` |
| `CONSOLE-AND-V1-PRD-011` | authority 决定 R0/R1，R2/R3 无执行入口 | partial: `REQ-MGMT-GATE-001`, `REQ-MGMT-SESSION-001`, `REQ-MGMT-SESSION-002` | not-implemented | `management-gate-denials.json` not-run | UNASSIGNED — Risk/Console | client/managed policy 无法降 risk；R2/R3 零 dispatch | mobile risk presentation |
| `CONSOLE-AND-V1-PRD-012` | versioned immutable `CanonicalDisplayEnvelope` 单源驱动 Compose display 与完整 envelope digest 签名；Class 3 auth-per-use device signature；current app identity 必须由 re-attestation/rebind 或本次 fresh request-bound Integrity 附加证据闭合；不宣称抵御 compromised client | partial: `REQ-CAP-001`, `REQ-CAP-003`, `REQ-CAP-005`, `REQ-MGMT-GATE-001`; display/signature carrier unregistered | not-implemented | none | UNASSIGNED — R1 security | stale/replayed challenge、旧 key + 新 app-build、envelope/session/device/nonce/idempotency 变化拒绝；需要 trusted display 时 R1 blocked | `CONSOLE-AND-V1-POC-004`, `CONSOLE-AND-V1-POC-005` |
| `CONSOLE-AND-V1-PRD-013` | 无合格 hardware-backed key/Class 3 时 R1 blocked，不降 device credential | unregistered | not-implemented | none | UNASSIGNED — R1 security | weak/no biometric、software/missing key 无 signature dispatch | `CONSOLE-AND-V1-POC-004`, `CONSOLE-AND-V1-POC-005` |
| `CONSOLE-AND-V1-PRD-014` | passkey 仅 upstream login，不作 device binding/R1 signer | product-only | not-implemented | none | UNASSIGNED — Identity | synced passkey 不能签 R1 envelope | `CONSOLE-AND-V1-POC-003`, `CONSOLE-AND-V1-POC-005` |
| `CONSOLE-AND-V1-PRD-015` | offline 不排队控制；draft 明确未提交 | partial: `REQ-CAP-005`, `REQ-EFF-004` | not-implemented | `effect-unknown-outcome.json` not-run | UNASSIGNED — Offline/Console | offline actions 零 Effect；draft 无 submitted 状态 | `CONSOLE-AND-V1-POC-009` |
| `CONSOLE-AND-V1-PRD-016` | `OUTCOME_UNKNOWN` 按原 key/ref reconcile，稳定 idempotency binding，禁止盲重试 | registered: `REQ-EFF-002`, `REQ-EFF-004`, `REQ-EFF-STATE-001` | not-implemented | `effect-idempotency-conflict.json`, `effect-unknown-outcome.json`, `effect-state-closure-008.json` not-run | UNASSIGNED — Effect/Console | 参数 digest 冲突、timeout/death 后换 key/retry、unknown 跳过 closure 均被拒绝 | backend Effect runner |
| `CONSOLE-AND-V1-PRD-017` | watch 使用 authorized snapshot+cursor；FCM 不是 watch | registered: `REQ-SHELL-WATCH-001` | not-implemented | `shell-watch-resume-006.json` not-run | UNASSIGNED — Watch/Console | stale/gap/乱序先 resnapshot，无 silent gap | watch API implementation |
| `CONSOLE-AND-V1-PRD-018` | remote completed/receipt/model text 不构成 Task acceptance | registered: `REQ-RUN-009`, `REQ-INTENT-ACCEPT-001` | not-implemented | `remote-completed-not-acceptance.json` not-run | UNASSIGNED — Task/Console | remote completed 保持 local Task 非 completed | acceptance carrier |
| `CONSOLE-AND-V1-PRD-019` | native escaped text + allowlist Markdown，无 raw HTML/JS/WebView bridge | product-only | not-implemented | none | UNASSIGNED — Renderer security | malicious corpus 无 script/network/system action；build/runtime WebView no-use gate 成立 | `CONSOLE-AND-V1-POC-012` |
| `CONSOLE-AND-V1-PRD-020` | 链接规范化后交 allowlisted browser/Custom Tabs provider，App Links 打开后重授权 | partial: `REQ-CAP-001`, `REQ-CAP-005`; mobile link carrier unregistered | not-implemented | none | UNASSIGNED — Android security | custom/invalid scheme、host/handle/provider signer-version mismatch 无 protected view/write | `CONSOLE-AND-V1-POC-003`, `CONSOLE-AND-V1-POC-011`, `CONSOLE-AND-V1-POC-012` |
| `CONSOLE-AND-V1-PRD-021` | SAF 只选不可信数据上传，不执行/解压/解释 | product-only | not-implemented | none | UNASSIGNED — File ingestion | URI/MIME/path/archive 负例无本地执行 | `CONSOLE-AND-V1-POC-012` |
| `CONSOLE-AND-V1-PRD-022` | token/key/FCM/binding/sensitive projection 排除 backup/D2D | unregistered | not-implemented | none | UNASSIGNED — Privacy/Identity | restore 后必须新 binding，旧 key/token 不存在 | `CONSOLE-AND-V1-POC-010` |
| `CONSOLE-AND-V1-PRD-023` | 所有用户 draft/metadata 使用 credential-encrypted storage；默认无 direct-boot-aware FCM/业务 component；DE 只允许固定无账号 build/support metadata | product-only | not-implemented | none | UNASSIGNED — Android privacy | reboot 首次解锁前无 draft/token/account/binding/R1 data 或业务 component | `CONSOLE-AND-V1-POC-010` |
| `CONSOLE-AND-V1-PRD-024` | 通知通用、Recents 遮蔽、R1/secret 禁 copy | product-only | not-implemented | none | UNASSIGNED — Privacy/UX | lockscreen/Recents/clipboard 无敏感内容 | `CONSOLE-AND-V1-POC-007`, `CONSOLE-AND-V1-POC-013` |
| `CONSOLE-AND-V1-PRD-025` | R1/sensitive 页使用 FLAG_SECURE/HIDE_OVERLAY_WINDOWS/occlusion checks 并披露限制 | product-only | not-implemented | none | UNASSIGNED — Android security | overlay/partial occlusion 不触发 R1；文案不宣称绝对防截 | `CONSOLE-AND-V1-POC-013` |
| `CONSOLE-AND-V1-PRD-026` | Agent acquisition 只传 authority catalog/package ref，手机永不接收 executable bundle | partial: `REQ-AGENT-INSTALL-001`, `REQ-AGENT-SANDBOX-001` | not-implemented | `agent-installation-verification.json`, `agent-adapter-bypass.json` not-run | UNASSIGNED — Agent lifecycle | phone traffic/storage 无 bundle；invalid signature authority unchanged | `CONSOLE-AND-V1-POC-014` |
| `CONSOLE-AND-V1-PRD-027` | install/upgrade/rollback/uninstall 都经 fixed proposal/Effect/authority commit | partial: `REQ-AGENT-INSTALL-001`, `REQ-MGMT-GATE-001`, `REQ-EFF-004` | not-implemented | `agent-installation-verification.json`, `effect-unknown-outcome.json` not-run | UNASSIGNED — Agent lifecycle | stale/failure/unknown 不覆盖旧 installation | `CONSOLE-AND-V1-POC-014` |
| `CONSOLE-AND-V1-PRD-028` | Public/Managed 使用独立 app/FCM/App Links/binding/signing identity；fresh `attestationChallenge` enrollment 验证 expected package/signing digest/version | unregistered | not-implemented | none | UNASSIGNED — Release/Identity | 双装时无共享 token/key/session/ref；stale/replayed challenge、repack/wrong-cert/wrong-version/旧 key+新 build/channel mismatch enrollment 或危险操作拒绝 | `CONSOLE-AND-V1-POC-004`, `CONSOLE-AND-V1-POC-015` |
| `CONSOLE-AND-V1-PRD-029` | managed policy 只能收窄，不能扩大 risk/capability ceiling | partial: `REQ-CAP-001`, `REQ-CAP-005`, `REQ-MGMT-GATE-001` | not-implemented | `management-gate-denials.json` not-run | UNASSIGNED — Android Enterprise | managed override 无法执行 authority 禁止动作 | `CONSOLE-AND-V1-POC-002`, `CONSOLE-AND-V1-POC-015` |
| `CONSOLE-AND-V1-PRD-030` | GA 使用 AAB/Play App Signing/internal→closed→production/private；direct APK excluded | product-only | not-implemented | none | UNASSIGNED — Android release | GA artifacts/listing 无 direct APK 路径 | `CONSOLE-AND-V1-POC-015`, `CONSOLE-AND-V1-POC-016` |
| `CONSOLE-AND-V1-PRD-031` | signed short-lived support metadata 绑定 current app identity evidence、device/patch/GMS/browser provider/WebView no-use gate 与 channel/device-binding-bound monotonic floor epoch | unregistered | not-implemented | none | UNASSIGNED — Release security | expired/mismatch/低 epoch metadata、旧 key+新 build、无法取得 fresh request-bound current-build evidence、unsupported browser provider 或 WebView use 无危险能力；clear-data/reinstall 不重置 high-water mark | `CONSOLE-AND-V1-POC-016`, `CONSOLE-AND-V1-POC-018` |
| `CONSOLE-AND-V1-PRD-032` | floor/kill switch 只收窄客户端，不终止 authority Task/Effect | partial: `REQ-CAP-005`, `REQ-EFF-STATE-001` | not-implemented | `effect-state-closure-008.json` not-run | UNASSIGNED — Release/Authority | blocked client 不产生 cancel/pause/complete transition | `CONSOLE-AND-V1-POC-016` |
| `CONSOLE-AND-V1-PRD-033` | root/bootloader/Integrity/attestation 仅风险信号，`UNEVALUATED`/error 不作通过，异常保留 safe recovery | unregistered | not-implemented | none | UNASSIGNED — Security | signal fail/unknown/UNEVALUATED 选择性阻断且可 revoke/update/sign-out，不替代 app identity binding | `CONSOLE-AND-V1-POC-001`, `CONSOLE-AND-V1-POC-004`, `CONSOLE-AND-V1-POC-016` |
| `CONSOLE-AND-V1-PRD-034` | 普通 FCM token rotation 只原子更新 routing；FID/install/profile/key identity 变化 revoke/rebind；app version/build/signing identity 变化必须 re-attest/rebind 或逐次 fresh request-bound current-build evidence，不恢复旧管理权 | partial: `REQ-MGMT-SESSION-002`, `REQ-MGMT-SESSION-003`, `REQ-CAP-003`, `REQ-CAP-005` | not-implemented | `management-session-denials.json`, `context-revocation-cache-reuse.json` not-run | UNASSIGNED — Identity | old token mapping 作废但 binding 保留；stale/replayed challenge、FID/install/profile/key 变化或旧 key+新 app identity 无 current-build evidence 时无新 proposal/lease/Agent lifecycle | `CONSOLE-AND-V1-POC-002`, `CONSOLE-AND-V1-POC-004`, `CONSOLE-AND-V1-POC-007`, `CONSOLE-AND-V1-POC-010` |
| `CONSOLE-AND-V1-PRD-035` | 五目的地 Material 3 phone IA、predictive back、edge-to-edge/insets、完整横屏 | product-only | not-implemented | none | UNASSIGNED — Android UX | portrait/landscape/back/IME/cutout 核心页面完整 | `CONSOLE-AND-V1-POC-017` |
| `CONSOLE-AND-V1-PRD-036` | 48dp、TalkBack/Switch/Voice Access、Android 14+ 200% font、每设备最大 Display size、high contrast/color correction、keyboard、Remove animations、Compose progress semantics 为 GA gate | product-only | not-implemented | none | UNASSIGNED — Accessibility | 全 16 journeys 在指定设备/辅助技术矩阵完成，loading 无 `aria-*` 且 announcement 适度 | `CONSOLE-AND-V1-POC-017` |
| `CONSOLE-AND-V1-PRD-037` | `zh-CN/en` 用户任务语言优先，machine enum/ref/digest 保真 | product-only | not-implemented | none | UNASSIGNED — Content design | 长文本/Bidi/Android 14+ 200% font/每设备最大 Display size/high contrast 下无关键语义丢失 | `CONSOLE-AND-V1-POC-017` |
| `CONSOLE-AND-V1-PRD-038` | 只用最小第一方 content-free telemetry；diagnostics 预览后显式上传 | product-only | not-implemented | none | UNASSIGNED — Privacy/Telemetry | egress payload 无正文/token/key/binding secret/3P endpoint | `CONSOLE-AND-V1-POC-012`, `CONSOLE-AND-V1-POC-013` |
| `CONSOLE-AND-V1-PRD-039` | 每型号/carrier firmware/build 独立 gate；Pixel 证据不外推 Samsung | product-only | not-implemented | none | UNASSIGNED — Device QA | support claim 可追到 exact build/patch/GMS/browser provider package-signer-version/WebView no-use/app build | `CONSOLE-AND-V1-POC-018` |
| `CONSOLE-AND-V1-PRD-040` | 状态严格区分 contract/implementation/evidence/Profile；mobile carrier 缺失写 unregistered | product-only | not-implemented | none | UNASSIGNED — Traceability | 文档/未来 UI 不把 none/not-run 显示 pass/implemented | M1 runner；mobile contracts |

## 20. 官方来源 ledger

以下页面均已实际打开正文；查询日期均为 2026-07-20。URL 使用完整 canonical page URL。

| 准确标题 | 完整 URL | 适用 Android/API/政策版本 | 可引用事实 | 不可过度推断 |
|---|---|---|---|---|
| The activity lifecycle | https://developer.android.com/guide/components/activities/activity-lifecycle | Activity/Compose；当前文档 | 系统回收时杀进程；Compose 使用 lifecycle-aware collection；`onPause` 不适合重工作 | lifecycle callback 不是 lease/commit 保证 |
| Save UI state in Compose | https://developer.android.com/develop/ui/compose/state-saving | Jetpack Compose | `rememberSaveable`/`SavedStateHandle` 只适合少量 UI 状态；ViewModel 不跨 process death | saved state 不是 authority truth 或 secret store |
| Set up Predictive back | https://developer.android.com/develop/ui/compose/system/predictive-back-setup | Android 13/14 测试；15+ 默认 | Navigation Compose 2.8+ 支持；根 Activity 拦截会破坏系统动画 | back 不代表远端取消 |
| Task scheduling | https://developer.android.com/develop/background-work/background-tasks/persistent | WorkManager | work 可跨 app restart/reboot；受 constraints/Doze；非所有立即工作的通用方案 | 不保证准时、永久运行或 force-stop 后执行 |
| Platform power management with Doze | https://source.android.com/docs/core/power/platform_mgmt | Doze；Android 7+ light Doze | 网络/job/sync/alarm 在 Doze 受限；维护窗执行 | AOSP integrator 行为不等于所有 OEM 精确实现 |
| Power management resource limits | https://developer.android.com/topic/performance/power/power-details | 含 Android 16/API 36 quota | WorkManager 使用 JobScheduler；bucket 数值只是近似且会变化；Android 16 FGS 下 job 仍可受 quota | FGS 不意味着无限后台预算 |
| System restrictions on background tasks | https://developer.android.com/develop/background-work/background-tasks/bg-work-restrictions | 当前 Android | 精确限制由 manufacturer 决定；AOSP restricted 示例禁 job/alarm/network | Pixel/AOSP 不可外推 Samsung/carrier |
| Restrictions on starting a foreground service from the background | https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start | target API 31+；API 34 while-in-use；API 35 overlay | 后台通常不能启动 FGS；high FCM 可被降级；while-in-use 有额外限制 | 例外不是永久 supervisor 授权 |
| Foreground service types | https://developer.android.com/develop/background-work/services/fgs/service-types | target API 34+ | 每个 FGS 必须类型/权限；`shortService` 约 3 分钟；`remoteMessaging` 是设备间文本连续性 | `remoteMessaging` 不覆盖任意 Console lease |
| Handle user-initiated stopping of apps running foreground services | https://developer.android.com/develop/background-work/services/fgs/handle-user-stopping | Android 13/API 33+；不依赖 target SDK | Active apps Task Manager Stop 杀整个进程、activity back stack、FGS/notification，且无 callback；scheduled jobs/alarms 仍执行；下次启动可查 `REASON_USER_REQUESTED` | 不得与 Settings Force stop/hibernation stopped state、Recents swipe 或普通 process kill 混称 |
| App hibernation | https://source.android.com/docs/core/perf/hiber | Android 12+ | hibernation force-stops App；不运行 jobs/alerts/push；再次使用后需重注册 | OEM 可有额外不同机制 |
| Dependencies of Firebase Android SDKs on Google Play services | https://firebase.google.com/docs/android/android-play-services | 当前 Firebase Messaging 25.1.1 | Cloud Messaging 明确 `Google Play services: Required` | Firebase 其他 SDK 的无 GMS 能力不可外推 FCM |
| Get started with Firebase Cloud Messaging in Android apps | https://firebase.google.com/docs/cloud-messaging/android/get-started | 当前 FCM Android client；Android 6+ + Play Store | client 需要 Play Store/GMS；registration token 可变化 | 可从 Play 外分发不等于无 GMS 可用 |
| Set and manage Android message priority | https://firebase.google.com/docs/cloud-messaging/android/message-priority | FCM Android | normal 在 Doze 延迟；high 只尝试立即投递且处理有限；滥用会降级 | high priority 不是 SLA |
| Receive messages in Android apps | https://firebase.google.com/docs/cloud-messaging/android/receive | FCM Android | `onMessageReceived` 约 10 秒且可能更短；大量 pending 可触发 `onDeletedMessages`/full sync | callback 不是 durable worker 或 authority event |
| Set the lifespan of a message | https://firebase.google.com/docs/cloud-messaging/customize-messages/setting-message-lifespan | FCM Android/Web | message ID 仅表示 accepted；TTL/collapse/offline 会延迟或丢弃 | accepted/sent 不等于 delivered |
| Best practices for FCM registration management | https://firebase.google.com/docs/cloud-messaging/manage-tokens | 当前 FID transition；Android | registration/FID 需 server timestamp；Android 270 天 inactivity 后过期 | FID/token 不是 identity/capability |
| Notification runtime permission | https://developer.android.com/develop/ui/compose/notifications/notification-permission | Android 13/API 33+ | 新安装通知默认关闭；用户可拒绝/撤销 | App 不能依赖通知作为监督可达性 |
| Create and manage notification channels | https://developer.android.com/develop/ui/compose/notifications/channels | Android 8/API 26+ | notification 必须 channel；创建后 App 不能改行为，用户最终控制 | channel importance 不保证 alert |
| Create a notification | https://developer.android.com/develop/ui/compose/notifications/create-notification | AndroidX；lockscreen visibility | PRIVATE/SECRET/public version；explicit immutable PendingIntent 示例 | visibility 不能防所有 OEM/物理泄露 |
| Pending intents | https://developer.android.com/privacy-and-security/risks/pending-intent | API 23+ guidance；API 31 mutability requirement | mutable PendingIntent 可被填充；one-shot 收窄 replay | one-shot 不替代 server nonce/idempotency |
| Behavior changes: Apps targeting Android 12 | https://developer.android.com/about/versions/12/behavior-changes-12 | target API 31+ | 显式 `android:exported`/PendingIntent mutability；禁 notification trampoline | target 合规不等于组件安全自动完成 |
| Android Keystore system | https://developer.android.com/privacy-and-security/keystore | AndroidKeyStore API 18+；StrongBox API 28+ optional | key non-exportable；可 auth-per-use；StrongBox 更慢且须 feature-check | hardware-backed 不是用户/authority 身份 |
| Verify hardware-backed key pairs with key attestation | https://developer.android.com/privacy-and-security/security-key-attestation | GMS/Android 7+；Android 8 required；2026 root rotation；字段依 attestation/API 版本 | key generation 可绑定 caller-provided attestation challenge；server 验链/security level/CRL/challenge；新 root 自 2026-02-01 签发；application identity 可由 attested package/signing digest/version 字段参与 validation；非 Play device 可有 OEM root | challenge 必须由产品协议保证 fresh/single-use/expiry；`attestationApplicationId` 只证明 key 生成时 identity，不证明更新后 current build、用户看到的内容、R1 同意或 client 未 compromised |
| Show a biometric authentication dialog | https://developer.android.com/identity/sign-in/biometric-auth | AndroidX Biometric；API 版本组合有差异 | Class 3、device credential、CryptoObject；auth-per-use；enrollment 可使 key 失效 | biometric success 不是 operation/authority decision |
| About Credential Manager | https://developer.android.com/identity/credential-manager | Jetpack；Android 14 platform API | 推荐 credential exchange API；支持 passkey/password/federated | credential provider 不等于 device binding |
| About passkeys | https://developer.android.com/identity/passkeys | passkeys API 28+ | server challenge + public-key verification；private key 在 provider | 可同步 passkey 不默认绑定当前 install |
| Support Direct Boot mode | https://developer.android.com/privacy-and-security/direct-boot | Android 7/API 24+ | CE storage 首次解锁前不可用；DE 只放确需 boot 前使用的数据；不迁移 auth token | 文档不要求 App direct-boot-aware；本产品默认无 FCM/业务 component，所有用户 draft 留在 CE |
| Overview of the Play Integrity API | https://developer.android.com/google/play/integrity/overview | 当前 library 1.4+ min API 23 | 只应作为 layered anti-abuse signal；默认 10,000 requests/day；有 outage/error；app integrity 是风险输入 | `PLAY_RECOGNIZED` 不替代 key attestation application identity；`UNEVALUATED`/error 不得当作通过 |
| Make a standard API request | https://developer.android.com/google/play/integrity/standard | standard request；页面/库 minSdk 须按版本核对 | `requestHash` 可把 fresh Integrity verdict 绑定具体危险操作，最大约 500 bytes；自动 replay protection | requestHash/`PLAY_RECOGNIZED` 仍只是附加 risk/build evidence，不是 device-key user signature、authority 或 attestation current-build proof |
| Handle Play Integrity API error codes | https://developer.android.com/google/play/integrity/error-codes | Play Integrity | 缺 Play Store/GMS、网络、quota、outdated 都可失败 | API error 不应一律解释为攻击 |
| Add Intent filters for App Links | https://developer.android.com/training/app-links/add-applinks | App Links API 23+；Dynamic API 35+ | HTTP/HTTPS、`autoVerify=true`、`assetlinks.json`；版本间 host 规则不同 | domain verify 不授权业务动作 |
| Unsafe use of deep links | https://developer.android.com/privacy-and-security/risks/unsafe-use-of-deeplinks | Android security guidance | 多 App 可注册同 URI；输入须 allowlist 并重新检查 auth/authz | custom scheme/参数不可信 |
| Intent redirection | https://developer.android.com/privacy-and-security/risks/intent-redirection | Android 12 StrictMode；Android 16 hardening | nested Intent 需 sanitize/重建、清 URI grants；PendingIntent 可收窄 | Android 16 hardening 不替代输入验证 |
| FileProvider | https://developer.android.com/reference/androidx/core/content/FileProvider | AndroidX Core | 使用 `content://` 临时 grant、`exported=false`、限定 paths | content URI/MIME 不证明文件可信 |
| Access documents and other files from shared storage | https://developer.android.com/training/data-storage/shared/documents-files | SAF API 19+ | user picker 授权选定 URI；persistable URI 在移动/删除后失效 | picker 不验证内容安全 |
| Data and file storage overview | https://developer.android.com/training/data-storage | scoped storage target API 29+ | app-specific/shared/SAF 边界；sensitive data 用 internal storage | scoped storage 不替代内容校验 |
| WebView – Native bridges | https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges | 仅适用于未来可能启用的 WebView | `addJavascriptInterface` 暴露所有 frame 且无可靠 origin 验证；JS 默认关闭 | 当前产品禁止 WebView；本页只支持 no-use gate，不构成当前 provider floor |
| Manage WebView objects | https://developer.android.com/develop/ui/views/layout/webapps/managing-webview | 仅适用于未来可能启用的 WebView；API 24+ provider；Safe Browsing API 26+ | 若使用 WebView，provider/version 可变且 Safe Browsing 只拦已知威胁 | 当前版本必须证明未引入/未使用 WebView；不能把已安装 provider 当作当前安全 floor |
| Webviews – Unsafe URI Loading | https://developer.android.com/privacy-and-security/risks/unsafe-uri-loading | Android security guidance | WebView URI 必须完整解析并校验 scheme+host | 字符串 starts/endsWith 不是充分 host 校验 |
| CustomTabsClient | https://developer.android.com/reference/androidx/browser/customtabs/CustomTabsClient | AndroidX Browser；Android 11+ package visibility 适用 | 可解析设备上支持 Custom Tabs 的 preferred provider package；无 provider 时可返回 null | package discovery 不证明 provider signer/version 受支持；产品仍须独立 allowlist 和 PackageManager 校验 |
| Browser | https://developer.android.com/jetpack/androidx/releases/browser | AndroidX Browser 当前发布线 | AndroidX Browser 提供 Custom Tabs/Auth Tab 能力；package identity primitive 可结合 package name 与 signing certificate | library release 或 provider capability 不自动证明 OIDC/外链 provider 达到产品 floor |
| Tapjacking | https://developer.android.com/privacy-and-security/risks/tapjacking | Android 12/API 31+ 有增强 | full/partial occlusion 区分；partial 仍需显式处理；可 hide overlays | 不能宣称阻止全部 accessibility/overlay |
| Secure sensitive activities | https://developer.android.com/security/fraud-prevention/activities | `FLAG_SECURE`; Android 12 overlay | FLAG_SECURE 阻常规 screenshot/non-secure display；官方披露旧设备/录屏限制 | 不是绝对截图或 overlay 防护 |
| Copy and paste | https://developer.android.com/develop/ui/views/touch-and-input/copy-paste | Android 12/13 clipboard UI | clipboard 是系统级；API 33 sensitive flag 隐藏 preview | sensitive flag 不加密/撤销 clipboard |
| Back up user data with Auto Backup | https://developer.android.com/identity/data/autobackup | target/run API 23+；data rules API 31+ | 默认备份多数 app data；应排除 device-specific/FCM data；部分 OEM D2D 不受 `allowBackup=false` 完全约束 | backup 配置不保证跨 OEM 相同 |
| About Android App Bundles | https://developer.android.com/guide/app-bundle | Google Play；new apps since 2021-08 | AAB 由 Play 生成/签名 optimized APK；sideload split 不完整会失败 | AAB 本身不是可安装 APK |
| Sign your app | https://developer.android.com/studio/publish/app-signing | Android/Play signing | upload key 与 app-signing key 分离；签名连续性决定 update | upload-key reset 不等于 app key rotation |
| Use Play App Signing | https://support.google.com/googleplay/android-developer/answer/9842756?hl=en | 当前 Play App Signing | annual key upgrade API 24+；API 33+ 平台严格执行新 key | 旧 API 的 Play Protect 行为不可外推无 GMS |
| Build and test your Android App Bundle | https://developer.android.com/guide/app-bundle/test | Play internal/closed/open、bundletool | Play track 最接近真实 delivery；pre-launch report 有基础检查 | pre-launch report 不替代指定真机/安全测试 |
| Publish private apps from the Play Console | https://support.google.com/googleplay/work/answer/6145139?hl=en | Managed Google Play | private app 限定组织、由 EMM 分发；AAB/title 基本发布输入 | private 不等于免全部政策/测试 |
| PackageInstaller.SessionParams | https://developer.android.com/reference/android/content/pm/PackageInstaller.SessionParams | update ownership API 34+ | 只能首次安装由有权限 installer 请求；其他 installer 自动更新受限 | 普通 App 不能自行取得 update ownership |
| Android developer verification | https://developer.android.com/developer-verification | 2026-09 起分阶段，certified devices | verified developer + package registration；Play 外分发可用 Android Developer Console | 初期 rollout 不等于所有 sideload 全球同时阻断 |
| Frequently asked questions | https://developer.android.com/developer-verification/guides/faq | 截至 2026-07-15 更新 | 初期 2026-09-30 只覆盖指定 stores/regions；直接 sideload 初期不适用，2027 扩大；ADB 不受影响 | direct APK 仍不能写成长期无治理 GA |
| Device and Network Abuse | https://support.google.com/googleplay/android-developer/answer/9888379?hl=en | 查询日现行 Play policy；target API 34+ FGS | 禁 Play App 自更新/外部 DEX/JAR/SO；FGS 必须核心、可感知、可终止、必要时长 | interpreter 例外不允许未审核 Agent acquisition |
| Use of the AccessibilityService API | https://support.google.com/googleplay/android-developer/answer/10964491?hl=en | 查询日现行 Play policy | 非 accessibility tool 需声明/披露/同意；自主规划执行被禁止 | Console/monitoring 不可标 `isAccessibilityTool=true` |
| Provide information for Google Play's Data safety section | https://support.google.com/googleplay/android-developer/answer/10787469?hl=en | 查询日现行；public/closed/open；internal/private 有例外 | App/SDK/受控 WebView 数据都需准确披露；developer 负责 | Data safety approval 不证明技术安全 |
| SDK Requirements | https://support.google.com/googleplay/android-developer/answer/13323374?hl=en | 查询日现行 Play policy | developer 对第三方 SDK 数据/权限/动态代码/FGS 违规负责 | SDK 自述不能替代 App audit/declaration |
| Target API level requirements for Google Play apps | https://support.google.com/googleplay/android-developer/answer/11926878?hl=en | 2026-08-31 起 phone new/update API 36 | 新 App/更新须 target API 36；永久 private organization App 有例外 | policy 例外不代表安全 floor 可降低 |
| Material Design 3 in Compose | https://developer.android.com/develop/ui/compose/designsystems/material3 | Compose Material 3；Android 16 visual alignment | M3 native components/typography/navigation/accessibility foundation | 默认组件不自动满足全部业务 a11y/security |
| Build adaptive navigation | https://developer.android.com/develop/adaptive-apps/guides/build-adaptive-navigation | Material3 adaptive | compact window 用 NavigationBar；window size runtime 变化 | phone-only 仍不能假设固定 viewport |
| Set up Edge-to-edge | https://developer.android.com/develop/ui/compose/system/setup-e2e | target API 35+ 强制 | 必须处理 system bars、cutout、IME insets | Material component 不保证所有 custom UI 正确 |
| Make apps more accessible | https://developer.android.com/guide/topics/ui/accessibility/apps | Android/Compose | 48dp target；普通小文本 4.5:1；语义/描述 | Material 默认不能替代自定义组件检查 |
| Test your app's accessibility | https://developer.android.com/guide/topics/ui/accessibility/testing | Android testing | manual TalkBack/Switch、analysis、automated、user testing 互补 | 自动检查不等于真实辅助技术通过 |
| Input compatibility on large screens | https://developer.android.com/develop/ui/compose/touch-input/input-compatibility-on-large-screens | Compose keyboard；也适用于 phone a11y | keyboard navigation 在 phone 也可能是无障碍必需；应 keyboard-only 测试所有 UI | phone-only 不可删除外接键盘 gate |
| Features and APIs Overview | https://developer.android.com/about/versions/14/features | Android 14/API 34 | 非线性 font scale 到 200%；应最大字体测试；screenshot callback 只通知不提供图 | 200% font 不证明每设备 Display size、high contrast 或布局通过 |
| Change text & display settings | https://support.google.com/accessibility/android/answer/11183305?hl=en | Android accessibility；部分功能版本限定 | Android 14+ font 可到 200%；用户还可选择设备提供的 Display size、color correction、high-contrast/outline text（按版本）和 Remove animations | Display size 没有统一 200% 保证；设置对部分 App/设备生效，custom layout/motion 仍须测试 |
| Android Compatibility program overview | https://source.android.com/docs/compatibility/overview | AOSP CDD/CTS | Android-compatible 需 CDD+CTS；只获得潜在 GMS/Play licensing 资格 | compatibility 不自动包含 GMS |
| AOSP frequently asked questions (FAQ) | https://source.android.com/docs/setup/about/faqs | AOSP | AOSP 可用于非 compatible derivative；Compatibility Program 定义生态基线 | AOSP 行为不可外推商业 OEM/GMS |
| Compatibility FAQs | https://source.android.com/docs/compatibility/compatibility-faq | Android compatibility | Google Play access 非自动；OEM 可在 CDD 范围内定制 UI/行为 | Pixel/一个 OEM 证据不可外推另一个 OEM |

## 21. 最终状态声明

- Android phone v1 产品方向与 16 项决策：已记录。
- Android platform/mobile carrier machine contract：未登记或仅有通用要求部分覆盖。
- Android Console implementation：未提供。
- Android emulator/physical device/Play/Managed/a11y/security/PoC：未执行，evidence `none`。
- 相关既有 conformance vectors：`not-run`。
- Android Console Profile：`planned / not implemented`。
