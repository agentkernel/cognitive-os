# CognitiveOS Console 移动平台 canonical 产品决策索引

> 类别：informative product decisions；append-only canonical index
>
> 查询日：2026-07-20
>
> Current state：`planned / implementation not-implemented / platform evidence none / Profile not implemented`；全局 conformance 分布见 PROGRESS，不构成移动证据
>
> 规范快照：`273 REQ / 55 errors / 61 schemas / 5 transitions / 84 vectors`

本文只索引已经冻结的 iPhone v1 与 Android phone v1 产品方向。它不新增或修改 requirement、error code、schema、transition、conformance vector 或 Profile，也不把 accepted product direction 写成实现、测试、平台证据或符合性事实。

详细设计与平台事实分别以 [iOS 产品设计](../../ios/docs/ios-product-design.md) 和 [Android 产品设计](../../android/docs/android-product-design.md) 为来源；桌面决策背景见 [Console v2 决策记录](../../../pc/docs/product/decision-log.md) 和 [桌面平台产品决策记录](../../../pc/docs/platforms/platform-decision-log.md)。本索引是移动平台冻结决策的 canonical 查找入口；平台来源 ledger、旅程、PoC 和 PRD 仍留在各自产品设计中。

## 1. 决策状态解释

- `accepted product direction`：产品方向已确认；只表示设计选择被冻结。
- `blocked`：方向成立，但 machine contract、实现、PoC、商店/真机验证或其他证据尚未闭合，不能声称能力可用。
- `planned`：属于产品计划，不表示已经排期、实现或发布。
- `unregistered`：移动 carrier 尚无已登记 machine contract；通用合同的部分覆盖不能替代该缺口。
- `partial machine contract`：存在相关通用合同，但不足以表达完整移动 carrier。
- iOS 与 Android 当前实现状态均为 `not-implemented`。
- 两个平台的 platform evidence 均为 `none`，全局 84 vectors 的执行分布不得外推为移动证据，移动 Profile 均为 `not implemented`。

## 2. 共同不可降级边界

以下内容只是对后续平台决策的共同解释，不是新的产品决策或 machine contract：

- 产品服务 Agent operator，并提供远端 Agent install、upgrade、rollback、uninstall 生命周期；手机始终只是受限远程 Console，不是 node、daemon、IdP、authority 或 final arbiter。
- Task、Loop、AgentExecution、Effect、Verification 是五个彼此独立的 authority lifecycle 域；远端 Runtime 是另一份独立 projection，不是第六个 authority lifecycle，也不能成为五域中任何一域的 authority。Runtime stop 不推进 Task 或 Effect，不证明 pause 或 completion；本地 cache、模型自述、远端 `completed`、receipt、push、biometric 或 integrity signal 同样不能推进 authority 状态或证明完成。
- risk floor 只能由 authority 判定。移动端只进入 authority 判定的 R0/R1；R2/R3 只解释和阻断，不得降级、旁路或由 managed policy 扩权。
- 登录必须 account-first，使用系统浏览器承载 OIDC/OAuth 2.1 Authorization Code + PKCE。每个 App install/profile 同时只允许一个活动账号与一个活动 device binding；一个账号可访问多个经 authority 返回的 tenant/node。
- supervision lease 只能在前台、已解锁、session/watch/UI fresh 且可响应时续租。后台 API、push、下载、job 或本地 timer 只能 hint/resync，不能续租或延长 authority grace。
- push 的唯一产品动作是打开 App；随后必须 reauthenticate、原子消费 opaque handle 并 resnapshot。push accepted、received、displayed 或 clicked 都不是 truth、authorization 或 evidence。
- digest-bound R1 必须由 app-owned native confirmation page 展示固定 canonical operation，以严格 device key 签名，并由 authority 最终验证和决定；passkey 仅用于 upstream login，不得替代 device key。该边界不宣称在 compromised client 下仍有 trusted display。
- Agent acquisition 只接受 authority catalog/package ref；下载、验签、检查、sandbox、安装和 commit 全部发生在远端 node。手机不得取得、解释、执行、缓存、解压、扫描或转发 Agent executable bundle。
- 持久化只允许最小非敏感 metadata、stable refs、设置和明确保存的受保护 draft；敏感 last-good 只在进程内并显示 `as_of`，离线不得排队控制动作。
- 内容只用 native escaped text 和版本化 allowlist Markdown；禁止 raw HTML、JavaScript、iframe、native bridge 和 remote image auto-load。通知、最近任务/app switcher、clipboard/pasteboard、capture、backup 和 diagnostics 必须采用严格情境隐私收窄。
- Public 与 managed 渠道使用独立应用、push、link、signing/device-binding 和 release identity；managed policy 只能收窄共同能力上限。
- OS/integrity/attestation/jailbreak/root/bootloader/MDM 信号只进入 risk policy，不能单独证明身份、授权、安全通过或完成。
- security floor 选择性 fail closed：阻断 lease、R1、Agent lifecycle 和 protected writes，同时保留安全只读、revoke、sign-out、diagnostics preview 和 update recovery；不得改变远端 authority Task/Effect 状态。
- 两个平台采用 native platform shell，并把完整辅助技术、字体缩放、横竖屏、键盘和 reduced-motion 等价路径作为未来 GA gate；当前没有通过证据。
- telemetry 仅允许最小、第一方、content-free 数据；无广告、tracking 或第三方 analytics。diagnostics 必须本地预览字段并由用户显式确认上传。
- 产品支持目标为滚动 24 个月，但必须受短期有效、签名、防回滚的 allowlist/support metadata 和动态 security floor 约束。

## 3. iPhone v1 决策

<a id="console-ios-v1-dec-001"></a>

### `CONSOLE-IOS-V1-DEC-001` 角色与能力上限

