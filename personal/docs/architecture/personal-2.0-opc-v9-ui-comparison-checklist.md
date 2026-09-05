# Personal 2.0.0 — frozen v9 ↔ daemon `/ui/` module-by-module comparison checklist

# 个人 2.0.0 — 冻结 v9 与 daemon `/ui/` 逐模块对照清单

- Status: **informative** judgement sheet / 非实现 / 非 Gate. Written by
  `P13-T12/D01`; **filled by `P13-T12/D02`** on exact daemon `/ui/` revision
  `c8691923cd3988f0ffee9123752e073480aea5e9` (`DEV-LINUX-NATIVE-01` guest +
  host Chrome). Remaining `not-run` is missing NVDA, missing Windows native
  chrome / forced-colors, or a disposable runtime without a live Project /
  later wizard step.
- Change class: `implementation-only` documentation (no product, contract,
  test, CSS, canvas or IA change).
- Denominator: the 19 maintainable units of
  [`00-maintenance-index.md` table A](../../../clients/docs/design/opc-2.0/00-maintenance-index.md)
  (`M-SHELL` … `M-X`; aliases `M-MEMBERS-INIT` = `M-CREATE-3`,
  `M-TEST-STAGE` = `M-CREATE-4`), each mapped to its v9 scene(s) and to its
  authoritative source in table B. Coverage of this file: **19 / 19**.
- Frozen prototype (not the product): canvas v9
  `clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx`
  (read only). Product origin: daemon-served `/ui/`. Canvas screenshots are
  never acceptance; Vite preview is never the product origin.
- Visual rules: [Visual UI specification](personal-2.0-opc-visual-ui-spec.md)
  (§5 themes, §6 layout, §7 keyboard/focus, §9 nine × nine). Routes and
  daemon facts: [v9 → daemon mapping](personal-2.0-opc-v9-implementation-mapping.md)
  §6.0 / §6.1; capability classes: [11 design-to-code matrix](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md).
- Lease: `lease/personal/P13-T12/visual-qualification` (D02). Claim ceiling `hypothesis`.
- Non-claims: this sheet proves nothing by existing. `not-run` is never pass.
  Rendered / NVDA / 200% / host-theme evidence from a registered host's local
  browser against a pushed exact-revision guest daemon `/ui/`
  (`DEV-LINUX-NATIVE-01` over the documented SSH tunnel) is implementation
  evidence only; Windows native chrome cells stay `not-run` until `P13-T13`
  qualifies `DEV-WINDOWS-NATIVE-OPC-01`. Nothing here is Gate, release,
  Profile, or `P11-T15`.

---

## 0. How D02 fills this sheet / D02 怎么填

1. Pin **one** exact `/ui/` revision (pushed commit SHA) and one environment ID
   from `PERSONAL-TEST-ENVIRONMENTS.md` (`DEV-LINUX-NATIVE-01` guest daemon
   viewed from the registered host browser; `DEV-WINDOWS-NATIVE-OPC-01` only
   after `P13-T13/D01`). Write both in §7 before the first cell.
2. Judgement vocabulary per cell: `pass` / `fail` / `partial` / `not-run` /
   `not_available` (+ one-line reason and evidence pointer: report row,
   screenshot digest kept outside Git, or DOM dump). A skipped cell is
   `not-run`, never `pass`. A cell whose backend does not exist yet is judged
   on the **honest state** (§9.10 of the spec): `pass` if the surface renders
   `Requires-backend` / empty honestly, `fail` if it draws a fake control.
3. Append each finished cell to the D02 running report first
   (`TEST-REPORT-INCREMENTAL-01`), then transcribe the judgement here in the
   same commit. Correct a published cell only by a superseding row.
4. Do not change the IA, regenerate the canvas, edit `tokens.css`, or narrow a
   rule to make a cell pass. Drift between v9 and the product docs is recorded
   in spec §13 and §6 below; D02 records, it does not decide.

Judgement columns below are pre-filled `not-run` (D01). Environment and
revision columns are `—` until D02.

---

## 1. Module rows (table A × table B × `/ui/`) / 逐模块

Columns: **Module** (table A id) · **v9 scene(s)** · **Authority** (table B
product truth → design chapter) · **v9 visual / behavioural expectation** (what
the frozen prototype shows; product docs win where they differ, see §6) ·
**`/ui/` route + component that must realise it** (mapping §6.0; selectors are
the `data-page` / `data-region` anchors in `clients/pc/web/src`) · **Daemon
fact it may render** (Now) and **honest state otherwise** (Requires-backend /
Requires-environment; zero fake Create / Activate / Approve / Connect / Install
/ Publish) · **Owning card** · **Judgement** (D02).

