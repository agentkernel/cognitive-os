# Personal 2.0 OPC v9 → daemon implementation mapping

# 个人 2.0 OPC v9 到 daemon 实现映射

- Status: **informative** / 非实现 / 非 support / 非 Gate
- Change class: owner-directed architecture mapping; no formal `P*-T*` claim
- Mapped chrome: owner-approved v9
  (`clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx`)
- Frozen current `/ui/`: `clients/pc/web/` served same-origin by the Personal
  daemon; Vite is not the product origin
- HEAD at writing (mapping first draft): `ed82bd1744e2ca71b71c63bfdcfeadaaa5c21311`
- 决议修订 HEAD: `ed82bd1744e2ca71b71c63bfdcfeadaaa5c21311`
  （2026-08-30 Owner 决议落盘；相对本映射初稿；本轮无新 Git commit）
- Lease: `lease/personal/DOC-PERSONAL-2.0-ARCH/v9-implementation-mapping`
- Writable this window:
  this file; `personal/docs/architecture/README.md` (Chapters pointer row
  only); `personal/handbook/_meta/source-map.json`; mapped bilingual
  handbook pages + fingerprints; `docs/plan/PARALLEL-LANES.md` (this
  lease row only, coordination)
- Evaluation routing: **OFF**. `PERSONAL-PERF-EVAL-015` closed.
  P11 implementation paused. `P11-T01` done; `P11-T02..T15` not-started /
  unclaimed. **本窗口不领取任何 P11-T***。
- Claim ceiling: `hypothesis`. Owner 批准本文 ≠ 后端已存在。Owner 批准 v9 ≠
  usability / a11y / backend / Gate / release / qualification / acceptance.
  Owner 批准映射 ≠ 已领取实现、≠ 已改 `router.tsx`、≠ 已改正式计划正文。

`personal/docs/architecture/README.md` 已在 source-map 规则
`personal-2-baseline`。**本映射文件** 在规则
`personal-2-opc-v9-implementation-mapping`。按
[docs-sync-contract](../../../docs/standards/docs-sync-contract.md) §2
同步 handbook（双语手写页 + 指纹；`generated: true` 页只经
`node tools/src/generate-handbook.mjs`，禁止手改正文）。

现网 architecture 章（`system-architecture.md`、`web-ui-architecture.md`、
`project-role-employee.md` 等）仍可能写 Team / Inbox 一级、Installed Agent、
Employee。产品旅程正文 **本窗口不改**。architecture / handbook 用词对账
按 Owner 决议 **完成后**（实现/任务收口后）再做，不在本窗口批量替换。

---

## Owner 决议（2026-08-30）

Owner 已确认下列七条。本窗口原样落盘。**已决议。** 未决 = 0（handbook
生成细节不再提问）。本窗口不领取 P11、不改正式计划正文、不改产品旅程、
不改 `clients/pc/web`、不改 core 合同、不 commit。

| # | 原问题 | Owner 答复（可执行） |
|---|---|---|
| 1 | 权威对象英文 id：Employee 还是 Member Runtime？架构/handbook 何时对账？ | **Employee**。**完成后对账**（实现/任务收口后再做 architecture / handbook 用词对账）。本窗口不改产品旅程。v9 chrome 仍可能写 Member Runtime：权威对象 id = Employee；产品表面 Member Runtime 用语保留到完成后对账。 |
| 2 与 3 | P11-T13 Team/Inbox 一级 vs v9；P11-T12 成员/Task budget stop vs 2.1 | **先冻结当前所有未开发开发，以 v9 为准，重新开发。** 不领取、不开始任何 P11 实现。正式计划卡（`PERSONAL-DEVELOPMENT-PLAN.md` 的 P11-T02–T15 正文）**本窗口不改写**；本文将其标为 **frozen / 与 v9 冲突时以 v9 chrome 为准**。「重新开发」= 将来 owner 解冻 P11 实现时，按已批准 v9 chrome 重新切分/实现，而不是按任务卡里的 Team/Inbox 一级或成员级 budget stop 当前 chrome 去实现。**不要**把「重新开发」理解成现在去改 `router.tsx` 或开 frontend。v9 为准的当前 chrome：Today / Projects / Knowledge + 底栏 Settings；Team、Inbox 不是一级；成员级预算不在当前 chrome（2.1 / Deferred）；HITL 见第 4 条；state-lab 见第 5 条。 |
| 4 | HITL：独立 `#/hitl/:approvalId`，还是仅项目中心画布 + Today 深链？ | **后者**。HITL **只**在项目中心画布；Today 用深链进入该画布。建议 hash **不要**把独立 `#/hitl/:approvalId` 写成产品一级或默认路由。若内部深链需要稳定 id，记为项目内画布锚点（例如 `#/projects/:id` 上的 HITL 表面），不是独立 Inbox/HITL 一级页。 |
| 5 | state-lab：仅开发构建，还是 Settings 高级且默认隐藏？ | **后者**。放在 **Settings 高级**，**默认隐藏**。不是纯开发构建开关；也不是一级导航。 |
| 6 | Conversation 新 private version：T05 内做，还是先独立 Lane-CTR？ | Owner 交本文件判断。见下方 **§ Owner 决议第 6 条判断**。 |
| 7 | 本映射文件是否纳入 handbook source-map？ | **纳入**。规则 id `personal-2-opc-v9-implementation-mapping`。按 docs-sync-contract 生成/刷新双语 handbook 页与指纹。禁止只加 map 条目却不刷新页面。`generated: true` 页禁止手改。 |

### Owner 决议第 6 条判断（本文件）

**判断：默认在 P11-T05 内做新的 Personal private projection version（禁止重解释 `cognitiveos.personal.conversation-projection/0.1`）。不要先开独立 Lane-CTR。**

**理由：** [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md) 已把该信封定为 Personal **私有**投影，不是 core 公共 schema。Lane-CTR 只用于公共机器合同语义变更。T05 的 Conversation archive 正是这个私有形态的演进点；重解释 0.1 是禁止的，所以必须新 private version，而不是改 0.1 语义。读 ADR-0058 后无硬冲突：§2 写明 later private revision 使用新 identifier，且 MUST NOT silently coerce older client；§5–§6 要求 conversation/history 走新 Personal-private envelope，不得给 Core `ConversationBinding` 加 transcript 字段。