- **状态**：`accepted / blocked on contracts and implementation`
- **决策**：iPhone v1 面向 Agent operator，是“更广但受限的远程 Console”；提供 Conversation/Task、监督纠偏、tenant/node 选择，以及远端 Agent install、upgrade、rollback、uninstall。App 不是 node、daemon、authority、IdP 或 final arbiter。
- **理由 / 平台事实约束**：iPhone 生命周期、本地 projection 和用户界面都不能承担远端 authority 的持久状态、授权或完成仲裁。
- **产品后果**：所有写入、风险判定、状态迁移和完成声明都回到 current authority；手机本地状态不得推进任何 authority lifecycle。
- **blocked_by**：Console backend gate；未登记的移动 account/session/lease/R1 carrier；iOS 实现尚未开始；platform evidence `none`。

<a id="console-ios-v1-dec-002"></a>

### `CONSOLE-IOS-V1-DEC-002` 支持形态与期限

- **状态**：`accepted`
- **决策**：GA 仅 iPhone、arm64、iOS 18+；竖屏为主且横屏功能完整。支持目标为滚动 24 个月，并由短期有效、签名、防回滚的 build allowlist 与动态 iOS/security/WebKit/app floor 共同约束。
- **理由 / 平台事实约束**：Apple security、WebKit、App Review、SDK 和 build 状态会变化；`iOS >= 18` 本身不足以证明某个 build 仍受支持。
- **产品后果**：iPad、Watch、widget、Live Activity、App Intents、Siri 和 Shortcuts 保持 `planned/blocked`；allowlist 过期、签名无效或低于 hard floor 时选择性 fail closed。
- **blocked_by**：`IOS-POC-01`、`IOS-POC-14`；真实 iPhone/build/channel 证据；未登记的 signed build allowlist/floor carrier。

<a id="console-ios-v1-dec-003"></a>

### `CONSOLE-IOS-V1-DEC-003` 市场与设备所有权

- **状态**：`accepted`
- **决策**：首发市场为美国和新加坡，`en` 与 `zh-CN` 同期提供；BYOD 与 managed device 都在范围内。managed policy 只能收窄，不能扩大共同能力上限。
- **理由 / 平台事实约束**：Apple Account、OS 用户、设备所有者、MDM 管理员与 CognitiveOS Owner/operator 是不同身份与权限边界。
- **产品后果**：市场、语言、所有权和渠道都必须在 UI、隐私披露、支持矩阵及发布证据中明确；machine enum、ref 和 digest 不翻译。
- **blocked_by**：Public/managed 分渠道证据；美国/新加坡与双语可用性证据；`IOS-POC-01`、`IOS-POC-13`、`IOS-POC-16`、`IOS-POC-17`。

<a id="console-ios-v1-dec-004"></a>

### `CONSOLE-IOS-V1-DEC-004` Account-first 身份与设备绑定

- **状态**：`accepted / mobile carrier unregistered`
- **决策**：先通过 `ASWebAuthenticationSession`/系统浏览器完成 OIDC/OAuth 2.1 Authorization Code + PKCE；passkey 只作 upstream login。App container 保存一个非 Keychain 的随机 install marker/nonce，并明确排除 backup；fresh enrollment 把 marker/nonce、account、不可迁移 device key、session 和 push registration 交给 server，server mint 新 `install_generation`。raw APNs token 的路由作用域是 App + device + topic/environment；authority mapping 只绑定一个活动 account/device/channel。tenant/node 不写入 raw token，只进入一次性 handle 的解析上下文和 R1 envelope。每个 App install 同时只有一个活动账号与一个活动 binding；一个账号可访问多个 tenant/node。
- **理由 / 平台事实约束**：iOS reinstall 通常移除 App container，但旧 Keychain item 可能保留；非 Keychain marker/nonce 缺失或不匹配只能帮助检测 honest reinstall，不能证明客户端未被篡改。系统 web authentication、同步 passkey、会轮换的 APNs token、Apple Account、install marker 和 App Attest 都不能替代 install-bound device identity；App Attest 只作 risk signal，不证明 compromised client。
- **产品后果**：marker/nonce 缺失或 mismatch 时先隔离旧 Keychain handle，禁止其用于 R1、lease 或 protected write，再执行 fresh enrollment，由 server mint 新 `install_generation`；不得由客户端沿用或自增旧 generation。换号、登出、reinstall、restore、换机、biometric enrollment 变化或渠道切换同样必须 revoke/rebind，旧 session/watch/cache/draft binding 与 push mapping 失效。支持 account creation 的 Public App Store 或 ABM Custom App target 必须在该 target 的 App 内提供可发起的账户删除入口；不支持 account creation 的 target 必须保留 flow/config evidence。高度监管行业只能在 App 内发起后增加客服确认，不能删除或替代该入口；账户删除与 revoke 单一 device binding 是两个独立动作。
- **blocked_by**：mobile auth、device enrollment、install marker/`install_generation`、session、push registration 与 revoke/rebind carrier 均未登记；`IOS-POC-02`、`IOS-POC-03`、`IOS-POC-07`、`IOS-POC-09`。

<a id="console-ios-v1-dec-005"></a>

### `CONSOLE-IOS-V1-DEC-005` iPhone 信息架构

- **状态**：`accepted`
- **决策**：采用 iOS native shell、`TabView`、每个 tab 独立 `NavigationStack` 和短决策 sheet；一级 tab 固定为 Work、Tasks、Agents、Inbox、More。
- **理由 / 平台事实约束**：iPhone compact viewport、平台导航与辅助技术要求需要原生、单列优先的交互；桌面三栏或像素外观不能直接缩放到手机。
- **产品后果**：Task、Loop、AgentExecution、Effect、Verification 五个 authority lifecycle 域、Trust Strip 和 Flow Thread 保持共享语义；远端 Runtime 作为独立 projection 呈现，不并入五域，Runtime stop 不推进 Task 或 Effect。布局、导航、焦点和系统 surface 按 iOS 原生行为实现。
- **blocked_by**：iOS 客户端实现；`IOS-POC-16`、`IOS-POC-17`；真实 iPhone navigation、rotation 与辅助技术证据。

