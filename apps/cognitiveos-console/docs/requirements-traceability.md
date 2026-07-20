# CognitiveOS Console v2 — 产品要求与追踪

> 状态：Draft / planned
>
> 地位：Informative product traceability
>
> 规则：本文件不新增 CognitiveOS normative `REQ-*`，也不把文件存在误写为实现或测试证据。
>
> Agent Hub 关系：Agent Hub 产品要求用独立 `CONSOLE-AGENTHUB-V1-PRD-*` 命名空间，追踪见 [agent-hub/traceability/product-requirements.md](./agent-hub/traceability/product-requirements.md)，同样不进入 normative registry，不与本文 `CONSOLE-V2-*` 混编。

## 1. 三维状态

每项产品要求分别记录：

### Contract

- `registered`：适用机器行为已由 registry/schema/transition/vector 登记；
- `partial`：存在相关资产，但不能完整表达产品行为；
- `prose-only`：只在 normative/informative prose 中出现；
- `missing`：需要新机器合同；
- `product-only`：纯客户端产品/可访问性要求，不要求成为 Core 合同。

### Implementation

- `not-implemented`：Console 或所需后端能力未实现；
- `partial`：存在实现骨架或部分路径；
- `available`：适用实现存在并可构建；不代表行为已证明。

### Evidence

- `none`：无可执行测试资产或结果；
- `not-run`：测试/vector 已被 runner 枚举，但未执行；
- `pass` / `fail`：已执行并保留可定位证据；
- `not-applicable`：有绑定声明范围的理由；
- `documented-degradation`：适用行为降级，必须同步收窄 scope/profile claim。

Profile 声明不属于 Evidence 档位，单独使用 `planned/experimental/implemented/unsupported`。Windows v1 当前 Profile 为 `planned`；Console `not-implemented`；相关既有 vectors 在 M0 runner 中为 `not-run`，产品级端到端证据为 `none`。仓库其他参考实现骨架不改变该结论。

macOS/Linux 平台专属产品要求使用独立 namespace，见 [桌面平台产品设计](../../../docs/platforms/README.md)。这些要求不进入 CognitiveOS normative registry；若平台行为缺少机器合同，必须标为 `missing/planned/blocked`。

## 2. v2 产品要求