| Module | v9 scene(s) | Authority (table B) | v9 visual / behavioural expectation | `/ui/` route + component | Daemon fact it may render (Now) → honest state otherwise | Owning card(s) | Judgement |
|---|---|---|---|---|---|---|---|
| `M-SHELL` App shell | all scenes (left nav shared); `empty-home` hides the rail | [scope §3.1](../product/personal-2.0-scope.md); [web-ui-design §4](../product/web-ui-design.md) → [02 IA](../../../clients/docs/design/opc-2.0/02-information-architecture-and-app-shell.md), [10](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md) | Locked three columns (nav / canvas / conversation); L1 今日 / 项目 / 知识, 设置 at the bottom; Projects submenu 详情 / 成员 / 运行 / 产出 under an open live Project; context header with location + state tag; rail always the third column; narrow window scrolls horizontally, never stacks; Team / Inbox absent | `AppShell` (`.cp-shell`: `StatusStrip` + `PrimaryNav` + `#main` + `AssistantRail`), `ProjectWorkNav`, hashes `#/` `#/projects` `#/knowledge` `#/settings` | `GET /personal/{status,readiness}` in the strip; Project list for rail hide/show → no daemon: shell still renders with honest empty | `P11-T13` (done), `P12-T02` (done); narrow no-stack CSS: **no owning card yet** (spec §13-a) | `pass` |
| `M-EMPTY` First-run / empty Home | `empty-home`; State Lab `today` × `empty` | [user-journeys §1](../product/user-journeys.md) → [03](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md), [04](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md), [09 empty](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md) | Two columns (rail hidden); one info tag 还没有项目, one title, one lede, exactly one primary 创建项目; Knowledge shows a 锁 tag; no KPI wall, no demo Project | `#/` `TodayPage` `[data-page=opc-today]` empty branch: `ProjectAuthorityPanel` `EmptyState` + `Start create`; `AppShell.hideAssistantRail` | `GET` Project list = empty (P11-T03) → honest `EmptyState`; never a demo row | `P12-T02` (done) | `pass` |
| `M-CREATE-1` Create ① | `create-init` | [user-journeys §1](../product/user-journeys.md), [product-design](../product/product-design.md) → [04](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md), [01](../../../clients/docs/design/opc-2.0/01-product-model-and-jtbd.md) | Setup header ① + 离开并保留草稿; item-by-item confirm rail (业务描述 + 16 confirm items) with step-dot tablist, one 确认本项 primary per item, 上一项/下一项, live status line; Provider unbound → 去设置连接模型 instead of 下一项; stale items tagged 已过时; 总预览 before ②; Requires-backend honesty block | `#/projects/new` `CreateWizardPage` `[data-page=opc-create-wizard][data-step]` | Draft/Charter preview digest via Project management face (P11-T03 confirm-before-activate) → no authority: local draft labelled sample, no fake Activate | `P12-T02` (done) | `pass` |
| `M-CREATE-2` Create ② | `create-process` | [user-journeys §1.4](../product/user-journeys.md), [scope §3.1](../product/personal-2.0-scope.md) → [04](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md) | Horizontal process axis (one stage open at a time, `aria-current="step"`, 先完成前环 disabled reason); per-stage 输入 / 执行方式 / 权限后果 fields with Enter-to-notify; 确认这一环 primary, 本环留缺口 secondary; 确认总目标与项目触发 before 进入 ③; gaps cannot be marked confirmed | `#/projects/new` step ② `CreateWizardPage` | Plan / process-axis preview digest (P11-T03) → no authority: sample axis, no silent write | `P12-T02` (done) | `not-run` |
| `M-CREATE-3` / `M-MEMBERS-INIT` Create ③ | `create-members` | [user-journeys §1.5](../product/user-journeys.md), [opc-product-model](../product/opc-product-model.md) → [04](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md), [05](../../../clients/docs/design/opc-2.0/05-team-roles-employees-and-conversations.md) | 创建成员 via confirm dialog; roster table 岗位 / 模型 / 就位 with `<progress>` 就位 n / m; 当前初始化 block = progress + current item title only (kicker / title / hint), tablist of members; model `select` required (unselected = 待定, never silent); 请助手生成本人执行方式 → 确认此人已就位 → 下一位; 拒绝加入 = not joined; 全员就位，进入测试 gated | `#/projects/new` step ③ `CreateWizardPage`; shared `MemberConfigPage` | Employee walking skeleton (P11-T04): `roster.register` → `seat.request` → `seat.confirm` (P12-T04) → no model: `pending`; runtime generation by the Assistant is candidate-only (P13-T03) | `P12-T02`, `P12-T04` (done); real generation `P13-T03` | `not-run` |
| `M-CREATE-4` / `M-TEST-STAGE` Create ④ | `create-test` | [user-journeys §1.6](../product/user-journeys.md) → [04](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md), [09](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md) | Process axis with owner seated/unseated per stage; 成员是否就位 notice (good / warn / bad) + seat-check list of runtime slots; 开始测 disabled unless seated and online; outcome pass / fail / unknown notices; 通过，下一环 / 末环通过，进入 ⑤ gated on `pass`; unknown cannot pass; offline cannot start; openable sub-result button on pass | `#/projects/new` step ④ `CreateWizardPage` | Real stage test = Attempt + independent verifier (P13-T02 / P13-T04) → until then `Requires-backend` honesty, no fake 开始测 | `P12-T02` (done); real test `P13-T02`, `P13-T04` | `not-run` |
| `M-CREATE-5` Create ⑤ | `create-joint` | [user-journeys §1.7](../product/user-journeys.md) → [04 验收](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md), [12](../../../clients/docs/design/opc-2.0/12-scenario-and-heuristic-review.md) | Ordered run-steps list (waiting / current / done / blocked); joint outcome notices; 验收，进入今日 is the **only** first success and the only primary; failure names the stage and offers 回 ④ / ② / ③; explicit "没有 Publish 按钮"; offline cannot start; unknown cannot accept | `#/projects/new` step ⑤ `CreateWizardPage` → `#/` | Acceptance = independent verification + daemon acceptance (A4; P13-T04) → until then honest sample, no fake Publish/Activate | `P12-T02` (done); real acceptance `P13-T04` | `not-run` |
| `M-TODAY-INCOMPLETE` Today (create unfinished) | `today-incomplete` | [user-journeys §1 end / §2](../product/user-journeys.md) → [03](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md) | Header 创建还没走完 + stage number; one work surface 继续未完成的创建 with one primary 继续第 n 段; no decision packet, no run overview | `#/` `TodayPage` `[data-surface=today-incomplete]` (`Continue create` → `#/projects/new`) | Project list rows in `creating` state (P11-T03) → creating-only table, no packet | `P12-T05` (done) | `not-run` |
| `M-TODAY` Today (live) | `today`; State Lab `today` × working / blocked / unknown / offline | [user-journeys §2](../product/user-journeys.md), [product-design](../product/product-design.md) → [03](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md), [07](../../../clients/docs/design/opc-2.0/07-inbox-approval-and-recovery.md) | Header + period segmented control 今日 / 本周 / 本月; host-state notices; **one** decision packet (要你拍板 tag, question, four facts 可逆性 / 备选 / 费用 / 为何先 A, primary 去处理这一件拍板 + text 以后再说) collapsed to a quiet surface when nothing is pending; three run counts (创建 / 已上线 / 发生阻塞, unknown = 说不清 never 0); run overview one row per live Project (state · completed runs · current stage · duration); no four swimlanes, no KPI wall; chat cannot approve | `#/` `TodayPage` `[data-surface=today]`, `[data-region=opc-today-packets]`, `DaemonReadPanel` region `opc-hitl` | Pending previews for the live Project (P11-T09 / P12-T05) → packets; Project list → overview rows; occurrence counts / current stage / duration = `P13-T05/D02` → until then rows show daemon-stated columns only, unknown never 0 | `P12-T05` (done); overview rows + period toggle `P13-T05/D02` | `not-run` |
| `M-PROJECTS` Projects list | `projects` | [user-journeys §1.8](../product/user-journeys.md) → [03 list](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md) | Empty: no create button here, 回今日; creating: only the draft with 继续创建; live: one panel per Project (name, industry · status meta, 目标 / 周期 / 费用 facts), **one primary 打开** + text links 成员 / 运行 / 产出 (+ 复制为草稿); copy = inactive 副本 banner (总预览 first, no secrets / in-flight tasks / receipts / skips) | `#/projects` `ProjectsPage` `[data-page=opc-projects]` | Project aggregate list (P11-T03 / P12-T03) → copy-project = `P13-T09/D01` → until then no fake 复制 | `P12-T03` (done); copy `P13-T09` | `pass` |
| `M-LIVE-PROJECT` Live Project (详情 / 运行 / 产出) | `project-detail`; `project-runs`; `project-outputs` | [user-journeys §3](../product/user-journeys.md), [scope §3.1](../product/personal-2.0-scope.md) → [03 operating canvas](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md), [10](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md) | 详情: read-only charter facts (名称 / 目标 / 周期 / 状态 / 费用 / 流程环节 / 成员) + read-only process axis + text links to 成员 / 运行 / 产出, Project switcher; no CEO six-step rail. 运行: three run counts, process axis with `data-mark` auth / verify keylines, stage detail (当前步骤, ledger facts, 要你授权 / 要你核对 tag → primary 去授权预览 / 去核对), collapsed 普通过程痕迹, acceptance link **only on the last ring**. 产出: 先选后看 listbox + composition, 请助手换一种展示, HITL-needing output deep-links to the canvas | `#/projects/:id` `ProjectDetailPage` `[data-page=opc-project-detail]`; `#/projects/:id/runs` `ProjectRunsPage`; `#/projects/:id/outputs` `ProjectOutputsPage` `[data-region=opc-output-selected]`; `ProjectWorkNav` | `GET detail` / `axis` / `roster` (P12-T03); occurrence ledger + Attempt history (`P13-T05/D02`), openable outputs + publication preview (`P13-T04/D02`) → until then axis-only / output_contract-only with honest empty | `P12-T03` (done); `P13-T04`, `P13-T05` | `not-run` |
| `M-ADD-MEMBER` Add a member (live) | `add-member` | [user-journeys §4](../product/user-journeys.md) → [05](../../../clients/docs/design/opc-2.0/05-team-roles-employees-and-conversations.md) | Setup header; 现有班子 list from the **current Project's real roster** (never a demo list); 新岗位 fields 岗位名 / 做什么、交出什么 / 模型（必选）; 确认加入 primary (gated on a name), 拒绝, 去设置选模型 when unselected; joined notice → 初始化执行方式; no "install MCP first" | `#/projects/:id/members/new` `AddMemberPage` `[data-page=opc-add-member]`, `[data-region=opc-join-written]` | Write join = `roster.register` → `seat.request` → `seat.confirm` (P12-T04) → no model: `pending` | `P12-T04` (done) | `not-run` |
| `M-MEMBER-CONFIG` Member configuration | `project-members`; `member-config` | [opc-product-model](../product/opc-product-model.md), [agent-integration](../product/agent-integration-and-conversations.md) → [05](../../../clients/docs/design/opc-2.0/05-team-roles-employees-and-conversations.md), [08 Skill/MCP](../../../clients/docs/design/opc-2.0/08-settings-agents-providers-and-usage.md) | 成员管理: listbox of members left, **unselected empty detail** right (never default-first), switching Project clears selection, 加人 in the header; eight tabs in product order 职责 / 输入 / 输出 / 技能 / 工具 / 工作说明 / 周期与触发 / 连接与权限; identity (model, seated, stage) in the detail header; 输入 read-only process contract; 输出 editable 交出什么; Enter → confirm dialog → chat; no Install button; member-config page shares the same panel | `#/projects/:id/members` `ProjectMembersPage` `[data-page=opc-project-members]`, `[data-region=opc-member-selected]`; `#/projects/:id/members/:mid` `MemberConfigPage` `[data-page=opc-member-config]`, `memberTabs.ts` | Employee roster + eight-tab facts (P11-T04 / P12-T04); Skill / MCP grants = `P13-T10` → until then honest text, no Install | `P12-T04` (done); grants `P13-T10` | `not-run` |
| `M-HITL` HITL canvas | `hitl`; State Lab `hitl` × blocked / unknown / offline | [user-journeys §6](../product/user-journeys.md), [scope §3.1](../product/personal-2.0-scope.md) → [07](../../../clients/docs/design/opc-2.0/07-inbox-approval-and-recovery.md) | On the Project canvas (not a standalone route): 将做什么 → 完整预览 / 差异 facts → checkbox 本周此类不再问（到期失效，设置里可收回）→ 批准 (primary; only pending ∧ fresh ∧ not executing ∧ not narrowed) / 改窄 (voids the preview, needs a new one) / 拒绝 / 停 (only while executing); stale and unknown notices block approval; offline cannot approve external; empty = 没有待拍板的预览; chat only announces and links | `#/projects/:id?preview=<id>` `HitlCanvasTable` `[data-region=opc-hitl-actions]`, `[data-region=opc-hitl-written]`; Today deep link `hitlCanvasPath`; rail `[data-region=opc-rail-hitl]` | Digest-bound Confirm / Narrow / Reject on management HTTP (P11-T09 / P12-T06); Stop only when execution is backed (P13-T02) → stale / unknown cannot confirm; no chat Approve | `P12-T06` (done); Stop `P13-T02` | `not-run` |
| `M-KNOWLEDGE` Knowledge | `knowledge`; from ② for the current draft | [user-journeys §5](../product/user-journeys.md), [knowledge-memory-vault](../product/knowledge-memory-vault.md) → [06](../../../clients/docs/design/opc-2.0/06-knowledge-vault-and-memory.md) | Locked header when no Project; segmented 项目资料 / 导入 / 为什么用这段 / 记忆; filters (范围 / 类型 / 关键词) + result list with kind tags; empty = 导入资料 primary; import form (范围 / 复制或引用 / 来源种类) with phases importing / duplicate / parse-fail (原件保留) / secret-detected (→ SecretStore, never into the Vault) / indexed; Why table 片段 / 为何选中 / 新鲜度; memory record with 忘记这条 → tombstone | `#/knowledge` `KnowledgePage` `[data-page=opc-knowledge]`, `[data-region=opc-vault-ingest]`, `[data-region=opc-why-fragment]`, `opc-vault`, `opc-memory` | `vault.import` (owner paste) + `vault.index` Why-this-fragment (P11-T10 / P12-T07); provenance / rights / freshness labels, reindex, Memory inspect / correct / promote / forget UI = `P13-T07/D01` → until then honest text on management HTTP | `P12-T07` (done); `P13-T07` | `pass` |
| `M-SETTINGS` Settings | `settings` | [user-journeys §8](../product/user-journeys.md), [account-hub](../product/account-hub.md) → [08](../../../clients/docs/design/opc-2.0/08-settings-agents-providers-and-usage.md) | Header 设置; 模型连接 panel: 供应商模板 `select` (Anthropic / OpenAI / Google / 自定义), custom URL / 兼容模式 / 模型名, password key field that never echoes and clears on handoff, one primary 交接密钥, status line 尚未连接 / 已交接 / 失败 · named cause; 本周不再问 panel with 收回跳过; 通知与恢复 panel; advanced diagnostics and `state-lab` one disclosure deeper, hidden by default; no billing, engine store, Inbox, Installed Agents | `#/settings` `SettingsPage` `[data-page=opc-settings]`, `[data-region=opc-connections]`, `opc-standing-policies`, `opc-close-background`, `[data-region=opc-settings-advanced]` | Honest connection table + retractable don't-ask-this-week + CloseBackgroundDialog (P12-T08); Model Connections write path through SecretStore (`P13-T08/D01`), notification / recovery groups, diagnostics, hidden nine-state `state-lab` (`P13-T08/D02`) → until then no fake Connect, no `/providers` detour claimed as done | `P12-T08` (done); `P13-T08` | `pass` |
| `M-CHAT-CANVAS` Conversation + canvas contract | create / members / test / joint default Assistant; live Project = project group; all scenes' right column | [scope §3.1](../product/personal-2.0-scope.md), [web-ui-design §6](../product/web-ui-design.md) → [02](../../../clients/docs/design/opc-2.0/02-information-architecture-and-app-shell.md), [05 group](../../../clients/docs/design/opc-2.0/05-team-roles-employees-and-conversations.md), [10](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md) | Rail is always the third column: header with identity (个人助手 / 项目群) + kicker, participants chips inside a Project, thread (owner / assistant / system rows, proposal cards 确认，写回画布 / 不用), composer (Enter sends, Shift+Enter newline), `@` inserts only into the unsent draft; canvas edit → Enter → confirm dialog → owner message → assistant proposes → owner confirms in chat → canvas applies; no Approve, no overlay "open conversation"; narrow canvas scrolls horizontally | `AssistantRail` + `RailCanvasWrite` `[data-region=opc-rail-write]`, `opc-rail-review` (dialog), `opc-rail-written`, `opc-rail-preview-announce`, `opc-rail-write-error`, `opc-rail-hitl` | `assistant.turn` then owner `draft.apply` (P11-T06 / P12-T09; candidate-only); real Pi inference `P13-T03/D01`; group chat `@manager` / `@member` routing `P13-T06/D01` → until then Assistant identity only, no Approve | `P12-T09` (done); `P13-T03`, `P13-T06`; rail visibility during create: drift §13-i (no owning card) | `partial` |
| `M-STATE` State grammar / State Lab | `state-lab` (`SurfaceKey` × `StateKey`) | [web-ui-design §8](../product/web-ui-design.md), [user-journeys](../product/user-journeys.md) states → [09](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md); v9 `state-lab` | Header + two selects (表面 / 状态); `StateBanner` (tone tag, surface label, message, 你还剩什么 / 你可以做什么 / 这一屏怎么露) followed by the **real** surface rendered in that state; nine × nine real layouts; not L1, not a "Designed" matrix; unknown ≠ 0, not success, no blind retry | Settings → Advanced `[data-region=opc-settings-advanced]` (hidden by default) → `P13-T08/D02` `state-lab` mounting the real `views/opc/*` components with the spec §9 patterns (`states.tsx`, `StateDot`, `HonestyNote`, `ReceiptLine`) | Client display states over daemon reads (`LOAD_STATE_CATEGORY`); no authority | `P13-T08/D02` (lab); `P13-T12/D02` (cells) | `fail` |
| `M-X` Parked connector | **no** v9 scene | [user-journeys §10](../product/user-journeys.md), [scope §3.6](../product/personal-2.0-scope.md) → [12](../../../clients/docs/design/opc-2.0/12-scenario-and-heuristic-review.md) parked | Not drawn in current chrome; not a P0 hero; not a Gate journey | none (no `/ui/` route); X connector walking skeleton exists only on management HTTP (P11-T14) | `P11-T14` (done, live X `not-run`) → honest absence: judged **pass only if no X chrome appears** in L1, Today, Settings or the rail | `P11-T14` (done) | `pass` |

