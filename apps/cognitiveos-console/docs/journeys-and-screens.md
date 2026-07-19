# CognitiveOS Console v2 — Windows v1 旅程与页面规格

> 状态：Draft / planned
>
> 范围：Windows v1
>
> 约束：页面只显示 authority projection；产品 ID 不等于已登记机器合同

## 1. 旅程规格模板

每条旅程必须包含：

- persona 与触发；
- 前置条件和 authority 依赖；
- 页面序列；
- 用户决定；
- 成功结果；
- 失败/取消出口；
- 恢复入口；
- 可访问性与观测点。

## 2. 核心旅程

### `CONSOLE-V2-JRN-001` 安装 Service 并领取首个 Owner

**触发**：用户首次启动，未发现受信 Windows Service。

**页面**：`PAGE-001 → PAGE-002 → PAGE-018 → PAGE-003`

1. Console 验证安装器签名、版本和发布者。
2. 用户审阅安装范围、Service 权限、数据位置和网络暴露。
3. 安装器经 UAC 安装 Service；失败时不留下“已安装”假状态。
4. Admin CLI/安装器生成一次性 bootstrap bundle（endpoint key、目标 Windows SID、secret）。
5. Console 验证实际 Service endpoint key 与 bundle 匹配；该步骤与普通 TOFU 页面分开。
6. 用户在 Owner 领取页输入账号信息；Console 仅向已匹配 endpoint key 的 Service 提交 secret。
7. Service 从 IPC peer token 取得 SID，原子领取 Owner、消费 secret并签发 AuthenticationSession。
8. Console 从 authority 取初始 snapshot 后进入工作首页。

**失败出口**：

- Service 签名/版本不可信：阻断，提供删除/修复入口；
- 身份与已固定身份不一致：进入 `PAGE-019` 阻断恢复，不提供普通忽略；
- secret 过期/已使用：回到 Admin CLI 生成新 secret；
- Owner 已存在：进入登录，不重新 bootstrap；
- store/audit 不可写：Owner 创建 fail closed。

**验收**：并发两个首次领取只能有一个成功；低权限用户仅凭先连接不能成为 Owner。

### `CONSOLE-V2-JRN-002` 登录并恢复上次工作

**触发**：已有 Service 与 Owner/用户账号。

**页面**：`PAGE-003 → PAGE-004`，必要时进入 `PAGE-014/015`

1. Console 验证固定节点身份与 readiness。
2. 用户登录；节点签发短期、SID-bound AuthenticationSession。
3. Console 获取授权 snapshot、最近 Conversation refs 和 watch cursor。
4. 若有安全待办，先展示不可忽略摘要；否则打开最近 Conversation。
5. 最近聊天内容与 Task 状态分别恢复，不能用聊天缓存推断任务结果。

**失败出口**：

- 固定节点身份变化：立即进入 `PAGE-019`，不显示登录表单或恢复敏感 projection；
- 密码错误：限速、无账号存在性泄露；
- 忘记密码：只引导 Admin CLI；
- session 失效：保留无敏感页面框架，清理正文并重新登录；
- 仅 management readiness：进入 `PAGE-015`，只开放节点实际声明的确定性入口。

### `CONSOLE-V2-JRN-003` 从 Shell 创建任务

**页面**：`PAGE-004 → PAGE-005 → PAGE-007`

1. 用户输入目标、资源和期望结果。
2. Console 先保存草稿；authority 固定 UserIntentRecord。
3. 若有实质歧义，显示选项、影响和缺失信息，而不是猜测。
4. authority 返回固定目标、风险下界、预算、deadline、验证和失联策略。
5. 用户在 Preview 中修改范围或确认。
6. R0 自动提交或 R1 结构化确认；提交后至少返回稳定 Task/AgentExecution refs，Intent/Loop/Effect refs 只在 authority 实际创建时返回。
7. UI 进入任务详情并开始 snapshot + watch。

**失败出口**：

