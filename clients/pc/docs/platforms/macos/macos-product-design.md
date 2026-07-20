# CognitiveOS Console macOS v1 产品设计

> 类别：informative product design
>
> 决策状态：accepted product direction
>
> 交付状态：`planned / implementation not started / test not executed / Profile not implemented`
> 官方资料查询日期：2026-07-20

本文定义 macOS 独立产品切片，不修改 Windows v1，也不新增任何 CognitiveOS machine contract。涉及 daemon、claim、signed lease、R1 display、update floor、notification routing 和 acquisition 的行为在对应机器合同登记前均为 `unregistered/planned/blocked`。

## 0. 事实与状态口径

- **平台事实**：仅来自 §15 官方来源 ledger，查询日期固定为 2026-07-20。
- **产品决策**：使用 `CONSOLE-MAC-V1-DEC-*`，约束体验和发布范围，但不进入 normative registry。
- **产品要求**：使用 `CONSOLE-MAC-V1-PRD-*`，只作产品追踪。
- **Machine contract**：只有 `specs/registry/requirements.yaml` 中真实存在的 `REQ-*` 才可称已登记；平台缺口写 `missing`。
- **Implementation**：本切片统一为 `not-implemented`。
- **Evidence**：相关既有向量保持 PROGRESS 的 `not-run`；平台 PoC/端到端证据为 `none`。
- **Profile**：`planned`，未符合。

## 1. 产品范围与非目标

### 1.1 首要用户与任务

首要用户是 Agent 操作者。v1 保持三个核心任务：

1. 通过 Conversation 创建和继续受治理 Task；
2. 监督、纠偏、请求暂停、对账和判断真实完成；
3. 检查并管理允许来源的 Agent 生命周期。

用户首先看到任务语言；Task、Loop、AgentExecution、Runtime、Effect、Verification、稳定引用和证据在详情中保真。

### 1.2 GA support matrix

| 维度 | macOS v1 决策 |
|---|---|
| OS 设计候选 | macOS 14+ |
| GA OS | 仅保留 GA 时经核实仍获安全修复且满足 WKWebView/helper floor 的版本 |
| CPU | Universal 2：Apple silicon + Intel |
| 包 | Developer ID 签名并公证的 PKG |
| 分发 | 官网手工安装；企业 MDM 复用同一 PKG |
| UI | 普通 Dock app + 可选 menu extra |
| 后台 | machine-wide daemon + per-login-user broker |
| 风险 | 只执行 R0/R1；R2/R3 只解释并阻断 |
| 内容 | 转义纯文本 + allowlist Markdown |
| acquisition | 受信 registry ID + 系统 picker 选择的本地签名 bundle |
| 语言 | 简体中文、英文；机器 enum/digest 不翻译 |

支持窗口不引用不存在的 Apple 固定生命周期承诺。任一 OS/CPU 组合低于平台、WKWebView、helper 或签名 security floor 时可被阻断；Universal 2 制品存在不等于 Intel 永久受支持。

### 1.3 非目标

- Mac App Store、DMG GA、任意 URL/Git/private repository acquisition；
- raw HTML、脚本或内嵌任意网页富内容；
- R2/R3、通知批准、密码批准、Touch ID 单独批准；
- Console 承载 node、IdP、authority 或最终安全仲裁；
- Console 自动下载、自动提权安装或自动终止已接受任务；
- authority state 的本机回滚；
- 在 App Sandbox PoC 失败时静默取消 sandbox；
- 把 Windows、Linux 或某一 Mac build 的安全证据外推到其他平台。

## 2. 角色、身份与所有权

| 主体 | 能做什么 | 不能因此获得什么 |
|---|---|---|
| macOS admin | 安装/修复 PKG、启动 claim、批准 OS 级更新/停止 | CognitiveOS Owner、tenant admin、R1 |
| macOS user | 运行自己的 Console/broker、使用系统 picker | 其他用户 projection、节点所有权 |
| CognitiveOS Owner/tenant admin | 通过 upstream IdP 登录并确认 digest-bound claim | macOS root/admin 权力 |
| Agent operator | 创建、监督、纠偏和收敛获授权任务 | authority commit、risk 降级 |
| machine daemon | 承载本机 node 端口并执行确定性门禁 | GUI、用户会话推断、任意 root 能力 |
| per-user broker | 绑定登录用户投递通知和报告 lock/session 状态 | authority 状态、跨用户转发 |
| privileged helper | 安装/更新/恢复/stop service 的最小 OS 操作 | 通用 shell、任意路径/URL、产品批准 |