| ID | 原子要求 | Contract | Implementation | Evidence | 主要规格 |
|---|---|---|---|---|---|
| `CONSOLE-V2-PRD-001` | Console 不拥有 authority、commit、root、账号验证或生命周期事实 | partial | not-implemented | none | [brief](./product-brief.md)、[trust](./trust-safety-ux.md) |
| `CONSOLE-V2-PRD-002` | Task 与 management channel 的 session、store、cache、proposal、approval 和 audit material 隔离 | partial | not-implemented | not-run | [trust §2](./trust-safety-ux.md#2-信任边界) |
| `CONSOLE-V2-PRD-003` | Session 短期、有界、SID/node/channel-bound、可撤销，重连不静默恢复特权 | partial | not-implemented | not-run | [scope §3](./windows-v1-scope.md#3-首次设置与账号) |
| `CONSOLE-V2-PRD-004` | 每个写操作使用固定 proposal/preview/gate/Effect/authority result | partial | not-implemented | not-run | [journey 003](./journeys-and-screens.md#console-v2-jrn-003-从-shell-创建任务) |
| `CONSOLE-V2-PRD-005` | 所有生命周期状态只来自 authority projection，客户端/Agent 文本不改变事实 | partial | not-implemented | none | [brief §5](./product-brief.md#5-产品原则) |
| `CONSOLE-V2-PRD-006` | AgentExecution 与 Runtime/进程载体分别展示和控制 | partial | not-implemented | none | [PAGE-007](./journeys-and-screens.md#console-v2-page-007-任务详情) |
| `CONSOLE-V2-PRD-007` | Task/Loop/AgentExecution/Effect/Verification 状态分离，用户摘要可展开到机器轨道 | partial | not-implemented | none | [Flow Thread](./design-system.md#console-v2-cmp-005-governed-flow-thread) |
| `CONSOLE-V2-PRD-008` | `CANDIDATE_COMPLETE` 只有在当前 Verification 和 AcceptanceDecision 均满足时才显示 `COMPLETED` | partial | not-implemented | not-run | [PAGE-007](./journeys-and-screens.md#console-v2-page-007-任务详情) |
| `CONSOLE-V2-PRD-009` | cancel/pause/stop runtime/terminate/revoke/reconcile/compensate/quarantine 不混同 | partial | not-implemented | not-run | [trust §7](./trust-safety-ux.md#7-监督暂停与紧急遏制) |
| `CONSOLE-V2-PRD-010` | Watch 使用授权 snapshot + cursor/delta；gap/stale/权限变化先 resnapshot | partial | not-implemented | not-run | [scope §7](./windows-v1-scope.md#7-离线与降级) |
| `CONSOLE-V2-PRD-011` | 写目标固定稳定 ref/version/digest；歧义、漂移和权限变化 fail closed | partial | not-implemented | not-run | [IA §7](./information-architecture.md#7-搜索与对象定位) |
| `CONSOLE-V2-PRD-012` | 节点、账号、channel 和权限版本变化时相关 cache/watch/preview 原子失效 | partial | not-implemented | not-run | [trust §2](./trust-safety-ux.md#2-信任边界) |
| `CONSOLE-V2-PRD-013` | 错误保留已登记 code/retryable；未知 code 不伪装成已知结果 | partial | not-implemented | none | [trust §12](./trust-safety-ux.md#12-错误与完整性) |
| `CONSOLE-V2-PRD-014` | readiness 只开放节点实际声明且用户有权使用的确定性能力 | prose-only | not-implemented | none | [IA §3](./information-architecture.md#3-启动落点) |
| `CONSOLE-V2-PRD-015` | Agent 安装/升级/回滚/卸载保留验证、风险、事务、旧版本和未决 Effect | partial | not-implemented | not-run | [journey 007](./journeys-and-screens.md#console-v2-jrn-007-安装升级回滚或卸载-agent) |
| `CONSOLE-V2-PRD-016` | Windows v1 关键旅程满足 WCAG 2.2 AA、Narrator、键盘、High Contrast、缩放和 reduced motion | product-only | not-implemented | none | [design §7](./design-system.md#7-无障碍) |
| `CONSOLE-V2-PRD-017` | 离线/不可达时所有写禁用，last-good 标 `as_of`，不可达不推断任务已暂停/停止 | partial | not-implemented | not-run | [scope §7](./windows-v1-scope.md#7-离线与降级) |
| `CONSOLE-V2-PRD-018` | Windows 通知只包含脱敏 hint，不携带 credential/正文且不能直接执行 R1 | missing | not-implemented | none | [trust §11](./trust-safety-ux.md#11-通知与深链) |
| `CONSOLE-V2-PRD-019` | 协议/schema/security floor 不兼容时相关能力 fail closed | partial | not-implemented | not-run | [scope §10](./windows-v1-scope.md#10-技术候选与-release-gate) |
| `CONSOLE-V2-PRD-020` | authority 计算风险下界；Windows v1 只执行 R0/R1，R2/R3 不降级 | partial | not-implemented | none | [scope §5](./windows-v1-scope.md#5-风险与写入范围) |
| `CONSOLE-V2-PRD-021` | 节点拥有并验证本地账号密码；Console 不保存密码哈希或成为 IdP | missing | not-implemented | none | [scope §3](./windows-v1-scope.md#3-首次设置与账号) |
| `CONSOLE-V2-PRD-022` | 共享节点运行于独立 Windows Service；renderer 只能使用版本化 allowlist IPC | missing | not-implemented | none | [scope §2](./windows-v1-scope.md#2-部署边界) |
| `CONSOLE-V2-PRD-023` | state store/audit 不可持久化时普通写 fail closed | partial | not-implemented | not-run | [trust §7.3](./trust-safety-ux.md#73-storeaudit-降级) |
| `CONSOLE-V2-PRD-024` | `OUTCOME_UNKNOWN` 禁止换 key/盲重试；仅按原绑定查询、对账、验证或隔离 | partial | not-implemented | not-run | [trust §8](./trust-safety-ux.md#8-结果未知与幂等) |
| `CONSOLE-V2-PRD-025` | supervision lease 到期后在安全检查点暂停；未收到证据前只显示 pause pending | missing | not-implemented | none | [scope §6](./windows-v1-scope.md#6-监督-lease-与退出) |
| `CONSOLE-V2-PRD-026` | 页面区分 loading/last-good/empty/partial/redacted/offline/denied/submitting/unknown/conflict/privacy-locked/reauth | product-only | not-implemented | none | [state matrix](./journeys-and-screens.md#4-通用页面状态矩阵) |
| `CONSOLE-V2-PRD-027` | 用户任务语言为主，机器 enum/ref/error 在详情保真且中英文一致 | product-only | not-implemented | none | [IA §6](./information-architecture.md#6-用户术语与机器术语) |
| `CONSOLE-V2-PRD-028` | 任意来源只进入获取/检查；运行要求证据完整且 authority 风险不高于 R1 | partial | not-implemented | not-run | [trust §9](./trust-safety-ux.md#9-agent-包与供应链) |
| `CONSOLE-V2-PRD-029` | 关闭窗口进托盘维持监督；“退出并请求暂停”有界等待 accepted，超时不宣称已暂停 | missing | not-implemented | none | [journey 005](./journeys-and-screens.md#console-v2-jrn-005-托盘监督退出和-lease-到期) |
| `CONSOLE-V2-PRD-030` | 产品价值以每位活跃操作者成功收敛的受验证任务数衡量，客户端点击不冒充完成 | product-only | not-implemented | none | [brief §7](./product-brief.md#7-成功定义) |
| `CONSOLE-V2-PRD-031` | 不可信内容与密码/bootstrap/R1/系统控件使用独立 WebView/进程或原生安全边界 | missing | not-implemented | none | [trust §4](./trust-safety-ux.md#4-不可信内容与系统控件) |
| `CONSOLE-V2-PRD-032` | PrivilegedManagementSession 只是权限上界，不能替代每个写操作的单次 gate/approval | registered | not-implemented | not-run | [trust §6](./trust-safety-ux.md#6-r0r1-安全交互) |
| `CONSOLE-V2-PRD-033` | 每个 governed write 产生独立、关联稳定 ref 的 authority audit，不复用其他操作记录 | partial | not-implemented | none | [trust §7.3](./trust-safety-ux.md#73-storeaudit-降级) |
| `CONSOLE-V2-PRD-034` | URL/Git/本地路径 acquisition 自身经低权限 broker、proposal/Effect、路径/网络/预算 gate | missing | not-implemented | none | [trust §9.1](./trust-safety-ux.md#91-来源) |
| `CONSOLE-V2-PRD-035` | Owner bootstrap bundle 固定 endpoint key/目标 SID；Service 从 IPC peer token 派生 SID | missing | not-implemented | none | [PAGE-018](./journeys-and-screens.md#console-v2-page-018-首个-owner-领取) |
| `CONSOLE-V2-PRD-036` | Service install/uninstall/stop/restart/update/bootstrap mint/trust reset/recovery 要求 OS 管理员/UAC、签名和 anti-rollback | missing | not-implemented | none | [scope §2](./windows-v1-scope.md#2-部署边界) |
| `CONSOLE-V2-PRD-037` | lease 续租绑定 task/principal/SID/logon-session/channel/client-epoch 与 session/watch/UI freshness | missing | not-implemented | none | [trust §7.1](./trust-safety-ux.md#71-监督-lease) |
| `CONSOLE-V2-PRD-038` | Service 经每用户 broker 投递一次性 opaque notification handle；消费后才解析 item ref | missing | not-implemented | none | [trust §11](./trust-safety-ux.md#11-通知与深链) |
| `CONSOLE-V2-PRD-039` | 锁屏/用户切换 teardown 敏感 renderer、清理 app-managed buffers，解锁后重认证/resnapshot | product-only | not-implemented | none | [scope §7.1](./windows-v1-scope.md#71-断线锁屏和本地残留) |
| `CONSOLE-V2-PRD-040` | 降级遏制仅使用预授权、限时、target/version/fencing-bound capability 和独立应急日志 | missing | not-implemented | none | [trust §7.3](./trust-safety-ux.md#73-storeaudit-降级) |
| `CONSOLE-V2-PRD-041` | C0–C3 兼容等级与 R0–R3 风险等级正交显示，不合成总分 | partial | not-implemented | none | [PAGE-013](./journeys-and-screens.md#console-v2-page-013-兼容性报告) |
| `CONSOLE-V2-PRD-042` | package/sandbox/compatibility 证据只适用于明确 host/channel/adapter 版本，不跨平台外推 | partial | not-implemented | not-run | [trust §9.2](./trust-safety-ux.md#92-证据) |
| `CONSOLE-V2-PRD-043` | 通知按稳定 event/proposal ref 去重；普通进度可聚合但安全事项不被吞掉 | missing | not-implemented | none | [trust §11](./trust-safety-ux.md#11-通知与深链) |
| `CONSOLE-V2-PRD-044` | 普通本地首连 TOFU 固定节点 identity；identity 变化阻断并经受控恢复 | missing | not-implemented | none | [PAGE-002](./journeys-and-screens.md#console-v2-page-002-节点信任)、[PAGE-019](./journeys-and-screens.md#console-v2-page-019-节点身份变化阻断) |
| `CONSOLE-V2-PRD-045` | 本地账号恢复只经 Admin CLI，恢复轮换相关 session/secret 并写 authority audit | missing | not-implemented | none | [scope §3.3](./windows-v1-scope.md#33-后续用户) |
| `CONSOLE-V2-PRD-046` | 每个 Windows SID 映射独立 CognitiveOS 账号；SID 本身不授予 CognitiveOS 权限 | missing | not-implemented | none | [scope §3.3](./windows-v1-scope.md#33-后续用户) |
| `CONSOLE-V2-PRD-047` | 应用不主动持久化敏感 snapshot，并通过临时 profile/storage/dump policy 收窄 OS 残留 | product-only | not-implemented | none | [scope §7.1](./windows-v1-scope.md#71-断线锁屏和本地残留) |
| `CONSOLE-V2-PRD-048` | quiet hours 只抑制通知呈现，不改变底层事项、deadline 或 authority 状态 | product-only | not-implemented | none | [trust §11](./trust-safety-ux.md#11-通知与深链) |
| `CONSOLE-V2-PRD-049` | 安全事项可升级；acknowledged 只改变提醒，不能替代 handled/decided/reconciled | missing | not-implemented | none | [trust §11](./trust-safety-ux.md#11-通知与深链) |

`partial` 表示相关资产只覆盖部分行为（例如幂等冲突已登记，但完整客户端收敛流程仍缺失）；不能简化成“已登记”。

## 3. 上游阻断登记

本节只登记发现，不修改上游资产。

### `CONSOLE-V2-BLK-001` Effect transition 版本与 schema 漂移（已关闭）

- `specs/transitions/effect.transitions.json` 为 `0.2`；
- `specs/schemas/state-transition-table.schema.json` 已按 D-005 接受 `0.1/0.2`；
- Effect table 的该项 schema-version 冲突已关闭。D-001/D-006 的全局 `$id`/版本策略仍待 M1，不能据此声称行为测试已执行。
- 影响：`BLK-001` 不再阻断 Console；相关行为仍由 `BLK-011/012/013` 和 M1 runner 证据约束。

### `CONSOLE-V2-BLK-002` Signed proposal/display contract 不闭合

- `management-action-proposal.schema.json` 有 `proposal_digest`，没有 signer/signature/canonical display fields，且 `additionalProperties:false`。
- 影响：R1 完整 display profile、未来 R2/R3 digest/signature 表述只能是产品依赖。

### `CONSOLE-V2-BLK-003` Error envelope prose/schema 冲突

- Core prose 需要安全原因/correlation 等字段；
- `common-defs.schema.json#/$defs/error` 没有相应字段并禁止附加字段。
- 影响：Console 不能保证所有错误都携带 correlation/safe reason/retry-after。

### `CONSOLE-V2-BLK-004` 当前评审引用已改版

- 当前 `CognitiveOS-Review-Conclusions.md` v2.0 用 F/IMP 条目并替代旧 V 系列基线。
- v2 产品文档不再引用失效的单项 V2/V6/V7/V11/V12；需要当前证据时引用 REQ/F/IMP 或稳定文件锚点。

### `CONSOLE-V2-BLK-005` Supervision lease

- 缺少 lease/heartbeat/grace/safe-checkpoint pause/resume 的机器对象、状态和故障语义。
- 影响：`PRD-025/029` 是 Windows v1 release blocker。

### `CONSOLE-V2-BLK-006` Windows Service 与 IPC

- 缺少 Service install/update/recovery、Host IPC、renderer capability 和 Windows SID binding 合同。
- 影响：`PRD-022` release blocker。

### `CONSOLE-V2-BLK-007` 本地账号与 bootstrap

- 缺少本地账号、password policy、Owner one-time secret、AuthenticationSession、SID mapping、Admin CLI recovery 合同。
- 影响：`PRD-003/021` release blocker。

### `CONSOLE-V2-BLK-008` Agent source/lifecycle

- Agent schemas 提供部分 manifest/install/compatibility 表面；缺少任意 source acquisition、完整安装 transition、installer/sandbox/adapter 后端和 host-specific evidence。
- 影响：`PRD-015/028` release blocker。

### `CONSOLE-V2-BLK-009` Audit-readiness 与应急日志

- 缺少 audit sink readiness、紧急 pause/stop 持久日志和恢复后 reconcile 合同。
- 影响：`PRD-023` release blocker。

### `CONSOLE-V2-BLK-010` Readiness projection

- `MANAGEMENT_READY/USER_READY/OPERATIONAL` 仍是架构 prose，没有登记的 readiness carrier。
- 影响：`PRD-014` 只能按实际 capability/operation 声明保守开放。

### `CONSOLE-V2-BLK-011` Lifecycle carriers

- AgentExecution 有 binding 资产，但完整 AgentExecution、Runtime、AcceptanceDecision、reconciliation/recovery report 等 carrier 仍不完整。
- 影响：`PRD-005..009/024/025` 不能形成完整端到端实现。

### `CONSOLE-V2-BLK-012` Safe retry 语义

- Effect table 支持 unknown→reconciled(still_unknown)，没有明确重新进入 executing 的边。
- 产品不把“same-key same-parameters resume”设计成可执行按钮，直到 normative transition 明确。

### `CONSOLE-V2-BLK-013` Governed object 双轨

- `docs/traceability/findings-ledger.md` 的 F-003 当前为 `partially-closed`：36 份 schema 已迁移到 GovernedObjectHeader/ObjectReference，legacy `$defs` 保留但零引用。
- M1 仍须完成 runner 负例复验、codegen 对齐和 legacy `$defs` 移除决策。
- 影响：仍阻断 M1 出口及 Console 对统一 header 行为的已验证声明；客户端不得把 schema 迁移落地误写为实现或执行证据。

### `CONSOLE-V2-BLK-014` State-store 降级下的 stop/revoke 授权语义

- `conformance/vectors/state-store-degradation.json` 当前无条件期望 deterministic stop/revoke available。
- Windows v1 安全设计要求 store 健康时预铸、限时、target/version/fencing-bound 遏制 capability；无法验证授权/撤销/fencing 时不允许用户发起 stop/revoke。
- 影响：`PRD-023/040` 在 vector、授权合同和应急日志语义一致前保持 blocked；产品不把 allowlist/日志当授权替代。

## 4. Windows v1 release-gate 矩阵

没有已领取的 Console implementation owner；`UNASSIGNED` 本身即阻断 release。每个 PRD 单独给出 oracle。未来测试 ID 只登记计划，当前结果仍为 `none`。

| PRD | Implementation owner | Executable oracle | Current evidence ref | blocked_by |
|---|---|---|---|---|
| `001` | `UNASSIGNED — Console` | Agent/local cache 伪造 commit/完成不能改变系统状态或启用动作 | future `CONSOLE-AUTHORITY-UI-001` (`none`) | `BLK-011/013` |
| `002` | `UNASSIGNED — Console security` | 跨 task/management 的 session/store/cache/proposal/approval/audit 访问全部拒绝 | `conformance/vectors/shell-channel-isolation-003.json` (`not-run`) | `BLK-006/007` |
| `003` | `UNASSIGNED — Identity` | session expiry/revoke/SID/node/channel mismatch 后特权不恢复 | `conformance/vectors/management-session-denials.json` (`not-run`) | `BLK-007` |
| `004` | `UNASSIGNED — Console/Contracts` | target/version/digest 变化使旧 preview/gate 失效 | `conformance/vectors/intent-supersede-002.json` (`not-run`) | `BLK-002/011` |
| `005` | `UNASSIGNED — Console` | Agent 文本/local cache 声称完成时 UI 保持 authority 状态 | future `CONSOLE-AUTHORITY-UI-002` (`none`) | `BLK-011/013` |
| `006` | `UNASSIGNED — Runtime/Console` | 停止 PID 只改变 Runtime 呈现，不改变 AgentExecution/Task | future `CONSOLE-RUNTIME-SEPARATION-001` (`none`) | `BLK-011` |
| `007` | `UNASSIGNED — Runtime/Console` | 五登记状态域独立显示且非法聚合状态被拒绝 | future `CONSOLE-STATE-TRACKS-001` (`none`) | `BLK-011/013` |
| `008` | `UNASSIGNED — Task/Console` | 缺 current Verification 或 AcceptanceDecision 时禁止显示 COMPLETED | `conformance/vectors/intent-acceptance-007.json` (`not-run`) | `BLK-011/013` |
| `009` | `UNASSIGNED — Runtime/Console` | 每个控制动作显示正确 target/不保证事项，互不替代 | `conformance/vectors/shell-cancel-semantics-005.json` (`not-run`) | `BLK-011` |
| `010` | `UNASSIGNED — Watch/Console` | gap/stale/权限变化均先 resnapshot，重复事件去重 | `conformance/vectors/shell-watch-resume-006.json` (`not-run`) | `BLK-010/011` |
| `011` | `UNASSIGNED — Contracts/Console` | ambiguous/stale/not-found 未固定强引用前不能提交 | `conformance/vectors/shell-target-ambiguity-001.json` (`not-run`) | `BLK-013` |
| `012` | `UNASSIGNED — Cache/Console` | revocation/tenant/channel/version 变化原子失效相关 cache/watch/preview | `conformance/vectors/context-revocation-cache-reuse.json` (`not-run`) | `BLK-006/007/013` |
| `013` | `UNASSIGNED — Console` | 已登记错误保留 code/retryable；未知错误不冒充成功/权限不足 | future `CONSOLE-ERROR-UI-001` (`none`) | `BLK-003` |
| `014` | `UNASSIGNED — Readiness/Console` | 未声明 operation 不出现；MANAGEMENT_READY 不扩大能力 | future `CONSOLE-READINESS-001` (`none`) | `BLK-010` |
| `015` | `UNASSIGNED — Agent lifecycle` | 生命周期失败保留旧 installation/rollback point/未决 Effect | `conformance/vectors/agent-installation-verification.json` (`not-run`) | `BLK-008` |
| `016` | `UNASSIGNED — Console accessibility` | Narrator+键盘在 High Contrast、200%/400% zoom、reduced motion 下完成全部关键旅程 | future `CONSOLE-A11Y-001` (`none`) | `BLK-006/007` |
| `017` | `UNASSIGNED — Console` | offline/不可达显示 `as_of`、禁写且不显示 paused/stopped | `conformance/vectors/shell-detach-attach-004.json` (`not-run`) | `BLK-010/011` |
| `018` | `UNASSIGNED — Windows integration` | 锁屏通知/托盘无正文/credential，通知 action 不能执行 R1 | future `CONSOLE-NOTIFY-PRIVACY-001` (`none`) | `BLK-006` |
| `019` | `UNASSIGNED — Protocol/Console` | unknown critical extension/schema/security floor 时能力 fail closed | `conformance/vectors/schema-version-001.json` (`not-run`) | `BLK-013` |
| `020` | `UNASSIGNED — Risk/Console` | 客户端/Agent 降 risk 失败；R2/R3 无执行控件 | future `CONSOLE-RISK-FLOOR-001` (`none`) | `BLK-002` |
| `021` | `UNASSIGNED — Identity` | Console 无 password verifier/hash；全部登录由节点返回 session | future `CONSOLE-LOCAL-AUTH-001` (`none`) | `BLK-007` |
| `022` | `UNASSIGNED — Windows integration` | renderer 无 Service Manager/进程/密钥权限，仅 allowlist IPC 可达 | future `CONSOLE-WIN-SERVICE-001` (`none`) | `BLK-006` |
| `023` | `UNASSIGNED — Recovery` | state/audit 持久化失败时普通 governed write/new Effect 全拒绝 | `conformance/vectors/state-store-degradation.json` (`not-run`) | `BLK-009/014` |
| `024` | `UNASSIGNED — Effect/Console` | unknown 时换 key/盲重试拒绝，原 binding 对账；NOT_EXECUTED 不进入 commit | `conformance/vectors/effect-unknown-outcome.json`, `effect-idempotency-conflict.json` (`not-run`) | `BLK-012` |
| `025` | `UNASSIGNED — Task` | lease 到期仅在安全检查点确认暂停，之前保持 pause pending | future `CONSOLE-LEASE-PAUSE-001` (`none`) | `BLK-005` |
| `026` | `UNASSIGNED — Console design` | 每个通用状态 fixture 呈现正确标题、动作禁用和 live announcement | future `CONSOLE-PAGE-STATES-001` (`none`) | — |
| `027` | `UNASSIGNED — Content design` | zh-CN/en 主标签用户化，机器 enum/ref/error 无信息损失 | future `CONSOLE-I18N-TERMS-001` (`none`) | — |
| `028` | `UNASSIGNED — Agent lifecycle` | 来源未知/证据缺失/R2+ 时可检查但不能运行 | `conformance/vectors/agent-adapter-bypass.json` (`not-run`) | `BLK-008` |
| `029` | `UNASSIGNED — Console` | 退出有界等待 accepted；超时留托盘或明确 lease-expiry，绝不显示已暂停 | future `CONSOLE-EXIT-PAUSE-001` (`none`) | `BLK-005` |
| `030` | `UNASSIGNED — Product analytics` | 只计 authority COMPLETED + current Verification + AcceptanceDecision | future `CONSOLE-METRIC-001` (`none`) | `BLK-011` |
| `031` | `UNASSIGNED — Console security` | compromise data WebView 不能访问密码/bootstrap/R1 surface 或写 IPC | future `CONSOLE-WEBVIEW-ISOLATION-001` (`none`) | `BLK-006` |
| `032` | `UNASSIGNED — Management` | 有 active management session 但无单次 gate 时写操作拒绝 | `conformance/vectors/management-gate-denials.json` (`not-run`) | `BLK-002` |
| `033` | `UNASSIGNED — Audit` | 两个 governed writes 产生两个独立、稳定关联的 authority audit records | future `CONSOLE-WRITE-AUDIT-001` (`none`) | `BLK-009` |
| `034` | `UNASSIGNED — Acquisition` | SSRF/UNC/path/ambient credential/redirect/budget 负例逐项拒绝 | future `CONSOLE-ACQUISITION-001` (`none`) | `BLK-008` |
| `035` | `UNASSIGNED — Identity` | endpoint key/SID/peer token 任一不匹配时 secret 不发送且领取失败 | future `CONSOLE-OWNER-BOOTSTRAP-001` (`none`) | `BLK-006/007` |
| `036` | `UNASSIGNED — Windows integration` | 非管理员不能 install/uninstall/stop/restart/update/mint bundle/trust reset/recover Service；签名/anti-rollback 失败拒绝 | future `CONSOLE-WIN-ADMIN-001` (`none`) | `BLK-006` |
| `037` | `UNASSIGNED — Task/Console` | 多实例/锁屏/用户切换/revoke/watch stale/UI hang 下旧实例不能续租 | future `CONSOLE-LEASE-ELIGIBILITY-001` (`none`) | `BLK-005` |
| `038` | `UNASSIGNED — Windows integration` | 多 SID/session 正确路由；handle 单次消费、重放失败并 resnapshot | future `CONSOLE-NOTIFY-BROKER-001` (`none`) | `BLK-006` |
| `039` | `UNASSIGNED — Console privacy` | 锁屏/用户切换 teardown renderer/清 buffer；解锁必须重认证/resnapshot | future `CONSOLE-PRIVACY-LOCK-001` (`none`) | `BLK-006/007` |
| `040` | `UNASSIGNED — Recovery` | capability 缺失/过期/target/fencing 不匹配或 emergency log 不可写时遏制拒绝 | future `CONSOLE-EMERGENCY-001` (`none`) | `BLK-009/014` |
| `041` | `UNASSIGNED — Agent/Console` | C 与 R 任意返回组合分轴呈现，准入不合并为总分 | future `CONSOLE-COMPAT-RISK-001` (`none`) | `BLK-008` |
| `042` | `UNASSIGNED — Agent/Console` | Windows/host-A 证据不能使其他 host/channel/adapter 显示 verified | `conformance/vectors/agent-adapter-bypass.json` (`not-run`) | `BLK-008` |
| `043` | `UNASSIGNED — Notification` | duplicate 合并；普通进度聚合；安全事项保持独立 | future `CONSOLE-NOTIFY-DEDUPE-001` (`none`) | `BLK-006` |
| `044` | `UNASSIGNED — Identity` | 首连 pin 后 identity 变化阻断登录，受控 reset 后才可重新信任 | future `CONSOLE-TOFU-001` (`none`) | `BLK-006/007` |
| `045` | `UNASSIGNED — Identity/Admin CLI` | recovery 只经 CLI；旧 session/secret 全失效且生成 audit | future `CONSOLE-ACCOUNT-RECOVERY-001` (`none`) | `BLK-007/009` |
| `046` | `UNASSIGNED — Identity` | SID A/B 无交叉账号/session；仅 SID 不授予节点角色 | future `CONSOLE-SID-MAPPING-001` (`none`) | `BLK-007` |
| `047` | `UNASSIGNED — Console privacy` | WebView UDF/DOM storage/cache/dump 配置和退出清理不产生应用主动敏感持久化 | future `CONSOLE-PRIVACY-RESIDUAL-001` (`none`) | `BLK-006` |
| `048` | `UNASSIGNED — Notification` | quiet hours 只抑制呈现，Inbox/deadline/authority 状态不变 | future `CONSOLE-NOTIFY-QUIET-001` (`none`) | `BLK-006` |
| `049` | `UNASSIGNED — Notification` | acknowledge 后安全事项仍待处理，直到 handled/decided/reconciled；升级策略生效 | future `CONSOLE-NOTIFY-HANDLED-001` (`none`) | `BLK-006/009` |

## 5. 旧产品要求映射

旧 ID 不再新增或复用。

| Legacy ID | v2 去向 | 状态/说明 |
|---|---|---|
| `CONSOLE-PRD-001` | `CONSOLE-V2-PRD-001` | current |
| `CONSOLE-PRD-002` | `CONSOLE-V2-PRD-002/012/022/031` | channel credential/Context/cache/proposal/approval/audit/IPC separation preserved |
| `CONSOLE-PRD-003` | `CONSOLE-V2-PRD-003/021/032/035` | session lifecycle and “session is not approval” preserved |
| `CONSOLE-PRD-004` | `CONSOLE-V2-PRD-004/020/033` | fixed write gate/Effect and independent audit split |
| `CONSOLE-PRD-005` | roadmap R2/R3；v1 `CONSOLE-V2-PRD-020` 只负责阻断 | future/split |
| `CONSOLE-PRD-006` | `CONSOLE-V2-PRD-005` | current |
| `CONSOLE-PRD-007` | `CONSOLE-V2-PRD-006` | current |
| `CONSOLE-PRD-008` | `CONSOLE-V2-PRD-007` | current |
| `CONSOLE-PRD-009` | `CONSOLE-V2-PRD-008` | current |
| `CONSOLE-PRD-010` | `CONSOLE-V2-PRD-009` | current |
| `CONSOLE-PRD-011` | roadmap enterprise governance | future |
| `CONSOLE-PRD-012` | roadmap Memory | future |
| `CONSOLE-PRD-013` | roadmap Knowledge | future |
| `CONSOLE-PRD-014` | roadmap Multi-Agent | future |
| `CONSOLE-PRD-015` | `CONSOLE-V2-PRD-041` | C0–C3 / R0–R3 orthogonal display preserved |
| `CONSOLE-PRD-016` | `CONSOLE-V2-PRD-042` | platform/host evidence non-transfer preserved |
| `CONSOLE-PRD-017` | roadmap Mobile Companion | future |
| `CONSOLE-PRD-018` | `CONSOLE-V2-PRD-010` | current |
| `CONSOLE-PRD-019` | `CONSOLE-V2-PRD-011` | current |
| `CONSOLE-PRD-020` | `CONSOLE-V2-PRD-002/012` | split |
| `CONSOLE-PRD-021` | `CONSOLE-V2-PRD-013` | current |
| `CONSOLE-PRD-022` | `CONSOLE-V2-PRD-014` | narrowed |
| `CONSOLE-PRD-023` | `CONSOLE-V2-PRD-015/028` | split |
| `CONSOLE-PRD-024` | `CONSOLE-V2-PRD-033` + `PAGE-017` + roadmap Audit | split/future |
| `CONSOLE-PRD-025` | `CONSOLE-V2-PRD-016` | current |
| `CONSOLE-PRD-026` | `CONSOLE-V2-PRD-017/039/047` | split into offline inference, lock teardown, no intentional persistence |
| `CONSOLE-PRD-027` | `CONSOLE-V2-PRD-018/038/043/048/049` | privacy, broker/handle, dedupe, quiet hours, escalation/handled split |
| `CONSOLE-PRD-028` | `CONSOLE-V2-PRD-019` | current |
| `CONSOLE-PRD-029` | roadmap concept separation；v1 `PRD-027` covers labels | future/split |
| `CONSOLE-PRD-030` | `CONSOLE-V2-PRD-020` | current |
| `CONSOLE-PRD-031` | `CONSOLE-V2-PRD-003/021/035/044/045/046` | local identity split; remote PKCE future |
| `CONSOLE-PRD-032` | `CONSOLE-V2-PRD-002/019/022/031` | split |
| `CONSOLE-PRD-033` | `CONSOLE-V2-PRD-023/040` | ordinary fail-closed vs authorized emergency path split |
| `CONSOLE-PRD-034` | `CONSOLE-V2-PRD-024` | current |

## 6. 旧页面映射

| Legacy page | v2 去向 | 状态/说明 |
|---|---|---|
| `A-01` 节点与工作区 | `PAGE-001/002/019` | narrowed to one local node |
| `A-02` 首次连接/认证 | `PAGE-002/018/003` | trust/bootstrap/sign-in split |
| `A-03` Shell 三栏 | `PAGE-004` | redesigned conversation-first |
| `A-04` Command Preview | `PAGE-005` | current |
| `A-05` 后台任务抽屉 | `PAGE-006/007` + tray | redesigned |
| `A-06` Agent 目录 | `PAGE-009` | Agent center + sources |
| `A-07` Package 详情 | `PAGE-010` | current |
| `A-08` 安装/升级向导 | `PAGE-011` | expanded lifecycle |
| `A-09` Compatibility | `PAGE-013` | current |
| `A-10` Installed Agent | `PAGE-012` | current |
| `A-11` Executions 列表 | `PAGE-006/007` | merged into task supervision |
| `A-12` Execution 详情 | `PAGE-007` | merged, identities remain separate |
| `A-13` Task 看板 | `PAGE-006` | current |
| `A-14` Task 生命周期 | `PAGE-007` | current |
| `A-15` Effect 对账 | `PAGE-008` | current |
| `A-16` Verification Evidence | `PAGE-007/017` | merged |
| `A-17` Memory Inbox | roadmap Memory | future |
| `A-18` Memory 冲突/删除 | roadmap Memory | future |
| `A-19` Knowledge 搜索 | roadmap Knowledge | future |
| `A-20` Knowledge graph | roadmap Knowledge | future |
| `A-21` Collaboration | roadmap Multi-Agent | future |
| `A-22` Handoff/Conflict | roadmap Multi-Agent | future |
| `A-23` Approval Inbox | `PAGE-014` for R1; full Approval future | split |
| `A-24` Trusted Confirmation | roadmap R2/R3 | future |
| `A-25` Users & Membership | `PAGE-016` current-account subset; enterprise future | split |
| `A-26` Delegation & Capability | roadmap enterprise governance | future |
| `A-27` Operation Catalog | roadmap Operation Catalog | future; not Agent package sources |
| `A-28` Audit Search | `PAGE-017` object subset; full Audit future | split |
| `A-29` Export Request | roadmap Audit | future |
| `A-30` System Overview | `PAGE-015` | current |
| `A-31` ResourceGraph | roadmap distributed/system | future |
| `A-32` Configuration/Update | `PAGE-015` update subset; broader config future | split |
| `A-33` Notification Settings | `PAGE-016` | current |
| `A-34` Device & Sessions | `PAGE-016` local subset; remote devices future | split |

## 7. v2 ID 维护规则

- ID 一经发布不得重用；删除项标 `deprecated` 并保留映射。
- 一个 PRD 只表达一个可独立通过/失败的产品行为。
- Journey/Page/Component 引用 PRD，但不复制机器合同。
- 每个 release-blocking PRD 必须有：
  - contract 状态；
  - implementation owner；
  - executable oracle；
  - evidence ref；
  - blocked_by。
- 仅存在 vector/fixture 不等于 `executed`；仅代码构建通过不等于 Profile 符合。