<a id="console-ios-v1-dec-006"></a>

### `CONSOLE-IOS-V1-DEC-006` 生命周期与 supervision lease

- **状态**：`accepted / lease carrier unregistered`
- **决策**：lease 仅在 scene active、设备解锁、AuthenticationSession 当前、watch/UI fresh 且 App 可响应时续租。inactive、background、lock、terminated、force-quit 都立即取消客户端续租资格；authority 只可使用预先固定的短 grace。
- **理由 / 平台事实约束**：iOS 后台执行、BGTask、background push、URLSession 和终止回调都有限、可延迟或无保证，不能承载永久 supervisor。
- **产品后果**：所有后台能力只作 hint/resync；恢复前台后先 reauthenticate、resnapshot、恢复 watch，再由用户显式恢复 supervision。
- **blocked_by**：mobile supervision lease/grace/eligibility carrier 未登记；`IOS-POC-06`；active/background/lock/kill/force-quit 真机证据。

<a id="console-ios-v1-dec-007"></a>

### `CONSOLE-IOS-V1-DEC-007` APNs 与通知

- **状态**：`accepted / notification carrier unregistered`
- **决策**：APNs payload 只含最小高熵 opaque handle 和通用文案；唯一 action 是打开 App。打开后 reauthenticate、原子消费/解析 handle 并 resnapshot。
- **理由 / 平台事实约束**：APNs 是 best-effort，可能延迟、重复、乱序、合并或丢失；accepted、delivered、displayed、clicked 和 badge 都不构成 authority truth。
- **产品后果**：通知不得包含敏感对象信息或直接批准、暂停、重试、安装；通知缺失不改变 Inbox、deadline、Task 或 Effect。
- **blocked_by**：APNs registration、account/device/channel/topic binding 与 opaque-handle carrier 未登记；`IOS-POC-07`；sandbox/production 真机证据。

<a id="console-ios-v1-dec-008"></a>

### `CONSOLE-IOS-V1-DEC-008` Digest-bound R1

- **状态**：`accepted / signature and display carrier unregistered`
- **决策**：authority 提供完整、versioned canonical display envelope；同一个 immutable model 同时驱动 app-owned native 页面渲染，并生成 canonical bytes 与 envelope digest 供签名，禁止分别重建 display payload 和 signing payload。envelope 至少固定 account、tenant/node、device binding、bundle/channel、session、operation/target/expected version、parameters、risk、budget、egress、deadline、verification/acceptance、nonce、expiry、idempotency 和 display-profile version。Face ID/Touch ID 只解锁 Secure Enclave P-256、不可导出、不可备份的 device key；authority 最终验证 envelope digest、签名、current state 与 policy 后决定。
- **理由 / 平台事实约束**：biometric success 只说明本地认证成功；passkey 可同步且只适合 upstream login；两者都不能替代 operation-bound device signature 与 authority gate。同一 immutable model 可减少 display/signing split，但不能抵御 compromised client 篡改 renderer、model 或签名调用。
- **产品后果**：biometric enrollment 变化使 key 失效并触发 rebind；没有合格 key 时 R1 blocked，不提供 device passcode、文件 key 或 passkey 降级。产品不得宣称在 compromised client 下仍有 trusted display；如果安全目标要求这种可信显示，相关能力保持 blocked，直至存在独立可信显示合同与证据。
- **blocked_by**：versioned canonical display envelope、device-signature 与 binding carrier 未登记；`IOS-POC-03`、`IOS-POC-04`、`IOS-POC-05`；compromised-client trusted display 未提供。

<a id="console-ios-v1-dec-009"></a>

### `CONSOLE-IOS-V1-DEC-009` Offline、内容与隐私

- **状态**：`accepted`
- **决策**：只持久化最小非敏感连接 metadata、stable refs、设置和用户明确保存的非敏感 draft；敏感 draft 与敏感 last-good 都不落盘，后者只在进程内并标 `as_of`。非敏感 draft 除 Data Protection 外还使用应用层 install-bound encryption，并排除 backup；离线不排队控制动作。内容只用 native escaped text 与版本化 allowlist Markdown。
- **理由 / 平台事实约束**：App switcher snapshot、pasteboard、capture、backup、notification、diagnostics 和 imported content 都跨越不同 OS 信任边界；Data Protection、应用层加密、backup exclusion 或 capture signal 都不能单独成为数据未泄露、未迁移或已安全删除的证明。
- **产品后果**：cold launch、backup/restore 和 reinstall 不恢复敏感 draft、敏感 projection 或旧 draft key；raw HTML/JS/iframe/native bridge/remote image auto-load 禁止；外链交给系统浏览器；通知、app switcher、pasteboard、capture、backup 和 diagnostics 按上下文严格遮蔽、最小化或显式确认。
- **blocked_by**：iOS storage/content/privacy 实现；`IOS-POC-08`、`IOS-POC-09`、`IOS-POC-10`、`IOS-POC-11`、`IOS-POC-18`；platform evidence `none`。

<a id="console-ios-v1-dec-010"></a>

### `CONSOLE-IOS-V1-DEC-010` 远端 Agent 生命周期与 acquisition

- **状态**：`accepted / partial machine contract`
- **决策**：Agent operator 可请求远端 install、upgrade、rollback、uninstall；iPhone 只提交 authority catalog/package ref。远端 node 负责下载、验签、检查、sandbox、执行生命周期操作和 authority commit。
- **理由 / 平台事实约束**：App Store self-contained 与 dynamic-code 边界不允许把远端 Agent executable 变成 iPhone 客户端能力；catalog trusted 也不能跳过远端 admission。
- **产品后果**：iPhone 永不下载、解释、执行、缓存、解压、扫描或转发 Agent executable bundle；URL、Git、本地文件和 file importer 不得成为 acquisition 旁路。
- **blocked_by**：M6 远端 Agent lifecycle/backend gate；通用 install/sandbox 合同仅部分覆盖；`IOS-POC-12`、`IOS-POC-13`；相关 vectors `not-run`。