账号和 tenant 只来自 upstream IdP 与 authority。OS admin/OS user 不自动成为产品主体。所有敏感请求绑定 node、principal、tenant、macOS audit/session identity、channel 和版本；客户端自报字段不能替代 OS peer evidence。

## 3. 组件与信任边界

```text
不可信 Agent 文本/Markdown/本地 bundle
        │
        ▼
受限内容解析与展示（无 raw HTML、无脚本、无 native bridge）
        │
Console system UI / trusted native R1 sheet
        │ versioned allowlist XPC
        ▼
每用户 broker ────── machine-wide low-privilege node daemon
                              │
                              ├── authorities / durable state
                              └── minimal privileged helper
```

边界规则：

- Console、broker、daemon、helper 使用不同身份、entitlements 和 IPC allowlist。
- daemon 不呈现 UI，不主动联系任意用户进程；broker 只能服务其登录 session。
- XPC peer 必须满足发布签名/identity requirement，并再次绑定 authority session；签名相同不自动授权动作。
- helper 接受抽象 operation ID 和固定参数结构，不接受 shell 字符串、任意 executable、URL 或未固定路径。
- 密码、claim、Touch ID R1 sheet 与不可信内容不共享 renderer、DOM、credential 或 IPC capability。
- ContextView 只作非 authority projection；写入决策重新读取 authority state/version。

## 4. 安装、claim 与启动

### 4.1 PKG 安装

1. 用户从官网取得或由 MDM 下发同一签名 PKG。
2. Installer 验证 Developer ID、notarization ticket、版本、minimum security floor 和组件清单。
3. PKG 安装 Universal 2 Console、低权限 daemon、per-user broker、最小 helper 及其 launchd/Service Management 配置。
4. 安装不创建 CognitiveOS Owner，不写入 tenant，不启用已 claim 的假状态。
5. 安装结束只启动 unclaimed quarantine 的本地 claim surface；持久 boot start 尚未启用。

### 4.2 node claim

1. daemon 在受控存储中生成 node key；私钥不交给 Console。
2. OS admin 发起短期 claim；claim 固定 node public key、daemon build、platform facts、tenant digest、nonce 和 expiry。
3. Console 通过 upstream IdP 登录，展示可信 native claim sheet。
4. 只有 CognitiveOS Owner/tenant admin 可确认 operation digest。
5. authority 原子登记 node owner/tenant 并返回固定 binding。
6. daemon 验证 authority binding 后才请求 helper enable boot start，并进入可声明 readiness 的流程。

claim 过期、digest 改变、并发已领取、tenant 不匹配、key/build 变化或 authority 不可达均 fail closed。不能回退为“第一个连接者自动成为 Owner”。

### 4.3 broker opt-in

- 首次 onboarding 或 managed policy 明示询问是否在登录时启动 broker。
- opt-in 与 notification permission、supervision eligibility 分开。
- 用户关闭 broker 时立即停止 broker heartbeat/lease renewal；不停止 daemon 或已接受任务。

## 5. 生命周期、分发与供应链

### 5.1 正常 stop

1. authority 标记 maintenance intent 并拒绝新任务准入；
2. 对相关 Task/AgentExecution 请求 drain 到安全检查点；
3. 所有未决 Effect 必须 closed、quarantined 或明确保持 unknown；
4. authority 返回 drain 结果和稳定引用；
5. OS admin 通过 helper 停 daemon。

GUI 关闭、Console 退出、Runtime 终止和 daemon stop 都不等于 Task 完成或 Effect 收敛。

### 5.2 紧急强停

OS admin 可执行 emergency stop，但：

- 不宣称 Task 已暂停/失败/完成；
- dispatch 后未确认的 Effect 转为 `OUTCOME_UNKNOWN`；
- 禁止换 idempotency key 或自动重放；
- daemon 恢复后从 Effect、Verification、Event 和 authority state 对账。

### 5.3 更新

