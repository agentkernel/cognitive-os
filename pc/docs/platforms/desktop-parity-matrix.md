# CognitiveOS Console 桌面平台 parity matrix

> 类别：informative product comparison
>
> 日期：2026-07-20
> 状态：Windows/macOS/Linux 均为产品设计；本表不构成实现或跨平台符合性声明
>
> Agent Hub：第三方 Agent Direct Takeover 的桌面接管能力差异见 [agent-hub-platform-parity.md](../../../agent-hub/docs/platforms/agent-hub-platform-parity.md)，为本表补充，不改写此处条目。

分类：

- **直接复用**：用户价值和治理语义不变；
- **需平台适配**：语义不变，OS surface、生命周期或交互不同；
- **必须替换**：Windows 机制不得移植；
- **暂时阻断**：缺 machine contract、实现、PoC 或已执行证据。

Windows v1 的 `CONSOLE-V2-*`、legacy `CONSOLE-PRD-*`/`A-*` 映射，以及 `PRODUCT-DESIGN.md` 的 §17/§20.3 anchors 均保持原义；本表不重编号或静默改写它们。

## 1. 产品与治理语义

| 能力 | Windows v1 anchor | macOS v1 | Linux v1 | 结论 |
|---|---|---|---|---|
| 首要 persona | `CONSOLE-V2-DEC-001` Agent operator | 相同 | 相同 | 直接复用 |
| 核心任务 | Conversation/Task、监督纠偏、Agent lifecycle | 相同 | 相同 | 直接复用 |
| 用户任务语言 | IA 用户标签 + machine details | 相同，macOS shell 适配 | 相同，GNOME shell 适配 | 直接复用语义、需平台文案适配 |
| Console 非 authority | `PRD-001/005` | GUI/broker/helper 均非 authority | GUI/broker/helper 均非 authority | 直接复用 |
| 状态域分离 | `PRD-006..009` | 相同 | 相同 | 直接复用；carrier 暂时阻断 |
| completion | candidate/current verification/acceptance | 相同 | 相同 | 直接复用 |
| unknown outcome | 原 key reconcile/quarantine | 相同 | 相同 | 直接复用 |
| risk floor | authority；R0/R1；R2/R3 阻断 | 相同 | 相同 | 直接复用 |
| 四态声明 | specified/implementation/test/Profile | 相同 | 相同 | 直接复用 |
| IA | 工作、任务、Agent、收件箱、记录、系统 | 相同分组，macOS 导航 shell | 相同分组，GNOME shell | 需平台适配 |
| Trust Strip/Flow Thread | `CMP-004/005` | 保持语义 | 保持语义 | 直接复用；视觉适配 |

## 2. 平台范围与组件

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类 |
|---|---|---|---|---|
| GA support matrix | Windows 桌面，具体发布 floor 待 gate | macOS 14+ 候选、Universal 2，GA 动态安全 floor | Ubuntu 24.04 x86_64 stock GNOME/Wayland，24 个月上限 | 必须分别声明 |
| machine node | Windows Service | low-privilege LaunchDaemon/service | low-privilege systemd system service | Windows 机制必须替换 |
| per-user broker | SID/session notification broker | 每登录用户 LaunchAgent/broker | 每 GNOME session user broker，无 linger | 需平台适配 |
| privileged helper | UAC/installer/recovery helper | Authorization Services/PKG helper | polkit/package/A-B helper | 必须替换 |
| peer identity | Windows token/SID/signing | XPC code-signing + OS session + authority session | D-Bus/UID/session + authority session | 必须替换 |
| owner/tenant | Windows 本地账号草案 | upstream IdP + authority owner/tenant | upstream IdP + authority owner/tenant | Windows 身份方案必须替换 |
| GUI isolation | WebView2 PoC | Hardened Runtime + App Sandbox target | native `.deb` app + WebKitGTK floor | 平台分别阻断 |

## 3. 安装、claim 与启动