Module count: 19 rows (16 single-scene or multi-scene modules + `M-SHELL`,
`M-CHAT-CANVAS`, `M-STATE` cross-cutting + `M-X` parked) = table A 19 / 19.

---

## 2. Grid A — nine states × nine surfaces (81 cells) / 九态 × 九表面

Definition of each cell: [spec §9.2](personal-2.0-opc-visual-ui-spec.md).
Host selectors: `today` `[data-page=opc-today]` · `create`
`[data-page=opc-create-wizard]` · `projects` `[data-page=opc-projects]` ·
`members` `[data-page=opc-project-members]` · `runs`
`[data-page=opc-project-runs]` · `outputs` `[data-page=opc-project-outputs]` ·
`hitl` `[data-region=opc-hitl-actions]` · `knowledge` `[data-page=opc-knowledge]`
· `settings` `[data-page=opc-settings]`. States are driven through the hidden
`state-lab` (Settings → Advanced, `P13-T08/D02`) or through real daemon
conditions; a cell that cannot be driven on the pinned revision is `not-run`
with the reason.

| Surface \ State | loading | empty | working | error | success | partial | blocked | unknown | offline |
|---|---|---|---|---|---|---|---|---|---|
| `today` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `create` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `projects` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `members` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `runs` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `outputs` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `hitl` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `knowledge` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |
| `settings` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` | `fail` |

Per-cell pass conditions (all must hold): the real component renders (no
static mock); the pattern of spec §9.1 is used (dot shape + verbatim word +
retained facts + one next action); `unknown` shows no `0`, no success word, no
retry control; `offline` blocks external approve / send / import / handoff and
labels facts stale with a time; `empty` has exactly one primary; `error` has
`role="alert"` and states whether retry is safe; `working` says working ≠
completion and shows Stop only when backed; no fake Create / Activate /
Approve / Connect / Install / Publish anywhere in the cell.

---

## 3. Grid B — keyboard reachability and focus restoration / 键盘与焦点

Definition: [spec §7](personal-2.0-opc-visual-ui-spec.md). Checks per surface:
**K1** route change moves focus to `#main` and the space title is the first
heading; **K2** Tab order = visual order through strip → nav (+ submenu) →
main → rail with no trap and no hidden stop; **K3** every actionable control
is reachable and shows the 2 px ring at ≥ 3:1 in the current theme; **K4**
widget pattern keys work (tablist arrows, listbox arrows, segmented group,
dialog Esc + focus restore, `<details>` toggle); **K5** after an error the
first invalid field is focused and the alert is announced; **K6** list
filter / selection / scroll survive a refresh, and a dropped row drops its
selection. Shell-level: **S1** skip link, **S2** ⌘K palette open / Esc restore,
**S3** rail composer Enter / Shift+Enter / `@` suggestions keyboard-only.

