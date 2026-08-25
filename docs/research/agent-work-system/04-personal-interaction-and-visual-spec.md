# CognitiveOS Personal 交互与视觉候选规范

Date: 2026-08-25
Status: **candidate / owner-confirmed discovery / non-canonical / no implementation authorization**

本规范依赖 [Personal 产品设计](03-personal-product-design.md)。所有像素、token、motion
与 component 值均为 **DESIGN CANDIDATE**，不是现有实现事实。现有 Web client、HTTP
capability 与后端真实性以 [Personal 架构](05-personal-architecture.md)为准。

## 1. Experience contract

- **OWNER DECISION**：共享 restrained CognitiveOS brand 与 semantic state system。
- **OWNER DECISION**：Personal 的性格是 calm、spacious、precise、local；macOS-like 指
  restraint、hierarchy、depth、fluid feedback，不复制 macOS window chrome。
- **OWNER DECISION**：Windows-first 遵循 Windows title bar、keyboard、high contrast、
  screen-reader 与 update conventions。
- **OWNER DECISION**：cards 仅用于 Home、onboarding、readiness；高频 Conversation、
  Library、Provider、binding、Activity、evidence 使用 list/table + master/detail。
- **FACT**：authority state、observation、cost source、freshness 不能通过装饰性 UI 混淆。
- **RECOMMENDATION**：首用优化 confidence 和 first value；重复使用优化 scan speed、
  keyboard、stable layout 与 recovery。

核心体验：

```text
First run: card-led resumable wizard → verified binding + first Conversation
Returning: Home Continue → Continuation Checkpoint → operational surface
Failure: inline cause → preserved input → one recovery action → receipt
```

## 2. Desktop shell and window behavior

### 2.1 Candidate dimensions

| Element | Candidate |
|---|---|
| Minimum supported window | 1024 × 680 CSS px；960 × 640 仅 bounded recovery |
| Target desktop | 1280–1600 × 800–1000 CSS px |
| Wide operations | ≥1440 px |
| Sidebar expanded | 220–240 px |
| Sidebar collapsed | 56–64 px；仅用户主动选择 |
| Master list | 320–420 px |
| Detail inspector | 340–480 px |
| Content max for prose/forms | 720–840 px |
| Top toolbar | 56 px |
| Conversation composer | 56–180 px auto-grow；不覆盖 Context status |
| Bottom status strip | 28–32 px only when durable state warrants |

960–1119 px：sidebar 可折叠，master/detail 成 sibling route；<960 px 不声称完整
administration，显示 narrow-window guidance，但必须保留 activation 与 bounded decision。

Window restart 恢复 route、selected object、filters、wizard checkpoint；不恢复未持久化
secret input。多个窗口默认不提供，避免并发编辑 ambiguity；未来多窗口必须有 object-level
dirty/lease policy。

### 2.2 Shell hierarchy

```text
┌ sidebar ┬ top toolbar: title / scope / search / primary action / status ┐
│ Home    ├────────────────────────────────────────────────────────────────┤
│ Work    │ page summary / source-freshness banner                        │
│ Convers.│ cards OR master list | detail / inspector                     │
│ Agents  │                                                                │
│ Library │                                                                │
│ Provider│                                                                │
│ Activity│ status / recovery footer                                      │
│ ─────── │                                                                │
│ System  │ footer utility                                                 │
└─────────┴────────────────────────────────────────────────────────────────┘
```

Sidebar 是 orientation，不是 feature-card gallery。active route、page title、object identity
至少同时提供三种 location cues。

## 3. Visual foundation candidates

### 3.1 Typography

使用 platform system UI stack。Windows 首选 `Segoe UI Variable` / `Segoe UI`；macOS
qualification 后使用 `-apple-system`；中文使用平台系统无衬线。不开 Web font download；
开启 optical sizing（可用时）。

