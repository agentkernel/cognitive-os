# CognitiveOS Console Linux v1 产品设计

> 类别：informative product design
>
> 决策状态：accepted product direction
>
> 交付状态：`planned / implementation not started / test not executed / Profile not implemented`
> 官方资料查询日期：2026-07-20

本文的“Linux v1”严格指 Ubuntu 24.04 LTS x86_64、stock GNOME、Wayland、原生 `.deb`。任何省略该矩阵而写“支持 Linux”的表述均不成立。本文不新增 CognitiveOS machine contract；平台专属行为在合同、实现和证据闭合前保持 `unregistered/planned/blocked`。

## 0. 事实与状态口径

- **平台事实**：仅来自 §15 官方来源 ledger，查询日期固定为 2026-07-20。
- **产品决策**：使用 `CONSOLE-LNX-V1-DEC-*`，不进入 normative registry。
- **产品要求**：使用 `CONSOLE-LNX-V1-PRD-*`，只作产品追踪。
- **Machine contract**：只有 registry 中真实存在的 `REQ-*` 才称已登记；systemd/polkit/claim/A-B 等缺口标 `missing`。
- **Implementation**：本切片统一为 `not-implemented`。
- **Evidence**：相关既有向量为 `not-run`；平台 PoC/端到端证据为 `none`。
- **Profile**：`planned`，未符合。

## 1. 产品范围与非目标

### 1.1 首要用户与任务

首要用户是 Agent 操作者：

1. 通过 Conversation 创建、继续和理解 Task；
2. 监督、纠偏、请求暂停、处理 unknown outcome；
3. 从允许来源检查并管理 Agent 生命周期。

用户任务语言优先；Task、Loop、AgentExecution、Runtime、Effect、Verification 在详情中分离且保留 authority/version/ref/evidence。

### 1.2 GA support matrix

| 维度 | Linux v1 决策 |
|---|---|
| Distribution | Ubuntu 24.04 LTS |
| Architecture | x86_64 |
| Desktop | stock GNOME |
| Display server | Wayland |
| 包 | 官网直接下载的原生 `.deb` |
| 支持期 | GA 起 24 个月，且不超过 distro/WebKitGTK/security floor |
| 后台 | systemd system service + 每登录 GNOME session 独立 broker |
| UI | GNOME header bar、system dialogs、XDG portal；不依赖 tray |
| 风险 | 只执行 R0/R1；R2/R3 只解释并阻断 |
| 内容 | 转义纯文本 + allowlist Markdown |
| acquisition | trusted registry ID + XDG portal/system picker 本地签名 bundle |
| a11y | Orca、键盘、高对比、100/200% scaling、app reduced-motion override |

每次发布必须记录 Ubuntu package revision、glibc、WebKitGTK API/engine build、GTK、GNOME、portal backend、Secret Service 和 systemd 版本。distro 的 CVE backport 状态优先于只比较上游版本号，但必须有可复核来源。

### 1.3 `planned/blocked`

- X11、KDE、其他 desktop/distro、arm64；
- RPM、Flatpak、AppImage、Snap；
- APT repository；
- URL/Git/private repo acquisition；
- raw HTML/script 和富内容预览；
- tray-dependent background UX；
- R2/R3 和通知批准；
- Console 自动获得 root、自动终止已接受任务或回滚 authority state。

## 2. 角色、身份与所有权

| 主体 | 能做什么 | 不能因此获得什么 |
|---|---|---|
| OS admin/polkit admin | 安装/更新/修复/stop system service、发起 claim | CognitiveOS Owner、tenant admin、R1 |
| Linux login user | 运行自己的 Console/broker、使用 portal | 其他 UID/session projection |
| CognitiveOS Owner/tenant admin | upstream 登录后确认 digest-bound claim | sudo/wheel/root |
| Agent operator | 创建和监督获授权任务 | authority commit、risk 降级 |
| system service | machine-wide node 端口、确定性门禁 | 用户 UI、任意 root、session 推断 |
| per-user broker | 当前 GNOME session 通知、lock/session 报告 | authority、跨用户 routing |
| privileged helper | `.deb`/A-B update/repair/stop 的固定 OS 操作 | 通用 shell、任意 package/path/URL |