<a id="console-ios-v1-dec-011"></a>

### `CONSOLE-IOS-V1-DEC-011` 分发身份隔离

- **状态**：`accepted`
- **决策**：Public App Store 与 ABM Custom App 使用不同 bundle identity、APNs topic、Associated Domains、Keychain/App Attest/device-binding namespace 和 release records；TestFlight 对应目标 bundle，Enterprise Program 不提供。
- **理由 / 平台事实约束**：Apple 分发、push、associated-domain、keychain 与 managed configuration 都依赖具体应用身份；共享源代码不代表共享信任或证据。
- **产品后果**：两渠道 credential、token、key、cache、binding 和 diagnostics namespace 不互通；managed configuration 只能关闭或收窄能力。支持 account creation 的 Public App Store 或 ABM Custom App target 必须分别在自身 App 内提供可发起的账户删除入口，不得用另一 bundle 的入口代替；不支持 account creation 的 target 必须保留 flow/config evidence。高度监管行业只能增加客服确认，不能删除或替代 App 内发起入口。
- **blocked_by**：mobile distribution identity carrier 未登记；`IOS-POC-01`、`IOS-POC-07`、`IOS-POC-13`；Public/managed 各自商店、APNs、MDM 与升级证据。

<a id="console-ios-v1-dec-012"></a>

### `CONSOLE-IOS-V1-DEC-012` 更新、security floor 与 kill switch

- **状态**：`accepted / floor carrier unregistered`
- **决策**：App 更新只经 App Store、TestFlight 或 ABM/MDM；recommended minimum 只提示，signed hard floor 可选择性阻断 lease、R1、Agent lifecycle 和 protected writes。allowlist metadata 必须短期有效并防回滚。
- **理由 / 平台事实约束**：商店审核/传播与 OS/WebKit 安全状态会变化，不能成为安全正确性的实时依赖；App 也不能自行下载或执行更新 binary。
- **产品后果**：hard floor、metadata 过期/签名失败/回滚时 fail closed，同时保留安全只读、revoke、diagnostics preview 和 update recovery；kill switch 不修改 authority Task/Effect 状态。
- **blocked_by**：signed build allowlist/security-floor carrier 未登记；`IOS-POC-14`；App Store/TestFlight/ABM 分渠道恢复证据。

<a id="console-ios-v1-dec-013"></a>

### `CONSOLE-IOS-V1-DEC-013` Recovery 与 integrity signals

- **状态**：`accepted`
- **决策**：jailbreak、App Attest、device key、系统完整性、MDM 和 OS 状态都只作 risk signal。明确异常可选择性阻断 lease、R1、Agent lifecycle 和 protected writes，并保留安全恢复。
- **理由 / 平台事实约束**：完整性 API 可能不支持、失败或受 outage 影响；单一 verdict 既不能证明设备完全可信，也不能证明攻击成功。
- **产品后果**：异常或 key/binding 生命周期事件统一通过 authority revoke/rebind 收敛；安全只读、device revoke、sign-out 和 update recovery 始终保留。
- **blocked_by**：integrity-risk 与 revoke/rebind carrier 未登记；`IOS-POC-03`、`IOS-POC-05`、`IOS-POC-15`；真机异常/恢复证据。

<a id="console-ios-v1-dec-014"></a>

### `CONSOLE-IOS-V1-DEC-014` Accessibility、rotation 与 motion

- **状态**：`accepted`
- **决策**：采用 iOS native controls；44 pt 触控目标、VoiceOver、Voice Control、Switch Control、Full Keyboard Access、外接键盘、最大 Dynamic Type、完整横竖屏和 Reduce Motion 静态等价是 GA gate，并覆盖支持矩阵中显式定义的每个 iPhone 设备等价类。
- **理由 / 平台事实约束**：自动 audit、默认控件、simulator 或单一 iPhone 不能替代真实设备上的辅助技术测试；证据不得跨未证明的屏幕、输入、生物识别或布局设备等价类外推，颜色、位置和动画也不能独立表达状态。
- **产品后果**：R1、pause、reconcile、Agent lifecycle、revoke 和 update recovery 必须能由辅助技术独立完成，且不降低风险或确认强度。
- **blocked_by**：iOS native UI 实现；`IOS-POC-16`、`IOS-POC-17`；每个已定义 iPhone 设备等价类的真实设备辅助技术矩阵 evidence `none`。

<a id="console-ios-v1-dec-015"></a>

### `CONSOLE-IOS-V1-DEC-015` Telemetry 与 diagnostics

- **状态**：`accepted`
- **决策**：仅使用最小、第一方、content-free telemetry；无广告、tracking 或第三方 analytics。diagnostics 在本地生成字段预览，用户显式确认后才上传。
- **理由 / 平台事实约束**：crash report、OS log、SDK、privacy manifest、required-reason API 和 Privacy Label 都可能泄露或声明过度，必须逐 build 对齐实际依赖与数据流。
- **产品后果**：正文、prompt、文件名、URL query、stable ref、digest、token、tenant/node alias 和 biometric detail 不进入 telemetry；取消 diagnostics 即零上传。
- **blocked_by**：telemetry/diagnostics 实现与数据治理；`IOS-POC-18`；逐 build privacy/egress evidence。

<a id="console-ios-v1-dec-016"></a>

### `CONSOLE-IOS-V1-DEC-016` 状态与证据纪律