| Token | Size/line | Weight | Use |
|---|---|---|---|
| `display-sm` | 28/36 | 650 | empty/activation promise |
| `title-lg` | 22/30 | 650 | page title |
| `title-md` | 18/26 | 600 | card/section title |
| `body` | 15/22 | 400 | normal copy |
| `body-compact` | 14/20 | 400 | operational rows |
| `label` | 13/18 | 550 | controls, metadata |
| `caption` | 12/16 | 450 | source/freshness/supporting facts |
| `mono` | 12/18 | 400 | digest/ref/revision；不用于普通正文 |

### 3.2 Spacing, radius, material

- base grid：4 px；layout rhythm：8 px；
- page gutter：24 px（target），16 px（narrow）；
- card padding：20–24 px；compact summary：16 px；
- card gap：16 px；operational row：40–48 px；
- radius：card 14–16 px；control 7–9 px；badge/pill 999 px only for compact labels；
- elevation：最多三层；base 0、card subtle、modal/dialog highest；
- material：opaque neutral surfaces；translucency 仅 sidebar/titlebar 的非关键层，不能承载
  status、form、evidence 或 economic truth；reduced transparency 可关闭；
- separators 优先于 nested card-on-card。

### 3.3 Semantic color

具体色值必须在 rendered contrast test 后固定。candidate roles：

| Role | Meaning |
|---|---|
| neutral | structure/unknown |
| accent | selected/primary action，不表示 success |
| success | independently verified/ready within named scope |
| warning | stale/partial/attention |
| danger | denied/failed/critical |
| info | observation/in-progress |

每种状态同时使用 icon + text + shape。green 不能表示“process exited”；gray 不能表示
healthy。light/dark theme 使用相同 semantic role，不直接反转 hue。

### 3.4 Interaction states

| State | Candidate treatment |
|---|---|
| Focus | 2 px high-contrast outer ring + 1 px offset |
| Hover | surface shift；不能承载唯一信息 |
| Pressed | 1–2 px inset/contrast change；无 layout jump |
| Selected | accent rail/background + `aria-selected` |
| Disabled | lower contrast + visible reason；若不可用需解释 |
| Busy | control keeps label + spinner；prevent duplicate submit |
| Error | inline field/action message；不只 toast |

## 4. Layout patterns

### 4.1 Card-led surfaces

用于 Home、wizard、readiness milestone、small status summary。card anatomy：

1. eyebrow/source；
2. title；
3. status + freshness；
4. one-sentence consequence；
5. one primary action；
6. optional secondary link；
7. provenance/limitations footer。

不用于超过约 8 个需要逐列比较的对象。不要用 giant metric、hero gradient、decorative KPI。

### 4.2 Operational list/table + master/detail

用于 Agents、Providers、Bindings、Activity、Evidence：

- sticky header、visible sort、reversible filters；
- stable columns、selected row remains visible；
- detail pane 支持 previous/next；
- filter/search/sort 保存在 route state；
- bulk action 默认只读/refresh/export；mutation bulk action 需 selection summary 与 partial
  failure receipt；
- narrow window 变成 list route → detail route，back 恢复 selection/scroll。

### 4.3 Inspector, tabs, breadcrumbs

- inspector 展示 provenance、raw bounded metadata、limitations，不替代 primary detail；
- tabs 只用于同一 object 的 sibling views；
- breadcrumbs 仅 deep object path 使用，如 Providers → Account → Binding；
- wizard 使用 stepper，有 current/completed/error states、save/resume 与 review step。

## 5. Component inventory and anatomy

### 5.1 Agent card

- Profile name/purpose；
- Instance/Installation count；
- readiness state + reason；
- capability source；
- current binding summary；
- freshness/provenance；
- CTA：Review / Verify / Bind / Open。

### 5.2 Readiness card

- scope（Agent、Provider、Binding、System）；
- state：ready/partial/stale/blocked/unknown；
- check source + timestamp；
- blocking fact；
- next action；
- “ready within scope” disclaimer。