Linux UID、group、active seat、D-Bus credential 和 polkit 只是 OS peer/authorization signals。产品账号、tenant、role、proposal 和 R1 decision 由 upstream IdP/authority 决定。

## 3. 组件与信任边界

```text
不可信 Agent 文本/Markdown/portal 文件
       │
       ▼
安全文本/allowlist Markdown renderer（无 raw HTML/IPC）
       │
Console system UI / trusted R1 confirmation page
       │ filtered session D-Bus / versioned local IPC
       ▼
per-user broker ───── systemd machine node service
                              │
                              ├── authority/durable state
                              ├── low-privilege staging updater
                              └── minimal polkit-gated helper
```

- system service 使用专用低权限 service identity；只有 helper 进入 root/polkit 边界。
- broker 是 systemd user service 或等价 session component，不启用 linger；关闭 broker 停止 lease renewal。
- D-Bus EXTERNAL/UID、bus policy 和 service activation 不构成产品授权；每次受保护请求仍绑定 authority session/capability/version。
- Flatpak 的 system-bus permission 不属于 GA 设计；GUI、broker、daemon 不共享全 session/system bus。
- helper 只接收 allowlisted operation ID、固定 slot/digest/version，不接受任意 command、path、repository 或 network URL。

## 4. `.deb` 安装、claim 与启动

### 4.1 `.deb` 责任边界

`.deb` 只拥有稳定、低变更的 control plane：

- bootstrap executable；
- updater/staging coordinator 及其最小 privileged helper；
- systemd system unit 与必要的 user broker unit；
- 上述组件必需的 polkit policy、公钥/threshold verifier 和 uninstall/repair manifest。

bootstrap 只提供本机 claim/repair 的最小可信界面与协议，不提供完整 Shell/Task/Agent UI；完整 Console 和 node payload 随首个签名 slot 激活。

签名 payload 安装到版本化 A/B 槽，例如 `/opt/cognitiveos/slots/<digest>`；`/opt/cognitiveos/current` 只由 updater 通过原子 rename/symlink exchange 切换。slot payload 不列为 dpkg-owned 文件，updater 不改写 `/usr` 下 dpkg-owned 内容。authority state 与 durable data 位于独立受控目录，不属于 rollback slot。

项目不运营 APT repository，因此不能借用 apt repository 的 signed Release 信任链。官网必须同时发布 threshold-signed release manifest/detached signature，将 `.deb` digest、target、version 和 floor 固定；用户或企业安装流程须先验证该绑定。企业可用其自有软件分发，但不得改变制品 digest。

### 4.2 unclaimed quarantine 与 claim

1. `.deb` 安装 stable components，但不把 daemon 标为 claimed/ready。
2. daemon 以 unclaimed quarantine one-shot/受限模式生成 node key，只开放 local claim。
3. OS admin 发起短期 claim，固定 key/build/Ubuntu facts/tenant digest/nonce/expiry。
4. Console 通过 upstream IdP 登录；只有 CognitiveOS Owner/tenant admin 可确认 digest。
5. authority 原子登记 owner/tenant，bootstrap 验证 binding。
6. updater 获取并验证首个 signed payload，将其写入 inactive slot；下载/验证失败时保持非 operational 且不 enable boot start。
7. OS admin/polkit 在维护确认中激活首个 slot；health 成功后 helper 才 enable persistent boot start。readiness 仍由 authority/backend 能力决定。

claim 重放、UID 自报、build/key/tenant mismatch、并发已领取、store/audit 不健康都 fail closed。

### 4.3 broker opt-in

- onboarding 或 managed policy 明示 opt-in 登录启动。
- broker unit 绑定图形 session，不启用 user linger，也不跨 logout 续存。
- 用户关闭 broker时停止通知和 lease renewal；system service/Task 不停止。

## 5. stop、A/B update、rollback、repair 与 uninstall

### 5.1 update metadata 和 staging

