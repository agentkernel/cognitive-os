# CognitiveOS Console v2 — Design System 方向

> 状态：Product design candidate
>
> 范围：Windows v1
>
> 品牌关键词：亲和易懂、未来感、强大可控
>
> Agent Hub 关系：Agent Hub 复用本设计系统，并额外要求 Direct/Governed 模式、接管层级与事实来源的视觉不可混同（见 [agent-hub/product/states-content-and-accessibility.md](../../../agent-hub/docs/product/states-content-and-accessibility.md)）；不新增并列设计系统。

本文定义可进入原型的设计方向，不声称 token 已通过视觉回归、WCAG 或品牌审批。实现前必须在真实 WebView2、Windows High Contrast 和中英文内容上验证。

## 1. 设计命题

### 1.1 “会说人话的控制界面”

Console 首先像一个清晰、可协作的现代助手：对话是主画布，系统主动解释正在发生什么和下一步。治理信息不是后台表格，也不被隐藏；它以稳定、可展开的结构附着在任务和动作上。

### 1.2 标志性元素：Governed Flow Thread

产品的记忆点不是渐变、发光或机器人插图，而是一条“受治理任务线”：

```text
目标固定 ── 执行中 ── 验证中 ── 已接受
              │
              ├─ 等待输入
              └─ 结果未知 → 对账
```

- 普通状态显示四个用户阶段，不用五个机器状态挤满主界面。
- 展开后显示 Task / Loop / AgentExecution / Effect / Verification 的独立状态。
- 只有收到新的 authority event 时，节点之间发生一次短促方向性过渡。
- `OUTCOME_UNKNOWN` 不沿主线“继续前进”，而进入清晰分支。
- reduced motion 模式使用静态位置、文本和“已更新”标记。

这条线同时表达产品的未来感和可控感，不能被用于装饰性环境动画。

## 2. 视觉基础

### 2.1 候选色板

候选浅色主题：

| Token | 候选值 | 用途 |
|---|---:|---|
| `canvas` | `#F3F7F6` | 主背景，轻微冷绿而非纯灰 |
| `surface` | `#FFFFFF` | 主画布、输入和关键容器 |
| `ink` | `#162022` | 主要文本 |
| `muted-ink` | `#536064` | 次要文本 |
| `line` | `#D9E2E0` | 结构分隔 |
| `brand-700` | `#075F63` | 主动作、焦点、选中 |
| `brand-500` | `#0B8588` | 图形和强调 |
| `brand-200` | `#A9DEDA` | 低强调背景，不承载正文 |
| `signal-blue` | `#2D62D0` | 等待/信息，不作为品牌主色 |
| `signal-amber` | `#9A5B00` | 阻塞/注意 |
| `signal-red` | `#B42336` | 结果未知/危险 |
| `signal-green` | `#16704A` | authority 已确认的成功 |

候选深色主题：

| Token | 候选值 | 用途 |
|---|---:|---|
| `canvas` | `#0E1516` | 主背景 |
| `surface` | `#151F21` | 主画布 |
| `ink` | `#ECF3F1` | 主要文本 |
| `muted-ink` | `#A7B6B3` | 次要文本 |
| `line` | `#2B393A` | 结构分隔 |
| `brand-500` | `#58C9C4` | 主动作和焦点 |

约束：

- 以上只是品牌候选；所有组合须实测 WCAG 2.2 AA 和 Windows High Contrast。
- Risk 色不作为品牌按钮色；主动作不能看起来像危险操作。
- 状态始终同时提供文字、形状/图标和位置。
- 禁止用紫色渐变、霓虹光晕或全屏 mesh 作为“AI”暗示。

### 2.2 字体方向

- Display / 品牌标题候选：`Sora`，只用于少量 H1、空状态和 onboarding；
- UI / 正文：`Segoe UI Variable`，中文回退 `Microsoft YaHei UI`；
- 技术值：`Cascadia Mono`，用于 digest、ID、错误码和固定版本；
- 后续跨平台品牌包需通过字体许可、CJK 覆盖、bundle 和离线部署评审。

排版尺度：

- H1：28/36，单页一个；
- H2：20/28；
- H3：16/24；
- Body：14/21（阅读密集页可 15/22）；
- Label：12/18，禁止低于 12 px；
- 技术值：12/18 mono，可复制、可完整展开。

标题使用 sentence case；中文不额外加空格。按钮使用明确动词。

### 2.3 间距、圆角与深度

- 4 px 为最小单位，主要布局使用 8 px 网格；
- 页面间距：24/32；组件间距：8/12/16；关联字段：4/8；
- 普通控件圆角：6；卡片/抽屉：10；大 onboarding surface：14；
- 卡片使用边框或层级背景，避免同时使用厚边框和阴影；
- 阴影只用于真正浮在内容之上的 popover/modal，不用于每张卡；
- 默认最小点击目标 40 × 40 CSS px；触摸/辅助模式提升到 44 × 44。

## 3. 布局

### 3.1 Shell

