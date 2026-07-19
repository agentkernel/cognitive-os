# CognitiveOS Console v2 产品简报

> 状态：Draft / planned
>
> 范围：Windows v1 产品方向
>
> 依据：[决策记录](./decision-log.md)
>
> 地位：Informative，不代表实现已提供、测试已执行或 Profile 已符合

## 1. 一句话定义

CognitiveOS Console 是面向 Agent 操作者的 Windows 可视化客户端：用户可以用自然语言开始工作，同时始终看见任务正在做什么、由谁授权、能做多远、何时需要介入，以及结果是否已被 authority 验证和接受。

Console 不是 CognitiveOS authority、Runtime、节点、IdP 或确定性 Admin CLI。它显示 authority projection、形成受治理 proposal、提交用户决定并监督结果；它不能凭本地状态宣称提交、批准或完成。

## 2. 要解决的问题

现有 Agent 界面通常在两个极端之间摇摆：

1. 对话体验简单，但隐藏了权限、预算、外部写入、后台执行和结果不确定性；
2. 管理界面信息完整，但以协议对象和后台表格为中心，普通操作者难以完成日常任务。

Windows v1 要解决的核心问题是：**在不牺牲治理语义的前提下，让操作者用接近现代 AI 助手的方式开始、监督和收敛真实任务。**

## 3. 用户与 Jobs to Be Done

### 3.1 首要 persona：Agent 操作者

典型特征：

- 熟悉自己的目标和工作资料，但不要求理解每个 CognitiveOS 机器对象；
- 会启动持续数分钟到数小时的任务；
- 需要在必要时纠偏、暂停、检查证据或恢复；
- 会安装和更新 Agent，但不应被迫成为平台安全专家；
- 对“是否真的完成”“关闭界面后还会发生什么”有高确定性需求。

首要 JTBD：

1. **继续工作**：回来后快速恢复最近对话和任务，不把聊天记录误当任务状态。
2. **委派工作**：描述目标、资源和约束，审阅结构化预览后创建任务。
3. **保持控制**：知道任务当前状态、剩余边界和失联策略，并能暂停、纠偏或安全遏制。
4. **判断结果**：区分候选完成、验证通过、用户验收和结果未知。
5. **管理能力**：从任意来源检查 Agent，安全地安装、升级、回滚或卸载。

### 3.2 次要 persona 模板

以下是产品导航和研究用模板，不是固定 RBAC role：

- 本地 Owner：首次设置、账号恢复、Agent 生命周期和节点健康；
- 平台运维：节点、执行、恢复和系统状态；
- 审批人：处理独立 proposal；
- 安全审计员：调查和导出证据；
- Agent 发布者：发布包、兼容性和回滚。

组织可把自定义角色映射到一个或多个 persona。Console 只能根据 authority 返回的权限和可发现对象决定入口；不能仅凭 persona 名称授予能力。

## 4. Windows v1 价值主张

### 4.1 对话优先，但不是聊天外壳

- 正常就绪且有权使用任务通道的操作者进入 Shell。
- 对话是启动和理解工作的主画布；提交至少返回稳定 Task/AgentExecution 引用，Intent/Loop/Effect 等引用只在 authority 实际创建并返回时出现。
- 管理能力通过明确入口和独立权限边界出现，不由自然语言措辞自动升级。

### 4.2 后台任务仍然可理解、可暂停

- 每个任务显示当前阶段、authority、更新时间、等待原因和安全动作。
- 托盘模式继续监督 lease；显式退出触发“退出并请求暂停”，有界等待 authority 接受而不误报最终暂停。
- lease 到期只产生 `pause_pending`，直到 authority 确认安全检查点暂停，UI 才显示“已暂停”。

### 4.3 Agent 生命周期既开放又有门禁

- Catalog、URL、Git、本地文件都可以成为“获取并检查”的输入。
- 来源未知不会被界面伪装成已验证。
- 能否运行由签名/provenance、兼容性、sandbox、权限和 authority risk floor 共同决定。
- Windows v1 只执行 R0/R1；R2/R3 清晰阻断。

### 4.4 状态与证据不被“助手感”隐藏

