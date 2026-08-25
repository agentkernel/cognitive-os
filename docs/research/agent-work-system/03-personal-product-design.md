# CognitiveOS Personal 产品设计

Date: 2026-08-25
Status: **candidate / owner-confirmed discovery / non-canonical / no implementation authorization**

本文件把 [owner 决策简报](./02-product-direction-decision-brief.md)转化为 Personal
候选产品设计。它不注册正式任务、PRD、ADR、contract、Gate 或实现授权。仓库事实仍由
`specs/`、规范、Accepted ADR、正式计划、Current snapshot、active lease 与实际代码按
既定优先级决定。

标记：

- **FACT**：由当前仓库或绑定治理来源支持。
- **OWNER DECISION**：2026-08-25 Rounds 1–5 明确选择。
- **INFERENCE / RECOMMENDATION**：可检验的设计推论。
- **OPEN QUESTION**：仍需研究或实现期 disposition。
- capability 必须分别标记 `designed`、`implemented`、`HTTP-accessible`、`tested`、
  `Gate-proven`；未给证据不得升级。

## 1. 产品定义

### 1.1 定位与承诺

**OWNER DECISION**：CognitiveOS 的 umbrella positioning 是 **AI Workforce OS**；
Personal 内部只使用 Agent、Provider、Task、Binding、Evidence 等字面 domain term，
不使用 employee/company/CEO/hiring 隐喻。

> Personal Desktop 1.0 candidate 是 local-first continuity workspace：让 office worker、
> programmer 与 researcher 在一个安静、可恢复的桌面空间里，把 Provider、Agent、
> Conversation、Knowledge、Memory、Skill、Tool 与 governed Work 组合成连续、可核验的工作。

核心产品循环：

```text
Ready → Continue → Review → Work → Verify → Retain
```

首个承诺不是“替用户完成所有工作”，而是：

1. 让用户知道有哪些 Agent、实例和 Provider access；
2. 让一个 Agent/Profile/Instance 与一个 account/model 显式绑定；
3. 让 readiness、entitlement、usage、cost 的来源与 freshness 可见；
4. 在一个 Conversation 中使用获准输入并检查 Context；
5. 明确决定什么进入 Memory，什么只留在 Conversation；
6. Work/Task 只在 daemon authority、hard budget、fencing、Intent/Effect 与独立验证下执行。

### 1.2 目标用户与 JTBD

**OWNER DECISION / scope expansion**：一个产品服务三类个人用户，不拆三个 edition。

| Persona | 高频工作 | 默认 saved view/preset | 共同对象 |
|---|---|---|---|
| Office worker | 文档、会议、调研、跟进与重复流程 | Today、Recent Conversations、Office sources | Conversation、Work、Provider、Library |
| Programmer | repository work、coding Agent、checks、context handoff | Active repos、Coding Agents、Token usage | Agent、Binding、Tool、Skill、Work |
| Researcher | source collection、comparison、citation、long-running synthesis | Research sources、Open threads、Context freshness | Knowledge、Context、Memory、Conversation |

| 情境 | JTBD | 成功信号 |
|---|---|---|
| 多工具散落 | 当我切换多个 Agent 工具时，我要在一个地方确认身份、能力、实例健康和绑定，避免选错执行者 | 60 秒内定位可用 Agent 与不可用原因 |
| 多种商业访问 | 当我同时持有 API account 与 consumer plan 时，我要分清 account/auth/entitlement/usage/cost，避免错误归因 | 不把 estimate、allowance 或 plan 当 invoice |
| 首次配置 | 当我首次使用时，我要按最少步骤达到一个真实可用 binding，而不是浏览空 dashboard | 完成一个 verified binding |
| 状态变化 | 当 auth 过期、readiness 丢失或 cost 未知时，我要看到原因、freshness 和 next action | 失败不清空已确认输入 |
| 后续受治理工作 | 当我把工作交给 Agent 时，我要在执行前看到 scope、Provider/model、resources、budget、acceptance | admission 后只在 exception 打断 |
| 跨会话继续 | 当我返回昨天的对话或任务时，我要看到 changed Context、未解决决定和安全 next action | 不重新解释、不重复 Effect、不隐式复活删除内容 |