```text
┌──────────────┬────────────────────────────┬──────────────────┐
│ 工作/任务…   │ Conversation               │ 当前任务/来源    │
│ 最近对话     │                            │ （可折叠）       │
│              │ 消息与系统卡               │                  │
│              │                            │                  │
│              │ 输入器                     │                  │
└──────────────┴────────────────────────────┴──────────────────┘
```

- 主画布拥有最大宽度和视觉权重。
- 右侧栏不是永久仪表盘；无当前任务时默认收起。
- 左导航可以收起，但当前 workspace/node 身份始终可找到。
- 提案、结果未知和 pause pending 可把侧栏固定到可见状态，用户处理后恢复。

### 3.2 列表与详情

- Task/Agent 列表使用稳定行布局而不是均匀卡片墙。
- 标题和推荐动作左侧对齐；状态/时间使用固定列。
- 数值右对齐，deadline 使用绝对时间并可查看相对时间。
- 详情页首屏只放摘要、Flow Thread、推荐动作和关键约束；历史/证据在后续区段。

### 3.3 Overlay 选择

- Modal：只用于短、单一、可立即取消的决定；
- Docked drawer：补充详情、来源、筛选，不覆盖主内容，不 trap focus；
- Modal drawer：窄窗口覆盖主内容时必须有可见关闭按钮、初始焦点、focus trap、背景 `inert`/不可操作和关闭后焦点恢复；
- 独立页面：安装/升级/回滚/卸载、复杂 preview、结果未知调查；
- Popover：无副作用的短选项；
- 禁止 modal 内再打开 modal。

## 4. 核心组件

### `CONSOLE-V2-CMP-001` App Frame

- 六分组导航、workspace/node 身份、主内容、全局安全状态；
- 提供“跳到主内容”“跳到当前任务/安全事项”；
- 路由切换后把焦点移动到新页面 H1 或明确目标。

### `CONSOLE-V2-CMP-002` Conversation Canvas

- Agent/用户消息、系统卡和输入器在视觉上组成主画布，但不可信内容与系统控件不共享安全边界；
- Agent Markdown/HTML 在独立低权限 WebView/进程或等价隔离面中渲染，不能创建真实按钮、菜单、系统图标或管理横幅；平台无法证明隔离时降级为纯文本/安全子集；
- 长内容提供结构化折叠和“在独立视图打开”。

### `CONSOLE-V2-CMP-003` System Card

- 只由 Console 根据 authority projection 构建；
- 顶部显示可验证来源标记、用户摘要、状态和时间；
- 动作区域位于系统 renderer/native surface；与 Agent 内容不共享 WebView/进程、credential 或 IPC capability；
- 密码、bootstrap secret 和 R1 控件不能与不可信内容同一 renderer；
- 恶意内容可以仿画外观，但不能获得真实系统 surface、IPC 或提交能力，因此文档不再承诺“视觉上不可复刻”。

### `CONSOLE-V2-CMP-004` Trust Strip

- 紧凑显示节点、authority、version、`as_of` 和 freshness；
- 正常时低强调；stale、身份变化、跨节点或高风险时提升；
- 点击打开完整稳定 ref、digest、权限与证据；
- 不能仅靠环境色区分生产/测试。

### `CONSOLE-V2-CMP-005` Governed Flow Thread

- 用户阶段摘要 + 可展开机器轨道；
- 每个节点拥有文本标签和可访问描述；
- `candidate complete` 与 `completed` 使用不同节点和文案；
- 未知分支提供 Reconcile 入口，不提供 Retry。

### `CONSOLE-V2-CMP-006` Task Summary

- 标题、状态摘要、下一 gate、deadline、监督 lease、推荐动作；
- 默认不显示全部 ID；
- 紧凑模式不隐藏 `OUTCOME_UNKNOWN`、pause pending 或 freshness。

### `CONSOLE-V2-CMP-007` Command Preview

- 结构化分区：目标、变化、数据/出域、风险与外部影响（技术详情 `Effect`）、预算、验证、失联/取消/补偿；
- R1 动作按钮写明对象和动作；
- stale 时禁用原按钮并显示差异；
- initial focus 在标题，批准不是 Enter 默认动作。

### `CONSOLE-V2-CMP-008` Agent Source & Evidence

- 分开呈现 source、resolved content digest、signature、provenance、dependencies、scan、compatibility；
- `unknown` 使用中性问号/文本，不当作通过；
- 证据适用 host/sandbox/adapter version 必须可见。

### `CONSOLE-V2-CMP-009` Lifecycle Diff

- 安装：新增能力和资源；
- 升级/回滚：old/new 并列，突出权限、数据格式、兼容性和任务影响；
- 卸载：运行依赖、数据保留和未决外部影响（`Effect`）；
- 数值、权限和删除项可由屏幕阅读器按组读取。

### `CONSOLE-V2-CMP-010` Inbox Row

- 固定图标槽、标题、原因、deadline、来源和状态；
- 焦点/展开/选择期间冻结排序；
- 动态更新以“有 N 项更新”提示，不移动当前目标。

### `CONSOLE-V2-CMP-011` Status Notice

- Alert 用于持续重要状态；Toast 只用于短暂确认；
- Toast 4–6 秒可关闭，但安全事项必须同时存在于 Inbox/页面；
- Error 说明发生了什么、哪些动作受影响、用户能做什么；不使用“出了点问题”。