| Surface | K1 | K2 | K3 | K4 | K5 | K6 |
|---|---|---|---|---|---|---|
| `today` | `fail` | `partial` | `pass` | `partial` | `not-run` | `not-run` |
| `create` (wizard tablist, step dots, confirm dialogs) | `fail` | `partial` | `pass` | `partial` | `not-run` | `not-run` |
| `projects` | `fail` | `partial` | `pass` | `partial` | `not-run` | `not-run` |
| `members` (listbox, eight-tab tablist) | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` |
| `runs` (process axis, disclosure) | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` |
| `outputs` (listbox) | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` |
| `hitl` (checkbox, four actions, disabled reasons) | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` | `not-run` |
| `knowledge` (segmented tabs, filters, form) | `fail` | `partial` | `pass` | `partial` | `not-run` | `not-run` |
| `settings` (select, password, disclosure) | `fail` | `partial` | `pass` | `partial` | `not-run` | `not-run` |
| shell | S1 `pass` | S2 `pass` | S3 `not-run` | — | — | — |

---

## 4. Grid C — 200 % zoom and narrow three-column scroll / 200% 与窄窗

Definition: [spec §6.2 / §6.4](personal-2.0-opc-visual-ui-spec.md). Viewports
(CSS px after zoom): **V1** 1440 × 900 @ 100 % (reference); **V2** 1440 × 900 @
200 % (= 720 × 450 CSS px); **V3** 1100 × 800 @ 100 % (shell minimum); **V4**
960 × 800 @ 100 % (below minimum). Pass = three columns locked (side / main /
rail visible, none stacked, no drawer / sheet / overlay), page scrolls
horizontally when narrower than the shell minimum, `overflow=0 clipped=0`
inside columns (text reflows, nothing overlapped), type and target floors
unchanged, focus ring not clipped, sticky actions do not cover a focused field
or error, dialogs fit with internal scroll.

