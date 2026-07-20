# CognitiveOS Console v2 — 可信与安全体验

> 状态：Draft / planned
>
> 范围：Windows v1（R0/R1）
>
> 地位：产品行为。机器安全义务必须由 normative contract 和实现证据闭合。
>
> Agent Hub 关系：本文覆盖 Governed R0/R1 信任模型。Direct Takeover 的接管威胁模型、本机控制面、凭据与电脑控制在 [agent-hub/security/](../../../../apps/cognitiveos-console/docs/agent-hub/security/)，其 `host-managed`/`*-observed` 事实来源不得升级为 authority。

## 1. 威胁模型

Windows v1 默认面对：

- 恶意或被攻陷的 Agent；
- prompt injection、网页、文件、日志和模型输出；
- 伪造系统卡/错误/完成状态的内容；
- XSS、DOM 污染、renderer compromise 和 IPC 滥用；
- 本机低权限用户抢占共享节点 Owner；
- 恶意/来源未知 Agent 包和供应链替换；
- session、preview、challenge、deep link 重放；
- Service/store/audit/watch 降级；
- 断线后任务或 Effect 仍在推进；
- 结果未知时重复执行；
- 锁屏、剪贴板、通知和诊断泄露。

安全目标不是让不可信内容“视觉上无法模仿”系统界面。像素可以被仿造；可验证目标是：不可信内容不能取得真实系统语义、凭证、IPC、焦点路径或提交能力。

## 2. 信任边界

```text
不可信数据
  Agent 文本 / 网页 / 文件 / 日志 / package source
        │
        ▼
受限数据 renderer
        │  独立低权限 WebView/进程；无 management IPC、无 secure storage、无系统动作
        ╳  不与系统 renderer 共享安全边界
Console system renderer / native surface
        │  固定 proposal ref / session / channel / workspace
        ▼
受控原生 Host
        │  versioned allowlist IPC
        ▼
CognitiveOS Windows Service / authorities
```

边界规则：

- 不可信数据 renderer 不持有节点账号验证材料、Service 管理权限、authority signing key 或任何写 IPC；密码、bootstrap 和 R1 位于独立系统 renderer/native surface。
- Task 与 management presentation 可以复用无副作用组件，不复用 credential、store、cache、session 或 IPC capability。
- Console 本地状态只能是缓存/草稿/投影；没有 authority ref/version 的对象不能启用受保护动作。
- Admin CLI 是独立恢复边界；Console 不能把自然语言或 Shell proposal 强转为 CLI/management command。
- CognitiveOS Owner 与 Windows 管理员是正交身份。Service 安装/卸载/stop/restart、机器级更新、bootstrap bundle mint、信任重置和恢复需要 OS token/UAC、签名来源与 anti-rollback，不能仅凭节点 Owner session。

## 3. 身份、TOFU 与账号

### 3.1 节点身份

- 普通 loopback 首连允许 TOFU，记录节点 key identity、Service 发布者和本机安装关联。
- Owner bootstrap 不把 secret 发送给只靠 TOFU 接受的未知端点：安装器/Admin CLI bundle 必须预置 endpoint key。
- 首次信任页面必须与 Owner bootstrap 分开说明：信任节点不等于拥有账号或权限。
- 身份变化阻断登录与敏感 projection；显示旧/新身份、最近已知更新时间和恢复入口。
- 普通警告或 R1 按钮不能覆盖节点身份变化。

### 3.2 Owner bootstrap

- 一次性 bootstrap bundle 必须由安装器/Admin CLI 产生，短时、单次并绑定 endpoint key、目标 Windows SID 和本机节点。
- Console 先验证 endpoint key；Service 从受认证 IPC peer token 派生 SID，禁止信任请求字段自报 SID。
- 领取请求绑定派生 SID 和新账号，secret 只发送给匹配 endpoint key 的 Service。
- 并发领取使用 authority CAS；失败方重新查询，不覆盖已存在 Owner。
- bootstrap secret 不进入日志、遥测、剪贴板历史或通知。

### 3.3 登录与 session

- 节点验证本地账号密码，Console 只接收短期 AuthenticationSession。
- 密码不能被用于 R1/R2/R3 approval。
- session 绑定 principal、节点、Windows SID、channel、issued/expiry/revocation epoch。
- session 过期/撤销后清理敏感内存 projection；草稿只保留用户明确允许的非敏感内容。
- Admin CLI recovery 后所有受影响 session 和 bootstrap/recovery material 失效。

