# CognitiveOS Console 桌面到移动端 parity matrix

> 类别：informative product comparison
>
> 日期：2026-07-20
>
> 状态：Windows、macOS、Linux、iPhone 与 Android phone 均为产品设计；本文不构成实现、测试或 Profile 符合性声明
>
> Agent Hub：第三方 Agent Direct Takeover 的移动 companion 差异见 [agent-hub-platform-parity.md](./agent-hub-platform-parity.md)，为本表补充，不改写此处条目或既有移动 canonical 决策。

本文建立 Windows/macOS/Linux 到 iPhone/Android phone 的逐项对等关系。基线为 [Windows 产品决策](../../clients/pc/docs/product/decision-log.md)、[桌面平台产品决策](../../clients/pc/docs/platforms/platform-decision-log.md)、[桌面 parity matrix](../../clients/pc/docs/platforms/desktop-parity-matrix.md)、[iPhone-only v1 产品设计](./ios-product-design.md)与 [Android phone v1 产品设计](./android-product-design.md)。

本文只引用既有决策，不重新编号、重定义或覆盖它们。`CONSOLE-V2-DEC-001..017`、`CONSOLE-MAC-V1-DEC-001..011`、`CONSOLE-LNX-V1-DEC-001..011` 的 ID、源文档 anchor、范围和含义保持不变；移动端对应项使用既有 `CONSOLE-IOS-V1-DEC-*` 与 `CONSOLE-AND-V1-DEC-*`。

## 0. 五类判定

本文所有 parity 判定只使用以下五类：

| 类别 | 判定规则 |
|---|---|
| 直接复用 | 用户价值、治理语义与安全边界可原样保留，不依赖桌面 OS carrier。 |
| 移动平台适配 | 核心语义保留，但必须适配手机生命周期、交互、系统 surface 或 carrier。 |
| 必须替换 | 桌面机制在移动平台不成立，必须由已明确的移动机制取代。 |
| 移动端明确不提供 | iPhone/Android phone v1 明确排除该能力或本地机制，不建立移动等价物。 |
| 暂时阻断 | 产品方向可讨论，但 machine contract、实现、PoC 或已执行证据不足，当前不能声明可用。 |

每个平台、每一项只取一个类别。类别描述的是产品关系，不表示 machine contract 已登记、实现已提供、测试已执行或 Profile 已符合。

## 1. Windows v1 → iPhone / Android phone

下表严格保持 17 项 Windows 决策各一行；“对应 DEC”是移动产品方向的落点，不改写 Windows 决策。