| Surface | V1 1440@100% | V2 1440@200% | V3 1100@100% | V4 960@100% |
|---|---|---|---|---|
| `today` | `pass` | `fail` | `fail` | `fail` |
| `create` | `pass` | `fail` | `fail` | `fail` |
| `projects` | `pass` | `fail` | `fail` | `fail` |
| `members` | `not-run` | `not-run` | `not-run` | `not-run` |
| `runs` | `not-run` | `not-run` | `not-run` | `not-run` |
| `outputs` | `not-run` | `not-run` | `not-run` | `not-run` |
| `hitl` | `not-run` | `not-run` | `not-run` | `not-run` |
| `knowledge` | `pass` | `fail` | `fail` | `fail` |
| `settings` | `pass` | `fail` | `fail` | `fail` |

Known pre-condition (spec §13-a): the `app.css` at `main@a0465653` stacks the
columns at ≤ 1279 px, so V3 / V4 (and V2, whose CSS viewport is 720 px) are
expected to record `fail` until the no-stack rule is implemented. D02 records
the fact; it does not narrow the rule.

---

## 5. Grid D — light / dark / high-contrast host themes / 三种宿主主题

Definition: [spec §5](personal-2.0-opc-visual-ui-spec.md). Themes: **L**
`prefers-color-scheme: light`; **D** `prefers-color-scheme: dark`; **HC**
`prefers-contrast: more` (tints transparent, strong hairlines); **FC**
`forced-colors: active` (Windows High Contrast; system colours; recorded
`not-run` / `not_available` unless the registered host can force it). Pass =
zero text pairs under 4.5:1, zero non-text state / focus pairs under 3:1, state
legible by text + shape, no information carried by background alone, primary
button and focus ring visible.

