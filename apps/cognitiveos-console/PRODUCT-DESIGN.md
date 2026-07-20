# CognitiveOS Console 产品设计 v2

<a id="doc-top"></a>

> 本文件是 CognitiveOS Console v2 的兼容入口。完整产品设计已拆分到 [`docs/`](./docs/)；本文件保留旧路径以及仓库依赖的 §17、§20.3 锚点。
>
> 文档性质：Informative。本文不新增/修改任何 CognitiveOS `REQ-*`、错误码、schema、transition table 或 conformance vector。
>
> Console 状态：`planned`。文档存在不表示实现已提供、测试已执行或 Profile 已符合。

## 漂移登记

| 日期 | 变更/发现 | 受影响章节 | 处置 |
|---|---|---|---|
| 2026-07-20 | F-003 governed-object 单轨迁移已落地，仍待 M1 runner/codegen 复验 | §20.3、产品追踪 BLK-013 | 更正为 `partially-closed`，不宣称行为已验证 |
| 2026-07-20 | D-005 已使 transition table schema 接受 `0.1/0.2` | §20.3、产品追踪 BLK-001 | 移除过时 blocker，保留已关闭历史 |
| 2026-07-20 | 旧 §12.6 PoC gate 指针在 v2 拆分后失效 | §17.4、§20.3、Lane-CON 治理 | 改指 [`docs/platforms/`](../../docs/platforms/README.md#console-实现-gate) 的可定位平台 gate |

## 0. v2 文档地图

| 文档 | 负责内容 |
|---|---|
| [产品简报](./docs/product-brief.md) | 产品问题、首要 persona/JTBD、价值、边界、成功指标 |
| [Windows v1 范围](./docs/windows-v1-scope.md) | 发布切片、Service/账号/TOFU、R0/R1、能力与 release gates |
| [信息架构](./docs/information-architecture.md) | 角色/readiness 落点、任务导航、Shell、术语和页面状态 |
| [旅程与页面](./docs/journeys-and-screens.md) | 核心旅程、Windows v1 页面清单、状态矩阵和未来验收 |
| [Design System](./docs/design-system.md) | 品牌方向、布局、组件、动效、无障碍和国际化 |
| [可信与安全体验](./docs/trust-safety-ux.md) | authority、身份、R0/R1、监督 lease、包来源、离线与错误 |
| [产品要求与追踪](./docs/requirements-traceability.md) | `CONSOLE-V2-*`、旧 ID 映射、三维状态与上游阻断 |
| [路线图](./docs/roadmap.md) | 非 Windows v1 feature briefs |
| [决策记录](./docs/decision-log.md) | 已确认和被替代的产品决策 |
| [桌面平台产品设计](../../docs/platforms/README.md) | macOS/Linux 独立范围、决策、要求、parity matrix 与真实 PoC gate |
| [Agent Hub / 直连接管](./docs/agent-hub/README.md) | Direct Takeover 与 Governed 两部署模式、第三方 Agent 接管层级/Adapter、威胁模型与受 gate 阻断的开发计划（informative） |

## 1. 当前产品基线

- 工作名：CognitiveOS Console；
- 首要用户：Agent 操作者；
- 首发平台：Windows 桌面；
- 节点：一个本机共享 CognitiveOS Windows Service；
- 核心任务：对话/创建任务、监督/纠偏/暂停、Agent 安装/升级/回滚/卸载；
- 风险范围：R0/R1；R2/R3 只识别并阻断；
- Shell：对话主画布 + 可折叠任务/上下文侧栏；
- 导航：工作、任务、Agent、收件箱、记录、系统；
- 当前技术：Tauri 2 + React/TypeScript 为候选，尚未形成批准 ADR。

Windows v1 的详细范围以 [windows-v1-scope.md](./docs/windows-v1-scope.md) 为准。macOS 与受限 Linux 已有独立的 `planned/blocked` 产品切片，但不属于 Windows v1，也不表示实现车道已激活；移动、远程/多节点、完整治理、Memory、Knowledge、Multi-Agent 和 R2/R3 仍见 [roadmap.md](./docs/roadmap.md)。

## 2. 状态与事实来源

所有 Console 文档严格区分：

1. **规范已登记（specified）**：REQ/schema/vector/transition 已在机器资产中存在；
2. **实现已提供（implementation available）**：适用代码存在并可构建；
3. **测试已执行（test executed）**：runner 实际执行并有证据；
4. **Profile 已符合（implemented）**：全部适用 MUST 有通过或有据不适用证据。

产品要求还使用三个正交维度：

- Contract：registered / partial / prose-only / missing / product-only；
- Implementation：not-implemented / partial / available；
- Evidence：none / not-run / pass / fail / not-applicable / documented-degradation。

Profile 声明单独使用 `planned/experimental/implemented/unsupported`，不混入 Evidence。

完整状态见 [requirements-traceability.md](./docs/requirements-traceability.md)。

## 3. 不可降级的边界

- Console 不是节点、authority、IdP、Runtime 或最终安全仲裁器；
- Agent/网页/文件/日志/package source 始终是不可信输入；
- 系统状态只来自 authority projection；
- Task/Loop/AgentExecution/Runtime/Effect/Verification 不合并成一个“运行状态”；
- `CANDIDATE_COMPLETE` 不等于 `COMPLETED`；
- `OUTCOME_UNKNOWN` 禁止盲重试和换 key；
- 风险下界由 authority 决定，客户端/Agent 不能降级；
- R2/R3 不在普通聊天、密码或 Windows 通知中批准；
- 敏感 snapshot 在 Windows v1 不离线落盘；
- store/audit 不可持久化时普通写 fail closed；
- specified、implementation、test、conformance 不互相替代。

## 4. v2 ID 与旧文档

- v2 使用 `CONSOLE-V2-PRD/JRN/PAGE/CMP/DEC-*`。
- 旧 `CONSOLE-PRD-001..034` 和 `A-01..34` 已停止新增和复用。
- 旧 ID 到 v2/current/future/deprecated 的完整映射见 [产品要求与追踪](./docs/requirements-traceability.md#5-旧产品要求映射)。
- 旧单体文档中的失效 V 系列评审引用、未定义“边界 #20”、过时 Memory 缺口、`MANAGEMENT_READY` 全管理可用等表述不再作为 v2 事实。

---

<a id="sec-17"></a>
## 17. MVP 与路线图（兼容摘要）

### 17.1 Windows v1

Windows v1 是当前唯一冻结的 Console 产品切片：

1. 安装/发现独立签名的本机 Windows Service；
2. 本机 TOFU、一次性 Owner bootstrap、本地节点账号和 AuthenticationSession；
3. 对话优先 Shell；
4. Task/Loop/AgentExecution/Runtime/Effect/Verification 监督；
5. supervision lease 到期后安全检查点暂停；
6. Agent 任意来源获取/检查和完整生命周期；
7. 只执行 authority 判为 R0/R1 的写；
8. 托盘监督、脱敏 Windows 通知和嵌入式最小治理；
9. 应用不主动持久化敏感 snapshot；锁屏 teardown 后需重认证/resnapshot；
10. store/audit/watch 降级时 fail closed 或进入有证据的应急遏制。

### 17.2 Windows v1 明确排除

- 远程节点、多节点、多工作区；
- macOS、Linux、iOS、Android；
- R2/R3 执行；
- 完整 Approval/Audit/Users & Access；
- Memory、Knowledge、Multi-Agent；
- 持久敏感离线缓存；
- renderer 直接管理 Windows Service/进程/密钥；
- 仅凭警告运行 authority 判为 R2/R3 的任意来源包。

### 17.3 后续路线

后续按以下顺序分别进入 feature discovery，而非一次性“五端 MVP”：

1. Windows 远程/多工作区；
2. macOS 与受限 Linux；
3. Mobile Remote Companion；
4. 完整企业治理；
5. R2/R3 可信确认；
6. Governed Memory / Knowledge；
7. Multi-Agent / Distributed。

每个 phase 的不可变边界和进入门禁见 [roadmap.md](./docs/roadmap.md)。
macOS/Linux 已确认的平台产品决策、支持边界和 Open PoC/GA gates 见 [桌面平台产品设计](../../docs/platforms/README.md)；这些文档属于激活前 informative 例外，不改变实现 gate。

### 17.4 Release gate

Windows v1 不能仅凭 UI prototype 发布。至少需要：

- 适用机器合同已登记且无 release-blocking schema/transition 冲突；
- Windows Service/IPC/OS 管理员边界、账号/bootstrap/session、supervision lease、Agent lifecycle、notification broker 和 audit emergency 路径实现已提供；
- authority state、risk floor、R1 preview、watch/reconcile 已闭合；
- 安全、故障、Narrator/键盘/High Contrast/reduced-motion 测试已执行；
- 真实证据证明无错误完成声明、无跨用户/channel 泄露、无重复 Effect；
- Tauri 2 + React/TypeScript（或替代方案）ADR 已批准。

跨平台 Console 实现还必须满足 [平台实现 gate](../../docs/platforms/README.md#console-实现-gate)，且目标平台 PoC 使用真实 API/真实 OS 行为留证。

---

<a id="sec-20"></a>
## 20. 风险、待决策与依赖（兼容摘要）

### 20.1 主要产品风险

- 共享 Service 的 Owner 抢占、SID/账号混淆和恢复旁路；
- 本地 TOFU 身份变化被用户忽略；
- renderer compromise 取得 Service/management IPC；
- supervision lease 失效却继续显示“受监督”；
- arbitrary package source 被误写为 trusted；
- R2/R3 被降级为 R1；
- pause/stop Runtime 被误解为 Effect 已收敛；
- state store/audit 故障时本地缓冲冒充 authority commit；
- Windows 通知/托盘泄露正文或被当授权入口；
- assistant 视觉隐藏状态、freshness 或未知结果。

产品缓解和未来安全验收见 [trust-safety-ux.md](./docs/trust-safety-ux.md)。

### 20.2 尚未冻结的产品选择

- Public Beta 正式名称与品牌资产；
- Tauri 2 + React/TypeScript 最终 ADR；
- Windows 安装包、企业分发和自动更新渠道；
- 遥测/崩溃服务提供方、数据驻留和 retention；
- 远程/企业节点信任与 IdP；
- R2/R3 的 authority-owned browser 与原生可信面取舍；
- 非 GA 桌面、移动与远程平台支持矩阵。

这些选择不能改变 [决策记录](./docs/decision-log.md) 中已确认的 authority、risk、状态和隔离边界。

### 20.3 当前最大产品依赖结论

以下依赖不因本文存在而成为“规范已登记”或“实现已提供”：

- Windows Service 安装、版本、IPC、OS 管理员/UAC、签名/anti-rollback、更新、恢复和 renderer capability 合同；
- 本地节点账号、Windows SID mapping、endpoint-key-bound Owner bootstrap、AuthenticationSession 和 Admin CLI recovery；
- supervision lease/heartbeat/eligibility/client-epoch/grace/safe-checkpoint pause；
- Task/Loop/AgentExecution/Runtime/Effect/Verification/Acceptance 完整 projection；
- arbitrary source acquisition、package verification、installer、sandbox、adapter、compatibility 和 lifecycle transition；
- server-side risk floor、R1 confirmation object 和 canonical display profile；
- readiness/capability projection、watch、每用户 Windows notification broker 与一次性 handle resolution；
- audit-readiness、应急 pause/stop 日志和恢复后 reconcile；
- Console 客户端实现、Playwright/原生端到端测试和执行证据。

已确认的上游一致性阻断包括：

1. Effect transition 为 `0.2`，通用 transition schema 已接受 `0.1/0.2`；D-005 已关闭，统一版本策略仍排 M1；
2. Management proposal 有 digest，但 signed/canonical display 字段不闭合；
3. Core error prose 与通用 error schema 的字段不闭合；
4. readiness、监督 lease、Windows Service 和本地账号合同缺失；
5. Effect `still_unknown` 后的 safe retry/resume 没有明确 transition；
6. findings-ledger F-003 单轨迁移已落地，但仍待 M1 runner 负例、codegen 与 legacy `$defs` 处置复验；
7. state-store 降级 vector 无条件要求 stop/revoke 可用，但预授权/fencing/audit emergency 合同尚未闭合。

详细证据和影响见 [上游阻断登记](./docs/requirements-traceability.md#3-上游阻断登记)。在对应 contract、implementation 和 executed evidence 闭合前，Console 保持 `planned/blocked`，不得包装成已实现的 CognitiveOS 管理能力。

---

## 旧章节锚点兼容

以下 anchor 保留旧链接可达；其内容已迁移到 v2 专题文档：

<a id="sec-1"></a>
- 旧 §1 文档元数据 → 本页 §0/§2
<a id="sec-2"></a>
- 旧 §2 产品定义 → [产品简报](./docs/product-brief.md)
<a id="sec-3"></a>
- 旧 §3 用户与权限 → [产品简报](./docs/product-brief.md#3-用户与-jobs-to-be-done)、[路线图](./docs/roadmap.md)
<a id="sec-4"></a>
- 旧 §4 部署与连接 → [Windows v1 范围](./docs/windows-v1-scope.md)
<a id="sec-5"></a>
- 旧 §5 平台矩阵 → [路线图](./docs/roadmap.md)
<a id="sec-6"></a>
- 旧 §6 信息架构 → [信息架构](./docs/information-architecture.md)
<a id="sec-7"></a>
- 旧 §7 Agent Shell → [信息架构](./docs/information-architecture.md#4-shell-信息结构)
<a id="sec-8"></a>
- 旧 §8 模块设计 → [旅程与页面](./docs/journeys-and-screens.md)
<a id="sec-9"></a>
- 旧 §9 用户旅程 → [旅程与页面](./docs/journeys-and-screens.md#2-核心旅程)
<a id="sec-10"></a>
- 旧 §10 状态语义 → [可信与安全体验](./docs/trust-safety-ux.md)
<a id="sec-11"></a>
- 旧 §11 安全隐私 → [可信与安全体验](./docs/trust-safety-ux.md)
<a id="sec-12"></a>
- 旧 §12 客户端架构 → [Windows v1 范围](./docs/windows-v1-scope.md#2-部署边界)
<a id="sec-13"></a>
- 旧 §13 API 映射 → [产品要求与追踪](./docs/requirements-traceability.md)
<a id="sec-14"></a>
- 旧 §14 视觉交互 → [Design System](./docs/design-system.md)
<a id="sec-15"></a>
- 旧 §15 通知审批 → [可信与安全体验](./docs/trust-safety-ux.md#11-通知与深链)
<a id="sec-16"></a>
- 旧 §16 发布更新 → [Windows v1 范围](./docs/windows-v1-scope.md#10-技术候选与-release-gate)
<a id="sec-18"></a>
- 旧 §18 产品要求 → [产品要求与追踪](./docs/requirements-traceability.md)
<a id="sec-19"></a>
- 旧 §19 指标 → [产品简报](./docs/product-brief.md#7-成功定义)
<a id="appendix-a"></a>
- 旧附录 A 页面清单 → [页面清单](./docs/journeys-and-screens.md#3-windows-v1-页面清单)
<a id="appendix-b"></a>
- 旧附录 B 术语 → [用户术语与机器术语](./docs/information-architecture.md#6-用户术语与机器术语)

## 文档结束说明

v2 设计只收敛 Console 产品层。规范真相仍以适用 registry/schema/transition/vector 与 normative companion 为准；发生冲突时采用不扩大权限、范围、风险、预算或完成声明的解释。