| Journey | Windows v1 | macOS v1 | Linux v1 | 分类/状态 |
|---|---|---|---|---|
| 安装制品 | 未冻结 installer/channel | Developer ID signed+notarized PKG | 官网 `.deb`，无 APT repo | 必须替换；platform PoC blocked |
| 企业分发 | Windows enterprise channel 待定 | MDM 复用同一 PKG | 企业自有软件分发可复用 `.deb` digest | 需平台适配 |
| 初始 node | Service/bootstrap 设计 | unclaimed quarantine | unclaimed quarantine | 复用安全语义 |
| node key | endpoint key 草案 | daemon-generated key | daemon-generated key | 语义复用、storage 适配 |
| claim actor | OS admin/UAC 与 Owner 分离 | OS admin 发起，upstream Owner 确认 | polkit admin 发起，upstream Owner 确认 | 直接复用身份正交原则 |
| claim binding | endpoint key/SID secret 草案 | key/build/tenant digest/nonce/expiry | key/build/tenant digest/nonce/expiry | Windows carrier 必须替换 |
| boot start | Windows Service 安装后策略待定 | authority binding 后 enable | authority binding 后 enable | 平台适配；contract missing |
| broker autostart | tray/notification setup | onboarding/managed opt-in | onboarding/managed opt-in，无 linger | 平台适配 |

## 4. stop、update、rollback、repair、uninstall

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类/状态 |
|---|---|---|---|---|
| normal stop | authority closure + UAC Service stop 方向 | authority drain → OS admin helper stop | authority drain → polkit stop | 复用语义、替换 OS surface |
| emergency stop | 不能冒充 Task/Effect closure | unknown 后 reconcile | unknown 后 reconcile | 直接复用 |
| update discovery | 渠道待决 | threshold metadata，仅提示/链接 | threshold metadata，可 stage | 需平台适配 |
| update install | Windows package/update contract missing | 用户 PKG/MDM；Console 不下载/提权 | A/B stage + maintenance + polkit atomic switch | 必须替换；blocked |
| recommended minimum | 尚未冻结 | warning only | warning only | 直接复用 |
| signed security floor | protocol/security floor 原则 | 选择性 client fail closed | 选择性 client fail closed | 直接复用语义、carrier missing |
| floor kill switch | schema/security mismatch fail closed | safe text/diagnostics/export/update only | safe text/diagnostics/export/update only | 直接复用 |
| rollback | installation/rollback point 方向 | signed/notarized version above floor | previous A/B slot above floor | 平台适配 |
| authority state | 不从客户端恢复 | 不随 binary rollback | 不随 slot rollback | 直接复用 |
| repair | installer/Admin CLI 方向 | signed PKG；key integrity gate | signed `.deb`/previous slot；key gate | 必须替换 |
| uninstall | 不删 Effect/audit | 同一原则 | 同一原则 | 直接复用语义 |

## 5. 窗口、后台与通知

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类 |
|---|---|---|---|---|
| close window | 默认进入 tray | 仅关闭 UI | 仅关闭 UI | Windows 体验不得直接复用 |
| background discoverability | tray | Dock + optional menu extra + Settings | GNOME Settings/System/Inbox；无 tray | 必须平台适配 |
| explicit exit | “退出并请求暂停”方向 | “退出 Console 并停止监督”；node/task 不停 | 同 macOS；system service/task 不停 | 共同语义已更新，OS surface 适配 |
| broker continuation | tray mode | 明示 opt-in | 明示 opt-in，无 linger | 需平台适配 |
| notification transport | Windows per-user broker/AUMID | per-user broker + UNUserNotificationCenter | per-user broker + freedesktop notifications | 必须替换 |
| notification action | 仅 opaque open handle | 仅 opaque open handle | 仅 opaque open handle；action capability 可缺失 | 直接复用安全语义 |
| lock notification | 脱敏 | 通用提示，无 control action | 通用提示，无 control action | 直接复用 |
| broker absent | authority 保留事项 | authority 保留事项 | authority 保留事项 | 直接复用 |
| user switching | SID/session 隔离 | fast-user-switch session 隔离 | UID/logind session 隔离 | 需平台适配 |

## 6. secure storage 与 session

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类/状态 |
|---|---|---|---|---|
| user secret store | Windows secure storage PoC | Keychain | Secret Service | 必须替换 |
| machine node key | contract missing | daemon storage PoC | system-service storage/TPM PoC | 暂时阻断 |
| locked/missing/cancel | 禁敏感写方向 | 无 fallback；停止 token/lease/R1/write | 无 fallback；停止 token/lease/R1/write | 直接复用 fail-closed |
| lock grace | supervision lease missing | authority-signed expiry only | authority-signed expiry only | contract missing |
| unlock recovery | reauth/resnapshot | reauth/resnapshot | reauth/resnapshot | 直接复用 |
| persistent sensitive projection | 禁应用主动持久化 | 同一原则，收窄 WebKit/cache/swap | 同一原则，收窄 WebKitGTK/cache/swap | 直接复用 |