- updater 可自动获取 threshold-signed recommended/security-floor metadata。
- recommended minimum 只警告；security floor 可触发 capability kill switch。
- updater 可在专用低权限 staging area 下载和验证 payload；无 ambient proxy credential、SSH agent、`.netrc` 或 Git credential helper。
- staging 验证 threshold signature、payload digest、target Ubuntu/arch、protocol/build compatibility、floor 和资源预算。
- 网络检查或 staging 不表示安装、authority commit 或新 slot active。

### 5.2 maintenance switch

切换 `current` 必须同时满足：

1. authority 接受 maintenance intent；
2. 拒绝新任务并完成 drain；
3. 没有 `OUTCOME_UNKNOWN` 或未处置 Effect；
4. store/audit/watch 满足 gate；
5. 用户/管理员进入明确维护窗；
6. polkit/OS admin 授权 OS 更新边界；
7. target slot 签名、floor、版本和 compatibility 仍有效。

切换使用原子 filesystem 操作；启动后执行 health/readiness，再由 authority 恢复准入。Console 或 updater 本地 success 不完成管理 Task。

### 5.3 rollback

- activation 失败只回退 previous slot；previous 必须签名有效且高于当前 security floor。
- previous slot 不兼容当前 durable data/schema 时禁止回退并进入 repair/quarantine。
- authority state、Effect、Verification、Event 不回滚。
- rollback 响应丢失时按稳定 update ref 和 active slot digest 查询，不重复切换。

### 5.4 stop、repair、uninstall

- normal stop：authority drain → no-new-task → safe checkpoint → OS admin/polkit stop service。
- emergency stop：允许立即停，但在途 Effect 为 `OUTCOME_UNKNOWN`，禁止盲重试，恢复后 reconcile。
- repair：只用签名且高于 floor 的 `.deb` 或 previous slot；projection 可重建，node key 只有完整性成立才保留。
- key 丢失/异常、机器转让或 owner/tenant 变化：quarantine、revoke、reclaim。
- uninstall：删除本机程序/slot/cache 的明确范围，不删除 authority 侧状态、审计和未决 Effect。

## 6. 窗口、后台、锁屏、用户切换与通知

### 6.1 无 tray 完整路径

- stock GNOME/Wayland 不依赖 tray 或 StatusNotifierItem。
- 关闭窗口仅关闭 UI；broker opt-in 时可继续脱敏通知和 authority 允许的监督。
- 设置页和显式“退出 Console 并停止监督”提供完整停止入口。
- explicit exit 停 GUI、broker、lease renewal，不停 system service 或已接受任务。
- 重新打开 Console 后从 authority resnapshot，不从 broker cache推断状态。

### 6.2 lock/logout/switch user

- broker 通过当前 session/logind 能力报告 lock、active/inactive、logout。
- 只有 authority 可签发/维持明确 expiry 的 signed lease；本地 grace timer 不具有权威性。
- lock 后停止 token refresh、R1、acquisition 和受保护写，并 teardown 敏感 renderer/buffer。
- logout 终止 broker；不启用 linger 保持用户监督。
- UID/session A 的 Secret Service、notification handle、draft、projection 和 lease 不得由 B 访问。

### 6.3 freedesktop notifications

- 通知按 server capabilities 降级；不依赖 action 或 persistence。
- 唯一 action 是打开 Console，携一次性 opaque handle；没有 action capability 时显示同样通用提示。
- handle 消费后重认证/resnapshot；通知正文不含稳定 item ref、credential、Task title 或参数。
- broker 缺失/权限关闭/quiet hours 时，事项留在 authority Inbox。
- notification read/close 不改变 handled/decided/reconciled。

## 7. 关键旅程

### 7.1 安装并 claim

`下载 .deb → 系统 package/polkit → unclaimed quarantine → upstream 登录 → Owner digest 确认 → authority binding → enable boot start`

失败出口：package/metadata/floor 不符、claim 过期或冲突、tenant/key/build mismatch、polkit 取消、store/audit degraded。

### 7.2 创建与监督 Task

`Conversation → fixed preview → authority risk floor → R0/R1 → stable refs → snapshot/watch → separated lifecycle`

