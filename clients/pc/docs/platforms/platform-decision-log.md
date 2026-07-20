# CognitiveOS Console 桌面平台产品决策记录

> 类别：informative product decisions
>
> 日期：2026-07-20
> 状态：accepted product direction / machine contract and implementation blocked

本文记录 macOS v1 与 Linux v1 已确认的产品决策。决策存在不表示 machine contract 已登记、实现已提供、测试已执行或 Profile 已符合。Windows v1 的 `CONSOLE-V2-*` ID 和旧 anchor 保持不变。

## 1. 共同解释规则

- `accepted`：产品方向已确认。
- `blocked`：产品方向成立，但 machine contract、实现、PoC 或证据不足，不能发布或声称可用。
- `planned`：明确后置，不属于目标平台 v1 GA。
- Console、broker、helper 均不得拥有 authority/IdP/node/final-arbiter 权力。
- 平台首版只执行 authority 判定的 R0/R1；R2/R3 不降级。
- 平台证据只对其明确 OS/build/WebView/daemon/helper 版本有效。

## 2. macOS v1

### `CONSOLE-MAC-V1-DEC-001` 支持范围

- 状态：`accepted / blocked by GA evidence`
- 发布架构为 Universal 2（Apple silicon + Intel）。
- 设计候选最低版本为 macOS 14。
- GA 仅保留查询时仍获安全修复、并满足 WKWebView/helper/security floor 的版本；必要时阻断旧 OS 或 Intel。
- 不宣称 Apple 提供固定 macOS 生命周期承诺。

### `CONSOLE-MAC-V1-DEC-002` 节点拓扑

- 状态：`accepted / machine contract missing`
- 一个 machine-wide、独立于 GUI 和登录会话的低权限节点 daemon。
- 每个登录用户拥有独立 broker。
- 只有极小安装/更新/恢复 helper 可跨越 OS 管理员边界。
- GUI、broker、helper 都不是 CognitiveOS node 或 authority。

### `CONSOLE-MAC-V1-DEC-003` 所有权、账号与 claim

- 状态：`accepted / machine contract missing`
- 账号与 tenant 由 upstream IdP 和 authority 管理；OS 用户/管理员不自动获得产品权限。
- OS admin 可安装、修复并发起 claim。
- daemon 生成 node key；短期 claim 固定 node key、build 和 tenant digest。
- 只有已登录 CognitiveOS Owner/tenant admin 可在 digest-bound 流程中确认节点归属。
- key 丢失、账号变化、机器转让或完整性失败进入 quarantine；撤销旧绑定后重新 claim，不静默沿用 node ID。

### `CONSOLE-MAC-V1-DEC-004` 分发与更新入口

- 状态：`accepted / implementation not started`
- GA 唯一制品是 Developer ID 签名并公证的 PKG。
- 官网手工安装与企业 MDM 复用同一 PKG；MAS/DMG 为非 GA。
- Console 只拉取 threshold-signed recommended/minimum metadata，提示并链接，不自动下载或提权安装。
- MDM 可接管或禁用 Console 更新检查。

### `CONSOLE-MAC-V1-DEC-005` 更新阈值与回退

- 状态：`accepted / machine contract missing`
- recommended minimum 只警告；signed security floor 可选择性 fail closed。
- 低于 floor 时停止不可信富内容、acquisition、bootstrap/R1、受保护写和 lease 续期；只保留安全诊断、导出和更新修复入口。
- Console 不得因 floor 触发而擅自终止 authority 已接受任务。
- 回退只允许到仍高于 floor 的签名版本；authority state 不从本机回滚。

### `CONSOLE-MAC-V1-DEC-006` 不可信内容与 acquisition

- 状态：`accepted / security PoC blocked`
- v1 主界面只呈现转义纯文本和 allowlist Markdown；剥离 raw HTML/script。
- 链接不自动抓取；富内容预览为 `planned/blocked`。
- `WKContentWorld`、CSP、Tauri isolation 不是独立安全边界；WKWebView 仍受 security floor/kill switch。
- v1 acquisition 只接受受信 registry ID 和用户经系统 picker 选择的本地签名 bundle。
- 任意 URL/Git/private repo 为 `planned`；所有输入仍执行 digest、预算、路径/symlink/archive/TOCTOU 校验且禁止 ambient credentials。

### `CONSOLE-MAC-V1-DEC-007` secure storage、锁屏与用户切换

- 状态：`accepted / machine contract missing`
- Keychain 缺失、锁定或用户取消时禁止明文/静默文件 fallback；清除或不刷新 token。
- 仅保留非敏感只读 projection、诊断和修复；停止 lease renewal、acquisition、bootstrap/R1 和受保护写。
- broker 报告锁屏；只有 authority 可维持或签发有明确 expiry 的 signed lease，Console 不本地延长。
- 锁屏 teardown 敏感 UI；用户切换严格隔离 session，恢复后重认证/resnapshot。