- **状态**：`accepted`
- **决策**：specified、implementation available、test executed、Profile implemented 四态严格分离；accepted product direction 不能替代其中任何一态。
- **理由 / 平台事实约束**：文档、schema、vector 枚举、remote completed、receipt、push click、biometric success 或客户端 cache 都不是已执行平台证据或完成证明。
- **产品后果**：iOS 固定当前声明为 `not-implemented / platform evidence none / Profile not implemented`；全局向量执行不得外推为 iOS 证据。
- **blocked_by**：本记录本身无 blocker；任何更强的可用、通过或符合性声明仍被未完成实现、未执行 vectors 与缺失平台证据阻断。

## 4. Android phone v1 决策

<a id="console-and-v1-dec-001"></a>

### `CONSOLE-AND-V1-DEC-001` 角色与权限上界

- **状态**：`accepted product direction / delivery blocked`
- **决策**：Android phone v1 面向 Agent operator，是更广但受限的远程 Console；提供 Conversation/Task、监督纠偏、tenant/node 选择和远端 Agent install、upgrade、rollback、uninstall。只进入 authority 判定的 R0/R1，R2/R3 只解释并阻断。
- **理由 / 平台事实约束**：Activity、FCM、BiometricPrompt、Play Integrity、本地 projection 和 managed policy 都不能承担远端 authority。
- **产品后果**：手机不是 node、daemon、authority、IdP 或 final arbiter；所有状态迁移、风险判定、授权和完成声明都来自 current authority。
- **blocked_by**：Console backend gate；未登记的 Android mobile carriers；Android implementation `not-implemented`；platform evidence `none`。

<a id="console-and-v1-dec-002"></a>

### `CONSOLE-AND-V1-DEC-002` 支持矩阵与形态

- **状态**：`accepted product direction / blocked by exact-device GA evidence`
- **决策**：GA 仅 Android phone，最低 Android 13/API 33，target Android 16/API 36，`arm64-v8a`，要求 GMS、官方 Play Store 与 Play Protect certification。候选 allowlist 精确为 Google Pixel 9、Pixel 9a、Pixel 10、Pixel 10a，以及 Samsung Galaxy S25、S25+、S25 Ultra、S26、S26+、S26 Ultra；所有 Fold 排除。
- **理由 / 平台事实约束**：Android/OEM/carrier 后台行为、firmware、patch、GMS、Play Protect 和 browser/Custom Tabs provider 会分化；Pixel 证据不能外推 Samsung，同型号 unlocked firmware 也不能外推 carrier firmware。WebView 不是受支持运行时依赖，而是禁止引入 gate。
- **产品后果**：tablet、foldable、watch、widget、desktop mode/DeX 保持 `planned/blocked`；每个型号必须固定 firmware、patch、GMS、browser/Custom Tabs provider、App build/channel、WebView dependency-absence check 与短期 signed support metadata。支持目标为滚动 24 个月，但不越过任一动态 floor。
- **blocked_by**：`CONSOLE-AND-V1-POC-001`、`CONSOLE-AND-V1-POC-018`；所列每个 Pixel/Samsung 组合的真机证据；signed support metadata/floor carrier 未登记。

<a id="console-and-v1-dec-003"></a>

### `CONSOLE-AND-V1-DEC-003` 市场、设备所有权与 profile

- **状态**：`accepted product direction / delivery blocked`
- **决策**：首发美国、新加坡，`zh-CN` 与 `en` 同期提供；同时支持 BYOD 与 Android Enterprise managed。personal/work profile 是不同 UID、App install、storage、session、FCM registration 和 device binding。
- **理由 / 平台事实约束**：Android profile、device owner/profile owner、OS 用户与 CognitiveOS Owner/operator 是不同身份边界；profile 隔离不能由 UI 标签或 managed policy 自报替代。
- **产品后果**：禁止跨 profile 复制、backup、restore 或继续 token/key/session/cache；managed policy 只能收窄共同能力上限，machine enum、ref 和 digest 保真。
- **blocked_by**：work/personal profile isolation carrier 与证据；`CONSOLE-AND-V1-POC-002`、`CONSOLE-AND-V1-POC-015`、`CONSOLE-AND-V1-POC-017`；美国/新加坡及双语矩阵 evidence `none`。

<a id="console-and-v1-dec-004"></a>

### `CONSOLE-AND-V1-DEC-004` Account-first 身份与设备绑定

- **状态**：`accepted product direction / mobile carrier unregistered`
- **决策**：通过 Custom Tabs/系统浏览器完成 OIDC/OAuth 2.1 Authorization Code + PKCE；Credential Manager/passkey 只作 upstream login。fresh enrollment 使用 server-minted、single-use、带 expiry 的 `attestationChallenge`，绑定 account、当前 profile/install/FID、device key、session 和 FCM registration，并验证 key attestation 中的 `attestationApplicationId` 与目标 channel 的 application ID/signing identity 一致。每个 install/profile 同时只有一个活动账号与一个活动 binding；一个账号可访问多个 tenant/node。
- **理由 / 平台事实约束**：`attestationApplicationId` 只证明 key 生成时的 package/version/signing identity，不证明当前运行 App 仍是该 build。Credential provider/passkey 可能跨设备同步；raw FCM token 会在同一 FID/install 下轮换，只是 routing address，不是 device identity。key attestation 与 Play Integrity 都只作为 authority risk evaluation 的输入，不是 identity、authorization、authority decision 或 completion evidence。
- **产品后果**：FCM token rotation 只在既有有效 binding 下更新 routing mapping，不触发 rebind，也不恢复任何能力；FID/install/profile/key/signing identity 变化，以及 logout、账号切换、clear data、restore 或换机，必须 revoke/rebind。App build/version 变化必须 re-attest/rebind；若策略允许正常 build/version update 保留 binding，则每个危险操作必须附加 fresh、request-bound Play Integrity app/version risk evidence，signing identity mismatch 仍必须 rebind。无法验证 current expected app identity 时，R1、lease 和 Agent lifecycle 全部 blocked；旧 challenge、idempotency、lease、cache 和 pending action 不恢复。
- **blocked_by**：mobile OIDC/session、fresh `attestationChallenge`、device enrollment/re-attestation、FCM binding 与 revoke/rebind carrier 未登记；`CONSOLE-AND-V1-POC-003`、`CONSOLE-AND-V1-POC-004`、`CONSOLE-AND-V1-POC-007`、`CONSOLE-AND-V1-POC-010`。