### 5.3 Provider account row/card

- Provider + account label/type；
- auth state（不显示 secret）；
- entitlement/model readiness；
- binding count；
- usage/cost source + period；
- attention reason；
- revision/provenance。

### 5.4 Entitlement, binding, usage/cost

| Component | Required anatomy |
|---|---|
| Entitlement | right/unit/window/source/freshness/unknown reason |
| Binding | Profile + Instance + account + auth ref + model + revision + eligibility |
| Usage | amount/unit/period/source/observed_at/coverage |
| Cost | amount/currency/period/source class/estimate-or-invoice label |
| Budget | scope/limit/enforcement class/remaining if authoritative only |

### 5.5 Provenance and freshness

- `ProvenanceBadge`：Provider-reported / Locally observed / User-declared /
  Derived estimate / External invoice ref / Unavailable；
- `FreshnessLabel`：timestamp + TTL state + refresh action；
- tooltip 只能补充，不是唯一 source；
- screen reader label 包含 source 与 freshness。

### 5.6 Error/recovery receipt

1. what failed；
2. durable facts preserved；
3. effects definitely/maybe/not occurred；
4. retry safety；
5. recommended next action；
6. owner；
7. copy-redacted diagnostic ref。

## 6. Forms and wizard behavior

### 6.1 Field anatomy

label → required/optional → control → helper/constraint → inline validation →
source/security note。placeholder 不是 label。unknown/unavailable 使用 explicit selector，
不要求编造值。

### 6.2 Validation timing

- syntax 在 blur 或 submit；
- uniqueness/remote verification 在显式 action 后；
- costly/security action 在 review step；
- stale revision 返回 conflict banner，保留全部 input；
- secret field 一次性 handoff，提交后清空，不能 reveal；
- back/forward 保留 non-secret draft。

### 6.3 Dialog/sheet

- modal 只用于 narrow consequential decision；
- destructive confirm 说明 object、blocking bindings、irreversibility；
- sheet 用于 supplementary inspector，不用于完整 multi-step wizard；
- focus trap、return focus、Esc/cancel、screen-reader title 必须存在。

## 7. State matrices

### 7.1 Data surface

| State | Visual | Action |
|---|---|---|
| Empty | promise card + concrete CTA | Discover/Register/Link |
| Loading | preserve shell/header；skeleton matches layout | cancel if long |
| Partial | populated content + missing-source card | repair source |
| Stale | banner + per-row freshness | refresh/reverify |
| Permission | scope/reason/deny path | request or continue manually |
| Error | local error receipt | retry/edit/copy details |
| Success | durable receipt, no confetti dependency | open object/next step |

### 7.2 Wizard recovery

| Failure | Preserve | Clear | Recovery |
|---|---|---|---|
| discovery denied | manual facts | scan temp data | manual register |
| import conflict | selections/corrections | none | review diff |
| auth expired | account metadata/wizard step | secret transient | reauth |
| readiness unknown | registration/binding draft | none | retry/change model |
| stale binding revision | all inputs | none | refresh/review |
| cost unavailable | binding/readiness | none | accept honest unknown |

## 8. Motion and progress

- standard transition 120–180 ms；larger spatial transition 180–240 ms；
- critically damped, interruptible；no fake progress；
- selection moves inspector without spring overshoot；
- wizard step transition preserves direction；
- skeleton only while layout unknown；known stale data stays visible；
- reduced motion：instant state change/cross-fade ≤80 ms；
- progress must expose durable stage、last update、cancel/retry/resume semantics。

## 9. Keyboard and command model

| Shortcut candidate | Action |
|---|---|
| `Ctrl/Cmd+K` | command/search |
| `Ctrl/Cmd+L` | current inventory search |
| `Ctrl/Cmd+R` | bounded refresh current scope；不触发 mutation |
| `Alt+Left/Right` | route navigation |
| `J/K` | next/previous row when list focused |
| `Enter` | open selected object |
| `Esc` | close inspector/dialog |

