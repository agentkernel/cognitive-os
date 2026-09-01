# Personal 2.0 OPC v9 → daemon implementation mapping

# 个人 2.0 OPC v9 到 daemon 实现映射

- Status: **informative** / 非实现 / 非 support / 非 Gate
- Change class: owner-directed post-P12 documentation alignment; no formal
  `P*-T*` claim; no `P11-T15` claim
- Frozen design prototype (not the product): owner-approved canvas v9
  (`clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx`)
- Product origin: daemon-served `/ui/` (`clients/pc/web/` same-origin). Vite
  is not the product origin
- Dual Track hashes on `/ui/`: **Now / hypothesis chrome** after
  `P11-T13` + `P12-T01`–`T09` (merged PR
  [#302](https://github.com/agentkernel/cognitive-os/pull/302) at
  `main@3a563e7c`). Authority remains the P11 walking skeleton.
  No authority → honest empty / Requires-backend. Zero fake Create /
  Activate / Approve.
- NVDA / 200% layout / host-theme: still **not-run** (hung)
- `P11-T15`: independent / **not-started**; not this campaign's mutex
- HEAD at this rewrite: `origin/main` `bf5965f0614e2a3b05835f7bd3afefecf05dec6a`
- Lease: `lease/personal/DOC-P12-ALIGN/docs-reconcile`
- Evaluation routing: **OFF**. `PERSONAL-PERF-EVAL-015` closed.
- Claim ceiling: `hypothesis`. Canvas v9 ≠ product. Dual Track chrome ≠
  Gate / release / Profile / Windows qualification / Agent-benefit.
  Walking-skeleton authority ≠ complete OPC acceptance.

`personal/docs/architecture/README.md` 已在 source-map 规则
`personal-2-baseline`。**本映射文件** 在规则
`personal-2-opc-v9-implementation-mapping`。按
[docs-sync-contract](../../../docs/standards/docs-sync-contract.md) §2
同步 handbook（双语手写页 + 指纹；`generated: true` 页只经
`node tools/src/generate-handbook.mjs`，禁止手改正文）。

2.0.0 do-nots 不变：无 Team / Inbox 一级；聊天无 Approve；无 DSH store /
Installed Agent 商店 / 原生 DSH UI 进默认 chrome；无假 Create / Activate /
Approve；`state-lab` 仍不是一级。Employee 权威 id 与 Member Runtime 表面
用语的完成后对账仍可保留在各 architecture 章正文，不在本窗口批量改旅程。

---

## Owner 决议（2026-08-30）

Owner 已确认下列七条（2026-08-30）。**已决议。** 未决 = 0。
2026-09-01 本文件只把事后事实对齐到 P12 Remaining = 0：Dual Track
hash 已在 `/ui/`；P11-T01–T14 已 done；T15 仍独立未开始。不新开
Phase 13 / PRD；不领取 T15；不实现新 chrome。

| # | 原问题 | Owner 答复（可执行） |
|---|---|---|
| 1 | 权威对象英文 id：Employee 还是 Member Runtime？架构/handbook 何时对账？ | **Employee**。**完成后对账**（实现/任务收口后再做 architecture / handbook 用词对账）。本窗口不改产品旅程。v9 chrome 仍可能写 Member Runtime：权威对象 id = Employee；产品表面 Member Runtime 用语保留到完成后对账。 |
| 2 与 3 | P11-T13 Team/Inbox 一级 vs v9；P11-T12 成员/Task budget stop vs 2.1 | 已按 v9 实现 Dual Track L1：**Today / Projects / Knowledge + 底栏 Settings**。Team、Inbox **不是一级**。成员级预算 **2.1 / Deferred**。`P11-T13` 与 `P12-T01`–`T09` 已 done。不要把 Dual Track chrome 写成完整 `/ui/` 验收或 Gate。 |
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

把 **frozen design prototype**（canvas v9 Scene）映射到 **post-P12** 事实：

- 产品源 daemon-served `/ui/` 上已落地的 Dual Track hash（**Now /
  hypothesis chrome**）；
- P11 walking-skeleton 权威（Project / Employee / archive / Vault /
  Routine / HITL / host / connector）；无权威则诚实 empty /
  Requires-backend；
- 现网 HTTP / 投影（management vs task、session gate）；
- core 1.0.0 合同约束与公理；
- Personal 2.0 scope §4 能力缺口（Windows 资格、完整 OPC 验收仍缺）；
- `P11-T15` 仍独立 / not-started。

### 1.2 本文不做什么

- 不领取、不启动 `P11-T15`，不新开 Phase 13 / PRD，不实现新 chrome。
- 不把 Dual Track chrome 升为 Gate、release、Profile、Windows
  qualification、Agent-benefit。
- 不把 canvas v9 写成已交付产品。
- 不把 Linux 1.0 六族写成 OPC IA。
- 不把 NVDA / 200% / host-theme 的 `not-run` 写成 pass。
- 不把本地 / 文档 / ordinary CI 升为 Gate。

### 1.3 权威顺序

1. [AXIOMS.md](../../../docs/governance/AXIOMS.md) A1–A8（只引用，不粘贴全文）
2. [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
   产品 / 运行时 / 记忆边界
3. [ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md) 现网 Web UI 栈
4. [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)
   `conversation-projection/0.1` **禁止重解释**
5. [personal-2.0-scope.md](../product/personal-2.0-scope.md) §3.1 / §4 / §5 / §7 / §8
6. Frozen canvas v9 + [00-maintenance-index.md](../../../clients/docs/design/opc-2.0/00-maintenance-index.md) 表 A
7. [11-design-to-code-and-backend-matrix.md](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md)
   （引用，不复制成第三张总表）
8. 现网 `clients/pc/web/src/router.tsx`（Dual Track L1 已落地；Linux 1.0
   六族页仍为 Advanced/secondary）。
   [web-ui-route-inventory.json](web-ui-route-inventory.json)
   维持 P7-T05 冻结输入，**不是 OPC 合同**。

### 1.4 产品 chrome 事实（v9 冻结原型 + Dual Track `/ui/`）

- 一级：**Today / Projects / Knowledge** + 底栏 **Settings**。
- **Team、Inbox 不是一级。**
- 聊天无 Approve；HITL 在中心画布；无假按钮。
- `state-lab` 仍不是一级（Settings 高级 / 默认隐藏）。
- Dual Track hash **已在** daemon `/ui/`（hypothesis chrome）。
- 对象名 chrome：**Member Runtime**（v9 产品表面用语）。权威对象英文
  id = **Employee**。产品表面 Member Runtime 用语保留到 **完成后对账**。
  实现期不得另发明第三套权威对象 id。

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
| `M-X` / `P11-T14` X connector | walking skeleton **done**；live X **not-run**；不是 P0 hero；无当前 P0 场景 |
| 成员级预算作为当前 chrome | **Deferred**（2.1）。`P11-T12` 诚实 usage **done**；成员级预算硬停仍不在当前 chrome |
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

Canvas v9 仍是 **frozen design prototype**。daemon `/ui/` 上的 Dual Track
hash 已是 **Now / hypothesis chrome**（`P11-T13` + `P12-T01`–`T09`）。
权威仍是 P11 walking skeleton：无权威 → 诚实 empty / Requires-backend；
**零**假 Create / Activate / Approve。Linux 1.0 Home / Work / Agents /
Resources / Activity / System 仍是 Advanced/secondary，不是 OPC L1。

### 6.0 Dual Track hash（Now / hypothesis chrome；与 Owner 决议第 4、5 条一致）

| 表面 | `/ui/` hash（Now chrome） | 不是 |
|---|---|---|
| Today（含 empty / incomplete） | `#/` | 不是 canvas 像素复制；不是完整 Windows OPC 验收 |
| 五段创建向导 | `#/projects/new` | 不是 `#/work/new` 单 Task 链冒充 Project |
| Projects 列表 | `#/projects` | 不是 `#/work` 改名 |
| 项目中心（含 HITL 画布） | `#/projects/:id`（HITL 为 `?preview=` 画布表面） | **不是**独立 `#/hitl/:approvalId`；**不是** Inbox 一级 |
| members / runs / outputs | `#/projects/:id/{members,runs,outputs}` | 不是 Task 行冒充 Project 聚合 |
| add-member / member-config | `#/projects/:id/members/new`、`…/members/:mid` | 无 Install 商店 |
| Knowledge | `#/knowledge` | 不是 `#/resources` 改名 |
| Settings | `#/settings` | 不是一级 Providers / System |
| 右栏写画布 | 上述表面的右栏 | 聊天无 Approve；candidate-only |
| state-lab | Settings 高级质检；默认隐藏 | **不是**一级导航 |
| Team / Inbox | 无一级 hash | 不得恢复为一级 |

横切 HTTP（P7-T05 inventory + P11/P12 management 面，**不是 OPC 合同**）：

- Session：`POST /local/session`（bootstrap）；`SessionGate` 分
  `management` / `task`。
- 管理：`GET /personal/{status,readiness,doctor,health}`、
  `GET /personal/dsh/runtime`、`/management/providers/*`、
  `/management/agent-bindings`、`/management/usage`、
  `/management/budgets`、`/management/alerts`、`/management/audit`、
  `/management/resource/v1/{list,inspect}`，以及 P11/P12 Project /
  Employee / Vault / Routine / host / connector 管理面（各任务边界内）。
- 任务：`POST /task/intent.record` → `intent.interpret` → `preview` →
  `admit`；`GET /task/{watch,evidence,effects,observation}`。
- Dual Track 数据层：`clients/pc/web/src/views/opc/`。Linux 1.0 投影
  仍在 `projections/{home,work,…}`，只作 Advanced/secondary。
- envelope 列表：`GET /management/resource/v1/list?family=task`。
  无 evidence 则 `state not exposed`。

### 6.1 每 Scene 一行

| Scene | Dual Track hash | 维护 ID | Chrome | 权威 | 判定 | Intent / HITL |
|---|---|---|---|---|---|---|
| `empty-home` | `#/` | `M-EMPTY` | **Now**（P12-T02：只创建、藏右栏） | P11-T03 walking skeleton；无权威诚实 empty | **Now chrome** + 权威 **Requires-backend** 边界 | 创建 CTA 只进向导，不写权威 |
| `create-init` | `#/projects/new` ① | `M-CREATE-1` | **Now**（P12-T02 五段向导） | Project/Charter 草稿仍须 confirm-before-activate | **Now chrome** + 权威 walking skeleton | 总预览前项目未上线。无假 Activate |
| `create-process` | wizard ② | `M-CREATE-2` | **Now** | Plan / 流程轴仍须 preview digest | 同上 | 「确认这一环」= 当前 preview digest |
| `create-members` | wizard ③ | `M-CREATE-3` | **Now** | Employee walking skeleton（P11-T04） | 同上 | 聊天不能批。无模型 = pending |
| `create-test` | wizard ④ | `M-CREATE-4` | **Now** | 独立 verify 仍是 A4 | 同上 | 通过 ≠ 模型文本 |
| `create-joint` | wizard ⑤ | `M-CREATE-5` | **Now** | 验收 = 独立 verification + daemon acceptance | 同上 | 无假发布 |
| `today-incomplete` | `#/` | `M-TODAY-INCOMPLETE` | **Now**（P12-T05：只「继续创建」） | 未验收不得进日常决策包 | **Now chrome** | 只「继续创建」 |
| `today` | `#/` | `M-TODAY` | **Now**（P12-T05 决策包） | pending-previews 来自 HITL 权威 | **Now chrome** | 拍板深链 HITL 画布。无 KPI 墙 |
| `projects` | `#/projects` | `M-PROJECTS` | **Now**（P12-T03） | P11-T03 Project 聚合；禁止 `#/work` 改名冒充 | **Now chrome** | 无权威诚实 empty |
| `project-detail` | `#/projects/:id` | `M-LIVE-PROJECT` | **Now** | 只读流程轴 + HITL 画布（P12-T06） | **Now chrome** | 详情不验收 |
| `project-members` | `#/projects/:id/members` | `M-MEMBER-CONFIG` | **Now** | 先选后看；P11-T04 | **Now chrome** | 无 Install |
| `project-runs` | `#/projects/:id/runs` | `M-LIVE-PROJECT` | **Now** | Routine/Attempt walking skeleton（P11-T08） | **Now chrome** | 「验收回今日」须 A3+A4 |
| `project-outputs` | `#/projects/:id/outputs` | `M-LIVE-PROJECT` | **Now** | 先选后看 | **Now chrome** | 聊天无 Confirm |
| `add-member` | `#/projects/:id/members/new` | `M-ADD-MEMBER` | **Now**（P12-T04） | 确认加入 = Intent | **Now chrome** | 拒绝 = 未加入 |
| `member-config` | `#/projects/:id/members/:mid` | `M-MEMBER-CONFIG` | **Now**（P12-T04 八标签） | 见 §6.2 | **Now chrome** | 无成员级预算 chrome |
| `hitl` | `#/projects/:id?preview=` + Today 深链 | `M-HITL` | **Now**（P12-T06） | persist-before-dispatch（P11-T09） | **Now chrome** | 聊天无 Approve。stale/unknown 不能批 |
| `knowledge` | `#/knowledge` | `M-KNOWLEDGE` | **Now**（P12-T07 ingest） | Vault walking skeleton（P11-T10）；files ≠ Project 权威 | **Now chrome** | 导入失败保留原件 |
| `settings` | `#/settings` | `M-SETTINGS` | **Now**（P12-T08） | 连接表诚实；unknown≠0；本周不再问可收回 | **Now chrome** | 密钥走 SecretStore |
| `state-lab` | Settings 高级；默认隐藏 | `M-STATE` | **not first-class** | 九态质检 | Settings 高级 / 非一级 | 不是验收、不是 Gate |

### 6.2 member-config 八标签

v9 `MemberConfigTab`（canvas 行 59–67、276–285）：

| Tab id | 中文 | 权威含义 | 判定 |
|---|---|---|---|
| `duty` | 职责 | 岗位职责；身份（模型、就位、负责环节）留在详情头 | **Now chrome**（P12-T04）+ Employee walking skeleton |
| `input` | 输入 | **只读**流程合同（上一环交出物），不是成员私有输入字段 | **Now chrome** |
| `output` | 输出 | 可编「交出什么」 | **Now chrome** |
| `skills` | 技能 | 能力包；安装 ≠ 授权 | **Now chrome**；无假 Install |
| `tools` | 工具 | 无假 Install | **Now chrome** |
| `prompt` | 工作说明 | 业务标签；底层可称提示词，默认 chrome 不暴露引擎名 | **Now chrome** |
| `loop` | 周期与触发 | 能力说法一层后才出现 loop；对应 Routine/Trigger | **Now chrome** + P11-T08 walking skeleton |
| `perms` | 连接与权限 | 精确版本与权限另批；底层有时称 MCP；无市场安装 | **Now chrome** |

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

Dual Track hash：**不要**把独立 `#/hitl/:approvalId` 写成产品一级或默认路由。
HITL **只**在项目中心画布（`#/projects/:id?preview=`）；Today 决策包用深链
进入该画布。`P11-T09` 与 `P12-T06` **done**。聊天无 Approve。

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

现网：`HashRouter`（`App.tsx`）；daemon 同源静态 `/ui/`；`SessionGate`
分通道。**L1 已是 Dual Track**：Today / Projects / Knowledge + Settings +
rail。Linux 1.0 Home / Work / Agents / Resources / Activity / System /
Providers 仍作为 **Advanced/secondary** 真路由存在。Team / Inbox /
`#/hitl` **不是**路由。

| 现网 | 通道 | 2.0 | 处置 |
|---|---|---|---|
| SessionGate、CSP、hash 路由、同源 `/ui/`、管理/任务分离、memory-only bearer | bootstrap / 双通道 | 保留 | **Now** / **Reusable foundation** |
| `#/` Today（P11-T13 + P12-T02/T05） | management | Today | **Now / hypothesis chrome**。无权威诚实 empty。投影纪律（unknown ≠ 0）保留 |
| `#/projects`、`#/projects/new`、`#/projects/:id` + members/runs/outputs | management | Projects / wizard / 四子菜单 | **Now chrome**（P12-T02–T04）。权威 = P11 walking skeleton |
| `#/knowledge` | management | Knowledge | **Now chrome**（P12-T07 ingest）。files ≠ Project 权威 |
| `#/settings` | management | Settings | **Now chrome**（P12-T08）。unknown≠0 |
| 右栏 rail write | management | 助手写画布 | **Now chrome**（P12-T09）。candidate-only；无 Approve |
| `#/home` Home | management | Linux 1.0 二级 | **Advanced/secondary**。不是 OPC L1 |
| `#/work`、`#/work/new`、`#/work/:taskRef` | task | 非 L1 | **Reusable foundation**（intent.record 链）。**禁止**改名冒充 Project |
| `#/agents`、`#/agents/:id` | management | 无一级 | **Forbidden** as 2.0 L1（可见 Installed Agents）。诊断进 Settings 高级 |
| `#/resources` + Memory / Skills / Tools | management | Knowledge 二级 | 六族操作 **Reusable foundation** |
| `#/activity` | management | 非一级 | 并入 Today / 项目运行概览 |
| `#/system` | management | Settings 一段 | **Advanced/secondary** |
| `#/providers`、`#/providers/:id` | management | Settings 连接二级 | CP **Reusable foundation** |
| `#/session` | bootstrap | 保留 | **Now** |
| `#/bindings` → providers、`#/tasks` → work | — | 历史重定向 | 不得把 `/tasks` 当产品入口 |
| 原生 `cognitive dsh web` `:3080` | 独立 | 非默认 chrome | **Now** 诊断面。**Forbidden** 嵌进 OPC 默认 IA |
| Pi client / dsh Path B / Provider CP / Task-Intent-Effect-verification | daemon | 引擎 / 权威 | **Reusable foundation**；不构成 OPC IA |

组件对照见
[10-component-map-and-prototype-flows.md](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md)。
能力矩阵见
[11-design-to-code-and-backend-matrix.md](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md)，
本文不复制第三张总表。

---

## 8. 对照 scope §4 缺口 / Gaps vs scope §4

[personal-2.0-scope.md](../product/personal-2.0-scope.md) §4 全表。Walking
skeleton + Dual Track chrome **不会**把目标行变成 Windows OPC support。

| Capability | Current 真相 | 2.0 处置 | 映射备注 |
|---|---|---|---|
| Windows host/install/background | P11-T02 walking skeleton **done**；native E2E **not-run** | **Requires-environment** | `P11-T02` |
| Project/Charter/Goal/Plan/Attempt | P11-T03 walking skeleton **done**；Dual Track 列表/向导 **Now chrome** | 权威仍是 walking skeleton；禁止 Task 行冒充 | `P11-T03` + `P12-T02/T03` |
| Role/Member Runtime | P11-T04 walking skeleton **done**；八标签 chrome **Now** | 权威 id = **Employee**；chrome = Member Runtime | `P11-T04` + `P12-T04` |
| Personal-owned Conversation archive | P11-T05 walking skeleton **done**；禁止重解释 `0.1` | host archive E2E **not-run** | `P11-T05` |
| Personal Assistant | P11-T06 hidden Pi **done**；rail write **Now**（P12-T09） | candidate-only；Pi Linux 不转移 Windows | `P11-T06` + `P12-T09` |
| Hidden managed DSH | P11-T07 walking skeleton **done** | **Requires-environment**；禁止原生 DSH UI 进默认 chrome | `P11-T07` |
| Routine/Trigger/missed-run | P11-T08 walking skeleton **done** | clock/sleep/restart E2E **not-run** | `P11-T08` |
| Contextual approval/recovery | P11-T09 + P12-T06 HITL canvas **Now chrome** | 不是一级 Inbox | `P11-T09` + `P12-T06` |
| Knowledge/Vault ingestion | P11-T10 + P12-T07 **done** | files ≠ Project 权威；host FS E2E **not-run** | `P11-T10` + `P12-T07` |
| Memory privacy/forget | P11-T11 walking skeleton **done** | privacy/rebuild E2E **not-run** | `P11-T11` |
| Provider 绑定与诚实费用 | P11-T12 + P12-T08 **done** | 成员级预算 **Deferred**；unknown≠0 | `P11-T12` + `P12-T08` |
| OPC UI | Dual Track L1 **Now / hypothesis chrome** | NVDA/200%/host-theme **not-run**；不是完整 `/ui/` 验收 | `P11-T13` + `P12-T01`–`T09` |
| X connector | P11-T14 walking skeleton **done** | live X **not-run**；不是 P0 hero | `P11-T14` |
| Skill/MCP 获取与授权 | 现有 Skill/Tool/MCP 传输不足 | 广谱市场 / family console out of scope | 仍 **Requires-backend** |
| 固定 15 场景验收 | `P11-T15` **not-started** / independent | 不是 Phase 12 mutex | 不自动领取 |

scope §5：DSH / Pi 日常隐藏。无 Installed Agents 产品面。
scope §7：2.1 = native mobile / 配对 / E2E relay。
scope §8：不承诺离线宿主 24/7、业务结果、消费订阅、可选 Harness 等。

---

## 9. P11 / P12 事后状态（Remaining = 0；不领取 T15）

`P11-T01`–`T14` **done**。`P12-T01`–`T09` **done**（#302 merged）。
`P11-T15` **unparked / not-started**，不是 prototype completeness mutex。
本窗口是文档对齐，不实现新 chrome，不新开 Phase 13。

Dual Track 当前 chrome：Today / Projects / Knowledge + 底栏 Settings；
Team、Inbox 不是一级；成员级预算不在当前 chrome（2.1 / Deferred）；
HITL 只在项目中心画布 + Today 深链；state-lab 在 Settings 高级、默认隐藏、
非一级。无权威 → 诚实 empty；零假 Create / Activate / Approve。

### 张力（事后）

| 张力 | 事后事实 | 处置 |
|---|---|---|
| `P11-T13` IA | Dual Track L1 **done**；Team/Inbox **不是一级** | architecture README mermaid 本窗口去掉 Team/Inbox 一级 |
| `P11-T12` 预算 | 诚实 usage **done**；成员级硬停仍 2.1 | 当前 chrome 不做成员级预算 |
| 对象名 | 权威 id = **Employee**；chrome = **Member Runtime** | 完成后对账仍可保留在各章正文 |
| `P11-T09` | HITL 画布 **done**；不是 Inbox L1 | 保持 |
| `P11-T15` | independent / not-started | 不自动领取 |

### 任务卡状态

| Task | 状态 | Dual Track / 权威 | 非声明 |
|---|---|---|---|
| **T02** | **done** #292 | host walking skeleton | native E2E **not-run** |
| **T03** | **done** #281 | Project 聚合 walking skeleton | 不是完整 `/ui/` 页 |
| **T04** | **done** #282 | Employee walking skeleton | 表面用语完成后对账 |
| **T05** | **done** #283 | Conversation archive walking skeleton | 禁止重解释 `0.1`；host E2E **not-run** |
| **T06** | **done** #284 | hidden Pi Assistant | draft-apply ≠ authority-approve |
| **T07** | **done** #287 | hidden hosted DSH | **Forbidden** 原生 DSH UI / 商店 |
| **T08** | **done** #290 | Routine/Trigger walking skeleton | 无第二 scheduler / Inbox L1 |
| **T09** | **done** #285 + P12-T06 #299 | HITL canvas | 聊天无 Approve |
| **T10** | **done** #288 + P12-T07 #300 | Vault + ingest chrome | files ≠ authority |
| **T11** | **done** #289 | Memory admission/privacy/forget | 无 Letta/Mem0 写路径 |
| **T12** | **done** #286 + P12-T08 #301 | 诚实 usage + Settings 连接 | 成员级预算 2.1 |
| **T13** | **done** #291 | Dual Track L1 chrome | 不是完整 `/ui/` 验收 |
| **T14** | **done** #293 | X connector walking skeleton | live X **not-run**；不是 P0 hero |
| **T15** | **not-started** / independent | 固定 15 场景验收 | 不是 P12 mutex；不自动领取 |
| **P12-T01**–**T09** | **done** #294–#302 | hypothesis chrome on `/ui/` | NVDA/200%/host-theme **not-run** |

---

## 10. 仍挂单 / hung punch list

不新开 canvas。下列仍 **not-run**：

| 项 | 状态 | 要求 |
|---|---|---|
| Canvas runtime / render | **not-run** | 不得把 v9 批准写成已渲染验收 |
| NVDA | **not-run** | 九态 + HITL 控件须可朗读 |
| host-theme contrast | **not-run** | 含系统深浅色 |
| 200% 真布局 | **not-run** | 三栏不叠成抽屉；窄画布横滑 |
| hash + 三栏键盘 / 焦点 | 现网 W12 是 Linux IA 证据，**不转移** OPC | 路由变化焦点到主标题；无陷阱 |
| unknown 费用 | Dual Track Settings 有诚实 unknown | **永不显示为 0** |
| 假按钮 | Dual Track 负例覆盖 | 无 Connect / Install / Confirm / 聊天 Approve |
| Vite | — | 不得当产品源；只允许 daemon `/ui/` |

---

## 11. 已决议 / Resolved（原待确认）

2026-08-30 Owner 已确认原 7 问。2026-09-01 对齐 P12 Remaining = 0。
未决 = 0。

| # | 原待确认 | 决议 + 事后 |
|---|---|---|
| 1 | Employee vs Member Runtime id / 何时对账 | 权威 id = **Employee**。完成后对账仍可保留。 |
| 2 | `P11-T13` Team/Inbox 一级 vs v9 | Dual Track L1 **done**。Team/Inbox **不是一级**。 |
| 3 | `P11-T12` 成员/Task budget stop | 成员级预算 **2.1 / 非当前 chrome**。诚实 usage **done**。 |
| 4 | HITL 独立 `#/hitl/:approvalId` vs 画布 + Today 深链 | **只**在项目中心画布；Today 深链。P12-T06 **done**。 |
| 5 | `state-lab` 仅开发构建 vs Settings 高级 | **Settings 高级 / 默认隐藏 / 非一级** |
| 6 | Conversation 新 private version：T05 vs 先 Lane-CTR | T05 **done**（新 private version）；禁止重解释 0.1 |
| 7 | 本文件是否纳入 handbook source-map | **纳入**（规则 `personal-2-opc-v9-implementation-mapping`） |

无需再确认即可成立的事实：evaluation OFF；P11-T01–T14 **done**；
T15 independent / not-started；canvas v9 **不是**产品；产品源 = daemon
`/ui/`；DSH 不是 authority writer；仿冒仓 `getpaperclipai/paperclip`
禁止；Dual Track chrome ≠ Gate / release。

---

## 附录 A — Dual Track vs Linux 1.0 hash

| hash | 角色 | 事实 |
|---|---|---|
| `#/` | Today（empty / incomplete / packets） | **Now / hypothesis chrome**（P11-T13 + P12-T02/T05） |
| `#/projects/new` | 五段向导 | **Now**（P12-T02） |
| `#/projects` | Projects 列表 | **Now**（P12-T03） |
| `#/projects/:id` + members/runs/outputs | 项目中心 / 四子菜单 / HITL | **Now**（P12-T03/T06） |
| `#/projects/:id/members/new`、`…/:mid` | 加成员 / 八标签 | **Now**（P12-T04） |
| `#/knowledge` | Knowledge ingest | **Now**（P12-T07） |
| `#/settings` | Settings 连接 | **Now**（P12-T08） |
| `#/home` | Linux 1.0 Home | Advanced/secondary |
| `#/work` | Linux 1.0 Work | 不改名冒充 Projects |
| `#/work/new` | 单 Task 创建链 | 可复用链，不是 OPC 向导 |
| `#/work/:taskRef` | Task 详情 | 可复用时间线，不是 Project 四子菜单 |
| `#/agents` | Agents | 不是 OPC L1 |
| `#/providers` | Providers | Settings 二级 |
| `#/resources`… | 六族 | Knowledge 二级 |
| `#/activity` | Activity | 非一级 |
| `#/system` | System | Settings 二级 |
| `#/session` | Session | 保留 |

独立 `#/hitl/:approvalId` **不是** 2.0 产品一级或默认路由（Owner 决议第 4 条）。
`state-lab` **不是**一级 hash（Owner 决议第 5 条）。

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

End of mapping. Informative only. Canvas v9 ≠ product. Dual Track chrome ≠
Gate / release. Authority remains the P11 walking skeleton.
