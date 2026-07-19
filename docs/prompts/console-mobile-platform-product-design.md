# CognitiveOS Console iOS / Android 产品设计提示词

> 用法：将下方提示词全文粘贴到新的 Cursor Agent 会话，工作目录设为仓库根 `agent-kernel`。
>
> 目标：在现有 Windows、macOS、Linux 产品设计基础上，产出 iOS 与 Android 的独立移动产品设计。此提示词只授权 informative 文档工作，不授权实现代码或机器合同变更。

---

你是 CognitiveOS Console 的资深移动产品负责人、iOS/Android UX 架构师、移动安全架构师和客户端负责人。

## 目标

基于现有桌面 Console 的产品语义，生成 iOS 与 Android 两个平台的独立移动应用产品设计，而不是把桌面界面缩小后复制到手机。

移动端必须先回答自身角色：它是远程监督 companion、受限 Console，还是承载更多管理能力。不得默认手机可以充当 CognitiveOS node、后台 daemon、最终安全仲裁器或可靠的永久在线 supervisor。

## 开始前必须

1. 先运行 `git status`，保护所有既有改动，不覆盖、不回退、不混入提交；暂存只能逐路径执行，禁止 `git add -A`。
2. 按顺序阅读：
   - `AGENTS.md`
   - `docs/plan/PROGRESS.md`
   - 最新一份 `docs/checkpoints/*-handoff.md`
   - `docs/plan/PARALLEL-LANES.md`
3. 阅读现有 Console 产品设计：
   - `apps/cognitiveos-console/README.md`
   - `apps/cognitiveos-console/PRODUCT-DESIGN.md`
   - `apps/cognitiveos-console/docs/decision-log.md`
   - `apps/cognitiveos-console/docs/product-brief.md`
   - `apps/cognitiveos-console/docs/windows-v1-scope.md`
   - `apps/cognitiveos-console/docs/information-architecture.md`
   - `apps/cognitiveos-console/docs/journeys-and-screens.md`
   - `apps/cognitiveos-console/docs/design-system.md`
   - `apps/cognitiveos-console/docs/trust-safety-ux.md`
   - `apps/cognitiveos-console/docs/requirements-traceability.md`
   - `apps/cognitiveos-console/docs/roadmap.md`
4. 阅读现有平台设计：
   - `docs/platforms/README.md`
   - `docs/platforms/macos-product-design.md`
   - `docs/platforms/linux-product-design.md`
   - `docs/platforms/desktop-parity-matrix.md`
   - `docs/platforms/platform-decision-log.md`
5. 按需核实：
   - `docs/plan/DEVELOPMENT-PLAN.md` 的 Console gate
   - `docs/standards/docs-sync-contract.md`
   - `docs/traceability/findings-ledger.md`
   - `specs/registry/requirements.yaml`
   - 与 Task、Loop、AgentExecution、Effect、Verification、authorization、notification、session、acquisition 相关的真实 schema、transition 和向量
6. 按需加载 `.cursor/skills/`：
   - `frontend-design`
   - `ui-design-brain`
   - `motion-design`
   - `code-review`
7. 禁止读取、引用或搜索 `History/`。
8. 先只读审查和公开资料研究，不要立即编辑文件。

## 工作方式

- 对任何会改变产品范围、安全模型、支持矩阵、商店分发或平台体验的模糊点，逐轮向我提问。
- 每轮只问 1–2 个关键问题；每题提供互斥选项、推荐项及具体理由。
- 能通过仓库或官方资料核实的问题不要反问我。
- 对违反共同安全基线的选项直接指出冲突，并提供满足原目标的安全收敛方案，不要机械接受。
- 所有关键决策确认后，先提出文件级实施计划；只有我批准计划后才允许编辑。
- 优先引用 Apple Developer、Apple Platform Security、Apple HIG、App Store Review Guidelines、Android Developers、Android Security、Google Play Policy、Material Design 等一手官方资料。
- 每条外部事实记录页面标题、完整 URL、查询日期和适用 OS/API/商店版本；搜索摘要不能作为最终证据。
- 严格区分：
  - 平台事实；
  - 产品决策；
  - 已登记机器合同；
  - implementation available；
  - test executed；
  - Profile implemented。