GNOME UI 先显示“发生什么/我该做什么”，机器对象、digest 和证据按需展开。

### 7.3 Linux R1

- trusted system confirmation page显示目标、变化、risk、Effect、budget、egress、deadline、verification、digest。
- 需要 reauth 时交给 upstream IdP；polkit 不显示或批准产品 proposal。
- authority 验证当前 proposal/version/session 后产生 decision。
- R2/R3 没有批准控件。

### 7.4 portal 本地 bundle

`XDG FileChooser → portal URI/access → file identity/symlink gate → archive budget → signature/provenance/digest → compatibility/sandbox → authority admission`

portal filter 和 user selection 不是信任证明；文件仍可能不匹配 filter 或在检查期间被替换。

### 7.5 A/B update

`signed metadata → stage slot → authority maintenance/drain → no unknown → polkit window → atomic switch → health/readiness → reconcile`

任一步失败保留当前 active slot；只有 target 已切换且 authority 对账后 UI 才显示管理动作收敛。

## 8. 页面与状态矩阵

### 8.1 页面

| 页面 | 主要任务 | Linux 约束 |
|---|---|---|
| Install/help | 下载 `.deb`、解释 system package | Console 不运行任意 shell |
| Unclaimed node | 显示 key/build/claim | 仅 local claim |
| Sign in/claim | upstream IdP、Owner digest | polkit 不替代产品确认 |
| Work/Shell | Conversation、Task preview | 安全文本/Markdown |
| Task center/detail | 五轨、lease、Effect | authority projection |
| Agent source/check | registry/portal bundle | 无 URL/Git/private repo |
| R1 confirmation | trusted page + upstream reauth | 无 notification/polkit approval |
| Inbox | pause pending/unknown/degraded | 通知不替代 Inbox |
| System health | service/store/audit/watch/WebKit/floor | readiness 不扩大权限 |
| Update slots | current/target/previous/drain | dpkg boundary、floor |
| Repair/quarantine | key/slot/integrity 恢复 | 不回滚 authority state |
| Settings | broker opt-in、notifications、motion | 无 tray 依赖 |

### 8.2 状态

| 状态 | 展示 | 动作 |
|---|---|---|
| `initial-loading` | 静态 skeleton | 无假控件 |
| `authoritative-empty` | authority 空集 | 适用创建动作 |
| `refreshing-last-good` | `as_of`、刷新状态 | freshness 允许的只读/写 |
| `partial/redacted` | 已加载范围和缺口 | 依赖缺口的写禁用 |
| `stale-offline` | 非实时 Trust Strip | 受保护写禁用 |
| `privacy-locked` | 无敏感正文 | 无通知控制 action |
| `reauth-required` | session/Secret Service 原因 | 登录后 resnapshot |
| `floor-warning` | recommended minimum | 查看更新 |
| `floor-blocked` | distro/WebKit/app floor | 诊断/导出/更新修复 |
| `staged-update` | target slot/digest、未 active | 进入维护窗 |
| `drain-pending` | authority 正在拒绝新任务/收敛 | 查看 unknown/Effect |
| `result-unknown` | 原 key/dispatch/reconcile | 无 retry |
| `quarantined` | claim/key/integrity/slot 原因 | repair/reclaim |
| `completed` | authority acceptance + current verification | 查看证据 |

## 9. Security threat model

必须覆盖：

- 恶意 Agent 文本、Markdown、portal 文件、bundle、Unicode/Bidi；
- WebKitGTK memory corruption/sandbox escape、renderer→IPC；
- D-Bus name spoofing、UID/session confusion、activation race、overbroad bus policy；
- polkit confused deputy、`*_keep` 被误用为产品批准；
- malicious local user、active/inactive seat、switch user、broker route；
- Secret Service absent/locked/cancel、attributes 泄密；
- symlink/hardlink/FIFO/device/FUSE/network mount、TOCTOU、archive bomb；
- threshold metadata/payload key compromise、stale mirror、downgrade；
- dpkg-owned file 被 updater 改写、slot/data incompatibility、非原子 switch；
- WebKit package 有 CVE 或 distro backport 状态无法证明；
- lock/logout/crash/network partition 后错误续租；
- emergency stop、timeout 和 `OUTCOME_UNKNOWN` 盲重试；
- store/audit/watch 不可用时 local buffer 冒充 commit。