### `CONSOLE-MAC-V1-DEC-008` 窗口、退出与通知

- 状态：`accepted / implementation not started`
- 关闭窗口仅关闭 UI；经明示 opt-in 的 broker 可继续脱敏通知和 authority 允许的监督。
- “退出 Console 并停止监督”才停止 GUI、broker 和 lease renewal；machine node 与已接受任务不停止。
- 使用普通 Dock app，可提供可选 menu extra。
- 通知只有一个 action：携一次性 opaque handle 打开 Console；随后重认证/resnapshot。
- 锁屏只显示通用提示；broker 缺失时事项留在 authority，禁止跨用户转发。

### `CONSOLE-MAC-V1-DEC-009` R1 本地认证与 App Sandbox

- 状态：`accepted / signed-display and sandbox contracts blocked`
- Touch ID 只解锁 device private key；可信 native sheet 显示 operation digest、nonce、session 和 expiry，并用该 key 签名。
- authority 校验后才可批准 R1；Touch ID 单独不构成 authority。
- Secure Enclave 仅在兼容硬件上作为可选 key protection；passkey 仅用于 upstream IdP。
- 缺少 Touch ID/Keychain 时走安全重认证。
- Hardened Runtime 必需；GUI App Sandbox 是 GA 目标和 PoC gate，使用最小 entitlements、XPC 与 daemon 通信、picker/security-scoped bookmark 访问文件。PoC 不成立则保持 blocked 并重新决策，不静默降级。

### `CONSOLE-MAC-V1-DEC-010` start/stop/repair/uninstall

- 状态：`accepted / lifecycle contract missing`
- 安装后节点以 unclaimed quarantine 启动，只开放本地 claim；authority 绑定后才 enable boot start。
- broker 登录启动必须由 onboarding 或 managed policy 明示 opt-in，可关闭；关闭即停止续租。
- 普通 stop 先由 authority drain、拒绝新任务并到安全点，再由 OS admin 停 daemon。
- 紧急强停允许，但在途 Effect 进入 `OUTCOME_UNKNOWN`，禁止盲重试，重连后按 Effect/Verification/Event 对账。
- repair 只接受高于 floor 的签名/公证 PKG；projection 可重建，node key 仅在完整性成立时保留。
- uninstall 不删除 authority 侧状态或未决 Effect 证据。

### `CONSOLE-MAC-V1-DEC-011` 平台体验与无障碍

- 状态：`accepted / test not executed`
- 共享任务语义、IA、Trust Strip 和品牌核心，不追求像素一致。
- 使用 macOS 菜单、toolbar/sidebar、Dock/menu extra 和 Command 快捷键惯例。
- GA gate 必须实测 VoiceOver、Full Keyboard Access、Increase Contrast、Reduce Motion 的核心旅程。
- 未执行前只能标 `planned/blocked`，不得称支持。

## 3. Linux v1

### `CONSOLE-LNX-V1-DEC-001` 支持范围

- 状态：`accepted / blocked by GA evidence`
- GA 仅 Ubuntu 24.04 LTS x86_64、stock GNOME、Wayland、原生 `.deb`。
- 产品支持期从 GA 起 24 个月，且不得超过 Ubuntu/WebKitGTK/security floor 的有效期。
- X11、KDE、其他 distro/arch、RPM、Flatpak、AppImage 均为 `planned/blocked`。
- 产品不得简称为“支持 Linux”而省略该矩阵。

### `CONSOLE-LNX-V1-DEC-002` 节点拓扑

- 状态：`accepted / machine contract missing`
- 一个 machine-wide systemd system service，运行于专用低权限 service identity。
- 每个登录 GNOME session 使用独立 broker；broker 不启用 linger。
- 极小 privileged helper 只处理安装、更新、恢复。
- Console、broker、helper 都不是 authority、IdP 或 node。

### `CONSOLE-LNX-V1-DEC-003` 所有权、账号与 claim

- 状态：`accepted / machine contract missing`
- upstream IdP 与 authority 管账号/tenant；Linux UID、wheel/sudo 或 polkit admin 不自动获得产品角色。
- OS admin 可安装、修复和发起 claim。
- daemon 生成 node key；短期 claim 固定 key/build/tenant digest，只有已登录 Owner/tenant admin 可确认。
- key 丢失、账号变化、机器转让或完整性失败进入 quarantine 并 revoke/reclaim。

### `CONSOLE-LNX-V1-DEC-004` `.deb` 与 A/B 更新

- 状态：`accepted / updater implementation not started`
- 官网提供 `.deb`，项目不运营 APT repository。
- `.deb` 由 threshold-signed release manifest/detached signature 固定 digest、target、version 和 floor；不把 HTTPS 下载或手工 `dpkg` 当作来源证明。
- `.deb` 只拥有稳定 bootstrap/updater/systemd unit；签名 payload 安装到版本化 A/B 槽并原子切换 `current`，不得改写 dpkg-owned 文件。
- 可自动检查和暂存；切换必须同时满足 authority drain 成功、无 unknown Effect、用户/管理员维护窗和 OS admin/polkit 边界。
- 失败只回退到仍高于 floor 的上一槽。