现有 evidence 是 repository capability 与公开弱信号，不是需求频率、付费意愿或
product-market fit 证明。

### 1.3 非目标

- chat-first client、人格画廊或一人公司模拟；
- consumer credential brokerage、cookie scraping、password/token 复制；
- 未支持的 consumer-plan purchase、mutation、quota 或 subscription lifecycle；
- 用本地 estimate 声称 invoice/remaining allowance；
- P0 内完整 Goal/Project lifecycle、Workstream、multi-Agent planning；
- 用 Agent 自报、process exit 或 Provider response 判定 Task complete；
- shell/WebView 直接写 authority state；
- 新 repository、monorepo、desktop framework 选型或 implementation commitment。

## 2. Scope 与优先级

### 2.1 P0：激活 + 最小 continuity journey

P0 必须形成以下终态：

```text
Discover
→ Review
→ Register
→ Link approved Provider access
→ Verify readiness
→ Bind Agent/Profile/Instance to account/model
→ Start one Conversation with one authorized input/source
→ Inspect Context, available Skills and Tools
→ Retain or reject one Memory candidate explicitly
→ Show source-typed token/usage/cost facts
→ Ready
```

P0 requirements：

1. user-triggered discovery；每个 proposed fact 有 provenance，import 前 review；
2. manual registration fallback；
3. 一个 qualified Provider API path；
4. supported read-only 或 user-declared consumer-plan facts；
5. Profile、Instance、Installation 不折叠；
6. 明确 account/model binding；
7. 一个 local-canonical Conversation 可 restart/resume；
8. 一个 authorized source/input 进入 Context 前展示 scope、source、loss、freshness；
9. Memory retention 是显式 admission，支持“不保留”；
10. Skill/Tool 只显示真实 installed/enabled/authorized/available facts；
11. usage/cost/entitlement 标注 source、freshness、coverage，不使用 generic confidence；
12. approved auth/SecretStore path；renderer/API 不回显 secret。

### 2.2 P1 / P2 / Deferred

| 层级 | 候选内容 |
|---|---|
| P1 | governed Work/Task；owner Assignment；preview→admit→execute-on-exception→independent acceptance |
| P1 | Knowledge source lifecycle、Context diff、Conversation import/export/delete、Continuation Checkpoint |
| P1 | expired auth、lost readiness、usage alert、binding repair、structured Activity、Library lifecycle |
| P2 | Goal-lite/external work link、更多 independently qualified Agent/Provider/source adapters |
| P2 | persona saved views、bulk-safe refresh、cost reconciliation、official delegated Provider controls |
| Deferred | full Workstream/OKR、generic persisted Run、routine marketplace、multi-Agent orchestration、unsupported consumer-plan mutation |

## 3. Information Architecture

### 3.1 Persistent sidebar

| Area | 核心问题 | P0/P1 |
|---|---|---|
| Home | 我现在应该继续什么、什么需要注意？ | P0 card-led |
| Work | 哪些 governed Tasks 存在，truth/evidence 是什么？ | P1；关联 Conversations |
| Conversations | 哪些对话可继续、其 Context/retention 状态是什么？ | P0 minimal / P1 lifecycle |
| Agents | 哪些 Profile/Instance/Installation 可用？ | P0 |
| Library | Knowledge、Memory、Skills、Tools 各自是否可用？ | P0 visibility / P1 lifecycle |
| Providers | 哪些 plan/account/model/binding/usage facts 可用？ | P0 |
| Activity | 哪些 authority/observation/cost facts改变？ | P1 |
| System | daemon、storage、version、privacy、recovery 是否健康？ | footer utility |

`Context` 不是 top-level destination：它属于当前 Conversation 或 Work 的 inspector/view。
`Usage` 位于 Providers，并提供 global deep link。`Binding` 是 Agent↔Provider/model relation，
不是 sidebar module。首次启动进入 resumable activation；完成后默认进入 card-led Home。

### 3.2 Candidate routes/screens