**例外：** 若 T05 发现必须改 **core 公共** conversation schema 才能表达 archive，把那一块标「须另走 Lane-CTR」，T05 其余私有投影仍继续。本窗口不改 `core/specs`。

---

## 判定词 / Status vocabulary

只用 architecture README 与 design-matrix 11 的词。本文件把 README 的
**Now** 与矩阵的 **Current** 视为同义。

| Label | Meaning |
|---|---|
| **Now** / **Current** | 仓库已落地的实现，限于其已记录平台与证据边界 |
| **2.0 target** | 已采纳的 Windows OPC 组成；文档批准 ≠ 已实现或已 support |
| **Reusable foundation** | 已有原语可支撑目标，但本身不完成目标 |
| **Requires-backend** | 需要新的或变更的 daemon / client / adapter / 数据行为 |
| **Requires-environment** | 缺少合格 Windows-native 或外部资格环境 |
| **Deferred** | 明确不在 2.0 成功路径内 |
| **Forbidden** | 违反产品边界或 A1–A8 |

禁止把 `/work` 改名为 Projects 却不做 Project 聚合，然后标 **Current**。

---

## 1. 范围与非声明 / Scope and non-claims

### 1.1 本文做什么

把 owner-approved **v9 Scene** 映射到：

- 现网 daemon-served `/ui/`（hash 路由、七空间 + session）；
- 现网 HTTP / 投影（management vs task、session gate、P7-T05 冻结 inventory）；
- core 1.0.0 合同约束与公理；
- Personal 2.0 scope §4 能力缺口；
- 正式计划 `P11-T02..T15` 的建议切分（**不领取**）。

### 1.2 本文不做什么

- 不领取、不启动、不实现任何 `P*-T*`（含 `P11-T02`）。
- 不改 `PROGRESS.md` 任务 / Remaining / campaign。
- 不 commit / push / PR。
- 不改 `clients/pc/web/`、core 合同、v8/v9 canvas、产品旅程正文、
  handbook 生成页、`00-maintenance-index.md`。
- 不把 v9 写成已交付 `/ui/`，不把 Linux 1.0 六族写成 OPC IA。
- 不把本地 / 文档 / ordinary CI 升为 Gate、release、Profile、support、
  B01、Agent-benefit。

### 1.3 权威顺序

