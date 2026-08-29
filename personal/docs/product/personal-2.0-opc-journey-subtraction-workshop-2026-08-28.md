# Personal 2.0 OPC journey-subtraction workshop record

- Date: 2026-08-28 / 2026-08-29
- Change class: `product-semantic`
- Kind: workshop record + scheme snapshot
- Not: Gate, usability, accessibility, backend, implementation, qualification,
  or acceptance evidence
- Source: owner-directed journey-subtraction workshop (Guided mode); every
  question prompt and owner answer below is **verbatim**, including custom
  text, option ids, and typos. Facilitation notes are labelled and are not
  owner speech.
- Canonical docs updated from this snapshot:
  - [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
  - [Product design](product-design.md)
  - [User journeys](user-journeys.md)
  - Minimal matching sentences in [Personal 2.0 scope](personal-2.0-scope.md)
- **Superseded as current chrome first by v5, then by v8, then by v9
  (2026-08-30).** This workshop record
  remains the verbatim Q&A + scheme snapshot. It does **not** own current
  chrome. Owner-approved current interaction:
  [personal-20-opc-e2e-optimized-v9](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx).
  v8 remains the prior approved baseline and must not be overwritten.
  The post-workshop canvas this file named is archived at
  [pre-v5-approval history](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/personal-20-opc-e2e.canvas.tsx).
- Owner-confirmed prototype amendment (2026-08-29; not rewritten below):
  create order is ① project → ② process → ③ members → ④ test → ⑤ joint.
  Workshop answers still record members then process as historical fact.
- Archived V2 and earlier e2e canvases (not current chrome):
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Scheme vs canvas: this workshop superseded V2 chrome (visible CEO six-step
  rail, Today four swimlanes as default blocks, X/Twitter as P0 hero, KNOW-04
  no-auto-memory, and permanent “Don’t ask again”). V2 is archived historical
  baseline, not current product chrome. v5 later superseded this workshop’s
  canvas as current chrome without changing the Q&A log. v8 later superseded
  v5 as current chrome (member select-then-configure; Project four work
  scenes), still without changing this Q&A log. v9 (2026-08-30) is current
  chrome, still without changing this Q&A log.

Owner close instruction (verbatim): 把本次对话中所有问答的问题和我的回答原文、项目最新方案保存为一份文档。同时可以改文档

---

## Part 1 — Complete Q&A log (original wording)

### Opening

**Q.** 本场约 6 条旅程、每条 3–5 个阶段、一轮一问。你要怎么开始？

**Options:** 1 Guided / 2 Context dump / 3 Best guess.

**Owner:** Guided.

---

### Journey A 首次打开 / 第一次成功

#### A 阶段1 Q1

**Q.** 首次打开 Today（还没有 Project），Owner 这一步要完成的唯一任务是什么？

**Options:** 用业务语言说明一个要办的事，点「创建 Project」（推荐） / 先连 Model Connection / 先看导览 / 默认进 X 样本.

**Owner:** 用业务语言说明一个要办的事，点「创建 Project」（推荐；对齐 V2 first-run 与 user-journeys §1）

#### A 阶段1 Q2

**Q.** 首次打开这一屏，默认只留哪 3 块？其余进「再披露一层 / 高级 / 不在 2.0」。

**Owner (custom, verbatim):** 首次打开应用，还没创建project，home页面中间展示页面为”创建project“按钮，右侧聊天对话框默认隐藏。用户点击创建project按钮后，跳转到项目栏目，进入创建页面的页面

#### A 阶段1 Q3

**Q.** 这一屏主按钮是哪一个？误点、拒绝、失败分别去哪？

**Owner:** 主按钮同上，但无 Project 前 Knowledge/Settings 不可点（灰掉或藏）

#### A 阶段1 Q4

**Q.** 首次打开这一屏：空、进行中、阻塞、未知、离线分别看到什么？

**Owner (verbatim):** C。但是取消展示daemon，默认用户是懂业务但不懂技术的小白，不要增加底层运行机制的展示和控制

**Option C (as offered in the workshop):** 空态仍锁 Knowledge，但 Settings 可进（只为连模型）；其余同推荐

#### A 阶段2 Q1

**Q.** 点「创建 Project」进入 Projects 创建页后，这一步用户要完成的唯一任务是什么？

**Owner (verbatim):** C。这个时候如果用户没有给ai助手配置provider，则在聊天框引导用户跳转设置界面配置provider，并绑定ai助手

**Option C (as offered in the workshop):** 中间是创建向导，右侧打开对话，用聊天来写业务描述

#### A 阶段2 Q2

**Q.** 创建页（描述这一步）默认只留哪 3 块？无 Provider 时怎么引导，但不讲底层？

**Owner:** 推荐：① 中间创建向导（当前步=业务描述）② 右侧助手对话（此时打开）③ 一个主行动：有模型则「继续」；无模型则只出现「去设置连接模型并绑定助手」。不露团队/能力/引擎。

#### A 阶段2 Q3

**Q.** 描述这一步：主按钮是哪一个？误点、拒绝、失败分别去哪？

**Owner (verbatim):** 有模型时，项目界面的右侧聊天栏直接显示正常ai助手聊天界面，聊天界面中引导用户描述项目情况、项目产出目标等，然后ai助手分析后给出建议的项目业务流程，以及各环节产出目标（子任务loop），并让用户确认。接着分析并联网调研GitHub、skillhub等，给出建议整个项目、每个环节推荐使用的skills、工具、mcp、知识库、外部工作环境（例如浏操作览器等）、文件权限等（harness）。接着确认项目的产出目标和执行周期（总任务loop）、产出成果保存形式等。完成后在项目界面中间展示项目设置卡片。中途离开则保存草稿，返回时接续恢复对话。连接不上则提示问题所在，让用户解决

#### A 决策 底层词

**Q.** 助手分析并调研之后，用户要点头确认的默认是哪些？底层词露到哪一层？

**Owner (verbatim):** B。如果有缺漏请你补充。

**Option B (as offered in the workshop):** 聊天里完整列出 Skill、工具、MCP、知识库、浏览器、文件权限、Harness，用户逐项确认

#### A 清单补全

**Q.** 按你选的「聊天里逐项确认」+ 我补缺漏。默认确认清单用哪份？

**Owner:** B + 自动/批准、触发、费用、权利、总预览，但 Harness 改叫「执行方式」不出现 Harness 一词

#### A 阶段2 Q4

**Q.** 创建过程（描述→分析→逐项确认）空、进行中、阻塞、未知、离线、成功分别看到什么？

**Owner:** 推荐：空=聊天引导该说什么。进行中=正在分析/调研，可离开，草稿接续。阻塞=缺模型去设置；缺批准就停在那一项。未知=冲突保留，不写成已就绪。离线=仍可描述，不能联网调研。成功=中间出现项目设置卡片，但未过总预览则未开始。

#### A aha

**Q.** 创建走完、中间已有项目设置卡片之后，旅程 A 的「第一次成功」以哪一次为准？

**Owner (verbatim):** 1、总预览确认后，项目创建完，只是项目初始化的旅程结束。2、接下来还要创建/分配角色，创建团队成员，完成项目成员初始化的旅程。3、再接下来确认业务流程、项目总目标、成员的子任务目标/阶段性交付成果的内容、成员的子任务输入内容（知识库内容或上一环节的交付成果）、成员子任务的执行环境例如工具/技能/mcp/文档范围/权限等（harness，请你删减或补充）、项目总任务的执行环境例如文档范等、成员子任务的触发方式/loop、项目的触发方式/loop，完成业务流程初始化的旅程。4、接下来针对业务流程上的每一个环节进行测试调试，直到产出预期的子任务目标，完成所有业务环节的测试调试酸味完成项目业务环节和初始化。5、最后针对业务流程进行全环节联合调试，直到产出预期的项目总目标成果，则酸味完成整个创建项目的旅程

#### A 创建深度

**Q.** 创建收成 5 段之后：2.0 首次打开必须做完哪几段，才允许进入日常 Today？

**Owner:** 五段全是创建向导；联合调通并交出总成果前，不算创建完成，也不进日常 Today

#### A 壳锁定

**Q.** 五段向导做完之前，左栏 Today / Projects / Knowledge / Settings 分别能不能进？

**Owner:** 推荐：⑤完成前 Today 不是日常决策包（只显示「继续未完成的创建」）。Knowledge 到③需要输入时才打开（仅当前草稿）。Settings 可连模型。Projects 只露出这个未完成创建。

#### A 成员 Q2

**Q.** ② 成员初始化这一屏默认只留哪 3 块？

**Owner:** 推荐：① 岗位名单（职责/交出什么）② 右侧对话 ③ 主行动「确认这个班子」。每人模型必选。Skill/MCP 放到③再逐项。

#### A 成员 Q3–Q4

**Q.** ② 主按钮、误点/拒绝/失败、以及空/进行中/阻塞/离线，用哪套？

**Owner:** 推荐：主按钮「确认这个班子」→ ③流程确认。某岗拒绝=不加入。某人没选模型=该人待定，不静默绑定。离开=草稿接续。空=还没有岗。进行中=助手在建议岗位。阻塞=缺模型去设置。离线=可改职责，不能联网找岗位方案。

#### A 流程 Q2

**Q.** ③ 业务流程初始化：这一屏默认只留哪 3 块？

**Owner:** 推荐：① 一条流程轴 ② 一次只打开一环的确认 ③ 右侧对话。项目总目标+总周期钉在轴上方一行。不一次铺开所有人的 MCP/文件树。

#### A 流程 Q3–Q4

**Q.** ③ 主按钮和空/进行中/阻塞/未知/离线，用哪套？

**Owner:** 推荐：主按钮「确认这一环」→ 下一环；最后一环后确认总目标+项目触发。拒绝=留在这一环。权限过大=收窄再问。离线=能改流程，不能联网补执行方式。空=还没有环节。进行中=助手在填建议。未知=缺口留在轴上，不标已就绪。

#### A ④测试

**Q.** ④ 每环测试调试：默认 3 块、主按钮、失败/空/进行中/阻塞/未知/离线，用哪套？

**Owner:** 推荐三块：① 正在测哪一环 ② 这次交出的可打开结果+是否达标 ③ 主按钮「通过，下一环」；不通过=回③改这一环。进行中=正在跑这一环。阻塞=缺权限/缺输入。未知=结果无法核对，不能点通过。离线=不能开测。空=还没开始测。不展示进程/引擎。

#### A ⑤联合调试

**Q.** ⑤ 全环节联合调试（第一次成功）：默认 3 块、主按钮、失败去哪、状态怎么露？

**Owner:** 推荐三块：① 全流程走到哪 ② 总成果可打开+核对状态 ③ 主按钮「验收，进入 Today」。不通过=指出失败环节并回④/③。未知/核对不上=不能验收。离线=不能联合调试。验收前 Today 仍不是日常包。不出现假发布按钮。

---

### Journey B Today

#### B Q1

**Q.** 联合调试验收之后，日常打开 Today，Owner 这一步要完成的唯一任务是什么？

**Owner (verbatim):** 看清并处理需要你拍板的事件，同时展示已上线项目的运行情况概览（当前状态、今日已完整执行的次数、当前任务环节、当前业务/子任务已经持续的时间），还有简单展示当前项目情况（创建的项目总数/已上线的项目数量/发生阻塞的项目数量）。点击某个已上线项目的运行情况，可以查看详情，展示每个环节的名称和负责成员，同时展示今日每个环节子任务的完整执行的次数、失败的次数、平均执行时长、执行成功率。再today页面可以点击切换查看今日/本周/本月的统计。在右侧与ai助手对话用自然语言询问所有项目的运行细节，ai助手都可以提供具体的运行数据和分析结论

#### B Q2 first

**Q.** 日常 Today 若只能留 3 块...

**Owner (verbatim):** 请你说明四泳道是什么，然后再问一次本问题让我选择

**Facilitator note (not owner speech):** four swimlanes as previously offered were Needs you / Can continue / Unknown / Missed. The question was then re-asked.

#### B Q2 retry

**Q.** 说明之后再选：日常 Today 默认只留哪 3 块？

**Owner:** ① 决策包 ② 运行概览+项目计数+周期切换 ③ 聊天。四泳道不单独成块（并进概览）

#### B Q3

**Q.** Today 主按钮是哪一个？点统计、拒绝、失败分别去哪？聊天能不能批准？

**Owner:** 推荐：主按钮只在决策包上（去处理这一件拍板）。点运行概览某一项目→该项目详情。周期切换是次要。聊天不能批准。误点统计不能发布。拒绝/以后再说=留在 Today，这件仍待办。

#### B Q4

**Q.** 日常 Today：空、进行中、阻塞、未知、离线分别看到什么？

**Owner:** 推荐：空=没有待拍板且没有上线项目（创建未完成则只显示继续创建）。进行中=概览在刷新，决策包仍可点。阻塞=计数里能看到阻塞项目，点进去处理。未知=那一行写说不清，费用未知不写0。离线=展示上次已知概览并标明已过时，不能当成当前成功。

---

### Journey C Project

#### C Q1

**Q.** 从 Today 点进一个已上线 Project，Owner 这一步要完成的唯一任务是什么？

**Owner:** 推荐：看清这个项目现在停在哪一步，只做这一步该做的事。

#### C Q2 first

**Q.** 已上线 Project 默认只留哪 3 块？CEO 循环怎么出现，X 清单要不要并掉？

**Owner (verbatim):** 进一步说明ceo循环和x清单是什么，然后再问一次本问题让我决策

**Facilitator note (not owner speech):** CEO 循环 = Ingest → Decide → Authorize → Execute → Verify → Report as a visible workbench rail. X 清单 = X/Twitter content-operation as the first P0 acceptance / default or demo project surface. The question was then re-asked.

#### C Q2 retry

**Q.** 说明之后再选：已上线 Project 默认 3 块里，钟留几条？

**Owner (verbatim):** 取消将x社交帐号运营作为p0发布内容，p0只发布完整能力，不发布默认或者示范项目

#### C Q2 generic

**Q.** 没有示范项目、也没有 X 作为 P0 之后：已上线 Project 默认只留哪 3 块？

**Owner:** 推荐：① CEO 状态一行 ② 当前步工作面 ③ 项目群对话。不要第二条业务钟，不要示范项目。经营统计从 Today 点进来或放在「汇报」步。

**Workshop note:** this answer was then superseded by cancelling visible CEO six steps (see C 六步工作面 and C 主钟决策).

#### C 六步工作面

**Q.** CEO 六步的「当前步工作面」默认各看什么？

**Owner (verbatim):** 取消ceo六步，改为按业务流程展示某环节的执行情况/该环节成员，需要用户授权、检验核对的内容标注并展示在环节页面中

#### C 主钟决策

**Q.** 已上线 Project：用业务流程当主界面，还是保留可见 CEO 六步？

**Owner:** 确认：前台只按业务流程展示（环节状态、谁在做、要你授权/核对的钉在该环节）。CEO 六步不再画成顶栏。授权仍只在画布、聊天不能批。

#### C 环节 Q2–Q3

**Q.** 按业务流程当主界面：Project 默认 3 块、主按钮放哪？

**Owner:** 推荐：① 业务流程轴 ② 这一环：状态、负责成员、要你授权/核对的标注与内容 ③ 项目群。需要你出手时主按钮在②里。聊天不能批。点轴切换环节。

#### C 环节 Q4

**Q.** 环节页：空、进行中、阻塞、未知、离线、核对失败分别看到什么？

**Owner:** 推荐：空=这一环还没开始，说明缺什么。进行中=谁在做、已持续多久。阻塞=要你授权/缺输入，主按钮去处理。未知=不能标完成。离线=上次状态标明已过时。核对失败=留在这一环，不跳去下一环。

---

### Journey D 组团队

#### D Q1

**Q.** 日常（创建五段已完成）要给项目加人时，唯一任务是什么？

**Owner:** 推荐：给这个已上线项目补一个岗位（要做什么、交出什么），确认后再进执行方式。不是先装 MCP，也不是先看引擎。

#### D Q2

**Q.** 日常加人这一屏默认只留哪 3 块？

**Owner:** 推荐：① 现有班子 ② 右侧对话建议新岗位 ③ 主行动「确认加入」。模型必选。Skill/MCP/文件权限放在「执行方式」再披露。

#### D Q3–Q4

**Q.** 日常加人：主按钮、拒绝/失败去哪、空/进行中/离线怎么露？

**Owner:** 推荐：主按钮「确认加入」。拒绝=不加入。没模型=该人待定，去设置连模型。加入后若改流程/权限=要再批准，不静默扩权。离开=草稿。空=还没有这个岗。进行中=正在建议岗位。离线=能写职责，不能联网搜方案。

---

### Journey E HITL

#### E Q1

**Q.** 对外动作时，Owner 的唯一任务是什么？

**Owner:** 提供「以后别再问」一次授权永久对外

#### E 决策

**Q.** 你选了「以后别再问」。这和现行「每次对外必须画布预览、禁止永久沉默授权」冲突。对外授权用哪条？

**Owner:** 折中：可设「本周同一类对外不再问」，到期失效，设置里可收回

#### E Q2

**Q.** 授权画布默认只留哪 3 块？

**Owner:** 推荐：① 将做什么 ② 完整预览/差异 ③ 三个行动：批准 / 改窄 / 拒绝。可另有「本周此类不再问」。聊天只给链接，无批准。执行中第四个行动：停。

#### E Q3–Q4

**Q.** 授权：主按钮、拒绝/失败去哪；空/进行中/未知/离线怎么露？

**Owner:** 推荐：主按钮「批准」。拒绝=不发出、回到环节页。改窄=改完重新预览。过期预览不能批。失败=说清停在哪，无假成功。未知对外结果=禁止盲着重试。离线=不能批准对外。空=没有待批。进行中=可停。成功=回执在环节页可打开。

---

### Journey F 收工 / Knowledge / Settings

#### F Q1

**Q.** 一轮做完后「核对证据与收工」，唯一任务是什么？

**Owner:** 推荐：打开成果，看核对是否通过，然后回到 Today。不是去 Inbox 清队列，也不是去看引擎。

#### F Q2–Q3

**Q.** 核对收工这一屏：默认 3 块、主按钮、失败去哪？Inbox 是否进左栏？

**Owner:** 推荐：① 可打开的成果 ② 核对通过/失败/说不清 ③ 主按钮「验收，回 Today」。失败留在环节。聊天不能当验收。无 Inbox 一级入口。

#### F Knowledge/Settings

**Q.** 收工之后回 Knowledge / Settings：各默认 3 块？Inbox 仍不进左栏？

**Owner (verbatim):** A的基础上，memory自动把聊天变记忆。知识库使用obsidian作为底座，ai助手使用GitHub上openai的codex项目的记忆架构作为底座

**Option A (as offered in the workshop):** Knowledge 三块：当前项目资料 / 为什么用了这段 / 导入。Settings 三块：连接模型 / 本周不再问的收回 / 通知恢复。无账单、无引擎商店、无 Inbox。无 Project 时 Knowledge 仍锁。

#### F 记忆底座

**Q.** 自动记忆 + Obsidian 底座 + Codex 记忆：按产品行为记，还是按必须装应用 / 必须集成 Codex 记？

**Owner:** 2.0 产品依赖并集成 GitHub 上的 OpenAI Codex 作为记忆底座（这是架构选型，将写入范围）

#### F 状态 Q4

**Q.** Knowledge / Settings / 核对收工：空、失败、离线怎么露？有没有假按钮、密钥进聊天？

**Owner:** 推荐：Knowledge 空=说明还没资料+导入。检索中=进行中。解析失败=保留原件可重试。Settings 空=还没连模型，去连。密钥永不回显。离线=知识只读上次索引，不能导入云端。核对说不清=不能验收。无假 Connect/Install。

---

### Follow-up after journeys

#### Owner item 1

**Owner (verbatim):** 1、设置页面中provider提供主流大模型配置（下拉选择）和自定义配置，用户可以输入密钥（以我为准，可以调整规则或者技术手段解决os-core的约束）。

#### Owner item 2

**Owner (verbatim):** 2、在已有可上线的项目的情况下，提供一键复制项目的选项，用户可以在成熟项目的基础上改动然后创建新项目。

#### Copy project Q

**Q.** 已有可上线项目时，「一键复制」默认复制什么，新项目从哪一步接着改？

**Owner:** 推荐：复制目标、流程轴、岗位/成员定义、执行方式、触发、成果形式。不复制密钥、进行中的任务、对外回执、「本周不再问」。落到未激活草稿（名称带副本），你改完走总预览再上线；④⑤可抽检或跳过。入口在 Projects 列表和项目页。

---

## Part 2 — Latest confirmed product scheme (Keep / Cut / Park)

This section is the scheme snapshot applied to the canonical product docs. It
synthesizes the Q&A; it does not replace Part 1.

### Copy principles

懂业务的小白。默认不出现 daemon / DSH / Pi / Harness / Loop。对外用「执行方式、周期、触发」。密钥永不进聊天。未知费用和未知结果不写 0、不标成功。无假按钮。

Daemon remains the sole authority writer (A1) but is not displayed or
controlled as product chrome for this user.

### Keep

- **Empty Home:** center only Create Project; right chat hidden; Settings
  allowed for models; Knowledge locked until process-init (③) needs input
  (current draft only); Today during incomplete create only shows continue-create.
- **Create jump:** Create Project goes to the Projects create page. With a
  model, right chat opens as the normal assistant. Without a model, chat only
  guides to Settings to bind the assistant. Leave saves draft and resumes.
  Connection failure explains the problem.
- **Confirm list (chat, item-by-item):** business process / per-stage outputs /
  cycle / save format + Skill / tools / MCP / knowledge / env / file
  permissions + auto-vs-approve (including external), triggers, cost, source
  rights, launch preview. Harness labelled **执行方式**; the word Harness does
  not appear. Secrets never in chat. No silent model bind. Inactive until
  总预览.
- **Create depth:** all five stages required before daily Today:
  ① project init ② member init ③ process init ④ per-stage test until expected
  sub-output ⑤ joint debug until expected overall outcome then 验收.
  Aha = ⑤ accept. Incomplete create is not daily Today.
- **Shell during wizard:** ⑤ complete 前 Today is not the daily decision
  packet (continue-create only). Knowledge opens at ③ when input is needed
  (current draft only). Settings may connect models. Projects exposes only
  this incomplete create.
- **Member init and returning add-member:** roster + chat + confirm; model
  required; Skill/MCP/file permissions in 执行方式 disclosure; no silent
  grant; members not shared across projects.
- **Process UI:** one axis, one stage at a time; tests require openable
  artifacts; unknown cannot pass/accept; no fake publish on accept.
- **Today (after ⑤):** ① decision packet ② live-project run overview +
  counts (created / live / blocked) + today/week/month toggle ③ assistant.
  Click a live project for stage / member / count / fail / avg / success
  detail. Primary CTA only on the decision packet. Chat can query run data,
  cannot approve. Four swimlanes are not default blocks (semantics may merge
  into the overview). Unknown cost never 0. Offline shows last-known aged
  overview.
- **Live Project:** ① business-process axis ② this stage: status, member,
  auth/verify marks ③ project group. No visible CEO six-step top rail (CEO
  remains backend discipline: canvas HITL + independent verify). Chat cannot
  approve.
- **HITL:** canvas preview (what / full preview / approve-narrow-reject);
  stop while executing; stale preview cannot confirm; unknown external
  outcome no blind retry; offline cannot approve external. Timeboxed
  「本周同一类对外不再问」, expires, revocable in Settings. No permanent
  don’t-ask-again. No chat approve.
- **Close-out:** openable artifact + verify state + 验收 back to Today. No
  Inbox L1.
- **Knowledge:** project files, why-this-fragment, import. Failed parse keeps
  original. Chat auto-admits to inspectable / forgettable Memory (overrides
  old KNOW-04). Knowledge base = Obsidian as 底座; 2.0 does not require
  installing the Obsidian app unless later specified.
- **Memory architecture:** 2.0 depends on and integrates GitHub OpenAI Codex
  as the assistant memory base (explicit owner architecture-in-scope). This
  is memory architecture, not a user-facing Codex execution-engine store.
- **Settings:** mainstream provider dropdown + custom (URL / compat / model);
  user enters keys (owner overrides any product rule that would block key
  entry). Technical means for os-core / A5: one-way SecretStore; UI shows
  connected / failed, never raw secret in DOM / chat / git. Also: revoke
  timeboxed skips; notify / recovery. No billing, no agent store.
- **Copy project:** from live / launchable projects; entries on Projects list
  and project page. Copy goals, process axis, role / member defs, 执行方式,
  triggers, output forms. Do not copy secrets, in-flight tasks, external
  receipts, timeboxed skips. Lands as inactive draft named 副本; owner edits
  then 总预览 to go live; stages ④⑤ optional spot-check or skip. Does not
  start from create step ①.

### Cut

- Teaching bullets, CEO lesson, demo / default project, daemon / runtime
  chrome on first-run Home.
- Visible CEO six-step top rail as product chrome.
- Four swimlanes as default Today blocks.
- X / Twitter social-account operations as P0 release content.
- Permanent 「以后别再问」 / Don’t ask again.
- Chat Approve, chat 验收, Inbox as a first-level destination.
- Fake Connect / Install / publish buttons.
- Silent model bind; silent permission expansion; unknown cost as 0;
  unknown result as success.
- The words daemon / DSH / Pi / Harness / Loop on the 小白 default UI.
- Second business clock on the live Project surface.

### Park

- X / Twitter (and other industry connectors) as later, non-P0 work.
- Native mobile / pairing / cloud 24/7 (already 2.1).
- Installing the Obsidian desktop app (not required unless later specified).
- Codex as a user-selectable Member execution engine / engine store (out);
  Codex as memory-architecture base is Keep, not Park.
- Visible CEO loop as UI chrome (parked); CEO remains backend discipline.

### P0 release content

Complete capabilities only. No default or demo projects. No X/Twitter as P0
hero.

### Mapping to `user-journeys.md` (no new taxonomy)

| Workshop journey | Canonical section |
|---|---|
| A ①–⑤ create | §1 Create and activate the first Project |
| B daily Today | §2 Use Today |
| C live Project process axis | §3 Use the Project group and operating canvas |
| D add member | §4 Add a Role and Project Member |
| E HITL | §6 Resolve attention, approval, and recovery |
| F close-out / Knowledge / Settings | §5 Knowledge; §8 Model Connections; close-out in §3/§6 |
| Copy project | §1 (copy path), not a new journey id |
| Parked X | §10 retained as parked scenario, not P0 |

---

## Evidence boundary

This file preserves workshop wording and the scheme snapshot. It does not
implement, qualify, or Gate anything. Architecture, ADR, formal-plan, and
handbook generated pages were not updated in this delivery.