- 用户首先看到普通语言摘要，再按需展开机器术语、强引用和证据。
- Agent 文本永远是数据；系统状态、proposal 和操作控件由 Console 的受控组件渲染。
- `CANDIDATE_COMPLETE`、`COMPLETED`、`OUTCOME_UNKNOWN` 等关键差异不可被“已完成”一句话抹平。

## 5. 产品原则

1. **先回答用户问题，再展示系统结构**
   页面先回答“正在发生什么、我该做什么”，机器对象和协议字段放在来源与详情中。

2. **控制权必须可见且真实**
   每个写入口显示目标、变化、风险、预算、失联策略、可取消边界和结果验证；不能暗示客户端拥有不存在的控制权。

3. **来源比装饰重要，但来源不应淹没任务**
   authority、version、`as_of` 和稳定引用以一致的 Trust Strip/详情入口呈现；普通状态保持简洁，高风险或陈旧状态自动提升可见性。

4. **不确定性是一级状态**
   结果未知、暂停待确认、刷新中使用旧快照、权限裁剪和部分结果都必须有独立表达。

5. **开放输入不等于开放执行**
   自然语言、任意包来源和外部内容可以进入受控检查路径，不能绕过解析、验证、风险下界和 authority commit。

6. **失联时收窄能力**
   无 authority 连接时只显示标有时间的内存快照；不提交新写，不把不可达翻译为已暂停、已停止或已完成。

7. **安全路径也必须可访问**
   键盘、Narrator、高对比、缩放和 reduced motion 用户可以完成与其他用户等强度的确认、暂停、恢复和对账。

## 6. Windows v1 非目标

- 不支持 macOS、Linux、iOS、Android 正式发行。
- 不支持远程节点、多节点和多工作区。
- 不支持 R2/R3 执行或聊天/通知批准。
- 不提供完整 Approval Center、Audit、Users & Access、Memory、Knowledge 或 Multi-Agent 工作区。
- 不把 Console 变成 identity authority、CognitiveOS 节点或最终安全仲裁器。
- 不允许敏感正文离线落盘。
- 不承诺 Tauri 2 + React/TypeScript 已被 ADR 批准。
- 不宣称任何已登记 REQ 已实现或任何 vector 已执行。

## 7. 成功定义

### 7.1 北极星指标

**每位周活跃操作者成功收敛的受验证任务数**

计数条件：

- Task 由适用 authority 进入 `COMPLETED`；
- 关联 Verification 当前有效；
- AcceptanceDecision 已由正确 authority 提交并与当前固定后态匹配；
- 客户端点击、Agent 文本或本地缓存不能单独计数；
- 重复重放、失败后新建的替代任务按稳定 Task ref 去重。

### 7.2 首版辅助指标

- 首次连接到第一个受验证结果的时间；
- `pause_requested → authority paused` 的时延与失败率；
- 退出时仍有活动任务但用户未理解后果的比例；
- R1 preview 的理解正确率和放弃/修改范围比例；
- Agent 安装/升级成功率，按 gate 失败原因分层；
- `OUTCOME_UNKNOWN` 到稳定收敛的时间；
- Narrator + 键盘完成关键旅程的成功率；
- 崩溃、断线、state store/audit 降级下的错误完成声明数（目标必须为零）。

### 7.3 指标约束

- 未获得真实基线前不虚构数值目标；Alpha 先建立采集完整性和分母。
- 产品指标不得抵消安全失败。跨用户泄露、错误批准、重复 Effect、客户端伪造完成单独作为 release blocker。
- 所有服务端结果指标使用 authority timestamp/ref；客户端遥测只描述界面行为。

## 8. 设计与发布假设

- 工作名：CognitiveOS Console；Public Beta 前需品牌决策。
- 品牌性格：亲和易懂、未来感、强大可控。
- 首发语言：简体中文和英文。
- 技术候选：Tauri 2 + React/TypeScript；只有在 Windows Service/IPC、WebView 隔离、升级、可访问性和资源预算 PoC 通过并形成 ADR 后才冻结。
- 当前仓库中的 Console 状态仍为 `planned`；本文只收敛产品方向。
