# Agent Hub — 页面状态、内容与无障碍

> 类别：informative product design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 无障碍目标：WCAG 2.2 AA + 各平台原生辅助技术；当前 evidence `none`，验收为未来门禁。

## 1. 正交状态模型

不得把所有情况压成一个 `running`。每个可写页面从以下正交维度组合状态：

- Mode：`direct` / `governed`
- Connectivity：`current` / `refreshing-last-good` / `stale` / `offline`
- Host：`ready` / `locked` / `restarting` / `incompatible` / `unavailable`
- Takeover：`managed-from-start` / `officially-adopted` / `terminal-attached` / `read-only-observed` / `unmanaged-observed` / `unsupported` / `blocked-by-policy`
- Ownership：`none` / `requested` / `active` / `lost` / `superseded`
- Provider/session：`connecting` / `authenticating` / `ready` / `waiting-user` / `interrupted` / `missing`
- File observation：`current` / `locked` / `partial` / `corrupt` / `version-unknown`
- Permission：`pending` / `local-confirm-required` / `approved` / `denied` / `expired` / `superseded`
- Outcome：`agent-reported-done` / `process-exited` / `checks-pass` / `checks-fail` / `user-accepted` / `unknown`
- Credential：`healthy` / `reauth` / `revoked` / `expired`
- Adapter：`compatible` / `degraded` / `incompatible` / `update-required`

## 2. 必备页面状态

所有主要页面至少覆盖：`no-host`、`no-agent`、`no-account`、`initial-loading`、`scanning`、`takeover-candidate-detected`、`takeover-consent-required`、`managed-from-start`、`officially-adopted`、`terminal-attached`、`read-only-observed`、`unmanaged-observed`、`process-identity-changed`、`ownership-lost`、`session-file-locked`、`session-file-partial`、`session-file-corrupt`、`session-schema-unknown`、`adapter-incompatible`、`authenticating`、`ready`、`running`、`waiting-user`、`stale`、`offline`、`rate-limited`、`quota-exhausted`、`cancelling`、`stopping`、`result-unknown`、`conflict`、`superseded`、`agent-reported-done`、`checks-observed`、`user-accepted`、`credential-revoked`、`update-required`、`unsupported`、`blocked-by-policy`。

规则：状态不得只靠颜色、图标或动画表达；必须同时提供文字、结构和可访问名称。Skeleton 不含可点击假控件；refresh last-good 时按 freshness/authority 决定动作是否禁用。

## 3. 内容与术语

- 用户任务语言优先；机器 enum、错误码、REQ-ID、digest 保留原文并旁附本地化解释。
- Direct 产品对象使用独立词汇，不复用 CognitiveOS 机器术语：`WorkItem`（非 Task）、`ManagedAgentRun`（非 AgentExecution）、`HostActionRecord`（非 Effect）、`CheckObservation`（非 Verification）、`UserDisposition`（非 Acceptance）、`Local Event Ledger`（非 authority audit）、`HostControlGrant`（非 capability）。
- 完成语言分开：`Agent reported done` / `Process exited` / `Checks observed pass/fail` / `User accepted` / `Result unknown`；不显示 “Verified/CognitiveOS completed”。
- 中英文文案从统一术语表生成；不拼接半句。首发语言 `zh-CN` 与 `en`。
- 不可信内容（terminal/Markdown/HTML/日志/包来源）在隔离低权限面渲染；系统控件由 Console 受控组件构建，二者不共享安全边界。

## 4. 跨端无障碍验收（未来门禁）

不使用百分比目标；验收标准是每个关键旅程均能完成且不降低确认强度。

### 4.1 PC（Windows v1）

- 全功能纯键盘可达、无键盘陷阱；modal/drawer 初始焦点在标题，Esc/Cancel 无决定退出，关闭后焦点回触发控件，背景 `inert`。
- Narrator 读出：mode、Host、takeover status、fact source、freshness、permission 目标与后果、pause/cancel/unknown 的真实状态。
- 高速终端/watch 事件聚合播报，不逐行朗读、不逐秒朗读 deadline。
- Windows 100% 与 225% 系统文本缩放；WebView/markup 400% reflow；全部 contrast themes；reduced motion 等价。
- 指针目标：普通 40×40 epx，touch-optimized 44×44 epx，高后果动作增加间距。
- raw terminal 提供进入/退出说明、可暂停流、结构化摘要与纯文本替代，不持续抢焦点。

### 4.2 iPhone

- 44×44 pt 触控目标；VoiceOver、Voice Control、Switch Control、Full Keyboard Access、外接键盘完成配对、Host 切换、permission、pause/cancel、unknown、revoke。
- 最大 Dynamic Type 不截断关键动词/takeover status/Host identity；portrait 与 landscape 功能一致；Reduce Motion 静态等价。
- QR 页同时提供可读短码、文本说明和非扫码路径。

### 4.3 Android phone

- 48×48 dp 触控目标；TalkBack、Switch Access、Voice Access、外接键盘完成全部关键旅程。
- Android 14+ 200% font 与每台设备最大 Display size 分别测试；high contrast/color correction 保留状态与焦点；portrait/landscape/IME/cutout/predictive back 不遮挡动作；Remove animations 静态等价。
- Compose 使用真实 role/state/action semantics，不使用 Web `aria-*` 术语。

### 4.4 跨端 WCAG 2.2

- 400% reflow 不丢信息或动作（等效 320 CSS px 宽度，除必要二维界面）。
- 焦点不被 sticky Host bar、底部导航、IME 或 toast 完全遮挡（SC 2.4.11）。
- 非焦点状态变化通过可编程状态消息通知辅助技术（SC 4.1.3），但不过度播报。
- DAG、任务板、Agent 排序、worktree 列表提供 Move up/down、菜单或目标选择，不只拖拽（SC 2.5.7）。
- 登录/配对允许扫码、粘贴、密码管理器等低认知负担路径（SC 3.3.8）；不锁定屏幕方向（SC 1.3.4）。
- 颜色、动画、位置、图标均不能是唯一状态信号；高风险确认按钮不获初始焦点、不绑定默认 Enter。

## 5. 外部无障碍依据

平台无障碍事实以 [../sources/platform-security-ledger.md](../sources/platform-security-ledger.md) 和现有平台产品设计为准（Apple HIG、Android/Material、Microsoft/Windows、WCAG 2.2，查询日 2026-07-20）。证据不跨平台外推。