```text
/home
/activate/{discover,review,register,provider,verify,bind,ready}
/work
/work/:taskRef
/work/:taskRef/{overview,context,effects,evidence,activity}
/conversations
/conversations/:conversationId
/conversations/:conversationId/{context,retention,activity}
/agents
/agents/:profileId
/agents/:profileId/instances/:instanceId
/library/{knowledge,memory,skills,tools}
/library/knowledge/:sourceId
/library/memory/:memoryId
/library/skills/:skillRef
/library/tools/:toolRef
/providers
/providers/:accountId
/providers/:accountId/{access,entitlements,models,bindings,usage-cost}
/providers/usage
/activity
/system/{readiness,storage,privacy,diagnostics,stewardship,about}
/settings/{appearance,notifications,privacy,advanced}
```

这些是 candidate route，不是现有 HTTP route 或 contract 声明。desktop deep link 应恢复
route、selected object、filter/sort 与 wizard checkpoint。

### 3.3 Object hierarchy

```text
Personal Workspace
├─ Conversation
│  ├─ Turn/Message archive
│  ├─ ContextView + ContextDiff
│  ├─ authorized source/input refs
│  └─ Continuation Checkpoint
├─ Agent Profile
│  └─ Agent Instance
│     └─ Installation / Runtime observation
├─ Provider
│  └─ AccessAccount
│     ├─ AuthenticationMethod → SecretRef
│     ├─ Entitlement → Allowance
│     ├─ Model catalog snapshot
│     ├─ Binding ← Agent Profile/Instance
│     ├─ UsageObservation
│     ├─ Budget
│     └─ CostObservation
├─ Library (navigation grouping only)
│  ├─ KnowledgeSource → derived index/reference
│  ├─ Memory → version/tombstone
│  ├─ Skill package/revision/binding
│  └─ Tool descriptor/permission/Effect path
├─ Goal-lite (P2)
└─ Task → Assignment → Attempt projection → Effects/Evidence (P1)
```

## 4. First-run resumable activation wizard

| Step | Primary action | 必须显示 | Recovery |
|---|---|---|---|
| Discover | 扫描 supported local sources | 扫描范围、permission、不会读取的内容 | cancel；manual entry |
| Review | 逐项确认 proposed facts | provenance、freshness、差异、unchecked facts | reject/edit/rescan |
| Register | 建立 Profile/Instance records | identity、purpose、capability source | 保留 review state |
| Provider | Link approved access | account type、auth path、secret handling boundary | denied/expired/unsupported |
| Verify | readiness check | model access、entitlement、health、unknowns | retry/change path |
| Bind | choose account/model | exact Profile/Instance/account/model/budget | conflict/stale CAS |
| Ready | inspect receipt | binding、sources、freshness、limitations | repair/deep link |

Wizard 只对风险点增加 friction；退出后保留非 secret progress。permission 在需要时请求，
不是开屏索取。success 页面必须落到真实 binding receipt，不是庆祝空页。

## 5. Object UX

### 5.1 Agent objects

| Object | 用户含义 | 必须字段 | 不得混淆 |
|---|---|---|---|
| Agent Package | 可安装 code/manifest/version/provenance | publisher、version、digest、source | Profile |
| Installation | 某环境中的安装事实 | path/ref、version、install source、status | runnable Instance |
| Agent Profile | purpose/capability/task fit 的逻辑视图 | name、purpose、capability source、compatibility | personality/employee |
| Agent Instance | 实际可 schedule/runtime target | instance identity、health、freshness、bindings | process observation |
| Runtime Process | OS/workload observation | pid/workload ref、observed_at、source | durable identity |

Agent inventory 使用 list/table + master/detail；Home 可使用 summary card。capability 是
claim；eligibility 还需要 registration、health、policy、Provider/model compatibility。

### 5.2 Provider and economic objects

| Object | UX rule |
|---|---|
| Provider | vendor/service identity；不等于 plan |
| AccessAccount | API/tenant/account context；显示 billed identity only if source proves |
| Auth | approved browser/device/native/API path；不展示 credential |
| SecretRef | 只显示 redacted label、store、last-verified；永不支持 reveal/copy |
| Entitlement | source-typed rights；`unknown` 不等于 denied |
| Allowance | window/unit/source/freshness；不从 local usage 推算 guaranteed remaining |
| Model | provider/model ref、catalog revision、access/readiness |
| Binding | Profile/Instance + AccessAccount + auth ref + model + revision |
| Usage | provider-reported/local/session estimate；显示 period 与 source |
| Budget | Task hard budget 或 Provider advisory budget，必须标 enforcement class |
| Cost | `provider_reported_accrual` / `local_estimate` / `invoice_ref` / `unavailable` |