## 10. Acquisition 与 secure storage

### 10.1 acquisition

v1 只接受 trusted registry ID 与 portal/system picker 本地签名 bundle。必须：

- 解析 portal URI 后验证实际文件和调用者授权；
- 拒绝不允许的 symlink/hardlink/FIFO/device/proc/sys/network mount；
- 通过 open handle/identity 固定规避检查后替换；
- 限制 archive path、大小、文件数、展开比、嵌套、CPU/time；
- 固定 signature/provenance/manifest/dependency/digest；
- 绑定 Ubuntu/arch/WebKit/sandbox/adapter evidence；
- 禁止 network prefetch、ambient proxy/SSH/Git/browser credentials；
- 由 authority 形成 acquisition proposal/Effect 和 risk floor。

### 10.2 Secret Service degradation

- per-user token/credential 只进入 Secret Service；lookup attributes 不放敏感正文。
- machine node key 的 system-service storage/TPM 方案是 Open PoC，不复用任一用户 keyring。
- Secret Service name 不存在、collection locked、prompt cancelled 或 session bus unavailable 均 fail closed。
- 禁止 home dotfile、environment、plaintext config、日志或 in-memory-to-disk fallback。
- 降级时停止 refresh/lease/R1/claim/acquisition/写，保留非敏感只读 projection、诊断和修复。
- 恢复后重认证/resnapshot，不把旧 token/cache 恢复为当前。

## 11. WebKitGTK floor 与 kill switch

- Tauri WebKitGTK API 4.1 只是 ABI/API 线索，不证明 CVE 已修复。
- release manifest 必须记录 Ubuntu package name/version/revision、upstream security advisory mapping 和 accepted backport evidence。
- 不可信内容无 native/D-Bus capability；system cards 只从 authority projection 构建。
- parser 剥离 raw HTML/script/iframe/object/embed/event handler、remote image 和危险 scheme。
- CSP/Tauri isolation 只是纵深防御；WebKitGTK process separation 也不替代 patched runtime。
- signed floor 低于要求或 backport 无法证明时立即使用安全文本模式，关闭 acquisition、claim/R1、写和 lease renewal。
- kill switch 只收窄客户端能力，不改变 authority Task/Effect，也不自动 stop daemon/任务。

## 12. GNOME/Wayland experience、accessibility 与 motion

- 使用 GNOME header bar、system dialog、portal、标准 window/menu/shortcut；不模拟 Windows tray。
- 窗口关闭后 broker 状态可在 Settings/System/Inbox 和通知权限中发现。
- 全局 shortcut 只有经 portal/desktop 支持并由用户配置时启用；不存在时不影响主旅程。
- Orca 下 heading、landmark、table、list、button、status/live region 完整；动态列表不移动当前焦点。
- High Contrast 下不硬编码低对比颜色或透明边界。
- 100/200% scaling 下不截断动作动词、digest 和安全 deadline。
- app-level reduced-motion override 始终存在；系统信号可用时同步，但不依赖 GTK/WebKit 自动传播。
- reduced motion 取消位移、缩放、parallax、shimmer 和连续 ambient，保留静态 Flow Thread 与 live announcement。

## 13. Open PoC and GA gates