### `CONSOLE-LNX-V1-DEC-005` 更新阈值与 kill switch

- 状态：`accepted / machine contract missing`
- recommended minimum 只警告；threshold-signed security floor 可选择性 fail closed。
- 低于 floor 时关闭不可信富内容、acquisition、bootstrap/R1、受保护写和 lease renewal，只保留诊断、导出和更新修复。
- Console 不自动停止 authority 已接受任务。
- WebKitGTK 实际 distro package/build 必须在 floor allowlist；API 4.1 不等于安全版本。

### `CONSOLE-LNX-V1-DEC-006` 不可信内容与 acquisition

- 状态：`accepted / security PoC blocked`
- 主界面只允许转义纯文本和 allowlist Markdown；raw HTML/script 删除，链接不自动抓取。
- 富内容预览为 `planned/blocked`；CSP/Tauri isolation 不构成独立边界。
- acquisition 仅受信 registry ID 和用户通过 XDG portal/system picker 选择的本地签名 bundle。
- URL/Git/private repo 为 `planned`；路径、symlink、archive、TOCTOU、digest、预算和无 ambient credential gate 全部适用。

### `CONSOLE-LNX-V1-DEC-007` Secret Service、锁屏与 session

- 状态：`accepted / machine contract missing`
- Secret Service 缺失、锁定或 prompt 取消时不使用明文/文件 fallback；清除或不刷新 token。
- 保留非敏感只读 projection、诊断/修复；停止 lease renewal、acquisition、bootstrap/R1 和受保护写。
- broker 报告 session lock；signed lease 的 grace/expiry 只能由 authority 决定。
- session switch/logout 清理敏感 UI 和 token；恢复后重认证/resnapshot。

### `CONSOLE-LNX-V1-DEC-008` 窗口、后台与通知

- 状态：`accepted / implementation not started`
- 关闭窗口仅关闭 UI；明确 opt-in 的 broker 可继续监督和通知。
- 显式“退出 Console 并停止监督”停止 GUI、broker 和续租，不停止 machine node 或已接受任务。
- GNOME/Wayland GA 路径不依赖 tray；没有 tray 时功能完整。
- freedesktop notification 只携通用提示和一次性 opaque handle；唯一 action 是打开 Console 后重认证/resnapshot。
- broker 缺失时事项留在 authority。

### `CONSOLE-LNX-V1-DEC-009` polkit 与产品 R1

- 状态：`accepted / R1 display contract blocked`
- polkit 只授权安装、更新、恢复、stop service 等 OS 边界。
- 产品 R1 使用可信确认页、upstream reauth 和 digest-bound authority decision。
- polkit temporary authorization、Linux 密码或通知 action 不构成 CognitiveOS R1。

### `CONSOLE-LNX-V1-DEC-010` start/stop/repair/uninstall

- 状态：`accepted / lifecycle contract missing`
- 安装后 daemon 进入 unclaimed quarantine，仅开放 claim；authority 绑定后才 enable boot start。
- broker 登录启动由 onboarding/managed policy opt-in，可关闭且关闭停止续租。
- 普通 stop 先 authority drain，再由 OS admin/polkit 停 system service。
- 紧急强停令在途 Effect 成为 `OUTCOME_UNKNOWN`；重连后对账。
- repair 只使用高于 floor 的签名 `.deb` 或 A/B previous slot；authority state 不从本机回滚。
- uninstall 不删除 authority 侧状态/证据。

### `CONSOLE-LNX-V1-DEC-011` GNOME 体验与无障碍

- 状态：`accepted / test not executed`
- 共享 IA、任务语义、Trust Strip 和品牌核心；采用 GNOME header bar、system dialog 和 portal。
- 不依赖 tray，不承诺 KDE/X11 表现。
- GA gate 必须在 Ubuntu 24.04 GNOME/Wayland 实测 Orca、纯键盘、高对比、100/200% scaling 与 app-level reduced-motion override。
- 未执行前保持 `planned/blocked`。

## 4. 共同机器合同缺口

以下方向均未因本决策记录而成为“规范已登记”：

- machine node service/helper/broker IPC 与 peer identity；
- node claim、upstream IdP session、owner/tenant binding；
- supervision signed lease、lock grace、broker eligibility；
- threshold-signed recommended/security-floor metadata；
- platform update drain/A-B switch/rollback/repair；
- platform notification opaque-handle routing；
- Touch ID digest signing与可信 native display；
- WebKit floor/kill-switch carrier；
- platform acquisition broker 和本地 bundle admission。

这些缺口只能由未来 Lane-CTR/CFR/KRN/RUN 合同与实现流程闭合，Lane-CON 文档不得代替。
