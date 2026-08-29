# 00 — Personal 2.0 OPC 维护索引（module × journey）

- Status: canonical PM/UI maintenance catalog; not a second product baseline
- Change class: `product-semantic`
- Date: 2026-08-30 (current chrome retarget to owner-approved v9)
- Product intent: [Product design](../../../../personal/docs/product/product-design.md)
- Version boundary: [Personal 2.0 scope](../../../../personal/docs/product/personal-2.0-scope.md)
- Design home: [OPC corpus README](README.md)
- Current chrome:
  [`personal-20-opc-e2e-optimized-v9.canvas.tsx`](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Prior approved baseline (not current chrome; **not overwritten**):
  [`personal-20-opc-e2e-optimized-v8.canvas.tsx`](personal-20-opc-e2e-optimized-v8.canvas.tsx)
- Prior live iterations (not current chrome; not overwritten):
  `personal-20-opc-e2e-optimized-v5.canvas.tsx`,
  `personal-20-opc-e2e-optimized-v6.canvas.tsx`,
  `personal-20-opc-e2e-optimized-v7.canvas.tsx`
- Archive (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Evidence boundary: Owner 批准 v9（2026-08-30）不是可用性、无障碍、backend、Gate、release、
  qualification 或 acceptance 证据。Canvas runtime/render、NVDA、host-theme
  contrast、200% 真布局仍为 `not-run`。

本页是 **一张表、两处入口**：UI 从本语料 README 进来；PM 从产品 README / 2.0
scope 进来。不要在产品文档里再复制整表。

---

## 怎么用（一次只动一个单元）

1. **选一行**（一个 ID = 一个可独立维护的模块或 feature flow）。
2. **读产品真相**（表 B 的产品文档）。聊天、旧 canvas、旧 handoff 不能覆盖它。
3. **打开设计章节**（表 B 的语料）。先看 IA 与该章状态语法，再看组件。
4. **打开 v9 场景**（表 A 的 scene id）。只改这一行对应的表面；不要顺手改相邻
   创建阶段或把 v8 / v5–v7 / archived v1–v4 / V2 当当前 chrome。不要覆盖 v8。
5. **状态要覆盖**：该表面在 empty / first-run、working、blocked、unknown、offline
   下怎么说。v9 的 `state-lab` 用同一套 StateKey 渲染，不是「Designed」矩阵。
6. **改了交互 chrome**：跑 `/personal-20-prototype-review`。在 owner 确认下一版
   原型之前，**不要**改产品文档。不要开 phase 4 新 canvas 迭代，除非 owner 明确要求。
7. 缺后端的能力标 `Requires-backend`；缺合格环境标 `Requires-environment`。不要做
   Connect / Install / Confirm 假按钮。聊天无 Approve；HITL 在画布。

### 当前 chrome 与归档

| 角色 | 路径 |
|---|---|
| 当前交互原型 | `clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx` |
| Cursor 可打开副本 | `C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-opc-e2e-optimized-v9.canvas.tsx` |
| 上一版已批准基线（保留，禁止覆盖） | `clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v8.canvas.tsx` |
| 已批准创建环 | ① project → ② process → ③ members（逐个就位；「当前初始化」只显示进度 + 当前项标题）→ ④ 在位成员测试门 → ⑤ joint。第一次成功只能是 ⑤「验收，进入 Today」。⑤ 之前禁止日常 Today。create/members/test/joint 右侧默认 Personal Assistant。聊天无 Approve。HITL 在画布。无 Install。Loop/Harness 不是默认 chrome。 |
| 已批准成员管理 | 先选人再展示配置；未选人为空态，不默认第一人。换项目清空选择。标签：职责 / 输入 / 输出 / 技能 / 工具 / 工作说明 / 周期与触发 / 连接与权限。身份（模型、就位、负责环节）留在详情头。输入只读流程合同（上一环交出物），不是成员私有输入字段。输出可编「交出什么」。加人用当前项目真实名单。对象名：Member Runtime。 |
| 已批准项目工作 | 四个子菜单：详情 / 成员 / 运行 / 产出。列表一行一个「打开」+ 成员/运行/产出文字链。详情只读流程轴 + 去成员/运行/产出。运行验收只在末环。产出同样先选后看。 |
| 已批准 Today | 决策包 + 运行概览（已上线项目行）+ 助手。不是四泳道。无待拍板则收起决策包。 |
| 同目录先前迭代（非当前） | v8（上一版已批准基线，禁止覆盖）；v5（更早同日曾批准）、未批准 v6/v7。源文件保留，不得当当前 chrome。 |
| 归档（非当前） | `clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/`（含 `personal-20-opc-e2e` 与 optimized v1–v4） |
| 更早归档 | `clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/`（CEO 六步顶栏 / X-hero） |

ID 别名：`M-MEMBERS-INIT` = `M-CREATE-3`；`M-TEST-STAGE` = `M-CREATE-4`。

---

## 表 A — 可维护单元（选哪一行）

| ID | 模块 | Feature flow（用户任务） | Surfaces / v9 scenes | Honesty / gate | Parked / 2.1 | Suggested next action |
|---|---|---|---|---|---|---|
| `M-SHELL` | App shell | 用 Today / Projects / Knowledge + 底栏 Settings 到达目标；三栏锁定 | 全部场景共用左栏；`empty-home` 隐藏右栏聊天 | 设计路由不是 API。可见 CEO 六步轨不是产品 chrome（02 章已随 v9 批准对齐） | 窄窗横滑 ≠ native mobile | 维护三栏与 L1 锚点；不要把 Team/Inbox 做成一级 |
| `M-EMPTY` | First-run / empty Home | 无 Project 时只创建；不进日常 Today | `empty-home`；State Lab `today` × `empty` | 空 Home 不是 KPI 墙。无默认/示范 Project | Knowledge 无 Project 时锁定 | 保持「只创建」。缺模型时聊天只指路 Settings，不静默绑模型 |
| `M-CREATE-1` | 创建 ① | 用业务语言立项；逐项确认产出/周期/权限/总预览 | `create-init` | `Requires-backend`。总预览前项目未上线。原型生成是本地目标态样品，不是 daemon 写 | 无 Install；无 X 作为 P0 | 保持 item-by-item 确认。离开留草稿。不要在 ① 后做「卡片在中间即成功」 |
| `M-CREATE-2` | 创建 ② | 一条流程轴、一环一环确认；未知缺口留在轴上 | `create-process` | process-before-members 是当前产品事实（v5 起，v9 保留）。workshop Q&A 仍是历史 members-then-process 快照，不覆盖当前 chrome | 架构/正式计划仍 deferred | 只改流程轴与「确认这一环」。不要把成员就位提前到 ② |
| `M-CREATE-3` / `M-MEMBERS-INIT` | 创建 ③ | 按已确认流程建班子，逐个就位；模型必选 | `create-members` | 「当前初始化」= 进度 + 当前项标题；完整配方在配置页。拒加入 = 未加入。缺模型 = pending | 成员不跨 Project 共享 | 保持顺序就位。不要把完整 recipe 塞进当前条。右侧默认 Assistant |
| `M-CREATE-4` / `M-TEST-STAGE` | 创建 ④ | 按环测试可打开结果；负责人未就位不能开始/通过 | `create-test` | 未知不能通过。离线不能开测。无过程/引擎 chrome | — | 保持就位门。失败回 ②/③ 该环，不要跳过 |
| `M-CREATE-5` | 创建 ⑤ | 全流程联调；「验收，进入 Today」是第一次成功 | `create-joint` | 未知不能验收。离线不能联调。无假发布 | 日常 Today 在 ⑤ 之前禁止 | 验收才是 aha。失败点名环节并回 ④/②/③ |
| `M-TODAY-INCOMPLETE` | Today（创建未完成） | 未验收时 Today 只继续创建 | `today-incomplete` | 不是决策包。Projects 只暴露这份未完成草稿 | — | 不要在未完成创建时画日常决策包或运行概览 |
| `M-TODAY` | Today（已上线） | 回访：一件拍板 + 上线项目运行概览 + 助手 | `today`；State Lab `today` × working/blocked/unknown/offline | 四泳道不是默认块。无待拍板则收起决策包。03 章已随 v9 批准对齐。Owner 批准 ≠ Gate | 泳道语义可并入概览，不是 2.1 | 以 v9 / user-journeys §2 为准维护默认三块 |
| `M-PROJECTS` | Projects 列表 | 找到/复制已上线 Project；副本未激活 | `projects` | 无默认/示范项目。副本不带密钥、在途任务、外部回执、时间盒跳过。列表一行一个「打开」，成员/运行/产出用文字链 | — | 复制后走编辑 + 总预览。④⑤ 可抽查或跳过，不从 ① 重来。不要改成四个并列「查看」按钮 |
| `M-LIVE-PROJECT` | 已上线 Project | 四个子菜单：详情 / 成员 / 运行 / 产出 | `project-detail`；`project-runs`；`project-outputs` | 详情只读流程轴 + 去成员/运行/产出。无可见 CEO 六步顶栏。聊天不能批、不能验收。运行「验收回今日」只在末环。产出先选后看 | 运营工作视图是后端纪律，不是顶栏 | 点轴换环只在运行管理。空环说缺什么；unknown 不能标完成；offline 标过时。中途环节不验收 |
| `M-ADD-MEMBER` | 日常加人 | 已上线后加一个岗位：职责 → 确认加入 → 再披露执行方式 | `add-member` | 不是「先装 MCP」。无模型 = pending。加入后改流程/权限要再批。加人名单 = 当前项目真实成员，不是全局示范名单 | — | 保持与创建 ③ 同一套就位规则。离线可改职责，不能在线搜岗位方案 |
| `M-MEMBER-CONFIG` | 成员配置 | 先选人再看完整执行方式 | `project-members`；`member-config` | 未选空态，不默认第一人。换项目清空选择。标签：职责 / 输入 / 输出 / 技能 / 工具 / 工作说明 / 周期与触发 / 连接与权限。身份留在详情头。输入只读流程合同。输出可编「交出什么」。无安装按钮 | Loop 不是默认 chrome；不要编造成员私有输入字段 | 配置页承载完整条文。创建 ③ 当前条不要复制整页 |
| `M-HITL` | HITL | 对外/关键动作：画布预览上批准 / 改窄 / 拒绝；可选本周同类不再问 | `hitl`；State Lab `hitl` × blocked/unknown/offline | 聊天只宣布并链接。无 Approve、无永久 Don’t ask again。过期预览不能批 | — | 执行中可停。改窄要新预览。时间盒跳过可在 Settings 收回 |
| `M-KNOWLEDGE` | Knowledge | 看资料、Why this fragment、导入；检查/忘记 Memory | `knowledge`；创建期可从 ② 打开当前草稿 | 无 Project 锁定。Obsidian 是底座，不必装该应用。聊天自动承认到可检查 Memory | 无 Project 的云导入、native mobile 库 | 导入失败保留原件。目标/角色/权限状编辑不能静默改权威 |
| `M-SETTINGS` | Settings | 连模型、收回本周不再问、通知与恢复 | `settings` | SecretStore 接管，密钥不进聊天/画布。无账单、无引擎商店、无 Inbox | 消费订阅管理 out of scope；成员级预算是 2.1，不是当前 chrome | 连接失败点名问题。未知费用永不写 0。不要做 Install 商店 |
| `M-CHAT-CANVAS` | 对话 + 画布契约 | 右栏始终是第三列；画布编辑 → Enter → 确认框 → 聊天里确认 → 再写画布 | 创建/成员/测试/联调默认 Assistant；上线后项目群 | 聊天无 Approve。`@` 只进未发送草稿。无 overlay「打开会话」 | 2.1 远程会话不是本 chrome | 窄画布横滑，不把三栏叠成抽屉。不要把 HITL 批准按钮放进聊天 |
| `M-STATE` | 状态语法 / State Lab | 同一表面在 loading/empty/working/error/success/partial/blocked/unknown/offline 下的真版式 | `state-lab`（SurfaceKey × StateKey） | State Lab 不是验收。runtime/NVDA/对比度/200% 仍 not-run | — | 改任一表面时在 State Lab 扫这九态。unknown 不是 0、不是成功、禁止盲重试 |
| `M-X` | 停放连接器 | X/Twitter 社交账号运营 | **无**当前 v9 场景 | 不是 P0。不是首个端到端验收路径 | **Parked / later connector**；2.1 另有 native mobile / pairing / E2E relay | 不要画进当前 chrome。不要当 Gate 旅程 |

---

## 表 B — 权威源与职责（同一 ID，不复制表 A）

| ID | 产品真相 | Design corpus | PM maintains | UI maintains |
|---|---|---|---|---|
| `M-SHELL` | [2.0 scope §3.1](../../../../personal/docs/product/personal-2.0-scope.md)；[web-ui-design](../../../../personal/docs/product/web-ui-design.md) | [02 IA](02-information-architecture-and-app-shell.md)；[10 组件](10-component-map-and-prototype-flows.md) | L1 锚点、Team/Inbox 降级、2.1 边界 | 三栏锁定、窄窗横滑、焦点/路标 |
| `M-EMPTY` | [user-journeys §1](../../../../personal/docs/product/user-journeys.md) | [03](03-today-projects-and-briefing.md)；[04](04-guided-project-setup.md)；[09 空态](09-state-accessibility-and-visual-system.md) | 无示范项目；空 Home 文案 | 只留创建 CTA；聊天隐藏 |
| `M-CREATE-1` | [user-journeys §1](../../../../personal/docs/product/user-journeys.md)；[product-design](../../../../personal/docs/product/product-design.md) | [04](04-guided-project-setup.md)；[01 JTBD](01-product-model-and-jtbd.md) | 确认清单条目、总预览前未上线 | ① 画布、确认列表、草稿/离线 |
| `M-CREATE-2` | [user-journeys §1.4](../../../../personal/docs/product/user-journeys.md)；[scope §3.1](../../../../personal/docs/product/personal-2.0-scope.md) | [04 五段序](04-guided-project-setup.md) | process-before-members 产品修正 vs workshop 快照 | 一环一确认、轴上未知缺口 |
| `M-CREATE-3` / `M-MEMBERS-INIT` | [user-journeys §1.5](../../../../personal/docs/product/user-journeys.md)；[opc-product-model](../../../../personal/docs/product/opc-product-model.md) | [04](04-guided-project-setup.md)；[05](05-team-roles-employees-and-conversations.md) | 逐个就位、模型必选、成员不共享 | 紧凑当前项标题；进度；无完整 recipe |
| `M-CREATE-4` / `M-TEST-STAGE` | [user-journeys §1.6](../../../../personal/docs/product/user-journeys.md) | [04](04-guided-project-setup.md)；[09](09-state-accessibility-and-visual-system.md) | 就位门、未知/离线不能过 | 可打开结果、通过/失败、无引擎名 |
| `M-CREATE-5` | [user-journeys §1.7](../../../../personal/docs/product/user-journeys.md) | [04 验收](04-guided-project-setup.md)；[12 场景](12-scenario-and-heuristic-review.md) | ⑤ 是第一次成功；非假发布 | 全流程位置、验收 CTA、失败回退 |
| `M-TODAY-INCOMPLETE` | [user-journeys §1 末 / §2](../../../../personal/docs/product/user-journeys.md) | [03](03-today-projects-and-briefing.md) | 未完成创建不得进入日常 Today | 只保留「继续创建」 |
| `M-TODAY` | [user-journeys §2](../../../../personal/docs/product/user-journeys.md)；[product-design](../../../../personal/docs/product/product-design.md) | [03](03-today-projects-and-briefing.md)；[07](07-inbox-approval-and-recovery.md) | 决策包 + 运行概览（已上线项目行）；无待拍板则收起；费用未知≠0 | 默认三块布局；周期切换；无 KPI 墙 |
| `M-PROJECTS` | [user-journeys §1.8](../../../../personal/docs/product/user-journeys.md) | [03 列表](03-today-projects-and-briefing.md) | 副本不含密钥/在途/跳过；一行打开 + 文字链 | 列表/筛选/复制入口；不要四个「查看」 |
| `M-LIVE-PROJECT` | [user-journeys §3](../../../../personal/docs/product/user-journeys.md)；[scope §3.1](../../../../personal/docs/product/personal-2.0-scope.md) | [03 运行画布](03-today-projects-and-briefing.md)；[10](10-component-map-and-prototype-flows.md) | 四子菜单；详情只读轴；运行末环才验收 | 详情/成员/运行/产出；产出先选后看 |
| `M-ADD-MEMBER` | [user-journeys §4](../../../../personal/docs/product/user-journeys.md) | [05](05-team-roles-employees-and-conversations.md) | 先岗位后执行方式；当前项目真实名单 | 建议岗位、确认加入、pending 模型 |
| `M-MEMBER-CONFIG` | [opc-product-model](../../../../personal/docs/product/opc-product-model.md)；[agent-integration](../../../../personal/docs/product/agent-integration-and-conversations.md) | [05](05-team-roles-employees-and-conversations.md)；[08 Skill/MCP](08-settings-agents-providers-and-usage.md) | 先选后看；八标签；输入只读；输出可编交出 | 共享配置页、分层披露、无 Install |
| `M-HITL` | [user-journeys §6](../../../../personal/docs/product/user-journeys.md)；[scope §3.1](../../../../personal/docs/product/personal-2.0-scope.md) | [07](07-inbox-approval-and-recovery.md) | 画布才是可确认对象；时间盒可收回 | 预览/diff/批准/改窄/拒绝/停 |
| `M-KNOWLEDGE` | [user-journeys §5](../../../../personal/docs/product/user-journeys.md)；[knowledge-memory-vault](../../../../personal/docs/product/knowledge-memory-vault.md) | [06](06-knowledge-vault-and-memory.md) | Why this fragment；自动承认可忘 | 导入/锁定/Memory 检查；无嵌入 Obsidian |
| `M-SETTINGS` | [user-journeys §8](../../../../personal/docs/product/user-journeys.md)；[account-hub](../../../../personal/docs/product/account-hub.md) | [08](08-settings-agents-providers-and-usage.md) | 模板+自定义连接；无订阅产品 | 连接态/失败、密钥不回显、跳过收回 |
| `M-CHAT-CANVAS` | [scope §3.1](../../../../personal/docs/product/personal-2.0-scope.md)；[web-ui-design](../../../../personal/docs/product/web-ui-design.md) | [02](02-information-architecture-and-app-shell.md)；[05 群聊](05-team-roles-employees-and-conversations.md)；[10](10-component-map-and-prototype-flows.md) | 创建环默认 Assistant；群聊发言规则 | 右栏恒在；无 overlay；编辑确认链 |
| `M-STATE` | [web-ui-design 状态](../../../../personal/docs/product/web-ui-design.md)；[user-journeys 各节态](../../../../personal/docs/product/user-journeys.md) | [09](09-state-accessibility-and-visual-system.md)；v9 `state-lab` | 诚实标签（unknown/stale/not-run） | 九态真版式、对比/焦点（仍 not-run 验证） |
| `M-X` | [user-journeys §10](../../../../personal/docs/product/user-journeys.md)；[scope §3.6](../../../../personal/docs/product/personal-2.0-scope.md) | [12](12-scenario-and-heuristic-review.md) 仅作停放 | 保持 parked；不是 P0 hero | 不画当前场景 |

跨切实现依赖见 [11 design-to-code](11-design-to-code-and-backend-matrix.md)。那是能力矩阵，不是本目录的第三张表。

---

## 语料漂移（维护时不要假装已经对齐）

产品文档与表 B 语料正文已随 **2026-08-30 owner 批准 v9** 对齐。下列只保留仍未改、且不得用本索引默改的层。

| 张力 | 当前产品/v9 | 状态 |
|---|---|---|
| 创建顺序 | process → members | **已随 v9 批准对齐**（产品文档与 04）。workshop Q&A 仍是历史 members-then-process 快照，不覆盖当前 chrome |
| Today 默认块 | 决策包 + 运行概览（已上线项目行）+ 助手；无待拍板则收起决策包 | **已随 v9 批准对齐**（[03](03-today-projects-and-briefing.md)、[web-ui-design](../../../../personal/docs/product/web-ui-design.md)、user-journeys §2） |
| 可见 CEO 轨 | 不是产品 chrome | **已随 v9 批准对齐**（[02](02-information-architecture-and-app-shell.md)、[opc-product-model](../../../../personal/docs/product/opc-product-model.md)）。六步轨只作后端纪律，不是顶栏 |
| 对象名 | Member Runtime（chrome）；Role Runtime Template 仍是可复用配方 | **已随 v9 批准对齐**（产品 chrome 用语）。架构 `project-role-employee.md` / handbook 生成页仍可能写 Employee（pending architecture/handbook reconciliation） |
| Handbook | 派生层；不拥有产品真相 | **已随本提交同步**：chrome 指针改到 v9，双语手写页已更新并刷新指纹；生成页只经生成器，未手改正文 |

---

## 不要当作本索引的事

- 不要领取 `P*-T*`、不要改 `PROGRESS.md` 任务/lease/campaign 行来「完成」某一行。
- 不要把 v9 写成 daemon `/ui/` 或 Linux 1.0 已交付面。
- 不要覆盖 v8。不要把 v5–v7 或 archived canvas 改回当前 chrome。
- 不要把 native mobile、pairing、E2E relay、X connector、engine store、成员级预算写成已交付。
- 不要为了维护一行而重跑 `/personal-20-prototype-review` 的 phase 4 全量重生，除非 owner 要求下一版原型。
