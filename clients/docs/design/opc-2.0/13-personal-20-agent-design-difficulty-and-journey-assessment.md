# 13 — Personal 2.0.0 实现技术难点研判：设计 Agent 与用户旅程

- Status: informative assessment; **not** implementation, thaw, Gate, release, Profile, support, or acceptance
- Change class: `product-semantic` research (docs-only)
- Date: 2026-08-30
- Product: **CognitiveOS Personal 2.0.0** (os-personal 2.0.0). Historical canvas filename still contains v9; **v9 is not the product version**.
- Chrome source: [`personal-20-opc-e2e-optimized-v9.canvas.tsx`](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Product intent: [Product design](../../../../personal/docs/product/product-design.md)
- Journeys: [User journeys](../../../../personal/docs/product/user-journeys.md)
- Agent identities: [Assistant, Members, conversations](../../../../personal/docs/product/agent-integration-and-conversations.md)
- Scene → daemon map: [implementation mapping](../../../../personal/docs/architecture/personal-2.0-opc-v9-implementation-mapping.md)
- OSS contrast: [informative matrix](../../../../personal/docs/product/oss-reference-matrix.md)
- Window C paste prompt: `window-c-hardness-solution-prompt.md` (docs-only HOW window; not landed in this T03 commit; output `14-personal-20-hardness-solution-guide.md` not written yet)
- Claim ceiling: `hypothesis`. Owner 阅读本文 ≠ 后端已存在、≠ 可用性/无障碍已验、≠ 可领取 `P11-T*`。
- Not-run: Canvas runtime/render、NVDA、host-theme contrast、200% 真布局；本文未跑用户访谈、未跑 Windows 宿主、未升任何 Gate。

Companion visual: Cursor-local canvas copy `personal-20-agent-difficulty-journey.canvas.tsx`
（可在对话旁打开；不是第二套产品基线；not in Git）。

---

## 0. 本文做什么 / 不做什么

**做：** 以已定档 Personal 2.0.0 chrome 为产品真相，从 Owner 视角走完主旅程，把「设计 Agent」（Personal Assistant 作为系统设计者）从一般 Agent 运行时里拆出来，对照仓库已登记的科研/工业参照与 2026 年前沿方向，给出实现难点分级与不可照搬边界。

**不做：** 不改 chrome、不领取 P11、不改 `clients/pc/web`、不改 core 合同、不把 Linux 1.0 六族写成已是 OPC IA、不把 Magentic-One / CrewAI / Codex / A2A 写成可替换公理层。

权威顺序不变：公理 A1–A8 → 已定档产品事实 → core 1.0.0 合同（本窗口不改）→ 实现映射 → 本文。

---

## 1. 产品一句话与 JTBD

Personal 2.0.0 是 **Windows 本机、主机在线时** 的一人公司数字员工控制台。一位人类 Owner 用业务语言立项、让经理组织工作、在项目群对话里指挥、在证据画布上验收可打开产物。它不是 Agent 安装商店，也不是把业务词贴在 Linux 1.0 Home/Work/Agents 上的改名。

> When I run a long-lived business Project with digital staff, I want to describe the outcome in business language, let a manager organize the work, talk to the team in one Project group conversation, and receive verified deliverables on a flexible canvas, so I can operate the company without becoming an Agent-infrastructure administrator.

**功能工作：** 把一件长期的事变成可验收的流程与可打开产物。
**情感工作：** 不懂 Agent 基础设施也能拍板，且知道系统没有偷偷替自己做对外动作。
**社会工作：** 看起来像在经营公司，而不是在调 Prompt / Harness。

现网 `/ui/` 仍是 Linux 1.0 六族。2.0.0 是设计冻结，不是已实现。矩阵里几乎所有 OPC 目标行仍是 **Requires-backend**；Windows 宿主与托管 DSH 另加 **Requires-environment**。

---

## 2. 「设计 Agent」指什么

产品里没有名叫 Design Agent 的一级对象。但 Owner 真正买到的第一段价值，是 **Personal Assistant 把「这件事」设计成可运行的公司操作系统**：

| 层 | 用户看见的身份 | 内部引擎 | 能不能写权威 |
|---|---|---|---|
| **设计 Agent** | 全局 Personal Assistant（创建向导与项目外） | 隐藏 Pi，candidate-only | 不能。只出调研/配方/预览候选 |
| **运营经理** | 项目内默认发言的 manager | 仍是 Member Runtime，不是第二套助手 | 不能直接写。改目标/团队/权限走 daemon preview |
| **执行员工** | 项目内 Member Runtime（chrome 名）；权威 id = **Employee** | 隐藏托管 DSH 子进程 | 不能写权威、不能持有密钥、不能自己宣布完成 |
| **一次性工人** | 用户默认看不见 | Task → Attempt → 可丢弃 Agent process | 观察/执行已准入动作；进程死不等于成员死 |

对象链（产品，不是 Core Resource 族）：

```text
Owner
  → Project（Charter / Goal / Plan revision）
       → Role Runtime Template（可复用配方）
            → Member Runtime / Employee（项目专属岗位）
                 → Task → Attempt → disposable process
       → 画布 HITL + 独立 verification → Receipt
```

工业界常把「一个 Orchestrator + 若干 specialist」做成 **同一进程里的对话图**。Personal 把同一问题拆成 **三套生命周期**：设计者（Pi 会话）、岗位定义（长期权威对象）、执行过程（有围栏的 Attempt）。混在一起就会出现「重启进程等于换员工」「聊天一句等于改项目」「模型说完了等于验收」——这三条都是产品失败模式，也是公理 A1/A4 的工程失败模式。

所以「设计 Agent 部分」的实现难点，不是把聊天 UI 做漂亮，而是：

1. 让一个 **概率系统设计者** 产出足够结构化的流程、岗位、权限、验收合同；
2. 每一条都经过 **Owner 逐项确认 + daemon 精确 preview** 才成为权威；
3. 设计阶段的候选、运营阶段的群聊、执行阶段的 DSH 上下文 **不得共享同一份可写记忆**。

---

## 3. 用户视角走完旅程

下列按已定档 18 个 scene 顺序走。情感是基于产品规格的假设，不是访谈证据。原型里的周报样品是 **本地目标态**，画布已标明 Requires-backend，不得当成 daemon 已写入。

### 3.1 空首页：还没有公司

**看见：** 中心只有「创建项目」。知识锁定。右侧聊天 **隐藏**。Settings 可以去连模型。

**想做什么：** 开始办一件长期的事，而不是先学 Home/Agents。

**情绪：** 干净，但也可能空得发慌——没有示范项目是产品决定（P0 不给默认/demo Project），不是疏忽。

**技术事实：** 现网 `#/` 是四区注意力 Home，不是空 Home。把 Home 改名成 Today 而不做「无 Project 则只创建」= 撒谎。

**断裂风险：** 若聊天在空首页就出现，Owner 会以为已经有一个在运行的助手公司；规格要求空首页藏聊天，创建页才打开助手。

### 3.2 ① 项目初始化：用业务语言立项

**看见：** 右侧 Personal Assistant 是主入口。画布逐项确认（流程、各环产出、周期、保存形式、能力包、工具、外部连接、知识、环境、文件权限、自动/批准、触发、费用、来源权利、执行方式、总预览）。无模型时聊天只指路 Settings，**禁止静默绑模型**。离开留草稿。

**交互纪律：** 画布编辑 → Enter → 确认对话框 → 以 Owner 名义写入对话 → 助手提案 → 用户在聊天里确认 → 画布才应用。聊天 **没有 Approve**。

**情绪（5 秒）：** 「终于能讲人话。」
**情绪（5 分钟）：** 「原来我要确认十几项，这不是一句话生成公司。」——这是特性，不是摩擦事故。卡片出现在中间 **不是** 第一次成功。

**设计 Agent 在这一步做什么：** 在线调研（GitHub、skill hub 等普通网页阅读不必逐条请示）；外部文本 **不可信、不能当指令执行**；产出 item-by-item 候选。总预览前项目 **未上线**。助手消息不能激活 Project。

**科研/工业对照：**

- Magentic-One 的 Orchestrator 用 Task Ledger / Progress Ledger **自省是否完成**。Personal 禁止把设计 Agent 的自省当激活条件。
- 未审核网页进入配方 = 经典间接注入。产品要求来源/覆盖/冲突可见，且 Secrets 永不进聊天。
- 工业界大量「一句话生成 Agent 团队」产品把安装与授权绑在一起。Personal：**能力包安装 ≠ 授权**；MCP 精确版本与权限另批。

**难点级别：高（产品语义 + 接地 + 注入）。** 现网 `#/work/new` 只是单 Task admit 链，不是这一整段系统设计。

### 3.3 ② 流程初始化：先有轴，才有人

Owner 已拍板 **process before members**（相对 workshop 历史快照 members-then-process）。一条流程轴，一环一环「确认这一环」；未知缺口留在轴上，不标 ready。

**情绪：** 「我在设计流水线，不是在招人。」若先加人再补流程，岗位会变成装饰。

**设计 Agent 难点：** 流程轴是后续 **只读输入合同**（成员「输入」标签不是私有字段）。Assistant 必须按已确认轴生成花名册，而不是按模型偏好发明岗位。轴上的未知不能在 ④ 被「测试通过」洗掉。

**对照：** n8n 类无代码编排把图本身当权威。Personal 的图只是投影；权威是 Plan revision + digest 绑定的 preview。LangGraph 的 interrupt/replay 可借鉴 **Attempt 内** 检查点，不能当 Project 流程轴的第二调度器。

**难点级别：高（领域对象缺失）。** 仓库有 Task/Intent/Effect，没有完整 Project 聚合。映射明确：**禁止改装既有 Task 行冒充 Project。**

### 3.4 ③ 成员初始化：逐个就位，配方在配置页

花名册来自已确认流程。每人必选模型。Init 生成工作说明 / 工具 / 能力包 / 周期与触发 / 外部连接 / 文档范围（Prompt/Skill/MCP 一层后再出现；Loop/Harness 不是默认 chrome）。「当前初始化」只显示 **进度 + 当前项标题**，完整 recipe 在共享配置页。拒绝 = 未加入。缺模型 = pending。

**情绪：** 进度条让人安心；若进度条在 daemon 尚未提交 Member revision 时就走完，会变成最危险的假完成。

**设计 Agent 难点（本产品最硬的一段）：**

1. **生成配方 ≠ 写入员工。** 生成物是候选；就位确认走画布/preview。
2. **顺序就位** 与并行多智能体框架的本能相反（CrewAI/Magentic 倾向并行 specialist）。产品选择顺序，是为了让 Owner 看清每一个岗位的边界。
3. **员工 ≠ 运行时。** 权威 id = Employee；chrome 可写 Member Runtime。进程退出不得删成员、对话、Memory、证据。
4. 成员不跨 Project 共享；只有 Role Template 可复用。全局 Template 升级不得静默改已有成员。

**对照：**

- CrewAI / OpenAI Agents SDK：角色与 handoff **词汇**可参考；crew 完成或 SDK runner 结果 **不是** daemon acceptance。
- Paperclip：公司编排 + heartbeat runner。仓库已裁：**heartbeat 不得写权威**；Company 聚合 Forbidden。
- A2A（Google，2025–2026，现与 MCP 同属 AAIF）：Agent Card 发现、跨组织委托。Personal 2.0 的 Member **不是** 可被外部 Agent 发现的对等体；委托必须变成 daemon 拥有的 Task/revision。把 A2A 当成员总线会打穿项目边界。
- MCP：工具插头，不是岗位身份。发现可以助手来做；授权必须 per-Project/Member 最小权限。

**难点级别：阻断级（T04）。** 没有 Employee 权威对象，③ 的 UI 只能演戏。

### 3.5 ④ 分环节测试：可打开产物 + 就位门

每环检查负责人已就位（六个槽 + 模型）。未就位不能开始/通过。失败回 ②/③ 该环。未知不能通过。离线不能开测。无过程/引擎 chrome。

**情绪：** 「终于看到东西了。」若通过条件是模型说「看起来不错」，aha 会是假的——这正是评测界 2026 年在打的假完成（hallucinated completion、LLM-as-judge 漂移、process exit ≠ 正确）。

**对照：** SWE-bench / GAIA 及后续审计（BenchGuard、Tool-Veritas、SWE-Judge）都在把 **工具调用、任务完成、结果验证** 拆开。Personal 公理 A4 更严：独立 verification，而不是执行 Agent 自报。④ 的「可打开结果 + pass/fail」必须钉在 verifier + 证据 digest，而不是聊天收条。

**难点级别：高（A4 接到业务产物）。** 现网 Work 时间线是 Reusable foundation，不是「按环测试门」。

### 3.6 ⑤ 联合调试：「验收，进入 Today」才是第一次成功

全流程联调。失败点名环节并回 ④/②/③。未知/离线不能验收。无假发布。

**情绪（aha）：** 第一次被允许进入日常 Today。之前 Today 只能「继续未完成的创建」。

**技术：** 验收 = 独立 verification + daemon acceptance。Prototype 的「验收」按钮在真后端到来之前必须标 Requires-backend 或不可用说明，禁止做成会跳转的假成功。

**难点级别：高。** 这是设计 Agent 交付物的关门：它设计的操作系统必须在联调中被 **证伪或证实**，而不是被演示。

### 3.7 日常 Today：一件拍板，不是 KPI 墙

⑤ 之后默认三块：决策包（唯一主 CTA）+ 已上线项目运行概览 + 助手。无待拍板则收起决策包。四泳道不是默认块。聊天可问运行数据，**不能批准**。未知费用永不显示为 0。

**情绪（回访 5 秒）：** 「今天只要决定一件事。」若做成仪表盘，产品会退回被否定的 Agent 控制面。

**对照：** 多数 Agent 控制塔（OpenHands、LobeChat、Paperclip）默认给状态墙。Personal 刻意做 **决策包**。实现上容易把现网 Home 四区改名复用——映射禁止这种 Current 冒充。

### 3.8 已上线项目：详情 / 成员 / 运行 / 产出

详情 = 只读流程轴 + 去三个工作面。运行 = 当前环工作面；「验收，回 Today」 **只在末环**。产出 = 先选后看。无可见 CEO 六步顶栏（CEO 是后端纪律：画布 HITL + 独立 verify）。

`@manager` / `@member` 只插入未发送草稿。改工作的消息必须先成为 Task/revision。成员主动发言仅限：被点名、交产物、交接、阻塞、请求决定。

**情绪：** 像项目群，不像 IDE Agent 面板。风险是群聊变成第二权威。

### 3.9 HITL：聊天宣布，画布拍板

待批动作在中心画布：将做什么 + 完整预览/diff + 批准 / 改窄 / 拒绝；执行中第四个是 **停**。过期预览不能批；改窄必须新预览；unknown 不能批、禁止盲重试；离线不能批对外。可选「本周同一类对外不再问」——时间盒、Settings 可收回，**禁止永久 Don’t ask again**。

这是 AUTONOMY 框架在产品里的落地：Preview、Override、Tiered authority、Observable、Outcome verify、Memory of actions、Yield。和工业界聊天里点 Approve（DSH 原生 UI、许多 harness）正面冲突——产品 **Forbidden** 聊天 Approve。

**对照：** Microsoft Agent Framework 的 Magentic plan review 用 `PlanReviewRequest/Response` 暂停工作流。Personal 还要求 **persist-before-dispatch Intent/Effect + fencing（A3）**，以及 stale digest 失效。AutoGen 已知 HITL 在 handoff 后丢上下文——根因是状态活在编排进程里。Personal 的解法必须是 **daemon 序列化的当前 preview**，不是 Pi/DSH 进程内存。

**难点级别：阻断级（T09 语义 + A3）。** 现网 alerts/preview 是部分输入，不是项目画布 HITL。

### 3.10 知识、设置、状态实验室

- Knowledge：无 Project 锁定；创建期 ② 起仅当前草稿。Why this fragment。导入失败保留原件。Vault 以 Obsidian Markdown 为底座，**不要求安装 Obsidian 应用**。聊天自动承认进可检查/可改/可忘的 Memory。
- Settings：模型连接、密钥单向进 SecretStore、收回时间盒跳过。无引擎商店、无 Inbox、无成员级预算硬停（2.1 / Deferred）。
- state-lab：Settings 高级、默认隐藏、非一级。九态质检，不是产品导航。

**设计 Agent 与记忆：** 助手 Memory 架构参考 GitHub OpenAI Codex 的分层/压缩（只作架构，不是 2.0 运行时 SKU）。Codex 2026 公开实现是两阶段异步抽取+全局合并、关键词检索而非向量库。Personal 必须额外保证：DSH 只收 **有界 Context**；长期 Memory 由 Personal 准入；Letta/Mem0 **不得**直接写权威。

---

## 4. 旅程研判（阶段表）

| 阶段 | Owner 动作 | 触点 | 体验假设 | 最大断裂 | 实现判定 |
|---|---|---|---|---|---|
| 意识/空首页 | 打开产品 | 空 Today | 清爽或茫然 | 聊天过早出现；示范项目诱惑 | Requires-backend（空 Home 语义） |
| 立项 ① | 描述业务、逐项确认 | 助手 + 画布 | 被认真对待，也感到仪式长 | 调研幻觉；静默绑模型；总预览前以为已上线 | Requires-backend |
| 设计流程 ② | 确认每一环 | 流程轴 | 掌控流水线 | 未知被标 ready；先加人 | Requires-backend（Plan 轴） |
| 设计班子 ③ | 逐个就位 | 进度 + 配置页 | 进度可见 | 配方生成即授权；员工=进程 | **阻断** Employee 对象 |
| 证伪 ④⑤ | 看可打开产物、联调验收 | 测试/联调画布 | aha 或失望 | 模型文本当通过 | 独立 verifier 接到产物 |
| 日常运营 | 拍板一件事、看运行行 | Today + 项目四子菜单 | 像管公司 | KPI 墙；聊天批准 | 投影纪律 + HITL 深链 |
| 例外 | 对外发送/扩权 | 画布 HITL | 紧张但清楚 | stale 仍能批；永久不再问 | A3 + 时间盒 |
| 知识/记忆 | 导入、问为什么选这段 | Knowledge | 信任来源 | 检索越权；摘要当完成 | T10/T11 负例 |
| 忠诚/恢复 | 错过运行、关机选择 | Today/运行/Settings | 主机不是云 | 暗示 24/7 | Routine ledger；2.1 才远程 |

**情感弧：** 空 → 被引导（①）→ 变严肃（②③ 确认变多）→ 第一次真东西（④）→ 被允许进入日常（⑤）→ 之后产品必须克制（Today 三块）。设计 Agent 若在 ① 过度承诺「团队已就绪」，后面 ④⑤ 会崩。

---

## 5. 设计 Agent 专章：为什么这是最难的一层

### 5.1 它不是聊天机器人，也不是编码 Agent

| 常见系统 | 设计的是什么 | 完成信号 | Personal 为什么不能照搬 |
|---|---|---|---|
| Claude Code / SWE-agent / OpenHands | 仓库补丁 | 测试/PR/进程结束 | 无 Project/Employee；自测不是 A4 |
| Codex CLI | 编码会话 + 记忆文件 | 用户继续会话 | 可参考记忆分层；禁止当 Member 引擎 |
| Magentic-One | 开放网页/文件任务 | Orchestrator ledger | 自省完成；HITL 易丢状态 |
| CrewAI | 角色团队一次任务 | crew 结果 | 无长期岗位对象；自批判 ≠ verify |
| DSH 原生 Web | 插件化 agent 会话 | harness 批准/结束 | **Forbidden** 嵌进默认 IA；聊天批准 Forbidden |
| Personal 设计 Agent | **公司操作系统本身**（流程、岗位、权限、验收合同） | 仅 Owner confirm + daemon receipt；日常完成另走独立 verify | 本产品 |

### 5.2 必须同时成立的六条不变量

1. **最高 UX 权限，最低写权限。** 助手可打开任何管理流，但写入只有 preview → Owner confirm → receipt。
2. **调研广度 vs 注入。** 普通网页阅读不必逐条请示；外部文本永不升级为指令或密钥。
3. **结构化接地。** 自然语言进，item-by-item 与流程轴出；禁止「生成一整页 JSON 当权威」。
4. **双引擎隔离。** Pi 设计、DSH 执行、daemon 授权。资格证据不互相转移（Linux Pi ≠ Windows OPC）。
5. **上下文工程，不是上下文堆积。** 注入顺序：当前 Task 合同 → 已固定决定 → 带出处摘录 → 摘要 → 旧叙述。超限先砍旧叙述。摘要不能证明完成。
6. **设计态样品诚实。** 原型生成必须标明本地目标态。实现期缺后端则 Requires-backend，禁止假按钮把设计 Agent 伪装成已接通。

### 5.3 设计 Agent 的内部工作流（实现时应对的算法，不是用户 chrome）

```text
业务描述
  → 覆盖可见的调研（可取消、可部分结果）
  → Charter / 产出合同候选
  → 流程轴候选（一环一环确认）
  → 由轴推导花名册（不是由模型偏好推导）
  → 逐成员配方候选（模型必选，权限另批）
  → ④ 每环：就位门 + 可打开产物 + 独立 verify
  → ⑤ 联调验收
  → 此后设计 Agent 降级为「解释/导航/再提案」，不再假装每天重新创立公司
```

失败模式（实现负例应覆盖）：

- 未确认激活；
- 用 Task 行冒充 Project；
- 聊天 Approve；
- 安装即授权；
- 跨项目写成员；
- 进程退出删除员工；
- unknown 费用写成 0；
- 检索超出当前 Project/草稿范围；
- 密钥进入 DOM/聊天/SQLite/日志。

### 5.4 与「Visual UI 设计战役」的关系

预备索引里的 Visual UI 窗口是 **人做视觉规格**，不是模型设计 Agent，也不开新的 `P11-T*`。二者易混：

- **设计 Agent：** 运行时产品能力（Pi + 向导 + 配方候选）。
- **Visual UI 战役：** 解冻后、T13 写 `clients/pc/web` 之前的视觉规格；禁止改 IA、禁止 phase 4 重生 canvas。

本文的难点主轴是前者。后者是并行文档，不降低 T03/T04 的权威难度。

---

## 6. 全栈难点分级（对照前沿）

打分是本评估的 **hypothesis 排序**（1–10，10 = 最难），不是测量、不是 Gate、不是工期承诺。

| ID | 难点 | 分数 | 为何难 | 前沿/工业在做什么 | Personal 约束 | 建议切片 |
|---|---|---|---|---|---|---|
| D1 | 真 Project 聚合 | 9 | 现网只有 Task；改名 Work 是 Forbidden | 工作流引擎把 run 当根对象 | Project ≠ Task 行 | **T03 第一刀** |
| D2 | Employee ≠ runtime | 9 | 三身份最容易被实现揉碎 | A2A 把 agent 当对等服务 | Member 不跨项目；进程可死 | T04 |
| D3 | 设计 Agent 接地与确认循环 | 9 | 概率系统产出操作系统 | Magentic 自省；一句话生成团队 | 逐项 confirm；总预览前未上线 | T06 接 T03 预览 |
| D4 | 画布 HITL + A3 | 8 | 与行业聊天批准相反；stale/unknown | Magentic plan review；AutoGen 丢上下文 | 聊天无 Approve；preview digest | T09 |
| D5 | 独立 verify 接到业务产物 | 8 | ④⑤ 的通过条件易被 UI 做假 | SWE-Judge、分解式 tool-eval | A4；完成 ≠ agent_end | T03 起带负例 |
| D6 | 新 Conversation private version | 7 | 禁止重解释 `0.1` | MCP 不管会话权威 | T05 内新信封；不先 Lane-CTR | T05 |
| D7 | 有界 Context / Memory 准入 | 7 | 设计 Agent 与执行引擎抢记忆 | Codex 两阶段记忆；Claude 偏钩子+MCP | Personal 拥有 Memory；DSH 只收 Context | T10/T11 |
| D8 | 隐藏托管 DSH（Windows） | 8 | Path B ≠ 合格 Windows 制品/沙箱/供应链 | harness 插件、本机 ACL | 非商店、非原生 UI、非 in-process | T07；E2E 常 not-run |
| D9 | 隐藏 Pi 助手 | 6 | Linux 资格不转移 | sidecar 会话 | candidate-only；default-deny | T06 |
| D10 | Routine / 错过 / 关机选择 | 7 | 本机在线产品，不是云 24/7 | Temporal catch-up | Temporal 不得当第二调度器 | T08 |
| D11 | MCP 获取与最小授权 | 7 | 市场模型与「安装即能力」 | MCP 生态爆发；AAIF | 无广谱市场；grant ≠ 安装 | 助手发现 + T04 权限 |
| D12 | 诚实费用与绑定层级 | 5 | 现网有 CP 基础 | 控制面预算墙 | 成员级预算 2.1；unknown≠0 | T12 |
| D13 | OPC IA 替换六族 `/ui/` | 7 | Dual Track 易先画假壳 | 各家 denser workbench | daemon `/ui/`；Vite 非产品源 | T13 在 T03 投影稳定后 |
| D14 | Windows 宿主 | 8 | 无合格 OPC 宿主 | 托盘/后台 | Requires-environment | T02 不挡 T03 |
| D15 | X 连接器 | — | 已停放 | 浏览器/反滥用 | 非 P0 | T14 parked |

**可复用、但构不成 OPC 的基底：** SessionGate、hash `/ui/`、Intent/Effect/verifier、Provider Control Plane、Pi client、dsh Path B、SecretStore、Home 投影的诚实 unknown 纪律。

---

## 7. 相关项目：只借形状，不借权威

以仓库 [OSS matrix](../../../../personal/docs/product/oss-reference-matrix.md) 与预备克隆政策为准。下列是本研判的用法，不更新 HEAD、不构成依赖。

| 项目 | 可借 | 不可借 |
|---|---|---|
| DeepSeek Harness | 沙箱/子进程/会话日志 **形状**；隐藏执行引擎 | 原生 Web、插件商店、harness 批准当产品 HITL、in-process |
| Pi | 助手 sidecar、内存会话 | 权威、Secret、长期 Memory、完成 |
| OpenAI Codex | 记忆分层、渐进披露、history/compaction | Member 引擎、把 memories 当权威库 |
| CrewAI / OpenAI Agents SDK | 经理/专家、handoff 词汇 | 依赖；self-critique 完成 |
| LangGraph | Attempt 内 checkpoint/interrupt | Project 权威；松序列化绕过 Intent/Effect |
| Temporal | overlap/missed 语义对照 | 第二调度器 |
| Obsidian API | Vault 文件兼容 | 嵌入专有应用 |
| Letta / Mem0 | 抽取/检索 UX | 直接写 Memory；默认遥测 |
| OpenHands / LobeChat / assistant-ui | 状态与线程原语 | 完成定义；受限协议代码 |
| Paperclip | 为何/何地/做什么的信息层次 | fork；Company；heartbeat 写权威 |
| Magentic-One / AutoGen | 编排器+专家、HITL 暂停 | 自省完成；进程内 ledger 当跨重启真相 |
| MCP | 工具发现与类型化调用 | 岗位身份；DSH 原生 MCP/base tools |
| A2A | 跨系统委托的产业方向 | Member 对等发现；绕过项目围栏 |

2026 协议栈的诚实读法：MCP = 工具垂直插头；A2A = Agent 水平协作。Personal 2.0 需要的是 **第三层：本机 daemon 权威**（Project/Employee/Task/Effect/verify）。缺这一层，前两层只会让数字员工在协议上互通、在产品上越权。

---

## 8. 实现顺序（不领取）

与 [dev-prep index](../../../../personal/docs/architecture/personal-2.0.0-dev-prep-index.md) 一致，本文只强调设计 Agent 依赖：

1. **T03 Project 聚合** — 没有对象，设计 Agent 无处落地。
2. **T04 Employee** — 没有岗位，③ 和日常成员页都是假的。
3. **T05 新会话投影** — 设计对话与项目群必须可归档、可检索、不可冒充完成。
4. **T06 隐藏 Pi** — 设计 Agent 引擎；必须打在已有 preview 链上，而不是先做「能聊」。
5. **T09 画布 HITL** — 设计/运营阶段所有对外与扩权的关门。
6. **T10/T11 知识与 Memory** — 设计 Agent 的调研与日常问答才不会变成上下文堆积。
7. **T07 DSH** — 执行引擎；可与 T12 诚实费用并行思路，但不挡 T03。
8. **T13 IA** — Dual Track 仅在 T03 投影/HTTP 稳定之后；禁止先画完整假壳。

T02 Windows 宿主不挡 T03。T14/T15 仍停放。正式计划卡与已定档 chrome 冲突处（Team/Inbox 一级、成员级预算、独立 HITL 路由）以 **Personal 2.0.0 chrome 为准**，解冻后重新切，而不是按旧卡实现。

---

## 9. 结论

Personal 2.0.0 的实现重心不是「再做一个 Agent 聊天产品」。聊天、流式、工具卡片在工业界已有成熟模式（也正因如此，产品禁止把它们当成权威）。

真正难、且与前沿同构的，是四件事叠在同一条 Owner 旅程上：

1. **元 Agent：** 一个只能出候选的设计者，要把一人公司的流程与岗位设计出来；
2. **长期岗位对象：** Employee/Member Runtime 与可丢弃 process 分离；
3. **反行业 HITL：** 批准在画布，且必须过 A3 digest，而不是聊天；
4. **反行业完成：** ④⑤ 与日常末环验收走独立 verification，对抗 2026 年评测界已点名的幻觉完成。

现网 Linux 1.0 给了可信的权威内核与诚实投影纪律，但 **没有** OPC 的 Project、Employee、设计向导、项目群、画布 HITL。解冻后第一刀仍应是 T03，而不是先做会说话的助手皮。

需求、采用、可用性、付费意愿仍是假设。本文不构成市场验证或 Agent-benefit。