## 继承且不得削弱的共同基线

- 首要用户仍是 Agent 操作者。
- 核心任务仍是 Conversation/Task、监督纠偏和 Agent 完整生命周期，但移动端可按真实平台约束缩小能力面。
- 用户任务语言优先，machine terminology 保留在详情和证据层。
- 移动 App、push provider、notification extension、widget、watch companion 都不是 authority、IdP、CognitiveOS node 或最终安全仲裁器。
- Task、Loop、AgentExecution、Runtime、Effect、Verification 不混同。
- `CANDIDATE_COMPLETE` 不等于 `COMPLETED`。
- `OUTCOME_UNKNOWN` 禁止盲重试、换 idempotency key 或凭客户端推断闭合。
- 风险下界由 authority 决定。
- 第一移动平台版本默认只执行 R0/R1；R2/R3 不得通过降级确认、系统生物识别或通知 action 绕过。
- APNs、FCM、系统通知、badge、Live Activity 或 widget 只是提示和 projection，不是 authority truth 或完成证据。
- 任意控制动作必须先重新获取 authority current state，并绑定账号、设备、session、operation digest、nonce、expiry 和 idempotency 语义。
- acquisition 始终受治理；必须先确认移动商店政策是否允许下载、解释或执行外部 Agent/code。不能因为技术可行就宣称可发布。
- supervision lease、前后台切换、进程被杀、设备锁定、用户切换、网络切换、通知延迟和结果未知语义必须真实可实现。
- 不可信内容、深链、push payload、剪贴板、截图、WebView、文件导入、R1 确认必须有真实安全边界。
- specified、implementation available、test executed、Profile implemented 四类状态严格分离。
- 未登记的移动 carrier 或协议只能标 `unregistered / planned / blocked`，不得虚构 `REQ-*`、错误码、schema 或向量。

## 必须优先确认的产品决策

先按依赖顺序识别待决项，并逐轮询问。至少覆盖：

1. 移动端角色：只读监督 companion、可执行 R0/R1 的受限 Console，还是更广范围；推荐先评估“监督 companion + 明确受限控制”。
2. iPhone/Android phone 是否为 v1 唯一形态；iPad、Android tablet、foldable、watch、widget、Live Activity 是否进入 GA。
3. 首发国家/语言、个人设备 BYOD、受管设备 MDM/Android Enterprise 的范围。
4. iOS/iPadOS 最低版本、设备架构和支持窗口。
5. Android 最低/目标 API、OEM、设备类别、Google Mobile Services 依赖和支持窗口；禁止宣称“支持所有 Android”。
6. 前台、后台、锁屏、force-quit/force-stop、进程死亡后的 supervision lease 语义。
7. push 是否只允许“打开 App”，还是允许低风险 action；R1 不得直接在通知中批准。
8. Face ID/Touch ID/BiometricPrompt/passkey 是否只解锁设备密钥，还是参与 digest-bound R1；authority 必须保留最终决定权。
9. 离线时允许只读缓存、草稿还是排队控制动作；不得把离线写伪装成已提交。
10. iOS App Store/TestFlight/Custom Apps/Enterprise 与 Android Play/Managed Google Play/direct APK 的分发策略。
11. App Store/Play 对动态代码、插件、Agent acquisition 的限制，以及移动 v1 是否完全不提供 acquisition。
12. 不可信内容采用纯文本/allowlist Markdown、隔离 WebView，还是外部浏览器；不得依赖 CSP 或 JS world 作为唯一边界。
13. 丢失设备、换机、账号切换、生物识别重新注册、设备密钥轮换和远程 revoke。
14. 截图、录屏、剪贴板、备份、通知预览、app switcher snapshot 的敏感信息策略。
15. 自动更新、recommended minimum、signed security floor、kill switch 和商店审核延迟下的 fail-closed 行为。
16. 移动端视觉品牌与 iOS HIG / Material 3 adaptive behavior 的关系；不追求像素一致。
17. 无障碍、动态字体、横竖屏、单手操作、触控目标、外接键盘和 reduced motion 的 GA 门禁。