| Gate | 必须证明 | 当前 |
|---|---|---|
| `LNX-POC-01` | Ubuntu 24.04 x86_64 stock GNOME/Wayland `.deb` 安装/卸载 | not-run |
| `LNX-POC-02` | system service、per-user broker、polkit helper 权限与 D-Bus isolation | not-run |
| `LNX-POC-03` | unclaimed quarantine、claim、revoke/reclaim、boot enable | not-run |
| `LNX-POC-04` | `.deb` 不改写 slot payload；A/B stage/atomic switch/rollback | not-run |
| `LNX-POC-05` | authority drain/no-unknown/maintenance/polkit 组合 gate | not-run |
| `LNX-POC-06` | WebKitGTK distro package floor、advisory/backport、kill switch | not-run |
| `LNX-POC-07` | Secret Service missing/locked/cancel 无 fallback | not-run |
| `LNX-POC-08` | lock/logout/switch user 下 broker 与 signed lease eligibility | not-run |
| `LNX-POC-09` | portal bundle path/symlink/archive/TOCTOU/budget 负例 | not-run |
| `LNX-POC-10` | 无 tray 的 close/exit/reopen/notification 完整旅程 | not-run |
| `LNX-POC-11` | opaque notification handle、跨 UID/session、action 缺失降级 | not-run |
| `LNX-POC-12` | Orca/keyboard/High Contrast/100-200%/reduced-motion 核心旅程 | not-run |

GA 还要求：

- Console 通用 backend gate 已通过；
- 支持期起止、Ubuntu security coverage 和 WebKit floor 有发布日证据；
- 两次 WebKitGTK/Tauri/systemd/portal 升级演练；
- 所有 release-blocking requirement 有 owner、实现和已执行 oracle；
- X11/KDE/其他 distro/package 不被安装页或营销文案暗示支持；
- 无错误完成、跨用户泄露、盲重试、dpkg 文件破坏、floor 下危险写。

## 14. Linux v1 产品要求与追踪