consumer plan 可 user-declared 或由 supported read-only source 提供。unsupported mutation
显示“由 Provider 管理”并 deep-link first-party surface，不放置假 disabled lifecycle。

## 6. Screen specifications

### 6.1 Home

内容顺序：

1. readiness/activation summary；
2. attention cards：auth expired、readiness lost、stale source、unknown cost；
3. Agent/Provider/binding summary；
4. latest source/freshness receipts；
5. P1 后的 current Work/evidence。

每张 card 只有一个 primary next action，并 deep-link 到 durable state。

### 6.2 Agents

- columns：Profile、Instance count、health、capability source、binding、freshness；
- filters：ready/not-ready/unbound/source/version；
- sort：attention、name、freshness、last used；
- search：name、purpose、package、instance ref；
- detail：identity→instances→capabilities→bindings→provenance→limitations；
- actions：discover、register manual、review update、verify、bind；lifecycle action 仅在真实
  daemon route 存在时显示。

### 6.3 Providers

- accounts master：status cause、auth health、model readiness、entitlement、usage/cost source；
- detail tabs：Overview / Access / Entitlements / Models / Bindings / Usage & Cost；
- create/link：account metadata→approved auth→verify→review；
- binding edit 使用 revision/CAS；stale revision 保留输入并要求 refresh；
- delete 先显示 blocking bindings；不支持的 Provider mutation 不渲染 active action。

### 6.4 Work / Activity / System

- Work P1：Task-first；Goal-lite 可选 reference；Workstream 不存在；
- Work detail：authority timeline 与 observation lane 分离；candidate/process/Effect/verifier/
  acceptance 不合并；
- Activity：coverage banner；source/event kind filters；authority、observation、owner decision、
  cost observation 分型；
- System：daemon readiness、session expiry、diagnostics、backup/restore、version；不把 UI
  session 当 authority session。

### 6.5 Settings

- Appearance：light/dark/system、density 仅作用 operational lists；
- Notifications：tray opt-in；activation success/failure、readiness loss/expiry、P1 Task
  decision/failure；
- Privacy：telemetry opt-in、data export、local cache summary；
- Advanced：diagnostic export（redacted）、refresh policy；不包含 secret reveal。

## 7. State and recovery contract

| State | 必须回答 |
|---|---|
| Empty | 数据如何出现？primary CTA 是 Discover/Register/Link 哪一个？ |
| Loading | 哪些旧事实保持可见？操作是否可取消？ |
| Partial | 哪些 source 缺失，哪些功能仍可用？ |
| Stale | observed_at、TTL、refresh action、是否影响 eligibility？ |
| Permission | 需要什么 scope、为什么、deny 后仍可做什么？ |
| Error | 哪一步失败、输入保留了什么、single next action？ |
| Success | 哪个 durable object 改变、receipt/deep link 在哪里？ |
| Recovery | retry 是否安全、是否需 reauth/rebind、owner 是谁？ |

copy 使用事实语言：`Ready within <scope>`、`Cost unavailable`、`Provider-reported`、
`User-declared`。禁止 generic “Connected/Healthy/Done” 无 scope。

## 8. Interaction, accessibility, keyboard

- sidebar、toolbar、list、detail、dialog、wizard 有可见 focus 与 logical order；
- `Ctrl/Cmd+K` 打开 command/search；`Ctrl/Cmd+L` 聚焦 object search（candidate）；
- `Alt+Left` 恢复 list filter/selection；`Esc` 关闭非破坏 inspector/dialog；
- table 支持 header announcement、sort state、row selected state；
- badges 不能只靠 color；freshness 必须有 text；
- touch target candidate ≥44 px；compact row 仍保留 keyboard hit area；
- 中文/英文长度扩展按 1.5× 检验，不截断 identity、reason、next action。

## 9. Telemetry minimization and metrics

默认不采集 prompt body、transcript、secret、source body、artifact content。candidate telemetry
只含 locally aggregated funnel、state kind、duration bucket、error reason code；跨设备上传需
opt-in、redaction 与 retention。