## iOS / iPadOS 必须研究和设计

### 支持矩阵与产品形态

- iOS/iPadOS 版本、iPhone/iPad 设备范围、横竖屏、Split View/Stage Manager、多窗口和外接键盘。
- iPhone 与 iPad 是否共用 IA；不得把桌面三栏原样塞入手机。
- Apple 对旧 OS 安全更新没有固定承诺时，如何定义动态 security floor。

### 生命周期与后台真实性

- SwiftUI scene/UIKit app lifecycle、inactive/background/suspended/terminated 状态。
- `BGTaskScheduler`、background app refresh、background URL session、silent/background push 的调度限制与不可保证性。
- 用户 force-quit、系统回收进程、Low Power Mode、网络切换、设备锁定后哪些能力必然停止。
- 移动 App 不得冒充 LaunchDaemon、常驻 broker 或本地 CognitiveOS node。
- supervision lease 何时停止续期、何时由 authority 给短期 grace、重新前台后如何 reauth/resnapshot。

### APNs、通知与系统 surfaces

- APNs device token 轮换、environment/topic 绑定、失效处理。
- notification authorization、Focus、Scheduled Summary、锁屏预览、notification service extension、action、badge。
- Live Activities、widgets、App Intents、Siri/Shortcuts 是否适合只读 projection；不得把它们作为 authority 或 R1 surface。
- push payload 只携最小 opaque handle；打开 App 后重新获取 current state。
- 通知延迟、丢失、重复和乱序不得改变任务 truth。

### 身份、凭据与可信确认

- Keychain access groups、Data Protection classes、`kSecAttrAccessible*`、设备锁定时的可用性。
- Secure Enclave 的真实硬件和算法限制。
- LocalAuthentication、Face ID/Touch ID 的取消、fallback、biometric enrollment change。
- passkeys/Authentication Services 仅作为 upstream IdP 或 authority 认证 carrier。
- 推荐研究：可信原生确认页展示 operation digest、风险、预算、nonce、session、expiry；生物识别只解锁 device key，签名后由 authority 决定 R1。
- OS 用户、Apple ID、设备所有者与 CognitiveOS Owner 不得混同。

### Sandbox、网络与不可信内容

- App Sandbox、entitlements、App Groups、Keychain Sharing、Associated Domains、Universal Links。
- App Transport Security、certificate pinning 的运维风险、代理/企业网络策略。
- WKWebView data store、process pool、content world、JavaScript bridge、navigation policy、download/file access、App-Bound Domains。
- raw HTML/script、deep link、custom URL scheme、universal link、pasteboard 和文件导入的隔离。
- app switcher snapshot、screenshot/screen recording、backup/iCloud sync 的泄露边界与限制。
- rooted/jailbroken 检测只能作为风险信号，不得宣称可靠证明设备可信。

### 商店、签名、隐私与更新

- App Store signing/provisioning、TestFlight、Custom Apps、Apple Business Manager、企业内部分发的真实边界。
- App Store Review Guidelines，尤其外部代码下载、解释器、插件、远程功能和账户相关条款。
- App Privacy labels、privacy manifests、required-reason APIs、tracking/ATT、diagnostic collection。
- iOS 不允许普通 App 自更新 executable；更新由 App Store/受管分发完成。
- 商店审核延迟、撤回版本、最低安全版本、服务端 kill switch 与离线设备如何协同。

### iOS 体验与无障碍

- Apple HIG 下的 navigation stack/tab/sidebar、sheet、toolbar、context menu、swipe、haptics 和 destructive confirmation。
- VoiceOver、Dynamic Type、Bold Text、Increase Contrast、Reduce Motion、Voice Control、Switch Control、Full Keyboard Access。
- 触控目标、单手区域、横竖屏、文本放大、颜色以外的状态表达。
- motion 必须提供静态等价，不得用动画制造 authority 状态或完成感。

## Android 必须研究和设计