<a id="console-and-v1-dec-005"></a>

### `CONSOLE-AND-V1-DEC-005` 信息架构

- **状态**：`accepted product direction / implementation blocked`
- **决策**：采用 native Jetpack Compose/Material 3 phone shell；一级目的地固定为 Work、Tasks、Agents、Inbox、More，并使用 compact `NavigationBar`、system/predictive back、edge-to-edge 与正确 insets。
- **理由 / 平台事实约束**：compact phone window、IME、cutout、system navigation 与辅助技术需要原生自适应布局；桌面三栏不能缩放成手机卡片墙。
- **产品后果**：竖屏为主但横屏功能完整；Task、Loop、AgentExecution、Effect、Verification 五个 authority lifecycle 域、Trust Strip 和 Flow Thread 保持共享语义。远端 Runtime 作为独立 projection 呈现，不并入五域；Runtime stop 不推进 Task 或 Effect。布局、返回、焦点和系统 surface 遵循 Android 原生行为。
- **blocked_by**：Android native UI implementation；`CONSOLE-AND-V1-POC-017`；navigation、rotation、IME 与 accessibility evidence `none`。

<a id="console-and-v1-dec-006"></a>

### `CONSOLE-AND-V1-DEC-006` 生命周期与 supervision lease

- **状态**：`accepted product direction / lease carrier unregistered`
- **决策**：lease 仅在 Activity `RESUMED`、设备已解锁、当前 profile active、session/watch/UI fresh 且 App 可响应时续租。background、lock、process death、force-stop 都立即停止客户端续租；authority 只可使用预先固定的短 grace。
- **理由 / 平台事实约束**：WorkManager、FGS、high-priority FCM、Doze、standby、OEM restriction、hibernation 和 force-stop 都不提供永久、准时或可靠后台 supervisor。
- **产品后果**：WorkManager、FGS 和 FCM 只作 hint/resync，永不续租；force-stop 后不宣称任何后台能力，重新打开后必须 reauthenticate、resnapshot 并恢复 watch。
- **blocked_by**：Android supervision lease/grace/eligibility carrier 未登记；`CONSOLE-AND-V1-POC-006`、`CONSOLE-AND-V1-POC-008`、`CONSOLE-AND-V1-POC-009`；OEM 真机证据。

<a id="console-and-v1-dec-007"></a>

### `CONSOLE-AND-V1-DEC-007` FCM 与系统通知

- **状态**：`accepted product direction / notification carrier unregistered`
- **决策**：FCM payload 只含最小高熵 opaque handle 和通用文案；唯一 action 是通过 explicit、immutable、one-shot `PendingIntent` 打开 App。随后 reauthenticate、原子消费/解析 handle 并 resnapshot。
- **理由 / 平台事实约束**：FCM 可延迟、降级、重复、折叠或丢失；message ID、accepted、received、displayed、clicked 和 PendingIntent 属性都不替代 server nonce、idempotency 或 authority evidence。
- **产品后果**：通知不得携敏感正文或直接执行 R0/R1、pause、retry、install；notification permission/channel 关闭不改变 Inbox、deadline、Task 或 Effect。
- **blocked_by**：FCM registration/routing 与 opaque-handle carrier 未登记；`CONSOLE-AND-V1-POC-007`、`CONSOLE-AND-V1-POC-011`；Public/managed 分渠道 evidence。

<a id="console-and-v1-dec-008"></a>

### `CONSOLE-AND-V1-DEC-008` Digest-bound R1

- **状态**：`accepted product direction / signature and display carrier unregistered`
- **决策**：authority 提供完整、versioned canonical display envelope；同一个 immutable model 同时驱动 app-owned canonical confirmation page 渲染，并生成 canonical bytes 与 envelope digest 供签名，禁止分别重建 display payload 和 signing payload。该页面是 App 拥有的确认面，不是 hardware trusted display。envelope 至少固定 account、tenant/node、profile/install/device binding、application/channel、session、operation/target/expected version、parameters、Effect、risk、budget、egress、deadline、verification/acceptance、supervision behavior、nonce、expiry、idempotency 和 display-profile version。BiometricPrompt 只接受 Class 3 biometric，用于解锁 auth-per-use、non-exportable、hardware-backed Android Keystore signing key；StrongBox 仅为可选增强，authority 最终验证后决定。任何 key attestation/re-attestation 都使用 fresh server-minted、single-use、带 expiry 的 `attestationChallenge`。
- **理由 / 平台事实约束**：`attestationApplicationId` 只记录 key 生成时的 App version/signing identity，不证明确认时运行的 current build。BiometricPrompt success、key attestation、Play Integrity 和 passkey 都不等于 operation approval；这些 signal 只进入 authority risk evaluation，passkey 只服务 upstream login。同一 immutable model 可减少 display/signing split，但不能抵御 compromised client 篡改 Compose renderer、model 或签名调用。
- **产品后果**：App build/version 变化必须 re-attest/rebind，或由每个危险操作提供 fresh、request-bound Play Integrity app/version 附加 risk evidence；signing identity mismatch 必须 rebind。无法验证 current expected app identity 时，R1、lease 和 Agent lifecycle 全部 blocked。无合格 key/Class 3、enrollment change 或 key invalidation 时 R1 同样 blocked 并 rebind；不得降级到 `DEVICE_CREDENTIAL`、software/file key 或旧 proposal。产品不得宣称 compromised-client trusted display；若安全目标要求这种能力，相关确认保持 blocked。
- **blocked_by**：versioned canonical display envelope/device-signature、fresh `attestationChallenge` 与 current-app-identity risk carrier 未登记；`CONSOLE-AND-V1-POC-004`、`CONSOLE-AND-V1-POC-005`；compromised-client trusted display 未提供。