| Success metric（pilot candidate） | Counter-metric |
|---|---|
| ≥80% 新用户完成一个 verified binding | median setup ≤10 min；secret exposure = 0 |
| ≥90% 用户可在 60 秒指出 readiness blocker | false-ready = 0 |
| ≥95% usage/cost displays 有 source+freshness | estimate shown as invoice = 0 |
| ≥80% expired-auth scenarios 保留 progress 并恢复 | duplicate account/binding mutation = 0 |
| keyboard-only activation completion ≥90% | inaccessible blocking control = 0 |

阈值均是 **RECOMMENDATION**，未测试、未 observation、未 Gate-proven。

## 10. Product acceptance and non-claims

P0 candidate acceptance：

1. real supported Agent discovery/manual registration；
2. review-before-import 与 per-fact provenance；
3. approved Provider access，不泄露 secret；
4. readiness failure honest；
5. explicit revisioned binding；
6. source/freshness-complete entitlement/usage/cost receipt；
7. restart 后可恢复 wizard/binding state；
8. unsupported capability 不通过 200 stub 或 UI 文案伪装。

Explicit non-claims：

- no product implementation or HTTP route is created by this document；
- no shell framework selected；
- no unsupported consumer-plan mutation、fabricated remaining allowance 或 fabricated invoice truth；
- no governed Task P0、market validation、release/Profile/Gate claim；
- no weakening of A1–A8、Intent/Effect、fencing、budget、SecretStore、
  independent verification or unknown-worktree protection。

## 11. Baseline delta and dependencies

相对 `docs/design/01–41` dated baseline，本文件：

- 保留其 shell/state/honesty、Providers、Home、Work detail 与 evidence layering；
- 把 P0 从 task-first 改为 inventory/readiness activation；
- 把产品形态改为 native desktop shell around existing Web client；
- 确认 card-led Home + operational list/master-detail hybrid；
- 把 Task execution 明确放到 next vertical slice；
- 不覆盖 baseline，也不把其 implementation waves 视为当前授权。

相关文档：[视觉交互](./04-personal-interaction-and-visual-spec.md) ·
[架构](./05-personal-architecture.md) ·
[共享边界](./09-shared-domain-and-contract-boundaries.md) ·
[验证与 readiness](./10-validation-and-delivery-readiness.md)。

## 12. Plain-language object model

| Term | 用户语言 | Authority truth |
|---|---|---|
| Conversation | 一段可搜索、可继续、可导出/删除的交互历史 | history container；不等于 Task/Run/Evidence |
| Work | 有目标、边界、状态、预算和验收的受治理工作 | Task/TaskContract + daemon transitions |
| Knowledge | 可被授权检索的来源或材料 | source/version/scope/provenance；index 可重建 |
| Memory | 用户允许跨会话保留的 durable fact/preference/decision | explicit admission、version、tombstone |
| Context | 当前 Conversation/Work 实际获得的有界输入视图 | versioned ContextView + loss/freshness |
| Skill | 可安装、固定 revision、可绑定的做事方法包 | package/revision/lifecycle，非 capability alias |
| Tool | 可调用的受治理能力 | descriptor + permission + Intent/Effect path |
| Binding | Agent 使用哪个 account/auth/model/budget 的明确关系 | daemon-authorized, revisioned relation |

用户不需要理解内部表名，但在 inspection/receipt 中必须能看到 source、scope、revision、
freshness、limitations 和 authority owner。

## 13. Provider/subscription truth and six management modes

### 13.1 Truth taxonomy

```text
Provider
→ Plan / externally declared commercial fact
→ AccessAccount
→ AuthenticationMethod / SecretRef / NativeSessionRef
→ Entitlement / Allowance
→ ModelCatalogSnapshot
→ AgentBinding
→ UsageObservation
→ BudgetPolicy
→ CostObservation / InvoiceRef / Unavailable
```

Plan、account、auth、entitlement、usage、budget 和 invoice 不可合并成 `Subscription` row。

### 13.2 Management modes