- Console 只获取 threshold-signed metadata；recommended minimum 只警告，security floor 可 fail closed。
- Console 不自动下载或提权安装；用户打开官网 PKG，MDM 可接管或禁用检查。
- PKG 必须同时验证 Developer ID、notarization、threshold metadata binding、版本单调性和组件兼容窗口。
- 安装前 authority drain；unknown Effect、store/audit 不健康或版本窗口不闭合时禁止切换。
- GUI、daemon、broker、helper 的协议兼容必须在同一发布 manifest 中固定。

### 5.4 rollback、repair、uninstall

- rollback 只到仍高于 signed security floor 的签名/公证版本。
- authority state、Event、Effect、Verification 不随本机二进制回滚。
- projection 可重建；schema/data migration 不支持安全后退时必须保持新版本或进入 repair/quarantine。
- repair 只接受签名且高于 floor 的 PKG；node key 完整性成立才可保留。
- key 丢失/异常、机器转让或账号归属变化时 revoke 旧 binding 并重新 claim。
- uninstall 可删除本机组件和明确选择的本机 cache，但不删除 authority 侧审计、未决 Effect 或历史稳定引用。

## 6. 窗口、退出、锁屏、用户切换与通知

### 6.1 窗口和退出

- 关闭窗口：仅关闭 UI；不解释为退出或 pause。
- broker 已 opt-in：可继续脱敏通知和 authority 明确允许的监督。
- 显式“退出 Console 并停止监督”：停止 GUI、broker 和 lease renewal；列出受影响任务与 lease expiry，但不停止 node/任务。
- macOS 提供 Dock app 与可选 menu extra；移除 extra 不应成为唯一停止控制路径。

### 6.2 锁屏和 fast user switching

- broker 从 OS session 观察 lock/switch 并报告 authority。
- 只有 authority 可维持或签发带明确 expiry 的 signed lease grace；Console 本地 timer 不能延长。
- lock 后 teardown 敏感 UI、清除 app-managed buffer，停止 token refresh 和受保护动作。
- 解锁后进入 `reauth-required`，重新认证、授权和 resnapshot。
- user A 的 broker、Keychain item、notification handle、cache 和 lease 不能被 user B 发现或复用。

### 6.3 通知

- 仅 authenticated per-user broker 按 authority account/node/session binding 投递。
- 锁屏只显示通用提示，不显示正文、节点敏感 alias、目标或风险参数。
- 唯一 action 是“打开 CognitiveOS Console”，payload 只有一次性、高熵、短时 opaque handle。
- 打开后验证 app、OS session、principal、node、audience、expiry，原子消费 handle，再重认证/resnapshot。
- broker 缺失、通知被拒或 quiet hours 时，事项仍留在 authority Inbox；acknowledged 不等于 handled。

## 7. 关键旅程

### 7.1 安装并 claim

`下载/MDM PKG → Installer 验签/公证 → unclaimed quarantine → upstream 登录 → Owner 确认 digest → authority binding → enable boot start`

失败出口：签名/floor 不符、claim 过期、并发已领取、tenant/key/build 不匹配、store/audit 不健康、App Sandbox/XPC gate 失败。

### 7.2 创建和监督任务

`Conversation → Intent preview → authority risk floor → R0/R1 → stable Task/AgentExecution refs → snapshot/watch → Flow Thread`

UI 必须保持五个生命周期域分离；远端 completed、receipt 或 Agent 文本不能把 Task 显示为 `COMPLETED`。

### 7.3 Touch ID R1

1. authority 返回固定 proposal digest、nonce、session、expiry 和 display fields；
2. trusted native sheet 显示完整动作，不把不可信内容混入；
3. Touch ID 解锁 device private key并签名绑定；
4. authority 验证签名、session、nonce、expiry、risk 和当前版本；
5. 只有 authority decision 可推进。

Touch ID 不可用、用户取消、Keychain 锁定或 display 字段不闭合时回到安全 reauth 或取消，不降级为普通按钮。

### 7.4 本地 Agent bundle

`系统 picker → security-scoped URL → 调用者授权 → symlink/TOCTOU 固定 → archive budget → 签名/provenance/digest → compatibility/sandbox → authority risk admission`

picker 授权只允许读取所选对象，不等于 package trusted 或可运行。

### 7.5 更新与修复