上述字段是产品所需合同，不表示当前 schema 已登记。

## 4. 不可信内容与系统控件

### 4.1 数据容器

- Markdown/HTML 使用独立低权限 WebView/进程（或等价隔离）、allowlist sanitizer、CSP、Trusted Types（可用时）和 opaque/isolated origin；无法证明隔离时降级为纯文本/安全子集。
- 链接显示目标并经 scheme/host policy；不允许 `file:`, arbitrary custom scheme 或 credential-bearing URL。
- 复制、下载、打开外部程序均显示来源和敏感度。
- Unicode/Bidi/同形字符不改变 canonical bytes；可信字段隔离方向控制符并显示稳定 ID。

### 4.2 系统容器

- 只有 Console 可以实例化 System Card、Trust Strip、Command Preview 和真实动作。
- 系统控件从 authority projection 构建；Agent 提供的 label/icon/style 只能进入数据字段。
- 系统控件位于独立 system renderer/native surface，拥有独立可访问名称、焦点顺序和原生 Host action binding；React component/DOM 边界本身不是安全边界。
- Renderer compromise 测试必须证明 task 内容无法调用 management/Service/secure-storage IPC。

## 5. Proposal 展示完整性

### 5.1 Canonical display profile

每个可确认 proposal 需要一个版本化 display profile。Windows v1 R1 至少显示：

- proposal ref/version/digest 短码；
- principal、节点和 channel；
- 固定目标/参数摘要/expected version；
- 外部影响与 risk floor；
- 数据类别、目的地/egress（适用时）；
- 预算、deadline、capability expiry；
- 任务失联后行为和 supervision lease；
- 验证/验收；
- 取消、pause、stop、reconcile、compensation 的适用与限制；
- preview expiry。

任何必填字段缺失、unknown、版本不一致或无法绑定时，确认入口 fail closed。

当前 `ManagementActionProposal` 只有 `proposal_digest` 等字段，没有完整 signer/signature/canonical display contract，且禁止附加字段。产品文档只能把签名/显示完整性列为 release blocker，不能称为现有 schema 已支持。

### 5.2 Stale 与 supersede

- 目标、参数、digest、risk、预算、egress、deadline、权限或版本变化使旧 preview stale。
- “修改范围”创建 superseding proposal；旧 challenge/确认不可复用。
- UI 必须显示差异，不能在原卡静默替换内容。

## 6. R0/R1 安全交互

### 6.1 R0

- 外部可观察写不能被客户端/Agent 标成 R0。
- authority 允许的内部治理记录可以自动提交，提交后在 Activity 可见。
- R0 失败不能被 UI 当成“无事发生”；显示安全可披露的失败与 correlation。

### 6.2 R1

- 确认控件由 Console 系统容器渲染。
- 初始焦点在标题/变化摘要；批准按钮不是默认 Enter 行为。
- 按钮使用完整动作，例如“升级 Agent 到 v2.1”。
- Cancel/Escape/返回均为无决定退出。
- 禁止滑动批准、长按作为唯一确认、单键批准或通知 action 批准。
- number matching 仅在 authority policy 要求时出现，且一次性、digest-bound、可访问。

### 6.3 R2/R3

- Windows v1 不执行。
- UI 只解释阻断原因、缺少的可信确认能力和可缩小范围路径。
- 不提供隐藏设置、开发模式、密码确认或“我理解风险”旁路。

## 7. 监督、暂停与紧急遏制

### 7.1 监督 lease

- Shell 创建任务前显示是否需要持续监督、lease 周期、grace 和安全检查点。
- Console 只显示 Service 返回的 lease 状态，不用本地心跳动画冒充。
- 续租资格绑定 task/principal/SID/logon-session/channel/client-epoch，并要求 AuthenticationSession 当前、watch freshness 满足门禁且 UI health 可响应；旧实例、锁屏、用户切换、session revoke、watch stale 和 UI hang 不能继续续租。
- lease 过期后可能出现：
  - pause pending；
  - 已在安全检查点暂停；
  - 无法暂停，仍有不可中断 Effect；
  - Service 不可达，状态未知。
- 任何状态都必须给出用户可理解的不保证事项。

### 7.2 动作不混同