- 目标歧义/不存在/版本漂移：重新选择；
- preview stale：显示差异，旧确认失效；
- R2/R3：禁用提交，允许缩小范围；
- state store/audit unavailable：fail closed；
- 提交响应丢失：按稳定 proposal/idempotency ref 查询，不生成新 key 盲重试。

### `CONSOLE-V2-JRN-004` 监督、纠偏和暂停任务

**页面**：`PAGE-004/006 → PAGE-007`

1. 用户查看普通语言摘要、下一 gate、预算、deadline 和 lease 状态。
2. 需要补充输入时，新增受治理输入，不改写历史。
3. 选择暂停时，Preview 解释 pause 的对象、时机和不保证事项。
4. authority 接收后显示“暂停请求中”；安全检查点确认后显示“已暂停”。
5. 恢复任务会创建新的监督 lease，并重新验证 capability/预算/版本。

**验收**：

- 停止 Runtime 不得自动显示 Task 已暂停；
- Effect 在执行中时，pause 解释仍可能发生的外部影响；
- `pause_pending` 超时显示原因和安全遏制入口；
- 所有状态以 authority 事件收敛。

### `CONSOLE-V2-JRN-005` 托盘监督、退出和 lease 到期

**页面**：`PAGE-004/006 → tray → PAGE-014`

1. 关闭主窗口隐藏到托盘并继续 heartbeat。
2. 托盘显示活动任务数量、等待用户数量和连接健康，不显示敏感正文。
3. 用户选择“退出并请求暂停”，Console 列出受影响任务。
4. 用户确认后提交 pause intents，并有界等待 authority 接受；接受后终止续租并退出，不等待最终暂停无限完成。
5. 接受结果超时时默认留在托盘；若用户仍强制退出，显示将依赖 lease 到期，保留稳定 pause request refs 且不宣称已暂停。
6. 意外崩溃/断网时由 Service 观察 lease 到期并进入安全暂停流程。
7. 下次启动先重认证/resnapshot，再显示各任务实际结果。

**验收**：终止进程、断网、睡眠、关机均不能依赖客户端最后一次回调来保证暂停。

### `CONSOLE-V2-JRN-006` 获取并检查任意来源 Agent

**页面**：`PAGE-009 → PAGE-010 → PAGE-013`

1. 用户选择 Catalog、URL、Git 或本地文件。
2. 获取本身形成固定 acquisition proposal/Effect，并由无 ambient credential 的低权限 broker 执行；Service 校验调用者对本地路径的访问、网络/SSRF/UNC policy、重定向和资源预算。
3. Console 把来源当不可信输入；Service 固定 source ref、最终解析来源、内容 digest 和获取证据。
4. Agent 包检查显示签名、provenance、依赖、权限声明、平台目标和未知项。
5. 节点执行静态检查、adapter/sandbox 选择和 compatibility 负例。
6. authority 返回 C 等级、risk floor、缺失证据和允许动作。

**失败出口**：

- 下载/clone/读取失败：保留 acquisition ref 和安全重试建议；禁止用宿主 ambient credential 自动重试；
- digest 变化：旧检查结果失效；
- 签名/provenance 未知：明确显示，不自动等同恶意或可信；
- R2/R3 或 gate 缺失：可查看报告，不能进入执行确认。

### `CONSOLE-V2-JRN-007` 安装、升级、回滚或卸载 Agent

**页面**：`PAGE-010/012 → PAGE-011 → PAGE-012`

共同步骤：

1. 固定当前 installation、目标 package 和 expected version。
2. 显示变化、兼容性、任务影响、数据保留、rollback point 和未决 Effect。
3. authority 计算 R0/R1/R2/R3；Windows v1 只允许 R0/R1。
4. 用户完成适用 R1；Service 执行事务并持续写 authority 状态。
5. 成功后生成新版本 installation；失败时旧版本不被静默覆盖。

操作差异：

- 安装：展示新增权限、资源和出域；
- 升级：并列 old/new，检查运行任务与 schema/adapter 变化；
- 回滚：解释可能无法回滚的数据、Effect 和安全 floor；
- 卸载：先处理运行实例、依赖、数据保留和未决 Effect，不删除历史证据。

