# CognitiveOS Console v2 — 信息架构

> 范围：Windows v1
>
> 原则：任务语言优先，机器语义可追溯；角色与 readiness 决定落点，不授予权限
>
> Agent Hub 关系：本文的六分组导航不变；Agent Hub 专属 shell 与手机 companion 导航在 [agent-hub/product/product-design.md](./agent-hub/product/product-design.md#6-信息架构pc) 与 [agent-hub/product/journeys-and-screens.md](./agent-hub/product/journeys-and-screens.md)，为扩展视图，不改写此处一级入口。

## 1. 导航模型

Windows v1 使用六个稳定任务分组，而不是把每个协议对象变成一级入口：

| 一级入口 | 用户问题 | Windows v1 内容 | 主要机器对象 |
|---|---|---|---|
| 工作 | 我现在要做什么？ | Shell、Conversation、最近工作 | Conversation、UserIntentRecord、Task |
| 任务 | 正在发生什么，需要我做什么？ | 任务列表、详情、暂停/恢复、结果收敛 | Task、Loop、AgentExecution、Effect、Verification |
| Agent | 我可以使用哪些能力，如何更新？ | 来源、检查、安装、升级、回滚、卸载 | AgentPackageManifest、AgentInstallation、CompatibilityReport |
| 收件箱 | 哪些事项正在等我？ | 等待输入、R1、结果未知、系统降级、session/通知 | proposal refs、Event、notification refs |
| 记录 | 为什么会这样，依据是什么？ | 当前对象 Activity、来源、关键事件和安全记录 | Event、StateTransitionRecord、authority projection |
| 系统 | 本机节点是否可用？ | Service、readiness、store/audit/watch/sandbox/update、账号设置 | readiness/health（产品依赖）、Session |

规则：

- 权限可以隐藏正文或禁用动作，但不把同一概念随机移动到另一个入口。
- Windows v1 的“记录”只提供当前工作相关的最小时间线，不是完整审计工作区。
- R2/R3、Memory、Knowledge、Collaboration、企业 Users & Access 不进入 v1 一级导航。
- 导航项显示当前用户可理解的名称；机器对象名在详情标题、来源面板和链接中出现。

## 2. 路由树

```text
/setup
  /service
  /trust
  /identity-change
  /owner
  /signin

/work
  /conversations
  /conversation/:conversationRef
  /preview/:proposalRef

/tasks
  /:taskRef
  /:taskRef/effects/:effectRef
  /:taskRef/verification

/agents
  /sources/new
  /packages/:packageRef
  /installed/:installationRef
  /lifecycle/:proposalRef

/inbox
  /:itemRef

/records
  /object/:objectRef

/system
  /overview
  /account
  /sessions
  /notifications
  /update
```

路由是产品词汇，不声明 HTTP/API path。`objectRef` 等参数只表示稳定引用；实际 URI grammar 和 transport 必须来自已登记合同。

## 3. 启动落点

按以下优先级选择落点：

1. **Service 不存在/不可验证** → `/setup/service`
2. **已固定节点身份不匹配** → `/setup/identity-change`，阻断登录
3. **首次节点身份未固定** → `/setup/trust`
4. **节点处于 bootstrap 且没有 Owner** → `/setup/owner`
5. **无有效 AuthenticationSession** → `/setup/signin`
6. **安全深链有效** → 先恢复 session/authorization，再打开固定对象
7. **存在必须立即处理的安全状态** → `/inbox/:itemRef`
8. **仅 management readiness** → `/system/overview`，只显示节点实际声明且用户有权使用的确定性操作
9. **Agent 操作者且任务通道可用** → 最近 Conversation；无最近项则 `/work`
10. **其他 persona** → 其首要授权入口；没有可用入口时显示明确的受限首页，而非空 Shell

`MANAGEMENT_READY` 不等于“全部管理视图可用”。Console 必须按节点返回的 operation/capability/readiness projection 逐项开放。

## 4. Shell 信息结构

### 4.1 主画布

主画布包含：

- Conversation 标题、当前节点/账号简要身份；
- 消息流；
- 受控系统卡；
- 输入器、附件和目标选择；
- 当前 proposal/preview；
- 最近提交后的稳定 Task 引用。

不在消息流中常驻完整五轨、所有权限或全量 Context。它们进入侧栏或对象详情；只有安全关键状态自动提升。

### 4.2 左侧导航

- 展开态：六个一级入口 + 最近 Conversation；
- 收起态：图标 + 可访问名称；
- 最近 Conversation 只显示用户有权发现的标题/元数据，不展示无权正文；
- 切换 Conversation 不继承未固定资源引用或管理 session。

### 4.3 任务/上下文侧栏

侧栏使用页签或分段：

- 当前任务：状态摘要、下一 gate、deadline、预算、失联策略；
- 来源：Context 来源、权限、freshness、被拒绝项；
- 活动：近期 authority 事件；
- 详情：机器术语、强引用、版本和证据。

侧栏默认可折叠。以下状态打开时必须保持显著、可直接返回：

- R1 proposal；
- `OUTCOME_UNKNOWN`；
- `pause_pending` 或失联 lease 临界；
- stale/offline/partial authority projection；
- store/audit degraded；
- Agent lifecycle gate 失败。

### 4.4 响应式窗口

Windows 窄窗口仍是完整桌面产品，不自动变成移动伴侣：

- 宽窗口：导航 + 主画布 + 侧栏；
- 中等窗口：导航可收起，侧栏变为可停靠抽屉；
- 窄窗口：主画布单列，导航和详情用独立 overlay/drawer；
- 能力范围不因 viewport 改变；只有布局改变。

## 5. 页面层级

每个对象页使用四层信息：

1. **用户摘要**：发生了什么、是否需要行动；
2. **行动区**：一个推荐主动作 + 必要安全替代；
3. **事实区**：状态、时间、目标、预算、来源和影响；
4. **技术详情**：机器状态、对象类型、稳定引用、authority、version、digest、证据链接。

禁止把稳定 ID、digest 和五个状态同时堆在每张普通列表卡；高风险、未知和审计场景可以展开显示。

## 6. 用户术语与机器术语

| 用户主标签 | 机器术语/详情 | 使用规则 |
|---|---|---|
| 任务 | `Task` | 用户目标的持久工作单元 |
| 执行 | `AgentExecution` | 任务的一次 Agent 执行身份；不等于进程 |
| 运行载体 | Runtime/PID/container（合同待登记） | 只在诊断和停止载体时显示 |
| 外部影响 | `Effect` | 任何可能影响外部世界的受治理动作 |
| 验证 | `Verification` | 检查固定后态；不等于用户验收 |
| 候选完成 | `CANDIDATE_COMPLETE` | Agent 认为可验收，尚未成为完成 |
| 已完成 | `COMPLETED` | authority 已接受适用验证/验收 |
| 结果未知 | `OUTCOME_UNKNOWN` | 可能已发生，禁止盲重试 |
| 暂停请求中 | `pause_pending` 产品投影 | 请求已接收但未在安全检查点确认 |
| 能力 | Agent / Operation | 用户层按“能做什么”描述，技术详情区分二者 |
| 记录 | Event/Transition/Audit | 普通活动与正式审计完整性不能混称 |

中英文文案必须从统一术语表生成，不拼接半句。机器 enum、错误码、REQ-ID 和 digest 保留原文，旁边提供本地化解释。

## 7. 搜索与对象定位

### 7.1 搜索范围

- 当前页面搜索：只过滤当前授权数据；
- 全局搜索：Conversation、Task、Agent、安装记录和可发现系统事件；
- 默认不搜索无权正文，不通过结果数量泄露对象存在性；
- 搜索结果显示类型、用户摘要、节点、更新时间和授权状态。

### 7.2 打开规则

- 打开结果时重新授权，不依赖搜索时权限。
- 写操作必须解析为唯一固定版本强引用。
- 歧义、版本漂移、权限变化或不可发现时 fail closed。
- 最近项、收藏和 pin 只保存引用，不缓存其正文或权限。

### 7.3 命令面板

- `Ctrl+K` 打开命令面板；
- 只列当前视图或 authority 声明可能的动作；
- 命令面板选择写操作后仍进入 preview/gate，不直接执行；
- “停止”“取消”“回滚”“重试”等词必须先显示作用对象和不保证事项。

## 8. 收件箱模型

Windows v1 收件箱按用户需要行动的原因分组：

- 需要输入；
- 需要 R1 确认；
- 暂停/退出待收敛；
- 结果未知/需要对账；
- Agent lifecycle gate 失败；
- session/权限变化；
- Service/store/audit/watch 降级；
- 普通进度摘要。

状态区分：

- `unread`：用户尚未查看；
- `acknowledged`：已知悉，只影响提醒；
- `handled`：底层 authority 事项已收敛；
- `expired/superseded`：不可再处理。

`acknowledged` 绝不能让未处理的 `OUTCOME_UNKNOWN`、pause pending 或安全故障从待办队列消失。

## 9. 通用页面状态

所有页面必须从以下状态集合选择，而不是只实现“有数据/错误”：

- `initial-loading`
- `refreshing-last-good`
- `authoritative-empty`
- `filtered-empty`
- `partial`
- `redacted`
- `stale-offline`
- `permission-denied`
- `submitting`
- `result-unknown`
- `conflict/superseded`
- `success`
- `service-error`
- `privacy-locked`
- `reauth-required`

页面标题、主动作和 live region 必须与状态一致。Skeleton 不能包含可点击假控件；刷新 last-good 时现有动作根据 freshness 和 authority 决定是否禁用。

## 10. 未来扩展

Windows v1 不冻结 Approval、Audit、企业 Users & Access、独立 Executions 或移动导航的位置。这些扩展进入各自 discovery 后再验证 persona、JTBD 和导航，不得让当前主导航出现空入口。未来范围与不可变边界见 [roadmap.md](./roadmap.md)。