| 用户动作 | 作用对象 | 不保证 |
|---|---|---|
| 取消请求 | 未开始/可取消的任务意图 | 不保证已 dispatch Effect 未发生 |
| 暂停任务 | Task/Loop 的后续推进 | 不保证当前外部 Effect 立即停止 |
| 停止载体 | Runtime/PID/container | 不完成 Task，不收敛 Effect |
| 终止执行 | AgentExecution（合同待闭合） | 不证明世界安全 |
| 撤销能力 | 未来授权 | 不自动撤回既有授权/dispatch |
| 对账 | Effect | 不执行新 Effect |
| 补偿 | 新的受治理 Effect | 不等于原 Effect 未发生 |
| 隔离 | 指定状态域 | 不等于所有关联对象终止 |

### 7.3 store/audit 降级

- 普通任务/Agent lifecycle/配置/账号/bootstrap/recovery/Service update/trust reset 全部 fail closed。
- 紧急 pause/stop 只有在 store 健康时预铸的限时遏制 capability 仍可验证、绑定目标/version/fencing、policy 明确允许且独立应急日志可持久化时继续；`stop` 不自动等于降低风险。
- 无法验证授权、撤销或 fencing 时，仅允许 Service 自身已登记的 lease-expiry safe pause。
- UI 显示“应急操作已接收/结果未知/已对账”，不直接显示原任务已暂停。
- audit readiness 恢复后，authority 建立关联记录；不能补造失败期间不存在的签名/时间。

| 操作 | store/audit degraded 时 | 前提 |
|---|---|---|
| 查看最后持久状态 | 只读允许 | 显示 `as_of`、完整性缺口和不可达范围 |
| Service 自动 lease-expiry pause | 条件允许 | 预登记策略、目标/fencing 固定、应急日志可写 |
| 用户请求 pause/stop | 条件允许 | store 健康时预铸的限时遏制 capability 仍可验证；policy 明确该 target/action 降低风险 |
| 新任务、恢复任务、Agent lifecycle、配置 | 拒绝 | 普通 governed write fail closed |
| 账号/bootstrap/recovery、trust reset | 拒绝 | 身份与审计不可闭合 |
| Service install/update/uninstall | 拒绝 | OS 提权与 anti-rollback 不足以替代 authority audit |
| capability revoke/fence 变更 | 默认拒绝 | 除非未来合同给出独立 authoritative emergency path |

## 8. 结果未知与幂等

- `OUTCOME_UNKNOWN` 是安全状态，不是 error toast。
- 页面保留原 Intent、idempotency key/parameter digest、dispatch evidence、责任主体和最后 reconcile。
- 禁止：
  - 更换 key 再执行；
  - 假定 timeout 等于失败；
  - 停止 Runtime 后清除 Effect；
  - 客户端本地合并相似请求。
- same-key same-parameters 是否允许“resume”必须以 Effect transition/machine contract 为准。当前 transition 没有从 `RECONCILED(still_unknown)` 回到执行的明确边，产品文档不自行发明。

## 9. Agent 包与供应链

### 9.1 来源

- URL/Git/本地路径都作为不可信 source descriptor。
- Acquisition 自身是受治理 proposal/Effect，在内容风险判定前由无 ambient credential 的低权限 broker 执行。
- Broker 必须校验调用者对本地路径的访问，阻断未经允许的 UNC/设备路径/凭证转发，执行 URL/Git SSRF/redirect/DNS/network policy，并施加字节、时间、文件数和解压预算。
- 重定向、Git ref、submodule/LFS、archive path 和内容 digest 固定后才可复现检查。
- 最近使用和用户输入不构成 trust。

### 9.2 证据

分开显示：

- publisher/signature；
- provenance/构建来源；
- manifest/声明；
- dependencies/SBOM；
- static analysis；
- adapter/sandbox；
- host-specific negative evidence；
- compatibility；
- authority risk floor。

未知字段不能被客户端补齐或猜测。Windows 证据不外推到 macOS/Linux。

### 9.3 生命周期

- 安装/升级/回滚/卸载均形成固定 proposal、Effect 和版本化 AgentInstallation。
- 失败不覆盖旧 installation。
- 回滚不能降到 security floor 以下。
- 卸载不删除未决 Effect、历史引用、审计或被其他对象依赖的数据。
- arbitrary source 只有 authority 判为 R0/R1 且 gate 完整时可运行。

## 10. 本地缓存、锁屏与剪贴板

### 10.1 内存快照