**结果未知**：进入 `PAGE-008`，禁止换 idempotency key 再执行。

### `CONSOLE-V2-JRN-008` 完成 R1 结构化确认

**页面**：`PAGE-005` 或 `PAGE-011`

1. 页面冻结目标、参数 digest、版本、risk、预算、egress、deadline 和验证。
2. 批准按钮使用具体动词和对象，例如“安装 Agent v2.1”，不使用“提交”。
3. 按 authority policy 决定是否显示 number matching。
4. 初始焦点在标题/摘要，不在批准按钮；Enter 不默认批准。
5. 用户可以取消、修改范围或确认。
6. 提交后显示 `submitting`；响应未知时按固定 proposal ref 查询。

**可访问性**：number matching 提供无需记忆/精细拖动的输入；倒计时不逐秒轰炸；重发 challenge 后旧值失效。

### `CONSOLE-V2-JRN-009` 处理结果未知

**页面**：`PAGE-007 → PAGE-008`

1. UI 固定显示原始目标、dispatch 证据、idempotency binding 和最后 authority 状态。
2. 禁用“重试”“换 key 重试”“假定失败”。
3. 用户启动授权 reconcile，或查看 Service 自动对账进度。
4. 收敛为 executed 后继续 verification/commit；收敛为 not-executed 时进入终态 `NOT_EXECUTED`，不得验证或提交该 Effect；仍未知时进入 quarantine/安全处置。

**验收**：关闭 Runtime、暂停 Task 或重新安装 Agent 都不能自动清除原 Effect 的未知状态。

### `CONSOLE-V2-JRN-010` store/audit/watch 降级

**页面**：任意页面 → `PAGE-014/015`

- state store/audit 持久化失败：冻结普通写；应急 pause/stop 走独立日志；
- watch delay：显示延迟和 last-good；
- gap/stale cursor：先 snapshot，再增量；
- Service 不可达：只显示进程内存快照，所有写禁用；
- 恢复：重新认证/授权，按稳定 ref 对账，不把本地时间当 authority 时间。

## 3. Windows v1 页面清单

### `CONSOLE-V2-PAGE-001` Service 设置

- 目的：安装、验证、修复或连接本机 Service。
- 主动作：`安装 Service` / `修复安装`。
- 必显：发布者、版本、安装范围、数据位置、网络暴露、权限。
- 禁止：从 renderer 直接运行任意安装命令。

### `CONSOLE-V2-PAGE-002` 节点信任

- 目的：验证安装 bundle endpoint key，或在无 bootstrap 的普通本地连接上完成 TOFU。
- 主动作：`信任此本机节点`。
- 必显：节点短身份、Service 发布者、安装关联和信任来源。
- 不收集账号/secret；身份变化进入 `PAGE-019`。

### `CONSOLE-V2-PAGE-003` 登录与恢复

- 目的：登录节点账号或转入 Admin CLI 恢复。
- 主动作：`登录`。
- 密码不是 approval 输入；Console 不提供“查看已保存密码”。
- 错误不泄露账号是否存在。

### `CONSOLE-V2-PAGE-004` 工作 / Shell

- 目的：继续 Conversation、创建任务、查看当前重点任务。
- 布局：对话主画布 + 可折叠任务/上下文侧栏。
- 主动作：输入目标；同一区域最多一个视觉主按钮。
- 系统卡与 Agent 内容使用不同语义容器和可访问标签。

### `CONSOLE-V2-PAGE-005` 命令预览与 R1

- 目的：理解将发生什么并确认/修改范围。
- 必显：目标、变化、风险等级、外部影响（技术详情 `Effect`）、预算、出域、deadline、验证、失联策略、取消/补偿。
- 状态：fresh/stale/submitting/result-unknown/superseded/expired。

### `CONSOLE-V2-PAGE-006` 任务中心

- 目的：按“需要我处理/运行中/已暂停/最近完成”组织任务。
- 支持舒适/紧凑密度；排序更新时保持焦点/选择项稳定。
- 数值和 deadline 对齐；列表行固定状态、标题、下一动作和更新时间列。