`signed metadata → 用户/MDM 取得 PKG → floor/组件验证 → authority drain → Installer/helper 切换 → health/readiness → reconcile`

响应丢失时按稳定 update/ref 查询，不再次安装不同版本来“试一次”。

## 8. 页面与状态矩阵

### 8.1 页面

| 页面 | 主要任务 | 安全约束 |
|---|---|---|
| Service setup | 验证/安装/修复 PKG | 不从 renderer 执行 installer |
| Unclaimed node | 显示 node key/build/claim | 只有本地 claim；无任务入口 |
| Sign in | upstream IdP 登录 | Console 不验证密码 |
| Claim confirmation | Owner/tenant admin 确认 | trusted native digest sheet |
| Work/Shell | Conversation 与 Task 创建 | Agent 内容无系统控件能力 |
| Task center/detail | 监督五轨和 lease | authority projection only |
| Agent source/check | registry/local bundle | 无 URL/Git；unknown 不伪装通过 |
| R1 confirmation | Touch ID/reauth | native sheet；digest-bound |
| Inbox | unknown、pause pending、degradation | ack 不替代处理 |
| System health | daemon/store/audit/watch/floor | readiness 不扩大 capability |
| Update/repair | metadata、PKG、drain、结果 | 不自动下载/提权 |
| Quarantine | key/claim/integrity 故障 | 禁普通写；只允许诊断/重 claim |

### 8.2 通用状态

| 状态 | 展示 | 可用动作 |
|---|---|---|
| `initial-loading` | 与真实结构匹配的静态 skeleton | 无假控件 |
| `authoritative-empty` | authority 确认空集 | 创建/导入的适用入口 |
| `refreshing-last-good` | `as_of`、authority、刷新标记 | 仅 freshness 允许的动作 |
| `partial/redacted` | 范围和缺口 | 依赖缺口的写禁用 |
| `stale-offline` | 非实时 Trust Strip | 所有受保护写禁用 |
| `privacy-locked` | 无敏感正文的遮罩 | 无通知控制 action |
| `reauth-required` | 原因和安全恢复 | 登录后 resnapshot |
| `floor-warning` | recommended minimum | 查看更新；不自动阻断 |
| `floor-blocked` | signed floor 与受影响能力 | 诊断/导出/更新修复 |
| `submitting` | 固定 proposal/ref | 防重复提交 |
| `result-unknown` | 原 binding/dispatch/reconcile | 查询/对账；无 retry |
| `pause-pending` | 请求已接收但未确认 | 查看仍可能发生的 Effect |
| `quarantined` | claim/key/integrity 原因 | revoke/reclaim/repair |
| `completed` | authority acceptance + current verification | 查看证据 |

## 9. Security threat model

必须假设：

- 恶意 Agent、Markdown、文件、bundle、链接和 Unicode/Bidi；
- renderer/WebKit compromise、XPC capability 滥用、系统卡像素仿冒；
- 恶意低权限本机用户、被攻陷的 admin session、fast user switching；
- helper confused deputy、路径替换、symlink、archive bomb、TOCTOU；
- claim 重放、node key 替换、旧机器转让、tenant 混淆；
- update metadata/PKG/镜像/签名 key compromise 和 anti-rollback 绕过；
- Keychain 锁定/取消、notification handle 重放、broker 跨用户路由；
- lock/sleep/crash/network loss 后 lease 错误续期；
- `OUTCOME_UNKNOWN` 盲重试、Runtime stop 被误报为 Task 安全；
- store/audit/watch 不可用和 last-good 冒充当前。

安全目标是阻止不可信组件取得真实 IPC/credential/commit 能力，而不是承诺像素无法仿冒。

## 10. Acquisition 与 secure storage

### 10.1 acquisition allowlist

v1 只允许：

1. authority 可发现且当前允许的 trusted registry ID；
2. 用户通过 `NSOpenPanel`/系统 picker 明确选择的本地签名 bundle。

即使来源允许，也必须：

- 校验 security-scoped access、调用者可读性和实际 object identity；
- 拒绝 symlink/device/FIFO/network mount 等未允许类别；
- 固定 inode/file identity、大小、mtime、archive manifest 和 digest，处理 TOCTOU；
- 限制字节、文件数、展开比、嵌套深度、CPU/时间；
- 验证签名、provenance、manifest、dependency、compatibility、sandbox；
- 无 ambient credential、无自动网络抓取；
- 由 authority 计算 risk floor 并形成独立 Effect。