<a id="console-and-v1-dec-009"></a>

### `CONSOLE-AND-V1-DEC-009` Offline、内容与隐私

- **状态**：`accepted product direction / implementation blocked`
- **决策**：只持久化最小非敏感 connection metadata、stable refs、设置和 non-sensitive drafts；所有 draft 只进入 credential-encrypted（CE）app-private storage，敏感 last-good 只在进程内并标 `as_of`，离线不排队控制动作。v1 不提供业务 Direct Boot 路径：device-encrypted（DE）storage 不存业务 metadata、draft、token、account、binding、handle 或 R1 data，首次解锁前不运行 Console 业务流程。内容只用 native escaped text 与版本化 allowlist Markdown。
- **理由 / 平台事实约束**：CE storage 在首次解锁前不可用；Auto Backup/D2D、Recents、clipboard、screen capture、overlay、SAF 和 diagnostics 是不同 Android 信任边界。平台 flag 或 backup exclusion 不能保证抵御 root、恶意 AccessibilityService、外部摄像、所有 OEM 行为或数据迁移。
- **产品后果**：Direct Boot receiver/job/push 不读取或推进任何 Console 业务状态；raw HTML/JS/iframe/WebView bridge/remote image auto-load 禁止；R1/secret 禁 copy，Recents 全遮蔽，敏感页面使用适用的 `FLAG_SECURE`、`HIDE_OVERLAY_WINDOWS` 与 occlusion checks，并准确披露限制。
- **blocked_by**：Android storage/content/privacy implementation；`CONSOLE-AND-V1-POC-009`、`CONSOLE-AND-V1-POC-010`、`CONSOLE-AND-V1-POC-012`、`CONSOLE-AND-V1-POC-013`；platform evidence `none`。

<a id="console-and-v1-dec-010"></a>

### `CONSOLE-AND-V1-DEC-010` Agent lifecycle 与 acquisition

- **状态**：`accepted product direction / partial machine contract`
- **决策**：Agent operator 可请求远端 install、upgrade、rollback、uninstall；Android App 只提交 authority catalog/package/installation ref。远端 node 负责下载、固定 source/digest、验签、检查、sandbox、执行生命周期操作和 authority commit。
- **理由 / 平台事实约束**：Google Play dynamic-code 与 device/network-abuse 边界不允许客户端从外部取得 DEX/JAR/SO 或以解释器引入未审核能力；catalog metadata 也不能跳过远端 admission。
- **产品后果**：手机永不下载、解释、执行、缓存、解压、扫描或转发 Agent executable bundle；URL、Git、custom repository、SAF、本地 archive 和 clipboard 不得成为 acquisition 输入。
- **blocked_by**：M6 远端 Agent lifecycle/backend gate；通用 install/sandbox 合同仅部分覆盖；`CONSOLE-AND-V1-POC-014`；相关 vectors `not-run`。

<a id="console-and-v1-dec-011"></a>

### `CONSOLE-AND-V1-DEC-011` 分发 identity

- **状态**：`accepted product direction / distribution identity carrier unregistered`
- **决策**：Public Google Play 与 Managed Google Play private app 使用不同 application ID、Play App Signing identity、FCM project、App Links、device-binding namespace 和 release records；两渠道共享代码与能力上限，managed policy 只能收窄。direct APK 不进入 GA。
- **理由 / 平台事实约束**：Android update continuity、App Links、FCM、storage/Keystore namespace 与 Play 分发都绑定具体应用/签名身份；同机双装也仍是两个独立安全主体。
- **产品后果**：Public/managed 的 key、token、session、lease、draft、cache 和 telemetry dataset 不合并；每个渠道分别完成 Play review、rollout 与恢复验证。
- **blocked_by**：public/managed application identity carrier 未登记；`CONSOLE-AND-V1-POC-015`、`CONSOLE-AND-V1-POC-016`；Managed/Public Play 分渠道 evidence。

<a id="console-and-v1-dec-012"></a>

### `CONSOLE-AND-V1-DEC-012` 更新、floor 与 kill switch

- **状态**：`accepted product direction / floor carrier unregistered`
- **决策**：Public/managed 均使用 AAB、Play App Signing 和各自 signing identity；recommended minimum 只提示，短期 signed security floor 可选择性阻断 lease、R1、Agent lifecycle 和 protected writes，并执行 anti-rollback/monthly review。signed metadata 固定 expected application ID/build/version/signing identity、browser/Custom Tabs provider floor 与 `floor_epoch`，authority 强制 `floor_epoch` 单调不回退并拒绝更低 epoch；WebView 只作为禁止引入的 build/dependency gate，不作为受支持 provider。floor evaluation 如需 key attestation/re-attestation，必须使用 fresh server-minted、single-use、带 expiry 的 `attestationChallenge`。
- **理由 / 平台事实约束**：Play review/rollout、OEM firmware、security patch、GMS、Play Protect、App build 和 browser/Custom Tabs provider 都会变化；`attestationApplicationId` 只证明 key 生成时版本，客户端本地版本比较也不能替代 authority 对 current app identity 与 `floor_epoch` 的验证。key attestation 和 Play Integrity 只提供附加 risk evidence，不成为 authority、authorization 或 completion evidence。
- **产品后果**：App build/version 变化必须 re-attest/rebind，或对每个危险操作使用 fresh、request-bound Play Integrity app/version 附加 risk evidence；signing identity mismatch 必须 rebind。无法验证 current expected app identity 时，R1、lease 和 Agent lifecycle 全部 blocked。metadata 过期、签名/identity/provider 不匹配、`floor_epoch` 缺失/回退或低于 floor 时 fail closed 到安全只读、revoke、sign-out 和 update recovery；依赖扫描一旦发现 WebView 被直接或传递引入即阻断发布。kill switch 不 cancel、pause、complete 或 rollback authority Task/Effect。
- **blocked_by**：signed mobile support metadata/security-floor、fresh `attestationChallenge` 与 current-app-identity risk carrier 未登记；`CONSOLE-AND-V1-POC-016`、`CONSOLE-AND-V1-POC-018`；Play/OEM 更新恢复 evidence。