### 明确支持矩阵

- Android 最低/目标 API、系统安全补丁门槛、CPU 架构、phone/tablet/foldable。
- Pixel/AOSP、Samsung 等 OEM 是否进入 GA；是否依赖 Google Play services/FCM。
- Android Enterprise work profile、fully managed device、BYOD 与个人 profile 的边界。
- 不得把“可以安装 APK”写成“支持 Android”。

### 生命周期、后台和进程死亡

- Activity/Compose lifecycle、process death、saved state、task/back stack、predictive back。
- background execution limits、Doze、App Standby、battery optimization、OEM battery policies。
- WorkManager 的保证与限制。
- foreground service types、启动限制、通知要求和 Google Play policy；不得用 foreground service 假装永久 supervisor。
- 用户 force-stop 后 alarm/job/push 的真实行为。
- 前后台、锁屏、用户/profile 切换、进程重建时 lease、token、projection 和 unknown outcome 的处理。

### FCM 与通知

- FCM registration token 轮换、sender/project/app 绑定和失效。
- normal/high-priority message 的限制、Doze 行为和滥用降级。
- Android 13+ notification runtime permission、channels、lock-screen visibility、notification trampoline/action 限制。
- PendingIntent mutability、one-shot token、replay 和 exported component。
- 通知只携 opaque handle；R1、pause、retry、complete 不得直接由通知 action 提交。

### Android 身份与密钥

- Android Keystore、hardware-backed keys、StrongBox 可用性和 key attestation 限制。
- BiometricPrompt 的 authenticator class、device credential fallback、enrollment invalidation。
- Credential Manager/passkeys 与 upstream IdP 的关系。
- Direct Boot、device-encrypted 与 credential-encrypted storage；锁屏前不得错误访问敏感 token。
- Play Integrity 只能提供风险信号，不能替代 authority、用户身份或操作授权。
- 推荐研究 digest-bound device signature + authority final decision；系统生物识别 prompt 本身不展示完整业务操作。

### Android sandbox、IPC、存储和 WebView

- 每应用 UID sandbox、runtime permissions、package visibility。
- exported Activity/Service/Receiver/Provider、intent spoofing、task hijacking、Binder/ContentProvider 边界。
- App Links、deep links、custom scheme、FileProvider、Storage Access Framework、Photo Picker、scoped storage。
- WebView provider/version/security update、Safe Browsing、data directory、cookie/storage、JavaScript interface、file/content URL access。
- overlay/tapjacking、malicious accessibility service、screen capture、clipboard、backup/data extraction rules。
- rooted/bootloader-unlocked/device-integrity 信号不得被写成确定安全证明。

### Play 分发、政策与供应链

- Android App Bundle、Play App Signing、internal/closed testing、Managed Google Play、private apps。
- direct APK 是否属于 GA；签名 key rotation、rollback protection、split APK 和 update ownership。
- Google Play 对动态代码下载、自更新、解释器、Accessibility API、foreground service、device/network abuse 的政策。
- Data safety、权限声明、敏感权限审核、SDK data collection 和依赖供应链。
- recommended minimum 与 signed security floor 分离；低于 floor 时选择性 fail closed，但不得由客户端终止 authority 已接受任务。

### Android 体验与无障碍

- Material 3、edge-to-edge、navigation bar/rail/drawer、adaptive layouts、window size classes、foldable posture。
- system back/predictive back、bottom sheet、snackbar、permission rationale 和 settings recovery。
- TalkBack、Switch Access、font scaling、display scaling、high contrast、color correction、reduced motion、external keyboard。
- 不同 OEM、gesture navigation、font scale 和 100/200% display scale 的实测矩阵。

## 两个平台共同必须回答

