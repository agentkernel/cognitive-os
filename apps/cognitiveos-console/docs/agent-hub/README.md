# CognitiveOS Console — Agent Hub 文档地图

> 类别：informative product design（Lane-CON 激活前文档例外）
>
> canonical 状态：`planned / implementation not-implemented / platform tests none / Profile not implemented`
>
> 查询基准日：2026-07-20 ｜ 规范快照：273 REQ / 55 error codes / 56 schemas / 5 transitions / 76 vectors（全部 `not-run`）

本目录是 CognitiveOS Console **Agent Hub** 的 canonical 产品/架构/安全/研究/追踪文档根。Agent Hub 只存在两种部署模式：**Direct Takeover（直连接管）** 与 **CognitiveOS Governed（完整治理）**；不存在只有 `cognitive-kernel` 的中间模式。

本目录不新增或修改任何 CognitiveOS `REQ-*`、错误码、schema、transition table、conformance vector 或实现代码，也不表示 Console/Host/Adapter/Relay/Vault 实现已启动。所有产品 ID 使用 `CONSOLE-AGENTHUB-V1-*` 命名空间，属于 informative 产品要求，不进入 normative registry。

## 1. 单一事实来源与阅读顺序

新会话按以下顺序进入：

1. 本 README（文档地图、状态、canonical owner）；
2. [GOVERNANCE.md](./GOVERNANCE.md)（文档治理、状态用语、同步契约、迁移与 deprecated 规则）；
3. [decisions/decision-log.md](./decisions/decision-log.md)（已冻结产品决策 `CONSOLE-AGENTHUB-V1-DEC-*`）；
4. [product/deployment-modes-and-guarantees.md](./product/deployment-modes-and-guarantees.md)（两模式能力与保证矩阵）；
5. 具体专题文档；
6. [planning/README.md](./planning/README.md)（开发计划、DAG、进度、提示词入口）。

工程里程碑真相仍以全局 [docs/plan/PROGRESS.md](../../../../docs/plan/PROGRESS.md) 为准；本目录的 [progress.md](./progress.md) 只维护 Agent Hub 局部文档进度，不与全局 PROGRESS 平行演化。

## 2. 文档地图

### 2.1 治理与进度

| 文档 | canonical 负责内容 | owner |
|---|---|---|
| [GOVERNANCE.md](./GOVERNANCE.md) | 文档地图规则、状态四态、同步契约、迁移/弃用、ID 规则 | Lane-CON + Lane-DOC |
| [progress.md](./progress.md) | Agent Hub 局部文档进度（不替代全局 PROGRESS） | Lane-CON |
| [planning/README.md](./planning/README.md) | 开发计划/DAG/提示词入口索引 | Lane-CON |

### 2.2 产品（`product/`）

| 文档 | canonical 负责内容 |
|---|---|
| [product/product-design.md](./product/product-design.md) | 产品定位、用户、JTBD、原则、范围、成功指标、信息架构 |
| [product/deployment-modes-and-guarantees.md](./product/deployment-modes-and-guarantees.md) | 两部署模式逐能力保证矩阵、事实来源、接管层级、状态用语 |
| [product/journeys-and-screens.md](./product/journeys-and-screens.md) | PC/手机关键旅程、页面清单、接管预览、恢复 |
| [product/states-content-and-accessibility.md](./product/states-content-and-accessibility.md) | 页面状态模型、内容/术语、WCAG 2.2 与跨端无障碍验收 |

### 2.3 架构（`architecture/`）

| 文档 | canonical 负责内容 |
|---|---|
| [architecture/takeover-architecture.md](./architecture/takeover-architecture.md) | Takeover Host 拓扑、职责分离、接管层级 L1–L8、ownership generation、controller lease |
| [architecture/process-and-terminal.md](./architecture/process-and-terminal.md) | 进程发现/身份/监管/reaping、ConPTY/PTY/tmux、信号与停止语义 |
| [architecture/session-and-file-adoption.md](./architecture/session-and-file-adoption.md) | 官方 session adopt、native 文件只读发现、parser/snapshot/digest |
| [architecture/relay-pairing-and-migration.md](./architecture/relay-pairing-and-migration.md) | Relay/E2EE 配对、device identity、恢复、Direct→Governed 迁移 |

### 2.4 安全（`security/`）

| 文档 | canonical 负责内容 |
|---|---|
| [security/threat-model.md](./security/threat-model.md) | 威胁清单（asset/attacker/entry/boundary/prevention/detection/failure/recovery/owner/oracle/evidence） |
| [security/security-and-credentials.md](./security/security-and-credentials.md) | 本机控制面、账号/密钥分层、secret 存储与不同步、多账号切换 |
| [security/computer-control.md](./security/computer-control.md) | selected-window 桌面控制、隔离浏览器、PC-local 确认矩阵 |
| [security/licensing-and-terms.md](./security/licensing-and-terms.md) | 供应商条款/许可、AGPL 与 Paseo 复用 gate |

### 2.5 协作、平台、决策、追踪、Adapter、来源、模板

| 目录/文档 | canonical 负责内容 |
|---|---|
| [collaboration/lead-workers.md](./collaboration/lead-workers.md) | 多 Agent 模式比较与首版 Lead+Workers 约束 |
| [platforms/product-scope.md](./platforms/product-scope.md) | Windows/macOS/Linux/iPhone/Android 范围与 gate |
| [decisions/decision-log.md](./decisions/decision-log.md) | `CONSOLE-AGENTHUB-V1-DEC-*` 已冻结决策 |
| [traceability/product-requirements.md](./traceability/product-requirements.md) | `CONSOLE-AGENTHUB-V1-PRD-*` 三维状态与 blocked_by |
| [traceability/evidence-index.md](./traceability/evidence-index.md) | 证据索引（当前全部 none/not-run） |
| [adapters/README.md](./adapters/README.md) | Adapter 能力模型、逐能力矩阵、Tier 分级 |
| [adapters/capability-matrix.md](./adapters/capability-matrix.md) | 33 项能力 × Agent × 平台 × 版本矩阵 |
| [adapters/tier1/](./adapters/tier1/) | 六个 Tier 1 Agent dossier |
| [sources/](./sources/) | provider 接口、条款/许可、平台安全、Paseo/竞品来源 ledger |
| [templates/](./templates/) | Adapter dossier / source record / threat record / Open PoC / task 模板 |

## 3. 状态真相（固定声明）

- Agent Hub 产品方向：已记录为 `accepted product direction`。
- machine contract：Direct Takeover carrier 多数 `product-only / unregistered`；Governed 复用既有部分登记合同但关键 carrier 仍缺失。
- implementation：`not-implemented`。
- platform / PoC evidence：`none`。
- 既有 76 conformance vectors：`not-run`。
- Console/Host/Adapter/Relay/Vault/移动 Profile：`not implemented`。

文档存在、schema 存在、vector 被枚举或产品方向被批准，都不构成实现、测试或符合性证据。

## 4. 上游依赖入口

- Console v2 基线：[apps/cognitiveos-console/PRODUCT-DESIGN.md](../../PRODUCT-DESIGN.md)（§17/§20.3 anchor 保持不变）。
- 平台产品设计：[docs/platforms/README.md](../../../../docs/platforms/README.md)。
- Console 实现 gate：[docs/platforms/README.md#console-实现-gate](../../../../docs/platforms/README.md#console-实现-gate)。
- 开发计划入口：[docs/plan/agent-hub-development-plan.md](../../../../docs/plan/agent-hub-development-plan.md)。
- 车道机制：[docs/plan/PARALLEL-LANES.md](../../../../docs/plan/PARALLEL-LANES.md)。