command palette order：context action → object → destination。permission-blocked/unsupported
command 显示 reason；执行后 focus 返回 invocation point。

## 10. Accessibility and localization

- WCAG 2.2 AA candidate；normal text contrast ≥4.5:1，large ≥3:1；
- focus appearance 不被 sticky chrome 遮挡；
- tables 使用 semantic header/sort；interactive grid 仅在 arrow-key model 完整时；
- status live region 只 announce meaningful transition，避免 noisy polling；
- chart/visual 必须有 text alternative；
- Chinese/English/German-like 1.5× expansion；number/date/currency/units locale-aware；
- digest/ref 允许 wrap/copy；不通过 truncation 隐藏 identity distinction；
- screen-reader 顺序：object→state→source→freshness→next action。

## 11. Tray and notifications

**OWNER DECISION**：system tray 必须 opt-in。允许通知：

1. activation success/failure；
2. actionable readiness loss/expiry；
3. P1 Task decision/failure。

通知包含 object、reason、timestamp、single action，并 deep-link 到同一 durable route。
禁止 routine usage/cost noise、secret/account detail、transient process events。permission denied
时仍在 Home 显示 equivalent attention。

## 12. Usability acceptance scenarios

| Scenario | Acceptance candidate |
|---|---|
| First run | 新用户 ≤10 min 达到 verified binding；可中断/resume；无 secret exposure |
| Returning | 从 Home 在 ≤60 s 找到唯一 blocking readiness fact |
| Expired auth | 保留 account/binding context；reauth 后 reverify；无 duplicate binding |
| Unknown cost | 明示 unavailable/source；仍可在 policy 允许时 ready |
| Lost readiness | notification/Home card deep-link 到 exact repair action |
| Keyboard only | wizard、table、detail、dialog、binding 全流程无 focus trap |
| Narrow 960 px | sidebar/details 不重叠；primary action 与 current location 可见 |
| Reduced motion | 无 orientation loss、fake progress 或不可用 transition |

## 13. Baseline delta and non-claims

相对 `clients/docs/design/11–25` 与 `34–41` dated baseline：

- 延续 token/state/honesty、sidebar、master/detail、evidence；
- 以 native desktop window behavior 包裹 existing Web presentation；
- 首用改为 spacious card wizard，P0 围绕 readiness/binding；
- operational density 只用于 comparison-heavy views；
- final hues、font metrics、framework、component library 尚未选择。

No rendered prototype、usability test、contrast measurement、implementation、Gate 或 release
evidence 在本文件中产生。

## 14. Expanded component anatomy

### 14.1 Provider and economic facts

`ProviderAccountRow`：

1. Provider icon candidate（first-party/trademark-cleared only）+ account label/type；
2. management mode；
3. auth state + exact reason；
4. entitlement/model readiness；
5. Agent Binding count and warnings；
6. usage period + source；
7. cost class + currency or `Unavailable`；
8. freshness/coverage；
9. primary action appropriate to mode。

`EconomicFact` never shows a generic confidence meter. It shows source class、period、coverage、
observed_at、price/invoice revision and limitations. Estimate uses neutral/info treatment, never
success green.

### 14.2 Agent and Binding readiness

`AgentReadinessHeader`：

- Profile identity/purpose；
- selected Instance/Installation；
- capability source；
- exact account/model Binding；
- health/freshness；
- allowed Library coverage；
- blockers and one next action。

`BindingPreview` uses relation grammar:

```text
Agent/Profile/Instance
uses Account + AuthRef + Model
within Budget + Resource scope
affecting Conversations/Work
at revision
```

### 14.3 Library family components

| Family | Row identity | Required state | Primary lifecycle |
|---|---|---|---|
| Knowledge | source + scope + version | authorization/index/freshness | enroll/reindex/purge |
| Memory | summary + source + scope | version/last-used/tombstone | retain/edit-version/forget |
| Skill | package + publisher + revision | installed/enabled/bound | install/pin/enable/remove |
| Tool | descriptor + provider | authorized/available/effect class | inspect/allow/revoke/test |

