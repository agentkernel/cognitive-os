# Agent Hub — 部署模式与能力保证矩阵

> 类别：informative product design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：`accepted product direction / implementation not-implemented / evidence none / Profile not implemented`

本文件是 Agent Hub 两部署模式的 canonical 保证矩阵。产品只存在两种部署模式，不存在只有 `cognitive-kernel` 的中间模式，也不得设计、保留或暗示该模式。

## 1. 两种部署模式

### 1.1 Direct Takeover（直连接管）

- 前提：用户没有安装 CognitiveOS。
- PC Takeover Host 是本地确定性控制层，只对自己启动、包装、附着和记录的本地进程及文件负责。
- Host **不是** CognitiveOS node、authority，也不构成 CognitiveOS Profile。
- 统一工作项是 product-only 对象，不得冒充已登记 Task，不得暗示具备 CognitiveOS 的 CAS、fencing、Effect、授权、审计、Verification 或 Acceptance 保证。

### 1.2 CognitiveOS Governed（完整治理）

- 前提：存在完整 CognitiveOS。
- 使用真实 authority projection；Task、Loop、AgentExecution、Effect、Verification 各自独立；Runtime 是独立 projection。
- 第三方 Agent 作为受治理 Adapter 接入，不能绕过确定性入口。
- 保留授权、预算、幂等、risk floor、Verification 和 Acceptance；不因兼容成熟 Agent 而降低 R0/R1/R2/R3。

### 1.3 连接状态不是第三模式

`检测中 / 不可达 / 降级 / 未知` 只能是连接状态；不得成为第三种部署模式。每个 Work 固定 mode、Host、账号、工作区和事实来源，禁止静默切换。检测到 binary、crate 或本机进程不足以进入 Governed；进入 Governed 需认证 endpoint、协议/schema pin、authority refs、session/capability 与 snapshot/watch handshake。

## 2. 事实来源（source of truth）

| 标签 | 含义 | 可推进什么 |
|---|---|---|
| `CognitiveOS authority projection` | Governed authority 状态/事件/Effect/Verification/Acceptance | Governed 模式唯一可推进 CognitiveOS 状态的来源 |
| `Takeover Host ledger`（`host-managed`） | Host 接受的请求、发出的信号、观察结果、用户决定 | 仅 Direct 产品级 WorkItem 记录，不是 authority audit |
| `provider-reported` | 供应商官方控制面返回的状态 | 显示；不单独构成完成 |
| `process-observed` | OS 进程观察 | 显示进程存在/健康；不证明完成 |
| `terminal-observed` | 受管终端 capture | 不可信输入；不证明权限或完成 |
| `file-observed` | 版本化只读文件 snapshot | 观察到磁盘变化；不证明完成 |
| `check-observed` | 固定检查/verifier 结果 | 记录命令/版本/范围/退出；单独不等于用户接受 |
| `user-accepted` | 用户关闭工作项 | Direct 产品级处置；不是 CognitiveOS Acceptance |
| `unknown` | 结果未知 | 禁止盲重试；进入对账 |

规则：`Agent claim`、进程退出、terminal capture、文件变化、通知、Relay receipt 均不能自动成为 CognitiveOS Verification、Acceptance 或 `COMPLETED`。

## 3. 接管层级（`CONSOLE-AGENTHUB-V1-LVL-*`）

按优先级从高到低设计；详细安全条件见 [takeover-architecture.md](../architecture/takeover-architecture.md)。

| Level | 名称 | v1 处置 | 结果标签 |
|---|---|---|---|
| L1 | Official Control（SDK/App Server/Gateway/ACP/REST/SSE/JSON-RPC/headless/官方 session API） | 允许 | 依动作 |
| L2 | Host Launched（Host 从任务开始启动并监管） | 允许，**默认** | `managed-from-start` |
| L3 | Official Session Adopted（官方 list/import/resume/fork） | 允许，仅旧 writer inactive 或供应商 exclusive lease/fencing 时可写 | `officially-adopted` |
| L4 | Managed Terminal Attached（仅 Host 创建的 ConPTY；未来独立 socket 的 tmux/screen） | 允许，仅 Host-owned | `terminal-attached` |
| L5 | Read-only File Observed（版本化只读 session/log/config metadata） | 允许，opt-in、documented root、敏感字段裁剪 | `read-only-observed` |
| L6 | Documented File Write（供应商明确支持外部写入并提供版本/并发/CAS/rollback/migration） | **v1 阻断**，仅保留为未来条件能力 | `blocked-by-policy` |
| L7 | Observe-only Process（任意既有 PID 只观察） | 允许；仅 PC-local 精确 PID 的独立 emergency containment 可发终止，仍不视为接管成功 | `unmanaged-observed` |
| L8 | Forbidden（内存注入/DLL 注入/调试器劫持/二进制 patch/任意 stdin 抢占/credential 篡改/内部 DB 伪造/token/cookie/keychain 抽取/绕过登录计费安全组织策略） | 永久禁止 | `blocked-by-policy` |