| Surface | L | D | HC | FC |
|---|---|---|---|---|
| `today` | `pass` | `pass` | `pass` | `not-run` |
| `create` | `pass` | `pass` | `pass` | `not-run` |
| `projects` | `pass` | `pass` | `pass` | `not-run` |
| `members` | `not-run` | `not-run` | `not-run` | `not-run` |
| `runs` | `not-run` | `not-run` | `not-run` | `not-run` |
| `outputs` | `not-run` | `not-run` | `not-run` | `not-run` |
| `hitl` | `not-run` | `not-run` | `not-run` | `not-run` |
| `knowledge` | `pass` | `pass` | `pass` | `not-run` |
| `settings` | `pass` | `pass` | `pass` | `not-run` |
| shell (strip, nav, rail, palette, dialog) | `pass` | `pass` | `pass` | `not-run` |

---

## 6. Grid E — NVDA key paths / NVDA 关键路径

Definition: [spec §7.5](personal-2.0-opc-visual-ui-spec.md). NVDA on the
registered host browser against the pinned guest `/ui/` (implementation
evidence only; Windows native chrome = `P13-T13`). Each path is one row;
pass = every step is announced with role + name + state, no focus theft by
streaming messages, no announcement flood, disabled controls announce their
visible reason.

| Path | Steps | Judgement |
|---|---|---|
| N1 shell | Ctrl+Home → landmarks (navigation / main / complementary) → nav items with current state → skip link → `#main` | `not-run` |
| N2 `today` | space title → decision packet heading → four facts → primary → run counts (unknown read as the word) → overview rows | `not-run` |
| N3 `create` ① | title → step tablist (selected / confirmed state in name) → field label incl. Enter-to-notify text → live status region → 确认本项 / disabled reason | `not-run` |
| N4 `create` ③–⑤ | roster table headers → `<progress>` value → 当前初始化 live region → member tablist → gated buttons + reasons; ④ seat notices; ⑤ 验收 primary | `not-run` |
| N5 `projects` / 详情 / 运行 | Project panel facts → 打开 + text links; process axis list with current step; stage tag 要你授权 / 要你核对; disclosure summary | `not-run` |
| N6 `members` / 产出 | listbox options with selected state → empty detail message when nothing selected → eight tabs (order + names) → read-only 输入 | `not-run` |
| N7 `hitl` | 将做什么 → preview facts list → checkbox name (本周此类不再问 · 到期失效 · 设置里可收回) → 批准 / 改窄 / 拒绝 (+ 停 when executing) → stale / unknown alert text | `not-run` |
| N8 `knowledge` | segmented tabs → filters → result list with kind tags → import form fields → phase status line → Why table headers → 忘记这条 | `not-run` |
| N9 `settings` | template select → custom fields → password field ("does not echo") → 交接密钥 → status line → 收回跳过 → Advanced disclosure (collapsed) | `not-run` |
| N10 rail | conversation identity → participants → thread rows (author labels) → proposal card actions → composer label → `@` suggestion list | `not-run` |