| Mode | UI action | Example | Required copy |
|---|---|---|---|
| Managed here | daemon mutation | create local binding, set advisory budget | receipt + revision |
| Link / reauthenticate | approved first-party auth handoff | OAuth/device/native session | provider/auth scope |
| Observe read-only | supported source query/import | provider usage endpoint, ccusage import | source + period + freshness |
| Open Provider | first-party deep link | purchase/cancel/upgrade/billing | “Managed by Provider” |
| Record manually | user-declared fact | plan name or allowance note | “User-declared; not verified” |
| Unavailable | no supported capability | remaining allowance unknown | reason + no fake action |

只有 official supported mutations 才能使用 “Manage here”。Unsupported purchase、cancel、upgrade、
quota reset、remaining allowance 和 invoice 不渲染 enabled control。

### 13.3 Token and cost facts

Token usage row 必须显示：input/output/cache/total（source 有则显示）、model、period、source、
observed_at、coverage、parser/price revision。Cost row 必须显示
`provider_reported` / `local_estimate` / `invoice_ref` / `unavailable`。不显示单一 “confidence
score”；直接展示来源与缺口。

## 14. Detailed screens and repeated-use ergonomics

### 14.1 Home

Order：

1. **Continue**：最近 Conversation/Work 的 next action 与 Context change；
2. **Needs attention**：auth、binding、stale source、blocked Work、storage/privacy；
3. **Ready**：可用 Agent/Binding/Library coverage；
4. **Recent evidence**：accepted Work、Memory retention、import/export/purge receipts；
5. **Usage link**：本期 source-typed token/cost summary，进入 Providers / Usage。

Home 不展示 total Agent/Task/Token 的 vanity KPI，也不作为八个模块的 card wall。

### 14.2 Conversations

- master fields：title、last activity、Agent/Binding、linked Work、Context freshness、retention；
- filters：active/archived/imported、Agent、Provider、Work、date、privacy；
- saved views：Today、Open threads、Imported、Needs Context review；
- detail：transcript、Context bar、Agent/model、token facts、linked Work、Memory candidates；
- actions：continue、new branch/copy、link Work、inspect Context、checkpoint、export、archive、
  delete；
- resume 前展示 Binding/Context/source drift；
- imported history 默认 inert，不自动重放 tool call、Memory 或 Task transition。

### 14.3 Library

- **Knowledge**：sources、scope、version、index status、authorization、last sync、purge state；
- **Memory**：content summary、source Conversation/Work、scope、version、last used、tombstone；
- **Skills**：package、publisher/source、revision、enabled、bindings、limitations；
- **Tools**：descriptor、source、permission、availability、Effect class、verification path。

Library 可以统一搜索与 saved view，但每个 family 有独立 lifecycle。禁止 generic “Create
Resource”。

### 14.4 Agents and Bindings

Agent detail anatomy：

1. Profile identity/purpose；
2. Instance/Installation readiness；
3. capability claims + source；
4. active/default Bindings；
5. allowed Library resources；
6. linked Conversations/Work；
7. activity/limitations。

Binding editor preview exact Agent/Profile/Instance、account/auth/model、budget、resource scope、
revision 与 affected Conversations/Work。Conflict 保留 input；删除先解除依赖或提供 blocked
receipt。

### 14.5 Work

Work list 提供 Tasks 与 linked Conversations；Conversation 不能改变 authority status。Work
detail 保留 overview、Context、Effects、Evidence、Activity。Create preview 显示 objective、
acceptance、Agent/Binding、Knowledge/Memory/Skill/Tool、budget、deadline 和 confirmation tier。

### 14.6 Activity and System

Activity 分 authority、observation、economic、owner decision、import/export/purge；coverage 不足
显示 banner。System 为 footer utility，包含 daemon、storage、privacy、index rebuild、backup/
restore、desktop update、diagnostics、about。

## 15. Actions, forms, bulk operations and undo

- Primary action per surface ≤1；secondary actions stay toolbar/menu。
- Create/link/bind/retain/import/export/delete 使用 review→commit→receipt。
- Consequential form shows affected objects、source、scope、revision、reversibility。
- Bulk defaults to refresh、tag/filter、export；mutation requires selection summary and per-item result。
- Reversible local changes provide time-bounded Undo only when daemon can guarantee inverse semantics。
- External mutation or irreversible purge never uses cosmetic Undo；show recovery/compensating action。
- Partial bulk receipt lists succeeded/failed/not-run；retry only failed idempotent items。
- Drafts persist non-secret fields；secret/auth material is one-way/transient。