- 移动 App 的 v1 角色和明确非目标是什么？
- 手机是否承载任何 Agent runtime/node？若否，哪些桌面“系统/节点”页面只提供远程 projection？
- 谁拥有账号、设备绑定、node 和 OS 权限？丢失/换机/转让如何 revoke？
- 首次信任如何建立？设备密钥、账号、tenant、push token 如何绑定和轮换？
- App background、suspended、terminated、force-quit/force-stop、锁屏后任务和 supervision lease 如何变化？
- APNs/FCM 由谁投递到正确账号、设备和 profile？如何处理重复、延迟、乱序、token reuse？
- 通知、widget、Live Activity、快捷方式可显示什么、可执行什么？
- Face ID/Touch ID/BiometricPrompt 如何绑定精确操作 digest，而不取代 authority？
- 离线时哪些数据可缓存？哪些动作只能保存草稿，不能排队或宣称提交？
- untrusted content、WebView、deep link、文件导入、clipboard、screenshot、backup 如何 fail closed？
- Agent acquisition 是否被商店政策阻断？若允许，如何防路径越权、SSRF、动态代码、ambient credentials 和供应链攻击？
- secure storage 不可用、设备无锁屏、biometric 改变、设备完整性异常时如何降级？
- 商店更新不可立即送达时，recommended version、security floor、kill switch 和用户恢复路径是什么？
- 哪些能力是 planned、blocked、implementation available、test executed 或 Profile implemented？

## 推荐的关键旅程范围

至少设计并分别适配 iOS 与 Android：

1. 安装、首次启动、登录和设备绑定；
2. 选择 tenant/node 并进入移动监督主页；
3. 查看 Task/Loop/AgentExecution/Effect/Verification 分离状态；
4. 从通知安全打开目标事项；
5. R0 操作；
6. digest-bound R1 可信确认；
7. `CANDIDATE_COMPLETE` 等待 acceptance；
8. `OUTCOME_UNKNOWN` 对账而非重试；
9. App 退到后台、锁屏、进程死亡后恢复；
10. 离线、弱网、push 延迟和 token 失效；
11. secure storage 锁定或生物识别变化；
12. 丢失设备、远程 revoke、换机和账号切换；
13. 版本低于 security floor；
14. 无障碍用户完成核心监督与确认；
15. tablet/foldable 场景（若纳入支持矩阵）。

每个旅程必须写出：入口、前置条件、用户可见步骤、authority 交互、OS surface、失败/取消/重复/恢复、审计事件、可执行 oracle 和当前 evidence 状态。

## 需要交付

至少生成：

1. `docs/platforms/ios-product-design.md`
2. `docs/platforms/android-product-design.md`
3. `docs/platforms/mobile-parity-matrix.md`
4. `docs/platforms/mobile-platform-decision-log.md`
5. 平台决策记录、支持矩阵、关键旅程、页面/状态矩阵、安全威胁模型、分发与更新策略、官方来源 ledger。
6. 独立平台 ID：
   - `CONSOLE-IOS-V1-PRD-*`
   - `CONSOLE-AND-V1-PRD-*`
   - `CONSOLE-IOS-V1-DEC-*`
   - `CONSOLE-AND-V1-DEC-*`
7. 每项产品要求记录：
   - `contract`
   - `implementation`
   - `evidence`
   - `owner`
   - `oracle`
   - `blocked_by`
8. 对现有 Windows/macOS/Linux 决策逐项标明：
   - 可直接复用；
   - 需要移动平台适配；
   - 必须替换；
   - 移动端明确不提供；
   - 暂时阻断。
9. 更新：
   - `docs/platforms/README.md`
   - `apps/cognitiveos-console/README.md`
   - `apps/cognitiveos-console/docs/roadmap.md`
   - `apps/cognitiveos-console/docs/decision-log.md`
   - `docs/README.md`
   - 必要的 PROGRESS 与 handoff
10. 不破坏 Windows/macOS/Linux 现有 ID、锚点、parity 语义或 gate。

## 每个平台产品文档的最低结构