Shared Library shell may provide search/filter/selection, but buttons and state models remain family-
specific.

### 14.4 Conversation and Context

`ConversationHeader`：

- title/status/local storage；
- Agent + Provider/model；
- linked Work；
- token source/period；
- retention class；
- Continue / Checkpoint action。

`ContextBar`：

- current ContextView revision/digest；
- included source count；
- omitted/truncated/conflicted count；
- freshness；
- token allocation；
- open inspector。

`ContextInspector` groups **Included / Omitted / Changed / Unavailable** and shows source, scope,
version, authorization, loss reason and token contribution. It does not expose hidden chain-of-thought.

### 14.5 Usage

Usage visual defaults to table/list, not chart：

- period；
- Agent/Binding/Conversation/Work dimensions；
- input/output/cache/total token if source provides；
- source and coverage；
- model/pricing revision；
- cost class；
- export/import provenance。

Charts are optional P2 summaries with accessible table equivalents and zero/unknown separated.

### 14.6 Evidence, errors and states

`EvidenceReceipt`：claim → verifier → observed post-state → source → timestamp → limitations →
open artifact/reference。

`RecoveryReceipt`：failure stage → durable facts preserved → definite/maybe/no Effect →
retry safety → next action → owner。

`StateBanner` is reserved for page-level conditions (offline、partial coverage、permission、
compatibility), not routine row status.

## 15. Signature interaction — Continuation Checkpoint

Continuation Checkpoint is the distinctive interaction between “open history” and “resume safely”.
It appears when a Conversation or Work returns after meaningful drift, transfer, crash, import or
explicit checkpoint.

### 15.1 Trigger

- Provider/account/model Binding changed；
- Context source/version/authorization changed；
- Memory retained/forgotten；
- Skill/Tool availability or permission changed；
- Work epoch/status/budget changed；
- previous Effect outcome remains unknown；
- imported/exported continuation requires review。

### 15.2 Anatomy

```text
Continue <Conversation / Work>
Last durable checkpoint · timestamp · revision

What changed
  Binding      old → new
  Context      +included / -removed / changed / unavailable
  Memory       retained / tombstoned
  Skills/Tools enabled / revoked / unsupported
  Work         epoch / budget / blocker

What will be used now
  Agent · account/model · sources · token bound · permissions

Risk/limitations
  unknown Effects · stale facts · omitted content

[Review details] [Continue with this state]
```

If no meaningful drift exists, checkpoint collapses to a quiet one-line receipt. It never invents
“safe” from a generic score.

### 15.3 Interaction

- changes animate as bounded 160–220 ms diff transitions；
- user can inspect before/after without losing place；
- primary continue is disabled only for real fail-closed blockers；
- owner decision produces durable receipt；
- unknown Effect cannot be accepted through a cosmetic checkbox；
- screen reader announces change count, blockers and primary action。

## 16. Motion and physicality details

| Motion | Candidate |
|---|---|
| hover/focus surface | 90–120 ms ease-out |
| selection/detail | 140–180 ms critically damped |
| route/master-detail | 180–240 ms, interruptible |
| dialog/sheet | 180–220 ms, no bounce |
| checkpoint diff | 160–220 ms |
| progress | real stage updates only |

Spring implementation may use a critically damped model around `stiffness 420–520`,
`damping 38–48`, `mass 1` only after rendered tuning. No overshoot for evidence、economic facts、
destructive confirmation or table selection. Input interrupts current animation and targets the new
state rather than queuing.

Reduced motion：disable spatial translation/spring；use instant update or ≤80 ms cross-fade。Reduced
transparency：replace material with opaque semantic surface while preserving separators and contrast。

## 17. Theme, icons, accessibility and localization details

### 17.1 Candidate color roles