- 应用不主动把敏感 snapshot 持久化到磁盘；只在当前已解锁会话的受控 renderer/内存中使用；
- 登出、session revoke、账号切换、进程退出或锁屏时 teardown 敏感 renderer、零化应用管理的 buffer，并清理临时 profile；
- 隐私遮罩只是视觉防护，不能替代 teardown/清理；
- 解锁后进入 `reauth-required`，重新认证/授权并 resnapshot，不恢复旧像素或旧 DOM；
- WebView2 UDF/cache/DOM storage、Windows pagefile/hibernation 和 crash dump 仍可能形成 OS 管理的残留；通过临时/隔离 profile、关闭不必要 storage、退出清理和 dump/pagefile policy 收窄，并作为 release gate 验证。

### 10.2 持久数据

可持久化：

- 非敏感节点 alias/identity pin；
- UI 偏好、语言、密度、窗口布局；
- 无正文稳定 refs（须按账号/节点隔离）；
- crash-safe 但不含 secret 的诊断元数据。

禁止持久化：

- 密码、bootstrap secret、approval input；
- management credential；
- Conversation/Task/Effect 敏感正文；
- break-glass 内容；
- 未经策略批准的附件和剪贴板。

### 10.3 导出/剪贴板

- 复制 digest/ID 与复制正文是不同动作；
- 敏感复制显示范围和清除策略，不承诺平台无法截取；
- Windows 通知、任务栏和托盘不显示正文/节点敏感 alias；
- crash/diagnostic 包生成前提供字段预览。

## 11. 通知与深链

- Windows 通知只做 hint，不是 authority、session 或 bearer。
- Session 0 的 Service 把脱敏事件按目标 SID 路由给每用户 notification broker；broker 负责 AUMID/Windows activation，用户未登录时不尝试跨 session 投递。
- payload 只使用高熵、一次性 opaque handle；不携带稳定 item ref、正文、approval secret 或 credential。
- 打开后验证 app、principal、SID/logon-session、节点、workspace、audience、expiry 和版本；原子消费 handle、解析稳定 item ref，再 resnapshot。重放失败。
- duplicate 由稳定 event/proposal ref 去重。
- 同一 Task 的普通进度可按策略聚合；`OUTCOME_UNKNOWN`、pause pending、session revoke、R1 决定和系统持久化故障不被聚合吞掉。
- quiet hours 只抑制声音/banner/push 呈现，不改变底层事项、deadline 或 authority 状态；安全策略可对临近 deadline/持续未知/账号事件升级提示。
- `read/acknowledged` 只改变提醒；底层 `handled/decided/reconciled` 来自 authority。
- 通知权限关闭、token/registration 失败在设置与系统概览可见。

## 12. 错误与完整性

- 保留已登记 error code、retryable 和安全 details ref；未知 code 标为未登记服务错误。
- 当前通用 error schema 没有 Core prose 要求的 `safe_reason`、`correlation_id`、`retry_after`，且 `additionalProperties:false`。Console 不得声称这些字段已经可由现有 schema 传输。
- 审计/记录完整性至少区分：
  - 权限裁剪；
  - 延迟/尚未到达；
  - 记录缺失；
  - digest/signature 无效；
  - trust root 未知/过期；
  - Service 不可达。
- UI 不补造缺失记录，不把客户端日志当 authority audit。

## 13. 安全验收

未来实现必须覆盖：

- 低权限 Windows 用户抢占 Owner；
- bootstrap bundle endpoint-key/目标 SID 不匹配、secret 重放/泄露/并发；
- TOFU 首次与 identity rotation；
- XSS/prompt injection/像素仿冒/IPC fuzz；
- 同 WebView DOM 隔离假设失败、task renderer 调用 Service/management/secure storage；
- preview 字段删除、digest mismatch、stale/replay；
- number matching 键盘/Narrator/超时/重发；
- lease 在多实例、锁屏、用户切换、session revoke、watch stale、UI hang、断网、崩溃、睡眠和关机下到期；
- state store/audit/emergency log 故障；
- arbitrary source SSRF/UNC/ambient credential/path 权限/resource exhaustion、替换、redirect、digest drift、sandbox bypass；
- `OUTCOME_UNKNOWN` 盲重试、换 key、停止 Runtime；
- 锁屏 renderer teardown、用户切换、session revoke、pagefile/hibernate/crash dump、per-user broker/通知预览/handle replay；
- Unicode/Bidi/同形字符和超长文本。

在对应 machine contract、实现和已执行证据存在前，以上均保持 `blocked` 或 `planned`。