| ID | 原子要求 | Contract | Implementation | Evidence | Owner | Oracle | blocked_by |
|---|---|---|---|---|---|---|---|
| `CONSOLE-LNX-V1-PRD-001` | Console/broker/helper 非 authority/IdP/node/final arbiter | partial: `REQ-SHELL-CHANNEL-001`, `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Console security | local UI/cache/D-Bus spoof 不能启用写或改状态 | M5; `LNX-POC-02` |
| `CONSOLE-LNX-V1-PRD-002` | 五生命周期域与 Runtime 分离 | partial: `REQ-RUN-009`, `REQ-EFF-STATE-001` | not-implemented | not-run | UNASSIGNED — Runtime/Console | Runtime stop/remote completed 不完成 Task | M2–M5 carriers |
| `CONSOLE-LNX-V1-PRD-003` | system service、session broker、polkit helper 最小分权 | missing | not-implemented | none | UNASSIGNED — Linux integration | compromise 任一组件不能取得其他身份能力 | `LNX-POC-02` |
| `CONSOLE-LNX-V1-PRD-004` | Linux admin/UID 与 Owner/tenant role 正交 | partial: `REQ-MGMT-SESSION-001` | not-implemented | not-run | UNASSIGNED — Identity | root/wheel/UID 不能绕过 upstream role | identity/claim contract |
| `CONSOLE-LNX-V1-PRD-005` | claim 固定 key/build/tenant digest，异常 revoke/reclaim | missing | not-implemented | none | UNASSIGNED — Identity | replay/mismatch/transfer 全部 quarantine | `LNX-POC-03` |
| `CONSOLE-LNX-V1-PRD-006` | authority risk floor；只执行 R0/R1 | partial: `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Risk | R2/R3 和客户端降 risk 无执行控件 | R1 contract |
| `CONSOLE-LNX-V1-PRD-007` | signed lease 仅 authority 签发，broker 不本地延长 | missing | not-implemented | none | UNASSIGNED — Supervision | lock/logout/revoke 后旧 broker 续租失败 | supervision contract; `LNX-POC-08` |
| `CONSOLE-LNX-V1-PRD-008` | close UI/exit supervision/system service 分离 | partial: `REQ-SHELL-DETACH-001` | not-implemented | not-run | UNASSIGNED — Console | close、exit、kill broker 得到规定状态且任务不误报 | broker/lease contract |
| `CONSOLE-LNX-V1-PRD-009` | 不依赖 tray 的完整监督和退出路径 | product-only | not-implemented | none | UNASSIGNED — GNOME UX | 无 SNI host 时所有旅程仍可达 | `LNX-POC-10` |
| `CONSOLE-LNX-V1-PRD-010` | notification 仅 opaque open handle，能力缺失可降级 | missing | not-implemented | none | UNASSIGNED — Notification | action/persistence 缺失、重放、错 UID 均安全 | `LNX-POC-11` |
| `CONSOLE-LNX-V1-PRD-011` | Secret Service 失败无明文 fallback并收窄能力 | missing | not-implemented | none | UNASSIGNED — Linux security | absent/locked/cancel/bus down fixtures 正确处理 | `LNX-POC-07` |
| `CONSOLE-LNX-V1-PRD-012` | lock/logout/switch teardown，恢复 reauth/resnapshot | missing | not-implemented | none | UNASSIGNED — Privacy | UID/session A 的 token/projection 对 B 不可见 | `LNX-POC-08/11` |
| `CONSOLE-LNX-V1-PRD-013` | 安全文本/allowlist Markdown，raw HTML/script 删除 | product-only | not-implemented | none | UNASSIGNED — Renderer | malicious corpus 无脚本、bridge、network fetch | `LNX-POC-06` |
| `CONSOLE-LNX-V1-PRD-014` | WebKitGTK floor 按 distro build/backport 证明并 kill switch | partial: `REQ-ERR-001` | not-implemented | none | UNASSIGNED — Release security | vulnerable/unknown build 只保留 safe mode | `LNX-POC-06` |
| `CONSOLE-LNX-V1-PRD-015` | acquisition 仅 registry ID 和 portal 本地签名 bundle | partial: `REQ-AGENT-INSTALL-001`, `REQ-AGENT-SANDBOX-001` | not-implemented | not-run | UNASSIGNED — Agent lifecycle | URL/Git/private repo 无入口，bundle negatives 拒绝 | M6; `LNX-POC-09` |
| `CONSOLE-LNX-V1-PRD-016` | GA 仅 Ubuntu 24.04 x86_64 GNOME/Wayland native `.deb` | product-only | not-implemented | none | UNASSIGNED — Release | unsupported matrix 安装/营销/telemetry 均明确 blocked | `LNX-POC-01/06/12` |
| `CONSOLE-LNX-V1-PRD-017` | `.deb` 只拥有 stable bootstrap/updater/unit，slot 非 dpkg-owned | missing | not-implemented | none | UNASSIGNED — Packaging | package upgrade/file manifest 证明无 dpkg overwrite | `LNX-POC-04` |
| `CONSOLE-LNX-V1-PRD-018` | A/B switch 需 drain/no unknown/maintenance/polkit 并原子化 | partial: `REQ-MGMT-IDEM-001`, `REQ-EFF-004` | not-implemented | not-run | UNASSIGNED — Update/Recovery | 任一 gate 缺失不切换，crash 保留单一 active | `LNX-POC-04/05` |
| `CONSOLE-LNX-V1-PRD-019` | rollback 仅 previous slot 且高于 floor，不回滚 authority state | partial: `REQ-MGMT-RECOVERY-001` | not-implemented | not-run | UNASSIGNED — Recovery | vulnerable/incompatible/state rollback 均拒绝 | `LNX-POC-04/06` |
| `CONSOLE-LNX-V1-PRD-020` | polkit 只处理 OS 边界，不替代产品 R1 | missing | not-implemented | none | UNASSIGNED — Linux/R1 security | cached polkit auth 无法批准 proposal | R1 display contract; `LNX-POC-02` |
| `CONSOLE-LNX-V1-PRD-021` | normal stop 先 drain；emergency unknown 后对账 | partial: `REQ-EFF-004`, `REQ-MGMT-RECOVERY-001` | not-implemented | not-run | UNASSIGNED — Recovery | strong stop 后无 completed/paused 假状态和盲重试 | M4/M5; `LNX-POC-05` |
| `CONSOLE-LNX-V1-PRD-022` | Orca/keyboard/contrast/100-200%/motion 完成核心旅程 | product-only | not-implemented | none | UNASSIGNED — Accessibility | 指定真实矩阵走查留证 | `LNX-POC-12` |
| `CONSOLE-LNX-V1-PRD-023` | 用户任务语言优先且 machine enum/ref/digest 保真 | product-only | not-implemented | none | UNASSIGNED — Content design | zh-CN/en 长文本和 Bidi 无信息损失 | content/a11y evidence |
| `CONSOLE-LNX-V1-PRD-024` | 四态状态声明严格分离 | product-only | not-implemented | none | UNASSIGNED — Traceability | none/not-run 不能显示 pass/implemented | M1 runner; platform evidence |