### 10.2 secure storage degradation

- 用户 token/device-key handle 使用 Keychain；daemon node key 的具体受控存储是 machine-contract/PoC gate。
- Keychain `not found/locked/interaction denied/user canceled/auth failed` 均不是“无秘密”成功。
- 不允许 plist、NSUserDefaults、文件、日志、环境变量或剪贴板 fallback。
- 降级时清除或停止刷新 token，停止 broker lease renewal、R1、claim、acquisition 和受保护写。
- 非敏感连接信息和带 `as_of` 的只读 projection 可保留；恢复后必须重新认证/resnapshot。

## 11. Renderer、WKWebView floor 与 kill switch

- bundled system UI 与不可信内容使用不同 capability；不可信内容没有 XPC/native bridge。
- Markdown parser 默认拒绝 raw HTML、script、event handler、iframe、object/embed、data/file/custom scheme 和自动 remote image。
- 链接显示规范化 host，用户动作后交给系统浏览器；Console 不预取。
- `WKContentWorld` 只是 JS namespace，DOM 仍共享；CSP 和 Tauri isolation 只作纵深防御。
- `WKWebsiteDataStore.nonPersistent` 只减少 website data 持久化，不消除 swap/crash dump 风险。
- floor metadata 固定 minimum macOS build、WKWebView/WebKit security state、Console/daemon/helper version 和撤销信息。
- 低于 signed floor 时 kill switch 禁用不可信富内容、acquisition、claim/R1、受保护写和 lease renewal；只保留 native/安全文本诊断、导出和更新修复。
- kill switch 不修改 authority Task/Effect 状态，也不向已接受任务发送未经授权的 terminate。

## 12. 平台体验、accessibility 与 motion

- CognitiveOS 品牌核心、Trust Strip、Flow Thread 和任务术语跨平台一致。
- macOS 使用系统 app menu、toolbar/sidebar、sheet、Dock、menu extra、Command 快捷键和系统 picker；不复制 Windows Fluent 外观。
- 关键更新先改文字/可访问状态，再执行一次短促 Flow Thread 方向动画；无 authority event 不播放推进。
- Reduce Motion 下取消位移、缩放、parallax、shimmer 和连续 spinner，使用静态替换、轮廓和 live announcement。
- Increase Contrast 下不依赖透明、细边框或颜色；Full Keyboard Access 下所有 action、drawer、sheet 和 picker 返回路径可达。
- VoiceOver live region 合并高速事件，不逐秒朗读 deadline，不因排序移动当前焦点。

GA 必测旅程：安装/claim、登录、创建 Task、R1、pause pending、unknown reconcile、Agent bundle、update/repair、lock/switch user、退出监督。当前均未执行，证据为 `none`。

## 13. Open PoC and GA gates

| Gate | 必须证明 | 当前 |
|---|---|---|
| `MAC-POC-01` | Universal 2 Console/daemon/broker/helper 在候选 OS/CPU 安装、签名、公证和启动 | not-run |
| `MAC-POC-02` | GUI App Sandbox + Hardened Runtime + 最小 entitlements 可通过 XPC 完成必要 IPC | not-run |
| `MAC-POC-03` | renderer compromise 不能访问 daemon/helper/Keychain/R1 capability | not-run |
| `MAC-POC-04` | node key、claim、revoke/reclaim 和机器转让故障模型 | not-run |
| `MAC-POC-05` | lock/switch user/sleep/crash 下 broker session 与 signed lease eligibility | not-run |
| `MAC-POC-06` | Keychain locked/cancelled/missing 无 fallback 且正确收窄能力 | not-run |
| `MAC-POC-07` | Touch ID device-key signature、native display、nonce/session/expiry replay negative | not-run |
| `MAC-POC-08` | PKG/MDM、threshold metadata、anti-rollback、drain/repair 失败恢复 | not-run |
| `MAC-POC-09` | local bundle symlink/archive/TOCTOU/budget/signature 负例 | not-run |
| `MAC-POC-10` | WKWebView floor/kill switch 和纯文本/Markdown 降级 | not-run |
| `MAC-POC-11` | 多用户 broker 通知路由、opaque handle 重放和锁屏隐私 | not-run |
| `MAC-POC-12` | VoiceOver/FKA/Contrast/Reduce Motion 核心旅程 | not-run |