Exact values require rendered measurement. Candidate light roles may begin from：

- canvas `#F5F6F8`；
- surface `#FFFFFF`；
- surface-muted `#EEF1F5`；
- text-primary `#171A1F`；
- text-secondary `#5D6673`；
- separator `#D8DDE5`；
- accent `#0A66D8`；
- success `#16834A`；
- warning `#A45A00`；
- danger `#C43131`。

Dark/high-contrast values are independently tuned, not mechanically inverted. Windows forced-colors
must retain selected/focus/disabled/error semantics without background images.

### 17.2 Icons

- consistent 16/20/24 px optical sizes；
- 1.5–1.75 px stroke candidate；
- label every unfamiliar icon；
- no colored vendor logo without trademark/license review；
- destructive icon never acts as sole warning；
- status combines icon + text + structure。

### 17.3 Keyboard

Windows primary shortcut labels use `Ctrl`; macOS qualification maps to `⌘` at runtime。`Ctrl+K`
global command/search；`Ctrl+Shift+P` optional command alias；`Ctrl+L` current-list search only if it
does not conflict with embedded browser conventions；`F6` cycles shell regions；`Shift+F10` opens
row context menu；`Alt+Left` route back；`Esc` dismisses bounded layer。

### 17.4 Screen reader and text expansion

- landmarks：titlebar/sidebar/main/inspector/status；
- object heading precedes state/source/freshness/action；
- Context diff uses additions/removals text, not color；
- table virtualisation retains row count/position and focus；
- live regions announce durable stage changes only；
- English/German-like expansion 1.5×；Chinese/English mixed labels 2-line safe；
- no truncation of account/model/revision when ambiguity affects action；
- date/time/number/currency/token units locale-aware。

## 18. Scenario-based usability acceptance

### 18.1 Office worker

| Scenario | UI acceptance |
|---|---|
| First run | reaches first Conversation from activation without learning daemon terms |
| Return | Home Continue identifies correct thread and source change within 30 s |
| Recovery | expired auth retains Conversation/Context and offers reauth, not blank reset |
| Retention | can explain why one item became Memory and another remained transcript |

### 18.2 Programmer

| Scenario | UI acceptance |
|---|---|
| First run | sees exact Agent Instance/account/model/Tool scope before ready |
| Return | checkpoint exposes repo/Binding/Tool drift before governed Work |
| Recovery | unknown Effect/worktree change blocks safely with preserved diagnostics |
| Evidence | process exit and verifier acceptance are visually distinct |

### 18.3 Researcher

| Scenario | UI acceptance |
|---|---|
| First run | adds one authorized source and sees provenance/version |
| Return | Context inspector explains added/removed/stale sources and token contribution |
| Recovery | offline mode keeps local archive and names unavailable refresh/index actions |
| Privacy | export/delete/purge receipt clearly separates transcript, index, Memory, evidence |

All scenarios run keyboard-only、screen reader、200% zoom、reduced motion、reduced transparency、
Windows high contrast、light/dark and 1.5× text expansion variants.

## 19. Explicitly rejected visual patterns

- copied macOS traffic-light chrome、menu bar、Finder iconography or trademarked wallpaper；
- decorative full-window glassmorphism、blur behind text/forms/evidence；
- card walls for comparison-heavy inventories；
- hero gradients、giant KPI numerals、dashboard charts without decisions；
- nested cards deeper than one level；
- generic confidence/progress/safety score hiding source and limitations；
- anthropomorphic employee portraits/org chart；
- color-only state、toast-only error、disabled action without reason；
- fake streaming/progress or confetti as completion proof；
- dense enterprise admin grid applied to first-run Personal。

See [Personal product design](03-personal-product-design.md),
[candidate architecture](05-personal-architecture.md),
[open-source reuse](12-open-source-reuse-assessment.md), and
[baseline delta](13-control-plane-baseline-to-personal-desktop-1.0-delta.md).