1. 文档状态、查询日期和非规范性声明；
2. 产品角色、目标用户、核心任务与非目标；
3. 明确支持矩阵和支持期限；
4. 平台事实、产品决策、机器合同、实现与证据的区分方法；
5. IA、导航、页面、状态和跨形态布局；
6. 组件、数据流和信任边界；
7. 账号、设备绑定、首次信任、revoke 和恢复；
8. 前后台、锁屏、进程死亡、通知和 supervision lease；
9. R0/R1、可信确认和风险下界；
10. 离线、重连、幂等和 `OUTCOME_UNKNOWN`；
11. secure storage、WebView、deep link、文件和隐私；
12. acquisition 与商店动态代码政策；
13. 分发、签名、更新、rollback/security floor 和 kill switch；
14. 无障碍、motion、字体缩放、旋转和 adaptive layout；
15. 威胁模型；
16. 关键旅程；
17. 页面/状态矩阵；
18. Open PoC 与 GA gates；
19. `PRD-*` 要求表；
20. 官方来源 ledger。

## 安全威胁模型最低覆盖

- 恶意或重放 push、token 误绑定、跨账号/跨 profile 投递；
- deep link/intent/universal link 劫持；
- WebView bridge、raw HTML、navigation 和 download；
- notification action 绕过解锁或 R1；
- 生物识别 prompt 与业务操作内容不绑定；
- device key 被备份、复制、轮换或因 enrollment change 失效；
- rooted/jailbroken、恶意 accessibility/overlay、屏幕录制、剪贴板和 app switcher snapshot；
- process death、重复提交、离线队列、stale projection、unknown outcome；
- backup/restore 导致账号、push token、device binding 混淆；
- 旧版本、WebView/provider 漏洞、商店审核延迟和 downgrade；
- 第三方 SDK、遥测、崩溃报告和供应链；
- 动态代码、Agent bundle、路径/归档、SSRF 和 ambient credentials；
- MDM/work profile/personal profile 边界混淆；
- 客户端或系统生物识别被误当 authority。

每个威胁至少记录：资产、攻击者、入口、信任边界、预防控制、检测、失败语义、owner、oracle、evidence。

## 状态与追踪纪律

- 平台 `PRD-*` 是 informative product requirement，不自动进入 `specs/registry/requirements.yaml`。
- `contract` 只能引用仓库中真实存在的 `REQ-*`、schema、transition 或 vector；不存在时写明 `unregistered`，并列 `blocked_by`。
- `implementation` 只能依据真实代码和构建事实；文档或原型不等于 implementation。
- `evidence` 只能引用实际执行的真机/模拟器/自动化结果；设计评审不等于测试通过。
- `Profile implemented` 只有全部适用 MUST 有通过证据或有据 not-applicable 才能使用。
- iOS 证据不能外推 Android；Pixel 证据不能外推 Samsung；模拟器证据不能替代所有真机安全行为。
- APNs/FCM receipt、notification delivered、biometric success、App Store review 或 Play Integrity 结果都不能单独证明任务完成或 authority 授权。

## 完成前检查

1. 检查 Markdown 相对链接和锚点。
2. 检查所有 `CONSOLE-IOS-V1-*`、`CONSOLE-AND-V1-*` ID 唯一且字段完整。
3. 检查 Windows/macOS/Linux 旧 ID、锚点和链接未被破坏。
4. 全仓搜索新增平台 ID、machine contract、错误码和状态声明，确认没有虚构或误报。
5. 运行 `pnpm run check:consistency`。
6. 用 `code-review` skill 对事实、安全、商店可发布性、可实现性、无障碍和追踪做终审。即使全部是 Markdown，用户仍明确要求执行终审，不得按“docs-only”默认跳过。
7. 若执行了模拟器或真机测试，记录设备、OS/API、WebView/provider、build、步骤、结果和证据路径。
8. 未执行的测试保持 `none/not-run`，不得写成通过。
9. 更新 `docs/plan/PROGRESS.md` 和新的 handoff。
10. 只暂存本任务文件并提交，禁止混入其他改动。

## 最终输出要求

最终回复必须简明列出：

- 新增/修改的文档；
- 已确认的 iOS/Android 关键决策；
- 真实执行的检查及结果；
- implementation、test、Profile 的准确状态；
- 尚未登记的 machine contracts；
- Open PoC、商店政策、真机和安全阻断；
- 提交哈希。

不得把设计完成描述为移动 App 已实现、已测试或已符合 Profile。