1. [AXIOMS.md](../../../docs/governance/AXIOMS.md) A1–A8（只引用，不粘贴全文）
2. [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
   产品 / 运行时 / 记忆边界
3. [ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md) 现网 Web UI 栈
4. [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)
   `conversation-projection/0.1` **禁止重解释**
5. [personal-2.0-scope.md](../product/personal-2.0-scope.md) §3.1 / §4 / §5 / §7 / §8
6. v9 canvas + [00-maintenance-index.md](../../../clients/docs/design/opc-2.0/00-maintenance-index.md) 表 A
7. [11-design-to-code-and-backend-matrix.md](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md)
   （引用，不复制成第三张总表）
8. 现网 `clients/pc/web/src/router.tsx` +
   [web-ui-route-inventory.json](web-ui-route-inventory.json)
   （P7-T05 冻结，**不是 OPC 合同**）

### 1.4 产品 chrome 事实（v9，不改旅程）

- 一级：**Today / Projects / Knowledge** + 底栏 **Settings**。
- **Team、Inbox 不是一级。**
- 聊天无 Approve；HITL 在中心画布；无假按钮。
- 对象名 chrome：**Member Runtime**（v9 产品表面用语）。权威对象英文
  id = **Employee**。产品表面 Member Runtime 用语保留到 **完成后对账**。
  本窗口不改产品旅程。实现期不得另发明第三套权威对象 id。

---

## 2. 外部对照表 / External contrast

每行：他们怎么做 / 为什么不能照搬 / 可借鉴窄点。禁止「改成 heartbeat 写权威」。

### 2.1 DeepSeek Harness（必做）

- 官方仓：<https://github.com/deepseek-ai/deepseek-harness>
  （`master`，MIT，TypeScript，topics: `ai-agents` / `cordis` / `dsh` /
  `dsh-plugin`）。口号 *Everything is a Plugin*。
- README（本窗口 `raw` **pass**）：开源 agent harness，Cordis 插件树；
  `npx @deepseek-ai/dsh web` 默认 `http://127.0.0.1:3080`；profiles
  `web` / `headless` / `sdk` / `sdk-minimal` / `acp`。
- 架构文档（本窗口 `docs/architecture.md` **pass**）：插件贡献
  session / tools / agent-loop / sandbox / UI；`dsh-base` 含 model /
  tools / persistence / sandbox / approval / credentials；`dsh-web-app`
  是**他们自己的浏览器应用**。
- Docs 站点 <https://deepseek-harness.github.io/deepseek-harness/> 本窗口
  **not-run**（HTTP 500）。<https://deepseek.com/harness> 本窗口
  **not-run**（timeout）。

| 维度 | 他们怎么做 | 为什么不能照搬 | 可借鉴窄点 |
|---|---|---|---|
| 产品身份 | 独立开源 harness + 自带 Web UI / 插件发现（`dsh-plugin`） | **不是**本仓 `core/crates/cognitive-kernel` 的确定性 harness 原语。DSH = **隐藏默认 Member 执行引擎**。禁止把 DSH Web UI / 插件商店 / Harness 切换 / in-process 放进默认 chrome | 已有 Path B + `cognitive dsh web` 是**独立诊断面**，保持与 `/ui/` 分离 |
| 权威 | Cordis 插件可替换 loop / tools / session；session log 是模型可见源 | harness **不能**成为 authority writer（A1）。session 事件 ≠ Task 完成（A4） | 隔离 child + 有界 stdio broker 的托管形态（`P11-T07`） |
| 批准 | harness 自带 approval policy / sandbox 插件 | 聊天无 Approve；批准只在 OPC 画布；对外 mutation 必须 persist-before-dispatch（A3） | 无（不要把 harness 批准当产品 HITL） |
| 供给 | `dsh plugin`、live patch、Web UI 商店发现 | 禁止 engine store / Installed Agent 商店 / 用户切换 Harness | 精确审计制品 + update/rollback 诊断（高级，非默认 chrome） |

### 2.2 Paperclip（必做）

- 唯一规范仓：<https://github.com/paperclipai/paperclip> ；站
  <https://paperclip.ing> ；`llms.txt` 本窗口 **pass**。
- `docs/start/architecture.md` 本窗口 **not-run**（404，`main` 与
  `docs.paperclip.ing/start/architecture` 皆 404）。未克隆。
- **禁止** <https://github.com/getpaperclipai/paperclip>
  （仿冒 / 木马克隆）。不得 clone、不得 `npm` 装该包、不得当文档源。

| 维度 | 他们怎么做 | 为什么不能照搬 | 可借鉴窄点 |
|---|---|---|---|
| 组织 | company 多租户；一部署多家公司；CEO 雇团队 | RFC-0001 是企业多租户。2.0 **禁止 Tenant / Company chrome**。单 Owner | 无 |
| 执行 | heartbeat 唤醒任意外部 agent（Claude / Codex / Cursor / Hermes / OpenClaw / Pi…） | 外部 coding agent 当执行面不能搬。2.0 只资格化 **托管 DSH**（Member）+ **隐藏 Pi**（Assistant）。Codex 只作助手 **Memory 架构参考**，不是可选引擎 | 无 |
| 预算 | 每 agent 月度硬停 | 成员级预算是 **2.1 / 非当前 chrome**。A1 下预算停必须是 daemon 权威，不是 agent 自停 | 未知费用永不显示为 0（诚实归因） |
| 治理 | Board 批雇佣 / 战略；ticket 即对话 | 无 Company、无 Board chrome。HITL 在画布，聊天只宣布 | 预览 → 确认 → 回执 的可见因果（仍须走 Intent/Effect） |

### 2.3 OpenAI Codex（必做）

- 规范仓：<https://github.com/openai/codex>（Rust，Apache-2.0）。
  README 本窗口 **pass**：本地 CLI / 安装器 / ChatGPT 登录。
  **不是** 2021 模型页、**不是** Copilot。
- `developers.openai.com/codex` 本窗口 **not-run**（timeout）。
- scope：助手 **Memory 架构参考**。不是可选 Member 引擎，不是 engine store。

| 列 | 他们怎么做 | 为什么不能照搬 | 可借鉴窄点 |
|---|---|---|---|
| 记忆 / 会话持久 | 本地 CLI 会话、计划级记忆、工作区上下文 | 浏览器 / CLI 不得写 SQLite 权威。Personal 拥有 Conversation archive；默认在 T05 内做新 private version，禁止重解释 `0.1`；仅 core 公共 schema 才 Lane-CTR | 会话与可检查 Memory 分层；自动承认可忘 |
| 执行环 / sandbox / MCP | 终端 agent、工具、沙箱、MCP | 不得把 Codex 当 Member 执行引擎或商店货架。完成 ≠ 模型文本 / 工具 receipt | 助手只产 candidate；执行走 daemon 托管 DSH |

### 2.4 可选（各五行；GitHub 落地页 pass，深文档未拉）

| 项目 | 他们怎么做 | 为什么不能照搬 | 可借鉴窄点 |
|---|---|---|---|
| Claude Code <https://github.com/anthropics/claude-code> | 终端 agentic coding；理解仓、跑任务、git | 不是 Member 引擎、不是 engine store、不是聊天批准面 | 无（2.0 禁止可选执行引擎） |
| SWE-agent <https://github.com/SWE-agent/SWE-agent> | issue → 自动修；也可用于攻防 / 竞赛 | 自动修 ≠ 独立 verification 完成（A4）。禁止当默认执行面 | 无 |
| OpenHands <https://github.com/All-Hands-AI/OpenHands> | AI-driven development 平台 / UI | 禁止把第三方 dev UI 嵌进默认 chrome；禁止 in-process harness | 无 |

未克隆任何外部仓。未安装 `getpaperclipai` 或 `paperclipai` 包。

---

## 3. core 约束摘要 / Core constraints

core 1.0.0 已定稿。权威在 **personal daemon**，合同在 `core/`。
**不改 specs。** 公共语义变更走 Lane-CTR。

| 约束 | 可执行结论 |
|---|---|
| 权威只在 daemon | UI / Assistant / Pi / DSH / Member / MCP / connector 只产 candidate / observation |
| 客户端禁写 | 禁写 SQLite；禁推进 Task / Effect / Verification |
| 完成定义 | 完成 ≠ 模型文本 / HTTP 200 / 工具 receipt / `agent_end`。须当前独立 evidence + daemon acceptance（A4） |
| 五条 lifecycle | 不要合成一台状态机：task / effect / verification / loop / agent-execution |
| kernel harness | `core/crates/cognitive-kernel` 内 harness = 确定性 harness **原语**，**不是** DeepSeek Harness 产品 |
| RFC-0001 | 企业多租户（Tenant / Membership / Company）。2.0 **禁止 Tenant / Company chrome** |
| Conversation | 默认在 **P11-T05 内**做新 Personal private projection version。禁止重解释 `cognitiveos.personal.conversation-projection/0.1`。不要先开独立 Lane-CTR。仅当必须改 **core 公共** conversation schema 时，那一块另走 Lane-CTR |
| UI 真源 | 绑定真源之一：`core/packages/contracts-ts`。客户端不得发明权威 DTO |
| 六族 1.0 | `memory\|skill\|tool\|context\|task\|runtime` 保持 fail-closed。MCP 是 Personal-private `mcp-family/0.1`，不是第七公共族 |

---

## 4. 不变量一页 / Invariants (one page)

完整条文只在 [AXIOMS.md](../../../docs/governance/AXIOMS.md)。此处只列映射期可执行提醒。

| ID | 映射期怎么用 |
|---|---|
| **A1** | daemon-only authority writer。v9 画布 / 聊天 / DSH / Pi 不写 Project / Task / Effect / 预算 |
| **A2** | 概率组件与第三方 agent 只产 candidate。助手建议、成员产出、harness 步骤都不是权威 |
| **A3** | 外部 mutation：persist-before-dispatch Intent/Effect + fencing。点名需要 Intent 的操作见 §6 / §7。聊天只宣布；批准在画布 |
| **A4** | 独立 verification 才能完成 Task。测试门 / 联调验收 / 运行「验收回今日」都不是模型自报 |
| **A5** | Secret 只进批准 SecretStore。Settings 连接不回显密钥；不进聊天 / 画布 / URL / DOM |
| **A6** | 合同与负例不为实现削弱。公共语义 Lane-CTR。不得为了跑通 Scene 改 core |
| **A7** | 本地 / fixture / WSL / ordinary CI 不升 Gate / release / Profile |
| **A8** | 未知工作树受保护。本窗口不混入、不覆盖无关改动 |

产品不变量（scope §3.1 / v9）：

- 聊天无 Approve；无永久 Don’t ask again。
- HITL：聊天链接 + 中心画布预览上 批准 / 改窄 / 拒绝；执行中可 **停**。
- 过期 / unknown 预览不能批；改窄作废旧预览。
- unknown 费用永不显示为 0。
- 无假 Connect / Install / Confirm。
- 五个 lifecycle 分列，不合成「项目状态机」。

---

## 5. 不碰清单 / Do-not-touch

| 项 | 处置 |
|---|---|
| 2.1 native mobile / 配对 / E2E relay / 云端 24/7 | **Deferred** |
| `M-X` / `P11-T14` X connector | **Deferred** / parked；无当前 v9 场景 |
| 成员级预算作为当前 chrome | **Deferred**（2.1）。`P11-T12` 任务卡 **frozen**；与 v9 冲突时以 v9 为准，解冻后重新开发，见 §9 |
| engine store / Installed Agent 商店 / Harness 切换 / 原生 DSH UI 进默认 chrome | **Forbidden** |
| 消费订阅管理 | **Deferred** / out of scope（scope §8） |
| Codex / Claude / Hermes 作为可选 Member 执行引擎 | **Forbidden**（Codex 仅 Memory 参考） |
| 永久 Don’t ask again | **Forbidden** |
| 聊天 Approve | **Forbidden** |
| Tenant / Company chrome | **Forbidden** |
| 把 `/work` 改名 Projects 而不做 Project 聚合 | **Forbidden** as Current |
| 可见 CEO 六步顶栏 | **Forbidden** as 2.0 chrome（后端纪律，不是导航） |
| 可见 Installed Agents 一级 | **Forbidden** as 2.0 chrome |
| 重解释 `conversation-projection/0.1` | **Forbidden** |
| `state-lab` 作为一级导航或默认路由 | **Forbidden** as product-nav。允许位置：Settings 高级 / 默认隐藏 / 非一级 |
| 仿冒仓 `getpaperclipai/paperclip` | **Forbidden** |

---

## 6. v9 Scene 主映射表 / Scene → daemon map

建议 hash 标明 **建议、尚未实现**。现网 hash 仍是 Home / Work / Agents / …
不要假装已经是 Today / Projects / Knowledge。

### 6.0 建议产品 hash（未实现；与 Owner 决议第 4、5 条一致）

| 表面 | 建议 hash | 不是 |
|---|---|---|
| Today（含 empty / incomplete） | `#/` 或 `#/today` | 不是现网 Home 改名即完成 |
| Projects 列表 | `#/projects` | 不是 `#/work` 改名 |
| 项目中心（含 HITL 画布） | `#/projects/:id`（HITL 为画布锚点/表面） | **不是**独立 `#/hitl/:approvalId` 一级或默认路由；**不是** Inbox 一级 |
| Knowledge | `#/knowledge` | 不是 `#/resources` 改名 |
| Settings | `#/settings` | 不是一级 Providers / System |
| state-lab | Settings 高级质检；默认隐藏 | **不是**一级导航；不是纯开发构建开关 |
| Team / Inbox | 无一级 hash | 解冻后也不得按任务卡恢复为一级 |

横切 HTTP（P7-T05 inventory，**不是 OPC 合同**）：

- Session：`POST /local/session`（bootstrap）；`SessionGate` 分
  `management` / `task`。
- 管理：`GET /personal/{status,readiness,doctor,health}`、
  `GET /personal/dsh/runtime`、`/management/providers/*`、
  `/management/agent-bindings`、`/management/usage`、
  `/management/budgets`、`/management/alerts`、`/management/audit`、
  `/management/resource/v1/{list,inspect}`。
- 任务：`POST /task/intent.record` → `intent.interpret` → `preview` →
  `admit`；`GET /task/{watch,evidence,effects,observation}`。
- 现网数据层：`clients/pc/web/src/data/`（`fetchProjection` /
  `useProjection` / `projections/{home,work,workDetail,agents,providers,resources,memory,skills,tools,activity,system}`）。
- envelope 列表：`GET /management/resource/v1/list?family=task`。
  现网 Work **不**发明 lifecycle；无 evidence 则 `state not exposed`。

### 6.1 每 Scene 一行

| Scene | 建议 hash（未实现） | 维护 ID | 现网近似 | 权威缺口 | 判定 | Intent / HITL |
|---|---|---|---|---|---|---|
| `empty-home` | `#/` 或 `#/today` | `M-EMPTY` | `#/` Home 注意力面（四区，非空 Home） | 无 Project 权威；空 Home = 只创建、藏右栏 | **2.0 target** + **Requires-backend** | 创建 CTA 只进向导，不写权威 |
| `create-init` | `#/projects/new` 或 wizard ① | `M-CREATE-1` | 无；`#/work/new` 是单 Task admit 链 | Project/Charter 草稿 + 总预览 | **Requires-backend** | 总预览前项目未上线。确认项走 preview，激活须 Owner confirm + Intent |
| `create-process` | wizard ② | `M-CREATE-2` | 无 | Plan / 流程轴修订 | **Requires-backend** | 「确认这一环」= 当前 preview digest，不是改名 Work |
| `create-members` | wizard ③ | `M-CREATE-3` | 无；`#/agents` 是安装 Agent 名册 | Role/Member Runtime 就位；模型必选 | **Requires-backend** | 就位确认走画布/预览。聊天不能批。无模型 = pending，不静默绑 |
| `create-test` | wizard ④ | `M-CREATE-4` | 无；`#/work/:ref` 是单 Task 时间线 | 每环可打开结果 + 就位门 + 独立 verify | **Requires-backend** | 未知/离线不能过。通过 ≠ 模型文本 |
| `create-joint` | wizard ⑤ | `M-CREATE-5` | 无 | 全流程联调；「验收，进入 Today」第一次成功 | **Requires-backend** | 验收 = 独立 verification + daemon acceptance。无假发布 |
| `today-incomplete` | `#/` / `#/today` | `M-TODAY-INCOMPLETE` | 无（Home 已当日常注意力墙） | 未验收不得进日常决策包 | **2.0 target** + **Requires-backend** | 只「继续创建」 |
| `today` | `#/` / `#/today` | `M-TODAY` | `#/` Home：**替换**，不是改名 | 决策包 + 已上线项目运行概览 + 助手 | **Requires-backend** | 拍板链接到 HITL 画布。无 KPI 墙 |
| `projects` | `#/projects` | `M-PROJECTS` | `#/work` **不是** Projects。改名而不做聚合 = **Forbidden** as Current | Project 列表 / 复制（副本无密钥、在途、跳过） | **Requires-backend** | 复制后编辑 + 总预览 |
| `project-detail` | `#/projects/:id` | `M-LIVE-PROJECT` | `#/work/:taskRef` 是 Task 详情，不是 Project | 只读流程轴 + 去成员/运行/产出 | **Requires-backend** | 点轴换环只在运行。详情不验收 |
| `project-members` | `#/projects/:id/members` | `M-MEMBER-CONFIG` | `#/agents` **替换**（可见 Installed Agents **Forbidden** as chrome） | 先选后看；未选空态 | **Requires-backend** | 配置写入走 preview；无 Install |
| `project-runs` | `#/projects/:id/runs` | `M-LIVE-PROJECT` | Work 时间线可作 **Reusable foundation**，不是运行子菜单 | Routine / Attempt / 末环验收 | **Requires-backend** | 「验收回今日」只在末环；须 A3+A4 |
| `project-outputs` | `#/projects/:id/outputs` | `M-LIVE-PROJECT` | 无 | 先选后看；源链接产出 | **Requires-backend** | 发布预览在画布，聊天无 Confirm |
| `add-member` | `#/projects/:id/members/new` | `M-ADD-MEMBER` | 无 | 当前项目真实名单；先职责后执行方式 | **Requires-backend** | 确认加入 = Intent。拒绝 = 未加入 |
| `member-config` | `#/projects/:id/members/:mid` | `M-MEMBER-CONFIG` | 无；Agent dossier **不是**八标签 | 见 §6.2 | **Requires-backend** | 输入只读流程合同；输出可编「交出什么」 |
| `hitl` | 项目中心画布（`#/projects/:id` HITL 表面）+ Today 深链。**不要**把独立 `#/hitl/:approvalId` 写成产品一级或默认路由 | `M-HITL` | alerts / Work preview **Reusable foundation**；不是一级 Inbox | 序列化当前 preview；过期不能批 | **Requires-backend** | **点名 persist-before-dispatch**。聊天只宣布 |
| `knowledge` | `#/knowledge` | `M-KNOWLEDGE` | `#/resources` + Memory/Skills/Tools **部分 foundation**；不是 Knowledge | Vault / 导入 / Why this fragment / 可检查 Memory | **Requires-backend** | 导入失败保留原件。无 Project 锁定 |
| `settings` | `#/settings` | `M-SETTINGS` | `#/providers` + `#/system` **拆入 Settings**（一级 Providers/System **替换**） | 连接 / 本周不再问收回 / 通知恢复 | **Requires-backend**（连接层级）+ 现网 CP **Reusable foundation** | 密钥走 SecretStore。未知费用 ≠ 0 |
| `state-lab` | `#/settings` 高级质检表面；**默认隐藏**；**非一级** | `M-STATE` | 无 | 九态真版式质检 | Settings 高级 / 默认隐藏 / 非一级。不是纯开发构建开关 | 不是验收、不是 Gate |

### 6.2 member-config 八标签

v9 `MemberConfigTab`（canvas 行 59–67、276–285）：

| Tab id | 中文 | 权威含义 | 判定 |
|---|---|---|---|
| `duty` | 职责 | 岗位职责；身份（模型、就位、负责环节）留在详情头 | **Requires-backend** |
| `input` | 输入 | **只读**流程合同（上一环交出物），不是成员私有输入字段 | **Requires-backend** |
| `output` | 输出 | 可编「交出什么」 | **Requires-backend** |
| `skills` | 技能 | 能力包；安装 ≠ 授权 | **Requires-backend** |
| `tools` | 工具 | 无假 Install | **Requires-backend** |
| `prompt` | 工作说明 | 业务标签；底层可称提示词，默认 chrome 不暴露引擎名 | **Requires-backend** |
| `loop` | 周期与触发 | 能力说法一层后才出现 loop；对应 Routine/Trigger | **Requires-backend** |
| `perms` | 连接与权限 | 精确版本与权限另批；底层有时称 MCP；无市场安装 | **Requires-backend** |

### 6.3 HITL 画布

v9 `HitlScene`：`HitlFate = idle | approved | narrowed | rejected | stopped`；
`PreviewAge = fresh | stale | unknown`。

| 规则 | 可执行结论 |
|---|---|
| 入口 | 聊天只宣布并链接。**无**聊天 Approve |
| 画布行动 | 批准 / 改窄 / 拒绝；执行中第四个是 **停** |
| 可批条件 | 有待批 + `fresh` + 非执行中 + 非已改窄 |
| stale | 不能批；改窄后必须新预览 |
| unknown | 不能批；不是过期也不是成功；禁止盲重试 |
| 本周同类不再问 | 可选、时间盒、Settings 可收回；**无永久 Don’t ask again** |
| 离线 | 不能批准对外 |
| 实现 | 必须 persist-before-dispatch Intent/Effect + fencing（A3）；回执钉在环节，unknown 先 reconcile |

建议 hash：**不要**把独立 `#/hitl/:approvalId` 写成产品一级或默认路由。
HITL **只**在项目中心画布；Today 决策包用深链进入该画布。内部深链若需
稳定 id，用项目内画布锚点（例如 `#/projects/:id` 上的 HITL 表面），不是
独立 Inbox/HITL 一级页。`P11-T09` 任务卡 **frozen**（仍可能写 Inbox
queue）；解冻后以 v9 重新开发，见 §9。

### 6.4 State Lab 九态

`StateKey`（canvas 72–81）：`loading` / `empty` / `working` / `error` /
`success` / `partial` / `blocked` / `unknown` / `offline`。

`SurfaceKey`（82–91）：`today` / `create` / `projects` / `members` /
`runs` / `outputs` / `hitl` / `knowledge` / `settings`。

| 规则 | 可执行结论 |
|---|---|
| 角色 | 设计 / 开发质检；**Settings 高级 / 默认隐藏 / 非一级**。不是纯开发构建开关；**Forbidden** as 一级导航 |
| unknown | 不是 0、不是成功、禁止盲重试 |
| 验证 | Canvas runtime/render、NVDA、host-theme contrast、200% 真布局仍 **not-run** |
| 实现期 | 每个产品表面必须覆盖这九态；缺 backend 用 `Requires-backend` 说明，不用禁用假按钮 |

---

## 7. 现网 `/ui/` 复用与替换 / Reuse vs replace

现网：`HashRouter`（`App.tsx`）；daemon 同源静态 `/ui/`；
`SessionGate` 分通道。空间名仍是 **Home / Work / Agents / Resources /
Activity / System / Providers / session**。

| 现网 | 通道 | 2.0 | 处置 |
|---|---|---|---|
| SessionGate、CSP、hash 路由、同源 `/ui/`、管理/任务分离、memory-only bearer | bootstrap / 双通道 | 保留 | **Current** / **Reusable foundation** |
| `#/` Home（readiness / attention / current work / evidence） | management | Today | **替换 chrome**。投影纪律（不发明总数、unknown ≠ 0）可复用 |
| `#/work`、`#/work/new`、`#/work/:taskRef` | task | Projects / runs | **Reusable foundation**（intent.record 链、envelope、evidence）。**禁止**改名冒充 Project 聚合 |
| `#/agents`、`#/agents/:id` | management | 无一级 | **Forbidden** as 2.0 chrome（可见 Installed Agents）。诊断进 Settings 高级 |
| `#/resources` + Memory / Skills / Tools | management | Knowledge | 六族操作 **Reusable foundation**；IA **替换**为 Knowledge |
| `#/activity` | management | 非一级 | 并入 Today / 项目运行概览，不保留一级 |
| `#/system` | management | Settings 一段 | **替换**一级；stewardship / doctor 可复用 |
| `#/providers`、`#/providers/:id` | management | Settings 连接 | CP **Reusable foundation**；一级 Providers **替换** |
| `#/session` | bootstrap | 保留 | **Current** |
| `#/bindings` → providers、`#/tasks` → work | — | 历史重定向 | 实现 OPC 后不得把 `/tasks` 当产品入口 |
| 原生 `cognitive dsh web` `:3080` | 独立 | 非默认 chrome | **Current** 诊断面。**Forbidden** 嵌进 OPC 默认 IA |
| Pi client / dsh Path B / Provider CP / Task-Intent-Effect-verification | daemon | 引擎 / 权威 | **Reusable foundation**；不构成 OPC IA |

组件对照见
[10-component-map-and-prototype-flows.md](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md)。
能力矩阵见
[11-design-to-code-and-backend-matrix.md](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md)，
本文不复制第三张总表。

---

## 8. 对照 scope §4 缺口 / Gaps vs scope §4

[personal-2.0-scope.md](../product/personal-2.0-scope.md) §4 全表。组合现网原语
**不会**把目标行变成 Current support。

| Capability | Current 真相 | 2.0 处置 | 映射备注 |
|---|---|---|---|
| Windows host/install/background | 既有 Windows 碎片 + ordinary MSVC CI ≠ 合格宿主产品 | **Requires-backend + Requires-environment** | `P11-T02` |
| Project/Charter/Goal/Plan/Attempt | Task 权威可复用；完整 Project 聚合与 UI 投影缺失 | **Requires-backend** | 禁止改装既有 Task 行当 Project |
| Role/Member Runtime | 无完整当前权威/投影 | **Requires-backend** | 权威对象 id = **Employee**；chrome 表面 = Member Runtime（完成后对账） |
| Personal-owned Conversation archive | ADR-0058 信封是决策；无 OPC archive/index/retrieval 产品 | **Requires-backend**；禁止重解释 `0.1`；**T05 内**新 private version，不先开 Lane-CTR | `P11-T05` |
| Personal Assistant | Pi Shell 原语可复用；无全局 OPC 助手 | **Requires-backend**；Pi 保持隐藏 / candidate-only | `P11-T06` |
| Hidden managed DSH | Path B 存在但非 Windows 打包/隔离/供给链合格产品 | **Requires-backend + Requires-environment** | `P11-T07`。禁止原生 DSH UI 进默认 chrome |
| Routine/Trigger/missed-run | scheduler 原语 ≠ 完整产品生命周期 | **Requires-backend** | `P11-T08` |
| Contextual approval/recovery | preview / Effect / alert / recovery 是部分输入 | **Requires-backend** | 产品 = 项目中心画布 HITL + Today 深链，不是一级 Inbox |
| Knowledge/Vault ingestion | Memory/Skill/Context ≠ OPC Vault/导入/索引 | **Requires-backend** | `P11-T10` |
| Memory privacy/forget | 已承认 Memory/forget 可复用；缺会话抽取/检索策略 | **Requires-backend** | `P11-T11` |
| Provider 绑定与诚实费用 | 固定 Agent 绑定、usage、咨询预算存在 | 成员绑定层级、自定义兼容、诚实归因 **Requires-backend** | 成员级预算当前 chrome **Deferred** |
| OPC UI | `/ui/` 是已交付 Linux 期非阻塞面 | 目标 IA + Windows 宿主 **Requires-backend** | `P11-T13` |
| X connector | 无合格 X/Twitter connector | **Requires-backend + Requires-environment** | `P11-T14` parked |
| Skill/MCP 获取与授权 | 现有 Skill/Tool/MCP 传输不足；无评审过的发现/授权流 | 安全评审获取 + 精确 per-scope grant **Requires-backend** | 广谱市场 / family console out of scope |

scope §5：DSH / Pi 日常隐藏。无 Installed Agents 产品面。
scope §7：2.1 = native mobile / 配对 / E2E relay。
scope §8：不承诺离线宿主 24/7、业务结果、消费订阅、可选 Harness 等。

---

## 9. P11-T02–T13 切分建议 / Suggested slices（不领取；任务卡 frozen）

**本窗口不可开始任何实现。** Owner 已暂停 P11 实现，并决议：**先冻结
当前所有未开发开发，以 v9 为准，重新开发。** 正式计划
`PERSONAL-DEVELOPMENT-PLAN.md` 的 P11-T02–T15 正文 **本窗口不改写**。
下列任务卡在本文标为 **frozen / 与 v9 冲突时以 v9 chrome 为准**。
「重新开发」只在将来 owner 解冻 P11 实现时发生：按已批准 v9 chrome
重新切分/实现，而不是按任务卡里的 Team/Inbox 一级或成员级 budget stop
当前 chrome 去实现。**不要**现在改 `router.tsx` 或开 frontend。
`P11-T14` / `P11-T15` 点名停放。

v9 为准的当前 chrome：Today / Projects / Knowledge + 底栏 Settings；
Team、Inbox 不是一级；成员级预算不在当前 chrome（2.1 / Deferred）；
HITL 只在项目中心画布 + Today 深链；state-lab 在 Settings 高级、默认隐藏、
非一级。

### 张力（写入本文，不改正式计划）

| 张力 | 正式计划 / 架构仍可能写 | v9 / 本决议 | 处置 |
|---|---|---|---|
| `P11-T13` IA | Today / Projects / **Team** / Knowledge / **Inbox** + Settings | Team、Inbox **不是一级** | **frozen**；解冻后按 v9 重新开发 |
| `P11-T12` 预算 | Project / **member** / Task budget stop | 成员级预算 **2.1 / 非当前 chrome** | **frozen**；解冻后按 v9 重新开发 |
| 对象名 | Employee / Digital Employee / Installed Agent | 权威 id = **Employee**；chrome = **Member Runtime** | 完成后对账；本窗口不改旅程 |
| `P11-T09` | Inbox queue 一级组合 | 上下文 HITL 只在项目中心画布 + Today 深链 | **frozen**；解冻后按 v9 重新开发 |
| architecture README mermaid | Today · Projects · Team · Knowledge · Inbox | 与 v9 未对账 | **完成后对账**；本窗口不改 README 正文 |

### 任务卡（各一条：依赖 / 垂直切片 / Scene / 为何本窗口不开始）

| Task | 依赖 | 建议垂直切片 | 对应 v9 Scene | 本窗口不开始 |
|---|---|---|---|---|
| **T02** | T01 done；ADR-0052 / P7-T07 碎片 | install → Home app/data → daemon/tray/background close → sleep/offline/missed → recover | Settings 恢复；无独立 Scene | 暂停；任务卡 **frozen**；**Requires-environment** |
| **T03** | T01；现有 Task/Intent/Effect/verifier | 草稿 → Charter/Goal/Plan preview → confirm → active Project → Task/Attempt → Effect/evidence | `create-*`、`projects`、`project-detail`、`project-runs` | 暂停；任务卡 **frozen**；无 Project 聚合则 UI 只能撒谎 |
| **T04** | T03；adapter identity | Blueprint → Assignment → Member Runtime；一当前 manager。权威对象 id = **Employee** | `create-members`、`project-members`、`add-member`、`member-config` | 暂停；任务卡 **frozen**；表面用语完成后对账 |
| **T05** | T03、T04；ADR-0058 | Personal Conversation archive/index/retrieval；**T05 内**新 private version；禁止重解释 `0.1`；不先开 Lane-CTR | 右栏助手 / 项目群（横切 `M-CHAT-CANVAS`） | 暂停；任务卡 **frozen**；core 公共 schema 变更才 Lane-CTR |
| **T06** | T03、T05；exact Pi | 隐藏 Pi 助手：解释/导航/研究/提案 → daemon preview | 右栏 Assistant；`empty-home` 藏聊天 | 暂停；任务卡 **frozen**；Pi Linux 证据不转移 Windows |
| **T07** | T02、T03、T04；dsh Path B | 托管 DSH 精确制品、隔离 child、stdio broker、Provider proxy、update/rollback | 无默认 Scene；失败诊断进 Settings 高级 | 暂停；任务卡 **frozen**；**Forbidden** 原生 DSH UI / 商店 |
| **T08** | T03；scheduler | Routine + Trigger；no-overlap；queue-latest；missed ledger | `member-config` `loop`；`project-runs`；Today 概览 | 暂停；任务卡 **frozen** |
| **T09** | T03、T08；preview/alert | 上下文审批 / 恢复；**不要**做一级 Inbox；HITL 只在项目中心画布 + Today 深链 | `hitl`、`today` 决策包 | 暂停；任务卡 **frozen**（Inbox 措辞与 v9 冲突）；解冻后按 v9 重新开发 |
| **T10** | T03、T05；Memory/Skill/Context | Home 目录 + Knowledge/Vault 导入/索引 | `knowledge` | 暂停；任务卡 **frozen** |
| **T11** | T05、T10 | 有界检索 + Memory 承认/改正/忘记 | `knowledge` Memory 标签 | 暂停；任务卡 **frozen** |
| **T12** | T03、T04、T06、T07；Provider CP | 账户/订阅/配额分离；global→Project→(**2.1** member)→Task 绑定；诚实 usage | `settings`；创建模型必选 | 暂停。**当前 chrome 不做成员级预算硬停**。任务卡 **frozen**；解冻后按 v9 重新开发 |
| **T13** | T02..T12 | daemon-served OPC IA：Today / Projects / Knowledge + Settings + 右栏助手。`state-lab` 只在 Settings 高级、默认隐藏 | 全部产品 Scene（`state-lab` 非一级） | 暂停。任务卡 **frozen**（仍写 Team/Inbox）；解冻后按 v9 重新开发。本窗口不对账计划正文 |
| **T14** | T03、T07..T13 | X/Twitter 场景 | **无** v9 Scene（`M-X`） | **Parked** |
| **T15** | T02..T14 + 合格 Windows | 固定 15 场景验收 campaign | 全部 + X | **Parked**；不是本窗口、不自动 release |

---

## 10. 实现期 UI 挂单 / UI punch list

实现开始后（须 owner 重新授权 P11）必须挂起，**本窗口不开新 canvas**：

| 项 | 状态 | 要求 |
|---|---|---|
| Canvas runtime / render | **not-run** | 不得把 v9 批准写成已渲染验收 |
| NVDA | **not-run** | 九态 + HITL 控件须可朗读 |
| host-theme contrast | **not-run** | 含系统深浅色 |
| 200% 真布局 | **not-run** | 三栏不叠成抽屉；窄画布横滑 |
| hash + 三栏键盘 / 焦点 | 现网 W12 是 Linux IA 证据，**不转移** OPC | 路由变化焦点到主标题；无陷阱 |
| unknown 费用 | 现网 Providers 有诚实 unknown | **永不显示为 0** |
| 假按钮 | — | 无 Connect / Install / Confirm / 聊天 Approve |
| Vite | — | 不得当产品源；只允许 daemon `/ui/` |

---

## 11. 已决议 / Resolved（原待确认）

2026-08-30 Owner 已确认原 7 问。全文见文首 **Owner 决议（2026-08-30）**。
未决 = 0。

| # | 原待确认 | 决议 |
|---|---|---|
| 1 | Employee vs Member Runtime id / 何时对账 | 权威 id = **Employee**。**完成后对账**。表面 Member Runtime 保留到那时。本窗口不改旅程 |
| 2 | `P11-T13` Team/Inbox 一级 vs v9 | 与第 3 条合并：**冻结未开发**；解冻后以 v9 重新开发。本窗口不改正式计划正文 |
| 3 | `P11-T12` 成员/Task budget stop | 同上。成员级预算 **2.1 / 非当前 chrome** |
| 4 | HITL 独立 `#/hitl/:approvalId` vs 画布 + Today 深链 | **只**在项目中心画布；Today 深链。独立 `#/hitl/:approvalId` 不是产品一级或默认路由 |
| 5 | `state-lab` 仅开发构建 vs Settings 高级 | **Settings 高级 / 默认隐藏 / 非一级** |
| 6 | Conversation 新 private version：T05 vs 先 Lane-CTR | **T05 内**新 private version；禁止重解释 0.1；不先开 Lane-CTR。core 公共变更才 Lane-CTR |
| 7 | 本文件是否纳入 handbook source-map | **纳入**（规则 `personal-2-opc-v9-implementation-mapping`） |

无需再确认即可成立的事实：evaluation OFF；P11 实现暂停且 **未领取**；
v9 不是 `/ui/`；DSH 不是 authority writer；仿冒仓 `getpaperclipai/paperclip`
禁止；Owner 批准映射 ≠ 后端已存在。

---

## 附录 A — 现网 hash 对照（冻结，非 OPC）

| 现网 hash | 页面 | 2.0 |
|---|---|---|
| `#/` | Home | 替换为 Today（含 empty / incomplete） |
| `#/work` | Work 清单 | 不改名冒充 Projects |
| `#/work/new` | 单 Task 创建链 | 可复用链，不是五段向导 |
| `#/work/:taskRef` | Task 详情 | 可复用时间线，不是 Project 四子菜单 |
| `#/agents` | Agents | 删除一级 |
| `#/providers` | Providers | 移入 Settings |
| `#/resources`… | 六族 | 重组为 Knowledge |
| `#/activity` | Activity | 取消一级 |
| `#/system` | System | 移入 Settings |
| `#/session` | Session | 保留 |

独立 `#/hitl/:approvalId` **不是** 2.0 产品一级或默认路由（Owner 决议第 4 条）。
`state-lab` **不是** 现网或建议一级 hash（Owner 决议第 5 条）。

[web-ui-route-inventory.json](web-ui-route-inventory.json) 维持 P7-T05 冻结输入。

## 附录 B — 本窗口 fetch / 克隆

| 源 | 结果 |
|---|---|
| `deepseek-ai/deepseek-harness` GitHub + `master` README + `docs/architecture.md` | **pass** |
| `deepseek-harness.github.io` | **not-run** HTTP 500 |
| `deepseek.com/harness` | **not-run** timeout |
| `paperclipai/paperclip` GitHub + `paperclip.ing` + `llms.txt` | **pass** |
| Paperclip `docs/start/architecture.md` | **not-run** 404 |
| `openai/codex` README + API | **pass** |
| `developers.openai.com/codex` | **not-run** timeout |
| Claude Code / SWE-agent / OpenHands GitHub 落地页 | **pass**（深文档未拉） |
| 外部克隆 | **未执行** |
| `getpaperclipai/paperclip` | **未取**（禁止） |

---

End of mapping. Informative only. Owner 批准本文 ≠ 后端已存在。