GA 还要求：

- Console 通用 backend gate 已通过；
- 所有 release-blocking platform requirement 有 owner、实现和已执行 oracle；
- 两次 framework/WebKit/helper 升级演练；
- 最新 Apple security/notarization/MDM 事实复核；
- 目标 OS/CPU support matrix 每项有独立证据；
- 无错误完成、跨用户泄露、重复 Effect、floor 下危险写或 R2/R3 降级。

## 14. macOS v1 产品要求与追踪

状态值沿用 Console 三维口径。`partial` 后只列本仓库真实存在的 REQ-ID；平台专属义务若无机器合同即标 `missing`。

| ID | 原子要求 | Contract | Implementation | Evidence | Owner | Oracle | blocked_by |
|---|---|---|---|---|---|---|---|
| `CONSOLE-MAC-V1-PRD-001` | Console/broker/helper 不拥有 authority、IdP、node 或 completion fact | partial: `REQ-SHELL-CHANNEL-001`, `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Console security | 伪造 UI/local cache 不能启用写或改变 authority 状态 | M5 backend; `MAC-POC-03` |
| `CONSOLE-MAC-V1-PRD-002` | Task/Loop/AgentExecution/Runtime/Effect/Verification 分离 | partial: `REQ-RUN-009`, `REQ-EFF-STATE-001` | not-implemented | not-run | UNASSIGNED — Runtime/Console | Runtime stop/remote completed 不改变 Task completion | M2–M5 carriers |
| `CONSOLE-MAC-V1-PRD-003` | machine-wide low-privilege daemon、per-user broker、minimal helper 分权 | missing | not-implemented | none | UNASSIGNED — macOS integration | compromise 任一组件不能取得其他组件权限 | `MAC-POC-01/02/03` |
| `CONSOLE-MAC-V1-PRD-004` | OS admin 与 CognitiveOS Owner/tenant admin 正交 | partial: `REQ-MGMT-SESSION-001` | not-implemented | not-run | UNASSIGNED — Identity | admin-only/owner-only 交叉负例均拒绝 | claim/auth contract |
| `CONSOLE-MAC-V1-PRD-005` | claim 固定 node key/build/tenant digest，异常 revoke/reclaim | missing | not-implemented | none | UNASSIGNED — Identity | key/build/tenant/nonce 任一变化使旧 claim 失效 | `MAC-POC-04` |
| `CONSOLE-MAC-V1-PRD-006` | risk floor 由 authority；仅 R0/R1，R2/R3 无执行入口 | partial: `REQ-MGMT-GATE-001` | not-implemented | not-run | UNASSIGNED — Risk/Console | 客户端降 risk 和 R2/R3 提交均失败 | R1 display contract |
| `CONSOLE-MAC-V1-PRD-007` | signed lease 只由 authority 维持/签发，Console 不本地延长 | missing | not-implemented | none | UNASSIGNED — Supervision | lock/switch/revoke/UI hang 后旧 broker 不能续租 | supervision contract; `MAC-POC-05` |
| `CONSOLE-MAC-V1-PRD-008` | 关闭 UI 与退出监督分离，node/已接受任务不停止 | partial: `REQ-SHELL-DETACH-001` | not-implemented | not-run | UNASSIGNED — Console | window close、explicit exit、force quit 分别得到规定结果 | broker/lease contract |
| `CONSOLE-MAC-V1-PRD-009` | 通知仅 opaque open handle，重认证/resnapshot 且不跨用户 | missing | not-implemented | none | UNASSIGNED — Notification | handle 重放/错用户/锁屏正文全部拒绝 | `MAC-POC-11` |
| `CONSOLE-MAC-V1-PRD-010` | Keychain 失败无明文 fallback，停止 token/lease/R1/写 | missing | not-implemented | none | UNASSIGNED — macOS security | missing/locked/cancelled/auth-failed fixture 均收窄能力 | `MAC-POC-06` |
| `CONSOLE-MAC-V1-PRD-011` | lock/switch user teardown 敏感 UI，恢复后 reauth/resnapshot | missing | not-implemented | none | UNASSIGNED — Privacy | user A projection/token/handle 对 B 不可见 | `MAC-POC-05/11` |
| `CONSOLE-MAC-V1-PRD-012` | 仅安全文本和 allowlist Markdown；raw HTML/script 被剥离 | product-only | not-implemented | none | UNASSIGNED — Renderer | malicious Markdown corpus 无脚本/bridge/自动网络 | `MAC-POC-03/10` |
| `CONSOLE-MAC-V1-PRD-013` | signed security floor 触发选择性 fail closed，不改变 authority task | partial: `REQ-ERR-001` | not-implemented | none | UNASSIGNED — Update/Console | 低于 floor 仅保留诊断/导出/修复，任务状态不变 | floor carrier; `MAC-POC-10` |
| `CONSOLE-MAC-V1-PRD-014` | acquisition 仅 registry ID 和 picker 本地签名 bundle | partial: `REQ-AGENT-INSTALL-001`, `REQ-AGENT-SANDBOX-001` | not-implemented | not-run | UNASSIGNED — Agent lifecycle | URL/Git/private repo 无入口，bundle 负例逐项拒绝 | M6; `MAC-POC-09` |
| `CONSOLE-MAC-V1-PRD-015` | GA 仅 signed/notarized PKG；MAS/DMG 非 GA | product-only | not-implemented | none | UNASSIGNED — Distribution | 官网与 MDM 对同一 digest 制品验签 | `MAC-POC-01/08` |
| `CONSOLE-MAC-V1-PRD-016` | Console 只提示 threshold metadata，不自动下载/提权 | missing | not-implemented | none | UNASSIGNED — Update | 网络响应不能直接触发 download/install/helper | update metadata contract |
| `CONSOLE-MAC-V1-PRD-017` | normal stop 先 authority drain；emergency unknown 后对账 | partial: `REQ-EFF-004`, `REQ-MGMT-RECOVERY-001` | not-implemented | not-run | UNASSIGNED — Recovery | unknown Effect 阻止 clean stop claim，强停后无盲重试 | M4/M5 |
| `CONSOLE-MAC-V1-PRD-018` | rollback/repair 不低于 floor，不回滚 authority state | partial: `REQ-MGMT-IDEM-001` | not-implemented | not-run | UNASSIGNED — Update/Recovery | vulnerable rollback、state restore、key anomaly 均拒绝 | `MAC-POC-08` |
| `CONSOLE-MAC-V1-PRD-019` | Touch ID 只解锁 device key；authority 验 digest/nonce/session/expiry | missing | not-implemented | none | UNASSIGNED — R1 security | replay、stale digest、wrong session、cancel 全拒绝 | signed display contract; `MAC-POC-07` |
| `CONSOLE-MAC-V1-PRD-020` | Hardened Runtime 必需，GUI App Sandbox 为 GA gate | product-only | not-implemented | none | UNASSIGNED — macOS security | entitlement diff 最小且 XPC/picker journey 可用 | `MAC-POC-02` |
| `CONSOLE-MAC-V1-PRD-021` | macOS 14+ Universal 2 仅在实时 security floor 内支持 | product-only | not-implemented | none | UNASSIGNED — Release | 每个 OS/CPU 组合有 build、security、upgrade evidence | `MAC-POC-01/08/10` |
| `CONSOLE-MAC-V1-PRD-022` | VoiceOver/FKA/Contrast/Reduce Motion 完成核心旅程 | product-only | not-implemented | none | UNASSIGNED — Accessibility | 辅助技术人工+自动走查均有证据 | `MAC-POC-12` |
| `CONSOLE-MAC-V1-PRD-023` | 用户任务语言优先，machine enum/ref/digest 保真 | product-only | not-implemented | none | UNASSIGNED — Content design | zh-CN/en fixtures 无信息损失或句子拼接 | content/a11y evidence |
| `CONSOLE-MAC-V1-PRD-024` | specified/implementation/test/Profile 四态在 UI/文档分离 | product-only | not-implemented | none | UNASSIGNED — Traceability | 无 evidence 时 UI/manifest 不显示 implemented/pass | M1 runner; platform evidence |

## 15. 官方来源 ledger

以下均查询于 2026-07-20；平台事实不自动成为 CognitiveOS machine contract。

| 标题 | URL | 用于核实 |
|---|---|---|
| Service Management | https://developer.apple.com/documentation/servicemanagement | login item/LaunchAgent/LaunchDaemon |
| SMAppService | https://developer.apple.com/documentation/servicemanagement/smappservice | macOS 13+ helper 注册与用户批准 |
| XPC | https://developer.apple.com/documentation/xpc | process/lifecycle/IPC 分类 |
| Creating Launch Daemons and Agents | https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html | launchd user/system context（Apple archive） |
| Authorization Services | https://developer.apple.com/documentation/security/authorization-services | OS authorization 与 App Sandbox 限制 |
| Designing Secure Helpers and Daemons | https://developer.apple.com/library/archive/documentation/Security/Conceptual/SecureCodingGuide/DesigningSecureHelpers/DesigningSecureHelpers.html | helper 互不信任/最小权限（Apple archive） |
| setCodeSigningRequirement(_:) | https://developer.apple.com/documentation/foundation/nsxpcconnection/setcodesigningrequirement(_:) | XPC peer signing requirement |
| Keychain services | https://developer.apple.com/documentation/security/keychain-services | 加密 secret storage |
| Protecting keys with the Secure Enclave | https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave | 硬件/P-256/不可导入限制 |
| Local Authentication | https://developer.apple.com/documentation/localauthentication | Touch ID 只返回认证结果 |
| Passkeys | https://developer.apple.com/passkeys/ | upstream IdP/passkey 平台事实 |
| Hardened Runtime | https://developer.apple.com/documentation/security/hardened-runtime | notarization 前提和 exception entitlements |
| Accessing files from the macOS App Sandbox | https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox | picker/security-scoped bookmark |
| Notarizing macOS software before distribution | https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution | Developer ID/notary/stapling |
| Packaging Mac software for distribution | https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution | PKG/DMG 与多组件产品 |
| Gatekeeper and runtime protection in macOS | https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/web | Gatekeeper 检查与用户/MDM 边界 |
| Distribute packages to Mac computers | https://support.apple.com/guide/deployment/distribute-custom-packages-for-mac-dep873c25ac4/web | MDM 安装 signed PKG |
| MenuBarExtra | https://developer.apple.com/documentation/swiftui/menubarextra | menu extra 与移除行为 |
| NSApplication.ActivationPolicy | https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum | Dock/accessory/background |
| UNUserNotificationCenter | https://developer.apple.com/documentation/usernotifications/unusernotificationcenter | 通知授权与 action |
| WKWebsiteDataStore | https://developer.apple.com/documentation/webkit/wkwebsitedatastore | persistent/nonpersistent web data |
| WKContentWorld | https://developer.apple.com/documentation/webkit/wkcontentworld | JS namespace 非 DOM 隔离 |
| VoiceOver User Guide for Mac | https://support.apple.com/guide/voiceover/welcome/mac | VoiceOver |
| Change Keyboard settings for accessibility on Mac | https://support.apple.com/guide/mac-help/change-keyboard-settings-for-accessibility-mchlae61a6de/mac | Full Keyboard Access |
| Change Display settings for accessibility on Mac | https://support.apple.com/guide/mac-help/change-display-settings-for-accessibility-unac089/mac | Increase Contrast |
| Customize onscreen motion on Mac | https://support.apple.com/guide/mac-help/customize-onscreen-motion-mchlc03f57a1/mac | Reduce Motion |
| About software updates for Apple devices | https://support.apple.com/guide/deployment/about-software-updates-depc4c80847a/web | 旧 OS 不保证覆盖全部已知安全问题 |
| Designing for macOS | https://developer.apple.com/design/human-interface-guidelines/designing-for-macos | Apple HIG 入口；本轮抓取返回 unknown-error shell，未据此编造具体规范 |
| Tauri Capabilities（third-party project） | https://v2.tauri.app/security/capabilities/ | 技术候选的 IPC scope；不覆盖 WebView 0-day |

## 16. 状态声明

- macOS v1 产品决策：已记录。
- macOS machine contract：仅有部分通用 REQ 锚点；平台专属合同未登记。
- Console/macOS implementation：未提供。
- macOS PoC、a11y、security、distribution tests：未执行。
- 相关既有 conformance vectors：依据 PROGRESS 全部 `not-run`。
- macOS Console Profile：未符合。