### `CONSOLE-V2-PAGE-007` 任务详情

- 目的：监督 Task、Loop、AgentExecution、Runtime、Effect、Verification。
- 首层是用户摘要和下一动作；机器轨道按需展开。
- `CANDIDATE_COMPLETE`、`COMPLETED`、`OUTCOME_UNKNOWN` 和 `pause_pending` 有独立结构。

### `CONSOLE-V2-PAGE-008` 外部影响对账

- 目的：理解未知外部影响并安全收敛。
- 无普通 retry 主按钮。
- 必显：原 intent、idempotency、dispatch/receipt、reconcile 结果、责任主体和安全限制。

### `CONSOLE-V2-PAGE-009` Agent 中心与来源

- 目的：作为 `/agents` 总览，查看已安装、可更新、处理中和被阻断的 Agent，并输入 Catalog/URL/Git/本地文件开始受控获取。
- 状态：initial-loading、authoritative-empty、partial、source-gate-blocked、update-available、service-error。
- 所有来源字段标为不可信输入；URL/Git 显示最终解析来源和 digest。
- 已识别 Catalog 的目录身份/传输信任不等于包可信；最近来源也不等于受信来源。

### `CONSOLE-V2-PAGE-010` Agent 包检查

- 目的：审阅签名、provenance、manifest、权限、依赖和静态检查。
- 报告清晰区分 known-good / known-bad / unknown / not-applicable。
- 未知不能用绿色“通过”或红色“恶意”替代。

### `CONSOLE-V2-PAGE-011` Agent 生命周期向导

- 目的：安装/升级/回滚/卸载的 preview、R1 和进度。
- 复杂流程使用独立页面/stepper，不放入 modal。
- 失败可回到检查/范围修改；事务结果以 authority installation/effect 为准。

### `CONSOLE-V2-PAGE-012` 已安装 Agent 详情

- 目的：查看当前版本、来源、权限、运行使用、版本历史和 rollback point。
- 主动作根据状态选择“升级”“回滚”或“修复”，不并列多个同权按钮。

### `CONSOLE-V2-PAGE-013` 兼容性报告

- 目的：解释该包在当前 Windows 节点上能做什么、不能做什么。
- C0–C3 与 R0–R3 分轴展示；界面主标签为“兼容等级/风险等级”，机器值在详情保留。准入组合受 authority policy 约束，不暗示全部 16 组合均可执行。
- 证据只适用于明确 host/sandbox/adapter 版本。

### `CONSOLE-V2-PAGE-014` 收件箱与活动

- 目的：处理等待输入、R1、结果未知、pause pending 和系统降级。
- `acknowledged` 只静音/已读，不替代 `handled`。
- Windows 通知只携带一次性 opaque handle；消费后解析固定 item ref 并 resnapshot，重放必须失败。

### `CONSOLE-V2-PAGE-015` 系统概览

- 目的：显示 Service、readiness、store、audit、watch、sandbox、更新和 security floor。
- `MANAGEMENT_READY` 只开放节点明确声明的操作。
- 每个降级项包含影响、不可用动作和安全恢复入口。

### `CONSOLE-V2-PAGE-016` 账号、登录会话与通知

- 目的：查看当前节点账号、session、通知权限和托盘行为。
- 其他本机用户的预置/启用通过 Admin CLI 完成，Windows v1 Console 不提供成员管理。
- 恢复密码深链到 Admin CLI 说明，不在 Console 内执行 recovery。
- 显示 Windows 通知权限、最后握手和测试通知；测试通知不包含敏感数据。

### `CONSOLE-V2-PAGE-017` 对象记录

- 目的：查看当前对象的 authority 来源、版本、状态变化和证据链接。
- v1 不是全局审计搜索/导出工具。
- 权限裁剪、数据缺失、延迟和签名无效使用不同完整性状态。

### `CONSOLE-V2-PAGE-018` 首个 Owner 领取