| Windows 决策 | iOS 分类 | iOS 对应 DEC | Android 分类 | Android 对应 DEC | 原因 / 当前边界 |
|---|---|---|---|---|---|
| [CONSOLE-V2-DEC-001 首要用户与核心任务](../../clients/pc/docs/product/decision-log.md#console-v2-dec-001-首要用户与核心任务) | 直接复用 | [CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 直接复用 | [CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | Agent operator、Conversation/Task、监督纠偏与 Agent lifecycle 的用户价值不变；移动 App 仍非 authority。 |
| [CONSOLE-V2-DEC-002 发布平台与范围](../../clients/pc/docs/product/decision-log.md#console-v2-dec-002-发布平台与范围) | 必须替换 | [CONSOLE-IOS-V1-DEC-002](./mobile-platform-decision-log.md#console-ios-v1-dec-002) | 必须替换 | [CONSOLE-AND-V1-DEC-002](./mobile-platform-decision-log.md#console-and-v1-dec-002) | Windows 桌面与单本机节点范围不能外推；分别改为 iPhone-only 与列名 Android phone/floor 支持矩阵。 |
| [CONSOLE-V2-DEC-003 本地节点承载模型](../../clients/pc/docs/product/decision-log.md#console-v2-dec-003-本地节点承载模型) | 移动端明确不提供 | [CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 移动端明确不提供 | [CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | 手机不承载 node、daemon、Agent runtime、authority 或 Windows Service 等价物；只连接远端 node。 |
| [CONSOLE-V2-DEC-004 本地身份与账号所有权](../../clients/pc/docs/product/decision-log.md#console-v2-dec-004-本地身份与账号所有权) | 必须替换 | [CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004) | 必须替换 | [CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004) | Windows SID、本地账号和 bootstrap bundle 改为 account-first upstream login、tenant/node projection 与 device enrollment/rebind。 |
| [CONSOLE-V2-DEC-005 首次信任](../../clients/pc/docs/product/decision-log.md#console-v2-dec-005-首次信任) | 必须替换 | [CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004) | 必须替换 | [CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004) | 本机 loopback TOFU 不适用于远端移动客户端；改为受信登录入口、authority 返回 scope 与绑定 challenge。 |
| [CONSOLE-V2-DEC-006 风险范围与 R1 确认](../../clients/pc/docs/product/decision-log.md#console-v2-dec-006-风险范围与-r1-确认) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-008](./mobile-platform-decision-log.md#console-ios-v1-dec-008) | 移动平台适配 | [CONSOLE-AND-V1-DEC-008](./mobile-platform-decision-log.md#console-and-v1-dec-008) | R0/R1 上限与 authority risk floor 保留；确认 carrier 分别适配 Secure Enclave 与 Android Keystore，R2/R3 仍无执行入口。 |
| [CONSOLE-V2-DEC-007 Agent 来源与全生命周期](../../clients/pc/docs/product/decision-log.md#console-v2-dec-007-agent-来源与全生命周期) | 必须替换 | [CONSOLE-IOS-V1-DEC-010](./mobile-platform-decision-log.md#console-ios-v1-dec-010) | 必须替换 | [CONSOLE-AND-V1-DEC-010](./mobile-platform-decision-log.md#console-and-v1-dec-010) | lifecycle 用户旅程保留，但 URL/Git/本地包获取改为 authority catalog/package ref；手机不接收 executable bundle。 |
| [CONSOLE-V2-DEC-008 失联暂停与监督 lease](../../clients/pc/docs/product/decision-log.md#console-v2-dec-008-失联暂停与监督-lease) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-006](./mobile-platform-decision-log.md#console-ios-v1-dec-006) | 移动平台适配 | [CONSOLE-AND-V1-DEC-006](./mobile-platform-decision-log.md#console-and-v1-dec-006) | authority lease/expiry 语义保留；仅前台、解锁、fresh 时续租，background/push/WorkManager 等均不得续租。 |
| [CONSOLE-V2-DEC-009 窗口关闭与通知](../../clients/pc/docs/product/decision-log.md#console-v2-dec-009-窗口关闭与通知) | 必须替换 | [CONSOLE-IOS-V1-DEC-006](./mobile-platform-decision-log.md#console-ios-v1-dec-006)、[007](./mobile-platform-decision-log.md#console-ios-v1-dec-007) | 必须替换 | [CONSOLE-AND-V1-DEC-006](./mobile-platform-decision-log.md#console-and-v1-dec-006)、[007](./mobile-platform-decision-log.md#console-and-v1-dec-007) | tray/窗口退出流程改为 scene/Activity 生命周期和 push hint；通知唯一动作是打开 App 后重新认证并 resnapshot。 |
| [CONSOLE-V2-DEC-010 离线与敏感快照](../../clients/pc/docs/product/decision-log.md#console-v2-dec-010-离线与敏感快照) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [CONSOLE-AND-V1-DEC-009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | last-good/`as_of`、离线禁写和敏感 projection 不落盘语义保留；存储、backup、app-switcher/Recents 与 capture surface 按平台收窄。 |
| [CONSOLE-V2-DEC-011 持久化/审计故障](../../clients/pc/docs/product/decision-log.md#console-v2-dec-011-持久化审计故障) | 暂时阻断 | —（共享核心语义，无专属移动 DEC） | 暂时阻断 | —（共享核心语义，无专属移动 DEC） | 桌面决策自身仍有 authority store/audit 合同漂移；移动端只投影远端 degradation，相关应急、授权与对账 carrier 未登记，不能本地闭合 unknown。 |
| [CONSOLE-V2-DEC-012 首页与导航](../../clients/pc/docs/product/decision-log.md#console-v2-dec-012-首页与导航) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-005](./mobile-platform-decision-log.md#console-ios-v1-dec-005) | 移动平台适配 | [CONSOLE-AND-V1-DEC-005](./mobile-platform-decision-log.md#console-and-v1-dec-005) | 任务导向与角色/readiness 落点保留；桌面导航改为 Work、Tasks、Agents、Inbox、More 五个移动入口。 |
| [CONSOLE-V2-DEC-013 Shell 工作区](../../clients/pc/docs/product/decision-log.md#console-v2-dec-013-shell-工作区) | 必须替换 | [CONSOLE-IOS-V1-DEC-005](./mobile-platform-decision-log.md#console-ios-v1-dec-005) | 必须替换 | [CONSOLE-AND-V1-DEC-005](./mobile-platform-decision-log.md#console-and-v1-dec-005) | 对话主画布与状态语义保留，但桌面侧栏/密度模型不能缩放照搬；手机采用单列导航、独立页面和短 sheet。 |
| [CONSOLE-V2-DEC-014 品牌与动效](../../clients/pc/docs/product/decision-log.md#console-v2-dec-014-品牌与动效) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-014](./mobile-platform-decision-log.md#console-ios-v1-dec-014) | 移动平台适配 | [CONSOLE-AND-V1-DEC-014](./mobile-platform-decision-log.md#console-and-v1-dec-014) | 品牌核心、功能性动效和 motion 不作事实信号保留；分别适配 Reduce Motion 与 Remove animations。 |
| [CONSOLE-V2-DEC-015 语言与技术候选](../../clients/pc/docs/product/decision-log.md#console-v2-dec-015-语言与技术候选) | 必须替换 | [CONSOLE-IOS-V1-DEC-005](./mobile-platform-decision-log.md#console-ios-v1-dec-005) | 必须替换 | [CONSOLE-AND-V1-DEC-005](./mobile-platform-decision-log.md#console-and-v1-dec-005) | `zh-CN/en` 范围保留；Windows 的 Tauri/React 候选不能外推，移动设计分别采用 native iOS shell 与 Native Compose。 |
| [CONSOLE-V2-DEC-016 文档结构](../../clients/pc/docs/product/decision-log.md#console-v2-dec-016-文档结构) | 直接复用 | [CONSOLE-IOS-V1-DEC-016](./mobile-platform-decision-log.md#console-ios-v1-dec-016) | 直接复用 | [CONSOLE-AND-V1-DEC-016](./mobile-platform-decision-log.md#console-and-v1-dec-016) | 模块化文档、非规范声明与状态分离纪律继续适用；既有 Windows anchors 不移动。 |
| [CONSOLE-V2-DEC-017 v2 ID](../../clients/pc/docs/product/decision-log.md#console-v2-dec-017-v2-id) | 必须替换 | [CONSOLE-IOS-V1-DEC-016](./mobile-platform-decision-log.md#console-ios-v1-dec-016) | 必须替换 | [CONSOLE-AND-V1-DEC-016](./mobile-platform-decision-log.md#console-and-v1-dec-016) | Windows `CONSOLE-V2-*` namespace 原样保留；移动条目必须使用各自既有 namespace，不能复用或重编号 Windows ID。 |

## 2. macOS v1 → iPhone / Android phone

下表严格保持 11 项 macOS 决策各一行。

| macOS 决策 | iOS 分类 | iOS 对应 DEC | Android 分类 | Android 对应 DEC | 原因 / 当前边界 |
|---|---|---|---|---|---|
| [CONSOLE-MAC-V1-DEC-001 支持范围](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-001-支持范围) | 必须替换 | [CONSOLE-IOS-V1-DEC-002](./mobile-platform-decision-log.md#console-ios-v1-dec-002) | 必须替换 | [CONSOLE-AND-V1-DEC-002](./mobile-platform-decision-log.md#console-and-v1-dec-002) | Universal 2、macOS 14 与桌面 security floor 不适用；按 iPhone 或 Android phone 的 OS/device/build floor 独立声明。 |
| [CONSOLE-MAC-V1-DEC-002 节点拓扑](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-002-节点拓扑) | 移动端明确不提供 | [CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 移动端明确不提供 | [CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | machine-wide daemon、per-user broker 与 privileged helper 均不在手机范围；移动 App 只访问远端 node。 |
| [CONSOLE-MAC-V1-DEC-003 所有权、账号与 claim](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-003-所有权账号与-claim) | 必须替换 | [CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004) | 必须替换 | [CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004) | upstream identity 与 OS 身份正交原则保留；machine node claim 改为移动 App/device enrollment、revoke 与 rebind。 |
| [CONSOLE-MAC-V1-DEC-004 分发与更新入口](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-004-分发与更新入口) | 必须替换 | [CONSOLE-IOS-V1-DEC-011](./mobile-platform-decision-log.md#console-ios-v1-dec-011)、[012](./mobile-platform-decision-log.md#console-ios-v1-dec-012) | 必须替换 | [CONSOLE-AND-V1-DEC-011](./mobile-platform-decision-log.md#console-and-v1-dec-011)、[012](./mobile-platform-decision-log.md#console-and-v1-dec-012) | signed/notarized PKG 与 MDM 安装入口改为 App Store/TestFlight/ABM 或 Play/Managed Play 的独立 channel identity。 |
| [CONSOLE-MAC-V1-DEC-005 更新阈值与回退](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-005-更新阈值与回退) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-012](./mobile-platform-decision-log.md#console-ios-v1-dec-012) | 移动平台适配 | [CONSOLE-AND-V1-DEC-012](./mobile-platform-decision-log.md#console-and-v1-dec-012) | recommended/security floor 与不改 authority state 的语义保留；App binary 更新/回退受移动商店渠道约束。 |
| [CONSOLE-MAC-V1-DEC-006 不可信内容与 acquisition](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-006-不可信内容与-acquisition) | 必须替换 | [CONSOLE-IOS-V1-DEC-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009)、[010](./mobile-platform-decision-log.md#console-ios-v1-dec-010) | 必须替换 | [CONSOLE-AND-V1-DEC-009](./mobile-platform-decision-log.md#console-and-v1-dec-009)、[010](./mobile-platform-decision-log.md#console-and-v1-dec-010) | native safe content 保留；本地签名 bundle/picker acquisition 收窄为远端 catalog/package ref，手机零 executable bytes。 |
| [CONSOLE-MAC-V1-DEC-007 secure storage、锁屏与用户切换](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-007-secure-storage锁屏与用户切换) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004)、[009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004)、[009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | 无明文 fallback、锁屏停 lease/write 与恢复后 reauth/resnapshot 保留；适配 iOS protected data 或 Android profile/Keystore。 |
| [CONSOLE-MAC-V1-DEC-008 窗口、退出与通知](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-008-窗口退出与通知) | 必须替换 | [CONSOLE-IOS-V1-DEC-006](./mobile-platform-decision-log.md#console-ios-v1-dec-006)、[007](./mobile-platform-decision-log.md#console-ios-v1-dec-007) | 必须替换 | [CONSOLE-AND-V1-DEC-006](./mobile-platform-decision-log.md#console-and-v1-dec-006)、[007](./mobile-platform-decision-log.md#console-and-v1-dec-007) | Dock/menu extra/broker opt-in 改为 scene/Activity 生命周期；push 只作 opaque open hint。 |
| [CONSOLE-MAC-V1-DEC-009 R1 本地认证与 App Sandbox](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-009-r1-本地认证与-app-sandbox) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-008](./mobile-platform-decision-log.md#console-ios-v1-dec-008) | 必须替换 | [CONSOLE-AND-V1-DEC-008](./mobile-platform-decision-log.md#console-and-v1-dec-008) | iOS 延续 native display + device key 方向并适配移动 binding；Android 必须改用 Class 3 BiometricPrompt 与 hardware-backed Keystore key。 |
| [CONSOLE-MAC-V1-DEC-010 start/stop/repair/uninstall](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-010-startstoprepairuninstall) | 移动端明确不提供 | [CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 移动端明确不提供 | [CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | 手机无本地 daemon/node lifecycle；远端 Agent lifecycle 不能冒充本机 service start/stop/repair/uninstall。 |
| [CONSOLE-MAC-V1-DEC-011 平台体验与无障碍](../../clients/pc/docs/platforms/platform-decision-log.md#console-mac-v1-dec-011-平台体验与无障碍) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-014](./mobile-platform-decision-log.md#console-ios-v1-dec-014) | 移动平台适配 | [CONSOLE-AND-V1-DEC-014](./mobile-platform-decision-log.md#console-and-v1-dec-014) | 共享语义与品牌核心保留；VoiceOver/FKA/macOS shell 改为各移动平台的触控、AT、键盘、rotation 与 motion gate。 |

## 3. Linux v1 → iPhone / Android phone

下表严格保持 11 项 Linux 决策各一行。

| Linux 决策 | iOS 分类 | iOS 对应 DEC | Android 分类 | Android 对应 DEC | 原因 / 当前边界 |
|---|---|---|---|---|---|
| [CONSOLE-LNX-V1-DEC-001 支持范围](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-001-支持范围) | 必须替换 | [CONSOLE-IOS-V1-DEC-002](./mobile-platform-decision-log.md#console-ios-v1-dec-002) | 必须替换 | [CONSOLE-AND-V1-DEC-002](./mobile-platform-decision-log.md#console-and-v1-dec-002) | Ubuntu/GNOME/Wayland/x86_64/24 个月桌面矩阵不能外推到 iPhone 或 Android device/OEM/build 矩阵。 |
| [CONSOLE-LNX-V1-DEC-002 节点拓扑](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-002-节点拓扑) | 移动端明确不提供 | [CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 移动端明确不提供 | [CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | systemd system service、GNOME session broker 和 privileged helper 均无移动等价物。 |
| [CONSOLE-LNX-V1-DEC-003 所有权、账号与 claim](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-003-所有权账号与-claim) | 必须替换 | [CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004) | 必须替换 | [CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004) | UID/sudo/polkit 与 machine claim 改为 account-first 登录和 mobile device enrollment；OS 管理身份仍不授予产品角色。 |
| [CONSOLE-LNX-V1-DEC-004 `.deb` 与 A/B 更新](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-004-deb-与-ab-更新) | 必须替换 | [CONSOLE-IOS-V1-DEC-011](./mobile-platform-decision-log.md#console-ios-v1-dec-011)、[012](./mobile-platform-decision-log.md#console-ios-v1-dec-012) | 必须替换 | [CONSOLE-AND-V1-DEC-011](./mobile-platform-decision-log.md#console-and-v1-dec-011)、[012](./mobile-platform-decision-log.md#console-and-v1-dec-012) | `.deb`、dpkg ownership、A/B slot 与 polkit switch 不适用；改为各自商店签名、channel identity 与 rollout/update recovery。 |
| [CONSOLE-LNX-V1-DEC-005 更新阈值与 kill switch](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-005-更新阈值与-kill-switch) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-012](./mobile-platform-decision-log.md#console-ios-v1-dec-012) | 移动平台适配 | [CONSOLE-AND-V1-DEC-012](./mobile-platform-decision-log.md#console-and-v1-dec-012) | floor 只收窄客户端、不改变远端 Task/Effect 的原则保留；floor 维度与更新入口按移动 build/channel/device 适配。 |
| [CONSOLE-LNX-V1-DEC-006 不可信内容与 acquisition](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-006-不可信内容与-acquisition) | 必须替换 | [CONSOLE-IOS-V1-DEC-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009)、[010](./mobile-platform-decision-log.md#console-ios-v1-dec-010) | 必须替换 | [CONSOLE-AND-V1-DEC-009](./mobile-platform-decision-log.md#console-and-v1-dec-009)、[010](./mobile-platform-decision-log.md#console-and-v1-dec-010) | safe text/Markdown 保留；portal/local bundle acquisition 改为远端 authority ref-only acquisition。 |
| [CONSOLE-LNX-V1-DEC-007 Secret Service、锁屏与 session](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-007-secret-service锁屏与-session) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004)、[009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [CONSOLE-AND-V1-DEC-004](./mobile-platform-decision-log.md#console-and-v1-dec-004)、[009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | fail-closed、锁屏清敏感 UI、停 lease/write 与恢复后重认证保留；Secret Service carrier 必须适配移动 secure storage。 |
| [CONSOLE-LNX-V1-DEC-008 窗口、后台与通知](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-008-窗口后台与通知) | 必须替换 | [CONSOLE-IOS-V1-DEC-006](./mobile-platform-decision-log.md#console-ios-v1-dec-006)、[007](./mobile-platform-decision-log.md#console-ios-v1-dec-007) | 必须替换 | [CONSOLE-AND-V1-DEC-006](./mobile-platform-decision-log.md#console-and-v1-dec-006)、[007](./mobile-platform-decision-log.md#console-and-v1-dec-007) | GNOME session broker/freedesktop notification 改为移动生命周期和 APNs/FCM；无可靠后台监督。 |
| [CONSOLE-LNX-V1-DEC-009 polkit 与产品 R1](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-009-polkit-与产品-r1) | 必须替换 | [CONSOLE-IOS-V1-DEC-008](./mobile-platform-decision-log.md#console-ios-v1-dec-008) | 必须替换 | [CONSOLE-AND-V1-DEC-008](./mobile-platform-decision-log.md#console-and-v1-dec-008) | polkit/本机密码不适用；改为 native canonical display、device-bound signing key 与 authority 最终决定。 |
| [CONSOLE-LNX-V1-DEC-010 start/stop/repair/uninstall](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-010-startstoprepairuninstall) | 移动端明确不提供 | [CONSOLE-IOS-V1-DEC-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 移动端明确不提供 | [CONSOLE-AND-V1-DEC-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | 手机不安装或管理本地 CognitiveOS service/node；移动 App 的卸载也不能删除远端 authority 状态或 Effect。 |
| [CONSOLE-LNX-V1-DEC-011 GNOME 体验与无障碍](../../clients/pc/docs/platforms/platform-decision-log.md#console-lnx-v1-dec-011-gnome-体验与无障碍) | 移动平台适配 | [CONSOLE-IOS-V1-DEC-014](./mobile-platform-decision-log.md#console-ios-v1-dec-014) | 移动平台适配 | [CONSOLE-AND-V1-DEC-014](./mobile-platform-decision-log.md#console-and-v1-dec-014) | IA/Trust Strip/任务语义保留；GNOME/Orca/scaling gate 改为移动 AT、触控目标、横竖屏与 motion gate。 |

## 4. 能力 / 平台矩阵

本节按用户可见能力和平台责任交叉检查三桌面切片到两个 phone 切片的覆盖。

| 能力 / 平台面 | 桌面基线 | iOS 分类 | iOS 对应 DEC | Android 分类 | Android 对应 DEC | 对等边界 |
|---|---|---|---|---|---|---|
| 首要角色与权限落点 | Agent operator；入口随 authority projection/readiness | 直接复用 | [IOS-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 直接复用 | [AND-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | OS 用户、设备所有者或管理员均不自动获得产品角色。 |
| Console 非 authority | 桌面 GUI/broker/helper 都不是 authority | 直接复用 | [IOS-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 直接复用 | [AND-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | App 只展示 projection、收集 proposal；本地状态不 commit。 |
| Conversation / Task | 继续对话、创建、监督、纠偏、请求暂停 | 直接复用 | [IOS-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 直接复用 | [AND-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | 用户任务保留，所有写仍经远端 authority。 |
| 五个独立 authority lifecycle 域 + 独立远端 Runtime projection | Task、Loop、AgentExecution、Effect、Verification 各自独立；远端 Runtime 另作独立 projection | 直接复用 | [IOS-005](./mobile-platform-decision-log.md#console-ios-v1-dec-005) | 直接复用 | [AND-005](./mobile-platform-decision-log.md#console-and-v1-dec-005) | phone UI 可折叠呈现，但不得合并五个 authority lifecycle；Runtime stop 只改变远端 Runtime projection，不推进 Task 或 Effect。 |
| completion / candidate | current Verification + acceptance 才可显示完成 | 直接复用 | [IOS-016](./mobile-platform-decision-log.md#console-ios-v1-dec-016) | 直接复用 | [AND-016](./mobile-platform-decision-log.md#console-and-v1-dec-016) | model claim、remote completed、receipt、push、biometric 均不是完成证据。 |
| `OUTCOME_UNKNOWN` | 原 binding 对账；禁止盲重试 | 直接复用 | —（共享 Effect 核心语义；见 [iOS §10](./ios-product-design.md#10-offlineidempotency-与-outcome_unknown) / 相关 PRD） | 直接复用 | —（共享 Effect 核心语义；见 [Android §10](./android-product-design.md#10-offline幂等与-outcome_unknown) / 相关 PRD） | 关闭 App、停 Runtime 或重装 Agent 不闭合 unknown。 |
| multi-tenant / node | 桌面首版范围不同，authority 决定可发现性 | 移动平台适配 | [IOS-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004) | 移动平台适配 | [AND-004](./mobile-platform-decision-log.md#console-and-v1-dec-004) | 单活动账号可选择多个 tenant/node；切换使旧 watch/preview/binding 失效。 |
| Agent lifecycle | install、upgrade、rollback、uninstall 旅程 | 移动平台适配 | [IOS-010](./mobile-platform-decision-log.md#console-ios-v1-dec-010) | 移动平台适配 | [AND-010](./mobile-platform-decision-log.md#console-and-v1-dec-010) | 手机提交 proposal、监督远端 transaction；执行和 commit 在远端。 |
| Agent acquisition 来源 | 桌面可有 registry/local，Windows 还设计 URL/Git | 必须替换 | [IOS-010](./mobile-platform-decision-log.md#console-ios-v1-dec-010) | 必须替换 | [AND-010](./mobile-platform-decision-log.md#console-and-v1-dec-010) | 移动端仅 authority catalog/package ref；无 URL/Git/file package source。 |
| 本地 node / runtime | Windows Service、LaunchDaemon 或 systemd service | 移动端明确不提供 | [IOS-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 移动端明确不提供 | [AND-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | 手机不是 node、runtime、daemon、broker 或 final arbiter。 |
| background / supervision lease | 桌面 broker/tray 可维持合格监督 | 移动平台适配 | [IOS-006](./mobile-platform-decision-log.md#console-ios-v1-dec-006) | 移动平台适配 | [AND-006](./mobile-platform-decision-log.md#console-and-v1-dec-006) | 仅 active/RESUMED、unlocked、fresh 时续租；后台 API 只作 hint/resync。 |
| push / 系统通知 | per-user desktop broker + opaque handle | 必须替换 | [IOS-007](./mobile-platform-decision-log.md#console-ios-v1-dec-007) | 必须替换 | [AND-007](./mobile-platform-decision-log.md#console-and-v1-dec-007) | 改为 APNs/FCM；delivery/click 不是 truth，唯一 action 为打开 App。 |
| R1 app-owned canonical confirmation | authority risk floor + 结构化确认 | 移动平台适配 | [IOS-008](./mobile-platform-decision-log.md#console-ios-v1-dec-008) | 移动平台适配 | [AND-008](./mobile-platform-decision-log.md#console-and-v1-dec-008) | 当前 native display + device-bound signature 只是 app-owned canonical confirmation，不是 hardware trusted display；若要求抵御 compromised client，则 iOS/Android 两端 R1 均暂时阻断。本地生物识别只解锁 key。 |
| R2 / R3 | 解释并阻断，不降级 | 直接复用 | [IOS-001](./mobile-platform-decision-log.md#console-ios-v1-dec-001) | 直接复用 | [AND-001](./mobile-platform-decision-log.md#console-and-v1-dec-001) | 无聊天、通知、passcode/device credential 或 managed-policy 旁路。 |
| offline / last-good | 标 `as_of`，写禁用，不宣称暂停或完成 | 移动平台适配 | [IOS-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [AND-009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | 敏感 last-good 仅进程内；draft 不是离线控制队列。 |
| 内容 renderer | escaped text + allowlist Markdown；无 active content | 移动平台适配 | [IOS-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [AND-009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | 使用 native renderer；无 raw HTML/JS/iframe/bridge/remote image auto-load。 |
| deep link | 桌面外部链接/opaque routing 原则 | 移动平台适配 | —（无专属移动 DEC；见 [iOS §11](./ios-product-design.md#11-storage内容deep-link文件与隐私)） | 移动平台适配 | —（无专属移动 DEC；见 [Android §11](./android-product-design.md#11-storage内容deep-link文件与隐私)） | Universal Link/App Link 只带 opaque handle；打开后 reauth/resnapshot，不直接写。 |
| 普通数据文件 | 桌面 picker/path gate | 移动平台适配 | [IOS-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [AND-009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | fileImporter/SAF 仅选不可信数据上传；限制类型、大小和 scope。 |
| Agent package 文件 | 桌面可从本地签名 bundle 进入 acquisition | 移动端明确不提供 | [IOS-010](./mobile-platform-decision-log.md#console-ios-v1-dec-010) | 移动端明确不提供 | [AND-010](./mobile-platform-decision-log.md#console-and-v1-dec-010) | package/archive/executable 不得由手机接收、解压、扫描、执行或转发。 |
| secure storage / device key | Windows secure store、Keychain、Secret Service | 必须替换 | [IOS-004](./mobile-platform-decision-log.md#console-ios-v1-dec-004)、[009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 必须替换 | [AND-004](./mobile-platform-decision-log.md#console-and-v1-dec-004)、[009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | 采用移动 Keychain/Secure Enclave 或 Android Keystore/profile storage；无文件 fallback，restore 后 rebind。 |
| distribution / channel identity | installer、PKG、`.deb` 等桌面制品 | 必须替换 | [IOS-011](./mobile-platform-decision-log.md#console-ios-v1-dec-011) | 必须替换 | [AND-011](./mobile-platform-decision-log.md#console-and-v1-dec-011) | Public/managed 渠道分别隔离 bundle/application identity、push、links、binding 与发布记录。 |
| app update / security floor | desktop updater、PKG 或 A/B；floor fail closed | 移动平台适配 | [IOS-012](./mobile-platform-decision-log.md#console-ios-v1-dec-012) | 移动平台适配 | [AND-012](./mobile-platform-decision-log.md#console-and-v1-dec-012) | 更新通过商店/MDM；floor 可收窄 lease/R1/lifecycle/write，但不改变远端 authority state。 |
| privacy / capture / backup | 敏感 projection 不主动落盘 | 移动平台适配 | [IOS-009](./mobile-platform-decision-log.md#console-ios-v1-dec-009) | 移动平台适配 | [AND-009](./mobile-platform-decision-log.md#console-and-v1-dec-009) | 适配 app switcher/Recents、pasteboard/clipboard、capture、backup/restore；不宣称绝对防截。 |
| telemetry / diagnostics | 最小诊断且不代替 authority evidence | 移动平台适配 | [IOS-015](./mobile-platform-decision-log.md#console-ios-v1-dec-015) | 移动平台适配 | [AND-015](./mobile-platform-decision-log.md#console-and-v1-dec-015) | 仅第一方 content-free telemetry；diagnostics 字段预览后显式上传。 |
| accessibility / motion | 桌面 Narrator/VoiceOver/Orca 与 reduced motion | 移动平台适配 | [IOS-014](./mobile-platform-decision-log.md#console-ios-v1-dec-014) | 移动平台适配 | [AND-014](./mobile-platform-decision-log.md#console-and-v1-dec-014) | iOS 真机 AT/44pt 与 Android TalkBack/48dp 等分别验收；motion 不作为事实。 |
| form factors / navigation | 桌面窗口、toolbar/sidebar、三栏或可折叠侧栏 | 必须替换 | [IOS-002](./mobile-platform-decision-log.md#console-ios-v1-dec-002)、[005](./mobile-platform-decision-log.md#console-ios-v1-dec-005) | 必须替换 | [AND-002](./mobile-platform-decision-log.md#console-and-v1-dec-002)、[005](./mobile-platform-decision-log.md#console-and-v1-dec-005) | v1 均 phone-only、竖屏为主且横屏完整；不把桌面布局压缩成卡片墙，也不暗示 tablet/foldable。 |

## 5. 桌面本地机制的移动处置

本表明确哪些桌面本地机制必须被移动机制取代，哪些在移动端不建立对应能力。

| 桌面机制 | 桌面含义 | iOS 分类 | iOS 处置 | Android 分类 | Android 处置 |
|---|---|---|---|---|---|
| machine-local node | 本机共享 CognitiveOS node | 移动端明确不提供 | 仅选择并连接远端 tenant/node。 | 移动端明确不提供 | 仅选择并连接远端 tenant/node。 |
| Windows Service / LaunchDaemon / systemd service | 独立于 GUI 的 machine service | 移动端明确不提供 | App lifecycle 不承载 daemon。 | 移动端明确不提供 | Activity、WorkManager、FGS 不承载 node daemon。 |
| per-user broker | session 通知与监督 carrier | 必须替换 | App scene + APNs opaque hint；后台不续租。 | 必须替换 | Activity + FCM opaque hint；后台/force-stop 不续租。 |
| privileged helper | UAC、Authorization Services 或 polkit 跨 OS admin 边界 | 移动端明确不提供 | 无本地 node 安装/修复 helper。 | 移动端明确不提供 | 无本地 node 安装/修复 helper。 |
| OS admin / machine claim | 管理员安装并发起本机 node claim | 必须替换 | upstream account + device enrollment；MDM 只能收窄。 | 必须替换 | upstream account + install/profile binding；device/profile owner 只能收窄。 |
| TOFU | loopback 本机节点首次身份固定 | 必须替换 | 受信 web auth、authority scope 与 device enrollment challenge。 | 必须替换 | Custom Tab/OIDC、verified return path 与 device enrollment challenge。 |
| tray / menu extra / linger broker | 关闭窗口后维持可发现后台状态 | 移动端明确不提供 | 无 tray/menu-extra 等价物；返回 App 后 resnapshot。 | 移动端明确不提供 | 无 tray/linger 等价物；force-stop 后只可显式重开恢复。 |
| PKG | macOS signed/notarized 安装、更新、repair | 必须替换 | App Store/TestFlight/ABM/MDM channel；App 不自更新 executable。 | 必须替换 | Google Play/Managed Play AAB channel。 |
| `.deb` | Linux 原生包与 dpkg ownership | 必须替换 | Apple 分发链，不接收 `.deb`。 | 必须替换 | Play AAB/update chain，不接收 `.deb`。 |
| A/B slots | Linux 本地 payload 原子切换与回退 | 必须替换 | 商店 build 更新与 signed floor recovery；无 App 内 binary slot switch。 | 必须替换 | Play rollout/update recovery；无 Console 自管 A/B payload。 |
| local service start/stop/repair/uninstall | OS admin 控制本机 node 生命周期 | 移动端明确不提供 | App 退出/卸载不停止远端 node、Task 或 Effect。 | 移动端明确不提供 | force-stop/卸载不停止远端 node、Task 或 Effect。 |
| URL/Git/local Agent package | 桌面 acquisition source | 移动端明确不提供 | 只提交 authority catalog/package ref。 | 移动端明确不提供 | 只提交 authority catalog/package ref。 |
| desktop secure-store carrier | Windows store、desktop Keychain、Secret Service | 必须替换 | mobile Keychain/Secure Enclave、Data Protection 与不备份 binding。 | 必须替换 | profile-scoped Keystore/storage、backup/D2D exclusion 与 rebind。 |

## 6. 当前阻断与证据边界

### 6.1 当前固定声明

| 维度 | 当前声明 | 不得推断 |
|---|---|---|
| Machine contract | 通用合同仅部分覆盖；移动 account/device/push/lease/R1/floor/revoke carrier 仍未闭合。 | 产品 DEC 或本文映射不使移动 carrier 成为已登记合同。 |
| Implementation | **implementation not provided**；iOS/Android source slice 均记为 `not-implemented`。 | 文档完成不表示客户端、backend carrier、商店集成或 native security flow 已实现。 |
| Mobile evidence | **mobile evidence none**。 | PoC、模拟器/emulator、真机、push、商店、MDM、security、a11y 均不能写成已通过。 |
| Existing vectors | **76 vectors all `not-run`**。 | vector 存在、被枚举或被文档引用不表示测试已执行。 |
| Profile | **Profile not implemented**。 | 产品方向 accepted 不表示 iOS 或 Android Console Profile 已符合。 |

### 6.2 禁止外推

- iOS 证据不得外推 Android；Android 证据也不得反向外推 iOS。
- iOS simulator 或 Android emulator 证据不得外推真机的 secure key、biometric、push、锁屏、backup、lifecycle 或商店行为。
- Pixel 的任何结果不得外推 Samsung；一个 Samsung 型号、unlocked firmware 或单一 carrier build 也不得外推其他 Samsung/carrier 组合。
- 单一 OS、设备、app build、push environment、Public channel 或 managed channel 的结果不得扩张到其他矩阵项。
- Apple/Google 平台事实、商店审核结果、APNs/FCM receipt、通知点击与 device integrity signal 均不得外推为 CognitiveOS authority truth、authorization、completion 或 Profile evidence。

## 7. 维护与自检规则

1. Windows 逐项映射固定为 17 行，macOS 为 11 行，Linux 为 11 行；源决策变化时只重评对应行，不静默改写既有 ID、anchor 或含义。
2. 分类列只能使用 §0 的五类，每个平台每行恰好一个类别。
3. 移动 DEC 链接只表达产品方向对应；缺 carrier、实现或 evidence 时继续按 §6 声明。
4. 不因追求视觉或功能一致而扩大移动后台、文件、acquisition、R1、分发或 form-factor 范围。
5. 本文不新增或推断 requirement、error code、schema、transition、vector 或 Profile；任何未来 normative 变更必须走独立契约流程。