### `CONSOLE-V2-CMP-012` Data Table

- sticky header；数字右对齐；可排序列明确；
- 虚拟化不能破坏语义行、焦点或屏幕阅读器位置；
- 横向滚动时固定对象和状态列；
- 支持舒适/紧凑密度。

## 5. 控件规则

### 按钮

- 每个页面/区段至多一个主按钮；
- 标签使用动作 + 对象：“暂停任务”“安装 Agent v2.1”；
- Destructive action 放最后，必须解释范围；
- 禁用控件同时显示原因和解除条件；
- 能立即生效的偏好用 Toggle；需要 Save 的表单用 Checkbox。

### 表单

- 单列优先，标签始终可见；
- placeholder 只作格式提示；
- 失焦后 inline validation，提交时错误摘要链接到字段；
- 账号、bootstrap secret、URL/Git 和 package source 使用不同输入语义；
- 粘贴不可信内容不能改变系统控件。

### 菜单与快捷键

- `Ctrl+K` 命令面板；
- 破坏性动作无单键快捷键；
- `Esc` 退出 modal/modal drawer 而不产生决定；所有 overlay 同时提供可见关闭按钮，关闭后焦点返回触发控件；
- 所有快捷键可在帮助中发现，不能覆盖 Narrator/Windows 保留组合。

## 6. 动效系统

### 6.1 性格

“快速理解、稳健落位”。不使用弹跳作为默认，不用持续脉冲制造紧迫感。

| Token | 时长 | 用途 |
|---|---:|---|
| `motion-instant` | 90 ms | hover/focus/press |
| `motion-quick` | 160 ms | icon、badge、短反馈 |
| `motion-standard` | 240 ms | drawer、card/state change |
| `motion-context` | 360 ms | 页面/大上下文切换 |

候选 easing：

- entrance：`cubic-bezier(0.2, 0, 0, 1)`；
- exit：`cubic-bezier(0.3, 0, 1, 1)`；
- on-screen state：`cubic-bezier(0.4, 0, 0.2, 1)`。

### 6.2 签名动效

- authority event 到达后，Flow Thread 的当前节点在 240 ms 内沿方向更新；
- 先更新状态文字，再移动连接/节点，避免动画先于事实；
- 总动效不超过 400 ms，不做逐卡长 stagger；
- 未收到 authority event 时不播放“处理中”推进动画；
- `OUTCOME_UNKNOWN` 使用分支显现而不是红色震动。

### 6.3 reduced motion

- 禁用位移、缩放、parallax、自动滚动和连续 spinner；
- 使用瞬时状态替换 + 轮廓高亮 + live announcement；
- skeleton 不 shimmer；
- Flow Thread 保持完整静态语义。

## 7. 无障碍

- 目标：WCAG 2.2 AA；
- 关键旅程：Service 设置、登录、创建任务、R1、暂停、结果未知、Agent 更新；
- Windows v1 基线：Narrator + 键盘 + High Contrast + 200%/400% zoom；
- landmark、单一 H1、正确 heading/table/list；
- 主内容、当前任务和安全事项提供 skip/focus target；
- live region 聚合高速事件；不逐秒朗读倒计时；
- 图标全部有名称；装饰图标从语义树隐藏；
- focus ring 在品牌主题和 High Contrast 下均可见；
- 颜色、动画、hover、拖动都不能是唯一操作路径。

## 8. 国际化与内容

- 首发 `zh-CN` 与 `en`；
- 使用完整 ICU message，不拼接句子；
- 预留 30–50% 文案扩张；按钮允许换行但不截断关键动词；
- 机器 enum/error code/digest 不翻译，提供本地化解释；
- 普通页面显示本地时间，可展开 UTC；安全 deadline 同时给绝对时间；
- Unicode/Bidi 控制符在可信字段中隔离/标记，稳定 ID 常驻可复制；
- 域名显示规范化值，必要时同时显示 punycode。

## 9. 禁止的通用 AI UI 模式

- 紫色/蓝色渐变 + 发光球体作为默认品牌；
- 每个对象都是同尺寸圆角卡；
- 所有元素 hover scale；
- 无语义的数字 `01/02/03` 装饰；
- 持续脉冲“AI 正在思考”；
- 大量彩色状态 badge；
- 用聊天气泡伪装 proposal/approval；
- “Submit”“Continue”“Something went wrong”等无上下文文案；
- 用模型生成插图替代真实空状态说明；
- 用动画暗示服务端已经提交或暂停。

## 10. Design System 验收

实现前需产出并评审：

1. 浅/深/High Contrast token 对比矩阵；
2. 中英文排版、长名称、digest 和错误码；
3. App Frame、System Card、Trust Strip、Flow Thread、Preview、Lifecycle Diff；
4. loading/empty/partial/stale/offline/denied/unknown；
5. Narrator 焦点与 live region 走查；
6. reduced-motion 视觉等价；
7. 恶意 Agent 内容仿冒系统卡的安全/可用性测试；
8. 375/768 仅用于布局压力测试，不把窄桌面降级成移动产品。