## 16. Complete state contract

Every screen/component specifies：

| State | Required behavior |
|---|---|
| Empty | explain value, source of data, one concrete CTA |
| Loading | preserve known facts/layout; cancel long operation |
| Partial | name missing sources and remaining usable capabilities |
| Stale | show timestamp, expiry effect and refresh/review |
| Offline | local-canonical read/search continue; external refresh/mutation disabled honestly |
| Conflict | keep input, show expected/current revision and review diff |
| Permission | show scope/purpose/deny consequence and manual alternative |
| Unsupported | hide fake control; show management mode and first-party path |
| Error | local cause, preserved facts, Effect certainty, retry safety, next action |
| Success | identify durable object/revision and receipt; no generic celebration |
| Purged/deleted | tombstone/receipt, retention exceptions, no resurrection |

## 17. Conversation privacy, retention and continuation

- node/local storage is canonical for Conversation and Context。
- default telemetry excludes transcript、prompt、Knowledge body、Memory body、secret、artifact。
- each Conversation exposes retention class、storage location、linked Work、export/delete eligibility。
- archive changes visibility, not authority or retention obligation。
- delete distinguishes local transcript、derived index、Memory retained from it、evidence/legal
  retention；one action cannot silently delete unrelated authority records。
- export is versioned、redacted、source-typed and lists omissions。
- cloud sync is absent unless separately authorized, encrypted, scoped and product-qualified。
- imported history remains archive until user links/retains/adopts specific facts。
- Continuation Package includes only authorized Task/objective/acceptance、decisions、approved
  transcript excerpts/summaries、Context refs、artifacts、Effects/evidence、non-secret Binding/
  budget、blocker、next action。
- hidden chain-of-thought、credentials、Provider-private session state、unauthorized body and
  unknown Effects are non-portable。

## 18. Persona usability scenarios and metrics

### 18.1 Office worker

1. First run links one supported Provider, imports/authorizes one document, starts a Conversation。
2. Reviews Context source and explicitly retains one follow-up Memory。
3. Returns next day from Home, sees source changed, reviews Continuation Checkpoint, continues。
4. Exports and deletes Conversation without deleting unrelated retained Memory。

### 18.2 Programmer

1. Discovers/registers one coding Agent and binds account/model。
2. Opens repository-scoped Conversation with visible Tool/Skill permissions。
3. Creates linked Work preview; unknown worktree changes fail closed。
4. Reviews Effects/check evidence and source-typed tokens; Agent/process does not self-complete。

### 18.3 Researcher

1. Enrolls authorized Knowledge source and sees index as derived/rebuildable。
2. Compares Context sources/versions and omission reasons。
3. Retains an explicit Memory with provenance, later tombstones it。
4. Resumes/export conversation with citations and declared losses。

### 18.4 Candidate metrics

| Success metric | Counter-metric |
|---|---|
| ≥80% complete coherent P0 journey without assistance | secret exposure / fake-ready = 0 |
| ≥80% returning users resume correct object in ≤30 s | duplicate Effect after resume = 0 |
| ≥90% identify source/freshness of token/cost fact | estimate shown as invoice = 0 |
| ≥90% distinguish Knowledge/Memory/Context in task test | implicit Memory retention = 0 |
| ≥85% review Binding/Context drift before continue | stale source silently reused = 0 |
| ≥90% keyboard-only completion for P0 | blocking inaccessible control = 0 |
| export/delete/purge scenarios give complete receipt | deleted Memory/index resurrection = 0 |

Thresholds are **RECOMMENDATION / untested**。See
[interaction scenarios](./04-personal-interaction-and-visual-spec.md),
[OSS boundaries](./12-open-source-reuse-assessment.md), and
[candidate acceptance](./10-validation-and-delivery-readiness.md).

## 19. Canonical release warning

This is **Personal Desktop 1.0 candidate** design。Accepted ADR-0036 and the formal plan still reserve
Personal `1.0.0` for Linux x86_64 and classify Web UI/Windows installer as post-1.0。No wording in this
document supersedes that release boundary。Canonical adoption requires the sequence in
[baseline delta map](./13-control-plane-baseline-to-personal-desktop-1.0-delta.md) after active
P7-T05/D13 ownership is resolved。