## 7. renderer 与 acquisition

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类/状态 |
|---|---|---|---|---|
| default content | 设计要求不可信隔离 | escaped text + allowlist Markdown | escaped text + allowlist Markdown | 收紧后直接复用 |
| raw HTML/script | 独立 renderer 或 safe subset | v1 blocked | v1 blocked | 暂时阻断 |
| native bridge | data renderer 无 management IPC | untrusted view 无 XPC | untrusted view 无 D-Bus/native IPC | 直接复用边界、平台适配 |
| framework controls | WebView2 isolation PoC | WKContentWorld/CSP/Tauri isolation 非边界 | CSP/Tauri isolation 非边界 | 直接复用结论 |
| Web engine floor | WebView2 security floor | WKWebView/macOS floor | distro WebKitGTK build/backport floor | 必须分别验证 |
| acquisition sources | Windows: registry/URL/Git/local 设计 | registry ID + picker local signed bundle | registry ID + portal local signed bundle | mac/Linux 收窄 |
| URL/Git/private | Windows v1 设计允许受治理获取 | planned/blocked | planned/blocked | 平台范围不同 |
| file access | Windows picker/UNC/path gates | security-scoped bookmark + file identity | portal URI + file identity | 必须替换 |
| package gates | signature/provenance/digest/budget/sandbox | 相同 | 相同 | 直接复用 |
| ambient credentials | 禁止 | 禁止 | 禁止 | 直接复用 |

## 8. R1 与可信确认

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类/状态 |
|---|---|---|---|---|
| R1 display | structured system card/number match policy | trusted native sheet + operation digest | trusted confirmation page + upstream reauth | 需平台适配；contract blocked |
| local OS auth | UAC 不等于 R1 | Touch ID 只解锁 device key | polkit 只处理 OS boundary | 直接复用身份正交原则 |
| passkey | future upstream | upstream IdP only | upstream IdP only | 直接复用 |
| digest binding | proposal/version/risk/budget | + nonce/session/expiry device signature | upstream reauth + authority decision | carrier missing |
| notification approval | 禁止 | 禁止 | 禁止 | 直接复用 |
| R2/R3 | 无执行入口 | 无执行入口 | 无执行入口 | 直接复用 |

## 9. accessibility 与 motion

| 能力 | Windows v1 | macOS v1 | Linux v1 | 分类/证据 |
|---|---|---|---|---|
| screen reader | Narrator | VoiceOver | Orca/AT-SPI | 必须平台实测；none |
| keyboard | Windows keyboard | Full Keyboard Access/Command | GNOME keyboard | 必须平台实测；none |
| contrast | Windows High Contrast | Increase Contrast | GNOME High Contrast | 必须平台实测；none |
| scaling | 200/400% 计划 | OS text/display + window resizing | 100/200% GA matrix | 必须平台实测；none |
| reduced motion | Windows setting | Reduce Motion | app override；系统 signal 仅辅助 | 需平台适配；none |
| signature motion | Flow Thread authority event | 静态等价 | 静态等价 | 直接复用语义 |
| no motion-as-fact | required | required | required | 直接复用 |

## 10. 当前阻断汇总

共同阻断：

- Console 依赖组 1/2/7 与 M5 出口未交付；
- supervision lease、claim、readiness、notification、platform update 等 machine contract 未登记；
- Console implementation 未提供；
- 全局 84 个向量中 46 `pass` / 38 `not-run`，但桌面平台产品测试 evidence 仍为 `none`；
- Tauri/原生 shell 技术 ADR 未批准。

macOS 特有阻断：

- Universal 2、App Sandbox/XPC/helper/PKG/MDM；
- Touch ID digest signing/native display；
- Keychain/lock/switch-user；
- WKWebView floor/kill switch；
- VoiceOver/FKA/Contrast/Motion。

Linux 特有阻断：

- Ubuntu 24.04 GNOME/Wayland 唯一矩阵；
- systemd/D-Bus/polkit/Secret Service；
- `.deb`/A-B/dpkg ownership；
- WebKitGTK distro security floor；
- 无 tray journey 与 Orca/a11y。

## 11. 维护规则

- Windows 决策变化先更新 Windows 文档，再评估 parity；不得为追求一致而削弱任一平台安全边界。
- macOS/Linux 事实变化只更新对应平台和本表，不外推。
- support matrix、security floor、官方 URL 每次 release review 重新核实。
- 未执行 oracle 始终写 `none/not-run`；不使用“已支持”“已实现”。