- 目的：使用安装器/Admin CLI 产生的 bootstrap bundle 创建首个节点 Owner。
- 必显：已验证的 endpoint key、目标 Windows SID、secret expiry 和账号归属。
- Console 不显示/保存已提交 secret；Service 从 IPC peer token 取得 SID。
- 并发领取失败、secret 过期/已使用、store/audit 不可写均 fail closed。

### `CONSOLE-V2-PAGE-019` 节点身份变化阻断

- 目的：在已固定 endpoint key 变化时阻断登录并解释恢复。
- 必显：旧/新身份、最后信任时间、Service/安装关联和可能原因。
- 无普通“继续”按钮；签名更新证据或 Admin CLI 重置信任后才能重新进入 `PAGE-002`。

## 4. 通用页面状态矩阵

| 状态 | 呈现 | 动作规则 | 可访问性 |
|---|---|---|---|
| initial-loading | 与真实布局匹配的静态 skeleton | 无假控件 | `aria-busy`，不反复宣告 |
| refreshing-last-good | 保留数据 + 更新时间/刷新标记 | 仅 freshness 允许的动作可用 | 宣告一次“正在刷新” |
| authoritative-empty | 说明 authority 返回空集 | 提供首要创建/导入动作 | 标题解释为空原因 |
| filtered-empty | 保留筛选条件 | 清除/修改筛选 | 不误称没有数据 |
| partial | 显示已加载范围和缺口 | 依赖缺失部分的写禁用 | 缺口可被读出 |
| redacted | 锁定占位 + 安全原因类别 | 请求权限（若支持） | 不泄露对象正文/名称 |
| stale-offline | `as_of` + 非实时横幅 | 所有写禁用 | 颜色非唯一 |
| permission-denied | 可安全披露的拒绝原因 | 返回/切换账号 | 不区分会泄露存在性的错误 |
| submitting | 固定 proposal ref | 禁止重复提交 | 状态 live region |
| result-unknown | 高注意安全状态 | 查询/reconcile，禁止 retry | 焦点进入解释而非危险按钮 |
| conflict/superseded | 差异与新版本 | 取新 preview | 旧控件不可继续激活 |
| success | authority ref + 下一步 | 查看对象/返回 | 不自动抢焦点 |
| service-error | error code/correlation（合同允许时） | 安全恢复动作 | 不显示原始敏感错误正文 |
| privacy-locked | 隐私遮罩；敏感 renderer 已 teardown | 解锁后重新认证 | 不在锁屏 surface 暴露标题/正文 |
| reauth-required | 非敏感框架 + 重新认证原因 | 登录并 resnapshot | 旧 projection 不恢复为当前状态 |

## 5. 旅程追踪矩阵

以下字段补充前述旅程的共享模板；所有“成功”均以 authority 结果为准。

| Journey | Persona | 关键依赖 | 恢复入口 | 观测/可访问性重点 |
|---|---|---|---|---|
| `JRN-001` | 首个 Owner | Service installer、bootstrap bundle、账号/session | `PAGE-001/018/019`、Admin CLI | 并发领取、UAC、纯键盘、secret 不朗读/不记录 |
| `JRN-002` | Agent 操作者 | 身份、snapshot/watch、最近 refs | `PAGE-003/015` | 登录限速、焦点到 H1、last-good 不冒充当前 |
| `JRN-003` | Agent 操作者 | Intent/preview/risk/Task/Effect | `PAGE-004/005/007` | 澄清、stale、R2 阻断、提交结果未知 |
| `JRN-004` | Agent 操作者 | supervision lease、pause/checkpoint | `PAGE-007/014` | pause pending、不可中断 Effect、Narrator 状态公告 |
| `JRN-005` | Agent 操作者 | tray、lease eligibility、notification broker | tray、`PAGE-014` | 有界等待、强退、崩溃、多实例/用户切换 |
| `JRN-006` | Agent 操作者/Owner | acquisition broker、source policy、scanner | `PAGE-009/010/013` | SSRF/UNC/path 权限、digest drift、unknown 语义 |
| `JRN-007` | Owner | installer/lifecycle transaction、rollback | `PAGE-010/011/012/008` | old/new diff、R2 阻断、失败保留旧版本 |
| `JRN-008` | Agent 操作者/Owner | canonical preview、R1 confirmation | `PAGE-005/011` | 初始焦点、Esc、number matching、倒计时 |
| `JRN-009` | 责任操作者 | reconciliation/verification | `PAGE-008/017` | 禁 retry、NOT_EXECUTED 终态、证据链接 |
| `JRN-010` | Agent 操作者/Owner | readiness、emergency capability/log | `PAGE-014/015/017` | fail closed、last-good、应急操作结果未知 |