## 15. 官方来源 ledger

以下均查询于 2026-07-20；第三方项目明确标注。

| 标题 | URL | 用于核实 |
|---|---|---|
| Ubuntu release cycle | https://ubuntu.com/about/release-cycle | Ubuntu 24.04 标准安全维护边界 |
| systemd | https://www.freedesktop.org/software/systemd/man/latest/systemd.html | system/user manager |
| systemd-run | https://www.freedesktop.org/software/systemd/man/latest/systemd-run.html | user service 与 linger/logout |
| D-Bus Specification | https://dbus.freedesktop.org/doc/dbus-specification.html | peer credentials、activation、interactive flag |
| polkit Reference Manual | https://polkit.pages.freedesktop.org/polkit/polkit.8.html | authority、per-session agent、active/inactive |
| Secret Service API Draft | https://specifications.freedesktop.org/secret-service-spec/latest-single/ | locked collections、prompt、attributes |
| WebKitGTK Security Advisories | https://webkitgtk.org/security.html | security advisory 索引 |
| WSA-2026-0004 | https://webkitgtk.org/security/WSA-2026-0004.html | 查询日最新 advisory、2.52.5 floor 事实 |
| WebKitGTK 2.52.5 released | https://webkitgtk.org/2026/07/09/webkitgtk2.52.5-released.html | 查询日 latest stable |
| Wayland Protocol | https://wayland.freedesktop.org/docs/html/ | compositor/client 模型 |
| XDG Desktop Portal for App Developers | https://flatpak.github.io/xdg-desktop-portal/docs/for-app-developers.html | portal/session service |
| FileChooser Portal | https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html | portal file access/filter 限制 |
| Background Portal | https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Background.html | background/autostart 是请求而非保证 |
| Desktop Notifications Specification | https://specifications.freedesktop.org/notification-spec/latest-single/ | action/persistence capability limits |
| Status Icons and GNOME | https://blogs.gnome.org/aday/2017/08/31/status-icons-and-gnome/ | GNOME 默认无 status icons |
| Orca Introduction | https://help.gnome.org/users/orca/stable/introduction.html.en | Orca/AT-SPI/WebKitGTK |
| GNOME Adjust the contrast | https://help.gnome.org/users/gnome-help/3.4/a11y-contrast.html.en | High Contrast |
| KDE Accessibility HIG | https://develop.kde.org/hig/accessibility/ | 仅作未来 KDE 研究；KDE 不在 GA |
| apt-secure | https://manpages.debian.org/stable/apt/apt-secure.8.en.html | Release authentication；本项目不运营 APT repo |
| Flatpak Sandbox Permissions | https://docs.flatpak.org/en/latest/sandbox-permissions.html | 未来 Flatpak system/session bus 限制 |
| AppImage Architecture | https://docs.appimage.org/reference/architecture.html | 未来 AppImage runtime 不做内容检查 |
| Tauri prerequisites（third-party project） | https://v2.tauri.app/start/prerequisites/ | WebKitGTK API 4.1/系统依赖，不等于 security floor |
| Tauri Capabilities（third-party project） | https://v2.tauri.app/security/capabilities/ | IPC scope 与 WebView 0-day 非覆盖范围 |

## 16. 状态声明

- Linux v1 产品范围与决策：已记录。
- Linux platform machine contract：未登记；仅有部分通用 REQ 锚点。
- Console/Linux implementation：未提供。
- Ubuntu/GNOME/Wayland/WebKitGTK/Secret Service/polkit/A-B/a11y 测试：未执行。
- 相关既有 conformance vectors：依据 PROGRESS 全部 `not-run`。
- Linux Console Profile：未符合。
