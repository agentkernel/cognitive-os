# Personal 2.0 OPC v9 战役审查台账

- Status: **owner 已批准 v9**（2026-08-30）— campaign notes, not a second product baseline
- Date started: 2026-08-29; owner approval: 2026-08-30
- Current chrome:
  [`personal-20-opc-e2e-optimized-v9.canvas.tsx`](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Prior approved baseline (do not overwrite):
  [`personal-20-opc-e2e-optimized-v8.canvas.tsx`](personal-20-opc-e2e-optimized-v8.canvas.tsx)
- Cursor 可打开路径：`C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-opc-e2e-optimized-v9.canvas.tsx`
- Owner instruction: record module findings here and keep updating after each review
- This file may now record product-doc alignment. Still not: architecture rewrite, handbook generation, Control Plane implementation, `P*-T*` claim, `PROGRESS.md` task status, Git commit/PR
- Evidence: Canvas runtime/render, NVDA, host-theme contrast, 200% layout remain `not-run`. **Owner 批准 v9 不是**可用性 / a11y / backend / Gate / release / qualification / acceptance 证据。

Authority when this log conflicts with product docs: product docs + approved v9 + [00-maintenance-index.md](00-maintenance-index.md) win. This file only accumulates prototype findings.

---

## 战役台账（滚动）

- 基线：v8（不覆盖；上一版已批准基线）
- 已批准当前 chrome：v9（2026-08-30）
- 已关闭模块：
  - `M-SHELL` — L1 与三栏大方向对；知识/设置的「打开对话」破坏三栏锁；空 Home 藏聊天是唯一已授权例外。
  - `M-EMPTY` — 空 Home 只创建、无示范项目、聊天隐藏，产品路径遵守；缺 loading/error/unknown/offline 真版式；空生命周期下创建入口重复。
  - `M-CREATE-1` — 逐项确认、未上线、留草稿、缺模型去设置均遵守；总预览不是清单最后一项；Skill/MCP 当步骤标题；State Lab `create` 未挂真版式。
  - `M-CREATE-2` — 一轴一环、process-before-members 文案遵守；可跳环确认；缺口不在轴上；末环一键跳进 ③。
  - `M-CREATE-3` — 当前条标题+进度遵守；名单表可跳人、塞进全员编辑；无拒绝加入；轴示意图过重。
  - `M-CREATE-4` — 就位门与未知不能通过的文案遵守；「通过，下一环」实际跳进 ⑤；结果样品切换器可作弊；测试态不跟环走。
  - `M-CREATE-5` — aha CTA「验收，进入今日」位置正确；未知不能验收；全流程位置写死前四环已完成；无失败回退按钮；无离线门。
  - `M-TODAY-INCOMPLETE` — 中栏无日常决策包/运行概览/四泳道，产品路径遵守；「继续」无视 `createGate` 一律回 ①；未完成创建时项目 L2 仍露出上线后四去向。
  - `M-TODAY` — 默认三块大方向对（决策包 + 概览 + 助手），无四泳道；概览用流程轴冒充「点项目」；阻塞计数不可点；无待拍板时仍画满包，State Lab 空态掉进空 Home。
  - `M-PROJECTS` — 一行一个「打开」+ 文字链遵守，无四个「查看」；副本文案诚实但无总预览门；live 列表「创建项目」开看不见的 ①。
  - `M-LIVE-PROJECT` — 运行点轴换环、末环才验收、产出先选后看、无 CEO 轨遵守；L2 缺「详情」；详情可编章程；末环验收跳过可打开成果直达 Today。
  - `M-ADD-MEMBER` — 现有班子用当前项目名单、职责在前、无 Install 遵守；无拒绝；无模型不能加入（产品允许 pending）；加入后无执行方式入口；岗位表单预填像示范岗。
  - `M-MEMBER-CONFIG` — 先选后看、未选空态、换项目清空、八标签与输入只读/输出可编遵守；模型不在详情头；槽位副文把 Loop/Skill/MCP 抬到默认 chrome。
  - `M-HITL` — 画布批准/改窄/拒绝、聊天只有链接、过期不能批、无永久不再问均遵守；改窄后仍可批旧预览；empty 仍画一份可批包；新鲜度分段器在主路径。
  - `M-KNOWLEDGE` — 无项目锁定、Why this fragment、失败留原件、记忆可忘、无 Install 遵守；导入主路径用四按钮点成功；知识默认藏右栏；离线未禁用导入。
  - `M-SETTINGS` — 模板+自定义、密钥交接后清空、无账单/引擎店/Inbox、时间盒可收回遵守；主路径交接必成功；失败不点名；设置默认藏右栏。
  - `M-CHAT-CANVAS` — 聊天无 Approve、@ 只进草稿、创建环个人助手、窄窗 min-width 横滑遵守；已上线详情/成员/产出被 `isSetupChat` 收成个人助手；知识/设置「打开对话」破坏三栏。
  - `M-STATE` — State Lab 九键与部分 native 映射遵守；create 全占位；today 缺 loading/error/unknown/offline 真版式；hitl empty 仍可批；runtime/NVDA/对比度/200% not-run。
- 进行中：无
- 下一模块：无（队列 1–18 已 closed；M-X 跳过）
- Must-fix 累积：（跨模块去重）
  1. 知识 / 设置不得用「打开对话」把右栏收成开关；只允许 `empty-home` 藏聊天。
  2. 知识 L1 不得 `aria-disabled` 却仍可点。
  3. 空生命周期只保留一条主创建路径（画布主按钮）。项目 L2 不要再挂一个并列「创建项目」。
  4. ① 清单最后一项必须是「总预览」；「执行方式」放在总预览之前。进入 ② 的主按钮只能出现在已确认总预览之后，且写明项目仍未上线。
  5. ② 必须顺序确认：未确认的后环不可点进工作面，不可标已确认。
  6. ② 轴节点必须能显示缺口/未知，且该态不能标「已确认」。提供「本环留缺口」；删掉无控件的「拒绝则留在这一环」。
  7. ② 末环「确认这一环」与「确认总目标 + 项目触发」分开；总确认之后才进入 ③，禁止同一点击跳进成员初始化。
  8. ③ 名单选择必须与圆点同一顺序门；禁止用表格点进未轮到的人。
  9. ③ 当前工作面只留进度 + 当前项标题 + 生成/确认/拒绝；全员职责/交出编辑下沉到配置页或当前人，禁止五行全员表当默认 chrome。
  10. ③ 必须有「拒绝加入」= 未加入（不是已就位）。
  11. ④ 「通过，下一环」只进入下一环；末环通过后才进入 ⑤。禁止任一环通过就跳联调。
  12. ④ 测试结果按环保存；换环不得沿用上一环的 pass。未知/未测不能通过。
  13. ④ 离线禁用「开始测」。失败提供「回 ② 改这一环」真按钮。
  14. ⑤ 失败必须能回 ④/②/③（点名环节的真按钮）。离线不能开始联调。全流程位置不得写死「前四环已完成」。
  15. 未完成创建的「继续」必须回到离开时的创建段（按 `createGate`：①–⑤），禁止一律 `create-init`。画面写明停在第几段，不要做成日常决策包卡片。
  16. 生命周期 `creating` 时项目 L2 不露出 详情/成员/运行/产出；项目面只留这一份草稿 + 继续创建。
  17. 日常 Today 运行概览按**已上线项目行**点进（状态 / 今日完成次数 / 当前环节 / 时长）；禁止把 ② 式全流程轴当 Today 默认 chrome。阻塞计数可点进该项目运行。
  18. 已上线但无待拍板：收起决策包、保留运行概览。禁止把 live Today 空包画成 `empty-home`。
  19. 副本落地必须有「总预览」才能上线；禁止只亮 banner。已上线列表的「创建项目」必须变成列表里的未激活草稿行（或拿掉该钮、第二份工作只走复制），禁止在 live 生命周期开一个列表看不见的 ①。
  20. 已上线 L2 必须是 **详情 / 成员 / 运行 / 产出**（列表留在 L1「项目」）。禁止把详情藏进「项目列表」高亮。
  21. 详情只读流程轴 + 去向链接；禁止把 live 详情做成名称/目标/周期表单。
  22. 末环「验收，回今日」必须先打开成果并给出核对态；禁止 `onClose` 直接 `today`。验收不得绕过该环拍板/核对。
  23. 加人必须有「拒绝」= 未加入。无模型允许「确认加入」为 pending 并去设置，禁止把未选模型做成不能加入。加入后主路径披露执行方式（打开配置页），禁止只写「回画布查看配置」而无控件。
  24. 成员配置：身份（模型、就位、负责环节）只出现在详情头，不要把模型下拉做成标签页上方的第二张表单。槽位默认只显示业务名（工作说明/能力包/外部连接）；Loop / Skill / MCP 进一层 `details`。
  25. HITL：改窄后旧预览立即过期，批准禁用直到新预览。无待批（empty）不得画可批准预览。unknown 写「说不清」，不要只标过期。
  27. 已上线项目工作面（详情 / 成员 / 运行 / 产出 / HITL）右栏必须是项目群；`isSetupChat` 不得把它们收成个人助手。`@` 写入未发送草稿。创建环仍用个人助手。
- Major 累积：
  1. 壳层 Context header 永远写「在线」，缺 offline / stale / unknown。
  2. 「今日」角标「1」在 live Today 上无条件出现，像未读 KPI。
  3. `empty-home` / State Lab `today × loading|error|unknown|offline` 没有该表面真版式（掉进占位段）。
  4. 空 Home 居中 28px 标题偏落地页；v9 改为与后续 Today 同密度的左对齐空态，仍只一个 CTA。
  5. ① 步骤标题用业务语言；Skill / MCP 下沉一层披露，确认项本身保留。
  6. ① 确认 brief 后应有 researching/partial/coverage，或明确标「原型未跑调研 · Requires-backend」，不要看起来像已经调研完。
  7. State Lab `create` 必须渲染各创建真表面，禁止占位段冒充真版式。
  8. ② 总目标 + 总周期画在轴头；用岗位名；offline 真态。
  9. ③ 只读流程轴改为「回 ②」链接；当前项 hint 不写 Loop/MCP。
  10. ④ 主路径不要用「测试结果样品」分段器直接点达标；样品切换放进 State Lab。通过态要有可打开的结果样品控件。
  11. ⑤ 同样：主路径不要用联合结果分段器点「核对通过」；总成果要有可打开样品；验收仍只在 pass。
  12. 未完成创建的空态不要复用 live Today 的 `.decision-packet` 版式；做成与空 Home 同密度的工作面 + 一个主按钮。
  13. 未完成创建 × offline/unknown：仍只继续创建，标过时/说不清；禁止掉进 live 决策包。State Lab `today × blocked` 不要只映射未完成创建（上线后 blocked 另有语义，留给 `M-STATE`）。
  14. 周期切换要带动概览里周期相关数字，不能只改「已完整执行」一格；offline 标过时；loading 时决策包仍可点。
  15. 「以后再说」保持决策包在今日，并给停留回执（不要无 handler 的死按钮）。
  16. 项目列表改成一行一项目的紧凑表；项目详情页也提供「复制为草稿」。
  17. 已上线 详情/成员/产出 右栏用项目群，不要创建期个人助手（细项 `M-CHAT-CANVAS`）。
  18. 产出页样品分段器进 State Lab；运行页补 empty/unknown/offline 真版式。
  19. 加人空态：岗位名/职责空白 +「这个岗位还不存在」；聊天建议岗位，不要预填全局示范岗。离线可改职责，禁用「联网搜岗位方案」（若无该能力则不要画搜方案按钮）。
  20. 成员配置页与成员管理右侧共用同一 `MemberConfigPanel`；创建 ③ 当前条仍禁止复制整页。State Lab `members` 的 loading/error/unknown/offline 要改版式，不能九态同一张名单。
  21. HITL 新鲜/过期分段器进 State Lab。主路径默认新鲜预览。拒绝后回到该环并留回执，不要无声 `onBack`。
  22. 知识导入「演示下一状态」四按钮进 State Lab。离线禁用开始导入、只读上次索引；unknown 不是 0 条资料。
  23. 设置：连接失败必须点名原因（State Lab `error` 已切 failed，文案仍泛）。离线不能交接。unknown 费用/额度不写 0。主路径成功仍标 Requires-backend，不要看起来像真连上 Provider。
- 明确不改 / Parked 2.1：Team / Inbox 不做一级；可见 CEO 六步轨不进 chrome；窄窗横滑 ≠ native mobile；pairing / E2E relay / 引擎商店 / 成员级预算不画；无示范/模板项目；不把 Skip 做成进日常 Today；不把成员就位 UI 画进 ②；③ 不把完整配方复制进当前条；③ 默认第一人是创建顺序，不是上线后成员页的「不默认第一人」；④⑤ 不画过程/引擎 chrome；⑤ 之前禁止日常 Today；workshop members-then-process 只记录不改产品文档。未完成 Today 不补决策包、运行概览、周期切换。不把 03 章四泳道画回 v9。不把四个「查看」画回列表。03 章搜索/筛选不是本战役 mutex。不先装 MCP。Loop 不是默认 chrome 标签名（「周期与触发」可留）。不编造成员私有输入字段。不把 HITL 批准放进聊天。不画永久 Don’t ask again。不引入 X/Twitter。无项目的云导入与 native mobile 库是 2.1。消费订阅、成员级预算、引擎商店不进 Settings chrome。
- 语料漂移：02 / 03 / web-ui-design / opc-product-model 已随 v9 批准对齐（见维护索引）。workshop Q&A 仍是历史 members-then-process 快照。架构 Employee 用语与 handbook 生成页未改。
- 生成状态：**owner 已批准 v9**（2026-08-30）
- 产品文档：已把当前 chrome 切到 v9，并修正与 v9 冲突的产品/语料锚点。未跑 handbook 生成器（source-map 映射 `personal/docs/product/**` → 生成页；待后续实现/文档同步任务）。
- 未做：Control Plane 实现、`clients/pc/web/`、daemon、git commit/PR、`P*-T*` 领取、PROGRESS 任务状态、Gate/可用性/a11y 证据。
- 未新开 lease：沿用已有 `lease/personal/DOC-PERSONAL-2.0-OPC-REFRAME/product-prototype-docs`；heartbeat 2026-08-30；在同一行补了 chrome 指针文件 `linux-1.0-scope.md` / `resource-manager-design.md` / `cognitive-resource-model.md` / `provider-control-plane.md`。未改 Layer 1 任务剩余、未开实现分支。
- 下一步（等 owner，不是自动实现）：开实现任务 / 同步 handbook / 驳回某模块再出 v10。

---

## 怎么用

每个已关闭 ID 保留：审查表、打分、建议分级、明确不改。v9 生成时只吃 Must-fix + Major；Minor 可选。不要为了漂亮重画无关表面。

Impeccable `onboard` 对本仓库是 **DEGRADED**（scripts 未 vendoring）。下列 onboard 建议已按产品真相过滤：禁止示范项目、禁止 Skip 进日常 Today、禁止欢迎仪式/时间估计。

---

## M-SHELL（closed）

| 项 | 内容 |
|---|---|
| ID | `M-SHELL` |
| 用户任务 | 用 Today / Projects / Knowledge + 底栏 Settings 到达目标；三栏锁定 |
| v8 场景 | 全部场景共用左栏；`empty-home` 隐藏右栏聊天 |
| 产品真相是否被原型遵守 | 漂移。遵守：scope §3.1 L1 锚点、Settings 钉底、无 Team/Inbox、无可见 CEO 轨、窄窗横滑、空 Home 藏聊天。漂移：知识/设置 header「打开对话」收起第三列。 |
| 上手障碍 | 5 秒能看见三锚点。知识灰态可点。知识/设置要先开对话才有助手。 |
| 布局问题 | 默认三栏正确。`chat-hidden` 在知识/设置变成两列。L2「项目列表」文案留给 `M-LIVE-PROJECT`。 |
| 友好性（含九态） | empty 按授权藏聊天。offline 壳层 Tag 写死「在线」。 |
| 前瞻性 | 默认 chrome 是目标导航。把对话做成开关是把同伴后置了。 |
| Honesty | 设计路由不是 API。无 Connect/Install/聊天 Approve。`aria-disabled` 可点是假禁用。 |
| 建议 | 见下 |
| 明确不改 | 相邻创建阶段；`PROJECT_SUBNAV` 详情文案；02 章 CEO 轨语料；v8 文件 |

### 分级

- **Must-fix**
  - 知识/设置默认保持第三列；删除「打开对话」。空 Home 继续藏聊天。
  - 知识锁定：可进说明页，或真正 `disabled` + 焦点解释。禁止 `aria-disabled` 仍导航。
- **Major**
  - Context header 跟 host 态（offline = 过时，不是成功）。
  - 今日角标只在真有决策包时出现。
- **Minor**
  - Today 的 `locationLabel`「个人」改为「今日」。
- **Parked 2.1**
  - 横滑不改抽屉；不画 native mobile / pairing / relay。

### 打分（plan-design-review，仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 6 |
| 旅程 | 8 |
| AI-slop | 8 |
| 设计系统 | 8 |
| 响应/a11y | 6 |
| 未决 | 7 |

未决：右栏常驻 vs 可关；锁定知识禁点 vs 进说明；壳层是否跟 State Lab 的 host 态。

源码锚点：`primary-nav`、`chatHidden`、`setChatOpen`、「打开对话」、`aria-disabled={!knowledgeOk}`。

---

## M-EMPTY（closed）

| 项 | 内容 |
|---|---|
| ID | `M-EMPTY` |
| 用户任务 | 无 Project 时只创建；不进日常 Today |
| v8 场景 | `empty-home`；State Lab `today × empty` |
| 产品真相是否被原型遵守 | **遵守**（布局密度有漂移）。[user-journeys §1](../../../../personal/docs/product/user-journeys.md)：空 Home 中画布只有创建，右栏隐藏，知识锁定，无默认/示范项目。v8：`EmptyHomeScene` 单 CTA；`listedProjects("empty")` 返回 `[]`；`chatHidden` 含 `empty-home`；知识 `locked` 文案诚实。不进 `TodayLiveScene`。缺模型引导属于创建页聊天（`M-CREATE-1`），空 Home 不静默绑模型 — 正确。 |
| 上手障碍 | 5 秒看见「创建项目」。5 分钟能进 ①。并列入口：空 Home 主按钮、项目列表空态主按钮、空生命周期项目 L2「创建项目」。Onboard「模板/示范/Skip」与产品冲突，**不采用**。 |
| 布局问题 | `.empty-home` `place-items: center` + `h2` 28px，像营销首屏，不像后续 Today 的 dense workbench。项目空态已是左对齐 header+CTA，两套空态语法。 |
| 友好性（含九态） | empty / first-run：有原因 + 一个主行动。State Lab `today × empty` 复用 `EmptyHomeScene`（好）。`today × loading\|error\|unknown\|offline` 落到「该状态已按…真实版式渲染」占位，**不是**空 Home 真版式。`today × blocked` 映射到未完成创建（属 `M-TODAY-INCOMPLETE`，本模块不改）。working/success/partial 映射 live Today，空生命周期不应出现。 |
| 前瞻性 | 配置/引擎未出现。没有把日常决策包画进未创建。正确把 aha 留在 ⑤。 |
| Honesty | 诚实。无 KPI 墙、无示范项目、无假 Connect。原型样品数据在其他 scene 存在，空生命周期不展示。 |
| 建议 | 见下 |
| 明确不改 | 不引入示范/模板项目；不 Skip 进日常 Today；不在空 Home 打开聊天（产品授权隐藏）；知识锁定细节留给 `M-KNOWLEDGE`；创建 ① 表单留给 `M-CREATE-1` |

### 分级

- **Must-fix**
  - 空生命周期只留一条主创建 CTA（空 Home / 项目空态共用同一动作）。去掉项目 L2 额外「创建项目」。
- **Major**
  - 为空 Home 补 State Lab 真版式：`loading`（稳定壳 +「正在看有没有项目」）、`error`（列表失败，保留创建）、`unknown`（说不清有没有项目，不是 0 也不是成功）、`offline`（若从未有项目仍是创建；若有上次事实则标过时，**仍禁止**日常决策包）。
  - 空 Home 改成左对齐、与 Today 同密度；保留单 CTA，去掉居中大标题落地页感。
- **Minor**
  - 副文收成一句（现在同时讲业务语言、聊天稍后、知识锁定）。
  - `h2` 与按钮不要同文「创建项目」；标题用「还没有项目」，按钮用「创建项目」。
- **Parked 2.1 / 明确不采用**
  - Impeccable onboard 的模板、Skip、欢迎仪式、时间估计、社交登录。
  - 空 Home 不预绑模型。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 8 |
| 状态 | 5 |
| 旅程 | 8 |
| AI-slop | 7 |
| 设计系统 | 7 |
| 响应/a11y | 7 |
| 未决 | 7 |

未决：空 Home 与项目空态是否视觉合并成同一组件（建议合并，避免两套语法）。

源码锚点：`EmptyHomeScene`；`listedProjects` empty → `[]`；State Lab `today × empty`；`ProjectsScene` `lifecycle === "empty"`；空生命周期 `projects-submenu` 内「创建项目」。

### Click path（源码，非浏览器验收）

1. 默认 scene `empty-home`：中栏只有创建；右栏无；知识带「锁」。
2. 主按钮 → `lifecycle=creating`，`create-init`。
3. 点「项目」→ 项目空态再一个「创建项目」。
4. 点「知识」→「知识已锁定」（不进资料库）。
5. 点「设置」可达（连模型，不在空 Home 完成）。
6. 无决策包、无运行概览、无示范行。

### 只留三件

1. 一句话：还没有项目。
2. 一个按钮：创建项目。
3. 锁定知识 + 可去设置（次要）。其余下沉到 ①。

没有把 Install、引擎、Team/Inbox、CEO 轨、四泳道、四个「查看」、聊天 Approve、日常决策包画回空 Home。

---

## M-CREATE-1（closed）

| 项 | 内容 |
|---|---|
| ID | `M-CREATE-1` |
| 用户任务 | 用业务语言立项；逐项确认产出/周期/权限/总预览 |
| v8 场景 | `create-init` |
| 产品真相是否被原型遵守 | **漂移**。[user-journeys §1.3](../../../../personal/docs/product/user-journeys.md) / [product-design P0-1](../../../../personal/docs/product/product-design.md)：逐项确认、总预览前未上线、离开留草稿、无模型则聊天指路 Settings、不静默绑、样品非 daemon、无 Harness 字样。v8 这些都做到了。漂移：`WIZARD_STEPS` 把 `method`（执行方式）放在 `preview`（总预览）**之后**；最后一屏主按钮却写「总预览后进入 ②」。产品清单以总预览为立项门。Skill/MCP 直接做步骤标题，与 scope「能力术语下一层」及战役简洁目标冲突（确认项本身仍应保留）。 |
| 上手障碍 | 5 秒看见「① 逐项确认」和当前项名。16 步无标签圆点难扫。5 分钟很难走完整表。缺模型时画布主按钮去设置、聊天说明不收密钥 — 好。 |
| 布局问题 | 单卡 wizard 是确认面，不是「卡片在中间=已上线」（状态写「项目仍未上线」）。圆点只有点没有字。不要把 ① 收成一页长表（会破坏逐项）。 |
| 友好性（含九态） | empty：brief 待填/预填样品。blocked：无模型不能确认。stale：改 brief 后后续项标过时。success：确认回执仍说未上线。**缺** researching（brief 确认后直接灌 `item.detail` 样品）。offline/error/unknown 在 State Lab `create` 上是占位段，未挂 `CreateInitScene`。 |
| 前瞻性 | 执行方式有业务定义、无引擎名。Skill/MCP 标题把配置层抬到默认 chrome。无 Install 按钮。 |
| Honesty | `Requires-backend` Gap 在。无假 Connect/Install。聊天「确认，写回画布」是产品规定的编辑链，不是 HITL Approve。调研被样品填满，看起来像已经调研完 — 诚实缺口。 |
| 建议 | 见下 |
| 明确不改 | 不改 ② 流程轴；不把 ① 后的卡片画成已上线成功；不删逐项确认；不在聊天放 HITL 批准；不引入 X；Harness/Loop 不进默认 chrome |

### 分级

- **Must-fix**
  - 清单顺序：… → 执行方式 → **总预览（最后）**。进入 ② 仅在总预览已确认之后；按钮文案不要在「执行方式」步冒充总预览。
- **Major**
  - 步骤标题业务化（例如「本项目要用的能力」），Skill / MCP 放进详情/一层披露。
  - brief 确认后增加 researching/partial 真态，或横幅：「原型未跑调研，下列是本地样品 · Requires-backend」。
  - State Lab `create` 渲染 `CreateInitScene`（loading/error/offline/unknown/stale）。
  - 步骤指示可读：保留「n / N · 当前项」，圆点补短标签或弱化成进度而非唯一导航。
- **Minor**
  - 默认 brief 样品加一句「请改成你自己的事」。
  - Skill 详情少用「安装」一词（已有「无假 Install」）。
- **Parked 2.1 / 不采用**
  - 不把 16 项压成一页营销表单。
  - 不在 ① 结束时庆祝上线。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 6 |
| 状态 | 6 |
| 旅程 | 7 |
| AI-slop | 8 |
| 设计系统 | 7 |
| 响应/a11y | 6 |
| 未决 | 7 |

未决：能力类确认是继续 16 步逐项，还是「能力与权限」一卡内再逐条确认（仍 item-by-item，只是导航更短）。

### Click path（源码）

1. 无模型：Notice + 主按钮「去设置」；聊天「请去设置」；无「下一项」。
2. 有模型：确认业务描述 → 后续项填入样品 detail → 须逐项「确认本项」才能下一项。
3. 改 brief → 后续标过时，不能当已就绪。
4. 离开 → `today-incomplete`。
5. 全确认后「总预览后进入 ②」（当前发生在 **执行方式** 步）。
6. 全程 Gap：创建/调研/总预览权威需 daemon。

### 只留三件

1. 你写业务描述。
2. 逐项确认（产出、周期、权限、执行方式）。
3. 总预览（未上线）+ 留草稿 / 去设置。

没有把 Install 商店、引擎名、Team/Inbox、CEO 轨、四泳道、聊天 HITL Approve 画进 ①。

---

## M-CREATE-2（closed）

| 项 | 内容 |
|---|---|
| ID | `M-CREATE-2` |
| 用户任务 | 一条流程轴、一环一环确认；未知缺口留在轴上 |
| v8 场景 | `create-process` |
| 产品真相是否被原型遵守 | **漂移**。[user-journeys §1.4](../../../../personal/docs/product/user-journeys.md)：一轴、一次一环、「确认这一环」、末环后再确认总目标+项目触发、缺口留在轴上不标就绪、process-before-members。v8 标题与「轴确认后再按流程创建成员」遵守，**没有**把就位 UI 画进 ②。漂移：轴上任意环可点、任意环可标已确认；节点只有「待确认/已确认」，缺口只写在草稿里；末环按钮「确认总目标，进入成员初始化」同一点击 `onMembers()`。文案「拒绝则留在这一环」无拒绝控件。 |
| 上手障碍 | 5 秒能看见轴和「确认这一环」。5 分钟能确认一环。跳环会让人以为后环已就绪。 |
| 布局问题 | 总目标/周期只在标题段，不在轴上。工作面输入/执行方式/权限后果三字段清楚。 |
| 友好性（含九态） | empty/working：待确认环。无 unknown 轴标记。无 offline 真态（仅 Gap 文字）。blocked 未表达。success 被做成直接进 ③。 |
| 前瞻性 | 默认是流程合同，Skill/MCP 只在执行方式样品句里。无引擎名、无 Install。意向岗位用人名，略像已有成员。 |
| Honesty | 顺序门是假的。Gap 有 Requires-backend。无假 Install。process-before-members 文案诚实。 |
| 建议 | 见下 |
| 明确不改 | 不把 ③ 就位画进 ②；不改 ④ 测试门；workshop 顺序只记录；不把四泳道/CEO 轨画进轴 |

### 分级

- **Must-fix**
  - 未确认前环时，后环轴节点不可进入工作面、不可标已确认。
  - 轴节点增加缺口/未知；该态禁用「已确认」。用「本环留缺口」替换空「拒绝」文案。
  - 末环先「确认这一环」，再单独「确认总目标与项目触发」，然后才进 ③。禁止一键跳成员初始化。
- **Major**
  - 总目标 + 总周期放在轴头。
  - 意向岗位用岗位名，不用像已就位的人名。
  - offline：可改输入；联网补执行方式标过时/不可跑。
  - State Lab `create` 挂上 `CreateProcessScene`（含 unknown/offline）。
- **Minor**
  - 总确认后停一拍「轴已齐」，再给进入 ③。
- **Parked**
  - members-then-process 语料；不在 ② 做班子配置。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 5 |
| 旅程 | 6 |
| AI-slop | 8 |
| 设计系统 | 8 |
| 响应/a11y | 7 |
| 未决 | 7 |

未决：缺口是轴上第三态，还是确认按钮旁的显式「留缺口」。建议两者都要，轴上必须看得见。

### Click path（源码）

1. 五个轴按钮均可点，无顺序门。
2. 「确认这一环」无空/未知校验，直接 `confirmStage`。
3. 非末环自动 `setStageId(next)`。
4. 末环确认后立刻 `create-members`。
5. 无「留缺口」控件。右侧为个人助手（setup chat）。

### 只留三件

1. 轴（含总目标/周期 + 缺口标记）。
2. 当前一环的输入 / 执行方式 / 权限。
3. 「确认这一环」；全部确认后再确认总目标与触发。

没有把成员就位、Install、引擎、Team/Inbox、CEO 轨、聊天 Approve 画进 ②。

---

## M-CREATE-3 / M-MEMBERS-INIT（closed）

| 项 | 内容 |
|---|---|
| ID | `M-CREATE-3` |
| 用户任务 | 按已确认流程建班子，逐个就位；模型必选 |
| v8 场景 | `create-members` |
| 产品真相是否被原型遵守 | **漂移**。[journeys §1.5](../../../../personal/docs/product/user-journeys.md)：按流程建名单、逐人、模型必选、当前初始化=进度+当前项标题、全文在配置页、顺序就位、拒加入=未加入、缺模型=pending。v8：`当前初始化 · n/N · 名` + `init-current-title` 只用业务项名；缺模型 Notice；配置页入口；无 Install。漂移：岗位表可点任何人（圆点有顺序门、表格没有）；表内全员编辑职责/交出；无「拒绝加入」；只读流程轴重放每环入/出；当前项 hint 写出 `runtimeLabel`（含 loop / MCP）。 |
| 上手障碍 | 5 秒能懂「先建岗位再逐人」。一屏同时是轴、大表、当前条、进测试，找不到「只看当前人」。 |
| 布局问题 | 默认 chrome 过重。当前条本身符合「不要完整 recipe」。 |
| 友好性（含九态） | generating / 缺模型 / 待确认有。无拒绝。unknown 有标签函数但 ③ 主路径难触发。offline 仅 Gap。 |
| 前瞻性 | 业务标签在前（工作说明/能力包/外部连接）。hint 把 Loop/MCP 抬上来。无 Loop 页签。 |
| Honesty | 就位不是 daemon。无静默绑模型。`confirmRoster` 进测试时按「有模型」标 joined，因按钮有全员就位门，实际无害。 |
| 建议 | 见下 |
| 明确不改 | 不把完整配方塞进当前条；不审配置页八标签（`M-MEMBER-CONFIG`）；不改 ④；创建期默认第一人 ≠ 上线后未选空态 |

### 分级

- **Must-fix**
  - 表格选人与圆点同一顺序门。
  - 默认名单改成紧凑行（岗位、模型、就位）；职责/交出只在当前人或配置页。
  - 「拒绝加入」→ 未加入，不能变成已就位。
- **Major**
  - 流程轴改为回 ②，不重绘入/出。
  - 当前项只显示业务标题；hint 不出现 Loop/MCP。
  - State Lab 挂 `CreateMembersScene`（缺模型 / generating / 拒绝）。
- **Minor**
  - 全员就位后停一拍再进 ④。
- **Parked**
  - 成员不跨项目共享（产品约束，本屏无需新 chrome）。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 6 |
| 状态 | 6 |
| 旅程 | 7 |
| AI-slop | 8 |
| 设计系统 | 6 |
| 响应/a11y | 6 |
| 未决 | 7 |

未决：紧凑名单是左侧人列表还是顶上一行 chips。建议 chips + 当前条，与「一次一个人」一致。

### Click path（源码）

1. 「创建成员」→ 对话框 → 按流程生成名单。
2. 表内可改所有人职责/模型；点第 3 人可绕过圆点禁用。
3. 未选模型不能生成。生成时当前条只换标题。
4. 「查看完整配置」进配置页（本模块不改）。
5. 确认就位后自动 `setActiveMemberId(next)`。
6. 全员就位才「进入测试」。无拒绝。

### 只留三件

1. 紧凑名单 + 选模型。
2. 当前初始化（进度 + 当前项标题）。
3. 生成 / 确认就位 / 拒绝加入；完整配置一层之外。

没有把 Install、引擎商店、Team/Inbox、CEO 轨、聊天 Approve、④ 测试门画进 ③。

---

## M-CREATE-4 / M-TEST-STAGE（closed）

| 项 | 内容 |
|---|---|
| ID | `M-CREATE-4` |
| 用户任务 | 按环测试可打开结果；负责人未就位不能开始/通过 |
| v8 场景 | `create-test` |
| 产品真相是否被原型遵守 | **违反**（主按钮语义）。[journeys §1.6](../../../../personal/docs/product/user-journeys.md)：按环测、可打开结果、通过下一环、未就位不能开始/通过、未知不能通过、离线不能开测、失败回 ②/③、无引擎 chrome。v8 就位门、未知禁用通过、六项+模型检查、回 ③、Requires-environment Gap **遵守**。违反：「通过，下一环」调用 `onJoint()` 进 ⑤；`testState` 全局，换环仍可带着 pass 点通过；顶部分段器可直接点「达标」；失败只有文案没有回 ②；离线只写在标题，开始测不检查离线。 |
| 上手障碍 | 5 秒能懂「先就位再测」。5 分钟会误以为测一环就联调。 |
| 布局问题 | 轴 + 就位清单 + 测试面清楚。分段器抢主任务。通过态没有真正「打开结果」控件。 |
| 友好性（含九态） | idle/running/fail/unknown/pass 有文案。offline 未接控件。换环状态串味。 |
| 前瞻性 | 无引擎名、无过程 chrome。就位检查用业务六项。 |
| Honesty | Gap 标明环境缺口。分段器让人把样品开关当成真实达标。主按钮名实不符。 |
| 建议 | 见下 |
| 明确不改 | 不画引擎/过程 chrome；不把 HITL 放进聊天；不改 ⑤ 验收文案（本轮） |

### 分级

- **Must-fix**
  - 「通过，下一环」只切下一环；末环通过后才出现进入 ⑤。
  - 每环独立测试态；换环重置或读取该环结果。
  - 离线禁用开始测；失败给「回 ② 改这一环」。
- **Major**
  - 主路径去掉达标分段器（放进 State Lab）。
  - 通过态提供可打开结果样品（标明不是 daemon 文件）。
- **Minor**
  - 轴上显示本环测过/未测，避免只显示就位。
- **Parked**
  - 真测试执行 Requires-environment。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 6 |
| 状态 | 5 |
| 旅程 | 4 |
| AI-slop | 8 |
| 设计系统 | 7 |
| 响应/a11y | 7 |
| 未决 | 6 |

未决：④ 是否允许抽查后环（复制项目路径可跳过 ④⑤）。创建主路径应按序；副本抽查可另开门，不要用「通过进 ⑤」冒充。

### Click path（源码）

1. 未就位：开始测可用？`disabled={!seated}`；通过还要求 `testState === "pass"`。
2. 分段器点「达标」后，通过可点 → `create-joint`。
3. 换环不改 `testState`。
4. 失败 Notice 无导航按钮。
5. 无 offline 分支。

### 只留三件

1. 这一环负责人是否就位。
2. 可打开的子产出 + 过/不过/说不清。
3. 通过下一环（或失败回 ②/③）。

没有把 Install、引擎、Team/Inbox、CEO 轨、聊天 Approve 画进 ④。

---

## M-CREATE-5（closed）

| 项 | 内容 |
|---|---|
| ID | `M-CREATE-5` |
| 用户任务 | 全流程联调；「验收，进入 Today」是第一次成功 |
| v8 场景 | `create-joint` |
| 产品真相是否被原型遵守 | **漂移**。[journeys §1.7](../../../../personal/docs/product/user-journeys.md)：总成果可打开、核对态、「验收，进入 Today」才是 aha、未知不能验收、离线不能联调、失败点名环节回 ④/②/③、无假发布。v8：主 CTA 文案正确；`disabled={jointState !== "pass"}`；无 Publish；Gap 写明聊天不能验收；验收后 `lifecycle=live` + `today`。漂移：联合结果分段器可直接点「核对通过」；`index < 4` 写死前四环已完成；失败 Notice 无回退按钮；无离线/开始联调；总成果只是文案不是可打开控件。 |
| 上手障碍 | 5 秒能看见「第一次成功」和验收按钮。5 分钟能点验收（若用分段器作弊则过快）。 |
| 布局问题 | 一列步骤 + 一颗验收按钮，比 ③ 简洁。分段器仍抢主任务。 |
| 友好性（含九态） | pass/fail/unknown/idle 有。offline 无。success 验收后进日常 Today（正确，本模块只确认出口）。 |
| 前瞻性 | 无发布、无引擎。aha 留在验收，不在中间卡片。 |
| Honesty | 样品标签有。位置清单对 ④ 实际进度不诚实。 |
| 建议 | 见下 |
| 明确不改 | 不把日常决策包画进 ⑤ 完成前；不审 Today 默认三块（`M-TODAY`）；聊天无 验收 |

### 分级

- **Must-fix**
  - 失败：点名环节 + 回 ④/②/③ 按钮。
  - 离线不能开始联调。
  - 全流程位置跟 ④ 真实通过环走，禁止写死前四环 done。
- **Major**
  - 分段器进 State Lab；主路径「开始联调」→ 结果。
  - 总成果可打开样品（标明非 daemon / 非发布）。
- **Minor**
  - 失败环节不要写死「核对证据」。
- **Parked**
  - 真验收/独立核对 Requires-backend。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 8 |
| 状态 | 6 |
| 旅程 | 8 |
| AI-slop | 8 |
| 设计系统 | 8 |
| 响应/a11y | 7 |
| 未决 | 7 |

未决：验收后落点是 Today 决策包（产品）还是「刚上线」收据页再进 Today。产品说进入 Today；不要加庆祝营销页。

### Click path（源码）

1. 分段器点「核对通过」→ 验收可点。
2. 验收 → `setLifecycle("live"); setScene("today")`。
3. unknown/fail 验收 disabled。
4. 失败无导航。无开始联调。无 offline。

### 只留三件

1. 全流程现在走到哪（真实）。
2. 可打开总成果 + 核对态。
3. 「验收，进入今日」（仅核对通过）。

没有把 Publish、Install、引擎、Team/Inbox、CEO 轨、聊天 Approve、日常决策包画进未验收的 ⑤。

---

## M-TODAY-INCOMPLETE（closed）

| 项 | 内容 |
|---|---|
| ID | `M-TODAY-INCOMPLETE` |
| 用户任务 | 未验收时 Today 只继续创建 |
| v8 场景 | `today-incomplete`；`ProjectsScene` 的 `lifecycle === "creating"`；State Lab `today × blocked` |
| 产品真相是否被原型遵守 | **漂移**（中栏主任务遵守，续跑违反）。[user-journeys §1 末](../../../../personal/docs/product/user-journeys.md)：⑤ 前日常 Today 禁止；Today 只显示「继续未完成的创建」；Projects 只露出这份未完成草稿；Knowledge 在 ② 才开当前草稿；离开留草稿并续跑。[product-design](../../../../personal/docs/product/product-design.md) 同句。v8 `TodayIncompleteScene` **没有**决策包事实、运行概览、周期切换、四泳道、Publish。Projects `creating` 列表只含 `creating-draft`。知识 `createGate >= 2` 才解锁。漂移：`onContinue` / 项目「继续创建」一律 `setScene("create-init")`，无视已走到的 ②–⑤；中栏用 `.decision-packet` 冒充拍板卡；`creating` 时项目 L2 仍可进 成员/运行/产出（空态诚实，但是上线后 IA）。 |
| 上手障碍 | 5 秒能读懂「还没走完、不要当成功」。5 分钟从 ③ 点今日再继续，会被丢回 ①。 |
| 布局问题 | 一张「不是日常拍板」的决策包，视觉上仍是 live Today 的包。项目创建态两个并列继续按钮。 |
| 友好性（含九态） | empty/blocked（未完成）有原因 + 一主按钮。offline/unknown/error 无本表面真版式。State Lab 把 `today × blocked` 等同未完成创建，与上线后「阻塞项目」撞车。 |
| 前瞻性 | 没有把日常三块提前画进来。aha 仍在 ⑤。聊天无 Approve。 |
| Honesty | 文案诚实。续跑不诚实。无假 Install/Connect。 |
| 建议 | 见下 |
| 明确不改 | 不审 live Today 三块（`M-TODAY`）；不把 Skip/模板做成进日常；不在未完成 Today 画运行统计 |

### 分级

- **Must-fix**
  - 「继续未完成的创建」按 `createGate` 回到对应段（1→① … 5→⑤），项目列表同一条续跑。
  - 写明停在第几段（例如「停在 ③ 成员初始化」），不要用日常决策包标题。
  - `creating` 隐藏项目 L2 四去向；只留草稿行 + 继续。
- **Major**
  - 不用 `.decision-packet`；左对齐工作面 + 一个主按钮。
  - 未完成 × offline/unknown：继续创建 + 过时/说不清，禁止 live 包。
  - State Lab：`today × blocked` 与未完成创建拆开（细节 `M-STATE`）。
- **Minor**
  - 项目创建态去掉第二个「继续这份草稿」。
  - `locationLabel`「个人」改为「今日」（与 `M-SHELL` 同一条）。
- **Parked / 不采用**
  - Impeccable onboard 的 Skip 进日常、模板项目、欢迎仪式。
  - 不在此表面加周期统计或拍板 CTA。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 5 |
| 旅程 | 4 |
| AI-slop | 8 |
| 设计系统 | 7 |
| 响应/a11y | 7 |
| 未决 | 6 |

未决：右栏在未完成 Today 是否保持个人助手（产品未要求隐藏；建议保留，只指路继续创建，不回答上线运行数据）。`plan-ceo-review` HOLD SCOPE：不要为了「更丰满」把决策包提前过来。

### Click path（源码）

1. ①「离开，留草稿」→ `today-incomplete`。
2. 创建中点「今日」→ 同场景（`onNavToday`）。
3. 主按钮 → **总是** `create-init`。
4. 点「项目」→ 仅 `creating-draft` + 两个继续，均回 ①。
5. 创建中点 L2「运行」→ `project-runs` 诚实空（「未上线」），不是 Today 概览，但是上线 IA。
6. State Lab `today × blocked` → 本场景。`today × empty` 仍是空 Home。

### 只留三件

1. 一句话：创建未完成，日常今日还没开始。
2. 现在停在第几段。
3. 一个按钮：从那里继续。

没有把 Install、引擎、Team/Inbox、CEO 轨、四泳道、四个「查看」、聊天 Approve、日常决策包、运行概览画进未完成 Today。

---

## M-TODAY（closed）

| 项 | 内容 |
|---|---|
| ID | `M-TODAY` |
| 用户任务 | 回访：一件拍板 + 上线项目运行概览 + 助手 |
| v8 场景 | `today`；State Lab `today` × working/success/partial（及错误地把 empty/blocked 映射走） |
| 产品真相是否被原型遵守 | **漂移**。[user-journeys §2](../../../../personal/docs/product/user-journeys.md)：三块=决策包（唯一主 CTA）+ 上线项目运行概览（含创建/上线/阻塞计数与周期）+ 助手；点**某个已上线项目**看环节；四泳道不是默认块；聊天不能批；点统计不能发布；拒绝/以后再说留在 Today；未知费用不写 0；offline 上次概览标过时。v8 **遵守**：决策包含后果/可逆/备选/内核真相/为何先 A；主按钮「去处理这一件拍板」进画布 HITL；聊天只链接预览、无 Approve；无四泳道墙、无 Publish、无 CEO 轨。**漂移**：概览默认是一条流程轴 +「打开运行管理」，不是可点的项目行；阻塞「1」不可点；无待拍板变体；State Lab `today × empty` 用空 Home（藏聊天）；`today × blocked` 用未完成创建；周期几乎只改一个数字；「以后再说」无 handler。 |
| 上手障碍 | 5 秒能看见拍板。5 分钟能进 HITL。两个上线项目无法从概览区分。 |
| 布局问题 | 拍板在上、计数、再概览，层次对。流程轴把 Today 画成运行管理的缩小版。 |
| 友好性（含九态） | working 有。empty（无拍板但仍有项目）、loading、offline、live-blocked 无真版式。 |
| 前瞻性 | 助手可查运行、不能批。内核真相承认无合格连接器。无引擎。 |
| Honesty | 费用未知不写 0。月「已完整执行」写说不清。计数写样品。轴上失败 0 是样品，不像未知。 |
| 建议 | 见下 |
| 明确不改 | 不把四泳道画回来；不审 HITL 预览细节（`M-HITL`）；不审项目四子菜单（`M-LIVE-PROJECT`）；03 章四泳道只记录漂移 |

### 分级

- **Must-fix**
  - 运行概览=已上线项目行（状态、今日完成次数、当前环节、时长）。点行进该项目运行（或行内展开环节表）。Today 默认不要全流程轴。
  - 「发生阻塞」可点，打开阻塞项目的运行，而不是一句说明。
  - 无待拍板：不画决策包（或空包一句），保留概览。禁止 `EmptyHomeScene`。
- **Major**
  - loading：概览刷新中，决策包仍可点。offline：上次概览 + 过时。unknown 行写说不清。live blocked ≠ `today-incomplete`。
  - 周期切换更新所有周期字段，否则标说不清。
  - 「以后再说」留包 + 回执。
- **Minor**
  - 今日角标只在有待拍板时出现（与 `M-SHELL` 同一条）。
  - 聊天 `approval-card` 改名/改样式，避免看起来像聊天里的批准钮。
- **Parked**
  - 03 章四泳道正文不在本战役改产品/语料。泳道语义可并进概览行，不单独成块。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 4 |
| 旅程 | 7 |
| AI-slop | 8 |
| 设计系统 | 7 |
| 响应/a11y | 7 |
| 未决 | 6 |

未决：点项目行是跳 `project-runs` 还是 Today 内展开。建议跳运行管理（产品「点进详情」），Today 只保留行级摘要。`plan-ceo-review` HOLD SCOPE：不把 Inbox/Team/KPI 墙加回来。

### Click path（源码）

1. ⑤ 验收 → `lifecycle=live` + `today`。
2. 「去处理这一件拍板」→ `hitl`。
3. 「以后再说」无 `onClick`，人留在 Today，无回执。
4. 周期 segmented → 仅「已完整执行」在今日/本周/说不清之间变。
5. 「打开运行管理」→ `project-runs`（不选是哪一个项目）。
6. 点轴上某环 → 行内成功/失败/时长样品。
7. 右栏：运行数据说明 +「打开画布预览」；无 Approve。
8. State Lab `today × empty` → 空 Home；`blocked` → 未完成创建。

### 只留三件

1. 这一件拍板（或明确没有）。
2. 每个上线项目一行运行摘要（含阻塞入口）。
3. 右栏助手查数、不能批。

没有把 Install、引擎、Team/Inbox、CEO 轨、四泳道、四个「查看」、聊天 Approve 画进日常 Today。

---

## M-PROJECTS（closed）

| 项 | 内容 |
|---|---|
| ID | `M-PROJECTS` |
| 用户任务 | 找到/复制已上线 Project；副本未激活 |
| v8 场景 | `projects`（live 分支） |
| 产品真相是否被原型遵守 | **漂移**。[journeys §1.8](../../../../personal/docs/product/user-journeys.md)：列表或项目页复制；不带密钥/在途/回执/时间盒跳过；落地未激活副本；改完总预览；④⑤ 可抽检或跳过；不从 ① 重来。列表一行一个「打开」，成员/运行/产出文字链，不要四个「查看」。v8 **遵守**：`打开` + `成员`/`运行`/`产出` 文字钮；复制文案点名不带密钥等；`weekly-copy` kind=`copy-draft`；成员/运行/产出对非 live 诚实空；无示范项目文案。**漂移**：复制只 `setCopied(true)`，无总预览 CTA；详情页不能复制；live「创建项目」只 `setScene("create-init")` 且生命周期仍是 live，列表不出现新草稿。 |
| 上手障碍 | 5 秒能扫到两个项目和打开。复制后仍停在列表，不知道下一步是总预览。 |
| 布局问题 | 每个项目一张大卡（目标/周期/费用定义列表），不像「一行」。 |
| 友好性（含九态） | empty 走空生命周期（已审）。live 列表无 loading/offline/unknown。State Lab `projects` 只切 empty/live。 |
| 前瞻性 | 无 Install、无引擎。复制不是从 ① 重来（文案对）。 |
| Honesty | 副本费用行诚实。副本详情仍画出带负责人名的流程轴，像班子已在。 |
| 建议 | 见下 |
| 明确不改 | 不恢复四个「查看」；不把 03 搜索/筛选当本战役必须；不深审四子菜单页（`M-LIVE-PROJECT`） |

### 分级

- **Must-fix**
  - 复制后：未激活草稿 + **总预览** 才可上线。④⑤ 抽检是总预览之后的可选，不是替代激活门。
  - live 列表「创建项目」：要么新增一条未激活草稿行（与副本同一套），要么删除该主按钮、第二份工作只走复制。禁止看不见的 ①。
- **Major**
  - 一行一项目的紧凑表。
  - 项目详情也提供复制。
  - 副本详情不要把已上线负责人轴画成已就位班子。
  - State Lab `projects` 补 loading/offline/unknown。
- **Minor**
  - 现场周检也应能复制（现在只有周报行）。
  - banner 与行内文案去重。
- **Parked**
  - 03 章搜索/状态筛选。多项目复杂过滤不是 2.0 mutex。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 6 |
| 旅程 | 5 |
| AI-slop | 8 |
| 设计系统 | 6 |
| 响应/a11y | 7 |
| 未决 | 6 |

未决：已有上线项目后，从零创建是一等路径还是复制才是一等。HOLD SCOPE 建议复制为一等；从零创建必须有草稿行，不能藏进向导。

### Click path（源码）

1. live → 项目：周报 + 现场两张卡，各一个打开 + 三个文字链。
2. 仅周报行有「复制为草稿」→ 列表多一行副本 + banner；仍停在列表。
3. 副本「打开」→ `project-detail`（可改名/目标/周期），无总预览。
4. 副本「运行」→ 诚实空「未激活副本不能当已上线项目跑」。
5. 「创建项目」→ `create-init`，`lifecycle` 仍 live；点今日回到日常决策包。
6. 无四个「查看」。

### 只留三件

1. 已上线项目一行：打开 + 成员/运行/产出。
2. 复制为未激活草稿。
3. 总预览（激活门）。

没有把四个「查看」、Install、引擎、Team/Inbox、CEO 轨、聊天 Approve 画进列表。

---

## M-LIVE-PROJECT（closed）

| 项 | 内容 |
|---|---|
| ID | `M-LIVE-PROJECT` |
| 用户任务 | 四个子菜单：详情 / 成员 / 运行 / 产出 |
| v8 场景 | `project-detail`；`project-runs`；`project-outputs`（成员入口只确认有子菜单，配置留给 `M-MEMBER-CONFIG`） |
| 产品真相是否被原型遵守 | **漂移**。[journeys §3](../../../../personal/docs/product/user-journeys.md)：L2=详情/成员/运行/产出；详情=只读轴+去向；运行=当前环工作面，点轴换环；「验收，回 Today」只在末环；产出先选后看；无可见 CEO 轨；聊天不能批/验收；空环说缺什么；unknown 不能完成；offline 标过时。v8 **遵守**：运行轴可点；中途文案禁止验收；末环才出验收控件；产出未选空态、不默认第一份；无 CEO 轨、无假发布；过程痕迹默认收起。**漂移**：`PROJECT_SUBNAV` 是 列表/成员/运行/产出，没有「详情」，详情被标成列表当前页；详情里名称/目标/周期可编；末环验收 `onClose`→`today`，不打开成果；验收与「去授权预览」同时出现且验收可先点。 |
| 上手障碍 | 5 秒能从 L2 进成员/运行/产出。从运行回详情要先回列表再打开。 |
| 布局问题 | 运行：计数 + 轴 + 工作面对。详情像设置表。 |
| 友好性（含九态） | 未开始环有「缺什么」。offline 无。产出 unknown/empty 有样品开关。 |
| 前瞻性 | 无引擎 chrome。HITL 在画布。 |
| Honesty | 样品标注有。L2 把详情假装成列表。验收按钮名「打开成果并验收」却不去产出。 |
| 建议 | 见下 |
| 明确不改 | 不画 CEO 轨；不审八标签；不审加人；不审 HITL 预览页 |

### 分级

- **Must-fix**
  - L2：详情 / 成员 / 运行 / 产出。L1「项目」才是列表。
  - live 详情只读章程 + 轴 + 去向。编辑走预览确认，不当表单页。
  - 末环验收：打开成果 + 核对态，再回今日。未完成该环拍板/核对比验收禁用。
- **Major**
  - 已上线详情/成员/产出用项目群（`M-CHAT-CANVAS` 细做）。
  - 产出样品分段器进 State Lab。
  - 运行补 offline（上次状态过时）/ unknown（不能标完成）。
- **Minor**
  - 验收文案改成产品句「验收，回今日」，主按钮级次低于当前环拍板。
  - 成员子菜单先选后看已遵守，此处不改。
- **Parked**
  - Goal→Attempt 钻取、引擎痕迹保持收起。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 6 |
| 状态 | 6 |
| 旅程 | 6 |
| AI-slop | 8 |
| 设计系统 | 7 |
| 响应/a11y | 7 |
| 未决 | 6 |

未决：L2 是否保留「返回列表」作为第五项。建议不要；列表用 L1。`plan-ceo-review` HOLD SCOPE：不把 Inbox 或 CEO 轨加回来。

### Click path（源码）

1. 「打开」→ `project-detail`；L2 高亮「项目列表」。
2. 详情可改名称并走画布确认链。轴是 div，不可点。
3. L2 运行 → 点任一环换工作面；末环出现「打开成果并验收，回今日」→ `today`。
4. 末环同时有「去授权预览」进 HITL。验收不检查预览结果。
5. 产出：未选右侧空；点一份看编排；聊天无批准。
6. 成员：未选空，不默认第一人（本模块只记入口）。

### 只留三件

1. 四个去向（含详情）。
2. 运行：当前环工作面 + 末环验收（先打开成果）。
3. 产出：先选后看。

没有把 CEO 轨、四泳道、四个「查看」、Install、引擎、Team/Inbox、聊天 Approve 画进已上线项目。

---

## M-ADD-MEMBER（closed）

| 项 | 内容 |
|---|---|
| ID | `M-ADD-MEMBER` |
| 用户任务 | 已上线后加一个岗位：职责 → 确认加入 → 再披露执行方式 |
| v8 场景 | `add-member`（从 `project-members`「加人」进入） |
| 产品真相是否被原型遵守 | **漂移**。[journeys §4](../../../../personal/docs/product/user-journeys.md)：现有名单 + 聊天建议岗位 +「确认加入」；先岗位后执行方式；不是先装 MCP；名单=当前项目真实成员；拒绝=未加入；无模型=pending 去设置；加入后改流程/权限再批；离线可改职责不能搜方案；空=岗位还不存在。v8 **遵守**：`workMembers` 是当前项目；职责/交出在前；模型字段有；无 Install；Gap 写离线/daemon；加入后文案「不是先装 MCP」。**漂移**：无「拒绝」控件；`disabled={model === "unselected"}` 使无模型无法加入（产品允许 pending）；加入后只有 Notice「回项目画布打开查看配置」，无打开配置页按钮；默认预填「客户跟进人」像全局示范岗；右侧是个人助手但没有建议岗位的样品回合；State Lab 无 `add-member` 表面（只有 `members`）。 |
| 上手障碍 | 5 秒看见「给已上线项目补一个岗位」和现有班子。5 分钟能点「确认加入」，但加入后找不到执行方式。文案说拒绝，控件没有。 |
| 布局问题 | 现有班子 + 新岗位两块清楚。执行方式被推迟到另一页却断了去路。 |
| 友好性（含九态） | empty 被预填掩盖。success=joined Notice。blocked=按钮 disabled 而非 pending。offline/unknown/loading/error 无真版式。 |
| 前瞻性 | 默认 chrome 是岗位与职责。Skill/MCP 未抬到加人首页（对）。模型出现在加入门上（身份，可留）。 |
| Honesty | Gap Requires-backend。无假 Connect/Install。加入是本地 `setJoined`，标注原型。假禁用：文案允许 pending，按钮不许。 |
| 建议 | 见下 |
| 明确不改 | 不审八标签（`M-MEMBER-CONFIG`）；不把完整配方画进加人首页；不先装 MCP；不引入全局示范名单；不画 2.1 成员共享 |

### 分级

- **Must-fix**
  - 「拒绝」= 未加入（不清空已有成员；新岗位不进名单）。
  - 无模型可「确认加入」为 pending，并给「去设置」；禁止 disabled 掉主按钮冒充不能加人。
  - 加入后主按钮「初始化执行方式」打开配置页（同一套 `MemberConfigPanel`）。禁止死文案「查看配置」。
- **Major**
  - 空态：岗位名/职责空白 +「这个岗位还不存在」。聊天里给一条建议岗位样品；Owner 确认后才填画布。
  - 离线：职责可编；不要画「搜岗位方案」假按钮。unknown 不能写成已加入。
  - State Lab `members × empty` 已挂成员页；加人页自身的 empty/offline 也要能扫（或把加人作为 members 的 working 变体说明）。
- **Minor**
  - 加入成功后现有名单应立刻出现新人（`onJoin` 已 merge；确认列表刷新）。
  - 标题「补一个岗位」保留；副文缩短。
- **Parked 2.1**
  - 不跨项目共享成员；不把 MCP 市场画进加人。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 4 |
| 旅程 | 6 |
| AI-slop | 8 |
| 设计系统 | 8 |
| 响应/a11y | 7 |
| 未决 | 7 |

未决：模型放在加入门还是加入后身份头。产品：模型必选 + 无模型=pending → 加入门保留模型，未选则 pending。

⚠️ Impeccable `critique` **DEGRADED**：scripts 未 vendoring；本会话禁止再开子代理。Assessment A 源码走查。Assessment B not-run。

### Click path（源码）

1. 成员管理「加人」→ `add-member`。名单来自 `workMembers`。
2. 预填岗位名/职责；模型默认未选 →「确认加入」disabled。
3. 选模型后加入：`setJoined(true)` 写入当前项目名单，`initStatus` idle 或 blocked。
4. Notice 指向「查看配置」，无 `onClick`。
5. 无拒绝。聊天无建议岗位回合。无 Approve。

### 只留三件

1. 当前项目已有名单。
2. 新岗位：做什么 / 交出什么 + 确认加入 / 拒绝。
3. 加入后打开执行方式（配置页）。模型/设置下沉为身份或 pending。

没有把 Install、引擎、Team/Inbox、CEO 轨、四泳道、四个「查看」、聊天 Approve 画进加人。

---

## M-MEMBER-CONFIG（closed）

| 项 | 内容 |
|---|---|
| ID | `M-MEMBER-CONFIG` |
| 用户任务 | 先选人再看完整执行方式 |
| v8 场景 | `project-members`（主路径）；`member-config`（创建/加人后的整页配置） |
| 产品真相是否被原型遵守 | **漂移**。[journeys §4](../../../../personal/docs/product/user-journeys.md) / [opc-product-model](../../../../personal/docs/product/opc-product-model.md) / 语料 05：未选空态、不默认第一人、换项目清空、八标签、身份留详情头、输入=只读流程合同、输出=可编交出什么、无安装。v8 **遵守**：`selectedId` 初值 null；`useEffect` 换项目清空；八标签文案正确；输入是 `dl` 不是私有字段；输出 `SyncedField`；无 Install 按钮。**漂移**：`MemberModelSelect` 在头下面、标签上面，像第二张表单；槽位 `small` 写出 `runtimeLabel`（「进阶也称 loop」「Skill」「MCP · 精确授权」）；`SURFACE_CONTEXT.members` 仍写「六项执行方式」。 |
| 上手障碍 | 5 秒能懂「先选人」。选人后八标签可扫完。能力术语出现在每个槽位副文，增加噪音。 |
| 布局问题 | 左名单右配置是对的 master-detail。八标签是产品要求，不要减成概览页。整页 `member-config` 与右侧面板应是同一组件（已基本共用 `MemberConfigPanel`）。 |
| 友好性（含九态） | 未选空态有。未激活副本诚实空。State Lab `members` 除 empty 外九态同一张 live 名单。 |
| 前瞻性 | 标签是业务名。槽位副文把引擎层抬上来。无 Loop 页签名（「周期与触发」正确）。 |
| Honesty | Gap 有。无假 Install。就位 Tag 标明不是 daemon。输入只读文案诚实。 |
| 建议 | 见下 |
| 明确不改 | 不把完整配方塞回 ③ 当前条；不默认第一人；不编造输入字段；不把 Loop 当标签名；加人流不在此重做 |

### 分级

- **Must-fix**
  - 模型 / 就位 / 负责环节只在详情头（模型可做成头内选择）。禁止标签上方再放一张模型表单。
  - 槽位默认只显示业务名与状态；Loop / Skill / MCP 放进一层 `details`「能力说法」。
- **Major**
  - State Lab `members`：loading 骨架名单、error 保留上次、unknown 不能当已就位、offline 标过时可看配置。
  - `SURFACE_CONTEXT.members` 改为八标签，去掉「六项」。
  - 已上线右栏项目群（与 Major 17 合并，`M-CHAT-CANVAS` 落地）。
- **Minor**
  - 未选空态文案可加「加人」次按钮（成员页头已有加人）。
  - 技能标签业务名「能力包」与页签「技能」统一用「技能」。
- **Parked**
  - 成员级预算、引擎商店、跨项目共享。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 8 |
| 状态 | 5 |
| 旅程 | 8 |
| AI-slop | 7 |
| 设计系统 | 8 |
| 响应/a11y | 7 |
| 未决 | 8 |

未决：整页 `member-config` 是否只保留为创建 ③ / 加人后的深链。建议保留，内容必须与右侧面板同一套。

### Click path（源码）

1. `project-members`：不选人 →「还没选人」。点第一人后才出面板（不是默认选中，要点一下）。
2. 换 `ProjectSwitcher` → `setSelectedId(null)`。
3. 标签：职责可编；输入只读；输出可编交出；技能/工具/工作说明/周期与触发/连接与权限走 runtime slots。
4. 无安装按钮。聊天「确认，写回画布」是编辑链，不是 HITL Approve。
5. `member-config` 无 member 时 Notice「还没有这个岗位」。

### 只留三件

1. 左侧名单（先选）。
2. 详情头：身份 + 就位。
3. 八标签（输入只读、输出可编）。其余术语下沉。

没有把 Install、引擎商店、Team/Inbox、CEO 轨、四泳道、四个「查看」、聊天 Approve、默认第一人画进来。

---

## M-HITL（closed）

| 项 | 内容 |
|---|---|
| ID | `M-HITL` |
| 用户任务 | 对外/关键动作：画布预览上批准 / 改窄 / 拒绝；可选本周同类不再问 |
| v8 场景 | `hitl`；State Lab `hitl` × blocked/unknown/offline |
| 产品真相是否被原型遵守 | **漂移**。[journeys §6](../../../../personal/docs/product/user-journeys.md)：将做什么 + 完整预览/差异 + 画布三键；执行中第四键「停」；改窄要新预览；过期不能批；聊天只链接无 Approve；时间盒可收回、无永久 Don’t ask again；拒绝不发出并回该环；离线不能批对外；empty=无待批。v8 **遵守**：三键在画布；过期禁用批准；聊天「打开画布预览」无批准；checkbox 文案到期失效；明确不是社交发帖；Stop 在 `executing` 时出现。**漂移**：改窄只 `setFate("narrowed")`，`previewAge` 仍 fresh，仍可批准旧预览；empty 仍渲染同一份可批包；unknown/offline 都映射 stale，文案只有「过期」；主路径「新鲜/过期」分段器可作弊；拒绝 `onBack()` 无回执。 |
| 上手障碍 | 5 秒看见将做什么和三键。5 分钟能批/拒。改窄后误以为旧预览仍有效。 |
| 布局问题 | 单一预览卡密度对。分段器是质检控件，不该占工作面。 |
| 友好性（含九态） | working=执行中+停。success=已请求发出（非完成回执）。stale=过期。缺 empty 无待批。unknown 被过期冒充。 |
| 前瞻性 | 默认是目标与预览，无引擎。时间盒在预览上，收回在设置。 |
| Honesty | Gap environment（Intent/Effect）。无聊天 Approve。改窄后仍能批是假门。 |
| 建议 | 见下 |
| 明确不改 | 不把批准放进聊天；不画 Inbox 一级；不画永久不再问；不引入 X；不把 Today 决策包改成本页 |

### 分级

- **Must-fix**
  - 改窄 → 旧预览过期，批准禁用，直到「新预览」（原型可用 State Lab working/fresh 表示）。
  - empty：无待批，不要可批准预览。
  - unknown：说不清，禁止批准、禁止盲重试；不要只写过期。
- **Major**
  - 新鲜/过期分段器进 State Lab。
  - 拒绝后回到该环并留「未发出」回执。
  - 离线：过时 + 不能批对外（State Lab 已用 stale，文案改成过时）。
- **Minor**
  - 「已请求发出」保持非成功完成；回执钉在环节页用文字链指出。
- **Parked**
  - 真 daemon Intent/Effect。Inbox 一级。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 8 |
| 状态 | 5 |
| 旅程 | 7 |
| AI-slop | 9 |
| 设计系统 | 8 |
| 响应/a11y | 7 |
| 未决 | 8 |

未决：时间盒 checkbox 是否只在对外类预览显示。建议保留，设置可收回。

ai-agent-ux：预览在画布、可停可改窄，符合 AUTONOMY 的 Preview / Override / Yield。缺口是改窄未作废预览。

### Click path（源码）

1. Today「去处理这一件拍板」或运行「去授权预览」→ `hitl`。
2. 新鲜：批准 → executing + fate approved；出现停。
3. 过期：批准 disabled。
4. 改窄：Notice，批准仍可用。
5. 拒绝：立刻 `project-runs`。
6. 聊天：`打开画布预览`，无 Approve。

### 只留三件

1. 将做什么 + 完整预览。
2. 批准 / 改窄 / 拒绝（执行中加停）。
3. 过期/未知/离线不能批。时间盒下沉设置可收回。

没有把聊天 Approve、Install、引擎、Team/Inbox、CEO 轨、四泳道、X 画进 HITL。

---

## M-KNOWLEDGE（closed）

| 项 | 内容 |
|---|---|
| ID | `M-KNOWLEDGE` |
| 用户任务 | 看资料、Why this fragment、导入；检查/忘记 Memory |
| v8 场景 | `knowledge`；创建期可从 ② 打开当前草稿；State Lab `knowledge` × empty/working/partial/success/loading |
| 产品真相是否被原型遵守 | **漂移**。[journeys §5](../../../../personal/docs/product/user-journeys.md)：无项目锁定；② 才为草稿打开；Why this fragment；导入失败留原件；聊天自动承认可检查 Memory；Obsidian 不必装该应用；密钥不进 Vault。v8 **遵守**：`locked={!knowledgeOk}`；`draftOnly`；Why 表；parse-fail/duplicate/secret 文案；忘记记忆；无 Install。**漂移**：知识默认 `chatHidden` 除非「打开对话」（已记 Must-fix 1）；导入进行中主路径四按钮可点「已索引」；离线文案有、导入钮不禁；L1 `aria-disabled` 仍可进（Must-fix 2）。 |
| 上手障碍 | 5 秒看见资料/导入/为什么/记忆。5 分钟能走导入样品。锁定页没有回创建的按钮（空 Home 已有创建，可接受）。 |
| 布局问题 | 四标签是产品面。筛选条偏密但不是 KPI 墙。右栏被收起破坏三栏。 |
| 友好性（含九态） | empty 资料有导入 CTA。partial=`filesEmpty`。working=importing。缺 offline 真禁导入。unknown 未做。 |
| 前瞻性 | 默认是资料与为什么用。Codex 只在记忆副文。无引擎商店。 |
| Honesty | 不写磁盘。密钥检出走 SecretStore。四按钮是原型作弊器，需进 State Lab。 |
| 建议 | 见下 |
| 明确不改 | 不嵌入 Obsidian；不把目标/角色/权限状编辑做进本页静默改权威；无项目云导入是 2.1 |

### 分级

- **Must-fix**
  - 与壳层合并：知识默认第三列常驻（Must-fix 1）。锁定可进说明或真 disabled（Must-fix 2）。
- **Major**
  - 导入四按钮进 State Lab。离线禁用开始导入。unknown ≠ 0 条。
- **Minor**
  - 锁定页加「去创建项目」或「回流程 ②」视生命周期。
  - 筛选默认收起，需要时再打开。
- **Parked 2.1**
  - 无项目云导入、native mobile 库。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 6 |
| 旅程 | 8 |
| AI-slop | 8 |
| 设计系统 | 8 |
| 响应/a11y | 5 |
| 未决 | 7 |

未决：记忆是否默认收在「为什么用这段」旁的次级。建议保持独立标签。

### Click path（源码）

1. 空生命周期点知识 → 锁定文案（且可导航）。
2. live：资料列表 + 导入 + Why 表 + 忘记记忆。
3. 开始导入 → 四按钮切 duplicate/fail/secret/indexed。
4. 创建 `createGate>=2` 时 `draftOnly` 只看当前草稿。

### 只留三件

1. 当前项目资料（空则导入）。
2. Why this fragment。
3. 可检查/忘记的 Memory。导入失败态下沉但必须可到达。

没有把 Install、引擎商店、Team/Inbox、CEO 轨、四泳道、聊天 Approve、X 画进知识。

---

## M-SETTINGS（closed）

| 项 | 内容 |
|---|---|
| ID | `M-SETTINGS` |
| 用户任务 | 连模型、收回本周不再问、通知与恢复 |
| v8 场景 | `settings`；State Lab `settings` × error/success |
| 产品真相是否被原型遵守 | **漂移**。[journeys §8](../../../../personal/docs/product/user-journeys.md) / [account-hub](../../../../personal/docs/product/account-hub.md)：主流模板+自定义；密钥单向 SecretStore、不回显、不进聊天；空=未连接去连接；收回时间盒；通知恢复；无账单、无引擎商店、无 Inbox。v8 **遵守**：下拉+自定义字段；password 交接后 `setKeyDraft("")`；无 Connect 打 Provider；跳过收回；无账单/商店/Inbox；诊断默认不写引擎名。**漂移**：主路径 `handoff` 必 `connected`；failed 文案不点名；设置默认藏右栏（Must-fix 1）；offline 仍可点交接。 |
| 上手障碍 | 5 秒看见模型连接。5 分钟能走完交接样品。失败怎么修说不清。 |
| 布局问题 | 三块（连接 / 不再问 / 通知）清楚，密度对。 |
| 友好性（含九态） | empty=尚未连接。success=已连接（原型）。error=failed 泛文案。缺 offline/unknown 真版式。 |
| 前瞻性 | 默认是连模型。引擎诊断后置为一段说明。无商店。 |
| Honesty | 按钮写「原型，不联网」。仍把状态写成「已连接」易被当成真 Provider。Requires-backend Gap 有。密钥在交接前存在 React state（原型限制，不进 git/聊天）。 |
| 建议 | 见下 |
| 明确不改 | 不画订阅管理；不画引擎商店；不把密钥画进聊天/画布；不把 Inbox 放进设置一级 |

### 分级

- **Must-fix**
  - 设置第三列常驻（Must-fix 1）。
- **Major**
  - 失败点名（超时 / 密钥被拒 / 自定义 URL 无效）。离线禁用交接。成功态标明 Requires-backend。unknown 额度不写 0。
- **Minor**
  - 「尚未连接」放在供应商下拉之上，避免已选 Anthropic 像已连上。
- **Parked 2.1**
  - 消费订阅、成员级预算、DSH/Pi 商店。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 8 |
| 状态 | 5 |
| 旅程 | 8 |
| AI-slop | 9 |
| 设计系统 | 8 |
| 响应/a11y | 6 |
| 未决 | 8 |

未决：自定义 URL 是否折叠。建议默认主流下拉，自定义才展开（已如此）。

### Click path（源码）

1. 底栏设置。无密钥时交接 disabled。
2. 输入密钥 → 交接 → 输入清空、status connected、`providerBound`。
3. skipWeek 时「收回跳过」。
4. 无账单、无 Install。

### 只留三件

1. 连接模型（模板 / 自定义 / 交接）。
2. 收回本周不再问。
3. 通知与恢复（主机离线则停）。其余诊断下沉。

没有把 Install 商店、引擎名默认 chrome、Team/Inbox、CEO 轨、聊天 Approve、订阅画进设置。

---

## M-CHAT-CANVAS（closed）

| 项 | 内容 |
|---|---|
| ID | `M-CHAT-CANVAS` |
| 用户任务 | 右栏始终是第三列；画布编辑 → Enter → 确认框 → 聊天里确认 → 再写画布 |
| v8 场景 | 创建/成员/测试/联调默认 Assistant；上线后项目群；`EditConfirmDialog` |
| 产品真相是否被原型遵守 | **漂移**。[scope §3.1](../../../../personal/docs/product/personal-2.0-scope.md) / [web-ui-design](../../../../personal/docs/product/web-ui-design.md)：三栏锁定；无 overlay「打开会话」；聊天无 Approve；`@` 只进草稿；创建环个人助手；上线后项目群。v8 **遵守**：`min-width: 1100px` + `overflow-x: auto`；编辑确认框；聊天「确认，写回画布」是编辑链不是 HITL；Today/运行 HITL 只有「打开画布预览」；mention 写入 drafts。**漂移**：`isSetupChat` 含 `project-detail` / `project-members` / `project-outputs` / `add-member` / `member-config`，已上线也显示个人助手；知识/设置「打开对话」把第三列收成开关（Must-fix 1）。 |
| 上手障碍 | 创建页 5 秒能看见助手。已上线成员页没有项目群发言条。 |
| 布局问题 | 默认三栏对。`chat-hidden` 把知识/设置叠成两列。确认框是产品规定的 modal，不是「打开会话」。 |
| 友好性 | 无模型时创建聊天指路设置。缺后端 Gap。 |
| 前瞻性 | 对话是同伴不是开关。把对话收起是把配置层做成默认。 |
| Honesty | 发送按钮写原型。无聊天 Approve。 |
| 建议 | 见下 |
| 明确不改 | 不把 HITL 批准放进聊天；不把三栏改抽屉；2.1 远程会话不画；编辑确认链保留 |

### 分级

- **Must-fix**
  - 已上线工作面右栏项目群（Must-fix 27）。知识/设置常驻第三列（Must-fix 1）。
- **Major**
  - `add-member` 可用个人助手建议岗位（产品允许），但加人完成后切回项目群。
- **Minor**
  - `locationLabel`「个人」在 Today 改为「今日」。
- **Parked**
  - overlay 远程会话。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 6 |
| 状态 | 7 |
| 旅程 | 7 |
| AI-slop | 8 |
| 设计系统 | 8 |
| 响应/a11y | 7 |
| 未决 | 7 |

未决：产出页助手编排 vs 项目群。产品：产出换展示要聊天确认，可用项目群里的助手发言，不要改成个人助手。

### Click path（源码）

1. 画布 `SyncedField` 回车 → `EditConfirmDialog` → 聊天 pending proposal →「确认，写回画布」。
2. `@` 按钮只改 drafts。
3. 无 Approve。知识/设置默认无第三列。

### 只留三件

1. 第三列常驻对话。
2. 编辑确认链。
3. HITL 只链到画布。身份：创建=助手，上线=项目群。

没有把 Install、引擎、Team/Inbox、CEO 轨、四泳道、聊天 Approve、打开会话 overlay 画成产品默认（知识/设置按钮是要删的漂移）。

---

## M-STATE（closed）

| 项 | 内容 |
|---|---|
| ID | `M-STATE` |
| 用户任务 | 同一表面在 loading/empty/working/error/success/partial/blocked/unknown/offline 下的真版式 |
| v8 场景 | `state-lab`（SurfaceKey × StateKey） |
| 产品真相是否被原型遵守 | **漂移**。[web-ui-design](../../../../personal/docs/product/web-ui-design.md) / journeys §12：unknown 不是 0/成功、禁止盲重试；offline 标过时；State Lab 用真版式不是 Designed 矩阵。v8 **遵守**：九个 StateKey；banner 文案正确；部分表面有 native 映射（today empty/working/blocked、knowledge 若干、projects、members、runs、outputs、hitl、settings）。**漂移**：`create` 全部掉进占位段；today 的 loading/error/unknown/offline 占位；hitl empty 仍是可批包；members 九态同一名单；runtime/NVDA/对比度/200% **not-run**（不得写成已验证）。 |
| 上手障碍 | 质检员能切表面。实现者会误以为占位段等于真版式。 |
| 布局问题 | Lab 控件在上、banner、再 native。占位段与真表面密度不一致。 |
| 友好性 | Banner 把 unknown/offline 说清楚。native 未跟。 |
| 前瞻性 | Lab 不是产品 chrome，是质检。不要画进 L1。 |
| Honesty | 标题已写 not-run。占位段自称「已按真版式渲染」是假话。 |
| 建议 | 见下 |
| 明确不改 | 不把 Lab 做成验收；不声称 NVDA/对比度已测 |

### 分级

- **Must-fix**
  - 占位段不得声称真版式。`create` 挂 ①–⑤；today 的 loading/error/unknown/offline 挂对应 Today/空 Home；hitl empty=无待批。
- **Major**
  - members/runs/settings 九态分版式（与各模块 Major 合并）。
- **Minor**
  - 占位段改成「本组合尚未挂 native」。
- **Parked / not-run**
  - Canvas runtime/render、NVDA、host-theme contrast、200% 真布局。

### 打分（仅本模块）

| Pass | 分 |
|---|---:|
| IA | 7 |
| 状态 | 4 |
| 旅程 | 6 |
| AI-slop | 8 |
| 设计系统 | 7 |
| 响应/a11y | 3 |
| 未决 | 8 |

未决：blocked 在 today 是未完成创建还是上线阻塞。v9：blocked 分 `creating` vs live 决策包阻塞，不要混成一张。

### Click path（源码）

1. 场景「状态实验室」切 Surface × State。
2. `renderNative` 未匹配则占位段。
3. 页头写明 not-run。

### 只留三件

1. 真版式（同一套 StateKey）。
2. unknown / offline 诚实标签。
3. not-run 边界。不要第二张 Designed 矩阵。

没有把 Lab 当 Gate 证据，没有把 2.1 画进九态。

---

## 模块队列

| # | ID | 状态 |
|---|---|---|
| 1 | M-SHELL | closed |
| 2 | M-EMPTY | closed |
| 3 | M-CREATE-1 | closed |
| 4 | M-CREATE-2 | closed |
| 5 | M-CREATE-3 / M-MEMBERS-INIT | closed |
| 6 | M-CREATE-4 / M-TEST-STAGE | closed |
| 7 | M-CREATE-5 | closed |
| 8 | M-TODAY-INCOMPLETE | closed |
| 9 | M-TODAY | closed |
| 10 | M-PROJECTS | closed |
| 11 | M-LIVE-PROJECT | closed |
| 12 | M-ADD-MEMBER | closed |
| 13 | M-MEMBER-CONFIG | closed |
| 14 | M-HITL | closed |
| 15 | M-KNOWLEDGE | closed |
| 16 | M-SETTINGS | closed |
| 17 | M-CHAT-CANVAS | closed |
| 18 | M-STATE | closed |
| — | M-X | skipped (Parked，无 v8 场景) |

---

## v9 生成说明（owner 已批准原型 · 2026-08-30）

- 基线未覆盖：`clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v8.canvas.tsx` 仍在（上一版已批准基线）。
- 当前 chrome：`clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx`
- Cursor 可打开副本：`C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-opc-e2e-optimized-v9.canvas.tsx`
- Canvas runtime / render、NVDA、host-theme contrast、200% 真布局：**not-run**。
- **证据边界：** Owner 批准 v9 不是可用性、无障碍、backend、Gate、release、qualification 或 acceptance 证据。不等于实现授权。
- 本回合已改产品文档与设计目录当前 chrome 指针。未 git commit。未改 Control Plane。未跑 handbook 生成器（产品源已改、handbook 生成待后续实现/文档同步任务）。
- 下一步不是自动实现。等待 owner：开实现任务、同步 handbook、或驳回某模块再出 v10。

### plan-ceo-review · HOLD SCOPE

1. 单一主体：本机 Windows Owner。工作：把一件长期的事做成可打开、可核对的产出，数字员工按环执行。
2. 第一次成功：⑤「验收，进入今日」。⑤ 之前禁止日常决策包。
3. 留：三栏、五段创建、项目四子菜单、画布 HITL、空 Home 只创建。砍：打开对话开关、并列创建、样品分段器当主路径、四泳道、可见 CEO 轨。
4. 不能诚实的控件一律 Requires-backend / Requires-environment：导入、密钥交接、测试/联调、发出。无假 Connect / Install / 聊天 Approve。
5. 十分钟：空 Home 创建，或今日处理一件拍板。

| 项 | 结论 |
|---|---|
| Keep | 一人 Owner + 本机在线项目环；L1 Today / Projects / Knowledge；Settings 钉底；创建 ①→⑤；成员先选后看；HITL 在画布。 |
| Cut from chrome | 「打开对话」；空生命周期第二条创建；① 后把中间卡片画成已上线；未完成创建的日常决策包；默认选第一人；四泳道；Team/Inbox 一级。 |
| Park 2.1 | native mobile / pairing / E2E relay / 引擎商店 / 成员级预算 / 消费订阅 / X connector / 无项目云导入。 |
| Must-fix-before-prototype | 本文件 Must-fix 1–25、27 已打进 v9。剩余见「仍开放风险」。 |

### 按表面改了什么（映射模块）

| 表面 | 模块 | 改动与原因 |
|---|---|---|
| 壳 / 三栏 | M-SHELL、M-CHAT-CANVAS | 只 `empty-home` 藏聊天；去掉「打开对话」；知识可点进锁定页（不再假 `aria-disabled`）；创建期不露 L2 详情/成员/运行/产出。 |
| Context header | M-SHELL、M-STATE | State Lab 的 offline/unknown/error 改 Tag；默认仍「在线时工作」。 |
| 空 Home | M-EMPTY | 左对齐、22px、单 CTA；无示范项目。 |
| ① | M-CREATE-1 | 总预览为最后一项；Skill/MCP 步骤改业务名；进入 ② 只在总预览已确认后出现；brief 后标「原型未跑调研」。 |
| ② | M-CREATE-2 | 顺序门；轴上缺口；「本环留缺口」；总目标确认与进入 ③ 分成两步。 |
| ③ | M-CREATE-3 | 顺序门名单；当前条进度+标题；拒绝加入；回 ②；表只留岗位/模型/就位。 |
| ④ | M-CREATE-4 | 按环存结果；通过才下一环/末环进 ⑤；失败回 ②；离线禁开测；样品三钮仅 running。 |
| ⑤ | M-CREATE-5 | 不写死前四环完成；失败回 ④/②/③；离线禁联调；验收仍只 pass。 |
| 未完成 Today | M-TODAY-INCOMPLETE | 无决策包；继续按 `createGate`；画面写第几段。 |
| 日常 Today | M-TODAY | 项目行概览；无拍板收起决策包（`hasDecision=false` / Lab success）；「以后再说」**仍留决策包**并给停留回执；周期数字随 period；阻塞可点运行。今日角标仅 `hasDecision`。 |
| 项目列表 | M-PROJECTS | live 无「创建项目」；空列表回今日；creating 只草稿+继续；副本总预览 details。 |
| 已上线工作 | M-LIVE-PROJECT | L2=详情/成员/运行/产出；详情只读轴；末环验收先打开产出。 |
| 加人 | M-ADD-MEMBER | 空岗；可 pending；拒绝；加入后「初始化执行方式」。 |
| 成员配置 | M-MEMBER-CONFIG | 身份在详情头；Loop/Skill/MCP 进 `details`。 |
| HITL | M-HITL | 改窄→stale；empty 无包；unknown=说不清；拒绝留回执。 |
| 知识 | M-KNOWLEDGE | 导入四按钮仅 State Lab；离线禁导入；unknown 不写 0。 |
| 设置 | M-SETTINGS | 失败点名 SecretStore；成功标 Requires-backend；离线不能交接。 |
| 右栏 | M-CHAT-CANVAS | 已上线工作面=项目群；创建环=个人助手；无 Approve；@ 进草稿。 |
| State Lab | M-STATE | today 多态挂真 Today；create 按九态挂 ①–⑤；占位段不再声称已按真版式渲染。 |

### web-design-guidelines（只扫 v9）

来源：`https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md`（本轮拉取）。Canvas runtime 未跑，下列为源码扫描。

```text
## personal-20-opc-e2e-optimized-v9.canvas.tsx

✓ skip-link → #opc-main
✓ :focus-visible 3px ring；未见 outline:none 无替代
✓ prefers-reduced-motion 关掉动画；transition 仅 transform
✓ SyncedField name + autocomplete=off；label 包裹控件
✓ 图标点阵（wizard-dot）有 aria-label
✓ 对话框 overscroll-behavior: contain；Escape 关闭

personal-20-opc-e2e-optimized-v9.canvas.tsx:7751 - L1 用 button onClick 切场景，不是 <a href>（原型场景机；深链/中键打开不适用）
personal-20-opc-e2e-optimized-v9.canvas.tsx:4851 - 聊天 TextArea 无 name/autocomplete（Canvas kit）
personal-20-opc-e2e-optimized-v9.canvas.tsx:4567 - 密钥 type=password，易触发系统密码管理器；文案已写不回显，仍非真 SecretStore
personal-20-opc-e2e-optimized-v9.canvas.tsx:3050 - 费用硬编码 ¥6.40，未走 Intl.NumberFormat（样品；未知项已写「说不清」）
personal-20-opc-e2e-optimized-v9.canvas.tsx - Title Case 规则不适用中文 chrome
personal-20-opc-e2e-optimized-v9.canvas.tsx - URL 不同步场景（原型下拉，不是产品路由）
NVDA / 对比度 / 200% / Canvas render：not-run
```

未按 WIG 重画无关表面。Title Case、深链、Intl 货币不作为 v9 合并条件。

### 仍开放风险 / 未做的 Minor

- ③ 仍有紧凑三人表（岗位/模型/就位），不是五行全员编辑；完整配方仍在配置页。
- State Lab `members` / `runs` 九态仍可能共用一张名单/运行页（Major 18/20 未全部拆版式）。
- 壳层主路径（非 Lab）Context Tag 仍默认「在线时工作」——没有全局主机开关。
- 项目列表仍是卡片段，不是更紧凑的一行表（Major 16 部分）。
- ④/⑤ 主路径在 running 后仍有「演示结果」三钮（标了「不是真测」）。
- `confirmRoster` 仍会把未选模型的人标未加入；进入 ④ 要求全员就位。
- 加人 `onOpenConfig` 用 `added-${name}`，若名单合并到已有 id，可能打不开刚加入的人。
- 2.1 全部停放。语料漂移已随 v9 批准对齐产品/表 B 语料；workshop Q&A 与 handbook 生成页未改。

### 硬约束复查

无聊天 Approve；无 Install 商店；无引擎名 / Loop / Harness 默认 chrome；无可见 CEO 六步轨；无四泳道；无 Team/Inbox 一级；无 X；不默认第一人（成员管理 `selectedId` 初值 null）；⑤ 前无日常决策包；不把 ① 后卡片画成已上线。

### 批准记录

**Owner 于 2026-08-30 批准此 v9 原型。** 当前 chrome 已切到 v9。产品文档追上原型。这不是实现、Gate、handbook 生成或可用性证据。

若要驳回某模块并出 v10，请点名模块 ID（例如 M-CREATE-2、M-TODAY）。