### 5.1 触发、决定与结果

| Journey | 触发 | 关键用户决定 | 成功结果 | 失败/取消出口 |
|---|---|---|---|---|
| `JRN-001` | 首次启动且 Service/Owner 不存在 | 安装、信任 endpoint、领取 Owner | authority 返回 Owner session 与初始 snapshot | 取消安装；签名/secret/store 失败；identity-change |
| `JRN-002` | 已有本地账号启动 Console | 登录并选择恢复最近工作 | 授权 snapshot/watch 完成 | identity-change、登录失败、session 失效、readiness 降级 |
| `JRN-003` | 在 Shell 提交新目标 | 澄清、修改范围、确认 R0/R1 | 返回稳定 Task/AgentExecution 及适用对象 refs | 取消、stale、R2/R3、store/audit 失败、提交结果未知 |
| `JRN-004` | 活动任务需要介入 | 补充输入、请求暂停或恢复 | authority 确认输入/暂停/恢复 | 取消；pause pending 超时；不可中断外部影响 |
| `JRN-005` | 关闭窗口或显式退出 | 留在托盘或退出并请求暂停 | pause request accepted 后退出，最终状态后续收敛 | 取消退出；accepted 超时留在托盘；强退依赖 lease expiry |
| `JRN-006` | 用户输入 Agent 来源 | 是否开始受控 acquisition/检查 | 固定 acquisition/package/compatibility refs | 取消；SSRF/path/budget/digest/signature/gate 失败 |
| `JRN-007` | 对已检查包或 installation 选择生命周期动作 | 修改范围、确认 R0/R1 | 新 installation 版本或明确卸载结果 | 取消、R2/R3、事务失败、结果未知、回滚阻断 |
| `JRN-008` | authority 返回 R1 proposal | 取消、修改范围或确认 | authority 接受固定 proposal 决定 | 取消、过期、stale、challenge 重发、提交结果未知 |
| `JRN-009` | Effect 进入 `OUTCOME_UNKNOWN` | 发起/等待 reconcile | executed 后验证，或 `NOT_EXECUTED` 终态 | 取消查看；仍未知则 quarantine/安全处置 |
| `JRN-010` | store/audit/watch/readiness 降级 | 只读检查或符合门禁的遏制 | 恢复后按稳定 ref 对账并 resnapshot | 普通写拒绝；无预授权/日志则遏制也拒绝 |

## 6. 未来自动化验收

当前没有 Console 实现，以下是未来 Playwright/原生辅助技术场景，不是已执行测试：

- 首次设置并发 Owner claim；
- 登录失败限速和账号存在性保护；
- endpoint key/secret/SID 绑定和 identity-change 阻断；
- preview stale / R2 阻断；
- 关闭到托盘、显式退出、进程崩溃与 lease 到期；
- 锁屏、用户切换、session revoke、watch stale、UI hang 和多实例续租竞争；
- pause pending 与不可中断 Effect；
- acquisition SSRF/UNC/ambient credential/path access/resource budget、包 digest 变化、未知签名和 R2 风险；
- lifecycle 事务失败、rollback、outcome unknown；
- state store/audit/watch 故障注入；
- Narrator + 纯键盘完成创建任务、R1、暂停、对账和 Agent 更新；
- 200%/400% 缩放、high contrast、reduced motion、zh-CN/en 长文本。