---

## 7. D02 pin (filled by D02) / D02 钉住

| Field | Value |
|---|---|
| Exact `/ui/` revision (pushed SHA) | `c8691923cd3988f0ffee9123752e073480aea5e9` |
| Guest daemon environment | `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, bind `127.0.0.1:48786`, SSH tunnel) |
| Host browser + version (registered host) | Chrome 151.0.7922.174 on `DEV-WIN-GNU-01` (headless CDP; dump-dom) |
| NVDA version | not installed on `DEV-WIN-GNU-01` (`C:\\Program Files\\NVDA` absent) — Grid E `not-run` |
| Windows native chrome environment | `DEV-WINDOWS-NATIVE-OPC-01` — not provisioned; cells `not-run` until `P13-T13/D01` |
| D02 running report | [2026-09-05-personal-p13-t12-d02-report.md](../../../docs/checkpoints/2026-09-05-personal-p13-t12-d02-report.md) |

---

## 8. Drift observed at module level / 模块级漂移

Spec-level drift is [spec §13](personal-2.0-opc-visual-ui-spec.md) (a–m).
Module-level notes the reviewer needs while judging:

| Module | Drift | Rule applied |
|---|---|---|
| `M-SHELL` | current `app.css` stacks columns ≤ 1279 px; product / v9 require horizontal scroll | product docs win; Grid C records `fail` until fixed; no owning card yet |
| `M-CHAT-CANVAS`, `M-CREATE-*`, `M-TODAY-INCOMPLETE` | `/ui/` hides the rail on the wizard and creating-only Today; v9 hides it only on empty Home; scope §3.1 says create-ring chat defaults to the Assistant | recorded (spec §13-i); judge the module on what is present; the visibility question is not decided here |
| `M-MEMBER-CONFIG`, `M-SHELL` | label locale (zh-CN product terms vs `/ui/` English) | judge order + meaning, not language (spec §13-h) |
| `M-HITL` | v9 standalone scene vs `/ui/` canvas `?preview=` | Owner decision #4 (mapping) wins |
| `M-TODAY`, `M-LIVE-PROJECT` | v9 run counts / overview rows / occurrence ledger / openable outputs vs `/ui/` axis-only | owned by `P13-T05/D02`, `P13-T04/D02`; until they land the honest-state judgement applies |
| `M-STATE` | v9 nine `StateKey`s vs `/ui/` seven categories | mapped in spec §5.2; one system |
| `M-X` | no scene by design | pass = absence |
| `M-SHELL` / Grid B K1 | document first heading is brand `h1` "CognitiveOS Personal"; space title is `h2` inside `#main` | spec §7 K1 recorded `fail`; not an IA change |
| `M-STATE` | State Lab cells are shared `LoadingState` / `EmptyState` widgets, not real `TodayPage` / wizard layouts | spec §9.2; D02 `fail` all 81; no TSX patch in this lease |

---

## 9. Summary counters (D02 filled) / 计数

| Grid | Cells | pass | fail | partial | not-run | not_available |
|---|---:|---:|---:|---:|---:|---:|
| §1 modules | 19 | 7 | 1 | 1 | 10 | 0 |
| §2 nine × nine | 81 | 0 | 81 | 0 | 0 | 0 |
| §3 keyboard / focus | 57 | 7 | 5 | 10 | 35 | 0 |
| §4 200 % / narrow | 36 | 5 | 15 | 0 | 16 | 0 |
| §5 themes | 40 | 18 | 0 | 0 | 22 | 0 |
| §6 NVDA paths | 10 | 0 | 0 | 0 | 10 | 0 |

End of checklist. Informative only. Canvas v9 ≠ product. D02 recorded these judgements on `c8691923cd3988f0ffee9123752e073480aea5e9` (`DEV-LINUX-NATIVE-01` guest `/ui/` + host Chrome). Remaining `not-run` is missing NVDA, missing Windows High Contrast / native chrome, or a disposable runtime with no live Project / later wizard step. `not-run` is never pass. Claim ceiling `hypothesis`.