<a id="console-and-v1-dec-013"></a>

### `CONSOLE-AND-V1-DEC-013` 恢复与完整性

- **状态**：`accepted product direction / integrity carrier unregistered`
- **决策**：root、bootloader、Play Integrity、Keystore attestation、patch/GMS/profile 状态都只作 risk signal。明确异常可选择性阻断 lease、R1、Agent lifecycle 和 protected writes，并保留安全恢复。
- **理由 / 平台事实约束**：Integrity/attestation 可能因不支持、网络、quota、outage、旧组件或 OEM root 差异失败；error 或 verdict 不能单独证明攻击成功、安全通过或用户同意。
- **产品后果**：异常与 device/profile/key/account 生命周期事件通过 authority revoke/rebind 收敛；safe read-only、device revoke、sign-out 和 update recovery 保留。
- **blocked_by**：integrity-risk 与 revoke/rebind carrier 未登记；`CONSOLE-AND-V1-POC-001`、`CONSOLE-AND-V1-POC-004`、`CONSOLE-AND-V1-POC-016`；跨 OEM 异常/恢复 evidence。

<a id="console-and-v1-dec-014"></a>

### `CONSOLE-AND-V1-DEC-014` Accessibility 与 motion

- **状态**：`accepted product direction / evidence none`
- **决策**：采用 native Compose/Material 3 controls 与显式 Compose semantics；48dp 触控目标、TalkBack、Switch Access、Voice Access、200% font size、设备可提供的最大 Display size、high contrast、color correction、外接键盘、完整横屏和 Remove animations 静态等价是 GA gate。
- **理由 / 平台事实约束**：Material 默认组件、自动 accessibility scan、单一 emulator 或只测 200% font 不能替代真实支持设备上的最大 Display size、辅助技术和视觉适配测试；状态不能只靠颜色或动画表达。
- **产品后果**：Compose semantics 必须提供稳定 traversal、label、state、role、action 与适用 live announcement；high contrast/color correction 下语义和焦点仍可辨。登录后核心旅程、R1、pause、reconcile、Agent lifecycle、revoke 和 update recovery 必须在各输入模式下独立完成，且不降低确认强度。
- **blocked_by**：Android native UI implementation；`CONSOLE-AND-V1-POC-017`；Pixel/Samsung accessibility matrix evidence `none`。

<a id="console-and-v1-dec-015"></a>

### `CONSOLE-AND-V1-DEC-015` Telemetry 与 diagnostics

- **状态**：`accepted product direction / implementation blocked`
- **决策**：仅使用最小、第一方、content-free telemetry；无广告、tracking 或 third-party analytics。crash/diagnostics 默认不上传敏感正文，diagnostics 在本地预览字段并由用户显式确认上传。
- **理由 / 平台事实约束**：SDK、crash collector、Data safety 声明、日志和 profile/channel routing 都可能扩大数据范围；商店声明不能替代实际 egress 审查。
- **产品后果**：正文、token、key、binding secret、FCM token、stable ref、文件名和 biometric/integrity detail 不进入 telemetry；取消即零上传，Public/managed 数据集保持隔离。
- **blocked_by**：telemetry/diagnostics implementation 与数据治理；`CONSOLE-AND-V1-POC-012`、`CONSOLE-AND-V1-POC-013`；逐 build egress/Data safety evidence。

<a id="console-and-v1-dec-016"></a>

### `CONSOLE-AND-V1-DEC-016` 状态与追踪纪律

- **状态**：`accepted product direction`
- **决策**：product decision、registered contract、implementation、evidence、Profile 始终分离；Android carrier 缺失必须写 `unregistered`，通用合同不得冒充移动 carrier 已登记。
- **理由 / 平台事实约束**：文档、schema、vector 枚举、Play/FCM result、biometric/integrity success、remote completed 或客户端 cache 都不等于已实现、已执行测试或已符合 Profile。
- **产品后果**：Android 固定当前声明为 `not-implemented / platform evidence none / Profile not implemented`；全局向量执行不得外推为 Android 证据。
- **blocked_by**：本记录本身无 blocker；任何更强的可用、通过或符合性声明仍被未提供实现、未执行 vectors 与缺失平台证据阻断。

## 5. 未来变更规则

1. 本文件是 append-only。不得删除、重排、复用或静默改写既有移动决策 ID 与正文。
2. 未来只能追加新决策，或追加带日期、替代原因和 successor 引用的 supersession record；旧 ID 与原始内容永久保留。不得通过就地编辑把旧决定伪装成从未存在。
3. 移动 account/session/device binding/push/lease/R1 signature/support floor/public-managed identity 等 carrier machine contract 当前仍为 `unregistered`，除非未来 normative 流程在 registry/schema/vector/transition 中真实登记；本文或产品设计不能代替该流程。
4. 后续决策不得扩大客户端 authority、降低 authority risk floor、恢复 R2/R3 旁路、把 ContextView/receipt/push/biometric/integrity/remote completed 当作完成证据，或扩大数据、权限、后台能力和支持范围而不经过新的显式决策与适用 gate。
5. 任何新增记录都必须继续区分 accepted product direction、registered contract、implementation、executed evidence 与 Profile；缺失项保持 `blocked`、`unregistered`、`none` 或 `not-run`，不得升级措辞。