## 4. 两模式能力与保证矩阵

每项逐列记录：用户能力 / 事实来源 / 控制主体 / 凭据主体 / 持久化主体 / 完成主体 / contract / implementation / evidence / blocked_by。

### 4.1 Direct Takeover 矩阵

| 能力 | 事实来源 | 控制主体 | 凭据主体 | 持久化主体 | 完成主体 | contract | implementation | evidence | blocked_by |
|---|---|---|---|---|---|---|---|---|---|
| 发现已安装 Agent | `process-observed` / `provider-reported` | Takeover Host | provider 原生登录 | Host ledger | n/a | product-only | not-implemented | none | Host/Adapter 实现 |
| Host 启动并监管新 Agent（L2） | `host-managed` | Takeover Host | OS secure store / broker | Host ledger | `user-accepted` + `check-observed` | product-only | not-implemented | none | Process Supervisor |
| 官方 session adopt（L3） | `provider-reported` | provider 官方控制面 | provider 原生 | provider 原生 + Host ledger | `user-accepted` | product-only（`partial: REQ-AGENT-*` 仅 Governed） | not-implemented | none | Session Adopter；exclusive-lease 证明 |
| 受管终端 attach（L4） | `terminal-observed` | Terminal Broker | n/a | Host ledger | n/a | product-only | not-implemented | none | Terminal Broker |
| 只读文件观察（L5） | `file-observed` | File Observer | n/a | Host ledger（digest/parser） | n/a | product-only | not-implemented | none | File Observer |
| 普通 PID 观察（L7） | `process-observed` | Host（只读） | n/a | Host ledger | n/a | product-only | not-implemented | none | — |
| PC-local 精确 PID 紧急终止 | `process-observed` + `unknown` | Host（本机确认） | n/a | Host ledger | `unknown` | product-only | not-implemented | none | emergency containment 设计 |
| 创建/派发/监督/纠偏 WorkItem | `host-managed` | Host + Multi-Agent scheduler | — | Host ledger | `user-accepted` | product-only | not-implemented | none | scheduler |
| 群组 Lead+Workers（单 Host/一层） | `host-managed` | 确定性调度器 | — | Host ledger | `user-accepted` | product-only | not-implemented | none | worktree/lock |
| 统一模型/账号/API key 视图 | `host-managed` | Credential Broker | OS secure store / broker | Host ledger（opaque handle） | n/a | product-only | not-implemented | none | Credential Broker |
| 手机远程监督/请求扩权 | `host-managed`（经 Relay 投影） | Host（PC-local 批准扩权） | 不下发 secret | Host ledger | `user-accepted` | product-only | not-implemented | none | Relay/Pairing |
| 完成判定 | `check-observed` + `user-accepted` | 用户 | — | Host ledger | 双轴：checks 记录 + 用户接受 | product-only | not-implemented | none | verifier |

Direct 明确不保证：CognitiveOS CAS、fencing、Effect、Verification、authority audit、Acceptance、Profile。

### 4.2 CognitiveOS Governed 矩阵

| 能力 | 事实来源 | 控制主体 | 凭据主体 | 持久化主体 | 完成主体 | contract | implementation | evidence | blocked_by |
|---|---|---|---|---|---|---|---|---|---|
| authority Task/五生命周期 | `CognitiveOS authority projection` | 确定性 kernel/runtime authorities | AuthenticationSession/Capability | authority object/event store | acceptance authority | partial（关键 carrier 缺失） | not-implemented | not-run | M2/M4/M5 |
| proposal/preview/R0-R1 | authority projection | management authority | AuthenticationSession + PrivilegedManagementSession | authority store/audit | acceptance authority | partial | not-implemented | not-run | F-011（M5）、signed proposal |
| Effect/reconcile/幂等 | authority projection | 确定性代码 | Capability | Effect store | Verification + acceptance | partial | not-implemented | not-run | M4 |
| Verification/Acceptance | authority projection | verification/acceptance authority | — | authority store | acceptance authority | partial | not-implemented | not-run | M4/M5 |
| 第三方 Agent 受治理 Adapter | authority projection | Adapter + authority gate | Capability | authority store | acceptance authority | partial（`REQ-AGENT-*`） | not-implemented | not-run | M6 |
| R2/R3 | authority projection | 可信确认面 | — | authority store | acceptance authority | missing | not-implemented | none | Phase F |

## 5. Direct 完成语言

至少分开显示，不得合并或直接显示 “Verified / CognitiveOS completed”：

- `Agent reported done`
- `Process exited`
- `Checks observed pass/fail`
- `User accepted`
- `Result unknown`

## 6. 模式迁移

Direct 历史升级到 Governed 只能是 evidence-only import：Governed authority 新建 UserIntentRecord/TaskContract/Effect 等对象；旧 Host ledger/artifact 只作带 digest 的外部证据导入，保留来源与历史，不追认历史 authority/Verification。禁止把 Host ledger 改写成 authority Event。详见 [relay-pairing-and-migration.md](../architecture/relay-pairing-and-migration.md)。
